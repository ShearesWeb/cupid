use std::collections::HashMap;

use super::allocation::Algorithm;
use super::applicant::{Applicant, ApplicantIdx};
use super::position::{Position, PositionIdx};

/// A borrowed, read-only index over the applicants and positions.
pub struct Pool<'a> {
    applicants: HashMap<ApplicantIdx, &'a Applicant>,
    positions: HashMap<PositionIdx, &'a Position>,
}

impl<'a> Pool<'a> {
    /// Index every applicant, and only the positions routed to `algorithm`.
    pub fn for_algorithm(
        applicants: &'a [Applicant],
        positions: &'a [Position],
        algorithm: Algorithm,
    ) -> Self {
        Pool {
            applicants: applicants.iter().map(|a| (a.id, a)).collect(),
            positions: positions
                .iter()
                .filter(|p| p.algorithm() == algorithm)
                .map(|p| (p.id, p))
                .collect(),
        }
    }

    /// Look up an applicant by id.
    pub fn applicant(&self, id: ApplicantIdx) -> Option<&'a Applicant> {
        self.applicants.get(&id).copied()
    }

    /// Look up a position by id.
    pub fn position(&self, id: PositionIdx) -> Option<&'a Position> {
        self.positions.get(&id).copied()
    }

    /// All applicants in this pool.
    pub fn applicants(&self) -> impl Iterator<Item = &'a Applicant> + '_ {
        self.applicants.values().copied()
    }

    /// The positions in this pool.
    pub fn positions(&self) -> impl Iterator<Item = &'a Position> + '_ {
        self.positions.values().copied()
    }
}
