use cupid::algorithm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single source of truth: the production Postgres tables (corpus and
    // appeals alike). Requires DATABASE_URL.
    let (pool, appeals, warnings) = cupid::data::db::load()?;
    for warning in &warnings {
        eprintln!("warning: {warning}");
    }

    let result = algorithm::run(&pool, &appeals);

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
