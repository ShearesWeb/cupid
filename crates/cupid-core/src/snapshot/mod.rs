//! The one immutable read model. `build` projects the corpus plus an optional
//! match run into a serializable Snapshot; every UI query becomes a lookup.
use std::collections::{BTreeSet, HashMap};

use serde::Serialize;

use crate::models::{
    Algorithm, Allocation, ApplicantIdx, CapacityStore, Event, EventKind, MatchResult, Pool,
    PositionIdx, PositionType, Preallocations, RejectReason,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Existing,
    Allocated,
    Preallocated,
    Displaced,
    Quota,
    Noreturn,
    Neutral,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub synced_at: String,
    /// Load-time anomalies (e.g. stale preallocation rows) surfaced to the operator.
    pub warnings: Vec<String>,
    pub ccas: Vec<CcaView>,
    pub positions: Vec<PositionView>,
    pub applicants: Vec<ApplicantView>,
    pub committed: Vec<PairView>,
    pub preallocations: Vec<PreallocationView>,
    pub quota: Vec<QuotaView>,
    pub seats: Vec<SeatsView>,
    pub outcomes: Vec<OutcomeView>,
    pub run: Option<RunView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaView {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionView {
    pub id: i32,
    pub cca_id: i32,
    pub name: String,
    pub r#type: String,
    pub capacity: usize,
    pub chair_rank: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantView {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub prefs: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairView {
    pub applicant_id: i32,
    pub position_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreallocationView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaView {
    pub applicant_id: i32,
    pub main: u8,
    pub block: u8,
    pub sub: u8,
    pub can_add_main: bool,
    pub can_add_block: bool,
    pub can_add_sub: bool,
    pub over: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub applicant_id: i32,
    pub status: Status,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatsView {
    pub position_id: i32,
    pub seated: Vec<SeatView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub status: Status,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunView {
    pub assignments: Vec<AssignmentView>,
    pub events: Vec<EventView>,
    pub unfilled: Vec<UnfilledView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub kind: AssignmentKind,
    pub chair_rank: Option<usize>,
    pub pref_rank: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssignmentKind {
    Allocated,
    Preallocated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub seq: u32,
    pub kind: EventKindView,
    pub by_applicant_id: Option<i32>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKindView {
    Accept,
    Reject,
    Displace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnfilledView {
    pub position_id: i32,
    pub open: usize,
}

fn type_str(t: PositionType) -> &'static str {
    match t {
        PositionType::BlockComm => "block",
        PositionType::MainComm => "main",
        PositionType::SubComm => "sub",
    }
}

pub fn build(
    pool: &Pool,
    preallocations: &Preallocations,
    result: Option<&MatchResult>,
    synced_at: String,
    warnings: Vec<String>,
) -> Snapshot {
    Snapshot {
        synced_at,
        warnings,
        ccas: build_ccas(pool),
        positions: build_positions(pool),
        applicants: build_applicants(pool),
        committed: build_pairs(pool.appointments().iter().map(|a| (a.applicant, a.position))),
        preallocations: build_preallocations(preallocations),
        quota: build_quota(pool, result),
        seats: build_seats(pool, preallocations, result),
        outcomes: build_outcomes(pool, preallocations, result),
        run: result.map(|r| build_run(pool, preallocations, r)),
    }
}

fn build_ccas(pool: &Pool) -> Vec<CcaView> {
    let mut views: Vec<CcaView> = pool
        .ccas()
        .map(|c| CcaView {
            id: c.id.0,
            name: c.name.clone(),
        })
        .collect();
    views.sort_by_key(|v| v.id);
    views
}

fn build_positions(pool: &Pool) -> Vec<PositionView> {
    let mut views: Vec<PositionView> = pool
        .positions()
        .map(|p| PositionView {
            id: p.id.0,
            cca_id: p.cca.id.0,
            name: p.name.clone(),
            r#type: type_str(p.position_type).to_string(),
            capacity: p.capacity,
            chair_rank: p.ranking().iter().map(|a| a.0).collect(),
        })
        .collect();
    views.sort_by_key(|v| v.id);
    views
}

fn build_applicants(pool: &Pool) -> Vec<ApplicantView> {
    let mut views: Vec<ApplicantView> = pool
        .applicants()
        .map(|a| ApplicantView {
            id: a.id.0,
            name: a.name.clone(),
            email: a.email.clone(),
            prefs: a.preferences().iter().map(|p| p.0).collect(),
        })
        .collect();
    views.sort_by_key(|v| v.id);
    views
}

fn build_preallocations(preallocations: &Preallocations) -> Vec<PreallocationView> {
    let mut views: Vec<PreallocationView> = preallocations
        .iter()
        .map(|(a, p)| PreallocationView {
            applicant_id: a.0,
            position_id: p.0,
            note: preallocations.note(a, p).map(str::to_owned),
        })
        .collect();
    views.sort_by_key(|v| (v.applicant_id, v.position_id));
    views
}

fn build_pairs(pairs: impl Iterator<Item = (ApplicantIdx, PositionIdx)>) -> Vec<PairView> {
    let mut views: Vec<PairView> = pairs
        .map(|(a, p)| PairView {
            applicant_id: a.0,
            position_id: p.0,
        })
        .collect();
    views.sort_by_key(|v| (v.applicant_id, v.position_id));
    views
}

fn build_quota(pool: &Pool, result: Option<&MatchResult>) -> Vec<QuotaView> {
    let mut store = CapacityStore::from_pool(pool);
    if let Some(result) = result {
        for alloc in result.all() {
            if let Some(position) = pool.position(alloc.position_id) {
                store.grant(alloc.applicant_id, position.position_type, position.cca.id);
            }
        }
    }
    let mut views: Vec<QuotaView> = pool
        .applicants()
        .map(|a| {
            let counts = store.get(a.id);
            QuotaView {
                applicant_id: a.id.0,
                main: counts.maincomm,
                block: counts.blockcomm,
                sub: counts.subcomm,
                can_add_main: counts.can_add(PositionType::MainComm),
                can_add_block: counts.can_add(PositionType::BlockComm),
                can_add_sub: counts.can_add(PositionType::SubComm),
                over: !counts.within_quota(),
            }
        })
        .collect();
    views.sort_by_key(|v| v.applicant_id);
    views
}

fn build_seats(
    pool: &Pool,
    preallocations: &Preallocations,
    result: Option<&MatchResult>,
) -> Vec<SeatsView> {
    let mut positions: Vec<_> = pool.positions().collect();
    positions.sort_by_key(|p| p.id.0);

    positions
        .into_iter()
        .map(|p| {
            // Ordering guarantee: existing -> allocated -> preallocated.
            let mut seated: Vec<SeatView> = pool
                .appointments()
                .holders(p.id)
                .iter()
                .map(|&applicant| SeatView {
                    applicant_id: applicant.0,
                    status: Status::Existing,
                })
                .collect();

            if let Some(result) = result {
                let mut allocated = Vec::new();
                let mut preallocated = Vec::new();
                for alloc in result.for_position(p.id) {
                    let status = if preallocations.contains(alloc.applicant_id, alloc.position_id)
                    {
                        Status::Preallocated
                    } else {
                        Status::Allocated
                    };
                    let view = SeatView {
                        applicant_id: alloc.applicant_id.0,
                        status,
                    };
                    match status {
                        Status::Preallocated => preallocated.push(view),
                        _ => allocated.push(view),
                    }
                }
                seated.extend(allocated);
                seated.extend(preallocated);
            } else {
                // Pre-run a preallocation is the only forward-looking claim on
                // a seat, so it shares the bar with the existing appointments.
                // Pairs already committed hold their seat as Existing above.
                let held = pool.appointments().holders(p.id).to_vec();
                let mut claimed: Vec<ApplicantIdx> = preallocations
                    .iter()
                    .filter(|&(_, position)| position == p.id)
                    .map(|(applicant, _)| applicant)
                    .filter(|applicant| !held.contains(applicant))
                    .collect();
                claimed.sort();
                seated.extend(claimed.into_iter().map(|applicant| SeatView {
                    applicant_id: applicant.0,
                    status: Status::Preallocated,
                }));
            }

            SeatsView {
                position_id: p.id.0,
                seated,
            }
        })
        .collect()
}

/// The applicant's name, or a placeholder if the id is somehow unknown.
fn name_of(pool: &Pool, id: ApplicantIdx) -> String {
    pool.applicant(id)
        .map(|a| a.name.clone())
        .unwrap_or_else(|| format!("Applicant {}", id.0))
}

fn rank_text(rank: Option<usize>) -> String {
    rank.map_or_else(|| "unranked".to_string(), |r| r.to_string())
}

/// Fixed detail text for a rejection reason (shared by events and outcomes).
// Detail strings render next to a pill or title that already names the
// outcome, so they carry only what the label cannot: who, how many, when.
fn reject_detail(reason: RejectReason) -> &'static str {
    match reason {
        RejectReason::NotRankedByChair => "Chair did not rank this applicant.",
        RejectReason::RoleCapacityFull => "All seats were taken at that point in the run.",
        RejectReason::ApplicantCapacityFull => {
            "Another seat here would exceed their personal quota at that point in the run."
        }
        RejectReason::DisplacedByHigherRank => "Chair preferred others who took the seats.",
    }
}

/// Detail text for a displacement (shared by events and outcomes).
fn displaced_detail(pool: &Pool, by: ApplicantIdx, by_chair_rank: Option<usize>) -> String {
    format!(
        "By {} (chair-rank {}).",
        name_of(pool, by),
        rank_text(by_chair_rank)
    )
}

/// Detail text for a successful allocation.
fn accept_detail(alloc: &Allocation) -> String {
    // A preallocation seats in its own pass: ranks played no part.
    if alloc.accepted_at.algorithm == Algorithm::Preallocation {
        return "Seated before matching by operator preallocation.".to_string();
    }
    format!(
        "Allocated at chair-rank {} (their preference #{}).",
        rank_text(alloc.chair_rank),
        rank_text(alloc.applicant_rank)
    )
}

/// Outcome classification, pre-run and run-aware. Precedence per pair:
/// committed > preallocated assignment > allocated assignment > last event
/// (by seq) > didn't-rank-back > neutral.
fn build_outcomes(
    pool: &Pool,
    preallocations: &Preallocations,
    result: Option<&MatchResult>,
) -> Vec<OutcomeView> {
    // Pair universe: every applicant's preferences, every position's chair
    // ranking, every committed appointment, every preallocation, and (with a
    // run) every assignment and event pair, deduped and sorted.
    let mut pairs: BTreeSet<(i32, i32)> = BTreeSet::new();
    for a in pool.applicants() {
        for &p in a.preferences() {
            pairs.insert((a.id.0, p.0));
        }
    }
    for p in pool.positions() {
        for &a in p.ranking() {
            pairs.insert((a.0, p.id.0));
        }
    }
    for appointment in pool.appointments().iter() {
        pairs.insert((appointment.applicant.0, appointment.position.0));
    }
    for (a, p) in preallocations.iter() {
        pairs.insert((a.0, p.0));
    }
    if let Some(result) = result {
        for alloc in result.all() {
            pairs.insert((alloc.applicant_id.0, alloc.position_id.0));
        }
        for event in result.events() {
            pairs.insert((event.applicant_id.0, event.position_id.0));
        }
    }

    // Index assignments and the last (highest-seq) event per pair, once.
    let assignment_by_pair: HashMap<(i32, i32), &Allocation> = result
        .map(|r| {
            r.all()
                .map(|a| ((a.applicant_id.0, a.position_id.0), a))
                .collect()
        })
        .unwrap_or_default();
    let mut last_event_by_pair: HashMap<(i32, i32), &Event> = HashMap::new();
    if let Some(result) = result {
        for event in result.events() {
            let key = (event.applicant_id.0, event.position_id.0);
            last_event_by_pair
                .entry(key)
                .and_modify(|current| {
                    if event.step.seq > current.step.seq {
                        *current = event;
                    }
                })
                .or_insert(event);
        }
    }

    pairs
        .into_iter()
        .map(|(aid, pid)| {
            let applicant_id = ApplicantIdx(aid);
            let position_id = PositionIdx(pid);

            // 1. Already a committed appointment.
            if pool
                .appointments()
                .holders(position_id)
                .contains(&applicant_id)
            {
                return OutcomeView {
                    applicant_id: aid,
                    position_id: pid,
                    status: Status::Existing,
                    label: "Appointment".into(),
                    detail: String::new(),
                };
            }

            if result.is_some() {
                // 2/3. Assignment this run, preallocated or plain allocation.
                if assignment_by_pair.contains_key(&(aid, pid)) {
                    if preallocations.contains(applicant_id, position_id) {
                        return OutcomeView {
                            applicant_id: aid,
                            position_id: pid,
                            status: Status::Preallocated,
                            label: "Preallocated".into(),
                            detail: String::new(),
                        };
                    }
                    return OutcomeView {
                        applicant_id: aid,
                        position_id: pid,
                        status: Status::Allocated,
                        label: "Allocated".into(),
                        detail: String::new(),
                    };
                }

                // 4. Last event for the pair.
                if let Some(&event) = last_event_by_pair.get(&(aid, pid)) {
                    let (status, label, detail) = match event.kind {
                        EventKind::Displaced { by, by_chair_rank } => (
                            Status::Displaced,
                            "Displaced",
                            displaced_detail(pool, by, by_chair_rank),
                        ),
                        EventKind::Rejected {
                            reason: RejectReason::ApplicantCapacityFull,
                        } => (
                            Status::Quota,
                            "Quota full",
                            reject_detail(RejectReason::ApplicantCapacityFull).to_string(),
                        ),
                        EventKind::Rejected {
                            reason: RejectReason::NotRankedByChair,
                        } => (
                            Status::Neutral,
                            "Not ranked",
                            reject_detail(RejectReason::NotRankedByChair).to_string(),
                        ),
                        EventKind::Rejected { reason } => {
                            (Status::Neutral, "Position full", reject_detail(reason).to_string())
                        }
                    };
                    return OutcomeView {
                        applicant_id: aid,
                        position_id: pid,
                        status,
                        label: label.into(),
                        detail,
                    };
                }
            }

            // 5. Applicant never ranked this position back. A preallocation
            // on the pair bypasses this: it exists precisely to grant the
            // seat without reciprocal ranking, so it shouldn't get flagged
            // as a no-return.
            let applicant = pool.applicant(applicant_id);
            let ranked_back =
                applicant.is_some_and(|a| a.preferences().contains(&position_id));
            if !ranked_back && !preallocations.contains(applicant_id, position_id) {
                let prefs_len = applicant.map_or(0, |a| a.preferences().len());
                return OutcomeView {
                    applicant_id: aid,
                    position_id: pid,
                    status: Status::Noreturn,
                    label: "Didn't rank back".into(),
                    detail: if prefs_len == 0 {
                        "Submitted no preferences at all.".to_string()
                    } else {
                        format!("Not among their {prefs_len} ranked preferences.")
                    },
                };
            }

            // 6. Otherwise: neutral, phrased pre-run vs. post-run.
            if result.is_some() {
                OutcomeView {
                    applicant_id: aid,
                    position_id: pid,
                    status: Status::Neutral,
                    label: "Not allocated".into(),
                    detail: String::new(),
                }
            } else {
                OutcomeView {
                    applicant_id: aid,
                    position_id: pid,
                    status: Status::Neutral,
                    label: "—".into(),
                    detail: "Run matching to see the outcome.".into(),
                }
            }
        })
        .collect()
}

fn build_run(pool: &Pool, preallocations: &Preallocations, result: &MatchResult) -> RunView {
    let mut assignments: Vec<AssignmentView> = result
        .all()
        .map(|alloc| {
            let kind = if preallocations.contains(alloc.applicant_id, alloc.position_id) {
                AssignmentKind::Preallocated
            } else {
                AssignmentKind::Allocated
            };
            AssignmentView {
                applicant_id: alloc.applicant_id.0,
                position_id: alloc.position_id.0,
                kind,
                chair_rank: alloc.chair_rank,
                pref_rank: alloc.applicant_rank,
            }
        })
        .collect();
    assignments.sort_by_key(|a| (a.position_id, a.applicant_id));

    let mut events: Vec<EventView> = Vec::new();
    for event in result.events() {
        let (kind, by_applicant_id, detail) = match event.kind {
            EventKind::Rejected { reason } => {
                (EventKindView::Reject, None, reject_detail(reason).to_string())
            }
            EventKind::Displaced { by, by_chair_rank } => (
                EventKindView::Displace,
                Some(by.0),
                displaced_detail(pool, by, by_chair_rank),
            ),
        };
        events.push(EventView {
            applicant_id: event.applicant_id.0,
            position_id: event.position_id.0,
            seq: event.step.seq,
            kind,
            by_applicant_id,
            detail,
        });
    }
    for alloc in result.all() {
        events.push(EventView {
            applicant_id: alloc.applicant_id.0,
            position_id: alloc.position_id.0,
            seq: alloc.accepted_at.seq,
            kind: EventKindView::Accept,
            by_applicant_id: None,
            detail: accept_detail(alloc),
        });
    }
    events.sort_by_key(|e| e.seq);

    let mut unfilled: Vec<UnfilledView> = result
        .unfilled(pool.positions())
        .into_iter()
        .map(|(position_id, open)| UnfilledView {
            position_id: position_id.0,
            open,
        })
        .collect();
    unfilled.sort_by_key(|u| u.position_id);

    RunView {
        assignments,
        events,
        unfilled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn fixture() -> (Pool, Preallocations) {
        // Chess(1): Head(10, main, cap 2, chair ranks [1,2]); Sub(11, sub, cap 1, ranks [2])
        // Choir(2): Lead(20, block, cap 1, ranks [])
        let positions = vec![
            Position::new(10, Cca::new(1, "Chess"), "Head".into(), None, 2, PositionType::MainComm,
                vec![ApplicantIdx(1), ApplicantIdx(2)]),
            Position::new(11, Cca::new(1, "Chess"), "Sub".into(), None, 1, PositionType::SubComm,
                vec![ApplicantIdx(2)]),
            Position::new(20, Cca::new(2, "Choir"), "Lead".into(), None, 1, PositionType::BlockComm, vec![])
                .with_appointed(1),
        ];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(10), PositionIdx(11)]),
            Applicant::new(3, "Cid".into(), "cid@x".into(), vec![]),
        ];
        let pool = Pool::new(applicants, positions).with_appointments(Appointments::from_iter([
            Appointment { applicant: ApplicantIdx(3), position: PositionIdx(20) },
        ]));
        (pool, Preallocations::new())
    }

    #[test]
    fn corpus_views_are_sorted_and_complete() {
        let (pool, preallocations) = fixture();
        let s = build(&pool, &preallocations, None, "2026-07-07T00:00:00Z".into(), vec![]);
        assert_eq!(s.ccas.iter().map(|c| c.id).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(s.positions.iter().map(|p| p.id).collect::<Vec<_>>(), vec![10, 11, 20]);
        assert_eq!(s.positions[0].r#type, "main");
        assert_eq!(s.positions[2].r#type, "block");
        assert_eq!(s.positions[0].chair_rank, vec![1, 2]);
        assert_eq!(s.applicants.iter().map(|a| a.id).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(s.committed, vec![PairView { applicant_id: 3, position_id: 20 }]);
        assert!(s.run.is_none());
    }

    #[test]
    fn quota_reflects_committed_holdings_pre_run() {
        let (pool, preallocations) = fixture();
        let s = build(&pool, &preallocations, None, "t".into(), vec![]);
        let q3 = s.quota.iter().find(|q| q.applicant_id == 3).unwrap();
        assert_eq!((q3.main, q3.block, q3.sub), (0, 1, 0), "appointment to block position 20");
        assert!(q3.can_add_main && q3.can_add_block && q3.can_add_sub);
        assert!(!q3.over);
        let q1 = s.quota.iter().find(|q| q.applicant_id == 1).unwrap();
        assert_eq!((q1.main, q1.block, q1.sub), (0, 0, 0));
    }

    #[test]
    fn seats_pre_run_show_existing_appointments() {
        let (pool, preallocations) = fixture();
        let s = build(&pool, &preallocations, None, "t".into(), vec![]);
        let seat20 = s.seats.iter().find(|x| x.position_id == 20).unwrap();
        assert_eq!(seat20.seated.len(), 1);
        assert_eq!(seat20.seated[0].applicant_id, 3);
        assert_eq!(seat20.seated[0].status, Status::Existing);
        assert!(s.seats.iter().find(|x| x.position_id == 10).unwrap().seated.is_empty());
    }

    #[test]
    fn seats_pre_run_include_preallocated_claims() {
        // A preallocation is a promised seat, so it fills the capacity bar
        // before any run, behind the appointments already committed.
        let (pool, mut preallocations) = fixture();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        preallocations.grant(ApplicantIdx(3), PositionIdx(20)); // already appointed there
        let s = build(&pool, &preallocations, None, "t".into(), vec![]);

        let seat10 = s.seats.iter().find(|x| x.position_id == 10).unwrap();
        assert_eq!(seat10.seated.len(), 1);
        assert_eq!(seat10.seated[0].applicant_id, 1);
        assert_eq!(seat10.seated[0].status, Status::Preallocated);

        let seat20 = s.seats.iter().find(|x| x.position_id == 20).unwrap();
        assert_eq!(
            seat20.seated.len(),
            1,
            "a committed pair holds one seat, not two"
        );
        assert_eq!(seat20.seated[0].status, Status::Existing);
    }

    #[test]
    fn outcomes_pre_run_cover_existing_noreturn_neutral() {
        let (pool, preallocations) = fixture();
        let s = build(&pool, &preallocations, None, "t".into(), vec![]);
        let get = |a: i32, p: i32| s.outcomes.iter().find(|o| o.applicant_id == a && o.position_id == p);
        assert_eq!(get(3, 20).unwrap().status, Status::Existing);
        // Chair of Sub(11) ranked Ben, Ben ranked back: neutral pre-run.
        assert_eq!(get(2, 11).unwrap().status, Status::Neutral);
        // Add a chair-ranked applicant who did not rank back to assert Noreturn:
        // position 10 ranks Ann(1) and Ben(2); both ranked back, so extend the
        // fixture inline for this case.
        let positions = vec![Position::new(30, Cca::new(1, "Chess"), "X".into(), None, 1,
            PositionType::MainComm, vec![ApplicantIdx(1)])];
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let pool2 = Pool::new(applicants, positions);
        let s2 = build(&pool2, &Preallocations::new(), None, "t".into(), vec![]);
        let o = s2.outcomes.iter().find(|o| o.applicant_id == 1 && o.position_id == 30).unwrap();
        assert_eq!(o.status, Status::Noreturn);
        assert_eq!(o.label, "Didn't rank back");
    }

    fn run_fixture() -> (Pool, Preallocations, MatchResult) {
        // One main seat (10), chair ranks Ann then Ben, both want it:
        // Ben proposes, seated, then displaced by Ann. Cid is preallocated into Sub(11).
        let positions = vec![
            Position::new(10, Cca::new(1, "Chess"), "Head".into(), None, 1, PositionType::MainComm,
                vec![ApplicantIdx(1), ApplicantIdx(2)]),
            Position::new(11, Cca::new(1, "Chess"), "Sub".into(), None, 2, PositionType::SubComm,
                vec![ApplicantIdx(3)]),
        ];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(10)]),
            Applicant::new(3, "Cid".into(), "cid@x".into(), vec![PositionIdx(11)]),
        ];
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(3), PositionIdx(11));

        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[1], &positions[0]);                // seq 0: Ben seated
        ledger.bump(&applicants[0], ApplicantIdx(2), &positions[0]); // seq 1 displace + seq 2 accept
        ledger.accept(&applicants[2], &positions[1]);                // seq 3: Cid (preallocated pair)
        let result = ledger.finish();

        let pool = Pool::new(applicants, positions);
        (pool, preallocations, result)
    }

    #[test]
    fn assignments_carry_kind_and_ranks() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let run = s.run.unwrap();
        let ann = run.assignments.iter().find(|a| a.applicant_id == 1).unwrap();
        assert!(matches!(ann.kind, AssignmentKind::Allocated));
        assert_eq!(ann.chair_rank, Some(1));
        assert_eq!(ann.pref_rank, Some(1));
        let cid = run.assignments.iter().find(|a| a.applicant_id == 3).unwrap();
        assert!(matches!(cid.kind, AssignmentKind::Preallocated));
    }

    #[test]
    fn events_are_seq_ordered_and_include_accepts() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let run = s.run.unwrap();
        let seqs: Vec<_> = run.events.iter().map(|e| e.seq).collect();
        let mut sorted = seqs.clone();
        sorted.sort();
        assert_eq!(seqs, sorted, "global seq order");
        assert!(run.events.iter().any(|e| matches!(e.kind, EventKindView::Accept)));
        let displace = run.events.iter().find(|e| matches!(e.kind, EventKindView::Displace)).unwrap();
        assert_eq!(displace.applicant_id, 2, "Ben was displaced");
        assert_eq!(displace.by_applicant_id, Some(1));
        assert!(displace.detail.contains("Ann"), "detail names the displacer: {}", displace.detail);
    }

    #[test]
    fn outcomes_with_run_classify_all_statuses() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let get = |a: i32, p: i32| s.outcomes.iter().find(|o| o.applicant_id == a && o.position_id == p).unwrap();
        assert_eq!(get(1, 10).status, Status::Allocated);
        assert_eq!(get(3, 11).status, Status::Preallocated);
        assert_eq!(get(2, 10).status, Status::Displaced);
    }

    #[test]
    fn outcome_details_never_repeat_their_label() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let get = |a: i32, p: i32| s.outcomes.iter().find(|o| o.applicant_id == a && o.position_id == p).unwrap();
        // The pill already says allocated/preallocated: no sub-text to repeat it.
        assert_eq!(get(1, 10).detail, "");
        assert_eq!(get(3, 11).detail, "");
        // Displacement detail adds the who, not the what.
        assert_eq!(get(2, 10).detail, "By Ann (chair-rank 1).");
    }

    #[test]
    fn noreturn_detail_reports_preference_usage_only() {
        let positions = vec![Position::new(30, Cca::new(1, "Chess"), "X".into(), None, 1,
            PositionType::MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)])];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(99), PositionIdx(98)]),
        ];
        let pool = Pool::new(applicants, positions);
        let s = build(&pool, &Preallocations::new(), None, "t".into(), vec![]);
        let get = |a: i32| s.outcomes.iter().find(|o| o.applicant_id == a && o.position_id == 30).unwrap();
        assert_eq!(get(1).detail, "Submitted no preferences at all.");
        assert_eq!(get(2).detail, "Not among their 2 ranked preferences.");
    }

    #[test]
    fn preallocation_accept_events_do_not_speak_of_ranks() {
        // A real run seats preallocations in their own pass; the event must
        // say so instead of reporting meaningless chair/preference ranks.
        let positions = vec![Position::new(10, Cca::new(1, "Chess"), "Head".into(), None, 1,
            PositionType::MainComm, vec![])];
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let pool = Pool::new(applicants, positions);
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));

        let result = crate::algorithm::run(&pool, &preallocations);
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let run = s.run.unwrap();
        let accept = run.events.iter().find(|e| matches!(e.kind, EventKindView::Accept)).unwrap();
        assert_eq!(accept.detail, "Seated before matching by operator preallocation.");
    }

    #[test]
    fn unfilled_reports_open_seats() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let run = s.run.unwrap();
        assert_eq!(run.unfilled, vec![UnfilledView { position_id: 11, open: 1 }]);
    }

    #[test]
    fn preallocation_only_pair_gets_an_outcome() {
        // Applicant never ranked the position, chair never ranked the
        // applicant: only a preallocation links the pair. Pre-run, it must
        // still surface as a neutral, unclassified outcome rather than vanish.
        let position = Position::new(
            40,
            Cca::new(1, "Chess"),
            "Ghost".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        );
        let applicant = Applicant::new(4, "Dee".into(), "dee@x".into(), vec![]);
        let pool = Pool::new(vec![applicant], vec![position]);
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(4), PositionIdx(40));

        let s = build(&pool, &preallocations, None, "t".into(), vec![]);
        let o = s
            .outcomes
            .iter()
            .find(|o| o.applicant_id == 4 && o.position_id == 40)
            .unwrap_or_else(|| panic!("preallocation-only pair (4, 40) missing from outcomes"));
        assert_eq!(o.status, Status::Neutral);
        assert_eq!(o.label, "—");
        assert_eq!(o.detail, "Run matching to see the outcome.");

        // With a run, a preallocated+assigned pair classifies as Preallocated.
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "t".into(), vec![]);
        let cid = s
            .outcomes
            .iter()
            .find(|o| o.applicant_id == 3 && o.position_id == 11)
            .unwrap();
        assert_eq!(cid.status, Status::Preallocated);
    }

    #[test]
    fn snapshot_serializes_camel_case_kebab_status() {
        let (pool, preallocations, result) = run_fixture();
        let s = build(&pool, &preallocations, Some(&result), "2026-07-07T00:00:00Z".into(), vec![]);
        let json = serde_json::to_value(&s).unwrap();
        assert!(json.get("syncedAt").is_some());
        assert_eq!(json["positions"][0]["type"], "main");
        assert_eq!(json["outcomes"].as_array().unwrap().iter()
            .find(|o| o["applicantId"] == 2 && o["positionId"] == 10).unwrap()["status"], "displaced");

        // Wire contract for the preallocation rename.
        assert!(json.get("preallocations").is_some(), "field renamed from appeals");
        assert!(json.get("appeals").is_none());
        let cid = json["run"]["assignments"].as_array().unwrap().iter()
            .find(|a| a["applicantId"] == 3 && a["positionId"] == 11).unwrap();
        assert_eq!(cid["kind"], "preallocated");
        assert!(json["quota"][0].get("appealed").is_none(), "quota bucket removed");
    }
}
