use std::error::Error;

use super::conn::ConnSpec;
use super::resolve::from_directory;
use super::{chair_preferences, directory, user_preferences};
use crate::directory::Directory;
use crate::models::Pool;
use postgres::IsolationLevel;

pub struct LoadedData {
    pub directory: Directory,
    pub pool: Pool,
}

/// Load the corpus from the database `spec` points at. Read-only: cupid's
/// preallocations live in a local store (see `data::preallocations`), not in
/// the database.
pub fn load(spec: &ConnSpec) -> Result<Pool, Box<dyn Error>> {
    Ok(load_all(spec)?.pool)
}

/// Read both views from one consistent database revision, including CCAs with
/// no positions. No directory edit or allocation result is persisted here.
pub fn load_all(spec: &ConnSpec) -> Result<LoadedData, Box<dyn Error>> {
    let mut client = spec.connect()?;
    let mut transaction = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()?;
    let directory = directory::load(&mut transaction)?;
    let user_prefs = user_preferences::load(&mut transaction)?;
    let chair_prefs = chair_preferences::load(&mut transaction)?;
    let pool = from_directory(&directory, &user_prefs, &chair_prefs)?;
    transaction.commit()?;
    Ok(LoadedData { directory, pool })
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
