use std::error::Error;

use postgres::Client;

/// One row of `cca_user_preferences` (user side), joined to the applicant's
/// identity. `rank` is the applicant's 1-based choice order (1 = top pick).
#[derive(Debug)]
pub struct UserPrefRecord {
    pub user_id: i32,
    pub position_id: i32,
    pub rank: i32,
    pub user_name: String,
    pub user_email: String,
}

/// Load the user side: `cca_user_preferences` joined to `users`.
///
/// Nullability: `rank` is NOT NULL (CHECK rank > 0) and the join is inner over a
/// NOT NULL FK, so `user_name`/`user_email` are always present — the non-Option
/// `row.get`s cannot panic on a NULL.
pub fn load(client: &mut Client) -> Result<Vec<UserPrefRecord>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT up.user_id, up.position_id, up.rank, \
                u.name AS user_name, u.email AS user_email \
         FROM cca_user_preferences up \
         JOIN users u ON u.id = up.user_id",
        &[],
    )?;
    Ok(rows
        .iter()
        .map(|row| UserPrefRecord {
            user_id: row.get("user_id"),
            position_id: row.get("position_id"),
            rank: row.get("rank"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
        })
        .collect())
}
