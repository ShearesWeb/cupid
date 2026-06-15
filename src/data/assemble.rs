use std::collections::{HashMap, HashSet};

use super::chair_pref::ChairPrefRecord;
use super::user_pref::UserPrefRecord;
use super::DataSourcePool;
use crate::models::{Applicant, ApplicantIdx, CCAIdx, Position, PositionIdx, PositionType};

/// Build the owned corpus from the two preference tables. Pure: no I/O.
///
/// The tables are independent inputs: `chair_prefs` defines positions and the
/// chair's ranking; `user_prefs` defines applicants and their preferences.
///
/// - Positions whose `position_type` is not cupid-allocated are dropped.
/// - Positions with NULL capacity are skipped (unknown seat count).
/// - An applicant's preferences are filtered to surviving positions.
/// - Rankings/preferences are sorted by their 1-based rank, ascending.
pub fn assemble(
    user_prefs: Vec<UserPrefRecord>,
    chair_prefs: Vec<ChairPrefRecord>,
) -> DataSourcePool {
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
    for r in &chair_prefs {
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
        let ranking: Vec<ApplicantIdx> =
            acc.ranked.iter().map(|&(uid, _)| ApplicantIdx(uid)).collect();
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
    for r in &user_prefs {
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

    DataSourcePool::new(applicants, positions, ccas)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let pool = assemble(user_prefs, chair_prefs);

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
        let pool = assemble(user_prefs, chair_prefs);

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
        let pool = assemble(Vec::new(), chair_prefs);

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
        let pool = assemble(user_prefs, chair_prefs);

        let ann = pool
            .applicants()
            .iter()
            .find(|a| a.id == ApplicantIdx(1))
            .unwrap();
        assert_eq!(ann.preferences(), &[PositionIdx(20), PositionIdx(10)]);
    }

    #[test]
    fn collects_cca_names() {
        let pool = assemble(
            vec![upref(1, 10, 1)],
            vec![cpref(10, 1, 1, "maincomm", Some(1), 7)],
        );
        assert_eq!(pool.cca_name(CCAIdx(7)), Some("C7"));
    }
}
