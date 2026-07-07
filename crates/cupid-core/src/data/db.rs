use std::error::Error;

use super::resolve::derive;
use super::{appeals, appointments, chair_preferences, user_preferences};
use crate::models::{Appeals, Pool};

/// Load the corpus and its appeals from the production database.
pub fn load() -> Result<(Pool, Appeals), Box<dyn Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

    let tls = postgres_native_tls::MakeTlsConnector::new(native_tls::TlsConnector::new()?);
    let mut client = postgres::Client::connect(&url, tls)?;

    let user_prefs = user_preferences::load(&mut client)?;
    let chair_prefs = chair_preferences::load(&mut client)?;
    let appts = appointments::load(&mut client)?;
    let pool = derive(&user_prefs, &chair_prefs, &appts);
    let appeals = appeals::load(&mut client, &pool)?;
    Ok((pool, appeals))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a live DATABASE_URL"]
    fn db_load_against_live_database() {
        let (pool, _appeals) = load().expect("load from DATABASE_URL");
        // Smoke check: a real run should produce a corpus we can match over.
        assert!(pool.positions().count() > 0, "expected some positions");
    }
}
