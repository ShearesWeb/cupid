//! Commit-as-CSV export: project a run's new allocations into the intranet
//! repo's `data/cca-appointment` CSV convention. Pure logic only — the git
//! plumbing that publishes these files lives in the desktop shell.

use std::collections::{BTreeMap, HashSet};

use crate::models::{MatchResult, Pool, PositionIdx};

/// Header shared by every intranet `cca-appointment` CSV.
pub const HEADER: &str = "user_email,cca_name,position_name,commitment_period";

/// Cupid ignores commitment periods and appoints for the full year.
pub const COMMITMENT_PERIOD: &str = "full-year";

/// One CSV data row: an appointment keyed the way intranet reconciles them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppointmentRow {
    pub cca_name: String,
    pub position_name: String,
    pub user_email: String,
}

/// The run's new appointments as CSV rows: every settled allocation —
/// preallocated seats included — that is not already an existing appointment
/// and whose position the operator did not hold back in `excluded`.
/// Sorted by (cca, position, email); adds-only by construction.
pub fn rows_from(
    result: &MatchResult,
    pool: &Pool,
    excluded: &HashSet<PositionIdx>,
) -> Vec<AppointmentRow> {
    let mut rows: Vec<AppointmentRow> = result
        .all()
        .filter(|a| !excluded.contains(&a.position_id))
        .filter(|a| !pool.appointments().held_by(a.applicant_id).contains(&a.position_id))
        .filter_map(|a| {
            let applicant = pool.applicant(a.applicant_id)?;
            let position = pool.position(a.position_id)?;
            Some(AppointmentRow {
                cca_name: position.cca.name.clone(),
                position_name: position.name.clone(),
                user_email: applicant.email.clone(),
            })
        })
        .collect();
    rows.sort();
    rows.dedup();
    rows
}

/// File-name slug for a CCA: lowercase, non-alphanumeric runs collapse to a
/// single underscore, no leading/trailing underscores.
pub fn slug(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    out.trim_end_matches('_').to_string()
}

/// Group rows into per-CCA files keyed by `<slug>.csv`.
pub fn by_file(rows: Vec<AppointmentRow>) -> BTreeMap<String, Vec<AppointmentRow>> {
    let mut files: BTreeMap<String, Vec<AppointmentRow>> = BTreeMap::new();
    for row in rows {
        files.entry(format!("{}.csv", slug(&row.cca_name))).or_default().push(row);
    }
    files
}

/// A row rendered as one CSV line with minimal quoting.
pub fn csv_line(row: &AppointmentRow) -> String {
    [
        row.user_email.as_str(),
        row.cca_name.as_str(),
        row.position_name.as_str(),
        COMMITMENT_PERIOD,
    ]
    .map(csv_field)
    .join(",")
}

/// Quote a field only when CSV requires it (comma, quote, or newline inside).
fn csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Merge rows into an existing file body (or `None` for a new file): header
/// first, then the union of existing data lines and the new rows, sorted and
/// deduplicated. Existing lines are never dropped, so the export stays
/// adds-only under intranet's declarative reconciliation.
pub fn merge(existing: Option<&str>, rows: &[AppointmentRow]) -> String {
    let mut lines: Vec<String> = existing
        .unwrap_or("")
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty() && *l != HEADER)
        .map(String::from)
        .collect();
    lines.extend(rows.iter().map(csv_line));
    lines.sort();
    lines.dedup();
    let mut body = String::from(HEADER);
    for line in lines {
        body.push('\n');
        body.push_str(&line);
    }
    body.push('\n');
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Algorithm, Applicant, ApplicantIdx, Appointment, Appointments, Cca, Ledger, Pool,
        Position, PositionIdx, PositionType,
    };

    fn row(cca: &str, position: &str, email: &str) -> AppointmentRow {
        AppointmentRow {
            cca_name: cca.into(),
            position_name: position.into(),
            user_email: email.into(),
        }
    }

    #[test]
    fn rows_map_allocations_to_names_and_exclude_existing_appointments() {
        let positions = vec![Position::new(10, Cca::new(1, "Sheares Media"), "Chair".into(),
            None, 2, PositionType::MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)])];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(10)]),
        ];
        // Ben already holds position 10: his pair must not re-export.
        let pool = Pool::new(applicants.clone(), positions.clone())
            .with_appointments(Appointments::from_iter([
                Appointment { applicant: ApplicantIdx(2), position: PositionIdx(10) },
            ]));
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[1], &positions[0]);

        let rows = rows_from(&ledger.finish(), &pool, &HashSet::new());
        assert_eq!(rows, vec![row("Sheares Media", "Chair", "ann@x")]);
    }

    #[test]
    fn rows_are_sorted_by_cca_then_position_then_email() {
        let positions = vec![
            Position::new(20, Cca::new(2, "Zeta"), "Chair".into(), None, 1,
                PositionType::MainComm, vec![ApplicantIdx(1)]),
            Position::new(10, Cca::new(1, "Alpha"), "Chair".into(), None, 2,
                PositionType::MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)]),
        ];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "b@x".into(), vec![PositionIdx(20), PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "a@x".into(), vec![PositionIdx(10)]),
        ];
        let pool = Pool::new(applicants.clone(), positions.clone());
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[0], &positions[1]);
        ledger.accept(&applicants[1], &positions[1]);

        let rows = rows_from(&ledger.finish(), &pool, &HashSet::new());
        assert_eq!(rows, vec![
            row("Alpha", "Chair", "a@x"),
            row("Alpha", "Chair", "b@x"),
            row("Zeta", "Chair", "b@x"),
        ]);
    }

    #[test]
    fn rows_include_preallocated_seats() {
        // A preallocated pair lands in the result via the preallocation pass
        // and must be exported like any other allocation.
        let positions = vec![Position::new(10, Cca::new(1, "Club"), "Chair".into(), None, 1,
            PositionType::MainComm, vec![])];
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let pool = Pool::new(applicants, positions);
        let mut preallocations = crate::models::Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));

        let result = crate::algorithm::run(&pool, &preallocations);
        assert_eq!(rows_from(&result, &pool, &HashSet::new()), vec![row("Club", "Chair", "ann@x")]);
    }

    #[test]
    fn rows_omit_excluded_positions() {
        let positions = vec![
            Position::new(10, Cca::new(1, "Alpha"), "Chair".into(), None, 1,
                PositionType::MainComm, vec![ApplicantIdx(1)]),
            Position::new(20, Cca::new(1, "Alpha"), "Vice".into(), None, 1,
                PositionType::MainComm, vec![ApplicantIdx(2)]),
        ];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(20)]),
        ];
        let pool = Pool::new(applicants.clone(), positions.clone());
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[1], &positions[1]);
        let result = ledger.finish();

        let excluded = HashSet::from([PositionIdx(20)]);
        assert_eq!(
            rows_from(&result, &pool, &excluded),
            vec![row("Alpha", "Chair", "ann@x")],
            "the excluded position contributes no rows"
        );
    }

    #[test]
    fn excluding_a_position_also_holds_back_its_preallocated_seats() {
        let positions = vec![Position::new(10, Cca::new(1, "Club"), "Chair".into(), None, 1,
            PositionType::MainComm, vec![])];
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let pool = Pool::new(applicants, positions);
        let mut preallocations = crate::models::Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));

        let result = crate::algorithm::run(&pool, &preallocations);
        let excluded = HashSet::from([PositionIdx(10)]);
        assert_eq!(rows_from(&result, &pool, &excluded), vec![]);
    }

    #[test]
    fn slug_lowercases_and_collapses_non_alphanumeric_runs() {
        assert_eq!(slug("Sheares Media"), "sheares_media");
        assert_eq!(slug("Dance & Drama! Club"), "dance_drama_club");
        assert_eq!(slug("  Padded  "), "padded");
    }

    #[test]
    fn by_file_groups_rows_under_cca_slug_filenames() {
        let rows = vec![
            row("Alpha Beta", "Chair", "a@x"),
            row("Zeta", "Chair", "z@x"),
            row("Alpha Beta", "Member", "b@x"),
        ];
        let files = by_file(rows.clone());
        assert_eq!(
            files.keys().cloned().collect::<Vec<_>>(),
            vec!["alpha_beta.csv", "zeta.csv"]
        );
        assert_eq!(files["alpha_beta.csv"], vec![rows[0].clone(), rows[2].clone()]);
    }

    #[test]
    fn csv_line_orders_fields_and_appends_commitment_period() {
        assert_eq!(csv_line(&row("Alpha", "Chair", "a@x")), "a@x,Alpha,Chair,full-year");
    }

    #[test]
    fn csv_line_quotes_fields_containing_commas_or_quotes() {
        assert_eq!(
            csv_line(&row("Say, \"Hi\"", "Chair", "a@x")),
            "a@x,\"Say, \"\"Hi\"\"\",Chair,full-year"
        );
    }

    #[test]
    fn merge_creates_a_new_file_with_header() {
        let body = merge(None, &[row("Alpha", "Chair", "a@x")]);
        assert_eq!(body, "user_email,cca_name,position_name,commitment_period\na@x,Alpha,Chair,full-year\n");
    }

    #[test]
    fn merge_appends_missing_rows_keeps_existing_and_deduplicates() {
        let existing = "user_email,cca_name,position_name,commitment_period\n\
                        old@x,Alpha,Chair,full-year\n";
        let body = merge(
            Some(existing),
            &[row("Alpha", "Chair", "old@x"), row("Alpha", "Chair", "new@x")],
        );
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\n\
             new@x,Alpha,Chair,full-year\n\
             old@x,Alpha,Chair,full-year\n"
        );
    }

    #[test]
    fn merge_tolerates_missing_trailing_newline_and_blank_lines() {
        let existing = "user_email,cca_name,position_name,commitment_period\n\
                        \n\
                        old@x,Alpha,Chair,full-year";
        let body = merge(Some(existing), &[row("Alpha", "Chair", "new@x")]);
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\n\
             new@x,Alpha,Chair,full-year\n\
             old@x,Alpha,Chair,full-year\n"
        );
    }
}
