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

    use crate::data::mock;
    use crate::models::PositionType::{BlockComm, MainComm, SubComm};
    use std::collections::HashMap;

    #[test]
    fn mock_run_respects_capacity_and_quota() {
        let pool = mock::load();
        let result = run(pool.applicants(), pool.positions(), &Appeals::new());

        // Capacity: no position seats more than its capacity.
        for p in pool.positions() {
            assert!(result.for_position(p.id).len() <= p.capacity, "position {} over capacity", p.id.0);
        }

        // Quota: per applicant, maincomm+blockcomm <= 2, subcomm <= 3, not (maincomm>=1 && subcomm>=2).
        let type_of: HashMap<PositionIdx, _> =
            pool.positions().iter().map(|p| (p.id, p.position_type)).collect();
        for a in pool.applicants() {
            let (mut block, mut main, mut sub) = (0, 0, 0);
            for pid in result.positions_of(a.id) {
                match type_of[pid] {
                    BlockComm => block += 1,
                    MainComm => main += 1,
                    SubComm => sub += 1,
                }
            }
            assert!(main + block <= 2, "applicant {} over main+block quota", a.id.0);
            assert!(sub <= 3, "applicant {} over subcomm quota", a.id.0);
            assert!(!(main >= 1 && sub >= 2), "applicant {} violates main/sub rule", a.id.0);
        }
    }

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
}
