use std::collections::HashMap;

use super::allocation::Algorithm;
use super::applicant::{Applicant, ApplicantIdx};
use super::position::{CCAIdx, Position, PositionIdx};

/// The owned corpus: every applicant, every position, and CCA display names.
/// The single source of truth — produced by the data layer, consumed by the
/// matcher and the report.
pub struct Pool {
    applicants: Vec<Applicant>,
    positions: Vec<Position>,
    ccas: HashMap<CCAIdx, String>,
    applicant_idx: HashMap<ApplicantIdx, usize>,
    position_idx: HashMap<PositionIdx, usize>,
}

impl Pool {
    pub fn new(
        applicants: Vec<Applicant>,
        positions: Vec<Position>,
        ccas: HashMap<CCAIdx, String>,
    ) -> Self {
        let applicant_idx = applicants.iter().enumerate().map(|(i, a)| (a.id, i)).collect();
        let position_idx = positions.iter().enumerate().map(|(i, p)| (p.id, i)).collect();
        Pool {
            applicants,
            positions,
            ccas,
            applicant_idx,
            position_idx,
        }
    }

    pub fn applicants(&self) -> &[Applicant] {
        &self.applicants
    }

    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    /// CCA display name for `id`, if known.
    pub fn cca_name(&self, id: CCAIdx) -> Option<&str> {
        self.ccas.get(&id).map(String::as_str)
    }

    /// Applicant by id, O(1).
    pub fn applicant(&self, id: ApplicantIdx) -> Option<&Applicant> {
        self.applicant_idx.get(&id).map(|&i| &self.applicants[i])
    }

    /// Position by id, O(1).
    pub fn position(&self, id: PositionIdx) -> Option<&Position> {
        self.position_idx.get(&id).map(|&i| &self.positions[i])
    }
}

/// A borrowed, read-only index over every applicant and the positions routed to
/// one algorithm. Built per matching pass from a [`Pool`]'s slices.
pub struct Roster<'a> {
    applicants: HashMap<ApplicantIdx, &'a Applicant>,
    positions: HashMap<PositionIdx, &'a Position>,
}

impl<'a> Roster<'a> {
    /// Index every applicant, and only the positions routed to `algorithm`.
    pub fn for_algorithm(
        applicants: &'a [Applicant],
        positions: &'a [Position],
        algorithm: Algorithm,
    ) -> Self {
        Roster {
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

    /// All applicants in this roster.
    pub fn applicants(&self) -> impl Iterator<Item = &'a Applicant> + '_ {
        self.applicants.values().copied()
    }

    /// The positions in this roster.
    pub fn positions(&self) -> impl Iterator<Item = &'a Position> + '_ {
        self.positions.values().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionType;

    // ---- owned corpus (`Pool`) ----

    #[test]
    fn pool_accessors_expose_corpus() {
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let positions = vec![Position::new(
            10, 5, "Head".into(), None, 1, PositionType::MainComm, vec![],
        )];
        let mut ccas = HashMap::new();
        ccas.insert(CCAIdx(5), "Chess".to_string());

        let pool = Pool::new(applicants, positions, ccas);

        assert_eq!(pool.applicants().len(), 1);
        assert_eq!(pool.positions().len(), 1);
        assert_eq!(pool.cca_name(CCAIdx(5)), Some("Chess"));
        assert_eq!(pool.cca_name(CCAIdx(99)), None);
    }

    #[test]
    fn id_accessors_find_by_id() {
        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![Position::new(
            10, 5, "Head".into(), None, 1, PositionType::MainComm, vec![],
        )];
        let pool = Pool::new(applicants, positions, HashMap::new());

        assert_eq!(pool.applicant(ApplicantIdx(1)).unwrap().name, "Ann");
        assert!(pool.applicant(ApplicantIdx(99)).is_none());
        assert_eq!(pool.position(PositionIdx(10)).unwrap().capacity, 1);
        assert!(pool.position(PositionIdx(99)).is_none());
    }

    // ---- per-algorithm view (`Roster`) ----

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
    fn ia_roster_keeps_only_blockcomm() {
        let (applicants, positions) = fixture();
        let roster = Roster::for_algorithm(&applicants, &positions, Algorithm::ImmediateAcceptance);
        assert_eq!(roster.positions().count(), 1);
        assert!(roster.position(PositionIdx(10)).is_some(), "blockcomm in scope");
        assert!(roster.position(PositionIdx(20)).is_none(), "maincomm filtered out");
        assert!(roster.position(PositionIdx(30)).is_none(), "subcomm filtered out");
    }

    #[test]
    fn gs_roster_keeps_main_and_sub() {
        let (applicants, positions) = fixture();
        let roster = Roster::for_algorithm(&applicants, &positions, Algorithm::GaleShapley);
        assert_eq!(roster.positions().count(), 2);
        assert!(roster.position(PositionIdx(20)).is_some());
        assert!(roster.position(PositionIdx(30)).is_some());
        assert!(roster.position(PositionIdx(10)).is_none(), "blockcomm filtered out");
    }

    #[test]
    fn every_applicant_is_present_in_each_roster() {
        // The roster partitions positions by algorithm but never drops applicants.
        let (applicants, positions) = fixture();
        for algo in [Algorithm::ImmediateAcceptance, Algorithm::GaleShapley] {
            let roster = Roster::for_algorithm(&applicants, &positions, algo);
            assert_eq!(roster.applicants().count(), 2);
            assert!(roster.applicant(ApplicantIdx(1)).is_some());
            assert!(roster.applicant(ApplicantIdx(2)).is_some());
            assert!(roster.applicant(ApplicantIdx(99)).is_none());
        }
    }
}
