use std::collections::HashMap;

use super::applicant::ApplicantIdx;
use super::cca::CcaIdx;
use super::pool::Pool;
use super::position::PositionType;

/// What one applicant currently holds, tallied by allocatable type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeldCounts {
    pub blockcomm: u8,
    pub maincomm: u8,
    pub subcomm: u8,
}

impl HeldCounts {
    /// Apply a delta of `+1` for taking a position.
    fn add(&mut self, ty: PositionType) {
        match ty {
            PositionType::BlockComm => self.blockcomm += 1,
            PositionType::MainComm => self.maincomm += 1,
            PositionType::SubComm => self.subcomm += 1,
        }
    }

    /// Apply a delta of `-1` for losing a position.
    fn remove(&mut self, ty: PositionType) {
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
        next.add(ty);
        next.within_quota()
    }
}

/// Mutable run-state: per-applicant tally of held positions, by type and by
/// CCA. The CCA tally mirrors the database rule that a user may hold at most
/// one non-resident position per CCA (everything cupid allocates is
/// non-resident, and cupid treats every holding as full-year).
#[derive(Debug, Default)]
pub struct CapacityStore {
    held: HashMap<ApplicantIdx, HeldCounts>,
    ccas: HashMap<ApplicantIdx, HashMap<CcaIdx, u8>>,
}

impl CapacityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current allocation tally for `applicant`.
    pub fn get(&self, applicant: ApplicantIdx) -> HeldCounts {
        self.held.get(&applicant).copied().unwrap_or_default()
    }

    /// How many non-resident positions `applicant` holds in `cca`.
    pub fn cca_held(&self, applicant: ApplicantIdx, cca: CcaIdx) -> u8 {
        self.ccas
            .get(&applicant)
            .and_then(|held| held.get(&cca))
            .copied()
            .unwrap_or(0)
    }

    /// Would granting `applicant` a position of `ty` in `cca` keep them
    /// within limits? Both the type quota and the one-per-CCA rule apply.
    pub fn can_grant(&self, applicant: ApplicantIdx, ty: PositionType, cca: CcaIdx) -> bool {
        self.get(applicant).can_add(ty) && self.cca_held(applicant, cca) == 0
    }

    /// Record that `applicant` took a position of `ty` in `cca`.
    pub fn grant(&mut self, applicant: ApplicantIdx, ty: PositionType, cca: CcaIdx) {
        self.held.entry(applicant).or_default().add(ty);
        self.bump_cca(applicant, cca, 1);
    }

    /// Record that `applicant` lost a position of `ty` in `cca`.
    pub fn revoke(&mut self, applicant: ApplicantIdx, ty: PositionType, cca: CcaIdx) {
        if let Some(counts) = self.held.get_mut(&applicant) {
            counts.remove(ty);
        }
        self.bump_cca(applicant, cca, -1);
    }

    /// Record a holding that only occupies a CCA slot: an appointment to a
    /// position cupid does not allocate (e.g. `member`). It never counts
    /// toward the type quota, but the database still enforces one
    /// non-resident position per CCA, so the matcher must see it.
    pub fn note_external(&mut self, applicant: ApplicantIdx, cca: CcaIdx) {
        self.bump_cca(applicant, cca, 1);
    }

    fn bump_cca(&mut self, applicant: ApplicantIdx, cca: CcaIdx, delta: i8) {
        let held = self.ccas.entry(applicant).or_default().entry(cca).or_insert(0);
        *held = held.saturating_add_signed(delta);
    }

    /// Seed every holder's tally from the pool: committed appointments count
    /// toward both the type quota and the CCA rule; external occupancy
    /// (appointments to positions outside the market) toward the CCA rule only.
    pub fn from_pool(pool: &Pool) -> Self {
        let mut store = CapacityStore::new();
        for appointment in pool.appointments().iter() {
            if let Some(position) = pool.position(appointment.position) {
                store.grant(appointment.applicant, position.position_type, position.cca.id);
            }
        }
        for &(applicant, cca) in pool.external_occupancy() {
            store.note_external(applicant, cca);
        }
        store
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
    fn within_quota_matches_rule() {
        assert!(held(1, 1, 0).within_quota());
        assert!(held(1, 1, 1).within_quota(), "2 main/block + 1 sub is legal");
        assert!(held(0, 0, 3).within_quota());
        assert!(!held(2, 1, 0).within_quota(), "3 main/block");
        assert!(!held(0, 1, 2).within_quota(), "cross rule");
        assert!(!held(0, 0, 4).within_quota(), "4 sub");
    }

    #[test]
    fn store_grant_tracks_each_type() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, BlockComm, CcaIdx(1));
        store.grant(a, MainComm, CcaIdx(2));
        store.grant(a, SubComm, CcaIdx(3));
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
        store.grant(a, SubComm, CcaIdx(1));
        store.revoke(a, SubComm, CcaIdx(1));
        assert_eq!(store.get(a).subcomm, 0);
        // Revoking below zero stays at zero rather than wrapping.
        store.revoke(a, SubComm, CcaIdx(1));
        assert_eq!(store.get(a).subcomm, 0);
        // Revoking an applicant with no record at all is a no-op.
        store.revoke(ApplicantIdx(99), MainComm, CcaIdx(1));
        assert_eq!(store.get(ApplicantIdx(99)), HeldCounts::default());
    }

    #[test]
    fn can_grant_delegates_to_quota() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, MainComm, CcaIdx(1));
        store.grant(a, SubComm, CcaIdx(2));
        // Now 1 main + 1 sub: a second sub is barred by the cross rule...
        assert!(!store.can_grant(a, SubComm, CcaIdx(3)));
        // ...but a block still fits (main+block = 2, sub = 1).
        assert!(store.can_grant(a, BlockComm, CcaIdx(3)));
    }

    #[test]
    fn second_position_in_same_cca_is_barred() {
        // The database allows one non-resident position per user per CCA.
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, MainComm, CcaIdx(5));
        assert_eq!(store.cca_held(a, CcaIdx(5)), 1);
        assert!(
            !store.can_grant(a, SubComm, CcaIdx(5)),
            "second position in CCA 5 must be barred even though the type quota allows it"
        );
        assert!(
            store.can_grant(a, SubComm, CcaIdx(6)),
            "a different CCA is unaffected"
        );
        // Another applicant is unaffected.
        assert!(store.can_grant(ApplicantIdx(2), SubComm, CcaIdx(5)));
    }

    #[test]
    fn revoke_frees_the_cca_slot() {
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.grant(a, SubComm, CcaIdx(5));
        assert!(!store.can_grant(a, MainComm, CcaIdx(5)));
        store.revoke(a, SubComm, CcaIdx(5));
        assert_eq!(store.cca_held(a, CcaIdx(5)), 0);
        assert!(store.can_grant(a, MainComm, CcaIdx(5)), "slot freed by revoke");
    }

    #[test]
    fn external_occupancy_blocks_the_cca_but_not_the_type_quota() {
        // A `member` appointment in CCA 5: invisible to the type quota,
        // but the CCA slot is taken.
        let mut store = CapacityStore::new();
        let a = ApplicantIdx(1);
        store.note_external(a, CcaIdx(5));
        assert_eq!(store.get(a), HeldCounts::default(), "no type tally");
        assert!(!store.can_grant(a, MainComm, CcaIdx(5)), "CCA slot taken");
        assert!(store.can_grant(a, MainComm, CcaIdx(6)));
    }

    #[test]
    fn from_pool_seeds_quota_and_cca_from_appointments() {
        use crate::models::{
            Applicant, Appointment, Appointments, Cca, CcaIdx, Pool, Position, PositionIdx,
        };

        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![
            Position::new(10, Cca::new(7, "C7"), "M".into(), None, 2, MainComm, vec![])
                .with_appointed(1),
            Position::new(20, Cca::new(8, "C8"), "S".into(), None, 2, SubComm, vec![])
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
        let pool = Pool::new(applicants, positions)
            .with_appointments(appointments)
            .with_external_occupancy(vec![(ApplicantIdx(1), CcaIdx(9))]);

        let store = CapacityStore::from_pool(&pool);
        let counts = store.get(ApplicantIdx(1));
        assert_eq!(counts.maincomm, 1);
        assert_eq!(counts.subcomm, 1);

        // Appointments occupy their CCA slots; external occupancy (an
        // appointment outside the market) occupies its CCA slot too.
        assert!(!store.can_grant(ApplicantIdx(1), BlockComm, CcaIdx(7)));
        assert!(!store.can_grant(ApplicantIdx(1), BlockComm, CcaIdx(8)));
        assert!(!store.can_grant(ApplicantIdx(1), BlockComm, CcaIdx(9)));
        assert!(store.can_grant(ApplicantIdx(1), BlockComm, CcaIdx(6)));
    }
}
