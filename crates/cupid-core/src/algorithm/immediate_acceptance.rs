use std::collections::{BTreeMap, BTreeSet};

use crate::models::{
    Applicant, ApplicantIdx, CapacityStore, Ledger, PositionIdx, RejectReason, Roster,
};

/// Pass 1 — Immediate Acceptance (Boston mechanism), BlockComm positions only.
/// Preallocated seats are already in the ledger and are skipped, not re-proposed.
///
/// Applicants are swept in id order and positions settled in id order so
/// identical inputs always produce the identical result, event for event.
pub fn run(pool: &Roster, store: &mut CapacityStore, ledger: &mut Ledger) {
    let mut applicants: Vec<&Applicant> = pool.applicants().collect();
    applicants.sort_by_key(|a| a.id.0);

    let mut rank: usize = 0;

    loop {
        // Each applicant proposes to their rank-th preference.
        let mut proposals: BTreeMap<PositionIdx, BTreeSet<ApplicantIdx>> = BTreeMap::new();
        let mut progressed = false;

        for applicant in &applicants {
            let Some(&pid) = applicant.preferences().get(rank) else {
                continue;
            };
            progressed = true;

            let Some(position) = pool.position(pid) else {
                continue;
            };

            // Already seated here by preallocation: nothing to propose.
            if ledger.holders(pid).contains(&applicant.id) {
                continue;
            }

            // Over-quota (type or CCA) = reject.
            if !store.can_grant(applicant.id, position.position_type, position.cca.id) {
                ledger.reject(applicant.id, pid, RejectReason::ApplicantCapacityFull);
                continue;
            }

            proposals.entry(pid).or_default().insert(applicant.id);
        }

        // No rank-th pref left anywhere = done.
        if !progressed {
            break;
        }
        rank += 1;

        // Each position seats proposers in chair's rank order until
        // seats run out. Acceptance is permanent.
        for (pid, proposers) in &proposals {
            let position = pool.position(*pid).unwrap();
            // Preallocations may overfill a position; saturate instead of underflowing.
            let mut seats_left = position.vacancies().saturating_sub(ledger.holder_count(*pid));

            // Walk chair ranking (best first) so seats go to top proposers.
            let mut seated: BTreeSet<ApplicantIdx> = BTreeSet::new();
            for &cand in position.ranking() {
                if seats_left == 0 {
                    break;
                }
                if proposers.contains(&cand) {
                    store.grant(cand, position.position_type, position.cca.id);
                    ledger.accept(pool.applicant(cand).unwrap(), position);
                    seated.insert(cand);
                    seats_left -= 1;
                }
            }

            // Proposers who missed seat: reject.
            for &cand in proposers {
                if seated.contains(&cand) {
                    continue;
                }
                let reason = if position.rank_of(cand).is_none() {
                    RejectReason::NotRankedByChair
                } else {
                    RejectReason::RoleCapacityFull
                };
                ledger.reject(cand, *pid, reason);
            }
        }
    }
}
