//! Publishing a run to the intranet repo: clone, write the allocation CSVs
//! and directory edits, branch, push, and hand back the merge-request URL. The
//! CSV content itself comes from `cupid::export`; this module owns the filesystem and git side.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use cupid::export::{
    AppointmentRow, DirectoryEdits, by_file, merge, merge_lines, rewrite, unplaced,
};

pub const INTRANET_REMOTE: &str = "git@github.com:ShearesWeb/intranet.git";
pub const INTRANET_WEB: &str = "https://github.com/ShearesWeb/intranet";

/// Intranet reconciles every CSV here as one set.
pub const APPOINTMENT_DIR: &str = "data/cca-appointment";

/// Where cupid adds new rows. Edits to existing rows land in whichever file holds them.
pub const ALLOCATION_DIR: &str = "data/cca-appointment/allocation";

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

/// Merge `rows` into the per-CCA CSV files under `repo`'s allocation
/// directory. Returns the repo-relative paths written, sorted.
pub fn write_rows(repo: &Path, rows: Vec<AppointmentRow>) -> Result<Vec<String>, String> {
    let dir = repo.join(ALLOCATION_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut written = Vec::new();
    for (file, rows) in by_file(rows) {
        let path = dir.join(&file);
        let existing = read_existing(&path)?;
        std::fs::write(&path, merge(existing.as_deref(), &rows))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(format!("{ALLOCATION_DIR}/{file}"));
    }
    Ok(written)
}

/// Returns the repo-relative paths written, sorted.
pub fn write_edits(repo: &Path, edits: &DirectoryEdits) -> Result<Vec<String>, String> {
    let mut found = BTreeSet::new();
    let mut written = Vec::new();
    for relative in csv_files(repo, APPOINTMENT_DIR)? {
        let path = repo.join(&relative);
        let body =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (rewritten, held) = rewrite(&body, edits);
        if held.is_empty() {
            continue;
        }
        std::fs::write(&path, rewritten).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(relative);
        found.extend(held);
    }

    let dir = repo.join(ALLOCATION_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (file, lines) in unplaced(edits, &found) {
        let path = dir.join(&file);
        let existing = read_existing(&path)?;
        std::fs::write(&path, merge_lines(existing.as_deref(), lines))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(format!("{ALLOCATION_DIR}/{file}"));
    }
    written.sort();
    written.dedup();
    Ok(written)
}

fn read_existing(path: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(path) {
        Ok(body) => Ok(Some(body)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Skips `*.csv.template`, which is not data.
fn csv_files(repo: &Path, dir: &str) -> Result<Vec<String>, String> {
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
            } else if name.ends_with(".csv") {
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
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            host,
            &format!("git-receive-pack '{path}'"),
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("ssh: {e}"))?;
    // The advertisement (refs + capabilities) only appears on success; exit
    // codes are unreliable here because we hang up mid-protocol.
    if !output.stdout.is_empty() {
        return Ok(());
    }
    Err(explain_ssh_failure(&String::from_utf8_lossy(
        &output.stderr,
    )))
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
    let mut files = write_rows(&checkout, rows)?;
    files.extend(write_edits(&checkout, edits)?);
    files.sort();
    files.dedup();
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

    #[test]
    fn write_rows_creates_per_cca_files_under_the_allocation_dir() {
        let repo = temp_repo("creates");
        let files = write_rows(
            &repo,
            vec![
                row("Alpha Beta", "Chair", "a@x"),
                row("Zeta", "Chair", "z@x"),
            ],
        )
        .unwrap();
        assert_eq!(
            files,
            vec![
                "data/cca-appointment/allocation/alpha_beta.csv",
                "data/cca-appointment/allocation/zeta.csv",
            ]
        );
        let body =
            std::fs::read_to_string(repo.join("data/cca-appointment/allocation/alpha_beta.csv"))
                .unwrap();
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\n\
             a@x,Alpha Beta,Chair,full-year\n"
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn write_rows_merges_into_existing_files_without_dropping_rows() {
        let repo = temp_repo("merges");
        let dir = repo.join(ALLOCATION_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("alpha.csv"),
            "user_email,cca_name,position_name,commitment_period\n\
             old@x,Alpha,Chair,full-year\n",
        )
        .unwrap();

        write_rows(&repo, vec![row("Alpha", "Chair", "new@x")]).unwrap();

        let body = std::fs::read_to_string(dir.join("alpha.csv")).unwrap();
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\n\
             new@x,Alpha,Chair,full-year\n\
             old@x,Alpha,Chair,full-year\n"
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn write_edits_changes_rows_in_the_files_that_list_them() {
        let repo = temp_repo("edits-in-place");
        let header = "user_email,cca_name,position_name,commitment_period\n";
        write(
            &repo,
            "data/cca-appointment/sports/all.csv",
            &format!(
                "{header}ann@x,Badminton F,Captain,full-year\nben@x,Netball,Captain,full-year\n"
            ),
        );
        write(
            &repo,
            "data/cca-appointment/committees/blockcomm/block_a.csv",
            &format!("{header}cat@x,Block A,Head,full-year\n"),
        );
        write(
            &repo,
            "data/cca-appointment/sports/all.csv.template",
            &format!("{header}ann@x,Badminton F,Captain,full-year\n"),
        );
        let edits = DirectoryEdits::from([
            (row("Badminton F", "Captain", "ann@x"), None),
            (
                row("Block A", "Head", "cat@x"),
                Some(cupid::directory::CommitmentPeriod::Semester1),
            ),
        ]);

        let files = write_edits(&repo, &edits).unwrap();

        assert_eq!(
            files,
            vec![
                "data/cca-appointment/committees/blockcomm/block_a.csv",
                "data/cca-appointment/sports/all.csv",
            ],
            "no new allocation file: every edited row already had a home"
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv"),
            format!("{header}ben@x,Netball,Captain,full-year\n")
        );
        assert_eq!(
            read(
                &repo,
                "data/cca-appointment/committees/blockcomm/block_a.csv"
            ),
            format!("{header}cat@x,Block A,Head,semester-1\n")
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv.template"),
            format!("{header}ann@x,Badminton F,Captain,full-year\n"),
            "templates are not data"
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn write_edits_adds_rows_no_file_lists_to_the_allocation_dir() {
        let repo = temp_repo("edits-added");
        let header = "user_email,cca_name,position_name,commitment_period\n";
        write(
            &repo,
            "data/cca-appointment/sports/all.csv",
            &format!("{header}ann@x,Badminton F,Captain,full-year\n"),
        );
        let edits = DirectoryEdits::from([
            (
                row("Badminton F", "Vice Captain", "ben@x"),
                Some(cupid::directory::CommitmentPeriod::Semester2),
            ),
            (row("Badminton F", "Captain", "gone@x"), None),
        ]);

        let files = write_edits(&repo, &edits).unwrap();

        assert_eq!(
            files,
            vec!["data/cca-appointment/allocation/badminton_f.csv"]
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/allocation/badminton_f.csv"),
            format!("{header}ben@x,Badminton F,Vice Captain,semester-2\n")
        );
        assert_eq!(
            read(&repo, "data/cca-appointment/sports/all.csv"),
            format!("{header}ann@x,Badminton F,Captain,full-year\n")
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }
}
