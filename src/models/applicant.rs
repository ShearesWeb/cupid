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
