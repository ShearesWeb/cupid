use std::collections::HashMap;

use crate::models::{Applicant, ApplicantIdx, CCAIdx, Position, PositionIdx, PositionType};
use super::DataSourcePool;

/// A small fixture corpus spanning all three allocatable position types.
/// Used by `cargo run` (non-production) and reused in tests.
pub fn load() -> DataSourcePool {
    let applicants = vec![
        Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(100), PositionIdx(200), PositionIdx(300)]),
        Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(200), PositionIdx(100)]),
        Applicant::new(3, "Cara".into(), "cara@x".into(), vec![PositionIdx(300), PositionIdx(100)]),
    ];
    let positions = vec![
        Position::new(100, 1, "Welfare".into(), None, 2, PositionType::BlockComm, vec![ApplicantIdx(1), ApplicantIdx(2), ApplicantIdx(3)]),
        Position::new(200, 2, "President".into(), None, 1, PositionType::MainComm, vec![ApplicantIdx(2), ApplicantIdx(1)]),
        Position::new(300, 2, "Secretary".into(), None, 2, PositionType::SubComm, vec![ApplicantIdx(1), ApplicantIdx(3)]),
    ];
    let mut ccas = HashMap::new();
    ccas.insert(CCAIdx(1), "Block Committee (A)".to_string());
    ccas.insert(CCAIdx(2), "Chess Club".to_string());

    DataSourcePool::new(applicants, positions, ccas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::run;
    use crate::models::Appeals;

    #[test]
    fn mock_corpus_runs_within_capacity() {
        let pool = load();
        assert_eq!(pool.applicants().len(), 3);
        assert_eq!(pool.positions().len(), 3);

        let result = run(pool.applicants(), pool.positions(), &Appeals::new());

        for p in pool.positions() {
            assert!(
                result.for_position(p.id).len() <= p.capacity,
                "position {} over capacity",
                p.id.0
            );
        }
    }
}
