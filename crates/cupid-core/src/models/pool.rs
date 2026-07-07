use std::collections::HashMap;

use super::allocation::Algorithm;
use super::applicant::{Applicant, ApplicantIdx};
use super::appointment::Appointments;
use super::position::{Position, PositionIdx};

/// Every applicant and every position, owned and keyed by id, plus the existing
/// appointments tying them together.
pub struct Pool {
    applicants: HashMap<ApplicantIdx, Applicant>,
    positions: HashMap<PositionIdx, Position>,
    appointments: Appointments,
}

impl Pool {
    pub fn new(applicants: Vec<Applicant>, positions: Vec<Position>) -> Self {
        Pool {
            applicants: applicants.into_iter().map(|a| (a.id, a)).collect(),
            positions: positions.into_iter().map(|p| (p.id, p)).collect(),
            appointments: Appointments::new(),
        }
    }

    /// Builder: attach the existing appointments (set once during corpus assembly).
    pub fn with_appointments(mut self, appointments: Appointments) -> Self {
        self.appointments = appointments;
        self
    }

    pub fn applicants(&self) -> impl Iterator<Item = &Applicant> + '_ {
        self.applicants.values()
    }

    pub fn positions(&self) -> impl Iterator<Item = &Position> + '_ {
        self.positions.values()
    }

    /// The existing appointments, queryable by applicant or by position.
    pub fn appointments(&self) -> &Appointments {
        &self.appointments
    }

    /// Applicant by id, O(1).
    pub fn applicant(&self, id: ApplicantIdx) -> Option<&Applicant> {
        self.applicants.get(&id)
    }

    /// Position by id, O(1).
    pub fn position(&self, id: PositionIdx) -> Option<&Position> {
        self.positions.get(&id)
    }
}

/// A borrowed, read-only index over every applicant and the positions.
pub struct Roster<'a> {
    applicants: HashMap<ApplicantIdx, &'a Applicant>,
    positions: HashMap<PositionIdx, &'a Position>,
}

impl<'a> Roster<'a> {
    /// Index every applicant, and only the positions routed to `algorithm`.
    pub fn for_algorithm(
        applicants: impl Iterator<Item = &'a Applicant>,
        positions: impl Iterator<Item = &'a Position>,
        algorithm: Algorithm,
    ) -> Self {
        Roster {
            applicants: applicants.map(|a| (a.id, a)).collect(),
            positions: positions
                .filter(|p| p.algorithm() == algorithm)
                .map(|p| (p.id, p))
                .collect(),
        }
    }

    /// Look up an applicant by id.
    pub fn applicant(&self, id: ApplicantIdx) -> Option<&'a Applicant> {
        self.applicants.get(&id).copied()
    }

    /// Look up a position by id.
    pub fn position(&self, id: PositionIdx) -> Option<&'a Position> {
        self.positions.get(&id).copied()
    }

    /// All applicants in this roster.
    pub fn applicants(&self) -> impl Iterator<Item = &'a Applicant> + '_ {
        self.applicants.values().copied()
    }

    /// The positions in this roster.
    pub fn positions(&self) -> impl Iterator<Item = &'a Position> + '_ {
        self.positions.values().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionType;

    // ---- owned corpus (`Pool`) ----

    #[test]
    fn pool_accessors_expose_corpus() {
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let positions = vec![Position::new(
            10,
            "C".into(),
            "Head".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )];

        let pool = Pool::new(applicants, positions);

        assert_eq!(pool.applicants().count(), 1);
        assert_eq!(pool.positions().count(), 1);
    }

    #[test]
    fn id_accessors_find_by_id() {
        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![Position::new(
            10,
            "C".into(),
            "Head".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )];
        let pool = Pool::new(applicants, positions);

        assert_eq!(pool.applicant(ApplicantIdx(1)).unwrap().name, "Ann");
        assert!(pool.applicant(ApplicantIdx(99)).is_none());
        assert_eq!(pool.position(PositionIdx(10)).unwrap().capacity, 1);
        assert!(pool.position(PositionIdx(99)).is_none());
    }

    #[test]
    fn appointments_default_empty_and_queryable_both_ways() {
        use crate::models::{Appointment, Appointments};

        let applicants = vec![Applicant::new(1, "Ann".into(), "a@x".into(), vec![])];
        let positions = vec![Position::new(
            10,
            "C".into(),
            "Head".into(),
            None,
            2,
            PositionType::MainComm,
            vec![],
        )];

        // A bare pool carries no appointments.
        let bare = Pool::new(applicants.clone(), positions.clone());
        assert!(bare.appointments().is_empty());

        // The builder threads the relation through, queryable from either side.
        let pool = Pool::new(applicants, positions).with_appointments(Appointments::from_iter([
            Appointment {
                applicant: ApplicantIdx(1),
                position: PositionIdx(10),
            },
        ]));
        assert_eq!(pool.appointments().held_by(ApplicantIdx(1)), &[PositionIdx(10)]);
        assert_eq!(pool.appointments().holders(PositionIdx(10)), &[ApplicantIdx(1)]);
    }

    // ---- per-algorithm view (`Roster`) ----

    fn fixture() -> (Vec<Applicant>, Vec<Position>) {
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![]),
        ];
        let positions = vec![
            Position::new(
                10,
                "C".into(),
                "Block".into(),
                None,
                1,
                PositionType::BlockComm,
                vec![],
            ),
            Position::new(
                20,
                "C".into(),
                "Main".into(),
                None,
                1,
                PositionType::MainComm,
                vec![],
            ),
            Position::new(
                30,
                "C".into(),
                "Sub".into(),
                None,
                1,
                PositionType::SubComm,
                vec![],
            ),
        ];
        (applicants, positions)
    }

    #[test]
    fn ia_roster_keeps_only_blockcomm() {
        let (applicants, positions) = fixture();
        let roster = Roster::for_algorithm(applicants.iter(), positions.iter(), Algorithm::ImmediateAcceptance);
        assert_eq!(roster.positions().count(), 1);
        assert!(
            roster.position(PositionIdx(10)).is_some(),
            "blockcomm in scope"
        );
        assert!(
            roster.position(PositionIdx(20)).is_none(),
            "maincomm filtered out"
        );
        assert!(
            roster.position(PositionIdx(30)).is_none(),
            "subcomm filtered out"
        );
    }

    #[test]
    fn gs_roster_keeps_main_and_sub() {
        let (applicants, positions) = fixture();
        let roster = Roster::for_algorithm(applicants.iter(), positions.iter(), Algorithm::GaleShapley);
        assert_eq!(roster.positions().count(), 2);
        assert!(roster.position(PositionIdx(20)).is_some());
        assert!(roster.position(PositionIdx(30)).is_some());
        assert!(
            roster.position(PositionIdx(10)).is_none(),
            "blockcomm filtered out"
        );
    }

    #[test]
    fn every_applicant_is_present_in_each_roster() {
        // The roster partitions positions by algorithm but never drops applicants.
        let (applicants, positions) = fixture();
        for algo in [Algorithm::ImmediateAcceptance, Algorithm::GaleShapley] {
            let roster = Roster::for_algorithm(applicants.iter(), positions.iter(), algo);
            assert_eq!(roster.applicants().count(), 2);
            assert!(roster.applicant(ApplicantIdx(1)).is_some());
            assert!(roster.applicant(ApplicantIdx(2)).is_some());
            assert!(roster.applicant(ApplicantIdx(99)).is_none());
        }
    }
}
