use std::collections::{HashMap, HashSet};

use super::applicant::ApplicantIdx;
use super::pool::Pool;
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

    /// Seed every holder's tally from existing appointments. Walks the position
    /// side so each appointment's type is the position's own type — no lookup.
    /// Non-applicant holders are tallied too; their tally is never queried.
    pub fn from_pool(pool: &Pool) -> Self {
        let mut store = CapacityStore::new();
        for position in pool.positions() {
            for &aid in position.appointments() {
                store.grant(aid, position.position_type, false);
            }
        }
        store
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

#[cfg(test)]
mod tests {
    use super::*;
    use PositionType::{BlockComm, MainComm, SubComm};

    /// Build a `HeldCounts` directly so each quota rule can be probed in isolation.
    fn held(block: u8, main: u8, sub: u8) -> HeldCounts {
        HeldCounts {
            blockcomm: block,
            maincomm: main,
            subcomm: sub,
            appealed: 0,
        }
    }

    #[test]
    fn empty_can_add_any_type() {
        let h = HeldCounts::default();
        assert!(h.can_add(BlockComm));
        assert!(h.can_add(MainComm));
        assert!(h.can_add(SubComm));
    }

    #[test]
    fn main_plus_block_caps_at_two() {
        // One main + one block = 2 held; a third of either kind tips over.
        let h = held(1, 1, 0);
        assert!(!h.can_add(MainComm), "main+block would be 3");
        assert!(!h.can_add(BlockComm), "main+block would be 3");
        // Two blocks (no main) leaves room for one sub under the cross rule.
        assert!(held(2, 0, 0).can_add(SubComm));
        assert!(!held(2, 0, 0).can_add(BlockComm), "would be 3 block");
    }

    #[test]
    fn subcomm_caps_at_three() {
        assert!(held(0, 0, 2).can_add(SubComm), "third sub is fine");
        assert!(!held(0, 0, 3).can_add(SubComm), "fourth sub over cap");
    }

    #[test]
    fn main_and_two_sub_forbidden() {
        // The cross rule: holding a main forbids a second subcomm.
        assert!(!held(0, 1, 1).can_add(SubComm), "1 main + 2 sub banned");
        assert!(held(0, 1, 0).can_add(SubComm), "1 main + 1 sub allowed");
        // Mirror: holding two subs forbids taking a main.
        assert!(!held(0, 0, 2).can_add(MainComm), "2 sub + 1 main banned");
        // No main: two subs may grow to three.
        assert!(held(0, 0, 2).can_add(SubComm));
    }

    #[test]
    fn store_grant_tracks_each_type() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, BlockComm, false);
        store.grant(a, MainComm, false);
        store.grant(a, SubComm, false);
        let counts = store.get(a);
        assert_eq!((counts.blockcomm, counts.maincomm, counts.subcomm), (1, 1, 1));
    }

    #[test]
    fn store_revoke_decrements_and_saturates() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, SubComm, false);
        store.revoke(a, SubComm, false);
        assert_eq!(store.get(a).subcomm, 0);
        // Revoking below zero stays at zero rather than wrapping.
        store.revoke(a, SubComm, false);
        assert_eq!(store.get(a).subcomm, 0);
        // Revoking an applicant with no record at all is a no-op.
        store.revoke(ApplicantIdx(99), MainComm, false);
        assert_eq!(store.get(ApplicantIdx(99)), HeldCounts::default());
    }

    #[test]
    fn can_grant_delegates_to_quota() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, MainComm, false);
        store.grant(a, SubComm, false);
        // Now 1 main + 1 sub: a second sub is barred by the cross rule...
        assert!(!store.can_grant(a, SubComm));
        // ...but a block still fits (main+block = 2, sub = 1).
        assert!(store.can_grant(a, BlockComm));
    }

    #[test]
    fn appealed_grant_is_invisible_to_quota() {
        // Appealed seats land in their own bucket and never restrict later grants.
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, MainComm, true);
        store.grant(a, MainComm, true);
        let counts = store.get(a);
        assert_eq!(counts.appealed, 2);
        assert_eq!(counts.maincomm, 0, "appealed seats skip the type bucket");
        // Quota still sees zero real holdings, so a real main remains grantable.
        assert!(store.can_grant(a, MainComm));
    }

    #[test]
    fn from_pool_seeds_quota_from_appointments() {
        use crate::models::{Applicant, Pool, Position};
        use std::collections::HashMap;

        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![
            Position::new(10, 1, "M".into(), None, 2, MainComm, vec![])
                .with_appointments(vec![ApplicantIdx(1)]),
            Position::new(20, 1, "S".into(), None, 2, SubComm, vec![])
                .with_appointments(vec![ApplicantIdx(1)]),
        ];
        let pool = Pool::new(applicants, positions, HashMap::new());

        let store = CapacityStore::from_pool(&pool);
        let counts = store.get(ApplicantIdx(1));
        assert_eq!(counts.maincomm, 1);
        assert_eq!(counts.subcomm, 1);
    }

    #[test]
    fn appeals_match_exact_pair_only() {
        let mut appeals = Appeals::new();
        appeals.grant(ApplicantIdx(1), PositionIdx(10));
        assert!(appeals.contains(ApplicantIdx(1), PositionIdx(10)));
        // Same applicant, different position: not exempt.
        assert!(!appeals.contains(ApplicantIdx(1), PositionIdx(11)));
        // Different applicant, same position: not exempt.
        assert!(!appeals.contains(ApplicantIdx(2), PositionIdx(10)));
    }
}
