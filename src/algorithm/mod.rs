mod gale_shapley;
mod immediate_acceptance;

use crate::models::{
    Algorithm, Appeals, Applicant, CapacityStore, Ledger, MatchResult, Pool, Position,
};

/// Run the full two-pass allocation.
///
///   Pass 1: Immediate Acceptance over all BlockComm positions.
///   Pass 2: Gale-Shapley over all MainComm + SubComm positions.
pub fn run(applicants: &[Applicant], positions: &[Position], appeals: &Appeals) -> MatchResult {
    // One borrowed view per pass: every applicant, plus the positions routed to
    // that algorithm.
    let ia_pool = Pool::for_algorithm(applicants, positions, Algorithm::ImmediateAcceptance);
    let gs_pool: Pool<'_> = Pool::for_algorithm(applicants, positions, Algorithm::GaleShapley);

    // One ledger and store carry across both passes
    let mut ledger: Ledger = Ledger::new(Algorithm::ImmediateAcceptance);
    let mut store: CapacityStore = CapacityStore::new();

    // Pass 1 — BlockComm
    immediate_acceptance::run(&ia_pool, appeals, &mut store, &mut ledger);

    // Switch the ledger's phase (same log, new stamp) — do NOT finalize yet.
    ledger.enter(Algorithm::GaleShapley, 0);

    // Pass 2 — MainComm + SubComm
    gale_shapley::run(&gs_pool, appeals, &mut store, &mut ledger);

    // Finalize exactly once, at the very end: freeze the log into the result.
    ledger.finish()
}

#[cfg(test)]
mod tests {
    use crate::algorithm::run;
    use crate::models::{Appeals, Applicant, ApplicantIdx, Position, PositionIdx, PositionType};

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
            Position::new(100, 1, "Block".into(), None, 1, PositionType::BlockComm, vec![ApplicantIdx(1)]),
            Position::new(200, 2, "Main".into(), None, 1, PositionType::MainComm, vec![ApplicantIdx(1)]),
        ];

        let result = run(&applicants, &positions, &Appeals::new());

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
            1, "Ann".into(), "a@x".into(),
            vec![PositionIdx(200), PositionIdx(300), PositionIdx(301)],
        )];
        let positions = vec![
            Position::new(200, 1, "Main".into(), None, 1, MainComm, vec![ApplicantIdx(1)]),
            Position::new(300, 1, "Sub A".into(), None, 1, SubComm, vec![ApplicantIdx(1)]),
            Position::new(301, 1, "Sub B".into(), None, 1, SubComm, vec![ApplicantIdx(1)]),
        ];

        // Without an appeal: 301 is rejected on quota.
        let plain = run(&applicants, &positions, &Appeals::new());
        assert!(!plain.positions_of(ApplicantIdx(1)).contains(&PositionIdx(301)));

        // With an appeal on (1, 301): the quota check is bypassed and 301 is seated.
        let mut appeals = Appeals::new();
        appeals.grant(ApplicantIdx(1), PositionIdx(301));
        let appealed = run(&applicants, &positions, &appeals);
        assert!(appealed.positions_of(ApplicantIdx(1)).contains(&PositionIdx(301)));
    }

    #[test]
    fn chair_unranked_applicant_is_not_seated() {
        // The chair ranks only applicant 2; applicant 1 proposes but is never ranked.
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![PositionIdx(200)]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![]),
        ];
        let positions = vec![Position::new(
            200, 1, "Main".into(), None, 1, MainComm, vec![ApplicantIdx(2)],
        )];

        let result = run(&applicants, &positions, &Appeals::new());
        assert!(!result.positions_of(ApplicantIdx(1)).contains(&PositionIdx(200)));
    }

    #[test]
    fn gs_bumps_weaker_held_seat() {
        // GS deferred acceptance: a chair-preferred late proposer displaces the seated.
        // Applicant 2 takes the lone seat first; applicant 1 (chair rank 1) bumps them.
        let applicants = vec![
            // 1 first proposes a seat the chair never ranked them for, then the contested M.
            Applicant::new(1, "Win".into(), "w@x".into(), vec![PositionIdx(40), PositionIdx(30)]),
            Applicant::new(2, "Lose".into(), "l@x".into(), vec![PositionIdx(30)]),
        ];
        let positions = vec![
            Position::new(30, 1, "M".into(), None, 1, MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)]),
            Position::new(40, 1, "N".into(), None, 1, MainComm, vec![ApplicantIdx(2)]), // 1 unranked here
        ];

        let result = run(&applicants, &positions, &Appeals::new());
        assert!(
            result.positions_of(ApplicantIdx(1)).contains(&PositionIdx(30)),
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
            50, 1, "B".into(), None, 1, BlockComm, vec![ApplicantIdx(1), ApplicantIdx(2)],
        )];

        let result = run(&applicants, &positions, &Appeals::new());
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
            Applicant::new(2, "Late".into(), "l@x".into(), vec![PositionIdx(61), PositionIdx(60)]),
        ];
        let positions = vec![
            Position::new(60, 1, "B".into(), None, 1, BlockComm, vec![ApplicantIdx(2), ApplicantIdx(1)]),
            Position::new(61, 1, "X".into(), None, 1, BlockComm, vec![]), // ranks nobody
        ];

        let result = run(&applicants, &positions, &Appeals::new());
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
            1, "Ann".into(), "a@x".into(),
            vec![PositionIdx(70), PositionIdx(71), PositionIdx(72)],
        )];
        let positions = vec![
            Position::new(70, 1, "B1".into(), None, 1, BlockComm, vec![ApplicantIdx(1)]),
            Position::new(71, 2, "B2".into(), None, 1, BlockComm, vec![ApplicantIdx(1)]),
            Position::new(72, 3, "M".into(), None, 1, MainComm, vec![ApplicantIdx(1)]),
        ];

        let result = run(&applicants, &positions, &Appeals::new());
        let held = result.positions_of(ApplicantIdx(1));
        assert!(held.contains(&PositionIdx(70)) && held.contains(&PositionIdx(71)));
        assert!(!held.contains(&PositionIdx(72)), "main rejected: main+block quota full");
    }
}
