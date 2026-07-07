use std::path::Path;

use cupid::{algorithm, data, models::Pool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single source of truth: the production Postgres view. Requires DATABASE_URL.
    let pool: Pool = data::db::load()?;

    // Appeals are optional: every CSV in `data/appeals/`, resolved against the
    // corpus. Missing dir or no files -> no appeals.
    let appeals = data::appeals::load_and_resolve(Path::new("data/appeals"), &pool)?;

    let result = algorithm::run(&pool, &appeals);
    Ok(())
}
