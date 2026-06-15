use std::error::Error;

use super::resolve::derive;
use super::{appointments, chair_preferences, user_preferences};
use crate::models::Pool;

/// Load the corpus from the production database.
pub fn load() -> Result<Pool, Box<dyn Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

    let tls = postgres_native_tls::MakeTlsConnector::new(native_tls::TlsConnector::new()?);
    let mut client = postgres::Client::connect(&url, tls)?;

    let user_prefs = user_preferences::load(&mut client)?;
    let chair_prefs = chair_preferences::load(&mut client)?;
    let appts = appointments::load(&mut client)?;
    Ok(derive(&user_prefs, &chair_prefs, &appts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a live DATABASE_URL"]
    fn db_load_against_live_database() {
        let pool = load().expect("load from DATABASE_URL");
        // Smoke check: a real run should produce a corpus we can match over.
        assert!(!pool.positions().is_empty(), "expected some positions");
    }
}
