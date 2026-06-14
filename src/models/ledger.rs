use std::collections::HashMap;

use super::allocation::{Algorithm, Allocation, Event, EventKind, MatchResult, RejectReason};
use super::applicant::{Applicant, ApplicantIdx};
use super::position::{Position, PositionIdx};

/// A logical clock locating a moment within the matching run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub algorithm: Algorithm,
    pub round: u8,
    pub seq: u16,
}

/// The write surface used DURING a matching run.
#[derive(Debug)]
pub struct Ledger {
    allocations: HashMap<PositionIdx, Vec<Allocation>>,
    events: Vec<Event>,
    step: Step,
}

impl Ledger {
    pub fn new(algorithm: Algorithm) -> Self {
        Ledger {
            allocations: HashMap::new(),
            events: Vec::new(),
            step: Step {
                algorithm,
                round: 0,
                seq: 0,
            },
        }
    }

    /// Advance the clock into a new phase.
    pub fn enter(&mut self, algorithm: Algorithm, round: u8) {
        self.step.algorithm = algorithm;
        self.step.round = round;
    }

    /// Mint the next `Step` (monotonic `seq` under the current phase).
    fn tick(&mut self) -> Step {
        let step = Step {
            algorithm: self.step.algorithm,
            round: self.step.round,
            seq: self.step.seq,
        };
        self.step.seq += 1;
        step
    }

    /// Applicants currently holding a seat in position.
    pub fn holders(&self, position: PositionIdx) -> Vec<ApplicantIdx> {
        self.allocations
            .get(&position)
            .map(|seats| seats.iter().map(|a| a.applicant_id).collect())
            .unwrap_or_default()
    }

    /// How many seats in position are currently held.
    pub fn holder_count(&self, position: PositionIdx) -> usize {
        self.allocations.get(&position).map_or(0, Vec::len)
    }

    /// Record that applicant's proposal to position was turned down (audit trail).
    pub fn reject(&mut self, applicant: ApplicantIdx, position: PositionIdx, reason: RejectReason) {
        let step = self.tick();
        self.events.push(Event {
            applicant_id: applicant,
            position_id: position,
            step,
            kind: EventKind::Rejected { reason },
        });
    }

    /// Record that applicant took a seat in position.
    pub fn accept(&mut self, a: &Applicant, p: &Position) {
        let step = self.tick();
        let allocation = Allocation {
            applicant_id: a.id,
            position_id: p.id,
            applicant_rank: a.preference_of(p.id),
            chair_rank: p.rank_of(a.id),
            accepted_at: step,
        };
        self.allocations.entry(p.id).or_default().push(allocation);
    }

    /// Record a `Displaced` event in GS allocation.
    pub fn bump(&mut self, applicant: &Applicant, loser: ApplicantIdx, p: &Position) {
        let step = self.tick();
        let applicant_chair_rank = p.rank_of(applicant.id);

        // Remove loser from allocations.
        if let Some(seats) = self.allocations.get_mut(&p.id) {
            seats.retain(|a| a.applicant_id != loser);
        }
        self.events.push(Event {
            applicant_id: loser,
            position_id: p.id,
            step,
            kind: EventKind::Displaced {
                by: applicant.id,
                by_chair_rank: applicant_chair_rank,
            },
        });

        // Applicant takes the seat.
        self.accept(applicant, p);
    }

    /// Consume the ledger, yielding the immutable result for querying.
    pub fn finish(self) -> MatchResult {
        let mut by_applicant: HashMap<ApplicantIdx, Vec<PositionIdx>> = HashMap::new();
        for (&position_id, allocations) in &self.allocations {
            for allocation in allocations {
                by_applicant
                    .entry(allocation.applicant_id)
                    .or_default()
                    .push(position_id);
            }
        }

        let mut events: HashMap<(ApplicantIdx, PositionIdx), Vec<Event>> = HashMap::new();
        for event in self.events {
            events
                .entry((event.applicant_id, event.position_id))
                .or_default()
                .push(event);
        }

        MatchResult::new(self.allocations, by_applicant, events)
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Ledger::new(Algorithm::ImmediateAcceptance)
    }
}
