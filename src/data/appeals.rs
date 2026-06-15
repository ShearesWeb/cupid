use std::error::Error;
use std::path::Path;

use serde::Deserialize;

use super::{csv, resolve};
use crate::models::{Appeals, Pool};

/// One appeal row matched by applicant cca position name.
#[derive(Debug, Deserialize)]
pub struct AppealRecord {
    pub applicant_name: String,
    pub cca_name: String,
    pub position_name: String,
}

/// Load every appeal CSV in `dir` and resolve them against `pool`.
pub fn load_and_resolve(dir: &Path, pool: &Pool) -> Result<Appeals, Box<dyn Error>> {
    let records = csv::load_dir::<AppealRecord>(dir)?;
    resolve::derive_appeals(&records, pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn record_trims_every_field() {
        let text = "applicant_name,cca_name,position_name\n  Cara  ,  Chess  ,  Head  \n";
        let records: Vec<AppealRecord> = csv::parse(Cursor::new(text)).unwrap();
        assert_eq!(records[0].applicant_name, "Cara");
        assert_eq!(records[0].cca_name, "Chess");
        assert_eq!(records[0].position_name, "Head");
    }
}
