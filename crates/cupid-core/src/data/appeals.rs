use std::error::Error;

use postgres::Client;

use crate::models::{Appeals, ApplicantIdx, Pool, PositionIdx};

/// Load the DB `cca_appeals` rows and resolve them against the corpus.
///
/// Rows whose applicant or position is not in the pool are skipped: an appeal
/// for a dropped position has no seat to exempt, and an applicant unknown to
/// the corpus never proposes, so neither can affect a run.
pub fn load(client: &mut Client, pool: &Pool) -> Result<Appeals, Box<dyn Error>> {
    let rows = client.query("SELECT user_id, position_id, note FROM cca_appeals", &[])?;

    let mut appeals = Appeals::new();
    for row in &rows {
        let applicant = ApplicantIdx(row.get("user_id"));
        let position = PositionIdx(row.get("position_id"));
        if pool.applicant(applicant).is_none() || pool.position(position).is_none() {
            continue;
        }
        appeals.grant_with_note(applicant, position, row.get("note"));
    }
    Ok(appeals)
}
