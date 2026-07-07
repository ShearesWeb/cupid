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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionType;

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

    #[test]
    fn accept_records_a_holder() {
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        let a = applicant(1, &[10]);
        let p = position(10, 2, &[1]);

        ledger.accept(&a, &p);

        assert_eq!(ledger.holder_count(PositionIdx(10)), 1);
        assert_eq!(ledger.holders(PositionIdx(10)), vec![ApplicantIdx(1)]);
        // An untouched position reports nothing.
        assert_eq!(ledger.holder_count(PositionIdx(99)), 0);
        assert!(ledger.holders(PositionIdx(99)).is_empty());
    }

    #[test]
    fn step_seq_is_monotonic_across_event_kinds() {
        // accept then reject: the clock advances once per recorded event.
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        let a = applicant(1, &[10]);
        let b = applicant(2, &[10]);
        let p = position(10, 2, &[1]);

        ledger.accept(&a, &p);
        ledger.reject(b.id, p.id, RejectReason::RoleCapacityFull);

        let result = ledger.finish();
        let accept_seq = result.for_position(PositionIdx(10))[0].accepted_at.seq;
        let reject_seq = result
            .history(ApplicantIdx(2), PositionIdx(10))
            .next()
            .unwrap()
            .step
            .seq;
        assert!(accept_seq < reject_seq, "seq must increase: {accept_seq} < {reject_seq}");
    }

    #[test]
    fn enter_switches_the_stamped_phase() {
        let mut ledger = Ledger::new(Algorithm::ImmediateAcceptance);
        let a = applicant(1, &[10]);
        let p1 = position(10, 1, &[1]);
        ledger.accept(&a, &p1);

        ledger.enter(Algorithm::GaleShapley, 3);
        let b = applicant(2, &[20]);
        let p2 = position(20, 1, &[2]);
        ledger.accept(&b, &p2);

        let result = ledger.finish();
        let first = result.for_position(PositionIdx(10))[0].accepted_at;
        let second = result.for_position(PositionIdx(20))[0].accepted_at;
        assert_eq!(first.algorithm, Algorithm::ImmediateAcceptance);
        assert_eq!(second.algorithm, Algorithm::GaleShapley);
        assert_eq!(second.round, 3);
    }

    #[test]
    fn bump_swaps_holder_and_logs_displacement() {
        // Chair prefers applicant 1 over 2; 2 holds the lone seat, 1 bumps them.
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        let winner = applicant(1, &[10]);
        let loser = applicant(2, &[10]);
        let p = position(10, 1, &[1, 2]); // chair: 1 best, 2 next

        ledger.accept(&loser, &p);
        ledger.bump(&winner, loser.id, &p);

        // Seat count is unchanged; the holder is swapped.
        assert_eq!(ledger.holder_count(PositionIdx(10)), 1);
        assert_eq!(ledger.holders(PositionIdx(10)), vec![ApplicantIdx(1)]);

        let result = ledger.finish();
        assert_eq!(result.positions_of(ApplicantIdx(1)), &[PositionIdx(10)]);
        assert!(result.positions_of(ApplicantIdx(2)).is_empty(), "loser holds nothing");

        let event = result.history(ApplicantIdx(2), PositionIdx(10)).next().unwrap();
        match event.kind {
            EventKind::Displaced { by, by_chair_rank } => {
                assert_eq!(by, ApplicantIdx(1));
                assert_eq!(by_chair_rank, Some(1), "winner is chair rank 1");
            }
            other => panic!("expected Displaced, got {other:?}"),
        }
    }

    #[test]
    fn finish_inverts_index_and_groups_events() {
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        let a = applicant(1, &[10, 20]);
        let b = applicant(2, &[20]);
        let m = position(10, 1, &[1]);
        let n = position(20, 1, &[2]);

        ledger.accept(&a, &m);
        ledger.accept(&b, &n);
        ledger.reject(a.id, n.id, RejectReason::RoleCapacityFull);

        let result = ledger.finish();
        // by_applicant inverse index.
        assert_eq!(result.positions_of(ApplicantIdx(1)), &[PositionIdx(10)]);
        assert_eq!(result.positions_of(ApplicantIdx(2)), &[PositionIdx(20)]);
        // Events keyed by (applicant, position).
        assert_eq!(result.history(ApplicantIdx(1), PositionIdx(20)).count(), 1);
        assert_eq!(result.history(ApplicantIdx(1), PositionIdx(10)).count(), 0);
    }
}
