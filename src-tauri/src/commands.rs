use cupid::models::{ApplicantIdx, Pool, PositionIdx};
use cupid::snapshot::Snapshot;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::task::block_in_place;

use crate::state::{snapshot_of, AppState, Inputs};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReceipt {
    pub path: String,
    pub rows: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReceipt {
    pub inserted: u32,
    pub snapshot: Snapshot,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeReceipt {
    pub deleted: u64,
    pub snapshot: Snapshot,
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).expect("rfc3339 format")
}

/// Connect to Postgres with the operator credentials.
fn connect() -> Result<postgres::Client, String> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let tls = postgres_native_tls::MakeTlsConnector::new(
        native_tls::TlsConnector::new().map_err(|e| e.to_string())?,
    );
    postgres::Client::connect(&url, tls).map_err(|e| e.to_string())
}

/// Load a complete, fresh input set from the database. Blocking.
fn load_inputs() -> Result<Inputs, String> {
    let (pool, appeals, warnings) = cupid::data::db::load().map_err(|e| e.to_string())?;
    Ok(Inputs { pool, appeals, warnings, last_result: None, synced_at: now_rfc3339() })
}

/// Reload the corpus and appeals from Postgres.
/// Invalidates any previous run: the returned snapshot has `run: None`.
#[tauri::command]
pub async fn sync(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = block_in_place(load_inputs)?;
    *guard = Some(inputs);
    Ok(snapshot_of(guard.as_ref().expect("just set")))
}

/// Run the two-pass matching (IA over BlockComm, then GS over Main/Sub)
/// against the loaded corpus. Stores the result and returns a snapshot
/// carrying the RunView.
#[tauri::command]
pub async fn run_matching(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;
    inputs.last_result =
        Some(block_in_place(|| cupid::algorithm::run(&inputs.pool, &inputs.appeals)));
    Ok(snapshot_of(inputs))
}

/// Grant a quota-exempt appeal for `(applicant, position)`. Persists to
/// cca_appeals, updates the in-memory whitelist, and invalidates any run
/// (its result no longer reflects the appeal set). The state lock is held
/// across the DB write, so a concurrent sync cannot interleave.
#[tauri::command]
pub async fn add_appeal(
    state: State<'_, AppState>,
    applicant_id: i32,
    position_id: i32,
    note: Option<String>,
) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;
    inputs.pool.applicant(ApplicantIdx(applicant_id)).ok_or("Unknown applicant.")?;
    inputs.pool.position(PositionIdx(position_id)).ok_or("Unknown position.")?;

    block_in_place(|| -> Result<(), String> {
        let mut client = connect()?;
        client
            .execute(
                "INSERT INTO cca_appeals (user_id, position_id, note) VALUES ($1, $2, $3) \
                 ON CONFLICT (user_id, position_id) DO UPDATE SET note = EXCLUDED.note",
                &[&applicant_id, &position_id, &note],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    })?;

    inputs.appeals.grant_with_note(ApplicantIdx(applicant_id), PositionIdx(position_id), note);
    inputs.last_result = None;
    Ok(snapshot_of(inputs))
}

/// Revoke an appeal. Deletes from cca_appeals, updates the in-memory
/// whitelist, and invalidates any run. The state lock is held across the
/// DB write, so a concurrent sync cannot interleave.
#[tauri::command]
pub async fn remove_appeal(
    state: State<'_, AppState>,
    applicant_id: i32,
    position_id: i32,
) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;

    block_in_place(|| -> Result<(), String> {
        let mut client = connect()?;
        client
            .execute(
                "DELETE FROM cca_appeals WHERE user_id = $1 AND position_id = $2",
                &[&applicant_id, &position_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    })?;

    inputs.appeals.revoke(ApplicantIdx(applicant_id), PositionIdx(position_id));
    inputs.last_result = None;
    Ok(snapshot_of(inputs))
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

/// Write the run's new allocations to cca_appointments (transactional,
/// idempotent via ON CONFLICT DO NOTHING), then reload the corpus so the
/// returned snapshot shows the new appointments as committed. The run is
/// consumed by the commit; a re-run starts from the fresh corpus.
#[tauri::command]
pub async fn commit(state: State<'_, AppState>) -> Result<CommitReceipt, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
    let result = inputs.last_result.as_ref().ok_or("Run matching first: nothing to commit.")?;
    let adds = adds_from(result, &inputs.pool);
    if adds.is_empty() {
        return Ok(CommitReceipt { inserted: 0, snapshot: snapshot_of(inputs) });
    }

    let inserted = block_in_place(|| -> Result<u32, String> {
        let mut client = connect()?;
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
    })?;

    let fresh = block_in_place(load_inputs)?;
    *guard = Some(fresh);
    Ok(CommitReceipt { inserted, snapshot: snapshot_of(guard.as_ref().expect("just set")) })
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
        let guard = state.inputs.lock().await;
        let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
        (snapshot_of(inputs), archive_rows(&inputs.pool))
        // Guard drops here: the file dialog can stay open indefinitely and
        // must not hold up other commands.
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

/// Permanently delete every preference row AND every appeal: both are
/// inputs to the allocation cycle that just ended, and neither may leak
/// into the next one. The corpus is reloaded afterwards so the returned
/// snapshot reflects the emptied market. Irreversible; the UI gates this
/// behind a completed archive and a typed confirmation.
#[tauri::command]
pub async fn purge(state: State<'_, AppState>) -> Result<PurgeReceipt, String> {
    let mut guard = state.inputs.lock().await;
    guard.as_ref().ok_or("Sync first: no corpus loaded.")?;

    let deleted = block_in_place(|| -> Result<u64, String> {
        let mut client = connect()?;
        let mut tx = client.transaction().map_err(|e| e.to_string())?;
        let user_rows = tx
            .execute("DELETE FROM cca_user_preferences", &[])
            .map_err(|e| e.to_string())?;
        let position_rows = tx
            .execute("DELETE FROM cca_position_preferences", &[])
            .map_err(|e| e.to_string())?;
        let appeal_rows = tx
            .execute("DELETE FROM cca_appeals", &[])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(user_rows + position_rows + appeal_rows)
    })?;

    let fresh = block_in_place(load_inputs)?;
    *guard = Some(fresh);
    Ok(PurgeReceipt { deleted, snapshot: snapshot_of(guard.as_ref().expect("just set")) })
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
