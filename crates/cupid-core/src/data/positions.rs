use std::error::Error;

use postgres::Client;

/// One row of `cca_positions` joined to its CCA. The market is this table, not
/// the chair shortlists: residents rank positions before any chair shortlists
/// a candidate, so a shortlist-derived universe would drop those picks.
#[derive(Debug)]
pub struct PositionRecord {
    pub position_id: i32,
    pub position_name: String,
    pub position_type: String,
    pub capacity: Option<i32>,
    pub cca_id: i32,
    pub cca_name: String,
}

/// Load the DB `cca_positions` into `PositionRecord`s.
pub fn load(client: &mut Client) -> Result<Vec<PositionRecord>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT cp.id AS position_id, cp.name AS position_name, \
                cp.position_type::text AS position_type, cp.capacity, \
                c.id AS cca_id, c.name AS cca_name \
         FROM cca_positions cp \
         JOIN ccas c ON c.id = cp.cca_id",
        &[],
    )?;
    Ok(rows
        .iter()
        .map(|row| PositionRecord {
            position_id: row.get("position_id"),
            position_name: row.get("position_name"),
            position_type: row.get("position_type"),
            capacity: row.get("capacity"),
            cca_id: row.get("cca_id"),
            cca_name: row.get("cca_name"),
        })
        .collect())
}
