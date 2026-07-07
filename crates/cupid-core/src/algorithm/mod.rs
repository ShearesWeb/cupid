mod gale_shapley;
mod immediate_acceptance;

use crate::models::{Algorithm, Appeals, CapacityStore, Ledger, MatchResult, Pool, Roster};

/// Run the full two-pass allocation.
///
///   Pass 1: Immediate Acceptance over all BlockComm positions.
///   Pass 2: Gale-Shapley over all MainComm + SubComm positions.
pub fn run(pool: &Pool, appeals: &Appeals) -> MatchResult {
    let ia = Roster::for_algorithm(
        pool.applicants(),
        pool.positions(),
        Algorithm::ImmediateAcceptance,
    );
    let gs = Roster::for_algorithm(pool.applicants(), pool.positions(), Algorithm::GaleShapley);

    // One ledger and store carry across both passes
    let mut ledger: Ledger = Ledger::new(Algorithm::ImmediateAcceptance);
    let mut store: CapacityStore = CapacityStore::from_pool(pool, appeals);

    immediate_acceptance::run(&ia, appeals, &mut store, &mut ledger);
    ledger.enter(Algorithm::GaleShapley, 0);
    gale_shapley::run(&gs, appeals, &mut store, &mut ledger);

    ledger.finish()
}

#[cfg(test)]
mod tests {
    use crate::algorithm::run;
    use crate::models::{
        Appeals, Applicant, ApplicantIdx, Appointment, Appointments, Cca, Pool, Position,
        PositionIdx, PositionType,
    };

    #[test]
    fn gs_proposes_below_a_higher_ranked_blockcomm() {
        // Ann ranks a blockcomm (IA) position ABOVE a maincomm (GS) position.
        // The GS pass must still seat her in the maincomm seat.
        let applicants = vec![Applicant::new(
            1,
            "Ann".into(),
            "ann@x".into(),
            vec![PositionIdx(100), PositionIdx(200)], // blockcomm first, maincomm second
        )];
        let positions = vec![
            Position::new(
                100,
                Cca::new(0, "C"),
                "Block".into(),
                None,
                1,
                PositionType::BlockComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                200,
                Cca::new(0, "C"),
                "Main".into(),
                None,
                1,
                PositionType::MainComm,
                vec![ApplicantIdx(1)],
            ),
        ];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());

        let held = result.positions_of(ApplicantIdx(1));
        assert!(
            held.contains(&PositionIdx(200)),
            "Ann should hold maincomm seat 200; held: {held:?}"
        );
    }

    use crate::models::PositionType::{BlockComm, MainComm, SubComm};

    #[test]
    fn appeal_bypasses_quota() {
        // Applicant 1 wants a maincomm + two subcomms. Holding 1 main + 2 sub violates
        // the (main>=1 && sub>=2) rule, so the second subcomm (301) is normally rejected.
        let applicants = vec![Applicant::new(
            1,
            "Ann".into(),
            "a@x".into(),
            vec![PositionIdx(200), PositionIdx(300), PositionIdx(301)],
        )];
        let positions = vec![
            Position::new(
                200,
                Cca::new(0, "C"),
                "Main".into(),
                None,
                1,
                MainComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                300,
                Cca::new(0, "C"),
                "Sub A".into(),
                None,
                1,
                SubComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                301,
                Cca::new(0, "C"),
                "Sub B".into(),
                None,
                1,
                SubComm,
                vec![ApplicantIdx(1)],
            ),
        ];

        // Without an appeal: 301 is rejected on quota.
        let plain = run(
            &Pool::new(applicants.clone(), positions.clone()),
            &Appeals::new(),
        );
        assert!(
            !plain
                .positions_of(ApplicantIdx(1))
                .contains(&PositionIdx(301))
        );

        // With an appeal on (1, 301): the quota check is bypassed and 301 is seated.
        let mut appeals = Appeals::new();
        appeals.grant(ApplicantIdx(1), PositionIdx(301));
        let appealed = run(&Pool::new(applicants, positions), &appeals);
        assert!(
            appealed
                .positions_of(ApplicantIdx(1))
                .contains(&PositionIdx(301))
        );
    }

    #[test]
    fn chair_unranked_applicant_is_not_seated() {
        // The chair ranks only applicant 2; applicant 1 proposes but is never ranked.
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![PositionIdx(200)]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![]),
        ];
        let positions = vec![Position::new(
            200,
            Cca::new(0, "C"),
            "Main".into(),
            None,
            1,
            MainComm,
            vec![ApplicantIdx(2)],
        )];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());
        assert!(
            !result
                .positions_of(ApplicantIdx(1))
                .contains(&PositionIdx(200))
        );
    }

    #[test]
    fn gs_quota_block_is_revisited_after_displacement_frees_quota() {
        // Ann (id 1) wants three maincomms. She seats M10 and M11 (quota
        // full), her M12 proposal is quota-blocked, and THEN Bea bumps her
        // from M10. Deferred acceptance must let Ann come back for M12 once
        // the displacement frees her quota; a permanent quota rejection
        // would leave M12 wrongly unfilled.
        let applicants = vec![
            Applicant::new(
                1,
                "Ann".into(),
                "a@x".into(),
                vec![PositionIdx(10), PositionIdx(11), PositionIdx(12)],
            ),
            // Bea burns two sweeps on positions that never rank her, reaching
            // M10 only after Ann's quota is already full.
            Applicant::new(
                2,
                "Bea".into(),
                "b@x".into(),
                vec![PositionIdx(90), PositionIdx(91), PositionIdx(10)],
            ),
        ];
        let main = |id: i32, ranking: Vec<ApplicantIdx>| {
            Position::new(id, Cca::new(0, "C"), format!("M{id}"), None, 1, MainComm, ranking)
        };
        let positions = vec![
            main(10, vec![ApplicantIdx(2), ApplicantIdx(1)]), // chair prefers Bea
            main(11, vec![ApplicantIdx(1)]),
            main(12, vec![ApplicantIdx(1)]),
            main(90, vec![]), // ranks nobody
            main(91, vec![]),
        ];
        let pool = Pool::new(applicants, positions);

        let result = run(&pool, &Appeals::new());
        let ann = result.positions_of(ApplicantIdx(1)).to_vec();
        assert!(
            ann.contains(&PositionIdx(11)) && ann.contains(&PositionIdx(12)),
            "Ann must hold M11 and M12 after being bumped from M10; held: {ann:?}"
        );
        assert_eq!(result.positions_of(ApplicantIdx(2)), &[PositionIdx(10)]);
        assert!(
            !result
                .unfilled(pool.positions())
                .iter()
                .any(|&(pid, _)| pid == PositionIdx(12)),
            "M12 must not be reported unfilled"
        );
    }

    #[test]
    fn runs_are_deterministic_across_identical_inputs() {
        // HashMap iteration order varies per instance; the passes must not.
        let build = || {
            let applicants: Vec<Applicant> = (1..=12)
                .map(|i| {
                    Applicant::new(
                        i,
                        format!("A{i}"),
                        format!("a{i}@x"),
                        vec![PositionIdx(10), PositionIdx(11), PositionIdx(12)],
                    )
                })
                .collect();
            let ranking: Vec<ApplicantIdx> = (1..=12).map(ApplicantIdx).collect();
            let positions = vec![
                Position::new(10, Cca::new(0, "C"), "M".into(), None, 2, MainComm, ranking.clone()),
                Position::new(11, Cca::new(0, "C"), "S".into(), None, 2, SubComm, ranking.clone()),
                Position::new(12, Cca::new(0, "C"), "B".into(), None, 2, BlockComm, ranking),
            ];
            Pool::new(applicants, positions)
        };

        let fingerprint = |result: &crate::models::MatchResult| {
            let allocs: Vec<(i32, i32, u32)> = {
                let mut v: Vec<_> = result
                    .all()
                    .map(|a| (a.applicant_id.0, a.position_id.0, a.accepted_at.seq))
                    .collect();
                v.sort();
                v
            };
            let events: Vec<(u32, i32, i32)> = {
                let mut v: Vec<_> = result
                    .events()
                    .map(|e| (e.step.seq, e.applicant_id.0, e.position_id.0))
                    .collect();
                v.sort();
                v
            };
            (allocs, events)
        };

        let first = fingerprint(&run(&build(), &Appeals::new()));
        for _ in 0..5 {
            assert_eq!(fingerprint(&run(&build(), &Appeals::new())), first);
        }
    }

    #[test]
    fn gs_bumps_weaker_held_seat() {
        // GS deferred acceptance: a chair-preferred late proposer displaces the seated.
        // Applicant 2 takes the lone seat first; applicant 1 (chair rank 1) bumps them.
        let applicants = vec![
            // 1 first proposes a seat the chair never ranked them for, then the contested M.
            Applicant::new(
                1,
                "Win".into(),
                "w@x".into(),
                vec![PositionIdx(40), PositionIdx(30)],
            ),
            Applicant::new(2, "Lose".into(), "l@x".into(), vec![PositionIdx(30)]),
        ];
        let positions = vec![
            Position::new(
                30,
                Cca::new(0, "C"),
                "M".into(),
                None,
                1,
                MainComm,
                vec![ApplicantIdx(1), ApplicantIdx(2)],
            ),
            Position::new(
                40,
                Cca::new(0, "C"),
                "N".into(),
                None,
                1,
                MainComm,
                vec![ApplicantIdx(2)],
            ), // 1 unranked here
        ];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());
        assert!(
            result
                .positions_of(ApplicantIdx(1))
                .contains(&PositionIdx(30)),
            "chair-preferred applicant 1 should hold M"
        );
        assert!(
            result.positions_of(ApplicantIdx(2)).is_empty(),
            "applicant 2 should be bumped out of M and hold nothing"
        );
    }

    #[test]
    fn ia_seats_in_chair_order_when_oversubscribed() {
        // Both rank the same single BlockComm seat at top choice. IA seats the
        // chair's preferred applicant regardless of proposal order.
        let applicants = vec![
            Applicant::new(1, "Top".into(), "t@x".into(), vec![PositionIdx(50)]),
            Applicant::new(2, "Snd".into(), "s@x".into(), vec![PositionIdx(50)]),
        ];
        let positions = vec![Position::new(
            50,
            Cca::new(0, "C"),
            "B".into(),
            None,
            1,
            BlockComm,
            vec![ApplicantIdx(1), ApplicantIdx(2)],
        )];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());
        assert_eq!(result.positions_of(ApplicantIdx(1)), &[PositionIdx(50)]);
        assert!(result.positions_of(ApplicantIdx(2)).is_empty());
    }

    #[test]
    fn ia_acceptance_is_permanent_no_bumping() {
        // Contrast with GS: here the chair PREFERS applicant 2, but applicant 1
        // claims the seat first (at a better applicant-rank). IA never bumps, so the
        // later, chair-preferred applicant 2 is turned away.
        let applicants = vec![
            Applicant::new(1, "Early".into(), "e@x".into(), vec![PositionIdx(60)]),
            // 2 wastes rank-0 on a seat that doesn't list them, reaching B one round late.
            Applicant::new(
                2,
                "Late".into(),
                "l@x".into(),
                vec![PositionIdx(61), PositionIdx(60)],
            ),
        ];
        let positions = vec![
            Position::new(
                60,
                Cca::new(0, "C"),
                "B".into(),
                None,
                1,
                BlockComm,
                vec![ApplicantIdx(2), ApplicantIdx(1)],
            ),
            Position::new(61, Cca::new(0, "C"), "X".into(), None, 1, BlockComm, vec![]), // ranks nobody
        ];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());
        assert_eq!(
            result.positions_of(ApplicantIdx(1)),
            &[PositionIdx(60)],
            "first claimer keeps the seat under IA permanence"
        );
        assert!(
            result.positions_of(ApplicantIdx(2)).is_empty(),
            "chair-preferred but late applicant gets no bump in IA"
        );
    }

    #[test]
    fn blockcomm_holdings_carry_into_gs_quota() {
        // Two BlockComm seats (pass 1) exhaust the main+block <= 2 quota, so the
        // MainComm proposal in pass 2 is rejected for capacity.
        let applicants = vec![Applicant::new(
            1,
            "Ann".into(),
            "a@x".into(),
            vec![PositionIdx(70), PositionIdx(71), PositionIdx(72)],
        )];
        let positions = vec![
            Position::new(
                70,
                Cca::new(0, "C"),
                "B1".into(),
                None,
                1,
                BlockComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                71,
                Cca::new(0, "C"),
                "B2".into(),
                None,
                1,
                BlockComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                72,
                Cca::new(0, "C"),
                "M".into(),
                None,
                1,
                MainComm,
                vec![ApplicantIdx(1)],
            ),
        ];

        let result = run(&Pool::new(applicants, positions), &Appeals::new());
        let held = result.positions_of(ApplicantIdx(1));
        assert!(held.contains(&PositionIdx(70)) && held.contains(&PositionIdx(71)));
        assert!(
            !held.contains(&PositionIdx(72)),
            "main rejected: main+block quota full"
        );
    }

    #[test]
    fn appointment_reduces_open_seats() {
        // Cap-1 blockcomm already held by an appointee -> the applicant gets nothing.
        let applicants = vec![Applicant::new(
            1,
            "Ann".into(),
            "a@x".into(),
            vec![PositionIdx(50)],
        )];
        let positions = vec![
            Position::new(
                50,
                Cca::new(0, "C"),
                "B".into(),
                None,
                1,
                PositionType::BlockComm,
                vec![ApplicantIdx(1)],
            )
            .with_appointed(1),
        ];
        let pool = Pool::new(applicants, positions).with_appointments(Appointments::from_iter([
            Appointment {
                applicant: ApplicantIdx(2),
                position: PositionIdx(50),
            },
        ]));

        let result = run(&pool, &Appeals::new());
        assert!(
            result.positions_of(ApplicantIdx(1)).is_empty(),
            "seat already taken by appointee"
        );
    }

    #[test]
    fn appointment_consumes_quota() {
        // Ann already holds 2 main+block via appointments -> a 3rd main is barred.
        let applicants = vec![Applicant::new(
            1,
            "Ann".into(),
            "a@x".into(),
            vec![PositionIdx(60)],
        )];
        let positions = vec![
            Position::new(
                40,
                Cca::new(0, "C"),
                "M1".into(),
                None,
                1,
                PositionType::MainComm,
                vec![],
            )
            .with_appointed(1),
            Position::new(
                41,
                Cca::new(0, "C"),
                "M2".into(),
                None,
                1,
                PositionType::MainComm,
                vec![],
            )
            .with_appointed(1),
            Position::new(
                60,
                Cca::new(0, "C"),
                "M3".into(),
                None,
                1,
                PositionType::MainComm,
                vec![ApplicantIdx(1)],
            ),
        ];
        let pool = Pool::new(applicants, positions).with_appointments(Appointments::from_iter([
            Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(40),
            },
            Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(41),
            },
        ]));

        let result = run(&pool, &Appeals::new());
        assert!(
            !result
                .positions_of(ApplicantIdx(1))
                .contains(&PositionIdx(60)),
            "main+block quota already full from appointments"
        );
    }
}
