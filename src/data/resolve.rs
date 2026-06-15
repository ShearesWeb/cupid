use std::collections::{HashMap, HashSet};
use std::error::Error;

use super::appeals::AppealRecord;
use super::chair_preferences::ChairPrefRecord;
use super::user_preferences::UserPrefRecord;
use crate::models::{
    Appeals, Applicant, ApplicantIdx, CCAIdx, Pool, Position, PositionIdx, PositionType,
};

/// Build the owned [`Pool`] from the two preference tables.
pub fn derive(
    user_prefs: &[UserPrefRecord],
    chair_prefs: &[ChairPrefRecord],
) -> Pool {
    // --- Positions (+ chair rankings) from the chair-preference side ---
    struct PosAcc {
        cca_id: i32,
        name: String,
        capacity: usize,
        position_type: PositionType,
        ranked: Vec<(i32, i32)>, // (user_id, rank)
    }
    let mut pos_acc: HashMap<i32, PosAcc> = HashMap::new();
    let mut ccas: HashMap<CCAIdx, String> = HashMap::new();
    for r in chair_prefs {
        // Skip rows whose position_type cupid doesn't allocate.
        let Ok(position_type) = r.position_type.parse::<PositionType>() else {
            continue;
        };
        // Skip positions with unknown capacity.
        let Some(capacity) = r.capacity else {
            continue;
        };
        ccas.entry(CCAIdx(r.cca_id))
            .or_insert_with(|| r.cca_name.clone());
        let acc = pos_acc.entry(r.position_id).or_insert_with(|| PosAcc {
            cca_id: r.cca_id,
            name: r.position_name.clone(),
            capacity: capacity.max(0) as usize,
            position_type,
            ranked: Vec::new(),
        });
        acc.ranked.push((r.user_id, r.rank));
    }

    let mut positions: Vec<Position> = Vec::new();
    let mut kept: HashSet<i32> = HashSet::new();
    for (position_id, mut acc) in pos_acc {
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
            acc.capacity,
            acc.position_type,
            ranking,
        ));
        kept.insert(position_id);
    }

    // --- Applicants (+ preferences) from the user-preference side ---
    struct AppAcc {
        name: String,
        email: String,
        prefs: Vec<(i32, i32)>, // (position_id, rank) over kept positions only
    }
    let mut app_acc: HashMap<i32, AppAcc> = HashMap::new();
    for r in user_prefs {
        let acc = app_acc.entry(r.user_id).or_insert_with(|| AppAcc {
            name: r.user_name.clone(),
            email: r.user_email.clone(),
            prefs: Vec::new(),
        });
        if kept.contains(&r.position_id) {
            acc.prefs.push((r.position_id, r.rank));
        }
    }

    let mut applicants: Vec<Applicant> = Vec::new();
    for (user_id, mut acc) in app_acc {
        acc.prefs.sort_by_key(|&(_, rank)| rank);
        let preferences: Vec<PositionIdx> =
            acc.prefs.iter().map(|&(pid, _)| PositionIdx(pid)).collect();
        applicants.push(Applicant::new(user_id, acc.name, acc.email, preferences));
    }

    Pool::new(applicants, positions, ccas)
}

/// Resolve appeal rows against the loaded corpus by name. Pure: no I/O.
///
/// Every row is matched to one applicant (by name) and one position (by CCA +
/// position name). Unmatched and ambiguous rows are collected and reported
/// together as a single error so the operator fixes them in one pass.
pub fn derive_appeals(
    records: &[AppealRecord],
    pool: &Pool,
) -> Result<Appeals, Box<dyn Error>> {
    let mut by_name: HashMap<String, Vec<ApplicantIdx>> = HashMap::new();
    for a in pool.applicants() {
        by_name
            .entry(a.name.trim().to_string())
            .or_default()
            .push(a.id);
    }

    let mut by_cca_pos: HashMap<(String, String), PositionIdx> = HashMap::new();
    for p in pool.positions() {
        if let Some(cca) = pool.cca_name(p.cca_id) {
            by_cca_pos.insert((cca.trim().to_string(), p.name.trim().to_string()), p.id);
        }
    }

    let mut appeals = Appeals::new();
    let mut problems: Vec<String> = Vec::new();

    for rec in records {
        let applicant_id = match by_name.get(&rec.applicant_name) {
            None => {
                problems.push(format!("unknown applicant '{}'", rec.applicant_name));
                continue;
            }
            Some(ids) if ids.len() > 1 => {
                problems.push(format!(
                    "ambiguous applicant '{}' ({} matches)",
                    rec.applicant_name,
                    ids.len()
                ));
                continue;
            }
            Some(ids) => ids[0],
        };

        let key = (rec.cca_name.clone(), rec.position_name.clone());
        let position_id = match by_cca_pos.get(&key) {
            None => {
                problems.push(format!(
                    "unknown position '{}' in CCA '{}'",
                    rec.position_name, rec.cca_name
                ));
                continue;
            }
            Some(id) => *id,
        };

        appeals.grant(applicant_id, position_id);
    }

    if problems.is_empty() {
        Ok(appeals)
    } else {
        Err(format!(
            "{} problem(s) resolving appeals:\n{}",
            problems.len(),
            problems.join("\n")
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- corpus derivation (`derive`) ----

    fn upref(user_id: i32, position_id: i32, rank: i32) -> UserPrefRecord {
        UserPrefRecord {
            user_id,
            position_id,
            rank,
            user_name: format!("U{user_id}"),
            user_email: format!("u{user_id}@x"),
        }
    }

    fn cpref(
        position_id: i32,
        user_id: i32,
        rank: i32,
        position_type: &str,
        capacity: Option<i32>,
        cca_id: i32,
    ) -> ChairPrefRecord {
        ChairPrefRecord {
            position_id,
            user_id,
            rank,
            position_name: format!("P{position_id}"),
            position_type: position_type.to_string(),
            capacity,
            cca_id,
            cca_name: format!("C{cca_id}"),
        }
    }

    #[test]
    fn filters_non_cupid_types() {
        let user_prefs = vec![upref(1, 10, 1), upref(2, 20, 1)];
        let chair_prefs = vec![
            cpref(10, 1, 1, "maincomm", Some(1), 5),
            cpref(20, 2, 1, "member", Some(1), 5), // non-cupid type -> dropped
        ];
        let pool = derive(&user_prefs, &chair_prefs);

        assert_eq!(pool.positions().len(), 1);
        assert_eq!(pool.positions()[0].id, PositionIdx(10));
        // Applicant 2 only ranked the dropped position -> kept, but with no prefs.
        let u2 = pool
            .applicants()
            .iter()
            .find(|a| a.id == ApplicantIdx(2))
            .unwrap();
        assert!(u2.preferences().is_empty());
    }

    #[test]
    fn skips_null_capacity_positions() {
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1, "subcomm", None, 5)]; // NULL capacity
        let pool = derive(&user_prefs, &chair_prefs);

        assert!(pool.positions().is_empty());
        // Applicant survives but the pref to the skipped position is dropped.
        assert_eq!(pool.applicants().len(), 1);
        assert!(pool.applicants()[0].preferences().is_empty());
    }

    #[test]
    fn ranking_sorted_ascending() {
        let chair_prefs = vec![
            cpref(10, 1, 2, "maincomm", Some(2), 5),
            cpref(10, 3, 1, "maincomm", Some(2), 5),
        ];
        let pool = derive(&[], &chair_prefs);

        assert_eq!(
            pool.positions()[0].ranking(),
            &[ApplicantIdx(3), ApplicantIdx(1)]
        );
    }

    #[test]
    fn preferences_sorted_and_filtered_to_kept_positions() {
        let user_prefs = vec![
            upref(1, 10, 2),
            upref(1, 20, 1),
            upref(1, 30, 3), // 30 is a member position -> not kept
        ];
        let chair_prefs = vec![
            cpref(10, 1, 1, "maincomm", Some(1), 5),
            cpref(20, 1, 1, "subcomm", Some(1), 5),
            cpref(30, 1, 1, "member", Some(1), 5), // dropped type
        ];
        let pool = derive(&user_prefs, &chair_prefs);

        let ann = pool
            .applicants()
            .iter()
            .find(|a| a.id == ApplicantIdx(1))
            .unwrap();
        assert_eq!(ann.preferences(), &[PositionIdx(20), PositionIdx(10)]);
    }

    #[test]
    fn collects_cca_names() {
        let pool = derive(
            &[upref(1, 10, 1)],
            &[cpref(10, 1, 1, "maincomm", Some(1), 7)],
        );
        assert_eq!(pool.cca_name(CCAIdx(7)), Some("C7"));
    }

    // ---- appeals resolution (`derive_appeals`) ----

    // Corpus: Ann(1), Ben(2), Ann(3) [duplicate name], Cara(4);
    // one position "Head" (id 10) under CCA "Chess" (id 1).
    fn pool() -> Pool {
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![]),
            Applicant::new(3, "Ann".into(), "ann2@x".into(), vec![]),
            Applicant::new(4, "Cara".into(), "cara@x".into(), vec![]),
        ];
        let positions = vec![Position::new(
            10,
            1,
            "Head".into(),
            None,
            1,
            PositionType::MainComm,
            vec![ApplicantIdx(4)],
        )];
        let mut ccas = HashMap::new();
        ccas.insert(CCAIdx(1), "Chess".to_string());
        Pool::new(applicants, positions, ccas)
    }

    fn appeal(applicant: &str, cca: &str, position: &str) -> AppealRecord {
        AppealRecord {
            applicant_name: applicant.to_owned(),
            cca_name: cca.to_owned(),
            position_name: position.to_owned(),
        }
    }

    #[test]
    fn resolves_valid_row() {
        let records = vec![appeal("Cara", "Chess", "Head")];
        let appeals = derive_appeals(&records, &pool()).unwrap();
        assert!(appeals.contains(ApplicantIdx(4), PositionIdx(10)));
    }

    #[test]
    fn unknown_applicant_is_a_problem() {
        let records = vec![appeal("Zed", "Chess", "Head")];
        let err = derive_appeals(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("unknown applicant 'Zed'"), "got: {err}");
    }

    #[test]
    fn ambiguous_applicant_is_a_problem() {
        let records = vec![appeal("Ann", "Chess", "Head")];
        let err = derive_appeals(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("ambiguous applicant 'Ann'"), "got: {err}");
    }

    #[test]
    fn unknown_position_is_a_problem() {
        let records = vec![appeal("Cara", "Chess", "Ghost")];
        let err = derive_appeals(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("unknown position 'Ghost'"), "got: {err}");
    }

    #[test]
    fn problems_aggregate() {
        let records = vec![
            appeal("Zed", "Chess", "Head"),
            appeal("Cara", "Chess", "Ghost"),
        ];
        let err = derive_appeals(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("2 problem(s)"), "got: {err}");
    }
}
