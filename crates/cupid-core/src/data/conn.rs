use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use postgres::config::{Host, SslMode};
use postgres::{Client, Config};
use rustls::{ClientConfig, RootCertStore};
use serde::Serialize;
use tokio_postgres_rustls::MakeRustlsConnect;

/// Supabase serves a chain rooted in its own private CA, and the leaf is
/// valid for five years, past Apple's 398-day cap for TLS server certs. The
/// platform verifiers therefore reject it; pinning the root and verifying
/// with rustls behaves identically on every platform.
const SUPABASE_ROOT_CA: &[u8] = include_bytes!("../../assets/supabase-prod-ca-2021.crt");

/// Pooler host prefixes Supabase assigns. A project sits on exactly one and
/// only the dashboard says which, so both are tried in turn.
const POOLER_PREFIXES: [&str; 2] = ["aws-1", "aws-0"];

/// Session pooler first, then the transaction pooler: some networks (NUS
/// among them) silently drop outbound 5432 but pass 6543.
const POOLER_PORTS: [u16; 2] = [5432, 6543];

/// Firewalls drop blocked ports without replying, and unbounded the OS waits
/// out its SYN retries (about two minutes on Linux) before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Reaching the wrong pooler prefix produces one of these (the session and
/// transaction poolers word it differently); it means "try the other one",
/// not "the credentials are wrong".
const WRONG_TENANT: [&str; 2] = ["Tenant or user not found", "tenant/user"];

fn is_wrong_tenant(message: &str) -> bool {
    WRONG_TENANT.iter().any(|m| message.contains(m))
}

/// The failure to report when every candidate fails: the one that got
/// furthest, since a sign-in verdict on one port outranks another port that
/// merely timed out. A wrong-prefix rejection ranks below any other sign-in
/// failure, as it says nothing about the credentials.
fn most_relevant(errors: &[(Stage, String)]) -> Option<&str> {
    errors
        .iter()
        .max_by_key(|(stage, message)| (*stage, !is_wrong_tenant(message)))
        .map(|(_, message)| message.as_str())
}

/// Resolver failure text, seen when the direct host has no address the
/// network can reach.
const NO_ADDRESS: &str = "failed to lookup address information";

/// Postgres errors render as a bare "db error"; everything an operator needs
/// sits in the source chain, so flatten it.
fn full_message(error: &(dyn Error + 'static)) -> String {
    let mut parts = vec![error.to_string()];
    let mut source = error.source();
    while let Some(inner) = source {
        parts.push(inner.to_string());
        source = inner.source();
    }
    parts.join(": ")
}

/// A verifying TLS connector that trusts the public web roots plus Supabase's
/// own root, so both hosted projects and arbitrary Postgres URLs work.
fn tls_connector() -> Result<MakeRustlsConnect, Box<dyn Error>> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    for cert in rustls_pemfile::certs(&mut &SUPABASE_ROOT_CA[..]) {
        roots.add(cert?)?;
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(MakeRustlsConnect::new(config))
}

/// The layers a connection attempt passes through, reported in order so an
/// operator can see which one a hostile network breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Dns,
    Tcp,
    SignIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StepStatus {
    Running,
    Ok,
    Failed,
    Skipped,
}

/// Progress of [`ConnSpec::connect_traced`]: a `Target` opens each candidate
/// endpoint, and the `Step`s that follow belong to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ConnectEvent {
    Target {
        host: String,
        port: u16,
    },
    Step {
        stage: Stage,
        status: StepStatus,
        detail: Option<String>,
    },
}

fn step(stage: Stage, status: StepStatus, detail: Option<String>) -> ConnectEvent {
    ConnectEvent::Step {
        stage,
        status,
        detail,
    }
}

/// Explain a failed TCP connect. The flag is true when the port itself looks
/// blocked, so later candidates on the same port can be skipped instead of
/// each waiting out the timeout.
fn tcp_failure(addr: SocketAddr, error: &io::Error) -> (String, bool) {
    let port = addr.port();
    match error.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => (
            format!(
                "no reply from {addr} within {}s: this network is likely dropping port {port}",
                CONNECT_TIMEOUT.as_secs()
            ),
            true,
        ),
        io::ErrorKind::ConnectionRefused => (format!("{addr} refused the connection"), false),
        io::ErrorKind::NetworkUnreachable | io::ErrorKind::HostUnreachable if addr.is_ipv6() => (
            format!("no route to {addr}: this network has no IPv6"),
            false,
        ),
        io::ErrorKind::NetworkUnreachable | io::ErrorKind::HostUnreachable => {
            (format!("no route to {addr}"), false)
        }
        _ => (format!("{addr}: {error}"), false),
    }
}

/// How to reach the database, supplied at runtime rather than baked into the
/// environment. `Url` carries a full libpq connection string; `Supabase`
/// derives every connection parameter from the project credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnSpec {
    Url(String),
    Supabase {
        project_ref: String,
        password: String,
        region: Option<String>,
        /// The pooler port that last answered. Once known, later connects go
        /// straight to it rather than timing out on a blocked port again.
        port: Option<u16>,
    },
}

impl ConnSpec {
    /// Build a Supabase spec, trimming inputs. An empty region collapses to
    /// `None` (direct connection).
    pub fn supabase(project_ref: &str, password: &str, region: Option<&str>) -> Self {
        ConnSpec::Supabase {
            project_ref: project_ref.trim().to_string(),
            // Passwords may legitimately start or end with whitespace.
            password: password.to_string(),
            region: region
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .map(String::from),
            port: None,
        }
    }

    /// This spec restricted to the pooler port a connect succeeded on.
    pub fn pinned_to(self, port: u16) -> Self {
        match self {
            ConnSpec::Supabase {
                project_ref,
                password,
                region: Some(region),
                ..
            } => ConnSpec::Supabase {
                project_ref,
                password,
                region: Some(region),
                port: Some(port),
            },
            other => other,
        }
    }

    /// The read-only postgres client configurations this spec describes, in
    /// the order they should be attempted. Every session carries
    /// `default_transaction_read_only=on`: cupid only ever reads, except for
    /// the explicit end-of-cycle purge (see [`ConnSpec::configs_read_write`]).
    pub fn configs(&self) -> Result<Vec<Config>, Box<dyn Error>> {
        let mut configs = self.configs_read_write()?;
        for config in &mut configs {
            // A Url spec may carry its own options: append, don't replace.
            let options = match config.get_options() {
                Some(existing) if !existing.is_empty() => {
                    format!("{existing} -c default_transaction_read_only=on")
                }
                _ => "-c default_transaction_read_only=on".to_string(),
            };
            config.options(&options);
        }
        Ok(configs)
    }

    /// Client configurations without the read-only guard. Purge-only.
    ///
    /// Supabase without region: direct host `db.<ref>.supabase.co:5432`,
    /// user `postgres`. With region: pooler
    /// `aws-<n>-<region>.pooler.supabase.com`, user `postgres.<ref>` (the
    /// pooler multiplexes projects, so the project travels in the user). One
    /// config per pooler prefix, since the project's prefix is unknown, and
    /// per port, since networks differ in which ones they pass.
    pub fn configs_read_write(&self) -> Result<Vec<Config>, Box<dyn Error>> {
        match self {
            ConnSpec::Url(url) => Ok(vec![url.parse::<Config>()?]),
            ConnSpec::Supabase {
                project_ref,
                password,
                region,
                port,
            } => {
                if project_ref.is_empty() {
                    return Err("Supabase project ref must not be empty".into());
                }
                let ports: Vec<u16> = match port {
                    Some(port) => vec![*port],
                    None => POOLER_PORTS.to_vec(),
                };
                let endpoints: Vec<(String, u16, String)> = match region {
                    Some(region) => POOLER_PREFIXES
                        .iter()
                        .flat_map(|prefix| {
                            ports.iter().map(move |port| {
                                (
                                    format!("{prefix}-{region}.pooler.supabase.com"),
                                    *port,
                                    format!("postgres.{project_ref}"),
                                )
                            })
                        })
                        .collect(),
                    None => vec![(
                        format!("db.{project_ref}.supabase.co"),
                        5432,
                        "postgres".to_string(),
                    )],
                };
                Ok(endpoints
                    .into_iter()
                    .map(|(host, port, user)| {
                        let mut config = Config::new();
                        config
                            .host(&host)
                            .user(&user)
                            .port(port)
                            .dbname("postgres")
                            .password(password)
                            .ssl_mode(SslMode::Require);
                        config
                    })
                    .collect())
            }
        }
    }

    /// Open a read-only TLS connection to the database this spec points at,
    /// trying each candidate host until one answers.
    pub fn connect(&self) -> Result<Client, Box<dyn Error>> {
        Ok(self.connect_with(self.configs()?, &mut |_| {})?.0)
    }

    /// [`ConnSpec::connect`], reporting each candidate's DNS, TCP and sign-in
    /// steps as they happen. Also returns the port that answered, for
    /// [`ConnSpec::pinned_to`].
    pub fn connect_traced(
        &self,
        report: &mut dyn FnMut(ConnectEvent),
    ) -> Result<(Client, u16), Box<dyn Error>> {
        self.connect_with(self.configs()?, report)
    }

    /// Open a writable connection. The end-of-cycle purge is the only caller;
    /// everything else must stay on the read-only [`ConnSpec::connect`].
    pub fn connect_read_write(&self) -> Result<Client, Box<dyn Error>> {
        Ok(self
            .connect_with(self.configs_read_write()?, &mut |_| {})?
            .0)
    }

    fn connect_with(
        &self,
        configs: Vec<Config>,
        report: &mut dyn FnMut(ConnectEvent),
    ) -> Result<(Client, u16), Box<dyn Error>> {
        let tls = tls_connector()?;
        let mut errors: Vec<(Stage, String)> = Vec::new();
        let mut blocked_ports: HashSet<u16> = HashSet::new();
        for mut config in configs {
            if config.get_connect_timeout().is_none() {
                config.connect_timeout(CONNECT_TIMEOUT);
            }
            let port = config.get_ports().first().copied().unwrap_or(5432);
            // Unix sockets have no DNS or TCP layer to probe.
            if let Some(Host::Tcp(host)) = config.get_hosts().first() {
                report(ConnectEvent::Target {
                    host: host.clone(),
                    port,
                });
                if let Err(error) = self.probe(host, port, &mut blocked_ports, report) {
                    errors.push(error);
                    continue;
                }
            }
            report(step(Stage::SignIn, StepStatus::Running, None));
            match config.connect(tls.clone()) {
                Ok(client) => {
                    report(step(
                        Stage::SignIn,
                        StepStatus::Ok,
                        config.get_user().map(String::from),
                    ));
                    return Ok((client, port));
                }
                Err(e) => {
                    let message = full_message(&e);
                    let detail = if is_wrong_tenant(&message) {
                        "project is not on this pooler".to_string()
                    } else {
                        message.clone()
                    };
                    report(step(Stage::SignIn, StepStatus::Failed, Some(detail)));
                    errors.push((Stage::SignIn, message));
                }
            }
        }
        let reported = most_relevant(&errors).unwrap_or("no connection candidates");
        Err(self.explain(reported.to_string()).into())
    }

    /// Resolve `host` and open a bare TCP connection to it, so a failure is
    /// pinned to the layer that broke instead of surfacing as one opaque
    /// postgres error. The postgres client redoes both; this only diagnoses.
    fn probe(
        &self,
        host: &str,
        port: u16,
        blocked_ports: &mut HashSet<u16>,
        report: &mut dyn FnMut(ConnectEvent),
    ) -> Result<(), (Stage, String)> {
        report(step(Stage::Dns, StepStatus::Running, None));
        let addrs: Vec<SocketAddr> = match (host, port).to_socket_addrs() {
            Ok(addrs) => addrs.collect(),
            Err(e) => {
                let message = self.explain(e.to_string());
                report(step(Stage::Dns, StepStatus::Failed, Some(message.clone())));
                return Err((Stage::Dns, message));
            }
        };
        if addrs.is_empty() {
            let message = format!("{host} has no addresses");
            report(step(Stage::Dns, StepStatus::Failed, Some(message.clone())));
            return Err((Stage::Dns, message));
        }
        let listed: Vec<String> = addrs.iter().map(|a| a.ip().to_string()).collect();
        report(step(Stage::Dns, StepStatus::Ok, Some(listed.join(", "))));

        if blocked_ports.contains(&port) {
            let message = format!("port {port} already timed out on this network");
            report(step(Stage::Tcp, StepStatus::Skipped, Some(message.clone())));
            return Err((Stage::Tcp, message));
        }
        report(step(Stage::Tcp, StepStatus::Running, None));
        let mut failure = String::new();
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, CONNECT_TIMEOUT) {
                Ok(_) => {
                    report(step(Stage::Tcp, StepStatus::Ok, Some(addr.to_string())));
                    return Ok(());
                }
                Err(e) => {
                    let (message, port_blocked) = tcp_failure(*addr, &e);
                    failure = message;
                    // A firewall drop hits every address alike; waiting out
                    // the timeout on each would only multiply the delay.
                    if port_blocked {
                        blocked_ports.insert(port);
                        break;
                    }
                }
            }
        }
        report(step(Stage::Tcp, StepStatus::Failed, Some(failure.clone())));
        Err((Stage::Tcp, failure))
    }

    /// Attach guidance to failures whose raw text hides the actual cause.
    fn explain(&self, error: String) -> String {
        if is_wrong_tenant(&error) {
            return format!(
                "{error}: no pooler in this region knows the project; \
                 check the project ref and region"
            );
        }
        let direct = matches!(self, ConnSpec::Supabase { region: None, .. });
        if direct && error.contains(NO_ADDRESS) {
            // Supabase dropped IPv4 from direct connections, so the host
            // publishes AAAA records only.
            return format!(
                "{error} — the direct host resolves over IPv6 only; \
                 supply the project's region to use the session pooler"
            );
        }
        error
    }

    /// Operator-facing description of the target. Never includes the password.
    pub fn describe(&self) -> String {
        match self {
            ConnSpec::Supabase {
                project_ref,
                region: Some(region),
                port: Some(port),
                ..
            } => format!("Supabase {project_ref} via {region} pooler, port {port}"),
            ConnSpec::Supabase {
                project_ref,
                region: Some(region),
                ..
            } => format!("Supabase {project_ref} via {region} pooler"),
            ConnSpec::Supabase { project_ref, .. } => {
                format!("Supabase {project_ref} (direct, IPv6 only)")
            }
            ConnSpec::Url(url) => match url.parse::<Config>() {
                Ok(config) => {
                    // Host::Unix is #[cfg(unix)] in tokio-postgres, so the arm
                    // cannot be named at all on Windows.
                    let host = match config.get_hosts().first() {
                        Some(Host::Tcp(host)) => host.clone(),
                        #[cfg(unix)]
                        Some(Host::Unix(path)) => path.display().to_string(),
                        None => "unknown host".to_string(),
                    };
                    let dbname = config.get_dbname().unwrap_or("postgres");
                    format!("{host}/{dbname}")
                }
                Err(_) => "custom connection string".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postgres::config::{Host, SslMode};

    fn tcp_host(config: &Config) -> &str {
        match &config.get_hosts()[0] {
            Host::Tcp(host) => host,
            other => panic!("expected tcp host, got {other:?}"),
        }
    }

    fn only(spec: &ConnSpec) -> Config {
        let mut configs = spec.configs().unwrap();
        assert_eq!(configs.len(), 1, "expected a single candidate");
        configs.remove(0)
    }

    #[test]
    fn supabase_without_region_uses_direct_host() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "hunter2", None);
        let config = only(&spec);
        assert_eq!(tcp_host(&config), "db.abcdefghijklmnopqrst.supabase.co");
        assert_eq!(config.get_ports(), &[5432]);
        assert_eq!(config.get_user(), Some("postgres"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        assert_eq!(config.get_password(), Some("hunter2".as_bytes()));
    }

    fn endpoints(spec: &ConnSpec) -> Vec<(String, u16)> {
        spec.configs()
            .unwrap()
            .iter()
            .map(|c| (tcp_host(c).to_string(), c.get_ports()[0]))
            .collect()
    }

    #[test]
    fn supabase_with_region_tries_every_pooler_prefix_and_port() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", Some("ap-southeast-1"));
        let aws1 = "aws-1-ap-southeast-1.pooler.supabase.com".to_string();
        let aws0 = "aws-0-ap-southeast-1.pooler.supabase.com".to_string();
        assert_eq!(
            endpoints(&spec),
            vec![
                (aws1.clone(), 5432),
                (aws1, 6543),
                (aws0.clone(), 5432),
                (aws0, 6543),
            ],
            "a project sits on one prefix, and some networks pass only 6543"
        );
        for config in &spec.configs().unwrap() {
            assert_eq!(
                config.get_user(),
                Some("postgres.abcdefghijklmnopqrst"),
                "pooler identifies the project via the user"
            );
            assert_eq!(config.get_dbname(), Some("postgres"));
        }
    }

    #[test]
    fn pinned_pooler_tries_only_the_port_that_answered() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", Some("ap-southeast-1"))
            .pinned_to(6543);
        let ports: Vec<u16> = endpoints(&spec).into_iter().map(|(_, p)| p).collect();
        assert_eq!(ports, vec![6543, 6543]);
        assert!(spec.describe().contains("port 6543"), "{}", spec.describe());
    }

    #[test]
    fn pinning_leaves_the_direct_host_alone() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        assert_eq!(spec.clone().pinned_to(6543), spec);
    }

    fn traced(spec: &ConnSpec) -> (Vec<ConnectEvent>, Result<(), String>) {
        let mut events = Vec::new();
        let result = spec
            .connect_traced(&mut |e| events.push(e))
            .map(|_| ())
            .map_err(|e| e.to_string());
        (events, result)
    }

    fn statuses(events: &[ConnectEvent]) -> Vec<(Stage, StepStatus)> {
        events
            .iter()
            .filter_map(|e| match e {
                ConnectEvent::Step { stage, status, .. } => Some((*stage, *status)),
                ConnectEvent::Target { .. } => None,
            })
            .collect()
    }

    #[test]
    fn trace_stops_at_dns_when_the_host_does_not_resolve() {
        let spec = ConnSpec::Url("host=cupid-test.invalid port=5432 user=u".into());
        let (events, result) = traced(&spec);
        assert!(result.is_err());
        assert_eq!(
            events[0],
            ConnectEvent::Target {
                host: "cupid-test.invalid".into(),
                port: 5432
            }
        );
        assert_eq!(
            statuses(&events),
            vec![
                (Stage::Dns, StepStatus::Running),
                (Stage::Dns, StepStatus::Failed)
            ]
        );
    }

    #[test]
    fn trace_stops_at_tcp_when_the_port_refuses() {
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let spec = ConnSpec::Url(format!("host=127.0.0.1 port={port} user=u"));
        let (events, result) = traced(&spec);
        assert!(result.is_err());
        assert_eq!(
            statuses(&events),
            vec![
                (Stage::Dns, StepStatus::Running),
                (Stage::Dns, StepStatus::Ok),
                (Stage::Tcp, StepStatus::Running),
                (Stage::Tcp, StepStatus::Failed),
            ]
        );
        let ConnectEvent::Step { detail, .. } = events.last().unwrap() else {
            panic!("last event is a step");
        };
        assert!(detail.as_deref().unwrap().contains("refused"), "{detail:?}");
    }

    #[test]
    fn trace_reaches_sign_in_once_tcp_connects() {
        // Accepts and hangs up: TCP succeeds, the postgres handshake cannot.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                drop(stream);
            }
        });
        let spec = ConnSpec::Url(format!("host=127.0.0.1 port={port} user=u"));
        let (events, result) = traced(&spec);
        assert!(result.is_err());
        assert_eq!(
            statuses(&events),
            vec![
                (Stage::Dns, StepStatus::Running),
                (Stage::Dns, StepStatus::Ok),
                (Stage::Tcp, StepStatus::Running),
                (Stage::Tcp, StepStatus::Ok),
                (Stage::SignIn, StepStatus::Running),
                (Stage::SignIn, StepStatus::Failed),
            ]
        );
    }

    #[test]
    fn a_timeout_marks_the_port_blocked() {
        let addr: SocketAddr = "13.213.241.248:5432".parse().unwrap();
        let (message, blocked) = tcp_failure(addr, &io::Error::from(io::ErrorKind::TimedOut));
        assert!(blocked);
        assert!(message.contains("port 5432"), "{message}");

        let (_, blocked) = tcp_failure(addr, &io::Error::from(io::ErrorKind::ConnectionRefused));
        assert!(
            !blocked,
            "a refusal is the server's answer, not a firewall drop"
        );
    }

    #[test]
    fn unreachable_ipv6_names_the_missing_ipv6() {
        let addr: SocketAddr = "[2406:da18::1]:5432".parse().unwrap();
        let (message, _) = tcp_failure(addr, &io::Error::from(io::ErrorKind::NetworkUnreachable));
        assert!(message.contains("no IPv6"), "{message}");
    }

    #[test]
    fn events_serialize_for_the_ui() {
        let json = serde_json::to_value(step(Stage::SignIn, StepStatus::Ok, None)).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"kind": "step", "stage": "signIn", "status": "ok", "detail": null})
        );
    }

    #[test]
    fn reported_failure_is_the_furthest_one_reached() {
        let timeout = (Stage::Tcp, "no reply on port 5432".to_string());
        let tenant = (
            Stage::SignIn,
            "FATAL: (ENOTFOUND) tenant/user postgres.x not found".to_string(),
        );
        let password = (Stage::SignIn, "password authentication failed".to_string());
        assert_eq!(
            most_relevant(&[timeout.clone(), password.clone(), tenant.clone()]),
            Some("password authentication failed"),
            "a credential verdict outranks a blocked port and a wrong prefix"
        );
        assert_eq!(
            most_relevant(&[timeout.clone(), tenant.clone()]),
            Some(tenant.1.as_str()),
            "a pooler that answered outranks a port that never did"
        );
        assert_eq!(
            most_relevant(std::slice::from_ref(&timeout)),
            Some(timeout.1.as_str())
        );
    }

    #[test]
    fn both_poolers_wordings_of_a_wrong_tenant_are_recognised() {
        assert!(is_wrong_tenant("FATAL: Tenant or user not found"));
        assert!(is_wrong_tenant(
            "FATAL: (ENOTFOUND) tenant/user postgres.x not found"
        ));
        assert!(!is_wrong_tenant("password authentication failed"));
    }

    #[test]
    fn supabase_requires_tls() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        assert_eq!(only(&spec).get_ssl_mode(), SslMode::Require);
    }

    #[test]
    fn sessions_default_to_read_only() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        assert!(
            only(&spec)
                .get_options()
                .unwrap_or("")
                .contains("default_transaction_read_only=on"),
            "every default session must refuse writes"
        );
    }

    #[test]
    fn url_specs_get_the_read_only_guard_too() {
        let spec = ConnSpec::Url("host=h user=u".into());
        let config = spec.configs().unwrap().remove(0);
        assert!(
            config
                .get_options()
                .unwrap_or("")
                .contains("default_transaction_read_only=on")
        );
    }

    #[test]
    fn read_write_configs_omit_the_read_only_guard() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        let config = spec.configs_read_write().unwrap().remove(0);
        assert!(
            !config
                .get_options()
                .unwrap_or("")
                .contains("default_transaction_read_only"),
            "purge must be able to write"
        );
    }

    #[test]
    fn supabase_inputs_are_trimmed_and_empty_region_is_direct() {
        let spec = ConnSpec::supabase("  abcdefghijklmnopqrst ", "pw", Some("   "));
        assert_eq!(
            tcp_host(&only(&spec)),
            "db.abcdefghijklmnopqrst.supabase.co"
        );
    }

    #[test]
    fn direct_resolver_failure_points_at_the_pooler() {
        let direct = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        let explained = direct.explain(format!("error connecting: {NO_ADDRESS}"));
        assert!(explained.contains("IPv6"), "{explained}");
        assert!(explained.contains("region"), "{explained}");

        // Pooled targets resolve fine, so the hint would only mislead.
        let pooled = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", Some("eu-west-2"));
        let untouched = pooled.explain(format!("error connecting: {NO_ADDRESS}"));
        assert!(!untouched.contains("IPv6"), "{untouched}");
    }

    #[test]
    fn full_message_unwraps_the_source_chain() {
        // `postgres::Error` prints as "db error"; the cause carries the text.
        #[derive(Debug)]
        struct Inner;
        impl std::fmt::Display for Inner {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "password authentication failed")
            }
        }
        impl Error for Inner {}

        #[derive(Debug)]
        struct Outer;
        impl std::fmt::Display for Outer {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "db error")
            }
        }
        impl Error for Outer {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                Some(&Inner)
            }
        }

        assert_eq!(
            full_message(&Outer),
            "db error: password authentication failed"
        );
    }

    #[test]
    fn tls_connector_trusts_public_roots_and_supabase() {
        // Supabase's private root must load alongside the webpki set, or
        // every hosted project fails the handshake.
        tls_connector().expect("connector builds");
    }

    #[test]
    fn supabase_password_is_taken_verbatim() {
        // Config carries the password out-of-band, so URL-hostile characters
        // must survive untouched.
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "p@ss:w/rd%25 #", None);
        assert_eq!(
            only(&spec).get_password(),
            Some("p@ss:w/rd%25 #".as_bytes())
        );
    }

    #[test]
    fn empty_project_ref_is_an_error() {
        let spec = ConnSpec::supabase("   ", "pw", None);
        let err = spec.configs().unwrap_err().to_string();
        assert!(err.contains("project ref"), "error names the field: {err}");
    }

    #[test]
    fn url_spec_parses_connection_string() {
        let spec = ConnSpec::Url("postgres://scott:tiger@example.com:5433/mydb".into());
        let config = only(&spec);
        assert_eq!(tcp_host(&config), "example.com");
        assert_eq!(config.get_ports(), &[5433]);
        assert_eq!(config.get_user(), Some("scott"));
        assert_eq!(config.get_dbname(), Some("mydb"));
    }

    #[test]
    fn invalid_url_is_an_error() {
        let spec = ConnSpec::Url("not a connection string %%%".into());
        assert!(spec.configs().is_err());
    }

    #[test]
    fn describe_names_target_without_password() {
        let direct = ConnSpec::supabase("abcdefghijklmnopqrst", "sekret", None);
        let described = direct.describe();
        assert!(described.contains("abcdefghijklmnopqrst"), "{described}");
        assert!(!described.contains("sekret"), "{described}");

        let pooled = ConnSpec::supabase("abcdefghijklmnopqrst", "sekret", Some("eu-west-2"));
        assert!(pooled.describe().contains("eu-west-2"));

        // A URL may embed a password; describe must not leak it.
        let url = ConnSpec::Url("postgres://scott:tiger@example.com/mydb".into());
        let described = url.describe();
        assert!(!described.contains("tiger"), "{described}");
        assert!(described.contains("example.com"), "{described}");
    }
}
