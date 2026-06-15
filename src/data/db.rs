use std::error::Error;

use super::assemble::assemble;
use super::{DataSourcePool, chair_pref, user_pref};

/// Load the corpus from the production database.
///
/// Opens one connection and reads the two preference tables directly (service-role,
/// bypassing RLS) — one loader per table — then assembles the owned corpus.
pub fn load() -> Result<DataSourcePool, Box<dyn Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

    let tls = postgres_native_tls::MakeTlsConnector::new(native_tls::TlsConnector::new()?);
    let mut client = postgres::Client::connect(&url, tls)?;

    let user_prefs = user_pref::load(&mut client)?;
    let chair_prefs = chair_pref::load(&mut client)?;
    Ok(assemble(user_prefs, chair_prefs))
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
