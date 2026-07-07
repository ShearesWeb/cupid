use cupid::algorithm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single source of truth: the production Postgres tables (corpus and
    // appeals alike). Requires DATABASE_URL.
    let (pool, appeals) = cupid::data::db::load()?;

    let _result = algorithm::run(&pool, &appeals);
    Ok(())
}
