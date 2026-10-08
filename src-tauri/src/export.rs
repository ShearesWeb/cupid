//! Publishing a run to the intranet repo: clone, write the allocation CSVs
//! and directory edits, branch, push, and hand back the merge-request URL. The
//! CSV content itself comes from `cupid::export`; this module owns the filesystem and git side.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::Read;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;

use cupid::export::{AppointmentRow, DirectoryEdits, additions, homes, insert, rewrite, slug};

pub const INTRANET_REMOTE: &str = "git@github.com:ShearesWeb/intranet.git";
pub const INTRANET_WEB: &str = "https://github.com/ShearesWeb/intranet";

/// Intranet reconciles every CSV here as one set.
pub const APPOINTMENT_DIR: &str = "data/cca-appointment";

/// Branch for one export, derived from an RFC 3339 timestamp:
/// `cupid/allocation-YYYYMMDD-HHMMSS`.
pub fn branch_name(rfc3339: &str) -> String {
    let digits: String = rfc3339
        .split('.')
        .next()
        .unwrap_or(rfc3339)
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    let (date, time) = digits.split_at(8.min(digits.len()));
    format!("cupid/allocation-{date}-{time}")
}

/// GitHub quick-pull URL: opening it shows the branch diff with the MR form
/// pre-filled, one click from submission.
pub fn pr_url(branch: &str) -> String {
    format!("{INTRANET_WEB}/compare/main...{branch}?quick_pull=1")
}

/// Edits rows where they stand and adds new ones to the file already listing
/// their CCA, else beside the template that names it, else a new
/// `<slug>.csv`. Returns the repo-relative paths written, sorted.
pub fn write_appointments(
    repo: &Path,
    rows: Vec<AppointmentRow>,
    edits: &DirectoryEdits,
) -> Result<Vec<String>, String> {
    let mut bodies = BTreeMap::new();
    for relative in files_ending(repo, APPOINTMENT_DIR, ".csv")? {
        let body = read(repo, &relative)?;
        bodies.insert(relative, body);
    }
    let mut templates = Vec::new();
    for relative in files_ending(repo, APPOINTMENT_DIR, ".csv.template")? {
        let body = read(repo, &relative)?;
        let data = relative.trim_end_matches(".template").to_string();
        templates.push((data, body));
    }

    let mut written = BTreeSet::new();
    let mut found = BTreeSet::new();
    for (relative, body) in &mut bodies {
        let (rewritten, held) = rewrite(body, edits);
        if !held.is_empty() {
            *body = rewritten;
            written.insert(relative.clone());
            found.extend(held);
        }
    }

    let homes = homes(
        bodies
            .iter()
            .chain(templates.iter().map(|(path, body)| (path, body)))
            .map(|(path, body)| (path.as_str(), body.as_str())),
    );
    for (cca, rows) in additions(rows, edits, &found) {
        let relative = homes
            .get(&cca)
            .cloned()
            .unwrap_or_else(|| format!("{APPOINTMENT_DIR}/{}.csv", slug(&cca)));
        let body = bodies.entry(relative.clone()).or_default();
        let updated = insert(body, &rows);
        if updated != *body {
            *body = updated;
            written.insert(relative);
        }
    }

    for relative in &written {
        let path = repo.join(relative);
        std::fs::write(&path, &bodies[relative]).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(written.into_iter().collect())
}

fn read(repo: &Path, relative: &str) -> Result<String, String> {
    let path = repo.join(relative);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// `.csv` does not match `*.csv.template`, which is not data.
fn files_ending(repo: &Path, dir: &str, suffix: &str) -> Result<Vec<String>, String> {
    let mut pending = vec![dir.to_string()];
    let mut files = Vec::new();
    while let Some(relative) = pending.pop() {
        let entries =
            std::fs::read_dir(repo.join(&relative)).map_err(|e| format!("{relative}: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{relative}: {e}"))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|name| format!("{relative}: non-UTF-8 file name {name:?}"))?;
            let child = format!("{relative}/{name}");
            if entry
                .file_type()
                .map_err(|e| format!("{child}: {e}"))?
                .is_dir()
            {
                pending.push(child);
            } else if name.ends_with(suffix) {
                files.push(child);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Split an scp-style remote (`git@host:path`) into its ssh destination and
/// repository path. `None` for anything that is not scp-style.
pub fn split_remote(remote: &str) -> Option<(&str, &str)> {
    if remote.contains("://") {
        return None;
    }
    let (host, path) = remote.split_once(':')?;
    (host.contains('@') && !path.is_empty()).then_some((host, path))
}

/// Translate raw ssh/git-receive-pack stderr into operator guidance.
pub fn explain_ssh_failure(stderr: &str) -> String {
    let stderr = stderr.trim();
    if stderr.contains("Permission denied") {
        return format!(
            "GitHub rejected this machine's SSH key — add one with access to \
             ShearesWeb/intranet (ssh -T git@github.com to verify). [{stderr}]"
        );
    }
    // GitHub answers "not granted" to read-only collaborators and hides the
    // repo entirely ("not found") from everyone else; both mean no push.
    if stderr.contains("not granted") || stderr.contains("Repository not found") {
        return format!(
            "The SSH key works but has no push access to ShearesWeb/intranet — \
             ask for write permission. [{stderr}]"
        );
    }
    stderr.to_string()
}

/// Probe push access without touching anything: ask the remote for its
/// `git-receive-pack` service. GitHub only serves it to users with write
/// access, so a ref advertisement on stdout proves the SSH key works AND may
/// push; read-only or unknown users get an error on stderr instead.
pub fn check_push_access() -> Result<(), String> {
    let (host, path) = split_remote(INTRANET_REMOTE)
        .ok_or_else(|| format!("unsupported remote: {INTRANET_REMOTE}"))?;
    let mut child = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ConnectionAttempts=1",
            host,
            &format!("git-receive-pack '{path}'"),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ssh: {e}"))?;

    let mut stdout = child.stdout.take().ok_or("ssh: stdout was not captured")?;
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut byte = [0; 1];
        let result = stdout.read(&mut byte);
        let _ = sender.send(result);
    });

    let first_byte = receiver.recv_timeout(Duration::from_secs(12));
    let access_granted = matches!(first_byte, Ok(Ok(count)) if count > 0);
    let timed_out = matches!(first_byte, Err(mpsc::RecvTimeoutError::Timeout));
    let _ = child.kill();
    let _ = child.wait();

    if access_granted {
        return Ok(());
    }
    if timed_out {
        return Err("SSH access check timed out after 12 seconds. Check network access and SSH configuration.".to_string());
    }
    let stderr = child
        .stderr
        .take()
        .map(|mut stream| {
            let mut output = Vec::new();
            let _ = stream.read_to_end(&mut output);
            output
        })
        .unwrap_or_default();
    Err(explain_ssh_failure(&String::from_utf8_lossy(&stderr)))
}

/// Run one git command in `dir`, surfacing stderr on failure. Never prompts:
/// the SSH key and git identity must already be configured on this machine.
pub fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
}

/// The full publish: fresh shallow clone under `export_root`, CSVs written,
/// branch committed and pushed. Returns (files written, branch, MR URL).
pub fn publish(
    export_root: &Path,
    rfc3339: &str,
    rows: Vec<AppointmentRow>,
    edits: &DirectoryEdits,
) -> Result<(Vec<String>, String, String), String> {
    let branch = branch_name(rfc3339);
    let checkout = clone_fresh(export_root, &branch)?;
    let files = write_appointments(&checkout, rows, edits)?;
    git(&checkout, &["checkout", "-b", &branch])?;
    git(&checkout, &["add", APPOINTMENT_DIR])?;
    git(
        &checkout,
        &["commit", "-m", "feat(cca-appointment): cupid export"],
    )?;
    git(&checkout, &["push", "origin", &branch])?;
    Ok((files, branch.clone(), pr_url(&branch)))
}

/// Shallow-clone the intranet repo into a directory named after the branch's
/// timestamp suffix. A leftover directory from a crashed run is removed: every
/// export starts from the remote's current state.
fn clone_fresh(export_root: &Path, branch: &str) -> Result<PathBuf, String> {
    let name = branch.rsplit('/').next().unwrap_or(branch);
    let checkout = export_root.join(name);
    if checkout.exists() {
        std::fs::remove_dir_all(&checkout).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(export_root).map_err(|e| e.to_string())?;
    git(
        export_root,
        &["clone", "--depth", "1", INTRANET_REMOTE, name],
    )?;
    Ok(checkout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cca: &str, position: &str, email: &str) -> AppointmentRow {
        AppointmentRow {
            cca_name: cca.into(),
            position_name: position.into(),
            user_email: email.into(),
        }
    }

    fn write(repo: &Path, relative: &str, body: &str) {
        let path = repo.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn read(repo: &Path, relative: &str) -> String {
        std::fs::read_to_string(repo.join(relative)).unwrap()
    }

    fn temp_repo(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cupid-export-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn split_remote_handles_scp_style_urls() {
        assert_eq!(
            split_remote("git@github.com:ShearesWeb/intranet.git"),
            Some(("git@github.com", "ShearesWeb/intranet.git"))
        );
    }

    #[test]
    fn split_remote_rejects_non_scp_urls() {
        assert_eq!(
            split_remote("https://github.com/ShearesWeb/intranet.git"),
            None
        );
        assert_eq!(split_remote("/local/path"), None);
    }

    #[test]
    fn ssh_failures_are_explained_for_the_operator() {
        assert!(
            explain_ssh_failure("git@github.com: Permission denied (publickey).")
                .contains("SSH key"),
            "missing/rejected key names the SSH key"
        );
        assert!(
            explain_ssh_failure("ERROR: Write access to repository not granted.")
                .contains("push access"),
            "read-only collaborators are told they lack push access"
        );
        assert!(
            explain_ssh_failure("ERROR: Repository not found.").contains("push access"),
            "GitHub hides private repos from outsiders; same guidance"
        );
    }

    #[test]
    fn unknown_ssh_failures_pass_through_verbatim() {
        assert!(
            explain_ssh_failure("ssh: connect to host github.com port 22: Network is unreachable")
                .contains("Network is unreachable")
        );
    }

    #[test]
    fn branch_name_compacts_the_timestamp() {
        assert_eq!(
            branch_name("2026-08-16T03:20:11Z"),
            "cupid/allocation-20260816-032011"
        );
    }

    #[test]
    fn branch_name_ignores_fractional_seconds() {
        assert_eq!(
            branch_name("2026-08-16T03:20:11.1234567Z"),
            "cupid/allocation-20260816-032011"
        );
    }

    #[test]
    fn pr_url_targets_the_quick_pull_compare_view() {
        assert_eq!(
            pr_url("cupid/allocation-20260816-032011"),
            "https://github.com/ShearesWeb/intranet/compare/main...cupid/allocation-20260816-032011?quick_pull=1"
        );
    }

    const HEADER: &str = "user_email,cca_name,position_name,commitment_period\n";

    #[test]
    fn write_appointments_adds_rows_to_the_file_listing_their_cca() {
        let repo = temp_repo("adds-in-place");
        write(
            &repo,
            "data/cca-appointment/sports/all.csv",
            &format!(
                "{HEADER}ann@x,Badminton F,Captain,full-year\nben@x,Netball,Captain,full-year\n"
            ),
        );
        write(
            &repo,
            "data/cca-appointment/committees/shweb.csv",
            &format!("{HEADER}cat@x,Sheares Web,Chairperson,full-year\n"),
        );

        let files = write_appointments(
            &repo,
            vec![
                row("Badminton F", "Member", "new@x"),
                row("Sheares Web", "Chairperson", "cat@x"),
            ],
            &DirectoryEdits::new(),
        )
        .unwrap();

        assert_eq!(
            files,
            vec!["data/cca-appointment/sports/all.csv"],
            "a row the file already lists changes nothing"
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv"),
            format!(
                "{HEADER}ann@x,Badminton F,Captain,full-year\n\
                 new@x,Badminton F,Member,full-year\n\
                 ben@x,Netball,Captain,full-year\n"
            )
        );
        assert!(!repo.join("data/cca-appointment/allocation").exists());
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn write_appointments_falls_back_to_the_template_then_a_new_file() {
        let repo = temp_repo("adds-fallback");
        write(
            &repo,
            "data/cca-appointment/committees/rag.csv.template",
            &format!("{HEADER}x@x,Receive And Give (RAG),Chairperson,full-year\n"),
        );

        let files = write_appointments(
            &repo,
            vec![
                row("Receive And Give (RAG)", "Member", "a@x"),
                row("New Club", "Chair", "b@x"),
            ],
            &DirectoryEdits::new(),
        )
        .unwrap();

        assert_eq!(
            files,
            vec![
                "data/cca-appointment/committees/rag.csv",
                "data/cca-appointment/new_club.csv",
            ]
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/committees/rag.csv"),
            format!("{HEADER}a@x,Receive And Give (RAG),Member,full-year\n")
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/committees/rag.csv.template"),
            format!("{HEADER}x@x,Receive And Give (RAG),Chairperson,full-year\n"),
            "templates are not data"
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/new_club.csv"),
            format!("{HEADER}b@x,New Club,Chair,full-year\n")
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn write_appointments_edits_rows_in_the_files_that_list_them() {
        let repo = temp_repo("edits-in-place");
        write(
            &repo,
            "data/cca-appointment/sports/all.csv",
            &format!(
                "{HEADER}ann@x,Badminton F,Captain,full-year\nben@x,Netball,Captain,full-year\n"
            ),
        );
        write(
            &repo,
            "data/cca-appointment/committees/blockcomm/block_a.csv",
            &format!("{HEADER}cat@x,Block A,Head,full-year\n"),
        );
        write(
            &repo,
            "data/cca-appointment/sports/all.csv.template",
            &format!("{HEADER}ann@x,Badminton F,Captain,full-year\n"),
        );
        let edits = DirectoryEdits::from([
            (row("Badminton F", "Captain", "ann@x"), None),
            (
                row("Block A", "Head", "cat@x"),
                Some(cupid::directory::CommitmentPeriod::Semester1),
            ),
            (
                row("Netball", "Vice Captain", "dan@x"),
                Some(cupid::directory::CommitmentPeriod::Semester2),
            ),
            (row("Netball", "Captain", "gone@x"), None),
        ]);

        let files = write_appointments(&repo, vec![], &edits).unwrap();

        assert_eq!(
            files,
            vec![
                "data/cca-appointment/committees/blockcomm/block_a.csv",
                "data/cca-appointment/sports/all.csv",
            ]
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv"),
            format!(
                "{HEADER}ben@x,Netball,Captain,full-year\ndan@x,Netball,Vice Captain,semester-2\n"
            )
        );
        assert_eq!(
            read(
                &repo,
                "data/cca-appointment/committees/blockcomm/block_a.csv"
            ),
            format!("{HEADER}cat@x,Block A,Head,semester-1\n")
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv.template"),
            format!("{HEADER}ann@x,Badminton F,Captain,full-year\n"),
            "templates are not data"
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }
}
