use cupid::algorithm;
use cupid::data::conn::ConnSpec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The corpus comes from Postgres (DATABASE_URL); preallocations are local
    // operator state, taken from the JSON file PREALLOCATIONS_FILE points at
    // (absent means none).
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let pool = cupid::data::db::load(&ConnSpec::Url(url))?;
    let records = match std::env::var("PREALLOCATIONS_FILE") {
        Ok(path) => cupid::data::preallocations::read_file(std::path::Path::new(&path))?,
        Err(_) => Vec::new(),
    };
    let (preallocations, warnings) = cupid::data::preallocations::resolve(&records, &pool);
    for warning in &warnings {
        eprintln!("warning: {warning}");
    }

    let result = algorithm::run(&pool, &preallocations);

    println!("{} allocations settled", result.all().count());

    let mut unmatched = result.unmatched(pool.applicants());
    unmatched.sort();
    println!("{} unmatched applicants", unmatched.len());
    for id in unmatched {
        if let Some(applicant) = pool.applicant(id) {
            println!("  - {} <{}>", applicant.name, applicant.email);
        }
    }

    let mut unfilled = result.unfilled(pool.positions());
    unfilled.sort();
    println!("{} positions with open seats", unfilled.len());
    for (pid, open) in unfilled {
        if let Some(position) = pool.position(pid) {
            println!("  - {} - {}: {} open", position.cca.name, position.name, open);
        }
    }
    Ok(())
}
