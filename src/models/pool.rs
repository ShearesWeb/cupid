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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionType;

    fn fixture() -> (Vec<Applicant>, Vec<Position>) {
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![]),
        ];
        let positions = vec![
            Position::new(10, 1, "Block".into(), None, 1, PositionType::BlockComm, vec![]),
            Position::new(20, 1, "Main".into(), None, 1, PositionType::MainComm, vec![]),
            Position::new(30, 1, "Sub".into(), None, 1, PositionType::SubComm, vec![]),
        ];
        (applicants, positions)
    }

    #[test]
    fn ia_pool_keeps_only_blockcomm() {
        let (applicants, positions) = fixture();
        let pool = Pool::for_algorithm(&applicants, &positions, Algorithm::ImmediateAcceptance);
        assert_eq!(pool.positions().count(), 1);
        assert!(pool.position(PositionIdx(10)).is_some(), "blockcomm in scope");
        assert!(pool.position(PositionIdx(20)).is_none(), "maincomm filtered out");
        assert!(pool.position(PositionIdx(30)).is_none(), "subcomm filtered out");
    }

    #[test]
    fn gs_pool_keeps_main_and_sub() {
        let (applicants, positions) = fixture();
        let pool = Pool::for_algorithm(&applicants, &positions, Algorithm::GaleShapley);
        assert_eq!(pool.positions().count(), 2);
        assert!(pool.position(PositionIdx(20)).is_some());
        assert!(pool.position(PositionIdx(30)).is_some());
        assert!(pool.position(PositionIdx(10)).is_none(), "blockcomm filtered out");
    }

    #[test]
    fn every_applicant_is_present_in_each_pool() {
        // The pool partitions positions by algorithm but never drops applicants.
        let (applicants, positions) = fixture();
        for algo in [Algorithm::ImmediateAcceptance, Algorithm::GaleShapley] {
            let pool = Pool::for_algorithm(&applicants, &positions, algo);
            assert_eq!(pool.applicants().count(), 2);
            assert!(pool.applicant(ApplicantIdx(1)).is_some());
            assert!(pool.applicant(ApplicantIdx(2)).is_some());
            assert!(pool.applicant(ApplicantIdx(99)).is_none());
        }
    }
}
