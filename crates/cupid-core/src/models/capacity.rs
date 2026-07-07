use std::collections::HashMap;

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
    /// Are the current holdings legal under the quota rule?
    pub fn within_quota(&self) -> bool {
        self.maincomm + self.blockcomm <= 2
            && self.subcomm <= 3
            && !(self.maincomm >= 1 && self.subcomm >= 2)
    }

    /// Would adding one position of `ty` to the current holdings exceed quota?
    pub fn can_add(&self, ty: PositionType) -> bool {
        let mut next = *self;
        next.add(ty, false);
        next.within_quota()
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

    /// Seed every holder's tally from existing appointments.
    pub fn from_pool(pool: &Pool) -> Self {
        let mut store = CapacityStore::new();
        for appointment in pool.appointments().iter() {
            if let Some(position) = pool.position(appointment.position) {
                store.grant(appointment.applicant, position.position_type, false);
            }
        }
        store
    }
}

/// An exempt proposal whitelists a specific `(applicant, position)` so it does not count
/// toward the applicant's capacity limits. Each pair may carry an operator
/// note explaining why the exemption was granted.
#[derive(Debug, Default)]
pub struct Appeals {
    whitelist: HashMap<(ApplicantIdx, PositionIdx), Option<String>>,
}

impl Appeals {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whitelist the `(applicant, position)` proposal as quota-exempt.
    pub fn grant(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.whitelist.insert((applicant, position), None);
    }

    /// Whitelist with an operator note. Re-granting replaces the note.
    pub fn grant_with_note(
        &mut self,
        applicant: ApplicantIdx,
        position: PositionIdx,
        note: Option<String>,
    ) {
        self.whitelist.insert((applicant, position), note);
    }

    /// Remove the exemption. A missing pair is a no-op.
    pub fn revoke(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.whitelist.remove(&(applicant, position));
    }

    /// Is this exact `(applicant, position)` proposal exempt from quota?
    pub fn contains(&self, applicant: ApplicantIdx, position: PositionIdx) -> bool {
        self.whitelist.contains_key(&(applicant, position))
    }

    /// The operator note attached to an exempt pair, if any.
    pub fn note(&self, applicant: ApplicantIdx, position: PositionIdx) -> Option<&str> {
        self.whitelist
            .get(&(applicant, position))
            .and_then(|n| n.as_deref())
    }

    /// Every exempt pair, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (ApplicantIdx, PositionIdx)> + '_ {
        self.whitelist.keys().copied()
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
        assert_eq!(
            (counts.blockcomm, counts.maincomm, counts.subcomm),
            (1, 1, 1)
        );
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
        use crate::models::{
            Applicant, Appointment, Appointments, Cca, Pool, Position, PositionIdx,
        };

        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![
            Position::new(10, Cca::new(0, "C"), "M".into(), None, 2, MainComm, vec![])
                .with_appointed(1),
            Position::new(20, Cca::new(0, "C"), "S".into(), None, 2, SubComm, vec![])
                .with_appointed(1),
        ];
        let appointments = Appointments::from_iter([
            Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(10),
            },
            Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(20),
            },
        ]);
        let pool = Pool::new(applicants, positions).with_appointments(appointments);

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

    #[test]
    fn within_quota_matches_rule() {
        assert!(held(1, 1, 0).within_quota());
        assert!(held(1, 1, 1).within_quota(), "2 main/block + 1 sub is legal");
        assert!(held(0, 0, 3).within_quota());
        assert!(!held(2, 1, 0).within_quota(), "3 main/block");
        assert!(!held(0, 1, 2).within_quota(), "cross rule");
        assert!(!held(0, 0, 4).within_quota(), "4 sub");
    }

    #[test]
    fn appeals_iterates_pairs() {
        let mut appeals = Appeals::new();
        appeals.grant(ApplicantIdx(1), PositionIdx(10));
        let pairs: Vec<_> = appeals.iter().collect();
        assert_eq!(pairs, vec![(ApplicantIdx(1), PositionIdx(10))]);
    }

    #[test]
    fn appeals_revoke_removes_only_that_pair() {
        let mut appeals = Appeals::new();
        appeals.grant(ApplicantIdx(1), PositionIdx(10));
        appeals.grant(ApplicantIdx(2), PositionIdx(10));
        appeals.revoke(ApplicantIdx(1), PositionIdx(10));
        assert!(!appeals.contains(ApplicantIdx(1), PositionIdx(10)));
        assert!(appeals.contains(ApplicantIdx(2), PositionIdx(10)));
        // Revoking a missing pair is a no-op.
        appeals.revoke(ApplicantIdx(9), PositionIdx(9));
    }

    #[test]
    fn appeals_note_round_trips_and_regrant_replaces() {
        let mut appeals = Appeals::new();
        appeals.grant_with_note(ApplicantIdx(1), PositionIdx(10), Some("chair request".into()));
        assert_eq!(appeals.note(ApplicantIdx(1), PositionIdx(10)), Some("chair request"));
        // Plain grant has no note; regranting the same pair replaces it.
        appeals.grant(ApplicantIdx(1), PositionIdx(10));
        assert_eq!(appeals.note(ApplicantIdx(1), PositionIdx(10)), None);
        // Unknown pair has no note.
        assert_eq!(appeals.note(ApplicantIdx(2), PositionIdx(10)), None);
    }
}
