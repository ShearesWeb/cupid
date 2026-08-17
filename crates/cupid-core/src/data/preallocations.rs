use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::models::{ApplicantIdx, CcaIdx, Pool, PositionIdx, Preallocations};

/// One stored preallocation. Preallocations are operator decisions local to
/// this machine: they live in a JSON file in the app's data directory, never
/// in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreallocationRecord {
    pub user_id: i32,
    pub position_id: i32,
    pub note: Option<String>,
}

/// Read the store file. A missing file is an empty store, not an error.
pub fn read_file(path: &Path) -> Result<Vec<PreallocationRecord>, Box<dyn Error>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

/// Write the store file, creating parent directories as needed.
pub fn write_file(path: &Path, records: &[PreallocationRecord]) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(std::fs::write(path, serde_json::to_vec_pretty(records)?)?)
}

/// Resolve stored records against the corpus.
///
/// Records whose applicant or position is not in the pool cannot affect a run
/// (an unknown applicant never proposes; a dropped position has no seat to
/// hold). They are skipped, and each skip is reported as a warning so the
/// operator can clean up the stale record instead of silently losing it.
pub fn resolve(records: &[PreallocationRecord], pool: &Pool) -> (Preallocations, Vec<String>) {
    let mut preallocations = Preallocations::new();
    let mut warnings = Vec::new();
    for record in records {
        let applicant = ApplicantIdx(record.user_id);
        let position = PositionIdx(record.position_id);
        if pool.applicant(applicant).is_none() {
            warnings.push(format!(
                "Stale preallocation skipped: applicant {} is not in the corpus (position {}).",
                applicant.0, position.0
            ));
            continue;
        }
        if pool.position(position).is_none() {
            warnings.push(format!(
                "Stale preallocation skipped: position {} is not allocatable (applicant {}).",
                position.0, applicant.0
            ));
            continue;
        }
        preallocations.grant_with_note(applicant, position, record.note.clone());
    }
    warnings.extend(validate(&preallocations, pool));
    (preallocations, warnings)
}

/// Warnings for preallocations that will collide with a database constraint
/// at commit time: more preallocated pairs than a position has open seats,
/// or a holder given two positions in the same CCA (counting committed
/// appointments and external non-resident roles). The run still seats them,
/// because the operator asked for them, but the operator must resolve the
/// conflict before commit succeeds.
pub fn validate(preallocations: &Preallocations, pool: &Pool) -> Vec<String> {
    let mut warnings = Vec::new();

    // Only pairs that will actually seat: resolvable and not already committed.
    let mut seated: Vec<(ApplicantIdx, PositionIdx)> = preallocations
        .iter()
        .filter(|&(a, p)| pool.applicant(a).is_some() && pool.position(p).is_some())
        .filter(|&(a, p)| !pool.appointments().held_by(a).contains(&p))
        .collect();
    seated.sort();

    // Overfilled positions: more preallocated pairs than open seats.
    let mut per_position: BTreeMap<PositionIdx, usize> = BTreeMap::new();
    for &(_, p) in &seated {
        *per_position.entry(p).or_insert(0) += 1;
    }
    for (pid, count) in per_position {
        let position = pool.position(pid).expect("seated pairs are resolvable");
        if count > position.vacancies() {
            warnings.push(format!(
                "Preallocations overfill {} - {}: {} preallocated for {} open seat(s).",
                position.cca.name,
                position.name,
                count,
                position.vacancies()
            ));
        }
    }

    // One-per-CCA conflicts, counting committed appointments and external
    // non-resident roles alongside the preallocations themselves.
    let mut cca_counts: BTreeMap<(ApplicantIdx, CcaIdx), usize> = BTreeMap::new();
    let mut involves_prealloc: BTreeMap<(ApplicantIdx, CcaIdx), bool> = BTreeMap::new();
    for &(a, p) in &seated {
        let cca = pool.position(p).expect("resolvable").cca.id;
        *cca_counts.entry((a, cca)).or_insert(0) += 1;
        involves_prealloc.insert((a, cca), true);
    }
    for appointment in pool.appointments().iter() {
        if let Some(position) = pool.position(appointment.position) {
            *cca_counts.entry((appointment.applicant, position.cca.id)).or_insert(0) += 1;
        }
    }
    for &(a, cca) in pool.external_occupancy() {
        *cca_counts.entry((a, cca)).or_insert(0) += 1;
    }
    for ((a, cca), count) in cca_counts {
        if count > 1 && involves_prealloc.get(&(a, cca)).copied().unwrap_or(false) {
            let applicant = pool
                .applicant(a)
                .map(|x| x.name.clone())
                .unwrap_or_else(|| format!("applicant {}", a.0));
            let cca_name = pool
                .cca(cca)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| format!("CCA {}", cca.0));
            warnings.push(format!(
                "Preallocation gives {applicant} {count} positions in {cca_name}: \
                 the database allows one per CCA, so commit will be rejected."
            ));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Applicant, Appointment, Appointments, Cca, CcaIdx, Position, PositionType,
    };

    fn applicant(id: i32) -> Applicant {
        Applicant::new(id, format!("U{id}"), format!("u{id}@x"), vec![])
    }

    fn position(id: i32, cca: i32, cap: usize) -> Position {
        Position::new(
            id,
            Cca::new(cca, format!("C{cca}")),
            format!("P{id}"),
            None,
            cap,
            PositionType::MainComm,
            vec![],
        )
    }

    #[test]
    fn clean_preallocations_produce_no_warnings() {
        let pool = Pool::new(
            vec![applicant(1), applicant(2)],
            vec![position(10, 1, 2), position(20, 2, 1)],
        );
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        preallocations.grant(ApplicantIdx(2), PositionIdx(20));

        assert!(validate(&preallocations, &pool).is_empty());
    }

    #[test]
    fn warns_when_preallocations_overfill_a_position() {
        let pool = Pool::new(
            vec![applicant(1), applicant(2)],
            vec![position(10, 1, 1)],
        );
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        preallocations.grant(ApplicantIdx(2), PositionIdx(10));

        let warnings = validate(&preallocations, &pool);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains("P10") && warnings[0].contains("2") && warnings[0].contains("1"),
            "warning names the position and the counts: {}",
            warnings[0]
        );
    }

    #[test]
    fn already_appointed_pairs_do_not_count_toward_overfill() {
        // The pair (1, 10) is already committed: the preallocation row is
        // redundant, not an extra seat.
        let pool = Pool::new(vec![applicant(1)], vec![position(10, 1, 1).with_appointed(1)])
            .with_appointments(Appointments::from_iter([Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(10),
            }]));
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));

        assert!(validate(&preallocations, &pool).is_empty());
    }

    #[test]
    fn warns_when_two_preallocations_share_a_cca() {
        let pool = Pool::new(
            vec![applicant(1)],
            vec![position(10, 5, 1), position(20, 5, 1)],
        );
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        preallocations.grant(ApplicantIdx(1), PositionIdx(20));

        let warnings = validate(&preallocations, &pool);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains("U1") && warnings[0].contains("C5"),
            "warning names the applicant and the CCA: {}",
            warnings[0]
        );
    }

    #[test]
    fn warns_when_preallocation_clashes_with_holdings_in_the_cca() {
        // Applicant 1 already holds a committed appointment in CCA 5;
        // applicant 2 holds an external (member) role in CCA 6.
        let pool = Pool::new(
            vec![applicant(1), applicant(2)],
            vec![
                position(10, 5, 1).with_appointed(1),
                position(11, 5, 1),
                position(12, 6, 1),
            ],
        )
        .with_appointments(Appointments::from_iter([Appointment {
            applicant: ApplicantIdx(1),
            position: PositionIdx(10),
        }]))
        .with_external_occupancy(vec![(ApplicantIdx(2), CcaIdx(6))]);

        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(11));
        preallocations.grant(ApplicantIdx(2), PositionIdx(12));

        let warnings = validate(&preallocations, &pool);
        assert_eq!(warnings.len(), 2, "{warnings:?}");
    }

    fn record(user: i32, position: i32, note: Option<&str>) -> PreallocationRecord {
        PreallocationRecord { user_id: user, position_id: position, note: note.map(String::from) }
    }

    fn temp_store(name: &str) -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!("cupid-prealloc-test-{}-{name}.json", std::process::id()))
    }

    #[test]
    fn reading_a_missing_store_yields_an_empty_list() {
        let path = temp_store("missing");
        let _ = std::fs::remove_file(&path);
        assert_eq!(read_file(&path).unwrap(), vec![]);
    }

    #[test]
    fn store_round_trip_preserves_records_and_notes() {
        let path = temp_store("round-trip");
        let records = vec![record(1, 10, Some("chair's pick")), record(2, 20, None)];
        write_file(&path, &records).unwrap();
        assert_eq!(read_file(&path).unwrap(), records);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_creates_missing_parent_directories() {
        let dir = std::env::temp_dir()
            .join(format!("cupid-prealloc-test-{}-nested", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("deep").join("store.json");
        write_file(&path, &[record(1, 10, None)]).unwrap();
        assert_eq!(read_file(&path).unwrap(), vec![record(1, 10, None)]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_grants_known_pairs_and_warns_on_stale_records() {
        let pool = Pool::new(vec![applicant(1)], vec![position(10, 1, 1)]);
        let records = vec![
            record(1, 10, Some("keep")),
            record(99, 10, None), // unknown applicant
            record(1, 99, None),  // unknown position
        ];

        let (preallocations, warnings) = resolve(&records, &pool);
        assert_eq!(
            preallocations.iter().collect::<Vec<_>>(),
            vec![(ApplicantIdx(1), PositionIdx(10))]
        );
        assert_eq!(preallocations.note(ApplicantIdx(1), PositionIdx(10)), Some("keep"));
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[0].contains("applicant 99"), "{}", warnings[0]);
        assert!(warnings[1].contains("position 99"), "{}", warnings[1]);
    }

    #[test]
    fn resolve_appends_validation_warnings() {
        // Two grants into a single seat: resolve must surface the overfill.
        let pool = Pool::new(vec![applicant(1), applicant(2)], vec![position(10, 1, 1)]);
        let records = vec![record(1, 10, None), record(2, 10, None)];

        let (_, warnings) = resolve(&records, &pool);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("overfill"), "{}", warnings[0]);
    }
}
