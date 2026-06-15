use std::fmt::Write;

use crate::models::{MatchResult, Pool};

/// Render a read-only text report of the allocation.
pub fn render(result: &MatchResult, pool: &Pool) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Allocations ===");
    for position in pool.positions() {
        let cca = pool.cca_name(position.cca_id).unwrap_or("?");
        let _ = writeln!(out, "[{}] {} (cap {}):", cca, position.name, position.capacity);
        let seats = result.for_position(position.id);
        if seats.is_empty() {
            let _ = writeln!(out, "    (none)");
        }
        for a in seats {
            let name = applicant_name(pool, a.applicant_id);
            let _ = writeln!(
                out,
                "    #{} {} (chair rank {:?}, pref {:?})",
                a.applicant_id.0, name, a.chair_rank, a.applicant_rank
            );
        }
    }

    let unmatched = result.unmatched(pool.applicants());
    let _ = writeln!(out, "\n=== Unmatched applicants: {} ===", unmatched.len());
    for id in unmatched {
        let _ = writeln!(out, "    #{} {}", id.0, applicant_name(pool, id));
    }

    let _ = writeln!(out, "\n=== Unfilled seats ===");
    for (pid, empty) in result.unfilled(pool.positions()) {
        let _ = writeln!(out, "    position #{}: {} seat(s) left", pid.0, empty);
    }

    out
}

fn applicant_name(pool: &Pool, id: crate::models::ApplicantIdx) -> &str {
    pool.applicants()
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.name.as_str())
        .unwrap_or("?")
}

/// Print the report to stdout.
pub fn print(result: &MatchResult, pool: &Pool) {
    print!("{}", render(result, pool));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::algorithm::run;
    use crate::models::{Appeals, Applicant, ApplicantIdx, CCAIdx, Position, PositionIdx, PositionType};

    #[test]
    fn render_lists_seated_and_unmatched() {
        let positions = vec![Position::new(
            1, 1, "Head".into(), None, 1, PositionType::MainComm, vec![ApplicantIdx(1)],
        )];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![PositionIdx(1)]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![]), // unmatched
        ];
        let mut ccas = HashMap::new();
        ccas.insert(CCAIdx(1), "Chess".to_string());
        let pool = Pool::new(applicants, positions, ccas);

        let result = run(pool.applicants(), pool.positions(), &Appeals::new());
        let text = render(&result, &pool);

        assert!(text.contains("Chess"), "cca name missing:\n{text}");
        assert!(text.contains("Head"), "position name missing:\n{text}");
        assert!(text.contains("Ann"), "seated applicant missing:\n{text}");
        assert!(text.contains("Ben"), "unmatched applicant missing:\n{text}");
    }
}
