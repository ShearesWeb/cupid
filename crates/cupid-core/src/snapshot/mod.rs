//! The one immutable read model. `build` projects the corpus plus an optional
//! match run into a serializable Snapshot; every UI query becomes a lookup.
use std::collections::BTreeSet;

use serde::Serialize;

use crate::models::{
    Appeals, ApplicantIdx, CapacityStore, MatchResult, Pool, PositionIdx, PositionType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Existing,
    Allocated,
    Appealed,
    Displaced,
    Quota,
    Noreturn,
    Neutral,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub synced_at: String,
    pub ccas: Vec<CcaView>,
    pub positions: Vec<PositionView>,
    pub applicants: Vec<ApplicantView>,
    pub committed: Vec<PairView>,
    pub appeals: Vec<PairView>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaView {
    pub applicant_id: i32,
    pub main: u8,
    pub block: u8,
    pub sub: u8,
    pub appealed: u8,
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

/// Task 5 fills these in from a real `MatchResult`; Task 4 only needs the
/// shape to exist so `Snapshot` compiles with `run: None`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunView {
    pub assignments: Vec<AssignmentView>,
    pub events: Vec<EventView>,
    pub unfilled: Vec<UnfilledView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub applicant_rank: Option<usize>,
    pub chair_rank: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub applicant_id: i32,
    pub position_id: i32,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnfilledView {
    pub position_id: i32,
    pub empty: usize,
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
    appeals: &Appeals,
    result: Option<&MatchResult>,
    synced_at: String,
) -> Snapshot {
    Snapshot {
        synced_at,
        ccas: build_ccas(pool),
        positions: build_positions(pool),
        applicants: build_applicants(pool),
        committed: build_pairs(pool.appointments().iter().map(|a| (a.applicant, a.position))),
        appeals: build_pairs(appeals.iter()),
        quota: build_quota(pool, appeals, result),
        seats: build_seats(pool, appeals, result),
        outcomes: build_outcomes(pool, appeals, result),
        run: result.map(|r| build_run(pool, appeals, r)),
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

fn build_quota(pool: &Pool, appeals: &Appeals, result: Option<&MatchResult>) -> Vec<QuotaView> {
    let mut store = CapacityStore::from_pool(pool);
    if let Some(result) = result {
        for alloc in result.all() {
            if let Some(position) = pool.position(alloc.position_id) {
                store.grant(
                    alloc.applicant_id,
                    position.position_type,
                    appeals.contains(alloc.applicant_id, alloc.position_id),
                );
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
                appealed: counts.appealed,
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

fn build_seats(pool: &Pool, appeals: &Appeals, result: Option<&MatchResult>) -> Vec<SeatsView> {
    let mut positions: Vec<_> = pool.positions().collect();
    positions.sort_by_key(|p| p.id.0);

    positions
        .into_iter()
        .map(|p| {
            // Ordering guarantee: existing -> allocated -> appealed.
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
                let mut appealed = Vec::new();
                for alloc in result.for_position(p.id) {
                    let status = if appeals.contains(alloc.applicant_id, alloc.position_id) {
                        Status::Appealed
                    } else {
                        Status::Allocated
                    };
                    let view = SeatView {
                        applicant_id: alloc.applicant_id.0,
                        status,
                    };
                    match status {
                        Status::Appealed => appealed.push(view),
                        _ => allocated.push(view),
                    }
                }
                seated.extend(allocated);
                seated.extend(appealed);
            }

            SeatsView {
                position_id: p.id.0,
                seated,
            }
        })
        .collect()
}

/// Pre-run outcome classification. Task 5 adds the run-aware arms (allocated,
/// appealed, displaced, quota-blocked) once `result` carries real allocations.
fn build_outcomes(
    pool: &Pool,
    _appeals: &Appeals,
    _result: Option<&MatchResult>,
) -> Vec<OutcomeView> {
    // Pair universe: every applicant's preferences, every position's chair
    // ranking, and every committed appointment, deduped and sorted.
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

    pairs
        .into_iter()
        .map(|(aid, pid)| {
            let applicant_id = ApplicantIdx(aid);
            let position_id = PositionIdx(pid);

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
                    detail: "Already a committed appointment.".into(),
                };
            }

            let applicant = pool.applicant(applicant_id);
            let ranked_back =
                applicant.is_some_and(|a| a.preferences().contains(&position_id));
            if !ranked_back {
                let prefs_len = applicant.map_or(0, |a| a.preferences().len());
                return OutcomeView {
                    applicant_id: aid,
                    position_id: pid,
                    status: Status::Noreturn,
                    label: "Didn't rank back".into(),
                    detail: format!(
                        "Didn't list this position ({prefs_len} preference slots used)."
                    ),
                };
            }

            OutcomeView {
                applicant_id: aid,
                position_id: pid,
                status: Status::Neutral,
                label: "—".into(),
                detail: "Run matching to see the outcome.".into(),
            }
        })
        .collect()
}

fn build_run(_pool: &Pool, _appeals: &Appeals, _result: &MatchResult) -> RunView {
    RunView {
        assignments: vec![],
        events: vec![],
        unfilled: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn fixture() -> (Pool, Appeals) {
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
        (pool, Appeals::new())
    }

    #[test]
    fn corpus_views_are_sorted_and_complete() {
        let (pool, appeals) = fixture();
        let s = build(&pool, &appeals, None, "2026-07-07T00:00:00Z".into());
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
        let (pool, appeals) = fixture();
        let s = build(&pool, &appeals, None, "t".into());
        let q3 = s.quota.iter().find(|q| q.applicant_id == 3).unwrap();
        assert_eq!((q3.main, q3.block, q3.sub), (0, 1, 0), "appointment to block position 20");
        assert!(q3.can_add_main && q3.can_add_block && q3.can_add_sub);
        assert!(!q3.over);
        let q1 = s.quota.iter().find(|q| q.applicant_id == 1).unwrap();
        assert_eq!((q1.main, q1.block, q1.sub), (0, 0, 0));
    }

    #[test]
    fn seats_pre_run_show_existing_only() {
        let (pool, appeals) = fixture();
        let s = build(&pool, &appeals, None, "t".into());
        let seat20 = s.seats.iter().find(|x| x.position_id == 20).unwrap();
        assert_eq!(seat20.seated.len(), 1);
        assert_eq!(seat20.seated[0].applicant_id, 3);
        assert_eq!(seat20.seated[0].status, Status::Existing);
        assert!(s.seats.iter().find(|x| x.position_id == 10).unwrap().seated.is_empty());
    }

    #[test]
    fn outcomes_pre_run_cover_existing_noreturn_neutral() {
        let (pool, appeals) = fixture();
        let s = build(&pool, &appeals, None, "t".into());
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
        let s2 = build(&pool2, &Appeals::new(), None, "t".into());
        let o = s2.outcomes.iter().find(|o| o.applicant_id == 1 && o.position_id == 30).unwrap();
        assert_eq!(o.status, Status::Noreturn);
        assert_eq!(o.label, "Didn't rank back");
    }
}
