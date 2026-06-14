use crate::models::{Appeals, Applicant, CapacityStore, Ledger, Pool, Position, RejectReason};

/// Pass 2 — Gale-Shapley (deferred acceptance), MainComm + SubComm positions.
pub fn run(pool: &Pool, appeals: &Appeals, store: &mut CapacityStore, ledger: &mut Ledger) {
    // Current rank.
    let mut rank: usize = 0;

    // Each applicant still has untried preferences until every rank is exhausted.
    loop {
        let mut progressed: bool = false;

        for applicant in pool.applicants() {
            let Some(&position_idx) = applicant.preferences().get(rank) else {
                continue;
            };

            // If position is in the pool (GS).
            if let Some(position) = pool.position(position_idx) {
                propose(applicant, position, appeals, store, ledger);
                progressed = true;
            }
        }

        // Move to next rank if any applicant proposed to a position.
        if progressed {
            rank += 1;
        }

        // No applicant had an untried preference this sweep: every rank exhausted.
        if !progressed {
            break;
        }
    }
}

/// Resolve a SINGLE proposal: applicant `a` proposes to position `p`.
fn propose(
    applicant: &Applicant,
    position: &Position,
    appeals: &Appeals,
    store: &mut CapacityStore,
    ledger: &mut Ledger,
) {
    if position.rank_of(applicant.id).is_none() {
        ledger.reject(applicant.id, position.id, RejectReason::NotRankedByChair);
        return;
    }

    let appealed = appeals.contains(applicant.id, position.id);

    // Safe to reject: applicant proposed to more desirable rankings, and quota is full
    if !appealed && !store.can_grant(applicant.id, position.position_type) {
        ledger.reject(
            applicant.id,
            position.id,
            RejectReason::ApplicantCapacityFull,
        );
        return;
    }

    // Role capacity not reached: tentatively accept proposal
    if ledger.holder_count(position.id) < position.capacity {
        store.grant(applicant.id, position.position_type, appealed);
        ledger.accept(applicant, position);
        return;
    }

    // Role capacity is full -> find the weakest currently-held seat (largest rank).
    let applicant_rank = position.rank_of(applicant.id).unwrap_or(usize::MAX);

    let loser = ledger
        .holders(position.id)
        .into_iter()
        .max_by_key(|&h| position.rank_of(h).unwrap_or(usize::MAX))
        .unwrap();

    let worst_rank = position.rank_of(loser).unwrap_or(usize::MAX);

    // Applicant outranks the weakest held: BUMP.
    if applicant_rank < worst_rank {
        store.revoke(
            loser,
            position.position_type,
            appeals.contains(loser, position.id),
        );
        store.grant(applicant.id, position.position_type, appealed);
        ledger.bump(applicant, loser, position);
        return;
    }

    ledger.reject(applicant.id, position.id, RejectReason::RoleCapacityFull);
}
