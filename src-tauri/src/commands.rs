use std::path::PathBuf;

use cupid::models::{Appeals, Pool};
use cupid::snapshot::Snapshot;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::state::{snapshot_of, AppState, Inputs};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReceipt {
    pub path: String,
    pub rows: usize,
}

fn appeals_dir() -> PathBuf {
    std::env::var("CUPID_APPEALS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/appeals"))
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).expect("rfc3339 format")
}

/// Reload the corpus from Postgres and appeals from disk.
/// Invalidates any previous run: the returned snapshot has `run: None`.
#[tauri::command]
pub async fn sync(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let (pool, appeals): (Pool, Appeals) =
        tauri::async_runtime::spawn_blocking(|| -> Result<_, String> {
            let pool = cupid::data::db::load().map_err(|e| e.to_string())?;
            let appeals = cupid::data::appeals::load_and_resolve(&appeals_dir(), &pool)
                .map_err(|e| e.to_string())?;
            Ok((pool, appeals))
        })
        .await
        .map_err(|e| e.to_string())??;

    let mut guard = state.inputs.lock().map_err(|e| e.to_string())?;
    *guard = Some(Inputs { pool, appeals, last_result: None, synced_at: now_rfc3339() });
    Ok(snapshot_of(guard.as_ref().expect("just set")))
}

/// New (applicant, position) pairs this run wants to write: every settled
/// allocation that is not already an existing appointment. Adds-only by
/// construction; nothing is ever updated or deleted.
pub fn adds_from(result: &cupid::models::MatchResult, pool: &Pool) -> Vec<(i32, i32)> {
    let mut adds: Vec<(i32, i32)> = result
        .all()
        .filter(|a| !pool.appointments().held_by(a.applicant_id).contains(&a.position_id))
        .map(|a| (a.applicant_id.0, a.position_id.0))
        .collect();
    adds.sort();
    adds
}

/// Write the run's new allocations to cca_appointments. Transactional and
/// idempotent: ON CONFLICT (user_id, position_id) DO NOTHING.
#[tauri::command]
pub async fn commit(state: State<'_, AppState>) -> Result<u32, String> {
    let adds = {
        let guard = state.inputs.lock().map_err(|e| e.to_string())?;
        let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
        let result = inputs.last_result.as_ref().ok_or("Run matching first: nothing to commit.")?;
        adds_from(result, &inputs.pool)
    };
    if adds.is_empty() {
        return Ok(0);
    }
    tauri::async_runtime::spawn_blocking(move || -> Result<u32, String> {
        let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
        let tls = postgres_native_tls::MakeTlsConnector::new(
            native_tls::TlsConnector::new().map_err(|e| e.to_string())?,
        );
        let mut client = postgres::Client::connect(&url, tls).map_err(|e| e.to_string())?;
        let mut tx = client.transaction().map_err(|e| e.to_string())?;
        let mut inserted = 0u32;
        for (user_id, position_id) in &adds {
            inserted += tx
                .execute(
                    "INSERT INTO cca_appointments (user_id, position_id) VALUES ($1, $2) \
                     ON CONFLICT (user_id, position_id) DO NOTHING",
                    &[user_id, position_id],
                )
                .map_err(|e| e.to_string())? as u32;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(inserted)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn archive_rows(pool: &Pool) -> usize {
    pool.applicants().map(|a| a.preferences().len()).sum::<usize>()
        + pool.positions().map(|p| p.ranking().len()).sum::<usize>()
}

/// Export a full verified backup (corpus + committed + run) as JSON via a
/// native save dialog. Must run before purge.
#[tauri::command]
pub async fn archive(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<ArchiveReceipt, String> {
    let (snapshot, rows) = {
        let guard = state.inputs.lock().map_err(|e| e.to_string())?;
        let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
        (snapshot_of(inputs), archive_rows(&inputs.pool))
    };
    let default_name = format!(
        "cupid-archive-{}.json",
        &now_rfc3339()[..10] // YYYY-MM-DD
    );
    let path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_file_name(&default_name).blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?
    .ok_or("cancelled")?;
    let path = path.into_path().map_err(|e| e.to_string())?;

    let export = serde_json::json!({
        "exportedAt": now_rfc3339(),
        "rows": rows,
        "snapshot": snapshot,
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&export).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(ArchiveReceipt { path: path.display().to_string(), rows })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cupid::models::*;

    #[test]
    fn archive_rows_counts_prefs_plus_rankings() {
        let positions = vec![Position::new(10, Cca::new(1, "C"), "P".into(), None, 1,
            PositionType::MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)])];
        let applicants = vec![
            Applicant::new(1, "A".into(), "a@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "B".into(), "b@x".into(), vec![]),
        ];
        let pool = Pool::new(applicants, positions);
        assert_eq!(archive_rows(&pool), 3, "1 pref row + 2 ranking rows");
    }

    #[test]
    fn adds_exclude_pairs_already_appointed() {
        let positions = vec![Position::new(10, Cca::new(1, "C"), "P".into(), None, 2,
            PositionType::MainComm, vec![ApplicantIdx(1), ApplicantIdx(2)])];
        let applicants = vec![
            Applicant::new(1, "Ann".into(), "a@x".into(), vec![PositionIdx(10)]),
            Applicant::new(2, "Ben".into(), "b@x".into(), vec![PositionIdx(10)]),
        ];
        // Ben (applicant 2) already holds position 10 by appointment.
        let pool = Pool::new(applicants.clone(), positions.clone())
            .with_appointments(Appointments::from_iter([
                Appointment { applicant: ApplicantIdx(2), position: PositionIdx(10) },
            ]));
        // A (stale or duplicated) run seats BOTH applicants in position 10.
        let mut ledger = Ledger::new(Algorithm::GaleShapley);
        ledger.accept(&applicants[0], &positions[0]);
        ledger.accept(&applicants[1], &positions[0]);
        let result = ledger.finish();

        // Ben's pair is filtered out: it is already an appointment.
        let adds = adds_from(&result, &pool);
        assert_eq!(adds, vec![(1, 10)], "only genuinely new pairs");
    }
}
