use std::path::Path;

use cupid::data::{self, DataSourcePool};
use cupid::{algorithm, models::Appeals, report};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single source of truth: the production Postgres view. Requires DATABASE_URL.
    let pool: DataSourcePool = data::db::load()?;

    // Appeals are independent of the source: optional CSV, resolved against the corpus.
    let appeals = match std::env::var_os("APPEALS_CSV") {
        Some(path) => data::appeals::load_and_resolve(Path::new(&path), &pool)?,
        None => Appeals::new(),
    };

    let result = algorithm::run(pool.applicants(), pool.positions(), &appeals);
    report::print(&result, &pool);
    Ok(())
}
