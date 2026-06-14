use super::position::PositionIdx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ApplicantIdx(pub i32);

/// An applicant with a strict preference list over positions, best first.
#[derive(Debug, Clone)]
pub struct Applicant {
    pub id: ApplicantIdx,
    pub name: String,
    pub email: String,

    /// Preferred positions based on ranking.
    pub preferences: Vec<PositionIdx>,
}

impl Applicant {
    pub fn new(id: i32, name: String, email: String, preferences: Vec<PositionIdx>) -> Self {
        Applicant {
            id: ApplicantIdx(id),
            name,
            email,
            preferences,
        }
    }

    /// Applicant's position rankings.
    pub fn preferences(&self) -> &[PositionIdx] {
        &self.preferences
    }

    /// Chair's 1-based rank of `position`, or `None` if unranked.
    pub fn preference_of(&self, position: PositionIdx) -> Option<usize> {
        self.preferences
            .iter()
            .position(|&pid| pid == position)
            .map(|i| i + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_of_is_one_based() {
        let a = Applicant::new(
            1,
            "Ann".into(),
            "ann@x".into(),
            vec![PositionIdx(30), PositionIdx(10), PositionIdx(20)],
        );
        assert_eq!(a.preference_of(PositionIdx(30)), Some(1));
        assert_eq!(a.preference_of(PositionIdx(10)), Some(2));
        assert_eq!(a.preference_of(PositionIdx(20)), Some(3));
    }

    #[test]
    fn preference_of_absent_is_none() {
        let a = Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(30)]);
        assert_eq!(a.preference_of(PositionIdx(99)), None);
    }

    #[test]
    fn empty_preferences_rank_nothing() {
        let a = Applicant::new(2, "Ben".into(), "ben@x".into(), vec![]);
        assert!(a.preferences().is_empty());
        assert_eq!(a.preference_of(PositionIdx(1)), None);
    }
}
