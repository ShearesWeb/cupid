use std::collections::HashMap;

use crate::models::{Applicant, CCAIdx, Position};

mod assemble;
mod chair_pref;
mod user_pref;

pub mod appeals;
pub mod db;

/// Owned corpus loaded from the database.
pub struct DataSourcePool {
    applicants: Vec<Applicant>,
    positions: Vec<Position>,
    ccas: HashMap<CCAIdx, String>,
}

impl DataSourcePool {
    pub fn new(
        applicants: Vec<Applicant>,
        positions: Vec<Position>,
        ccas: HashMap<CCAIdx, String>,
    ) -> Self {
        DataSourcePool {
            applicants,
            positions,
            ccas,
        }
    }

    pub fn applicants(&self) -> &[Applicant] {
        &self.applicants
    }

    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    /// CCA display name for `id`, if known.
    pub fn cca_name(&self, id: CCAIdx) -> Option<&str> {
        self.ccas.get(&id).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::models::{Applicant, CCAIdx, Position, PositionType};

    #[test]
    fn accessors_expose_corpus() {
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let positions = vec![Position::new(
            10,
            5,
            "Head".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )];
        let mut ccas = HashMap::new();
        ccas.insert(CCAIdx(5), "Chess".to_string());

        let pool = DataSourcePool::new(applicants, positions, ccas);

        assert_eq!(pool.applicants().len(), 1);
        assert_eq!(pool.positions().len(), 1);
        assert_eq!(pool.cca_name(CCAIdx(5)), Some("Chess"));
        assert_eq!(pool.cca_name(CCAIdx(99)), None);
    }
}
