use std::collections::HashMap;

use super::applicant::{Applicant, ApplicantIdx};
use super::ledger::Step;
use super::position::{Position, PositionIdx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    ImmediateAcceptance,
    GaleShapley,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Chair never ranked this applicant.
    NotRankedByChair,
    /// Role capacity is full.
    RoleCapacityFull,
    /// Applicant can't take more positions.
    ApplicantCapacityFull,
    /// Chair preferred others who took the seats.
    DisplacedByHigherRank,
}

/// A successful assignment: `applicant_id` holds `position_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub applicant_id: ApplicantIdx,
    pub position_id: PositionIdx,
    /// 1-indexed applicant preference rank for this position.
    pub applicant_rank: Option<usize>,
    /// 1-indexed chair rank, or `None` if the chair never ranked this applicant.
    pub chair_rank: Option<usize>,
    pub accepted_at: Step,
}

/// Audit trail of events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub applicant_id: ApplicantIdx,
    pub position_id: PositionIdx,
    pub step: Step,
    pub kind: EventKind,
}

/// What kind of non-allocation event this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Proposal turned down outright.
    Rejected { reason: RejectReason },
    /// Lost a tentative seat to a higher-ranked applicant (GS).
    Displaced {
        by: ApplicantIdx,
        /// The displacing applicant's 1-indexed chair rank. `None` only if the
        /// chair never ranked them (which cannot happen for GS).
        by_chair_rank: Option<usize>,
    },
}

/// The settled allocations plus the audit trail. Read-only: produced by `Ledger::finish`.
#[derive(Debug, Default)]
pub struct MatchResult {
    by_position: HashMap<PositionIdx, Vec<Allocation>>,
    by_applicant: HashMap<ApplicantIdx, Vec<PositionIdx>>,
    events: HashMap<(ApplicantIdx, PositionIdx), Vec<Event>>,
}

impl MatchResult {
    pub fn new(
        by_position: HashMap<PositionIdx, Vec<Allocation>>,
        by_applicant: HashMap<ApplicantIdx, Vec<PositionIdx>>,
        events: HashMap<(ApplicantIdx, PositionIdx), Vec<Event>>,
    ) -> Self {
        Self {
            by_position,
            by_applicant,
            events,
        }
    }

    /// Every settled assignment, flattened across positions.
    pub fn all(&self) -> impl Iterator<Item = &Allocation> {
        self.by_position.values().flatten()
    }

    /// Assignments for one position.
    pub fn for_position(&self, position_id: PositionIdx) -> &[Allocation] {
        self.by_position
            .get(&position_id)
            .map_or(&[][..], Vec::as_slice)
    }

    /// Positions currently held by one applicant. O(1) lookup.
    pub fn positions_of(&self, applicant_id: ApplicantIdx) -> &[PositionIdx] {
        self.by_applicant
            .get(&applicant_id)
            .map_or(&[][..], Vec::as_slice)
    }

    /// Assignments for one applicant. O(positions held) via the `by_applicant` index.
    pub fn for_applicant(&self, applicant_id: ApplicantIdx) -> impl Iterator<Item = &Allocation> {
        self.positions_of(applicant_id)
            .iter()
            .flat_map(move |&pid| self.for_position(pid))
            .filter(move |a| a.applicant_id == applicant_id)
    }

    /// The full event ledger for one `(applicant, position)` pairing, in recorded order.
    pub fn history(
        &self,
        applicant_id: ApplicantIdx,
        position_id: PositionIdx,
    ) -> impl Iterator<Item = &Event> {
        self.events
            .get(&(applicant_id, position_id))
            .into_iter()
            .flatten()
    }

    /// Applicant ids that ended holding no positions.
    pub fn unmatched<'a>(
        &self,
        applicants: impl Iterator<Item = &'a Applicant>,
    ) -> Vec<ApplicantIdx> {
        applicants
            .map(|a| a.id)
            .filter(|&id| self.for_applicant(id).next().is_none())
            .collect()
    }

    /// Position id -> empty capacity remaining.
    pub fn unfilled<'a>(
        &self,
        positions: impl Iterator<Item = &'a Position>,
    ) -> Vec<(PositionIdx, usize)> {
        positions
            .filter_map(|p| {
                let filled = self.for_position(p.id).len();
                let empty = p.vacancies().saturating_sub(filled);
                (empty > 0).then_some((p.id, empty))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Ledger, PositionType};

    fn applicant(id: i32, prefs: &[i32]) -> Applicant {
        Applicant::new(
            id,
            format!("A{id}"),
            format!("a{id}@x"),
            prefs.iter().map(|&p| PositionIdx(p)).collect(),
        )
    }

    fn position(id: i32, cap: usize, ranking: &[i32]) -> Position {
        Position::new(
            id,
            "C".into(),
            format!("P{id}"),
            None,
            cap,
            PositionType::MainComm,
            ranking.iter().map(|&a| ApplicantIdx(a)).collect(),
        )
    }

    /// Applicant 1 holds both positions; applicant 2 holds nothing. M is full, S has a free seat.
    fn sample() -> (Vec<Applicant>, Vec<Position>, MatchResult) {
        let applicants = vec![applicant(1, &[10, 20]), applicant(2, &[])];
        let positions = vec![position(10, 1, &[1]), position(20, 2, &[1])];

        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[0], &positions[1]);
        (applicants, positions, ledger.finish())
    }

    #[test]
    fn for_applicant_collects_every_seat() {
        let (_, _, result) = sample();
        let held: Vec<_> = result
            .for_applicant(ApplicantIdx(1))
            .map(|a| a.position_id)
            .collect();
        assert_eq!(held.len(), 2);
        assert!(held.contains(&PositionIdx(10)) && held.contains(&PositionIdx(20)));
        assert_eq!(result.for_applicant(ApplicantIdx(2)).count(), 0);
    }

    #[test]
    fn all_flattens_across_positions() {
        let (_, _, result) = sample();
        assert_eq!(result.all().count(), 2);
    }

    #[test]
    fn unmatched_lists_applicants_with_no_seat() {
        let (applicants, _, result) = sample();
        assert_eq!(result.unmatched(applicants.iter()), vec![ApplicantIdx(2)]);
    }

    #[test]
    fn unfilled_reports_only_positions_with_room() {
        let (_, positions, result) = sample();
        // M (cap 1) is full and omitted; S (cap 2) has one empty seat.
        assert_eq!(result.unfilled(positions.iter()), vec![(PositionIdx(20), 1)]);
    }

    #[test]
    fn unfilled_counts_against_vacancies_not_capacity() {
        // P30 has cap 1 but its only seat is taken by an appointee (not a matched
        // holder), so it has zero vacancies and must NOT report as unfilled.
        let positions = [position(30, 1, &[]).with_appointed(1)];
        let result = MatchResult::default();
        assert!(
            result.unfilled(positions.iter()).is_empty(),
            "fully-appointed position has no open seats"
        );
    }

    #[test]
    fn empty_result_queries_are_safe() {
        let result = MatchResult::default();
        assert_eq!(result.for_position(PositionIdx(1)), &[]);
        assert!(result.positions_of(ApplicantIdx(1)).is_empty());
        assert_eq!(result.all().count(), 0);
    }
}
