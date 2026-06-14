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
    pub fn unmatched(&self, applicants: &[Applicant]) -> Vec<ApplicantIdx> {
        applicants
            .iter()
            .map(|a| a.id)
            .filter(|&id| self.for_applicant(id).next().is_none())
            .collect()
    }

    /// Position id -> empty capacity remaining.
    pub fn unfilled(&self, positions: &[Position]) -> Vec<(PositionIdx, usize)> {
        positions
            .iter()
            .filter_map(|p| {
                let capacity = p.capacity;
                let filled = self.for_position(p.id).len();
                let empty = capacity.saturating_sub(filled);
                (empty > 0).then_some((p.id, empty))
            })
            .collect()
    }
}
