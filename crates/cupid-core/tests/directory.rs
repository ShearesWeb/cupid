use cupid::directory::{
    Cca, CcaAppointment, CcaKind, CcaPosition, CcaTier, CcaType, CommitmentPeriod, Directory,
    DirectoryError, PositionKind, TeamStatus, User,
};

fn position(id: i32, kind: PositionKind, capacity: Option<usize>) -> CcaPosition {
    CcaPosition {
        id,
        cca_id: 1,
        reporting_position_id: None,
        position_type: kind,
        name: format!("P{id}"),
        description: None,
        capacity,
    }
}

fn fixture(appointments: Vec<CcaAppointment>) -> Directory {
    Directory::new(
        (1..=3)
            .map(|id| User {
                id,
                name: format!("U{id}"),
                email: format!("u{id}@x"),
            })
            .collect(),
        (1..=2)
            .map(|id| Cca {
                id,
                name: format!("C{id}"),
                kind: CcaKind::Committee,
                tier: CcaTier::None,
                cca_type: CcaType::TypeA,
                description: Some("CCA description".into()),
                image_url: None,
            })
            .collect(),
        vec![
            position(10, PositionKind::Lead, Some(1)),
            position(11, PositionKind::Vice, None),
            position(12, PositionKind::BlockComm, Some(2)),
            position(13, PositionKind::MainComm, Some(2)),
            position(14, PositionKind::SubComm, Some(2)),
            position(15, PositionKind::TeamManager, Some(0)),
            position(16, PositionKind::Member, None),
            position(17, PositionKind::Resident, None),
        ],
        appointments,
    )
    .unwrap()
}

fn holding(user_id: i32, position_id: i32, period: CommitmentPeriod) -> CcaAppointment {
    CcaAppointment {
        user_id,
        position_id,
        commitment_period: period,
        points: 42,
        team_status: TeamStatus::Varsity,
        created_at: Some("2026-08-01T00:00:00Z".into()),
    }
}

#[test]
fn snapshot_preserves_all_roles_empty_ccas_and_appointment_metadata() {
    let appointment = holding(1, 16, CommitmentPeriod::Semester1);
    let directory = fixture(vec![appointment.clone()]);
    let snapshot = directory.snapshot();
    assert_eq!(
        snapshot.ccas.iter().map(|c| c.id).collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(snapshot.positions.len(), 8);
    assert_eq!(snapshot.appointments, [appointment]);
    let json = serde_json::to_value(snapshot).unwrap();
    assert_eq!(json["positions"][5]["positionType"], "team-manager");
    assert!(json["positions"][5].get("canManageAppointments").is_none());
    assert_eq!(json["positions"][1]["capacity"], serde_json::Value::Null);
    assert_eq!(json["ccas"][0]["type"], "type-a");
    assert_eq!(json["appointments"][0]["commitmentPeriod"], "semester-1");
    assert_eq!(json["appointments"][0]["teamStatus"], "varsity");
    assert!(serde_json::from_str::<PositionKind>("\"unrecognized\"").is_err());
}

#[test]
fn edits_protect_member_and_resident_even_when_removing_a_missing_pair() {
    let mut directory = fixture(vec![holding(1, 16, CommitmentPeriod::FullYear)]);
    for id in [16, 17] {
        assert!(matches!(
            directory.add_appointment(2, id, CommitmentPeriod::FullYear),
            Err(DirectoryError::ReadOnlyPosition(_))
        ));
        assert!(matches!(
            directory.remove_appointment(1, id),
            Err(DirectoryError::ReadOnlyPosition(_))
        ));
        assert!(matches!(
            directory.update_appointment_period(1, id, CommitmentPeriod::Semester2),
            Err(DirectoryError::ReadOnlyPosition(_))
        ));
    }
    assert_eq!(directory.appointments().count(), 1);
}

#[test]
fn edits_enforce_identity_and_duplicate_rules_without_changing_the_directory() {
    let mut directory = fixture(vec![]);
    assert!(matches!(
        directory.add_appointment(99, 10, CommitmentPeriod::FullYear),
        Err(DirectoryError::UnknownUser(99))
    ));
    assert!(matches!(
        directory.add_appointment(1, 99, CommitmentPeriod::FullYear),
        Err(DirectoryError::UnknownPosition(99))
    ));
    directory
        .add_appointment(1, 10, CommitmentPeriod::Semester1)
        .unwrap();
    assert!(matches!(
        directory.add_appointment(1, 10, CommitmentPeriod::Semester2),
        Err(DirectoryError::DuplicateAppointment { .. })
    ));
    assert_eq!(directory.appointments().count(), 1);
    assert_eq!(
        directory.appointment(1, 10).unwrap().commitment_period,
        CommitmentPeriod::Semester1
    );
    assert!(directory.remove_appointment(1, 10).unwrap());
    assert!(!directory.remove_appointment(1, 10).unwrap());
    assert_eq!(directory.appointments().count(), 0);
}

#[test]
fn capacity_is_per_semester_and_failed_updates_are_atomic() {
    let first = holding(1, 10, CommitmentPeriod::Semester1);
    let mut directory = fixture(vec![first.clone()]);
    assert!(matches!(
        directory.add_appointment(2, 10, CommitmentPeriod::Semester1),
        Err(DirectoryError::CapacityExceeded { .. })
    ));
    directory
        .add_appointment(2, 10, CommitmentPeriod::Semester2)
        .unwrap();
    assert!(matches!(
        directory.update_appointment_period(1, 10, CommitmentPeriod::FullYear),
        Err(DirectoryError::CapacityExceeded { .. })
    ));
    assert_eq!(directory.appointment(1, 10), Some(&first));
    directory.remove_appointment(2, 10).unwrap();
    directory
        .update_appointment_period(1, 10, CommitmentPeriod::FullYear)
        .unwrap();
    let updated = directory.appointment(1, 10).unwrap();
    assert_eq!(updated.points, 42);
    assert_eq!(updated.team_status, TeamStatus::Varsity);
    assert_eq!(updated.created_at, first.created_at);
    assert_eq!(updated.commitment_period, CommitmentPeriod::FullYear);
}

#[test]
fn cca_conflicts_include_members_but_exempt_residents_and_disjoint_semesters() {
    let mut directory = fixture(vec![
        holding(1, 16, CommitmentPeriod::Semester1),
        holding(2, 17, CommitmentPeriod::FullYear),
    ]);
    assert!(matches!(
        directory.add_appointment(1, 11, CommitmentPeriod::FullYear),
        Err(DirectoryError::CcaConflict { .. })
    ));
    directory
        .add_appointment(1, 11, CommitmentPeriod::Semester2)
        .unwrap();
    directory
        .add_appointment(2, 11, CommitmentPeriod::FullYear)
        .unwrap();
    assert!(matches!(
        directory.update_appointment_period(1, 11, CommitmentPeriod::FullYear),
        Err(DirectoryError::CcaConflict { .. })
    ));
    assert_eq!(
        directory.appointment(1, 11).unwrap().commitment_period,
        CommitmentPeriod::Semester2
    );
}

#[test]
fn null_capacity_is_unlimited_zero_capacity_is_closed_and_new_metadata_defaults() {
    let mut directory = fixture(vec![]);
    assert!(matches!(
        directory.add_appointment(1, 15, CommitmentPeriod::FullYear),
        Err(DirectoryError::CapacityExceeded { .. })
    ));
    for user in 1..=3 {
        directory
            .add_appointment(user, 11, CommitmentPeriod::FullYear)
            .unwrap();
    }
    let new = directory.appointment(1, 11).unwrap();
    assert_eq!(new.points, 0);
    assert_eq!(new.team_status, TeamStatus::None);
    assert_eq!(new.created_at, None);
}

#[test]
fn loaded_directory_rejects_duplicate_and_dangling_appointments() {
    let original = fixture(vec![]).snapshot();
    let build = |appointments| {
        Directory::new(
            original.users.clone(),
            original.ccas.clone(),
            original
                .positions
                .iter()
                .map(|v| v.position.clone())
                .collect(),
            appointments,
        )
    };
    let a = holding(1, 10, CommitmentPeriod::FullYear);
    assert!(matches!(
        build(vec![a.clone(), a]),
        Err(DirectoryError::DuplicateAppointment { .. })
    ));
    assert!(matches!(
        build(vec![holding(99, 10, CommitmentPeriod::FullYear)]),
        Err(DirectoryError::UnknownUser(99))
    ));
    assert!(matches!(
        build(vec![holding(1, 99, CommitmentPeriod::FullYear)]),
        Err(DirectoryError::UnknownPosition(99))
    ));
}

#[test]
fn ex_shearite_period_preserves_database_full_year_occupancy() {
    let period: CommitmentPeriod = serde_json::from_str("\"ex-shearite\"").unwrap();
    let mut directory = fixture(vec![holding(1, 11, period)]);
    assert!(matches!(
        directory.add_appointment(1, 10, CommitmentPeriod::Semester2),
        Err(DirectoryError::CcaConflict { .. })
    ));
}

#[test]
fn reporting_links_can_cross_ccas_as_per_the_database_schema() {
    let snapshot = fixture(vec![]).snapshot();
    let mut positions: Vec<_> = snapshot.positions.into_iter().map(|v| v.position).collect();
    positions[1].cca_id = 2;
    positions[1].reporting_position_id = Some(10);
    let directory = Directory::new(snapshot.users, snapshot.ccas, positions, vec![]).unwrap();
    assert_eq!(
        directory.position(11).unwrap().reporting_position_id,
        Some(10)
    );
}

#[test]
fn loaded_directory_rejects_duplicate_entities_and_broken_position_references() {
    let snapshot = fixture(vec![]).snapshot();
    let positions: Vec<_> = snapshot
        .positions
        .iter()
        .map(|v| v.position.clone())
        .collect();
    let mut duplicate_users = snapshot.users.clone();
    duplicate_users.push(duplicate_users[0].clone());
    assert!(matches!(
        Directory::new(
            duplicate_users,
            snapshot.ccas.clone(),
            positions.clone(),
            vec![]
        ),
        Err(DirectoryError::DuplicateEntity {
            entity: "user",
            id: 1
        })
    ));
    let mut duplicate_ccas = snapshot.ccas.clone();
    duplicate_ccas.push(duplicate_ccas[0].clone());
    assert!(matches!(
        Directory::new(
            snapshot.users.clone(),
            duplicate_ccas,
            positions.clone(),
            vec![]
        ),
        Err(DirectoryError::DuplicateEntity {
            entity: "CCA",
            id: 1
        })
    ));
    let mut duplicate_positions = positions.clone();
    duplicate_positions.push(duplicate_positions[0].clone());
    assert!(matches!(
        Directory::new(
            snapshot.users.clone(),
            snapshot.ccas.clone(),
            duplicate_positions,
            vec![]
        ),
        Err(DirectoryError::DuplicateEntity {
            entity: "position",
            id: 10
        })
    ));
    let mut dangling_cca = positions.clone();
    dangling_cca[0].cca_id = 99;
    assert!(matches!(
        Directory::new(
            snapshot.users.clone(),
            snapshot.ccas.clone(),
            dangling_cca,
            vec![]
        ),
        Err(DirectoryError::InvalidReference(_))
    ));
    let mut dangling_parent = positions.clone();
    dangling_parent[0].reporting_position_id = Some(99);
    assert!(matches!(
        Directory::new(
            snapshot.users.clone(),
            snapshot.ccas.clone(),
            dangling_parent,
            vec![]
        ),
        Err(DirectoryError::UnknownPosition(99))
    ));
    let mut self_parent = positions;
    self_parent[0].reporting_position_id = Some(10);
    assert!(matches!(
        Directory::new(snapshot.users, snapshot.ccas, self_parent, vec![]),
        Err(DirectoryError::InvalidReference(_))
    ));
}
