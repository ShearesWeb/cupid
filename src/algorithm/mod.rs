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
