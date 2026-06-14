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
}
