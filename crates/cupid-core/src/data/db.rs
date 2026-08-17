use std::error::Error;

use super::conn::ConnSpec;
use super::resolve::{derive, Records};
use super::{appointments, chair_preferences, positions, user_preferences, users};
use crate::models::Pool;

/// Load the corpus from the database `spec` points at. Read-only: cupid's
/// preallocations live in a local store (see `data::preallocations`), not in
/// the database.
pub fn load(spec: &ConnSpec) -> Result<Pool, Box<dyn Error>> {
    let mut client = spec.connect()?;

    let user_records = users::load(&mut client)?;
    let position_records = positions::load(&mut client)?;
    let user_prefs = user_preferences::load(&mut client)?;
    let chair_prefs = chair_preferences::load(&mut client)?;
    let appts = appointments::load(&mut client)?;
    Ok(derive(&Records {
        users: &user_records,
        positions: &position_records,
        user_prefs: &user_prefs,
        chair_prefs: &chair_prefs,
        appointments: &appts,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a live DATABASE_URL"]
    fn db_load_against_live_database() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = load(&ConnSpec::Url(url)).expect("load from DATABASE_URL");
        // Smoke check: a real run should produce a corpus we can match over.
        assert!(pool.positions().count() > 0, "expected some positions");
    }
}
