use std::error::Error;

use postgres::Client;

/// One row of `cca_appointments` joined to the holder's identity. An appointment
/// is a position the user already holds; `commitment_period` is intentionally not
/// loaded (cupid treats every appointment as full-year).
#[derive(Debug)]
pub struct AppointmentRecord {
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub position_id: i32,
}

/// Load the DB `cca_appointments` joined to `users`.
pub fn load(client: &mut Client) -> Result<Vec<AppointmentRecord>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT ca.user_id, u.name AS user_name, u.email AS user_email, ca.position_id \
         FROM cca_appointments ca \
         JOIN users u ON u.id = ca.user_id",
        &[],
    )?;
    Ok(rows
        .iter()
        .map(|row| AppointmentRecord {
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            position_id: row.get("position_id"),
        })
        .collect())
}
