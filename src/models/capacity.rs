use std::collections::{HashMap, HashSet};

use super::applicant::ApplicantIdx;
use super::position::{PositionIdx, PositionType};

/// What one applicant currently holds, tallied by allocatable type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeldCounts {
    pub blockcomm: u8,
    pub maincomm: u8,
    pub subcomm: u8,
    pub appealed: u8,
}

impl HeldCounts {
    /// Apply a delta of `+1` for taking a position.
    fn add(&mut self, ty: PositionType, appealed: bool) {
        if appealed {
            self.appealed += 1;
            return;
        }
        match ty {
            PositionType::BlockComm => self.blockcomm += 1,
            PositionType::MainComm => self.maincomm += 1,
            PositionType::SubComm => self.subcomm += 1,
        }
    }

    /// Apply a delta of `-1` for losing a position.
    fn remove(&mut self, ty: PositionType, appealed: bool) {
        if appealed {
            self.appealed = self.appealed.saturating_sub(1);
            return;
        }
        match ty {
            PositionType::BlockComm => self.blockcomm = self.blockcomm.saturating_sub(1),
            PositionType::MainComm => self.maincomm = self.maincomm.saturating_sub(1),
            PositionType::SubComm => self.subcomm = self.subcomm.saturating_sub(1),
        }
    }
    /// Would adding one position of `ty` to the current holdings exceed quota?
    pub fn can_add(&self, ty: PositionType) -> bool {
        let mut blockcomm = self.blockcomm;
        let mut maincomm = self.maincomm;
        let mut subcomm = self.subcomm;

        match ty {
            PositionType::BlockComm => blockcomm += 1,
            PositionType::MainComm => maincomm += 1,
            PositionType::SubComm => subcomm += 1,
        }

        maincomm + blockcomm <= 2 && subcomm <= 3 && !(maincomm >= 1 && subcomm >= 2)
    }
}

/// Mutable run-state: per-applicant tally of held positions.
#[derive(Debug, Default)]
pub struct CapacityStore {
    held: HashMap<ApplicantIdx, HeldCounts>,
}

impl CapacityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current allocation tally for `applicant`.
    pub fn get(&self, applicant: ApplicantIdx) -> HeldCounts {
        self.held.get(&applicant).copied().unwrap_or_default()
    }

    /// Would granting `applicant` a position of `ty` keep them within limits?
    /// Appealed seats must bypass this check entirely.
    pub fn can_grant(&self, applicant: ApplicantIdx, ty: PositionType) -> bool {
        self.get(applicant).can_add(ty)
    }

    /// Record that `applicant` took a position of `ty` or `appealed` in tally.
    pub fn grant(&mut self, applicant: ApplicantIdx, ty: PositionType, appealed: bool) {
        self.held.entry(applicant).or_default().add(ty, appealed);
    }

    /// Record that `applicant` lost a position of `ty` or `appealed` in tally.
    pub fn revoke(&mut self, applicant: ApplicantIdx, ty: PositionType, appealed: bool) {
        if let Some(counts) = self.held.get_mut(&applicant) {
            counts.remove(ty, appealed);
        }
    }
}

/// An exempt proposal whitelists a specific `(applicant, position)` so it does not count
/// toward the applicant's capacity limits.
#[derive(Debug, Default)]
pub struct Appeals {
    whitelist: HashSet<(ApplicantIdx, PositionIdx)>,
}

impl Appeals {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whitelist the `(applicant, position)` proposal as quota-exempt.
    pub fn grant(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.whitelist.insert((applicant, position));
    }

    /// Is this exact `(applicant, position)` proposal exempt from quota?
    pub fn contains(&self, applicant: ApplicantIdx, position: PositionIdx) -> bool {
        self.whitelist.contains(&(applicant, position))
    }
}
