use std::collections::HashMap;
use std::str::FromStr;

use super::allocation::Algorithm;
use super::applicant::ApplicantIdx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionIdx(pub i32);

/// CCA Positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Variants share the "Comm" suffix by design: block / main / sub committee.
#[allow(clippy::enum_variant_names)]
pub enum PositionType {
    BlockComm,
    MainComm,
    SubComm,
}

impl PositionType {
    /// Get the matching algorithm used to allocate this position type.
    pub fn algorithm(self) -> Algorithm {
        match self {
            PositionType::BlockComm => Algorithm::ImmediateAcceptance,
            PositionType::MainComm | PositionType::SubComm => Algorithm::GaleShapley,
        }
    }
}

/// Parse a position type from a string.
impl FromStr for PositionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "blockcomm" => Ok(PositionType::BlockComm),
            "maincomm" => Ok(PositionType::MainComm),
            "subcomm" => Ok(PositionType::SubComm),
            other => Err(format!("unknown position type: {other}")),
        }
    }
}

/// A position to be filled, carrying the chair's strict ranking over applicants.
#[derive(Debug, Clone)]
pub struct Position {
    /// External database id, retained for output and audit display.
    pub id: PositionIdx,
    pub cca_name: String,
    pub name: String,
    pub description: Option<String>,
    pub capacity: usize,
    pub position_type: PositionType,

    /// Chair's ranking of applicants. Private.
    ranking: Vec<ApplicantIdx>,

    /// Inverse `ranking` for O(1) lookup: applicant -> 1-based rank.
    inverse_rank: HashMap<ApplicantIdx, usize>,

    /// Seats open to the matcher = capacity minus appointed holders. Computed
    /// once at corpus assembly; the appointee identities live in `Appointments`.
    vacancies: usize,
}

impl Position {
    pub fn new(
        id: i32,
        cca_name: String,
        name: String,
        description: Option<String>,
        capacity: usize,
        position_type: PositionType,
        ranking: Vec<ApplicantIdx>,
    ) -> Self {
        // Generate inverse `ranking`: iter -> enumerate -> map (applicant, 1-based rank)
        let inverse_rank = ranking
            .iter()
            .enumerate()
            .map(|(rank, &applicant)| (applicant, rank + 1))
            .collect();
        Position {
            id: PositionIdx(id),
            cca_name,
            name,
            description,
            capacity,
            position_type,
            ranking,
            inverse_rank,
            vacancies: capacity,
        }
    }

    /// Chair's ranking array.
    pub fn ranking(&self) -> &[ApplicantIdx] {
        &self.ranking
    }

    /// Chair's 1-based rank of `applicant`, or `None` if unranked.
    pub fn rank_of(&self, applicant: ApplicantIdx) -> Option<usize> {
        self.inverse_rank.get(&applicant).copied()
    }

    pub fn algorithm(&self) -> Algorithm {
        self.position_type.algorithm()
    }

    /// Seats open to the matcher = real capacity minus appointed holders.
    pub fn vacancies(&self) -> usize {
        self.vacancies
    }

    /// Builder: record how many seats are already taken by appointment, shrinking
    /// the vacancies open to the matcher. Set once during corpus assembly; the
    /// holder identities are kept separately in [`Appointments`](super::Appointments).
    pub fn with_appointed(mut self, appointed: usize) -> Self {
        self.vacancies = self.capacity.saturating_sub(appointed);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_routes_to_algorithm() {
        assert_eq!(
            PositionType::BlockComm.algorithm(),
            Algorithm::ImmediateAcceptance
        );
        assert_eq!(PositionType::MainComm.algorithm(), Algorithm::GaleShapley);
        assert_eq!(PositionType::SubComm.algorithm(), Algorithm::GaleShapley);
    }

    #[test]
    fn parse_is_case_insensitive_and_trimmed() {
        assert_eq!(
            "blockcomm".parse::<PositionType>().unwrap(),
            PositionType::BlockComm
        );
        assert_eq!(
            "  MainComm ".parse::<PositionType>().unwrap(),
            PositionType::MainComm
        );
        assert_eq!(
            "SUBCOMM".parse::<PositionType>().unwrap(),
            PositionType::SubComm
        );
    }

    #[test]
    fn parse_rejects_unknown() {
        let err = "exco".parse::<PositionType>().unwrap_err();
        assert!(err.contains("exco"), "error names the bad input: {err}");
    }

    #[test]
    fn vacancies_is_capacity_minus_appointed() {
        let p = Position::new(
            10,
            "C".into(),
            "P".into(),
            None,
            3,
            PositionType::MainComm,
            vec![],
        );
        assert_eq!(p.vacancies(), 3, "no appointments -> full capacity");

        let p = p.with_appointed(2);
        assert_eq!(p.vacancies(), 1, "2 of 3 seats already held");

        // More appointees than capacity saturates to 0, never underflows.
        let full = Position::new(
            11,
            "C".into(),
            "Q".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )
        .with_appointed(2);
        assert_eq!(full.vacancies(), 0);
    }

    #[test]
    fn inverse_rank_is_one_based_and_lookups_match() {
        // Chair ranks applicants 7, 3, 9 (best first).
        let p = Position::new(
            100,
            "C".into(),
            "Pos".into(),
            None,
            2,
            PositionType::MainComm,
            vec![ApplicantIdx(7), ApplicantIdx(3), ApplicantIdx(9)],
        );
        assert_eq!(p.rank_of(ApplicantIdx(7)), Some(1));
        assert_eq!(p.rank_of(ApplicantIdx(3)), Some(2));
        assert_eq!(p.rank_of(ApplicantIdx(9)), Some(3));
        // An applicant the chair never listed has no rank.
        assert_eq!(p.rank_of(ApplicantIdx(42)), None);
        // The public ranking slice preserves chair order.
        assert_eq!(
            p.ranking(),
            &[ApplicantIdx(7), ApplicantIdx(3), ApplicantIdx(9)]
        );
    }
}
