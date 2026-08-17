use std::collections::HashSet;

use crate::models::{
    Applicant, ApplicantIdx, CapacityStore, Ledger, Position, PositionIdx, Preallocations,
    RejectReason, Roster,
};

/// Pass 2 — Gale-Shapley (applicant-proposing deferred acceptance) over
/// MainComm + SubComm positions.
///
/// Each round, every applicant proposes to their best preference that is
/// still live: in scope, not currently held, not permanently rejected, and
/// fitting their quota RIGHT NOW. Quota-blocked preferences are skipped, not
/// rejected — a later displacement can free the quota, and deferred
/// acceptance must then revisit them. Only position-side verdicts
/// (chair unranked, seat lost to better-ranked applicants) are permanent.
///
/// Preallocated holders were seated before this pass and can never be
/// displaced by a proposal.
///
/// Applicants are swept in id order so identical inputs always produce the
/// identical result, event for event.
pub fn run(
    pool: &Roster,
    preallocations: &Preallocations,
    store: &mut CapacityStore,
    ledger: &mut Ledger,
) {
    let mut applicants: Vec<&Applicant> = pool.applicants().collect();
    applicants.sort_by_key(|a| a.id.0);

    // (applicant, position) pairs settled for good: seat won and later lost,
    // or proposal turned down by the position. Never re-proposed.
    let mut settled: HashSet<(ApplicantIdx, PositionIdx)> = HashSet::new();

    loop {
        let mut progressed = false;

        for applicant in &applicants {
            let Some(position) = target(applicant, pool, store, ledger, &settled) else {
                continue;
            };
            progressed = true;

            match propose(applicant, position, preallocations, store, ledger) {
                Proposal::Seated { displaced } => {
                    if let Some(loser) = displaced {
                        // The bump proves every remaining holder outranks the
                        // loser, so re-proposing could never win the seat back.
                        settled.insert((loser, position.id));
                    }
                }
                Proposal::Rejected => {
                    settled.insert((applicant.id, position.id));
                }
            }
        }

        if !progressed {
            break;
        }
    }

    // Audit trail: preferences that stayed quota-blocked to the very end are
    // rejections the operator should see, even though they were never
    // eligible proposals.
    for applicant in &applicants {
        for &pid in applicant.preferences() {
            let Some(position) = pool.position(pid) else {
                continue;
            };
            if settled.contains(&(applicant.id, pid)) || holds(ledger, applicant.id, pid) {
                continue;
            }
            if !store.can_grant(applicant.id, position.position_type, position.cca.id) {
                ledger.reject(applicant.id, pid, RejectReason::ApplicantCapacityFull);
            }
        }
    }
}

/// The applicant's best preference that is live this round, if any.
fn target<'a>(
    applicant: &Applicant,
    pool: &Roster<'a>,
    store: &CapacityStore,
    ledger: &Ledger,
    settled: &HashSet<(ApplicantIdx, PositionIdx)>,
) -> Option<&'a Position> {
    applicant.preferences().iter().find_map(|&pid| {
        let position = pool.position(pid)?; // out-of-scope (non-GS) preference
        if settled.contains(&(applicant.id, pid)) || holds(ledger, applicant.id, pid) {
            return None;
        }
        // Quota-blocked: skip without settling; a displacement may free it.
        if !store.can_grant(applicant.id, position.position_type, position.cca.id) {
            return None;
        }
        Some(position)
    })
}

fn holds(ledger: &Ledger, applicant: ApplicantIdx, position: PositionIdx) -> bool {
    ledger.holders(position).contains(&applicant)
}

enum Proposal {
    Seated { displaced: Option<ApplicantIdx> },
    Rejected,
}

/// Resolve a single proposal that already passed the quota gate.
fn propose(
    applicant: &Applicant,
    position: &Position,
    preallocations: &Preallocations,
    store: &mut CapacityStore,
    ledger: &mut Ledger,
) -> Proposal {
    if position.rank_of(applicant.id).is_none() {
        ledger.reject(applicant.id, position.id, RejectReason::NotRankedByChair);
        return Proposal::Rejected;
    }

    // Role capacity not reached: tentatively accept.
    if ledger.holder_count(position.id) < position.vacancies() {
        store.grant(applicant.id, position.position_type, position.cca.id);
        ledger.accept(applicant, position);
        return Proposal::Seated { displaced: None };
    }

    // Role capacity is full -> find the weakest currently-held seat.
    // Preallocated holders own their seats outright and are never displaced.
    let applicant_rank = position.rank_of(applicant.id).unwrap_or(usize::MAX);

    let loser = ledger
        .holders(position.id)
        .into_iter()
        .filter(|&h| !preallocations.contains(h, position.id))
        .max_by_key(|&h| position.rank_of(h).unwrap_or(usize::MAX));

    // Every seat is held by an appointment or a preallocation; no one to displace.
    let Some(loser) = loser else {
        ledger.reject(applicant.id, position.id, RejectReason::RoleCapacityFull);
        return Proposal::Rejected;
    };

    let worst_rank = position.rank_of(loser).unwrap_or(usize::MAX);

    // Applicant outranks the weakest held: BUMP.
    if applicant_rank < worst_rank {
        store.revoke(loser, position.position_type, position.cca.id);
        store.grant(applicant.id, position.position_type, position.cca.id);
        ledger.bump(applicant, loser, position);
        return Proposal::Seated { displaced: Some(loser) };
    }

    ledger.reject(applicant.id, position.id, RejectReason::RoleCapacityFull);
    Proposal::Rejected
}
