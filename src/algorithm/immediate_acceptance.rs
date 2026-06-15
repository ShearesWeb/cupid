use std::collections::{HashMap, HashSet};

use crate::models::{
    Appeals, ApplicantIdx, CapacityStore, Ledger, Roster, PositionIdx, RejectReason,
};

/// Pass 1 — Immediate Acceptance (Boston mechanism), BlockComm positions only.
pub fn run(pool: &Roster, appeals: &Appeals, store: &mut CapacityStore, ledger: &mut Ledger) {
    let mut rank: usize = 0;

    loop {
        // Each applicant proposes to their rank-th preference.
        let mut proposals: HashMap<PositionIdx, HashSet<ApplicantIdx>> = HashMap::new();
        let mut progressed = false;

        for applicant in pool.applicants() {
            let Some(&pid) = applicant.preferences().get(rank) else {
                continue;
            };
            progressed = true;

            let Some(position) = pool.position(pid) else {
                continue;
            };

            let appealed = appeals.contains(applicant.id, pid);
            // Appeal bypass quota; else over-quota = reject.
            if !appealed && !store.can_grant(applicant.id, position.position_type) {
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
            let mut seats_left = position.capacity - ledger.holder_count(*pid);

            // Walk chair ranking (best first) so seats go to top proposers.
            let mut seated: HashSet<ApplicantIdx> = HashSet::new();
            for &cand in position.ranking() {
                if seats_left == 0 {
                    break;
                }
                if proposers.contains(&cand) {
                    let appealed = appeals.contains(cand, *pid);
                    store.grant(cand, position.position_type, appealed);
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
