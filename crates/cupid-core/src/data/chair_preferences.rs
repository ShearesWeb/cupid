use std::error::Error;

use postgres::Client;

/// One row of `preferred_candidates`, joined to the candidate's identity. A
/// chair may shortlist someone who never applied, so the ranking is the only
/// place that candidate's name appears.
#[derive(Debug)]
pub struct ChairPrefRecord {
    pub position_id: i32,
    pub user_id: i32,
    pub rank: i32,
    pub user_name: String,
    pub user_email: String,
}

/// Load the DB `preferred_candidates` into `ChairPrefRecord`s.
pub fn load(client: &mut Client) -> Result<Vec<ChairPrefRecord>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT pc.position_id, pc.user_id, pc.rank, \
                u.name AS user_name, u.email AS user_email \
         FROM preferred_candidates pc \
         JOIN users u ON u.id = pc.user_id",
        &[],
    )?;
    Ok(rows
        .iter()
        .map(|row| ChairPrefRecord {
            position_id: row.get("position_id"),
            user_id: row.get("user_id"),
            rank: row.get("rank"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
        })
        .collect())
}
