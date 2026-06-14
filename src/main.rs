mod algorithm;
mod data;
mod models;
mod report;

use std::path::Path;

use data::DataSourcePool;
use models::Appeals;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PRODUCTION set -> load from the database; unset -> mock fixtures (the safe default).
    let pool: DataSourcePool = if std::env::var_os("PRODUCTION").is_some() {
        data::db::load()?
    } else {
        data::mock::load()
    };

    // Appeals are independent of the source: optional CSV, resolved against the corpus.
    let appeals = match std::env::var_os("APPEALS_CSV") {
        Some(path) => data::appeals::load_and_resolve(Path::new(&path), &pool)?,
        None => Appeals::new(),
    };

    let result = algorithm::run(pool.applicants(), pool.positions(), &appeals);
    report::print(&result, &pool);
    Ok(())
}
