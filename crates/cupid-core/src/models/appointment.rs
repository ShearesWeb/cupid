use std::collections::HashMap;

use super::applicant::ApplicantIdx;
use super::position::PositionIdx;

/// One pre-existing holding: `applicant` already occupies `position` coming into
/// the run. Appointments are immutable corpus input — they seed capacity and
/// shrink a position's vacancies, but the matcher never creates or removes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Appointment {
    pub applicant: ApplicantIdx,
    pub position: PositionIdx,
}

/// The appointment relation, indexed in both directions for O(1) lookup either
/// way. Single source of truth: `Applicant` and `Position` no longer each carry
/// a private copy, so the two sides cannot drift apart.
#[derive(Debug, Clone, Default)]
pub struct Appointments {
    by_position: HashMap<PositionIdx, Vec<ApplicantIdx>>,
    by_applicant: HashMap<ApplicantIdx, Vec<PositionIdx>>,
}

impl Appointments {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `applicant` already holds `position`.
    pub fn insert(&mut self, applicant: ApplicantIdx, position: PositionIdx) {
        self.by_position
            .entry(position)
            .or_default()
            .push(applicant);
        self.by_applicant
            .entry(applicant)
            .or_default()
            .push(position);
    }

    /// Applicants appointed to `position`.
    pub fn holders(&self, position: PositionIdx) -> &[ApplicantIdx] {
        self.by_position
            .get(&position)
            .map_or(&[][..], Vec::as_slice)
    }

    /// Positions `applicant` already holds by appointment.
    pub fn held_by(&self, applicant: ApplicantIdx) -> &[PositionIdx] {
        self.by_applicant
            .get(&applicant)
            .map_or(&[][..], Vec::as_slice)
    }

    /// How many seats in `position` are taken by appointment.
    pub fn count_at(&self, position: PositionIdx) -> usize {
        self.by_position.get(&position).map_or(0, Vec::len)
    }

    /// Every appointment as an `(applicant, position)` pair, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = Appointment> + '_ {
        self.by_applicant
            .iter()
            .flat_map(|(&applicant, positions)| {
                positions.iter().map(move |&position| Appointment {
                    applicant,
                    position,
                })
            })
    }

    /// Total number of appointments recorded.
    pub fn len(&self) -> usize {
        self.by_applicant.values().map(Vec::len).sum()
    }

    /// No appointments recorded.
    pub fn is_empty(&self) -> bool {
        self.by_applicant.is_empty()
    }
}

impl FromIterator<Appointment> for Appointments {
    fn from_iter<I: IntoIterator<Item = Appointment>>(iter: I) -> Self {
        let mut appointments = Appointments::new();
        for a in iter {
            appointments.insert(a.applicant, a.position);
        }
        appointments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appt(applicant: i32, position: i32) -> Appointment {
        Appointment {
            applicant: ApplicantIdx(applicant),
            position: PositionIdx(position),
        }
    }

    #[test]
    fn indexes_both_directions() {
        // Applicant 1 holds positions 10 and 20; position 10 is also held by 2.
        let appointments = Appointments::from_iter([appt(1, 10), appt(1, 20), appt(2, 10)]);

        assert_eq!(
            appointments.held_by(ApplicantIdx(1)),
            &[PositionIdx(10), PositionIdx(20)]
        );
        assert_eq!(appointments.held_by(ApplicantIdx(2)), &[PositionIdx(10)]);

        let mut holders = appointments.holders(PositionIdx(10)).to_vec();
        holders.sort_by_key(|a| a.0);
        assert_eq!(holders, vec![ApplicantIdx(1), ApplicantIdx(2)]);
        assert_eq!(appointments.holders(PositionIdx(20)), &[ApplicantIdx(1)]);
    }

    #[test]
    fn count_at_tallies_holders_per_position() {
        let appointments = Appointments::from_iter([appt(1, 10), appt(2, 10), appt(1, 20)]);
        assert_eq!(appointments.count_at(PositionIdx(10)), 2);
        assert_eq!(appointments.count_at(PositionIdx(20)), 1);
        // A position with no appointees counts zero, never panics.
        assert_eq!(appointments.count_at(PositionIdx(99)), 0);
    }

    #[test]
    fn empty_queries_are_safe() {
        let appointments = Appointments::new();
        assert!(appointments.is_empty());
        assert_eq!(appointments.len(), 0);
        assert!(appointments.holders(PositionIdx(1)).is_empty());
        assert!(appointments.held_by(ApplicantIdx(1)).is_empty());
        assert_eq!(appointments.iter().count(), 0);
    }

    #[test]
    fn iter_yields_every_pair() {
        let appointments = Appointments::from_iter([appt(1, 10), appt(1, 20), appt(2, 10)]);
        assert_eq!(appointments.len(), 3);

        let mut pairs: Vec<(i32, i32)> = appointments
            .iter()
            .map(|a| (a.applicant.0, a.position.0))
            .collect();
        pairs.sort();
        assert_eq!(pairs, vec![(1, 10), (1, 20), (2, 10)]);
    }
}
