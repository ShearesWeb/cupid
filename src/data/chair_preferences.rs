use std::error::Error;

use postgres::Client;

/// One row of `cca_position_preferences`, joined to the position's metadata.
#[derive(Debug)]
pub struct ChairPrefRecord {
    pub position_id: i32,
    pub user_id: i32,
    pub rank: i32,
    pub position_name: String,
    pub position_type: String,
    pub capacity: Option<i32>,
    pub cca_id: i32,
    pub cca_name: String,
}

/// Load the DB `cca_position_preferences` into `ChairPrefRecord>`.
pub fn load(client: &mut Client) -> Result<Vec<ChairPrefRecord>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT pp.position_id, pp.user_id, pp.rank, \
                cp.name AS position_name, cp.position_type::text AS position_type, \
                cp.capacity, cp.cca_id, c.name AS cca_name \
         FROM cca_position_preferences pp \
         JOIN cca_positions cp ON cp.id = pp.position_id \
         JOIN ccas c ON c.id = cp.cca_id",
        &[],
    )?;
    Ok(rows
        .iter()
        .map(|row| ChairPrefRecord {
            position_id: row.get("position_id"),
            user_id: row.get("user_id"),
            rank: row.get("rank"),
            position_name: row.get("position_name"),
            position_type: row.get("position_type"),
            capacity: row.get("capacity"),
            cca_id: row.get("cca_id"),
            cca_name: row.get("cca_name"),
        })
        .collect())
}
