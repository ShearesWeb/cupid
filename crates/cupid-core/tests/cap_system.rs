//! System test of the CCA caps over a realistic Sheares Hall: the committees
//! named in the AY26/27 white paper (Type A and Type B), sports and cultural
//! CCAs, JCRC and ad hoc roles. Everything goes through the public crate API
//! the desktop app uses: build a `Directory`, apply `CapSettings` (rules and
//! appeals), read the cap report and the UI snapshot, save and reload the
//! settings file, edit appointments, and change the rules for a new year.

use std::collections::BTreeMap;

use cupid::data::cap_settings;
use cupid::directory::{
    CapRules, CapSettings, CapWindow, Cca, CcaAppointment, CcaKind, CcaPosition, CcaTier, CcaType,
    CommitmentPeriod, CommitteeCapEntry, CommitteeCombo, Directory, PositionKind, TeamStatus, User,
};
use CommitmentPeriod::{FullYear, Semester1, Semester2};

const TYPE_A: [&str; 13] = [
    "ERC", "CMB", "DND", "OCSP", "SHM", "SL", "SMB", "SP", "SWOC", "VC", "RAG", "FLAG", "IHG Convening",
];
const TYPE_B: [&str; 5] = ["Block Committee (A)", "SECC", "HWAC", "ConSHeares", "SHeares Web"];
const SPORTS: [&str; 7] = ["Basketball (M)", "Badminton", "Floorball", "Volleyball (W)", "Swimming", "Table Tennis", "Squash"];
const CULTURE: [&str; 3] = ["Choir", "Dance", "Drama"];

/// The hall: CCAs with their positions, and residents added per scenario.
struct Hall {
    ccas: Vec<Cca>,
    positions: Vec<CcaPosition>,
    /// (CCA name, role) → position ID.
    seat: BTreeMap<(String, &'static str), i32>,
    users: Vec<User>,
    appointments: Vec<CcaAppointment>,
}

impl Hall {
    fn new() -> Self {
        let mut hall = Hall {
            ccas: Vec::new(),
            positions: Vec::new(),
            seat: BTreeMap::new(),
            users: Vec::new(),
            appointments: Vec::new(),
        };
        for name in TYPE_A {
            hall.add_cca(name, CcaKind::Committee, CcaType::TypeA, &[
                ("chair", PositionKind::Lead),
                ("vice", PositionKind::Vice),
                ("main", PositionKind::MainComm),
                ("sub", PositionKind::SubComm),
                ("sub2", PositionKind::SubComm),
            ]);
        }
        for name in TYPE_B {
            let member = if name.starts_with("Block") { PositionKind::BlockComm } else { PositionKind::Member };
            hall.add_cca(name, CcaKind::Committee, CcaType::TypeB, &[
                ("chair", PositionKind::Lead),
                ("member", member),
                ("resident", PositionKind::Resident),
            ]);
        }
        for name in SPORTS {
            hall.add_cca(name, CcaKind::Sports, CcaType::None, &[
                ("captain", PositionKind::Lead),
                ("member", PositionKind::Member),
                ("tm", PositionKind::TeamManager),
            ]);
        }
        for name in CULTURE {
            hall.add_cca(name, CcaKind::Culture, CcaType::None, &[
                ("ic", PositionKind::Lead),
                ("member", PositionKind::Member),
            ]);
        }
        hall.add_cca("JCRC", CcaKind::Jcrc, CcaType::None, &[("member", PositionKind::Member)]);
        hall.add_cca("Rag Day Helpers", CcaKind::Adhoc, CcaType::None, &[("member", PositionKind::Member)]);
        hall
    }

    fn add_cca(&mut self, name: &str, kind: CcaKind, cca_type: CcaType, roles: &[(&'static str, PositionKind)]) {
        let id = self.ccas.len() as i32 + 1;
        self.ccas.push(Cca {
            id,
            name: name.into(),
            kind,
            tier: CcaTier::None,
            cca_type,
            description: None,
            image_url: None,
        });
        for &(role, position_type) in roles {
            let pid = id * 100 + self.positions.len() as i32;
            self.positions.push(CcaPosition {
                id: pid,
                cca_id: id,
                reporting_position_id: None,
                position_type,
                name: format!("{name} {role}"),
                description: None,
                capacity: None,
            });
            self.seat.insert((name.into(), role), pid);
        }
    }

    fn cca_id(&self, name: &str) -> i32 {
        self.ccas.iter().find(|c| c.name == name).unwrap().id
    }

    /// A resident holding `(CCA, role, period)` seats; returns their user ID.
    fn resident(&mut self, name: &str, holdings: &[(&str, &'static str, CommitmentPeriod)]) -> i32 {
        let id = self.users.len() as i32 + 1;
        self.users.push(User {
            id,
            name: name.into(),
            email: format!("{}@u.nus.edu", name.to_lowercase().replace(' ', ".")),
        });
        for &(cca, role, period) in holdings {
            let position_id = *self
                .seat
                .get(&(cca.to_string(), role))
                .unwrap_or_else(|| panic!("no {role} seat in {cca}"));
            self.appointments.push(CcaAppointment {
                user_id: id,
                position_id,
                commitment_period: period,
                points: 0,
                team_status: TeamStatus::MainTeam,
                created_at: None,
            });
        }
        id
    }

    fn directory(&self) -> Directory {
        Directory::new(self.users.clone(), self.ccas.clone(), self.positions.clone(), self.appointments.clone()).unwrap()
    }
}

fn year(a: &str, role: &'static str) -> (&'static str, &'static str, CommitmentPeriod) {
    // Leaked so scenario tables can mix literals and built names.
    (Box::leak(a.to_string().into_boxed_str()), role, FullYear)
}

/// `n` distinct Sports/Cultural memberships.
fn sports_culture(n: usize) -> Vec<(&'static str, &'static str, CommitmentPeriod)> {
    SPORTS.iter().chain(CULTURE.iter()).take(n).map(|c| year(c, "member")).collect()
}

#[derive(Debug, PartialEq)]
struct Outcome {
    over_committee: bool,
    over_cca: bool,
    duplicate: bool,
    listed: bool,
}

fn outcome(report: &[CommitteeCapEntry], user_id: i32) -> Outcome {
    match report.iter().find(|e| e.user_id == user_id) {
        Some(e) => Outcome {
            over_committee: e.over_cap,
            over_cca: e.over_combined_cap,
            duplicate: !e.duplicate_ccas.is_empty(),
            listed: true,
        },
        None => Outcome { over_committee: false, over_cca: false, duplicate: false, listed: false },
    }
}

const CLEAN: Outcome = Outcome { over_committee: false, over_cca: false, duplicate: false, listed: false };

fn appeals(committee: &[i32], combined: &[i32]) -> CapSettings {
    CapSettings {
        committee_appeals: committee.iter().copied().collect(),
        combined_appeals: combined.iter().copied().collect(),
        ..Default::default()
    }
}

#[test]
fn every_resident_scenario_gets_the_white_paper_result() {
    let mut hall = Hall::new();
    let mut expect: Vec<(i32, &str, Outcome)> = Vec::new();
    let mut add = |hall: &mut Hall, name: &'static str, holdings: Vec<(&'static str, &'static str, CommitmentPeriod)>, o: Outcome| {
        let id = hall.resident(name, &holdings);
        expect.push((id, name, o));
        id
    };

    add(&mut hall, "Two subs two sports", [vec![year("SHM", "sub"), year("ERC", "sub")], sports_culture(2)].concat(), CLEAN);
    add(&mut hall, "Three subs three sports", [vec![year("SHM", "sub"), year("ERC", "sub"), year("DND", "sub")], sports_culture(3)].concat(), CLEAN);
    add(&mut hall, "One main one sub four sports", [vec![year("RAG", "main"), year("SL", "sub")], sports_culture(4)].concat(), CLEAN);
    add(&mut hall, "Block and SECC", vec![year("Block Committee (A)", "member"), year("SECC", "member")], CLEAN);
    add(&mut hall, "Chair and vice", vec![year("CMB", "chair"), year("VC", "vice")], CLEAN);
    add(
        &mut hall,
        "Block RAG main SHM sub",
        vec![year("Block Committee (A)", "member"), year("RAG", "main"), year("SHM", "sub")],
        Outcome { over_committee: true, over_cca: false, duplicate: false, listed: true },
    );
    let appealed_2m1s = add(
        &mut hall,
        "Same with committee appeal",
        vec![year("Block Committee (A)", "member"), year("RAG", "main"), year("SHM", "sub")],
        Outcome { over_committee: false, over_cca: false, duplicate: false, listed: true },
    );
    let three_main = add(
        &mut hall,
        "Three main with appeal",
        vec![year("CMB", "chair"), year("VC", "vice"), year("DND", "main")],
        Outcome { over_committee: true, over_cca: false, duplicate: false, listed: true },
    );
    let four_four = add(
        &mut hall,
        "Four sub four sports both appeals",
        [vec![year("ERC", "sub"), year("CMB", "sub"), year("DND", "sub"), year("OCSP", "sub")], sports_culture(4)].concat(),
        Outcome { over_committee: false, over_cca: false, duplicate: false, listed: true },
    );
    let four_four_one = add(
        &mut hall,
        "Four sub four sports committee appeal only",
        [vec![year("ERC", "sub"), year("CMB", "sub"), year("DND", "sub"), year("OCSP", "sub")], sports_culture(4)].concat(),
        Outcome { over_committee: false, over_cca: true, duplicate: false, listed: true },
    );
    add(&mut hall, "Six sports no committee", sports_culture(6), Outcome { over_committee: false, over_cca: true, duplicate: false, listed: true });
    let six_appealed = add(
        &mut hall,
        "Six sports with CCA appeal",
        sports_culture(6),
        Outcome { over_committee: false, over_cca: false, duplicate: false, listed: true },
    );
    add(
        &mut hall,
        "Team manager plus five sports plus main",
        [vec![year("SWOC", "main"), year("Squash", "tm")], sports_culture(5)].concat(),
        Outcome { over_committee: false, over_cca: true, duplicate: false, listed: true },
    );
    add(
        &mut hall,
        "Main and sub in one committee",
        vec![year("SHM", "main"), ("SHM", "sub", Semester2)],
        Outcome { over_committee: false, over_cca: false, duplicate: true, listed: true },
    );
    add(&mut hall, "Role change mid year", vec![("SHM", "main", Semester1), ("SHM", "sub", Semester2)], CLEAN);
    let jcrc_only = add(&mut hall, "JCRC and ad hoc only", vec![year("JCRC", "member"), year("Rag Day Helpers", "member")], CLEAN);
    add(&mut hall, "Block resident only", vec![year("Block Committee (A)", "resident")], CLEAN);

    let settings = appeals(&[appealed_2m1s, three_main, four_four, four_four_one], &[four_four, six_appealed]);
    let directory = hall.directory();
    let report = directory.committee_caps(&settings);
    let mut wrong = Vec::new();
    for (id, name, want) in &expect {
        let got = outcome(&report.entries, *id);
        if &got != want {
            wrong.push(format!("{name}: expected {want:?}, got {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{} scenario(s) wrong:\n{}", wrong.len(), wrong.join("\n"));
    assert!(directory.committee_cap_for(jcrc_only, &settings).is_none(), "JCRC and ad hoc count towards nothing");
    assert!(report.unclassified_ccas.is_empty(), "every committee in the hall has a type");
}

/// The white paper's caps, restated directly from its lists rather than
/// through `CapRules`, to check the engine against.
fn white_paper(main: usize, sub: usize, sc: usize, committee_appeal: bool, cca_appeal: bool) -> (bool, bool) {
    let standard = (main <= 2 && sub == 0) || (main <= 1 && sub <= 1) || (main == 0 && sub <= 3);
    let appealed = (main <= 2 && sub <= 1) || (main <= 1 && sub <= 2) || (main == 0 && sub <= 4);
    let over_committee = !if committee_appeal { appealed } else { standard };
    let sc_limit = if cca_appeal { 6 } else { 5 };
    let total_limit = match (committee_appeal, cca_appeal) {
        (true, true) => 8,
        (_, true) => 7,
        _ => 6,
    };
    (over_committee, sc > sc_limit || main + sub + sc > total_limit)
}

#[test]
fn engine_matches_the_white_paper_for_every_holding_and_appeal() {
    let mut hall = Hall::new();
    let mut cases = Vec::new();
    for main in 0..=3 {
        for sub in 0..=5 {
            for sc in 0..=8 {
                // Main from Type A Main-Comm seats, Sub from other Type A
                // committees, so no committee is held twice.
                let mut holdings: Vec<_> = TYPE_A[..main].iter().map(|c| year(c, "main")).collect();
                holdings.extend(TYPE_A[3..3 + sub].iter().map(|c| year(c, "sub")));
                holdings.extend(sports_culture(sc));
                let id = hall.resident(&format!("{main}M{sub}S{sc}"), &holdings);
                cases.push((id, main, sub, sc));
            }
        }
    }
    let directory = hall.directory();
    let ids: Vec<i32> = cases.iter().map(|c| c.0).collect();
    let mut checked = 0;
    for (committee_appeal, cca_appeal) in [(false, false), (true, false), (false, true), (true, true)] {
        let settings = appeals(
            if committee_appeal { &ids } else { &[] },
            if cca_appeal { &ids } else { &[] },
        );
        for &(id, main, sub, sc) in &cases {
            let entry = directory.committee_cap_for(id, &settings);
            let got = entry.as_ref().map_or((false, false), |e| (e.over_cap, e.over_combined_cap));
            let want = white_paper(main, sub, sc, committee_appeal, cca_appeal);
            assert_eq!(
                got, want,
                "{main}M {sub}S {sc} Sports/Culture, committee appeal {committee_appeal}, CCA cap appeal {cca_appeal}"
            );
            if let Some(e) = entry {
                assert_eq!((e.counts[0].main, e.counts[0].sub, e.counts[0].sports_culture), (main, sub, sc));
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 4 * 6 * 9 * 4);
}

#[test]
fn saved_settings_reload_to_the_same_report() {
    let mut hall = Hall::new();
    let over = hall.resident("Over", &[year("Block Committee (A)", "member"), year("RAG", "main"), year("SHM", "sub")]);
    let appealed = hall.resident("Appealed", &sports_culture(6));
    let directory = hall.directory();

    let mut settings = appeals(&[], &[appealed]);
    settings.window = CapWindow::AcademicYear;
    settings.rules.academic_year = "AY26/27".into();
    let dir = std::env::temp_dir().join(format!("cupid-cap-system-{}", std::process::id()));
    let path = dir.join("appeals.json");
    cap_settings::write_file(&path, &settings).unwrap();
    let reloaded = cap_settings::read_file(&path).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(reloaded, settings);
    assert_eq!(directory.committee_caps(&reloaded), directory.committee_caps(&settings));
    let report = directory.committee_caps(&reloaded);
    assert!(outcome(&report.entries, over).over_committee);
    assert!(!outcome(&report.entries, appealed).over_cca);
}

#[test]
fn a_new_years_rules_and_committee_types_take_effect() {
    let mut hall = Hall::new();
    let two_main = hall.resident("Two main", &[year("Block Committee (A)", "member"), year("SECC", "member")]);
    let secc_sub = hall.resident("Two subs", &[year("ERC", "sub"), year("CMB", "sub")]);
    let five_sports = hall.resident("Five sports", &sports_culture(5));
    let directory = hall.directory();
    assert!(directory.committee_caps(&CapSettings::default()).entries.is_empty(), "all within AY26/27 caps");

    // A stricter year: 1M1S or 2S, at most 4 Sports/Culture, at most 5 CCAs,
    // and CMB moved to Type B (its Sub-Comm seats now count as Main).
    let next_year = CapSettings {
        rules: CapRules {
            academic_year: "AY27/28".into(),
            committee_combos: vec![CommitteeCombo { main: 1, sub: 1 }, CommitteeCombo { main: 0, sub: 2 }],
            sports_culture_max: 4,
            total_max: 5,
            cca_types: [(hall.cca_id("CMB"), CcaType::TypeB)].into(),
            ..CapRules::default()
        },
        ..Default::default()
    };
    next_year.rules.validate().unwrap();
    let report = directory.committee_caps(&next_year);
    assert!(outcome(&report.entries, two_main).over_committee, "2M is no longer allowed");
    let retyped = directory.committee_cap_for(secc_sub, &next_year).unwrap();
    assert_eq!((retyped.counts[0].main, retyped.counts[0].sub), (1, 1), "CMB is Type B now");
    assert!(!retyped.over_cap, "1M1S is still allowed");
    assert!(outcome(&report.entries, five_sports).over_cca, "5 Sports/Culture is over the new cap of 4");
    assert_eq!(report.rules.academic_year, "AY27/28");
}

#[test]
fn edits_and_previews_update_the_flags() {
    let mut hall = Hall::new();
    let resident = hall.resident("Busy", &[year("RAG", "main"), year("SHM", "sub")]);
    let mut directory = hall.directory();
    let settings = CapSettings::default();
    assert!(directory.committee_caps(&settings).entries.is_empty(), "1M1S is within the cap");

    // The add-member preview: the same edit on a copy, nothing changed yet.
    let erc_sub = hall.seat[&("ERC".to_string(), "sub")];
    let mut preview = directory.clone();
    preview.add_appointment(resident, erc_sub, FullYear).unwrap();
    assert!(preview.committee_cap_for(resident, &settings).unwrap().over_cap, "1M2S would be over");
    assert!(directory.committee_caps(&settings).entries.is_empty(), "the preview changed nothing");

    // The real edit flags them; removing the seat clears it again.
    directory.add_appointment(resident, erc_sub, FullYear).unwrap();
    assert!(outcome(&directory.committee_caps(&settings).entries, resident).over_committee);
    directory.remove_appointment(resident, erc_sub).unwrap();
    assert!(directory.committee_caps(&settings).entries.is_empty());
}

#[test]
fn semester_counting_checks_each_semester_on_its_own() {
    let mut hall = Hall::new();
    let resident = hall.resident(
        "Split year",
        &[("ERC", "main", Semester1), ("CMB", "main", Semester1), ("DND", "main", Semester2), ("SL", "main", Semester2)],
    );
    let directory = hall.directory();
    assert!(outcome(&directory.committee_caps(&CapSettings::default()).entries, resident).over_committee, "4M across the year");
    let per_semester = CapSettings { window: CapWindow::Semester, ..Default::default() };
    assert_eq!(outcome(&directory.committee_caps(&per_semester).entries, resident), CLEAN, "2M in each semester");
}

#[test]
fn the_ui_snapshot_carries_the_report_and_rules() {
    let mut hall = Hall::new();
    let resident = hall.resident("Over", &[year("Block Committee (A)", "member"), year("RAG", "main"), year("SHM", "sub")]);
    let json = serde_json::to_value(hall.directory().snapshot()).unwrap();
    let caps = &json["committeeCaps"];
    assert_eq!(caps["rules"]["academicYear"], "AY26/27");
    assert_eq!(caps["rules"]["committeeCombos"][0], serde_json::json!({ "main": 2, "sub": 0 }));
    assert_eq!(caps["entries"][0]["userId"], resident);
    assert_eq!(caps["entries"][0]["overCap"], true);
    assert_eq!(caps["entries"][0]["counts"][0]["main"], 2);
    assert_eq!(caps["entries"][0]["counts"][0]["totalLimit"], 6);
}
