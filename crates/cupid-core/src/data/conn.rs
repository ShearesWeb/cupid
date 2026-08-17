use std::error::Error;
use std::sync::Arc;

use postgres::config::{Host, SslMode};
use postgres::{Client, Config};
use rustls::{ClientConfig, RootCertStore};
use tokio_postgres_rustls::MakeRustlsConnect;

/// Supabase serves a chain rooted in its own private CA, and the leaf is
/// valid for five years, past Apple's 398-day cap for TLS server certs. The
/// platform verifiers therefore reject it; pinning the root and verifying
/// with rustls behaves identically on every platform.
const SUPABASE_ROOT_CA: &[u8] = include_bytes!("../../assets/supabase-prod-ca-2021.crt");

/// Pooler host prefixes Supabase assigns. A project sits on exactly one and
/// only the dashboard says which, so both are tried in turn.
const POOLER_PREFIXES: [&str; 2] = ["aws-1", "aws-0"];

/// Reaching the wrong pooler prefix produces this; it means "try the other
/// one", not "the credentials are wrong".
const WRONG_TENANT: &str = "Tenant or user not found";

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
    /// user `postgres`. With region: session pooler
    /// `aws-<n>-<region>.pooler.supabase.com:5432`, user `postgres.<ref>`
    /// (the pooler multiplexes projects, so the project travels in the user).
    /// One config per pooler prefix, since the project's prefix is unknown.
    pub fn configs_read_write(&self) -> Result<Vec<Config>, Box<dyn Error>> {
        match self {
            ConnSpec::Url(url) => Ok(vec![url.parse::<Config>()?]),
            ConnSpec::Supabase {
                project_ref,
                password,
                region,
            } => {
                if project_ref.is_empty() {
                    return Err("Supabase project ref must not be empty".into());
                }
                let hosts: Vec<(String, String)> = match region {
                    Some(region) => POOLER_PREFIXES
                        .iter()
                        .map(|prefix| {
                            (
                                format!("{prefix}-{region}.pooler.supabase.com"),
                                format!("postgres.{project_ref}"),
                            )
                        })
                        .collect(),
                    None => vec![(
                        format!("db.{project_ref}.supabase.co"),
                        "postgres".to_string(),
                    )],
                };
                Ok(hosts
                    .into_iter()
                    .map(|(host, user)| {
                        let mut config = Config::new();
                        config
                            .host(&host)
                            .user(&user)
                            .port(5432)
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
        self.connect_with(self.configs()?)
    }

    /// Open a writable connection. The end-of-cycle purge is the only caller;
    /// everything else must stay on the read-only [`ConnSpec::connect`].
    pub fn connect_read_write(&self) -> Result<Client, Box<dyn Error>> {
        self.connect_with(self.configs_read_write()?)
    }

    fn connect_with(&self, configs: Vec<Config>) -> Result<Client, Box<dyn Error>> {
        let tls = tls_connector()?;
        let mut errors: Vec<String> = Vec::new();
        for config in configs {
            match config.connect(tls.clone()) {
                Ok(client) => return Ok(client),
                Err(e) => errors.push(full_message(&e)),
            }
        }
        // A wrong-prefix rejection says nothing about the credentials, so
        // report a substantive failure ahead of it when one exists.
        let reported = errors
            .iter()
            .find(|e| !e.contains(WRONG_TENANT))
            .or_else(|| errors.first())
            .cloned()
            .unwrap_or_else(|| "no connection candidates".to_string());
        Err(self.explain(reported).into())
    }

    /// Attach guidance to failures whose raw text hides the actual cause.
    fn explain(&self, error: String) -> String {
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

    #[test]
    fn supabase_with_region_tries_every_pooler_prefix() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", Some("ap-southeast-1"));
        let configs = spec.configs().unwrap();
        let hosts: Vec<&str> = configs.iter().map(tcp_host).collect();
        assert_eq!(
            hosts,
            vec![
                "aws-1-ap-southeast-1.pooler.supabase.com",
                "aws-0-ap-southeast-1.pooler.supabase.com",
            ],
            "a project sits on one prefix and the dashboard is the only clue"
        );
        for config in &configs {
            assert_eq!(config.get_ports(), &[5432]);
            assert_eq!(
                config.get_user(),
                Some("postgres.abcdefghijklmnopqrst"),
                "pooler identifies the project via the user"
            );
            assert_eq!(config.get_dbname(), Some("postgres"));
        }
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
            config.get_options().unwrap_or("").contains("default_transaction_read_only=on")
        );
    }

    #[test]
    fn read_write_configs_omit_the_read_only_guard() {
        let spec = ConnSpec::supabase("abcdefghijklmnopqrst", "pw", None);
        let config = spec.configs_read_write().unwrap().remove(0);
        assert!(
            !config.get_options().unwrap_or("").contains("default_transaction_read_only"),
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
        assert_eq!(only(&spec).get_password(), Some("p@ss:w/rd%25 #".as_bytes()));
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
