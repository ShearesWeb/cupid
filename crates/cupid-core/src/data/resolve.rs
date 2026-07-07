use std::collections::{HashMap, HashSet};

use super::appointments::AppointmentRecord;
use super::chair_preferences::ChairPrefRecord;
use super::user_preferences::UserPrefRecord;
use crate::models::{
    Applicant, ApplicantIdx, Appointments, Cca, Pool, Position, PositionIdx, PositionType,
};

/// Build the owned [`Pool`] from the preference tables and existing appointments.
pub fn derive(
    user_prefs: &[UserPrefRecord],
    chair_prefs: &[ChairPrefRecord],
    appointment_records: &[AppointmentRecord],
) -> Pool {
    // --- Positions (+ chair rankings) from the chair-preference side ---
    struct PosAcc {
        cca_id: i32,
        cca_name: String,
        name: String,
        capacity: usize,
        position_type: PositionType,
        ranked: Vec<(i32, i32)>, // (user_id, rank)
    }
    let mut pos_acc: HashMap<i32, PosAcc> = HashMap::new();
    for r in chair_prefs {
        let Ok(position_type) = r.position_type.parse::<PositionType>() else {
            continue;
        };
        let Some(capacity) = r.capacity else {
            continue;
        };
        let acc = pos_acc.entry(r.position_id).or_insert_with(|| PosAcc {
            cca_id: r.cca_id,
            cca_name: r.cca_name.clone(),
            name: r.position_name.clone(),
            capacity: capacity.max(0) as usize,
            position_type,
            ranked: Vec::new(),
        });
        acc.ranked.push((r.user_id, r.rank));
    }

    // Positions that survived (allocatable type + known capacity).
    let kept: HashSet<i32> = pos_acc.keys().copied().collect();

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

    // --- Existing appointments (over kept positions only) ---
    // The bidirectional relation lives in one place; positions and applicants
    // derive their counts/filters from it rather than each holding a copy.
    let mut appointments = Appointments::new();
    for a in appointment_records {
        if !kept.contains(&a.position_id) {
            continue; // appointment to a dropped position has no seat to occupy
        }
        appointments.insert(ApplicantIdx(a.user_id), PositionIdx(a.position_id));
        // A holder who did not apply becomes an Applicant with empty preferences.
        app_acc.entry(a.user_id).or_insert_with(|| AppAcc {
            name: a.user_name.clone(),
            email: a.user_email.clone(),
            prefs: Vec::new(),
        });
    }

    // --- Build positions, shrinking vacancies by their appointee count ---
    let mut positions: Vec<Position> = Vec::new();
    for (position_id, mut acc) in pos_acc {
        acc.ranked.sort_by_key(|&(_, rank)| rank);
        let ranking: Vec<ApplicantIdx> = acc
            .ranked
            .iter()
            .map(|&(uid, _)| ApplicantIdx(uid))
            .collect();
        positions.push(
            Position::new(
                position_id,
                Cca::new(acc.cca_id, acc.cca_name),
                acc.name,
                None,
                acc.capacity,
                acc.position_type,
                ranking,
            )
            .with_appointed(appointments.count_at(PositionIdx(position_id))),
        );
    }

    // --- Build applicants, dropping any preference already held by appointment ---
    let mut applicants: Vec<Applicant> = Vec::new();
    for (user_id, mut acc) in app_acc {
        acc.prefs.sort_by_key(|&(_, rank)| rank);
        let held = appointments.held_by(ApplicantIdx(user_id));
        // A position already held by appointment must not be matched again.
        let preferences: Vec<PositionIdx> = acc
            .prefs
            .iter()
            .map(|&(pid, _)| PositionIdx(pid))
            .filter(|pid| !held.contains(pid))
            .collect();
        applicants.push(Applicant::new(user_id, acc.name, acc.email, preferences));
    }

    Pool::new(applicants, positions).with_appointments(appointments)
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
        let pool = derive(&user_prefs, &chair_prefs, &[]);

        assert_eq!(pool.positions().count(), 1);
        assert_eq!(pool.position(PositionIdx(10)).unwrap().id, PositionIdx(10));
        // Applicant 2 only ranked the dropped position -> kept, but with no prefs.
        let u2 = pool.applicants().find(|a| a.id == ApplicantIdx(2)).unwrap();
        assert!(u2.preferences().is_empty());
    }

    #[test]
    fn skips_null_capacity_positions() {
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1, "subcomm", None, 5)]; // NULL capacity
        let pool = derive(&user_prefs, &chair_prefs, &[]);

        assert_eq!(pool.positions().count(), 0);
        // Applicant survives but the pref to the skipped position is dropped.
        assert_eq!(pool.applicants().count(), 1);
        assert!(pool.applicants().next().unwrap().preferences().is_empty());
    }

    #[test]
    fn ranking_sorted_ascending() {
        let chair_prefs = vec![
            cpref(10, 1, 2, "maincomm", Some(2), 5),
            cpref(10, 3, 1, "maincomm", Some(2), 5),
        ];
        let pool = derive(&[], &chair_prefs, &[]);

        assert_eq!(
            pool.position(PositionIdx(10)).unwrap().ranking(),
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
        let pool = derive(&user_prefs, &chair_prefs, &[]);

        let ann = pool.applicants().find(|a| a.id == ApplicantIdx(1)).unwrap();
        assert_eq!(ann.preferences(), &[PositionIdx(20), PositionIdx(10)]);
    }

    fn apptrec(user_id: i32, position_id: i32) -> AppointmentRecord {
        AppointmentRecord {
            user_id,
            user_name: format!("U{user_id}"),
            user_email: format!("u{user_id}@x"),
            position_id,
        }
    }

    #[test]
    fn appointments_fold_into_capacity_and_quota() {
        // Position 10 (cap 2, maincomm) has an existing appointee: applicant 2,
        // who did NOT submit preferences (stub applicant).
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1, "maincomm", Some(2), 5)];
        let appts = vec![apptrec(2, 10)];

        let pool = derive(&user_prefs, &chair_prefs, &appts);

        // The appointment shrinks the position's vacancies; one seat already gone.
        let p = pool.position(PositionIdx(10)).unwrap();
        assert_eq!(
            pool.appointments().holders(PositionIdx(10)),
            &[ApplicantIdx(2)]
        );
        assert_eq!(p.vacancies(), 1, "cap 2 minus 1 appointee");

        // The non-applicant holder is now an Applicant with empty preferences,
        // and the relation records the appointment on their side too.
        let holder = pool
            .applicant(ApplicantIdx(2))
            .expect("stub applicant added");
        assert!(holder.preferences.is_empty());
        assert_eq!(
            pool.appointments().held_by(ApplicantIdx(2)),
            &[PositionIdx(10)]
        );
    }

    #[test]
    fn appointment_to_dropped_position_is_ignored() {
        // Position 20 is a non-cupid type -> dropped; its appointment must vanish.
        let chair_prefs = vec![cpref(20, 1, 1, "member", Some(2), 5)];
        let appts = vec![apptrec(2, 20)];

        let pool = derive(&[], &chair_prefs, &appts);

        assert_eq!(pool.positions().count(), 0, "member position dropped");
        assert!(
            pool.applicant(ApplicantIdx(2)).is_none(),
            "holder of a dropped position not added"
        );
    }

    #[test]
    fn held_position_is_removed_from_preferences() {
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1, "maincomm", Some(2), 5)];
        let appts = vec![apptrec(1, 10)];

        let pool = derive(&user_prefs, &chair_prefs, &appts);
        let a = pool.applicant(ApplicantIdx(1)).unwrap();
        assert!(
            a.preferences().is_empty(),
            "held position filtered out of prefs"
        );
        assert_eq!(
            pool.appointments().held_by(ApplicantIdx(1)),
            &[PositionIdx(10)]
        );
    }

}
