use std::error::Error;

use postgres::Client;

/// One row of `users`. Every resident belongs in the corpus whether or not
/// they applied: the operator preallocates against this list, and a person
/// who submitted nothing is still someone a seat can be handed to.
#[derive(Debug)]
pub struct UserRecord {
    pub user_id: i32,
    pub name: String,
    pub email: String,
}

/// Load the DB `users` into `UserRecord`s.
pub fn load(client: &mut Client) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    let rows = client.query("SELECT id, name, email FROM users", &[])?;
    Ok(rows
        .iter()
        .map(|row| UserRecord {
            user_id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        })
        .collect())
}
