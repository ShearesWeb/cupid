//! End-to-end allocation over a realistic Sheares Hall corpus, exercised through
//! the public crate API. Names come from `intranet/data`; capacities are reduced
//! from the production seed (e.g. Block Committee is cap 8 live) so a compact
//! 8-applicant corpus is oversubscribed and the two-pass behaviour is visible.

use std::collections::HashMap;

use cupid::algorithm::run;
use cupid::models::{
    Appeals, Applicant, ApplicantIdx, Appointment, Appointments, Cca, Pool, Position, PositionIdx,
    PositionType,
};

fn corpus() -> Pool {
    use PositionType::{BlockComm, MainComm, SubComm};

    let applicants = vec![
        Applicant::new(
            1,
            "Aria Tan".into(),
            "aria@x".into(),
            vec![PositionIdx(102), PositionIdx(101), PositionIdx(201)],
        ),
        Applicant::new(
            2,
            "Mason Lim".into(),
            "mason@x".into(),
            vec![PositionIdx(102)],
        ),
        Applicant::new(
            3,
            "Chloe Ng".into(),
            "chloe@x".into(),
            vec![PositionIdx(102), PositionIdx(202)],
        ),
        Applicant::new(
            4,
            "Declan Ho".into(),
            "declan@x".into(),
            vec![PositionIdx(101), PositionIdx(202)],
        ),
        Applicant::new(
            5,
            "Brielle Koh".into(),
            "brielle@x".into(),
            vec![PositionIdx(201)],
        ),
        Applicant::new(
            6,
            "Jasper Chua".into(),
            "jasper@x".into(),
            vec![PositionIdx(301)],
        ),
        Applicant::new(
            7,
            "Harper Tan".into(),
            "harper@x".into(),
            vec![PositionIdx(301)],
        ),
        Applicant::new(
            8,
            "Ezra Sim".into(),
            "ezra@x".into(),
            vec![PositionIdx(201)],
        ),
    ];

    // (id, cca_id, name, capacity, type, chair ranking best-first)
    let positions = vec![
        Position::new(
            101,
            Cca::new(0, "C"),
            "Product Manager".into(),
            None,
            1,
            MainComm,
            vec![ApplicantIdx(1), ApplicantIdx(4)],
        ),
        Position::new(
            102,
            Cca::new(0, "C"),
            "Block Committee".into(),
            None,
            2,
            BlockComm,
            vec![ApplicantIdx(1), ApplicantIdx(2), ApplicantIdx(3)],
        ),
        Position::new(
            201,
            Cca::new(0, "C"),
            "Developer".into(),
            None,
            2,
            MainComm,
            vec![ApplicantIdx(1), ApplicantIdx(8), ApplicantIdx(5)],
        ),
        Position::new(
            202,
            Cca::new(0, "C"),
            "Design Subcomm".into(),
            None,
            3,
            SubComm,
            vec![ApplicantIdx(4), ApplicantIdx(3)],
        ),
        Position::new(
            301,
            Cca::new(0, "C"),
            "Programmes IC".into(),
            None,
            1,
            MainComm,
            vec![ApplicantIdx(7)],
        ),
    ];

    Pool::new(applicants, positions)
}

fn sorted(mut v: Vec<i32>) -> Vec<i32> {
    v.sort();
    v
}

#[test]
fn allocates_real_committee_corpus() {
    let pool = corpus();
    let result = run(&pool, &Appeals::new());

    // --- exact holdings per applicant (sorted position-id sets) ---
    let held = |id: i32| {
        sorted(
            result
                .positions_of(ApplicantIdx(id))
                .iter()
                .map(|p| p.0)
                .collect(),
        )
    };
    assert_eq!(
        held(1),
        vec![101, 102],
        "Aria: Product Manager + Block Committee"
    );
    assert_eq!(held(2), vec![102], "Mason: Block Committee");
    assert_eq!(
        held(3),
        vec![202],
        "Chloe: Design Subcomm (lost block seat, fell to sub)"
    );
    assert_eq!(
        held(4),
        vec![202],
        "Declan: Design Subcomm (bumped off Product Manager)"
    );
    assert_eq!(held(5), vec![201], "Brielle: Developer");
    assert_eq!(
        held(6),
        Vec::<i32>::new(),
        "Jasper: unmatched (chair never ranked him)"
    );
    assert_eq!(held(7), vec![301], "Harper: Programmes IC");
    assert_eq!(held(8), vec![201], "Ezra: Developer");

    // --- exact occupants per position (sorted applicant-id sets) ---
    let seated = |pid: i32| {
        sorted(
            result
                .for_position(PositionIdx(pid))
                .iter()
                .map(|a| a.applicant_id.0)
                .collect(),
        )
    };
    assert_eq!(seated(101), vec![1]);
    assert_eq!(seated(102), vec![1, 2]);
    assert_eq!(seated(201), vec![5, 8]);
    assert_eq!(seated(202), vec![3, 4]);
    assert_eq!(seated(301), vec![7]);

    // --- only Jasper is unmatched ---
    assert_eq!(result.unmatched(pool.applicants()), vec![ApplicantIdx(6)]);

    // --- invariants over the whole corpus ---
    let type_of: HashMap<PositionIdx, PositionType> =
        pool.positions().map(|p| (p.id, p.position_type)).collect();
    for p in pool.positions() {
        assert!(
            result.for_position(p.id).len() <= p.capacity,
            "position {} over capacity",
            p.id.0
        );
    }
    for a in pool.applicants() {
        let (mut block, mut main, mut sub) = (0, 0, 0);
        for pid in result.positions_of(a.id) {
            match type_of[pid] {
                PositionType::BlockComm => block += 1,
                PositionType::MainComm => main += 1,
                PositionType::SubComm => sub += 1,
            }
        }
        assert!(main + block <= 2, "{} over main+block quota", a.name);
        assert!(sub <= 3, "{} over subcomm quota", a.name);
        assert!(
            !(main >= 1 && sub >= 2),
            "{} violates main/sub rule",
            a.name
        );
    }
}

#[test]
fn appointments_reduce_seats_and_quota() {
    use PositionType::MainComm;

    // P100 cap 1, already held by appointee #9 (not an applicant).
    // Applicant 1 ranks P100 but cannot get it (no vacancy).
    let applicants = vec![
        Applicant::new(1, "Aria".into(), "aria@x".into(), vec![PositionIdx(100)]),
        // Applicant 2 already holds a maincomm via appointment, then ranks another.
        Applicant::new(2, "Mason".into(), "mason@x".into(), vec![PositionIdx(101)]),
    ];
    let positions = vec![
        Position::new(
            100,
            Cca::new(0, "C"),
            "Solo".into(),
            None,
            1,
            MainComm,
            vec![ApplicantIdx(1)],
        )
        .with_appointed(1),
        Position::new(
            101,
            Cca::new(0, "C"),
            "Open".into(),
            None,
            1,
            MainComm,
            vec![ApplicantIdx(2)],
        ),
        Position::new(
            102,
            Cca::new(0, "C"),
            "Held".into(),
            None,
            1,
            MainComm,
            vec![],
        )
        .with_appointed(1),
    ];
    let pool = Pool::new(applicants, positions).with_appointments(Appointments::from_iter([
        Appointment {
            applicant: ApplicantIdx(9),
            position: PositionIdx(100),
        },
        Appointment {
            applicant: ApplicantIdx(2),
            position: PositionIdx(102),
        },
    ]));

    let result = run(&pool, &Appeals::new());

    // P100's only seat is taken by the appointee -> applicant 1 gets nothing there.
    assert!(
        result.positions_of(ApplicantIdx(1)).is_empty(),
        "no vacancy in P100"
    );
    // Applicant 2 holds P102 (appointment) + wins P101 = 2 main, still within quota.
    assert!(
        result
            .positions_of(ApplicantIdx(2))
            .contains(&PositionIdx(101))
    );
}
