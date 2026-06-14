use std::collections::HashMap;
use std::str::FromStr;

use super::allocation::Algorithm;
use super::applicant::ApplicantIdx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionIdx(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CCAIdx(pub i32);

/// CCA Positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub cca_id: CCAIdx,
    pub name: String,
    pub description: Option<String>,
    pub capacity: usize,
    pub position_type: PositionType,

    /// Chair's ranking of applicants. Private.
    ranking: Vec<ApplicantIdx>,

    /// Inverse `ranking` for O(1) lookup: applicant -> 1-based rank.
    inverse_rank: HashMap<ApplicantIdx, usize>,
}

impl Position {
    pub fn new(
        id: i32,
        cca_id: i32,
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
            cca_id: CCAIdx(cca_id),
            name,
            description,
            capacity,
            position_type,
            ranking,
            inverse_rank,
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
}
