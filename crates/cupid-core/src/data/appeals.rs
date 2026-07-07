use std::error::Error;

use postgres::Client;

use crate::models::{Appeals, ApplicantIdx, Pool, PositionIdx};

/// Load the DB `cca_appeals` rows and resolve them against the corpus.
///
/// Rows whose applicant or position is not in the pool cannot affect a run
/// (an unknown applicant never proposes; a dropped position has no seat to
/// exempt). They are skipped, and each skip is reported as a warning so the
/// operator can clean up the stale row instead of silently losing it.
pub fn load(client: &mut Client, pool: &Pool) -> Result<(Appeals, Vec<String>), Box<dyn Error>> {
    let rows = client.query("SELECT user_id, position_id, note FROM cca_appeals", &[])?;

    let mut appeals = Appeals::new();
    let mut warnings = Vec::new();
    for row in &rows {
        let applicant = ApplicantIdx(row.get("user_id"));
        let position = PositionIdx(row.get("position_id"));
        if pool.applicant(applicant).is_none() {
            warnings.push(format!(
                "Stale appeal skipped: applicant {} is not in the corpus (position {}).",
                applicant.0, position.0
            ));
            continue;
        }
        if pool.position(position).is_none() {
            warnings.push(format!(
                "Stale appeal skipped: position {} is not allocatable (applicant {}).",
                position.0, applicant.0
            ));
            continue;
        }
        appeals.grant_with_note(applicant, position, row.get("note"));
    }
    Ok((appeals, warnings))
}
