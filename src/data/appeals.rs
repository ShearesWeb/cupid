use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;

use super::DataSourcePool;
use crate::models::{Appeals, ApplicantIdx, PositionIdx};

/// One appeal row: every column matched by name, never by database ID.
#[derive(Debug, Deserialize)]
pub struct AppealRecord {
    pub applicant_name: String,
    pub cca_name: String,
    pub position_name: String,
}

/// Read + parse the appeals CSV file. Thin I/O.
fn load(path: &Path) -> Result<Vec<AppealRecord>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    parse(file)
}

/// Parse appeal records from any reader. Split out so it is testable without a file.
fn parse<R: Read>(reader: R) -> Result<Vec<AppealRecord>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut records: Vec<AppealRecord> = Vec::new();
    for result in rdr.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

/// Resolve appeal rows against the loaded corpus by name. Pure.
///
/// Applicant names are not unique in `users`, so 0 or >1 matches is a problem.
/// Positions are unique per `(cca_name, position_name)`. All problems are
/// collected (with 1-based line numbers) and returned as one error.
fn resolve(records: &[AppealRecord], pool: &DataSourcePool) -> Result<Appeals, Box<dyn Error>> {
    let mut by_name: HashMap<String, Vec<ApplicantIdx>> = HashMap::new();
    for a in pool.applicants() {
        by_name
            .entry(a.name.trim().to_string())
            .or_default()
            .push(a.id);
    }

    let mut by_cca_pos: HashMap<(String, String), PositionIdx> = HashMap::new();
    for p in pool.positions() {
        if let Some(cca) = pool.cca_name(p.cca_id) {
            by_cca_pos.insert((cca.trim().to_string(), p.name.trim().to_string()), p.id);
        }
    }

    let mut appeals = Appeals::new();
    let mut problems: Vec<String> = Vec::new();

    for (i, rec) in records.iter().enumerate() {
        let line = i + 2; // header is line 1; first record is line 2
        let applicant_name = rec.applicant_name.trim();
        let cca_name = rec.cca_name.trim();
        let position_name = rec.position_name.trim();

        let applicant_id = match by_name.get(applicant_name) {
            None => {
                problems.push(format!("line {line}: unknown applicant '{applicant_name}'"));
                continue;
            }
            Some(ids) if ids.len() > 1 => {
                problems.push(format!(
                    "line {line}: ambiguous applicant '{applicant_name}' ({} matches)",
                    ids.len()
                ));
                continue;
            }
            Some(ids) => ids[0],
        };

        let key = (cca_name.to_string(), position_name.to_string());
        let position_id = match by_cca_pos.get(&key) {
            None => {
                problems.push(format!(
                    "line {line}: unknown position '{position_name}' in CCA '{cca_name}'"
                ));
                continue;
            }
            Some(id) => *id,
        };

        appeals.grant(applicant_id, position_id);
    }

    if problems.is_empty() {
        Ok(appeals)
    } else {
        Err(format!(
            "appeals CSV had {} problem(s):\n{}",
            problems.len(),
            problems.join("\n")
        )
        .into())
    }
}

/// Load the CSV at `path` and resolve it against `pool`.
pub fn load_and_resolve(path: &Path, pool: &DataSourcePool) -> Result<Appeals, Box<dyn Error>> {
    let records = load(path)?;
    resolve(&records, pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Cursor;

    use crate::models::{Applicant, CCAIdx, Position, PositionType};

    // Corpus: Ann(1), Ben(2), Ann(3) [duplicate name], Cara(4);
    // one position "Head" (id 10) under CCA "Chess" (id 1).
    fn pool() -> DataSourcePool {
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![]),
            Applicant::new(3, "Ann".into(), "ann2@x".into(), vec![]),
            Applicant::new(4, "Cara".into(), "cara@x".into(), vec![]),
        ];
        let positions = vec![Position::new(
            10,
            1,
            "Head".into(),
            None,
            1,
            PositionType::MainComm,
            vec![ApplicantIdx(4)],
        )];
        let mut ccas = HashMap::new();
        ccas.insert(CCAIdx(1), "Chess".to_string());
        DataSourcePool::new(applicants, positions, ccas)
    }

    #[test]
    fn parse_reads_records() {
        let csv = "applicant_name,cca_name,position_name\nCara,Chess,Head\n";
        let records = parse(Cursor::new(csv)).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].applicant_name, "Cara");
    }

    #[test]
    fn resolves_valid_row() {
        let records = parse(Cursor::new(
            "applicant_name,cca_name,position_name\nCara,Chess,Head\n",
        ))
        .unwrap();
        let appeals = resolve(&records, &pool()).unwrap();
        assert!(appeals.contains(ApplicantIdx(4), PositionIdx(10)));
    }

    #[test]
    fn unknown_applicant_is_a_problem() {
        let records = parse(Cursor::new(
            "applicant_name,cca_name,position_name\nZed,Chess,Head\n",
        ))
        .unwrap();
        let err = resolve(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("unknown applicant 'Zed'"), "got: {err}");
    }

    #[test]
    fn ambiguous_applicant_is_a_problem() {
        let records = parse(Cursor::new(
            "applicant_name,cca_name,position_name\nAnn,Chess,Head\n",
        ))
        .unwrap();
        let err = resolve(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("ambiguous applicant 'Ann'"), "got: {err}");
    }

    #[test]
    fn unknown_position_is_a_problem() {
        let records = parse(Cursor::new(
            "applicant_name,cca_name,position_name\nCara,Chess,Ghost\n",
        ))
        .unwrap();
        let err = resolve(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("unknown position 'Ghost'"), "got: {err}");
    }

    #[test]
    fn problems_aggregate() {
        let csv = "applicant_name,cca_name,position_name\nZed,Chess,Head\nCara,Chess,Ghost\n";
        let records = parse(Cursor::new(csv)).unwrap();
        let err = resolve(&records, &pool()).unwrap_err().to_string();
        assert!(err.contains("2 problem(s)"), "got: {err}");
    }
}
