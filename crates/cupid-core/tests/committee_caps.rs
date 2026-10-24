use cupid::directory::{
    CapRules, CapSettings, CapWindow, Cca, CommitteeCombo, CcaAppointment, CcaKind, CcaPosition, CcaTier, CcaType,
    CommitmentPeriod, Directory, PositionKind, TeamStatus, User,
};
use CommitmentPeriod::{FullYear, Semester1, Semester2};

// CCA ids: 1-4 Type A committees, 5-6 Type B committees, 7 sports, 8 untyped committee,
// 9-13 sports, 14-15 culture.
// Position id = cca * 10 + offset: 0 lead, 1 maincomm, 2 subcomm, 3 team manager,
// 4 member, 5 blockcomm.
const LEAD: i32 = 0;
const MAIN: i32 = 1;
const SUB: i32 = 2;
const TM: i32 = 3;
const MEMBER: i32 = 4;
const BLOCK: i32 = 5;

fn pos(cca: i32, offset: i32) -> i32 {
    cca * 10 + offset
}

fn cca(id: i32, kind: CcaKind, cca_type: CcaType) -> Cca {
    Cca {
        id,
        name: format!("C{id}"),
        kind,
        tier: CcaTier::None,
        cca_type,
        description: None,
        image_url: None,
    }
}

fn fixture(holdings: &[(i32, i32, CommitmentPeriod)]) -> Directory {
    let ccas = vec![
        cca(1, CcaKind::Committee, CcaType::TypeA),
        cca(2, CcaKind::Committee, CcaType::TypeA),
        cca(3, CcaKind::Committee, CcaType::TypeA),
        cca(4, CcaKind::Committee, CcaType::TypeA),
        cca(5, CcaKind::Committee, CcaType::TypeB),
        cca(6, CcaKind::Committee, CcaType::TypeB),
        cca(7, CcaKind::Sports, CcaType::None),
        cca(8, CcaKind::Committee, CcaType::None),
        cca(9, CcaKind::Sports, CcaType::None),
        cca(10, CcaKind::Sports, CcaType::None),
        cca(11, CcaKind::Sports, CcaType::None),
        cca(12, CcaKind::Sports, CcaType::None),
        cca(13, CcaKind::Sports, CcaType::None),
        cca(14, CcaKind::Culture, CcaType::None),
        cca(15, CcaKind::Culture, CcaType::None),
    ];
    let kinds = [
        (LEAD, PositionKind::Lead),
        (MAIN, PositionKind::MainComm),
        (SUB, PositionKind::SubComm),
        (TM, PositionKind::TeamManager),
        (MEMBER, PositionKind::Member),
        (BLOCK, PositionKind::BlockComm),
    ];
    let positions = ccas
        .iter()
        .flat_map(|c| {
            kinds.iter().map(move |&(offset, kind)| CcaPosition {
                id: pos(c.id, offset),
                cca_id: c.id,
                reporting_position_id: None,
                position_type: kind,
                name: format!("P{}", pos(c.id, offset)),
                description: None,
                capacity: None,
            })
        })
        .collect();
    let appointments = holdings
        .iter()
        .map(|&(user_id, position_id, commitment_period)| CcaAppointment {
            user_id,
            position_id,
            commitment_period,
            points: 0,
            team_status: TeamStatus::None,
            created_at: None,
        })
        .collect();
    Directory::new(
        (1..=3)
            .map(|id| User {
                id,
                name: format!("U{id}"),
                email: format!("u{id}@x"),
            })
            .collect(),
        ccas,
        positions,
        appointments,
    )
    .unwrap()
}

fn year() -> CapSettings {
    CapSettings::default()
}

fn appealed(users: &[i32]) -> CapSettings {
    CapSettings {
        window: CapWindow::AcademicYear,
        committee_appeals: users.iter().copied().collect(),
        ..Default::default()
    }
}

/// (main, sub) for user 1 in the academic-year window.
fn counts(directory: &Directory) -> (usize, usize) {
    let entry = directory.committee_cap_for(1, &year()).unwrap();
    (entry.counts[0].main, entry.counts[0].sub)
}

#[test]
fn type_a_sub_is_sub_and_every_other_committee_seat_is_main() {
    let d = fixture(&[(1, pos(1, SUB), FullYear), (1, pos(2, MAIN), FullYear)]);
    assert_eq!(counts(&d), (1, 1));
    // Chair, vice and member seats count as Main.
    let d = fixture(&[(1, pos(1, LEAD), FullYear), (1, pos(2, MEMBER), FullYear)]);
    assert_eq!(counts(&d), (2, 0));
    // Every Type B seat is Main, even one modelled as a sub comm.
    let d = fixture(&[(1, pos(5, BLOCK), FullYear), (1, pos(6, SUB), FullYear)]);
    assert_eq!(counts(&d), (2, 0));
}

#[test]
fn team_managers_and_non_committee_ccas_do_not_count_as_committee_seats() {
    let d = fixture(&[(1, pos(1, TM), FullYear), (1, pos(7, MAIN), FullYear)]);
    let entry = d.committee_cap_for(1, &year()).unwrap();
    assert_eq!((entry.counts[0].main, entry.counts[0].sub), (0, 0));
    assert_eq!(entry.counts[0].sports_culture, 1, "the sports seat still counts");
    assert!(d.committee_cap_for(2, &year()).is_none(), "no counted seat at all");
}

#[test]
fn only_residents_over_the_cap_are_flagged() {
    let d = fixture(&[
        // User 1: 1M1S, allowed.
        (1, pos(1, MAIN), FullYear),
        (1, pos(2, SUB), FullYear),
        // User 2: 1M2S, over the standard cap.
        (2, pos(1, MAIN), FullYear),
        (2, pos(2, SUB), FullYear),
        (2, pos(3, SUB), FullYear),
    ]);
    let report = d.committee_caps(&year());
    assert_eq!(report.entries.len(), 1);
    let entry = &report.entries[0];
    assert_eq!(entry.user_id, 2);
    assert!(entry.over_cap);
    assert!(entry.counts[0].exceeds_standard);
    assert!(!entry.counts[0].exceeds_appeal);
    assert_eq!(entry.seats.len(), 3);
}

#[test]
fn appeal_lifts_to_the_appeal_cap_but_no_further() {
    let one_main_two_sub = [
        (2, pos(1, MAIN), FullYear),
        (2, pos(2, SUB), FullYear),
        (2, pos(3, SUB), FullYear),
    ];
    let d = fixture(&one_main_two_sub);
    let entry = &d.committee_caps(&appealed(&[2])).entries[0];
    assert!(entry.appealed);
    assert!(!entry.over_cap, "1M2S is within the appeal cap");

    let mut beyond = one_main_two_sub.to_vec();
    beyond.push((2, pos(4, SUB), FullYear));
    let d = fixture(&beyond);
    let entry = &d.committee_caps(&appealed(&[2])).entries[0];
    assert!(entry.over_cap, "1M3S breaks even the appeal cap");
}

#[test]
fn appealed_residents_stay_listed_within_the_cap() {
    let d = fixture(&[(1, pos(1, SUB), FullYear)]);
    let report = d.committee_caps(&appealed(&[1, 3, 99]));
    let ids: Vec<_> = report.entries.iter().map(|e| e.user_id).collect();
    // 3 holds nothing but appealed; 99 is not a known user.
    assert_eq!(ids, [1, 3]);
    assert!(report.entries.iter().all(|e| e.appealed && !e.over_cap));
}

#[test]
fn semester_window_counts_each_semester_separately() {
    // Two Main seats in Sem 1 and two different Main seats in Sem 2.
    let d = fixture(&[
        (1, pos(1, MAIN), Semester1),
        (1, pos(2, MAIN), Semester1),
        (1, pos(3, MAIN), Semester2),
        (1, pos(4, MAIN), Semester2),
    ]);
    assert!(d.committee_caps(&year()).entries[0].over_cap, "4M across the year");
    let per_semester = CapSettings {
        window: CapWindow::Semester,
        ..Default::default()
    };
    assert!(d.committee_caps(&per_semester).entries.is_empty(), "2M in each semester");

    let d = fixture(&[
        (1, pos(1, MAIN), FullYear),
        (1, pos(2, MAIN), Semester2),
        (1, pos(3, SUB), Semester2),
    ]);
    let entry = &d.committee_caps(&per_semester).entries[0];
    assert_eq!(entry.counts.len(), 2);
    assert!(!entry.counts[0].exceeds_standard, "Sem 1: 1M");
    assert!(entry.counts[1].exceeds_standard, "Sem 2: 2M1S");
}

#[test]
fn two_seats_in_one_committee_are_flagged_only_when_periods_overlap() {
    let d = fixture(&[(1, pos(1, MAIN), FullYear), (1, pos(1, SUB), Semester2)]);
    let entry = &d.committee_caps(&year()).entries[0];
    assert_eq!(entry.duplicate_ccas, [1]);
    assert!(!entry.over_cap);

    // A mid-year role change is not a duplicate.
    let d = fixture(&[(1, pos(1, MAIN), Semester1), (1, pos(1, SUB), Semester2)]);
    assert!(d.committee_caps(&year()).entries.is_empty());
}

#[test]
fn untyped_committees_are_reported() {
    let d = fixture(&[]);
    assert_eq!(d.committee_caps(&year()).unclassified_ccas, [8]);
}

#[test]
fn report_serializes_for_the_ui() {
    let d = fixture(&[
        (2, pos(1, MAIN), FullYear),
        (2, pos(2, MAIN), FullYear),
        (2, pos(3, SUB), FullYear),
    ]);
    let json = serde_json::to_value(d.snapshot()).unwrap();
    let caps = &json["committeeCaps"];
    assert_eq!(caps["window"], "academic-year");
    assert_eq!(caps["entries"][0]["userId"], 2);
    assert_eq!(caps["entries"][0]["overCap"], true);
    assert_eq!(caps["entries"][0]["counts"][0]["exceedsStandard"], true);
    assert_eq!(caps["entries"][0]["seats"][2]["level"], "sub");
    assert_eq!(caps["unclassifiedCcas"][0], 8);
}

fn combined_appealed(users: &[i32]) -> CapSettings {
    CapSettings {
        combined_appeals: users.iter().copied().collect(),
        ..Default::default()
    }
}

/// `n` distinct Sports/Cultural CCAs for the user, full year.
fn sports_culture(user_id: i32, n: usize) -> Vec<(i32, i32, CommitmentPeriod)> {
    [7, 9, 10, 11, 12, 13, 14, 15][..n]
        .iter()
        .map(|&cca| (user_id, pos(cca, MEMBER), FullYear))
        .collect()
}

#[test]
fn combined_cap_allows_one_plus_five_but_not_one_plus_six() {
    let mut holdings = sports_culture(1, 5);
    holdings.push((1, pos(1, MAIN), FullYear));
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries.is_empty(), "1 + 5");

    let mut holdings = sports_culture(1, 6);
    holdings.push((1, pos(1, MAIN), FullYear));
    let d = fixture(&holdings);
    let entry = &d.committee_caps(&year()).entries[0];
    assert!(entry.over_combined_cap, "1 + 6");
    assert!(!entry.over_cap, "the committee cap itself is fine");
    assert_eq!(entry.counts[0].sports_culture, 6);
    assert_eq!(entry.sports_culture_ccas, [7, 9, 10, 11, 12, 13]);
}

#[test]
fn combined_cap_counts_every_committee_seat_and_mixes_sports_with_culture() {
    // 3 Sub + 2 sports + 2 culture = 7.
    let mut holdings = vec![
        (1, pos(1, SUB), FullYear),
        (1, pos(2, SUB), FullYear),
        (1, pos(3, SUB), FullYear),
        (1, pos(7, MEMBER), FullYear),
        (1, pos(9, MEMBER), FullYear),
        (1, pos(14, MEMBER), FullYear),
    ];
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries.is_empty(), "3 + 3");
    holdings.push((1, pos(15, MEMBER), FullYear));
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries[0].over_combined_cap, "3 + 4");
}

#[test]
fn sports_culture_cap_applies_without_a_committee_seat() {
    let d = fixture(&sports_culture(1, 5));
    assert!(d.committee_caps(&year()).entries.is_empty(), "5 is the Sports/Culture cap");
    let d = fixture(&sports_culture(1, 6));
    let entry = &d.committee_caps(&year()).entries[0];
    assert!(entry.over_combined_cap);
    assert!(entry.counts[0].exceeds_sports_culture);
    assert!(!entry.counts[0].exceeds_total, "6 is still within the total cap");
    let entry = &d.committee_caps(&combined_appealed(&[1])).entries[0];
    assert!(!entry.over_combined_cap, "6 with a CCA cap appeal");
}

#[test]
fn edited_rules_and_committee_types_change_the_report() {
    // 2 Main: within this year's table.
    let holdings = [(1, pos(5, BLOCK), FullYear), (1, pos(1, MAIN), FullYear)];
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries.is_empty());

    let stricter = CapSettings {
        rules: CapRules {
            committee_combos: vec![
                CommitteeCombo { main: 1, sub: 1 },
                CommitteeCombo { main: 0, sub: 3 },
            ],
            ..CapRules::default()
        },
        ..Default::default()
    };
    assert!(d.committee_caps(&stricter).entries[0].over_cap, "2M is no longer listed");

    // A Type A Sub-Comm seat counts as Main once the committee is set to Type B.
    let d = fixture(&[(1, pos(1, SUB), FullYear), (1, pos(2, SUB), FullYear)]);
    assert_eq!(d.committee_cap_for(1, &year()).unwrap().counts[0].main, 0);
    let retyped = CapSettings {
        rules: CapRules {
            cca_types: [(1, CcaType::TypeB)].into(),
            ..CapRules::default()
        },
        ..Default::default()
    };
    let count = &d.committee_cap_for(1, &retyped).unwrap().counts[0];
    assert_eq!((count.main, count.sub), (1, 1));

    // Setting a type clears the untyped-committee warning.
    let typed = CapSettings {
        rules: CapRules {
            cca_types: [(8, CcaType::TypeA)].into(),
            ..CapRules::default()
        },
        ..Default::default()
    };
    assert!(d.committee_caps(&typed).unclassified_ccas.is_empty());
    assert_eq!(d.committee_caps(&typed).rules.cca_types[&8], CcaType::TypeA);
}

#[test]
fn team_managers_count_towards_their_sports_cca_but_residents_do_not() {
    let mut holdings = sports_culture(1, 5);
    holdings.push((1, pos(1, MAIN), FullYear));
    holdings.push((1, pos(15, TM), FullYear));
    let d = fixture(&holdings);
    assert_eq!(d.committee_caps(&year()).entries[0].counts[0].sports_culture, 6);

    // Two seats in one sports CCA are still one CCA.
    let mut holdings = sports_culture(1, 5);
    holdings.push((1, pos(1, MAIN), FullYear));
    holdings.push((1, pos(7, TM), FullYear));
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries.is_empty());
}

#[test]
fn combined_appeal_raises_the_combined_cap_to_seven_only() {
    // 1M2S + 4 Sports/Culture = 7.
    let mut holdings = sports_culture(2, 4);
    holdings.extend([
        (2, pos(1, MAIN), FullYear),
        (2, pos(2, SUB), FullYear),
        (2, pos(3, SUB), FullYear),
    ]);
    let d = fixture(&holdings);
    let entry = &d.committee_caps(&combined_appealed(&[2])).entries[0];
    assert!(entry.combined_appealed);
    assert!(!entry.over_combined_cap, "7 with a combined appeal");
    assert_eq!(entry.counts[0].total_limit, 7);
    assert!(entry.over_cap, "1M2S still breaks the committee cap");

    holdings.push((2, pos(15, MEMBER), FullYear));
    let d = fixture(&holdings);
    let entry = &d.committee_caps(&combined_appealed(&[2])).entries[0];
    assert!(entry.over_combined_cap, "8 is over even with a combined appeal");
}

#[test]
fn four_sub_and_four_sports_culture_needs_both_appeals() {
    let mut holdings = sports_culture(3, 4);
    holdings.extend((1..=4).map(|cca| (3, pos(cca, SUB), FullYear)));
    let d = fixture(&holdings);

    let only_combined = d.committee_caps(&combined_appealed(&[3]));
    assert!(only_combined.entries[0].over_combined_cap);
    assert!(only_combined.entries[0].over_cap, "4S without the committee appeal");

    let both = CapSettings {
        committee_appeals: [3].into(),
        combined_appeals: [3].into(),
        ..Default::default()
    };
    let entry = &d.committee_caps(&both).entries[0];
    assert!(!entry.over_cap);
    assert!(!entry.over_combined_cap, "4S + 4 = 8 with both appeals");
    assert_eq!(entry.counts[0].total_limit, 8);

    holdings.push((3, pos(15, MEMBER), FullYear));
    let d = fixture(&holdings);
    assert!(d.committee_caps(&both).entries[0].over_combined_cap, "9 is never allowed");
}

#[test]
fn combined_cap_follows_the_semester_window() {
    let mut holdings: Vec<_> = [7, 9, 10]
        .into_iter()
        .map(|cca| (1, pos(cca, MEMBER), Semester1))
        .chain([11, 12, 13].into_iter().map(|cca| (1, pos(cca, MEMBER), Semester2)))
        .collect();
    holdings.push((1, pos(1, MAIN), FullYear));
    let d = fixture(&holdings);
    assert!(d.committee_caps(&year()).entries[0].over_combined_cap, "1 + 6 across the year");
    let per_semester = CapSettings {
        window: CapWindow::Semester,
        ..Default::default()
    };
    assert!(d.committee_caps(&per_semester).entries.is_empty(), "1 + 3 in each semester");
}
