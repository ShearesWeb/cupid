use std::collections::HashMap;

use super::applicant::ApplicantIdx;
use super::position::PositionIdx;

/// Operator-granted fixed assignments: a preallocated `(applicant, position)`
/// pair holds the position outright. The matcher seats it before any pass,
/// never scores it, and never lets another proposal displace it. Each pair may
/// carry an operator note explaining why it was granted.
#[derive(Debug, Default)]
pub struct Preallocations {
    pairs: HashMap<(ApplicantIdx, PositionIdx), Option<String>>,
}

impl Preallocations {
    pub fn new() -> Self {
        Self::default()
    }

    /// Preallocate `position` to `applicant`.
    pub fn grant(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.pairs.insert((applicant, position), None);
    }

    /// Preallocate with an operator note. Re-granting replaces the note.
    pub fn grant_with_note(
        &mut self,
        applicant: ApplicantIdx,
        position: PositionIdx,
        note: Option<String>,
    ) {
        self.pairs.insert((applicant, position), note);
    }

    /// Remove the preallocation. A missing pair is a no-op.
    pub fn revoke(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.pairs.remove(&(applicant, position));
    }

    /// Is this exact `(applicant, position)` pair preallocated?
    pub fn contains(&self, applicant: ApplicantIdx, position: PositionIdx) -> bool {
        self.pairs.contains_key(&(applicant, position))
    }

    /// The operator note attached to a preallocated pair, if any.
    pub fn note(&self, applicant: ApplicantIdx, position: PositionIdx) -> Option<&str> {
        self.pairs
            .get(&(applicant, position))
            .and_then(|n| n.as_deref())
    }

    /// Every preallocated pair, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (ApplicantIdx, PositionIdx)> + '_ {
        self.pairs.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_pair_only() {
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        assert!(preallocations.contains(ApplicantIdx(1), PositionIdx(10)));
        // Same applicant, different position: not preallocated.
        assert!(!preallocations.contains(ApplicantIdx(1), PositionIdx(11)));
        // Different applicant, same position: not preallocated.
        assert!(!preallocations.contains(ApplicantIdx(2), PositionIdx(10)));
    }

    #[test]
    fn iterates_pairs() {
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        let pairs: Vec<_> = preallocations.iter().collect();
        assert_eq!(pairs, vec![(ApplicantIdx(1), PositionIdx(10))]);
    }

    #[test]
    fn revoke_removes_only_that_pair() {
        let mut preallocations = Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        preallocations.grant(ApplicantIdx(2), PositionIdx(10));
        preallocations.revoke(ApplicantIdx(1), PositionIdx(10));
        assert!(!preallocations.contains(ApplicantIdx(1), PositionIdx(10)));
        assert!(preallocations.contains(ApplicantIdx(2), PositionIdx(10)));
        // Revoking a missing pair is a no-op.
        preallocations.revoke(ApplicantIdx(9), PositionIdx(9));
    }

    #[test]
    fn note_round_trips_and_regrant_replaces() {
        let mut preallocations = Preallocations::new();
        preallocations.grant_with_note(
            ApplicantIdx(1),
            PositionIdx(10),
            Some("chair request".into()),
        );
        assert_eq!(
            preallocations.note(ApplicantIdx(1), PositionIdx(10)),
            Some("chair request")
        );
        // Plain grant has no note; regranting the same pair replaces it.
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));
        assert_eq!(preallocations.note(ApplicantIdx(1), PositionIdx(10)), None);
        // Unknown pair has no note.
        assert_eq!(preallocations.note(ApplicantIdx(2), PositionIdx(10)), None);
    }
}
