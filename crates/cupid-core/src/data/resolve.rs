use std::collections::{HashMap, HashSet};

use super::appointments::AppointmentRecord;
use super::chair_preferences::ChairPrefRecord;
use super::positions::PositionRecord;
use super::user_preferences::UserPrefRecord;
use super::users::UserRecord;
use crate::models::{
    Applicant, ApplicantIdx, Appointments, Cca, CcaIdx, Pool, Position, PositionIdx, PositionType,
};

/// The raw rows a corpus is built from. Every field defaults to empty, so a
/// caller names only the tables it exercises and no two slices of the same
/// type can be transposed at a call site.
#[derive(Default)]
pub struct Records<'a> {
    pub users: &'a [UserRecord],
    pub positions: &'a [PositionRecord],
    pub user_prefs: &'a [UserPrefRecord],
    pub chair_prefs: &'a [ChairPrefRecord],
    pub appointments: &'a [AppointmentRecord],
}

/// Build the owned [`Pool`] from the roll, the position catalogue, the
/// preference tables and existing appointments.
pub fn derive(records: &Records<'_>) -> Pool {
    let Records {
        users,
        positions: position_records,
        user_prefs,
        chair_prefs,
        appointments: appointment_records,
    } = *records;

    // --- Positions from the catalogue, keeping only what cupid can allocate ---
    struct PosAcc {
        cca_id: i32,
        cca_name: String,
        name: String,
        capacity: usize,
        position_type: PositionType,
        ranked: Vec<(i32, i32)>, // (user_id, rank)
    }
    let mut pos_acc: HashMap<i32, PosAcc> = HashMap::new();
    for r in position_records {
        let Ok(position_type) = r.position_type.parse::<PositionType>() else {
            continue;
        };
        let Some(capacity) = r.capacity else {
            continue;
        };
        pos_acc.insert(
            r.position_id,
            PosAcc {
                cca_id: r.cca_id,
                cca_name: r.cca_name.clone(),
                name: r.position_name.clone(),
                capacity: capacity.max(0) as usize,
                position_type,
                ranked: Vec::new(),
            },
        );
    }

    // --- Chair rankings overlay the catalogue, never extend it ---
    for r in chair_prefs {
        if let Some(acc) = pos_acc.get_mut(&r.position_id) {
            acc.ranked.push((r.user_id, r.rank));
        }
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

    // The whole roll, so the operator can preallocate anyone and every
    // resident is visible whether or not they applied.
    for r in users {
        app_acc.insert(
            r.user_id,
            AppAcc {
                name: r.name.clone(),
                email: r.email.clone(),
                prefs: Vec::new(),
            },
        );
    }

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

    // A chair may shortlist someone who never applied. That candidate still
    // belongs in the corpus: without an Applicant there is no name to show
    // and no no-return to report against the chair's ranking.
    for r in chair_prefs {
        if !kept.contains(&r.position_id) {
            continue;
        }
        app_acc.entry(r.user_id).or_insert_with(|| AppAcc {
            name: r.user_name.clone(),
            email: r.user_email.clone(),
            prefs: Vec::new(),
        });
    }

    // --- Existing appointments (over kept positions only) ---
    // The bidirectional relation lives in one place; positions and applicants
    // derive their counts/filters from it rather than each holding a copy.
    let mut appointments = Appointments::new();
    let mut external_occupancy: Vec<(ApplicantIdx, CcaIdx)> = Vec::new();
    for a in appointment_records {
        if !kept.contains(&a.position_id) {
            // An appointment to a dropped position has no seat to occupy, but
            // any non-resident role still takes the holder's one-per-CCA slot.
            if !a.position_type.eq_ignore_ascii_case("resident") {
                external_occupancy.push((ApplicantIdx(a.user_id), CcaIdx(a.cca_id)));
            }
            continue;
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

    external_occupancy.sort();
    Pool::new(applicants, positions)
        .with_appointments(appointments)
        .with_external_occupancy(external_occupancy)
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

    fn posrec(
        position_id: i32,
        position_type: &str,
        capacity: Option<i32>,
        cca_id: i32,
    ) -> PositionRecord {
        PositionRecord {
            position_id,
            position_name: format!("P{position_id}"),
            position_type: position_type.to_string(),
            capacity,
            cca_id,
            cca_name: format!("C{cca_id}"),
        }
    }

    fn cpref(position_id: i32, user_id: i32, rank: i32) -> ChairPrefRecord {
        ChairPrefRecord {
            position_id,
            user_id,
            rank,
            user_name: format!("U{user_id}"),
            user_email: format!("u{user_id}@x"),
        }
    }

    #[test]
    fn filters_non_cupid_types() {
        let position_records = vec![
            posrec(10, "maincomm", Some(1), 5),
            posrec(20, "member", Some(1), 5), // non-cupid type -> dropped
        ];
        let user_prefs = vec![upref(1, 10, 1), upref(2, 20, 1)];
        let chair_prefs = vec![cpref(10, 1, 1), cpref(20, 2, 1)];
        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, chair_prefs: &chair_prefs, ..Default::default() });

        assert_eq!(pool.positions().count(), 1);
        assert_eq!(pool.position(PositionIdx(10)).unwrap().id, PositionIdx(10));
        // Applicant 2 only ranked the dropped position -> kept, but with no prefs.
        let u2 = pool.applicants().find(|a| a.id == ApplicantIdx(2)).unwrap();
        assert!(u2.preferences().is_empty());
    }

    #[test]
    fn skips_null_capacity_positions() {
        let position_records = vec![posrec(10, "subcomm", None, 5)]; // NULL capacity
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1)];
        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, chair_prefs: &chair_prefs, ..Default::default() });

        assert_eq!(pool.positions().count(), 0);
        // Applicant survives but the pref to the skipped position is dropped.
        assert_eq!(pool.applicants().count(), 1);
        assert!(pool.applicants().next().unwrap().preferences().is_empty());
    }

    #[test]
    fn ranking_sorted_ascending() {
        let position_records = vec![posrec(10, "maincomm", Some(2), 5)];
        let chair_prefs = vec![cpref(10, 1, 2), cpref(10, 3, 1)];
        let pool = derive(&Records { positions: &position_records, chair_prefs: &chair_prefs, ..Default::default() });

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
        let position_records = vec![
            posrec(10, "maincomm", Some(1), 5),
            posrec(20, "subcomm", Some(1), 5),
            posrec(30, "member", Some(1), 5), // dropped type
        ];
        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, ..Default::default() });

        let ann = pool.applicants().find(|a| a.id == ApplicantIdx(1)).unwrap();
        assert_eq!(ann.preferences(), &[PositionIdx(20), PositionIdx(10)]);
    }

    fn apptrec(user_id: i32, position_id: i32) -> AppointmentRecord {
        apptrec_typed(user_id, position_id, 5, "maincomm")
    }

    fn apptrec_typed(
        user_id: i32,
        position_id: i32,
        cca_id: i32,
        position_type: &str,
    ) -> AppointmentRecord {
        AppointmentRecord {
            user_id,
            user_name: format!("U{user_id}"),
            user_email: format!("u{user_id}@x"),
            position_id,
            cca_id,
            position_type: position_type.to_string(),
        }
    }

    #[test]
    fn appointments_fold_into_capacity_and_quota() {
        // Position 10 (cap 2, maincomm) has an existing appointee: applicant 2,
        // who did NOT submit preferences (stub applicant).
        let position_records = vec![posrec(10, "maincomm", Some(2), 5)];
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1)];
        let appts = vec![apptrec(2, 10)];

        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, chair_prefs: &chair_prefs, appointments: &appts, ..Default::default() });

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
        let position_records = vec![posrec(20, "member", Some(2), 5)];
        let appts = vec![apptrec(2, 20)];

        let pool = derive(&Records { positions: &position_records, appointments: &appts, ..Default::default() });

        assert_eq!(pool.positions().count(), 0, "member position dropped");
        assert!(
            pool.applicant(ApplicantIdx(2)).is_none(),
            "holder of a dropped position not added"
        );
    }

    fn userrec(user_id: i32) -> UserRecord {
        UserRecord {
            user_id,
            name: format!("U{user_id}"),
            email: format!("u{user_id}@x"),
        }
    }

    #[test]
    fn every_user_is_an_applicant_even_without_preferences() {
        // The operator preallocates against this list, so the whole roll has
        // to be present, not just the people who submitted something.
        let users = vec![userrec(1), userrec(2), userrec(3)];
        let position_records = vec![posrec(10, "maincomm", Some(2), 5)];
        let user_prefs = vec![upref(1, 10, 1)];
        let pool = derive(&Records {
            users: &users,
            positions: &position_records,
            user_prefs: &user_prefs,
            ..Default::default()
        });

        assert_eq!(pool.applicants().count(), 3);
        assert_eq!(
            pool.applicant(ApplicantIdx(1)).unwrap().preferences(),
            &[PositionIdx(10)]
        );
        let quiet = pool.applicant(ApplicantIdx(2)).expect("non-applicant kept");
        assert_eq!(quiet.name, "U2");
        assert!(quiet.preferences().is_empty());
    }

    #[test]
    fn position_with_no_shortlist_keeps_the_preferences_aimed_at_it() {
        // Residents rank before any chair shortlists. Sourcing the market from
        // the shortlists would drop both the position and every pick for it.
        let position_records = vec![posrec(10, "blockcomm", Some(8), 5)];
        let user_prefs = vec![upref(1, 10, 1)];
        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, ..Default::default() });

        let position = pool
            .position(PositionIdx(10))
            .expect("catalogue position is in the market");
        assert!(position.ranking().is_empty(), "no chair shortlist yet");
        assert_eq!(
            pool.applicant(ApplicantIdx(1)).unwrap().preferences(),
            &[PositionIdx(10)]
        );
    }

    #[test]
    fn chair_ranked_candidate_who_never_applied_is_still_an_applicant() {
        // The chair added user 7 to the shortlist; user 7 submitted nothing.
        // They must appear with an empty preference list so the ranking row
        // can name them and report the no-return.
        let position_records = vec![posrec(10, "blockcomm", Some(8), 5)];
        let chair_prefs = vec![cpref(10, 7, 1)];
        let pool = derive(&Records { positions: &position_records, chair_prefs: &chair_prefs, ..Default::default() });

        let applicant = pool
            .applicant(ApplicantIdx(7))
            .expect("chair's pick is in the corpus");
        assert_eq!(applicant.name, "U7");
        assert!(
            applicant.preferences().is_empty(),
            "they never ranked anything back"
        );
        assert_eq!(
            pool.position(PositionIdx(10)).unwrap().ranking(),
            &[ApplicantIdx(7)]
        );
    }

    #[test]
    fn chair_ranking_for_a_dropped_position_adds_no_applicant() {
        // Position type cupid cannot allocate, so the position never enters
        // the market and its shortlist must not conjure applicants.
        let position_records = vec![posrec(10, "resident", Some(8), 5)];
        let chair_prefs = vec![cpref(10, 7, 1)];
        let pool = derive(&Records { positions: &position_records, chair_prefs: &chair_prefs, ..Default::default() });
        assert!(pool.applicant(ApplicantIdx(7)).is_none());
    }

    #[test]
    fn dropped_nonresident_appointment_occupies_its_cca() {
        use crate::models::CcaIdx;

        // Applicant 2 holds a `member` role (not allocatable, dropped from the
        // market) in CCA 7, and a room-derived `resident` appointment in CCA 8.
        // Only the member role occupies a one-per-CCA slot; kept appointments
        // (position 10) are counted via the position instead.
        let position_records = vec![posrec(10, "maincomm", Some(2), 5)];
        let chair_prefs = vec![cpref(10, 1, 1)];
        let appts = vec![
            apptrec_typed(2, 90, 7, "member"),
            apptrec_typed(2, 91, 8, "resident"),
            apptrec_typed(3, 10, 5, "maincomm"),
        ];

        let pool = derive(&Records { positions: &position_records, chair_prefs: &chair_prefs, appointments: &appts, ..Default::default() });
        assert_eq!(
            pool.external_occupancy(),
            &[(ApplicantIdx(2), CcaIdx(7))],
            "member occupies CCA 7; resident exempt; kept appointment not duplicated here"
        );
    }

    #[test]
    fn held_position_is_removed_from_preferences() {
        let position_records = vec![posrec(10, "maincomm", Some(2), 5)];
        let user_prefs = vec![upref(1, 10, 1)];
        let chair_prefs = vec![cpref(10, 1, 1)];
        let appts = vec![apptrec(1, 10)];

        let pool = derive(&Records { positions: &position_records, user_prefs: &user_prefs, chair_prefs: &chair_prefs, appointments: &appts, ..Default::default() });
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
