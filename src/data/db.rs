use std::collections::{HashMap, HashSet};
use std::error::Error;

use super::DataSourcePool;
use crate::models::{Applicant, ApplicantIdx, CCAIdx, Position, PositionIdx, PositionType};

/// One row of the `cca_preference_pairs` view.
#[derive(Debug, Clone)]
pub struct PrefRow {
    pub user_id: i32,
    pub position_id: i32,
    pub user_rank: Option<i32>,
    pub position_rank: Option<i32>,
    pub user_name: String,
    pub user_email: String,
    pub position_name: String,
    pub position_type: String,
    pub capacity: Option<i32>,
    pub cca_id: i32,
    pub cca_name: String,
}

/// Build the owned corpus from raw view rows. Pure: no I/O.
///
/// - Keeps only rows whose `position_type` is one cupid allocates.
/// - Positions with NULL capacity are skipped (unknown seat count).
/// - Rankings/preferences are sorted by their 1-based rank, ascending.
pub fn assemble(rows: Vec<PrefRow>) -> DataSourcePool {
    // Keep only rows whose position_type is a cupid-allocated type.
    let rows: Vec<PrefRow> = rows
        .into_iter()
        .filter(|r| r.position_type.parse::<PositionType>().is_ok())
        .collect();

    // --- Positions: group by position_id ---
    struct PosAcc {
        cca_id: i32,
        name: String,
        capacity: Option<i32>,
        position_type: PositionType,
        ranked: Vec<(i32, i32)>, // (user_id, position_rank)
    }
    let mut pos_acc: HashMap<i32, PosAcc> = HashMap::new();
    for r in &rows {
        let ty: PositionType = r.position_type.parse().expect("filtered to valid types");
        let acc = pos_acc.entry(r.position_id).or_insert_with(|| PosAcc {
            cca_id: r.cca_id,
            name: r.position_name.clone(),
            capacity: r.capacity,
            position_type: ty,
            ranked: Vec::new(),
        });
        if let Some(rank) = r.position_rank {
            acc.ranked.push((r.user_id, rank));
        }
    }

    let mut positions: Vec<Position> = Vec::new();
    let mut kept: HashSet<i32> = HashSet::new();
    for (position_id, mut acc) in pos_acc {
        let Some(capacity) = acc.capacity else {
            continue; // NULL capacity -> skip
        };
        acc.ranked.sort_by_key(|&(_, rank)| rank);
        let ranking: Vec<ApplicantIdx> = acc
            .ranked
            .iter()
            .map(|&(uid, _)| ApplicantIdx(uid))
            .collect();
        positions.push(Position::new(
            position_id,
            acc.cca_id,
            acc.name,
            None,
            capacity.max(0) as usize,
            acc.position_type,
            ranking,
        ));
        kept.insert(position_id);
    }

    // --- Applicants: group by user_id ---
    struct AppAcc {
        name: String,
        email: String,
        prefs: Vec<(i32, i32)>, // (position_id, user_rank) over kept positions only
    }
    let mut app_acc: HashMap<i32, AppAcc> = HashMap::new();
    for r in &rows {
        let acc = app_acc.entry(r.user_id).or_insert_with(|| AppAcc {
            name: r.user_name.clone(),
            email: r.user_email.clone(),
            prefs: Vec::new(),
        });
        if let Some(rank) = r.user_rank
            && kept.contains(&r.position_id)
        {
            acc.prefs.push((r.position_id, rank));
        }
    }

    let mut applicants: Vec<Applicant> = Vec::new();
    for (user_id, mut acc) in app_acc {
        acc.prefs.sort_by_key(|&(_, rank)| rank);
        let preferences: Vec<PositionIdx> =
            acc.prefs.iter().map(|&(pid, _)| PositionIdx(pid)).collect();
        applicants.push(Applicant::new(user_id, acc.name, acc.email, preferences));
    }

    // --- CCAs: id -> name ---
    let mut ccas: HashMap<CCAIdx, String> = HashMap::new();
    for r in &rows {
        ccas.entry(CCAIdx(r.cca_id))
            .or_insert_with(|| r.cca_name.clone());
    }

    DataSourcePool::new(applicants, positions, ccas)
}

/// Load the corpus from the production database.
///
/// Queries the `cca_preference_pairs` view (service-role only) over `DATABASE_URL`,
/// maps rows to `PrefRow`, and assembles. The Postgres enum `position_type` is cast
/// to text so it deserializes as a `String`.
pub fn load() -> Result<DataSourcePool, Box<dyn Error>> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set when PRODUCTION is set")?;

    let tls = postgres_native_tls::MakeTlsConnector::new(native_tls::TlsConnector::new()?);
    let mut client = postgres::Client::connect(&url, tls)?;

    let rows = client.query(
        "SELECT user_id, position_id, user_rank, position_rank, user_name, user_email, \
                position_name, position_type::text AS position_type, capacity, cca_id, cca_name \
         FROM cca_preference_pairs",
        &[],
    )?;

    // Nullability note: `user_id`/`position_id` are COALESCE(up, pp) over the view's
    // FULL OUTER JOIN, so at least one side is always present — never NULL. The name/
    // email/cca columns reach the row via LEFT JOINs on those non-null keys, and the
    // preference tables FK to users/cca_positions with ON DELETE CASCADE, so the joined
    // row always exists — also never NULL. Only `user_rank`/`position_rank`/`capacity`
    // are nullable, and those are the `Option<i32>` fields. Hence the non-Option
    // `row.get`s below cannot panic on a NULL.
    let parsed: Vec<PrefRow> = rows
        .iter()
        .map(|row| PrefRow {
            user_id: row.get("user_id"),
            position_id: row.get("position_id"),
            user_rank: row.get("user_rank"),
            position_rank: row.get("position_rank"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            position_name: row.get("position_name"),
            position_type: row.get("position_type"),
            capacity: row.get("capacity"),
            cca_id: row.get("cca_id"),
            cca_name: row.get("cca_name"),
        })
        .collect();

    Ok(assemble(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        user_id: i32,
        position_id: i32,
        user_rank: Option<i32>,
        position_rank: Option<i32>,
        position_type: &str,
        capacity: Option<i32>,
        cca_id: i32,
    ) -> PrefRow {
        PrefRow {
            user_id,
            position_id,
            user_rank,
            position_rank,
            user_name: format!("U{user_id}"),
            user_email: format!("u{user_id}@x"),
            position_name: format!("P{position_id}"),
            position_type: position_type.to_string(),
            capacity,
            cca_id,
            cca_name: format!("C{cca_id}"),
        }
    }

    #[test]
    fn filters_non_cupid_types() {
        let rows = vec![
            row(1, 10, Some(1), Some(1), "maincomm", Some(1), 5),
            row(2, 20, Some(1), Some(1), "member", Some(1), 5), // dropped
        ];
        let pool = assemble(rows);
        assert_eq!(pool.positions().len(), 1);
        assert_eq!(pool.positions()[0].id, PositionIdx(10));
        // Applicant 2 only ranked a dropped position -> absent.
        assert_eq!(pool.applicants().len(), 1);
    }

    #[test]
    fn skips_null_capacity_positions() {
        let rows = vec![row(1, 10, Some(1), Some(1), "subcomm", None, 5)];
        let pool = assemble(rows);
        assert!(pool.positions().is_empty());
        // Applicant survives (valid type) but the pref to the skipped position is dropped.
        assert_eq!(pool.applicants().len(), 1);
        assert!(pool.applicants()[0].preferences().is_empty());
    }

    #[test]
    fn ranking_sorted_ascending() {
        let rows = vec![
            row(1, 10, None, Some(2), "maincomm", Some(2), 5),
            row(3, 10, None, Some(1), "maincomm", Some(2), 5),
        ];
        let pool = assemble(rows);
        assert_eq!(
            pool.positions()[0].ranking(),
            &[ApplicantIdx(3), ApplicantIdx(1)]
        );
    }

    #[test]
    fn preferences_sorted_and_filtered_to_kept_positions() {
        let rows = vec![
            row(1, 10, Some(2), Some(1), "maincomm", Some(1), 5),
            row(1, 20, Some(1), Some(1), "subcomm", Some(1), 5),
            row(1, 30, Some(3), Some(1), "member", Some(1), 5), // dropped type
        ];
        let pool = assemble(rows);
        let ann = pool
            .applicants()
            .iter()
            .find(|a| a.id == ApplicantIdx(1))
            .unwrap();
        assert_eq!(ann.preferences(), &[PositionIdx(20), PositionIdx(10)]);
    }

    #[test]
    fn collects_cca_names() {
        let rows = vec![row(1, 10, Some(1), Some(1), "maincomm", Some(1), 7)];
        let pool = assemble(rows);
        assert_eq!(pool.cca_name(CCAIdx(7)), Some("C7"));
    }

    #[test]
    #[ignore = "requires a live DATABASE_URL"]
    fn db_load_against_live_database() {
        let pool = load().expect("load from DATABASE_URL");
        // Smoke check: a real run should produce a corpus we can match over.
        assert!(!pool.positions().is_empty(), "expected some positions");
    }
}
