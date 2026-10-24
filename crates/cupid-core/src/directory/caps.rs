//! CCA caps from the hall's CCA points white paper, adjustable each academic
//! year on the Cap settings page ([`CapRules`]).
//!
//! Every committee seat is either Main or Sub. Type A Sub-Comm seats are Sub;
//! every other committee seat (Type A Main-Comm, chairs, vices and members, and
//! all Type B seats) is Main. Three caps are checked, each on its own:
//!
//! - **committee**: a list of allowed Main + Sub combinations; a holding is
//!   allowed when it fits inside any one of them. An approved committee appeal
//!   switches to the appealed list;
//! - **Sports/Culture**: the most Sports and Cultural CCAs; an approved CCA cap
//!   appeal raises it;
//! - **total**: committee seats plus Sports/Cultural CCAs; an approved CCA cap
//!   appeal raises it, and with both appeals it rises again (which is how
//!   4 Sub + 4 Sports/Culture reaches 8 under this year's rules).
//!
//! The caps apply to every resident, with or without a committee seat. The
//! report only flags; it never blocks an edit, since the JCRC can approve
//! exceptions.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    Cca, CcaAppointment, CcaKind, CcaPosition, CcaType, CommitmentPeriod, Directory, PositionKind,
};

/// One allowed committee holding: at most `main` Main seats and `sub` Sub
/// seats together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeCombo {
    pub main: usize,
    pub sub: usize,
}

const fn combo(main: usize, sub: usize) -> CommitteeCombo {
    CommitteeCombo { main, sub }
}

/// This year's caps and committee types, edited on the Cap settings page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CapRules {
    /// Label only, e.g. "AY26/27".
    pub academic_year: String,
    /// Allowed committee holdings without an appeal (e.g. 2M, 1M1S, 3S).
    pub committee_combos: Vec<CommitteeCombo>,
    /// Allowed committee holdings with an approved committee appeal.
    pub committee_combos_appealed: Vec<CommitteeCombo>,
    pub sports_culture_max: usize,
    /// With an approved CCA cap appeal.
    pub sports_culture_max_appealed: usize,
    /// Committee seats plus Sports/Cultural CCAs.
    pub total_max: usize,
    /// With an approved CCA cap appeal.
    pub total_max_appealed: usize,
    /// With both the committee and the CCA cap appeal approved.
    pub total_max_both_appeals: usize,
    /// Type A/B set on the Cap settings page, by CCA ID; overrides the
    /// database's `ccas.type` for committees listed here.
    pub cca_types: BTreeMap<i32, CcaType>,
}

impl Default for CapRules {
    /// The AY26/27 white paper.
    fn default() -> Self {
        Self {
            academic_year: "AY26/27".into(),
            committee_combos: vec![combo(2, 0), combo(1, 1), combo(0, 3)],
            committee_combos_appealed: vec![combo(2, 1), combo(1, 2), combo(0, 4)],
            sports_culture_max: 5,
            sports_culture_max_appealed: 6,
            total_max: 6,
            total_max_appealed: 7,
            total_max_both_appeals: 8,
            cca_types: BTreeMap::new(),
        }
    }
}

/// Upper bound for any single cap, so a typo cannot disable a check.
const CAP_LIMIT: usize = 50;
/// The most combinations one committee list may have.
const MAX_COMBOS: usize = 10;

impl CapRules {
    /// A holding is allowed when it fits inside any one listed combination.
    pub fn committee_allows(&self, main: usize, sub: usize, appealed: bool) -> bool {
        let combos = if appealed {
            &self.committee_combos_appealed
        } else {
            &self.committee_combos
        };
        combos.iter().any(|c| main <= c.main && sub <= c.sub)
    }

    pub fn sports_culture_limit(&self, cca_appealed: bool) -> usize {
        if cca_appealed {
            self.sports_culture_max_appealed
        } else {
            self.sports_culture_max
        }
    }

    pub fn total_limit(&self, committee_appealed: bool, cca_appealed: bool) -> usize {
        match (committee_appealed, cca_appealed) {
            (true, true) => self.total_max_both_appeals,
            (_, true) => self.total_max_appealed,
            _ => self.total_max,
        }
    }

    /// The committee's type: this year's setting, else the database's.
    pub fn cca_type(&self, cca: &Cca) -> CcaType {
        self.cca_types.get(&cca.id).copied().unwrap_or(cca.cca_type)
    }

    /// `None` when the seat does not count towards the committee cap:
    /// non-committee CCAs (sports, culture, JCRC, ad hoc), team managers and
    /// residents.
    pub fn committee_level(&self, cca: &Cca, position: &CcaPosition) -> Option<CommitteeLevel> {
        if cca.kind != CcaKind::Committee {
            return None;
        }
        match position.position_type {
            PositionKind::TeamManager | PositionKind::Resident => None,
            PositionKind::SubComm if self.cca_type(cca) != CcaType::TypeB => {
                Some(CommitteeLevel::Sub)
            }
            _ => Some(CommitteeLevel::Main),
        }
    }

    /// Refuses settings that would make a check meaningless or unbounded.
    pub fn validate(&self) -> Result<(), String> {
        if self.academic_year.chars().count() > 40 {
            return Err("The academic year label is too long (40 characters at most).".into());
        }
        for (name, combos) in [
            ("without appeal", &self.committee_combos),
            ("with appeal", &self.committee_combos_appealed),
        ] {
            if combos.is_empty() || combos.len() > MAX_COMBOS {
                return Err(format!(
                    "The committee cap {name} needs between 1 and {MAX_COMBOS} combinations."
                ));
            }
            if combos.iter().any(|c| c.main > CAP_LIMIT || c.sub > CAP_LIMIT) {
                return Err(format!("A committee combination {name} is above {CAP_LIMIT}."));
            }
        }
        for (name, value) in [
            ("Sports/Culture cap", self.sports_culture_max),
            ("Sports/Culture cap with appeal", self.sports_culture_max_appealed),
            ("total cap", self.total_max),
            ("total cap with CCA cap appeal", self.total_max_appealed),
            ("total cap with both appeals", self.total_max_both_appeals),
        ] {
            if value > CAP_LIMIT {
                return Err(format!("The {name} is above {CAP_LIMIT}."));
            }
        }
        if self.cca_types.values().any(|t| *t == CcaType::None) {
            return Err("A committee type must be Type A or Type B.".into());
        }
        Ok(())
    }
}

/// Whether seats are counted across the whole academic year or per semester.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapWindow {
    #[default]
    AcademicYear,
    Semester,
}

/// Operator decisions local to this machine, like preallocations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapSettings {
    #[serde(default)]
    pub window: CapWindow,
    /// Residents whose appeal to exceed the standard committee cap was approved.
    #[serde(default)]
    pub committee_appeals: BTreeSet<i32>,
    /// Residents whose appeal to exceed the Sports/Culture and total caps was approved.
    #[serde(default)]
    pub combined_appeals: BTreeSet<i32>,
    /// This year's caps and committee types.
    #[serde(default)]
    pub rules: CapRules,
}

/// Which cap an appeal lifts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppealKind {
    /// Committee combinations: switches to the appealed combination list.
    Committee,
    /// Sports/Culture and total: raises both to their appealed limits.
    Combined,
}

/// One staged appeal decision, saved together with the rest of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppealChange {
    pub user_id: i32,
    pub kind: AppealKind,
    pub appealed: bool,
}

impl CapSettings {
    /// Apply staged decisions in order; a later change to the same appeal wins.
    pub fn apply_appeals(&mut self, changes: &[AppealChange]) {
        for change in changes {
            let appeals = match change.kind {
                AppealKind::Committee => &mut self.committee_appeals,
                AppealKind::Combined => &mut self.combined_appeals,
            };
            if change.appealed {
                appeals.insert(change.user_id);
            } else {
                appeals.remove(&change.user_id);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommitteeLevel {
    Main,
    Sub,
}

/// Whether the seat makes the resident part of a Sports or Cultural CCA.
/// Team managers count; resident roles do not.
pub fn counts_as_sports_culture(cca: &Cca, position: &CcaPosition) -> bool {
    matches!(cca.kind, CcaKind::Sports | CcaKind::Culture)
        && position.position_type != PositionKind::Resident
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeSeat {
    pub cca_id: i32,
    pub position_id: i32,
    pub level: CommitteeLevel,
}

/// Seat counts in one window: the academic year (`semester: None`) or one semester.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeCapCount {
    pub semester: Option<u8>,
    pub main: usize,
    pub sub: usize,
    /// Distinct Sports and Cultural CCAs.
    pub sports_culture: usize,
    /// Fits no committee combination without appeal.
    pub exceeds_standard: bool,
    /// Fits no committee combination with appeal.
    pub exceeds_appeal: bool,
    /// Over the Sports/Culture cap without appeal.
    pub exceeds_sports_culture: bool,
    /// The Sports/Culture limit that applies, given their appeals.
    pub sports_culture_limit: usize,
    /// Over the total cap without appeal.
    pub exceeds_total: bool,
    /// The total limit that applies, given their appeals.
    pub total_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeCapEntry {
    pub user_id: i32,
    pub appealed: bool,
    /// Breaks the committee cap that applies to them: the appealed list if
    /// appealed, otherwise the standard one.
    pub over_cap: bool,
    pub combined_appealed: bool,
    /// Breaks the Sports/Culture or total cap that applies to them.
    pub over_combined_cap: bool,
    pub counts: Vec<CommitteeCapCount>,
    /// CCAs where they hold two committee seats in an overlapping period.
    pub duplicate_ccas: Vec<i32>,
    pub seats: Vec<CommitteeSeat>,
    /// Sports and Cultural CCAs held in the academic year, sorted.
    pub sports_culture_ccas: Vec<i32>,
}

impl CommitteeCapEntry {
    /// Worth showing: broken cap, duplicate seat, or an appeal to review.
    pub fn is_flagged(&self) -> bool {
        self.over_cap
            || self.over_combined_cap
            || !self.duplicate_ccas.is_empty()
            || self.appealed
            || self.combined_appealed
    }
}

/// A Sports or Cultural CCA held in some period.
type SportsCultureSeat = (i32, CommitmentPeriod);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeCapReport {
    pub window: CapWindow,
    /// The rules the report was checked against, for the UI to show.
    pub rules: CapRules,
    /// Flagged residents only, sorted by user ID.
    pub entries: Vec<CommitteeCapEntry>,
    /// Committees with no type in the database or the settings: their Sub-Comm
    /// seats count as Sub, but the type should be set.
    pub unclassified_ccas: Vec<i32>,
}

impl Directory {
    fn committee_seats<'a>(
        &'a self,
        rules: &'a CapRules,
    ) -> impl Iterator<Item = (&'a CcaAppointment, CommitteeSeat)> + 'a {
        self.appointments().filter_map(move |appointment| {
            let position = self.position(appointment.position_id)?;
            let level = rules.committee_level(self.cca(position.cca_id)?, position)?;
            Some((
                appointment,
                CommitteeSeat {
                    cca_id: position.cca_id,
                    position_id: position.id,
                    level,
                },
            ))
        })
    }

    /// Every Sports/Cultural seat, with its holder.
    fn sports_culture_seats(&self) -> impl Iterator<Item = (i32, SportsCultureSeat)> + '_ {
        self.appointments().filter_map(|appointment| {
            let position = self.position(appointment.position_id)?;
            let cca = self.cca(position.cca_id)?;
            counts_as_sports_culture(cca, position)
                .then_some((appointment.user_id, (cca.id, appointment.commitment_period)))
        })
    }

    pub fn committee_caps(&self, settings: &CapSettings) -> CommitteeCapReport {
        type Seats<'a> = (Vec<(&'a CcaAppointment, CommitteeSeat)>, Vec<SportsCultureSeat>);
        let rules = &settings.rules;
        let mut by_user: BTreeMap<i32, Seats> = BTreeMap::new();
        for (appointment, seat) in self.committee_seats(rules) {
            by_user.entry(appointment.user_id).or_default().0.push((appointment, seat));
        }
        for (user_id, seat) in self.sports_culture_seats() {
            by_user.entry(user_id).or_default().1.push(seat);
        }
        // Appealed residents stay listed after they drop their extra seats, so
        // the operator can still see and clear the appeal.
        for &user_id in settings.committee_appeals.iter().chain(&settings.combined_appeals) {
            if self.user(user_id).is_some() {
                by_user.entry(user_id).or_default();
            }
        }
        let entries = by_user
            .into_iter()
            .map(|(user_id, (seats, sports_culture))| {
                cap_entry(user_id, &seats, &sports_culture, settings)
            })
            .filter(CommitteeCapEntry::is_flagged)
            .collect();
        let unclassified_ccas = self
            .ccas()
            .filter(|c| c.kind == CcaKind::Committee && rules.cca_type(c) == CcaType::None)
            .map(|c| c.id)
            .collect();
        CommitteeCapReport {
            window: settings.window,
            rules: rules.clone(),
            entries,
            unclassified_ccas,
        }
    }

    /// One resident's entry, flagged or not; `None` if they hold no counted
    /// seat at all. Used for profiles and to preview an edit on a cloned
    /// directory.
    pub fn committee_cap_for(&self, user_id: i32, settings: &CapSettings) -> Option<CommitteeCapEntry> {
        let seats: Vec<_> = self
            .committee_seats(&settings.rules)
            .filter(|(a, _)| a.user_id == user_id)
            .collect();
        let sports_culture: Vec<_> = self
            .sports_culture_seats()
            .filter(|(id, _)| *id == user_id)
            .map(|(_, seat)| seat)
            .collect();
        (!seats.is_empty() || !sports_culture.is_empty())
            .then(|| cap_entry(user_id, &seats, &sports_culture, settings))
    }
}

fn cap_entry(
    user_id: i32,
    seats: &[(&CcaAppointment, CommitteeSeat)],
    sports_culture: &[SportsCultureSeat],
    settings: &CapSettings,
) -> CommitteeCapEntry {
    let rules = &settings.rules;
    let appealed = settings.committee_appeals.contains(&user_id);
    let combined_appealed = settings.combined_appeals.contains(&user_id);
    let in_semester = |period: CommitmentPeriod, semester: Option<u8>| {
        semester.is_none_or(|s| period.semesters().contains(&s))
    };
    let windows: &[Option<u8>] = match settings.window {
        CapWindow::AcademicYear => &[None],
        CapWindow::Semester => &[Some(1), Some(2)],
    };
    let sports_culture_limit = rules.sports_culture_limit(combined_appealed);
    let total_limit = rules.total_limit(appealed, combined_appealed);
    let counts: Vec<CommitteeCapCount> = windows
        .iter()
        .map(|&semester| {
            let (mut main, mut sub) = (0, 0);
            for (_, seat) in seats
                .iter()
                .filter(|(a, _)| in_semester(a.commitment_period, semester))
            {
                match seat.level {
                    CommitteeLevel::Main => main += 1,
                    CommitteeLevel::Sub => sub += 1,
                }
            }
            let sports_culture = sports_culture
                .iter()
                .filter(|(_, period)| in_semester(*period, semester))
                .map(|(cca_id, _)| cca_id)
                .collect::<BTreeSet<_>>()
                .len();
            let total = main + sub + sports_culture;
            CommitteeCapCount {
                semester,
                main,
                sub,
                sports_culture,
                exceeds_standard: !rules.committee_allows(main, sub, false),
                exceeds_appeal: !rules.committee_allows(main, sub, true),
                exceeds_sports_culture: sports_culture > rules.sports_culture_max,
                sports_culture_limit,
                exceeds_total: total > rules.total_max,
                total_limit,
            }
        })
        .collect();
    let over_cap = counts.iter().any(|c| {
        if appealed {
            c.exceeds_appeal
        } else {
            c.exceeds_standard
        }
    });
    let over_combined_cap = counts.iter().any(|c| {
        c.sports_culture > c.sports_culture_limit
            || c.main + c.sub + c.sports_culture > c.total_limit
    });
    let mut duplicate_ccas = BTreeSet::new();
    for (i, (a, seat)) in seats.iter().enumerate() {
        for (b, other) in &seats[i + 1..] {
            if seat.cca_id == other.cca_id && a.commitment_period.overlaps(b.commitment_period) {
                duplicate_ccas.insert(seat.cca_id);
            }
        }
    }
    CommitteeCapEntry {
        user_id,
        appealed,
        over_cap,
        combined_appealed,
        over_combined_cap,
        counts,
        duplicate_ccas: duplicate_ccas.into_iter().collect(),
        seats: seats.iter().map(|(_, seat)| seat.clone()).collect(),
        sports_culture_ccas: sports_culture
            .iter()
            .map(|(cca_id, _)| *cca_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_committee_combinations_match_the_white_paper() {
        let rules = CapRules::default();
        for (main, sub) in [(0, 0), (0, 3), (1, 0), (1, 1), (2, 0)] {
            assert!(rules.committee_allows(main, sub, false), "{main}M{sub}S");
        }
        for (main, sub) in [(0, 4), (1, 2), (2, 1), (3, 0)] {
            assert!(!rules.committee_allows(main, sub, false), "{main}M{sub}S");
        }
        for (main, sub) in [(0, 4), (1, 2), (2, 1)] {
            assert!(rules.committee_allows(main, sub, true), "{main}M{sub}S");
        }
        for (main, sub) in [(0, 5), (1, 3), (2, 2), (3, 0)] {
            assert!(!rules.committee_allows(main, sub, true), "{main}M{sub}S");
        }
    }

    #[test]
    fn total_limit_rises_with_each_appeal() {
        let rules = CapRules::default();
        assert_eq!(rules.total_limit(false, false), 6);
        assert_eq!(rules.total_limit(true, false), 6, "the committee appeal alone");
        assert_eq!(rules.total_limit(false, true), 7);
        assert_eq!(rules.total_limit(true, true), 8);
        assert_eq!(rules.sports_culture_limit(false), 5);
        assert_eq!(rules.sports_culture_limit(true), 6);
    }

    #[test]
    fn a_holding_is_allowed_when_it_fits_inside_any_combination() {
        let rules = CapRules {
            committee_combos: vec![combo(3, 0), combo(1, 2)],
            ..CapRules::default()
        };
        assert!(rules.committee_allows(3, 0, false));
        assert!(rules.committee_allows(2, 0, false), "smaller than 3M");
        assert!(rules.committee_allows(1, 2, false));
        assert!(rules.committee_allows(0, 2, false), "smaller than 1M2S");
        assert!(!rules.committee_allows(0, 3, false));
        assert!(!rules.committee_allows(2, 1, false), "fits neither");
        assert!(!rules.committee_allows(4, 0, false));
    }

    #[test]
    fn validation_refuses_empty_tables_huge_caps_and_unset_types() {
        assert!(CapRules::default().validate().is_ok());
        let empty = CapRules {
            committee_combos_appealed: vec![],
            ..CapRules::default()
        };
        assert!(empty.validate().is_err());
        let huge = CapRules {
            total_max: 500,
            ..CapRules::default()
        };
        assert!(huge.validate().is_err());
        let unset = CapRules {
            cca_types: [(1, CcaType::None)].into(),
            ..CapRules::default()
        };
        assert!(unset.validate().is_err());
    }

    #[test]
    fn settings_files_without_rules_load_this_years_defaults() {
        let settings: CapSettings =
            serde_json::from_str(r#"{"window":"semester","committeeAppeals":[2]}"#).unwrap();
        assert_eq!(settings.rules, CapRules::default());
        let partial: CapRules =
            serde_json::from_str(r#"{"totalMax":5,"ccaTypes":{"3":"type-b"}}"#).unwrap();
        assert_eq!(partial.total_max, 5);
        assert_eq!(partial.committee_combos, CapRules::default().committee_combos);
        assert_eq!(partial.cca_types[&3], CcaType::TypeB);
    }

    #[test]
    fn appeal_batches_approve_and_clear_each_kind_independently() {
        let mut settings = CapSettings {
            committee_appeals: [1, 2].into(),
            combined_appeals: [2].into(),
            ..Default::default()
        };
        let changes: Vec<AppealChange> = serde_json::from_str(
            r#"[
                {"userId": 1, "kind": "committee", "appealed": false},
                {"userId": 3, "kind": "combined", "appealed": true},
                {"userId": 2, "kind": "combined", "appealed": false},
                {"userId": 2, "kind": "combined", "appealed": true}
            ]"#,
        )
        .unwrap();
        settings.apply_appeals(&changes);
        assert_eq!(settings.committee_appeals, [2].into());
        assert_eq!(settings.combined_appeals, [2, 3].into());
    }
}
