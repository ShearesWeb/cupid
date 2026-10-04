//! Commit-as-CSV export: project a run's new allocations and directory edits
//! into the intranet repo's `data/cca-appointment` CSV convention. Pure logic
//! only — the git plumbing that publishes these files lives in the desktop shell.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::directory::{CcaAppointmentChange, CommitmentPeriod, Directory};
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
        .filter(|a| {
            !pool
                .appointments()
                .held_by(a.applicant_id)
                .contains(&a.position_id)
        })
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

/// Desired state per edited row: listed with this period, or absent (`None`).
pub type DirectoryEdits = BTreeMap<AppointmentRow, Option<CommitmentPeriod>>;

/// Directory edits never delete users, positions or CCAs, so lookups cannot miss.
pub fn directory_edits(changes: &[CcaAppointmentChange], directory: &Directory) -> DirectoryEdits {
    changes
        .iter()
        .map(|change| {
            let (user_id, position_id, period) = match change {
                CcaAppointmentChange::Add { appointment } => (
                    appointment.user_id,
                    appointment.position_id,
                    Some(appointment.commitment_period),
                ),
                CcaAppointmentChange::Remove {
                    user_id,
                    position_id,
                } => (*user_id, *position_id, None),
                CcaAppointmentChange::ChangePeriod {
                    user_id,
                    position_id,
                    to,
                    ..
                } => (*user_id, *position_id, Some(*to)),
            };
            let position = directory
                .position(position_id)
                .expect("validated position reference");
            let row = AppointmentRow {
                cca_name: directory
                    .cca(position.cca_id)
                    .expect("validated CCA reference")
                    .name
                    .clone(),
                position_name: position.name.clone(),
                user_email: directory
                    .user(user_id)
                    .expect("validated user reference")
                    .email
                    .clone(),
            };
            (row, period)
        })
        .collect()
}

/// Edits matching rows in place, leaving hand-maintained lines untouched.
/// Returns the new body and the edited keys found.
pub fn rewrite(body: &str, edits: &DirectoryEdits) -> (String, BTreeSet<AppointmentRow>) {
    let mut found = BTreeSet::new();
    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        match row_of(line).and_then(|row| edits.get_key_value(&row)) {
            None => out.push_str(line),
            Some((row, period)) => {
                found.insert(row.clone());
                match period {
                    Some(period) => out.push_str(&line_with_period(row, period.as_str())),
                    None => continue,
                }
            }
        }
        out.push('\n');
    }
    (out, found)
}

/// Edits no file held, as new lines keyed by `<slug>.csv`. Unlisted removals
/// need nothing: intranet deletes what no CSV lists.
pub fn unplaced(
    edits: &DirectoryEdits,
    found: &BTreeSet<AppointmentRow>,
) -> BTreeMap<String, Vec<String>> {
    let mut files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (row, period) in edits {
        if let (Some(period), false) = (period, found.contains(row)) {
            files
                .entry(format!("{}.csv", slug(&row.cca_name)))
                .or_default()
                .push(line_with_period(row, period.as_str()));
        }
    }
    files
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
        files
            .entry(format!("{}.csv", slug(&row.cca_name)))
            .or_default()
            .push(row);
    }
    files
}

/// A row rendered as one CSV line with minimal quoting.
pub fn csv_line(row: &AppointmentRow) -> String {
    line_with_period(row, COMMITMENT_PERIOD)
}

fn line_with_period(row: &AppointmentRow, period: &str) -> String {
    [
        row.user_email.as_str(),
        row.cca_name.as_str(),
        row.position_name.as_str(),
        period,
    ]
    .map(csv_field)
    .join(",")
}

/// Trimmed like intranet's keys.
fn row_of(line: &str) -> Option<AppointmentRow> {
    let fields = csv_fields(line)?;
    let [email, cca, position, _period] = fields.as_slice() else {
        return None;
    };
    Some(AppointmentRow {
        cca_name: cca.trim().to_string(),
        position_name: position.trim().to_string(),
        user_email: email.trim().to_string(),
    })
}

fn csv_fields(line: &str) -> Option<Vec<String>> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    if quoted {
        return None;
    }
    fields.push(field);
    Some(fields)
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
    merge_lines(existing, rows.iter().map(csv_line))
}

/// `merge` for rows carrying the operator's period.
pub fn merge_lines(existing: Option<&str>, new: impl IntoIterator<Item = String>) -> String {
    let mut lines: Vec<String> = existing
        .unwrap_or("")
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty() && *l != HEADER)
        .map(String::from)
        .collect();
    lines.extend(new);
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
        Algorithm, Applicant, ApplicantIdx, Appointment, Appointments, Cca, Ledger, Pool, Position,
        PositionIdx, PositionType,
    };

    fn row(cca: &str, position: &str, email: &str) -> AppointmentRow {
        AppointmentRow {
            cca_name: cca.into(),
            position_name: position.into(),
            user_email: email.into(),
        }
    }

    const EXISTING: &str = "user_email,cca_name,position_name,commitment_period\n\
                            ann@x,Badminton F,Captain,full-year\n\
                            ben@x,Badminton F,Vice Captain,full-year\n\
                            cat@x,\"Say, \"\"Hi\"\"\",Chair,full-year\n";

    fn directory() -> crate::directory::Directory {
        use crate::directory::*;
        let user = |id, email: &str| User {
            id,
            name: email.into(),
            email: email.into(),
        };
        let position = |id, name: &str| CcaPosition {
            id,
            cca_id: 1,
            reporting_position_id: None,
            position_type: PositionKind::Lead,
            name: name.into(),
            description: None,
            capacity: None,
        };
        Directory::new(
            vec![user(1, "ann@x"), user(2, "ben@x")],
            vec![Cca {
                id: 1,
                name: "Badminton F".into(),
                kind: CcaKind::Sports,
                tier: CcaTier::None,
                cca_type: CcaType::TypeA,
                description: None,
                image_url: None,
            }],
            vec![position(10, "Captain"), position(11, "Vice Captain")],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn directory_edits_name_each_change_by_its_intranet_key() {
        use crate::directory::{CcaAppointment, CcaAppointmentChange, TeamStatus};
        let changes = [
            CcaAppointmentChange::Add {
                appointment: CcaAppointment {
                    user_id: 2,
                    position_id: 10,
                    commitment_period: CommitmentPeriod::Semester1,
                    points: 0,
                    team_status: TeamStatus::None,
                    created_at: None,
                },
            },
            CcaAppointmentChange::Remove {
                user_id: 1,
                position_id: 10,
            },
            CcaAppointmentChange::ChangePeriod {
                user_id: 2,
                position_id: 11,
                from: CommitmentPeriod::FullYear,
                to: CommitmentPeriod::Semester2,
            },
        ];
        assert_eq!(
            directory_edits(&changes, &directory()),
            DirectoryEdits::from([
                (
                    row("Badminton F", "Captain", "ben@x"),
                    Some(CommitmentPeriod::Semester1)
                ),
                (row("Badminton F", "Captain", "ann@x"), None),
                (
                    row("Badminton F", "Vice Captain", "ben@x"),
                    Some(CommitmentPeriod::Semester2)
                ),
            ])
        );
    }

    #[test]
    fn rewrite_drops_and_reperiods_edited_rows_where_they_stand() {
        let edits = DirectoryEdits::from([
            (row("Badminton F", "Captain", "ann@x"), None),
            (
                row("Say, \"Hi\"", "Chair", "cat@x"),
                Some(CommitmentPeriod::Semester2),
            ),
            (
                row("Badminton F", "Captain", "new@x"),
                Some(CommitmentPeriod::FullYear),
            ),
        ]);
        let (body, found) = rewrite(EXISTING, &edits);
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\n\
             ben@x,Badminton F,Vice Captain,full-year\n\
             cat@x,\"Say, \"\"Hi\"\"\",Chair,semester-2\n"
        );
        assert_eq!(
            found,
            BTreeSet::from([
                row("Badminton F", "Captain", "ann@x"),
                row("Say, \"Hi\"", "Chair", "cat@x"),
            ]),
            "a key this file does not list is left for another file"
        );
    }

    #[test]
    fn rewrite_matches_keys_the_way_intranet_trims_them() {
        let body = "ann@x , Badminton F,Captain ,full-year\n";
        let edits = DirectoryEdits::from([(row("Badminton F", "Captain", "ann@x"), None)]);
        assert_eq!(rewrite(body, &edits).0, "");
    }

    #[test]
    fn rewrite_leaves_a_file_without_edited_keys_unchanged() {
        let edits = DirectoryEdits::from([(row("Other", "Chair", "ann@x"), None)]);
        let (body, found) = rewrite(EXISTING, &edits);
        assert_eq!(body, EXISTING);
        assert!(found.is_empty());
    }

    #[test]
    fn unplaced_keeps_adds_no_file_took_and_drops_settled_removals() {
        let edits = DirectoryEdits::from([
            (
                row("Badminton F", "Captain", "new@x"),
                Some(CommitmentPeriod::Semester1),
            ),
            (
                row("Badminton F", "Captain", "ben@x"),
                Some(CommitmentPeriod::FullYear),
            ),
            (row("Badminton F", "Captain", "gone@x"), None),
        ]);
        let found = BTreeSet::from([row("Badminton F", "Captain", "ben@x")]);
        assert_eq!(
            unplaced(&edits, &found),
            BTreeMap::from([(
                "badminton_f.csv".to_string(),
                vec!["new@x,Badminton F,Captain,semester-1".to_string()]
            )])
        );
    }

    #[test]
    fn rows_map_allocations_to_names_and_exclude_existing_appointments() {
        let positions = vec![Position::new(
            10,
            Cca::new(1, "Sheares Media"),
            "Chair".into(),
            None,
            2,
            PositionType::MainComm,
            vec![ApplicantIdx(1), ApplicantIdx(2)],
        )];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "ann@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "ben@x".into(), vec![PositionIdx(10)]),
        ];
        // Ben already holds position 10: his pair must not re-export.
        let pool = Pool::new(applicants.clone(), positions.clone()).with_appointments(
            Appointments::from_iter([Appointment {
                applicant: ApplicantIdx(2),
                position: PositionIdx(10),
            }]),
        );
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[1], &positions[0]);

        let rows = rows_from(&ledger.finish(), &pool, &HashSet::new());
        assert_eq!(rows, vec![row("Sheares Media", "Chair", "ann@x")]);
    }

    #[test]
    fn rows_are_sorted_by_cca_then_position_then_email() {
        let positions = vec![
            Position::new(
                20,
                Cca::new(2, "Zeta"),
                "Chair".into(),
                None,
                1,
                PositionType::MainComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                10,
                Cca::new(1, "Alpha"),
                "Chair".into(),
                None,
                2,
                PositionType::MainComm,
                vec![ApplicantIdx(1), ApplicantIdx(2)],
            ),
        ];
        let applicants = vec![
            Applicant::new(
                1,
                "Ann".into(),
                "b@x".into(),
                vec![PositionIdx(20), PositionIdx(10)],
            ),
            Applicant::new(2, "Ben".into(), "a@x".into(), vec![PositionIdx(10)]),
        ];
        let pool = Pool::new(applicants.clone(), positions.clone());
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[0], &positions[1]);
        ledger.accept(&applicants[1], &positions[1]);

        let rows = rows_from(&ledger.finish(), &pool, &HashSet::new());
        assert_eq!(
            rows,
            vec![
                row("Alpha", "Chair", "a@x"),
                row("Alpha", "Chair", "b@x"),
                row("Zeta", "Chair", "b@x"),
            ]
        );
    }

    #[test]
    fn rows_include_preallocated_seats() {
        // A preallocated pair lands in the result via the preallocation pass
        // and must be exported like any other allocation.
        let positions = vec![Position::new(
            10,
            Cca::new(1, "Club"),
            "Chair".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )];
        let applicants = vec![Applicant::new(1, "Ann".into(), "ann@x".into(), vec![])];
        let pool = Pool::new(applicants, positions);
        let mut preallocations = crate::models::Preallocations::new();
        preallocations.grant(ApplicantIdx(1), PositionIdx(10));

        let result = crate::algorithm::run(&pool, &preallocations);
        assert_eq!(
            rows_from(&result, &pool, &HashSet::new()),
            vec![row("Club", "Chair", "ann@x")]
        );
    }

    #[test]
    fn rows_omit_excluded_positions() {
        let positions = vec![
            Position::new(
                10,
                Cca::new(1, "Alpha"),
                "Chair".into(),
                None,
                1,
                PositionType::MainComm,
                vec![ApplicantIdx(1)],
            ),
            Position::new(
                20,
                Cca::new(1, "Alpha"),
                "Vice".into(),
                None,
                1,
                PositionType::MainComm,
                vec![ApplicantIdx(2)],
            ),
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
        let positions = vec![Position::new(
            10,
            Cca::new(1, "Club"),
            "Chair".into(),
            None,
            1,
            PositionType::MainComm,
            vec![],
        )];
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
        assert_eq!(
            files["alpha_beta.csv"],
            vec![rows[0].clone(), rows[2].clone()]
        );
    }

    #[test]
    fn csv_line_orders_fields_and_appends_commitment_period() {
        assert_eq!(
            csv_line(&row("Alpha", "Chair", "a@x")),
            "a@x,Alpha,Chair,full-year"
        );
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
        assert_eq!(
            body,
            "user_email,cca_name,position_name,commitment_period\na@x,Alpha,Chair,full-year\n"
        );
    }

    #[test]
    fn merge_appends_missing_rows_keeps_existing_and_deduplicates() {
        let existing = "user_email,cca_name,position_name,commitment_period\n\
                        old@x,Alpha,Chair,full-year\n";
        let body = merge(
            Some(existing),
            &[
                row("Alpha", "Chair", "old@x"),
                row("Alpha", "Chair", "new@x"),
            ],
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
