use std::collections::HashSet;
use std::path::Path;

use cupid::data::conn::ConnSpec;
use cupid::data::preallocations::{self, PreallocationRecord};
use cupid::models::{ApplicantIdx, Pool, PositionIdx};
use cupid::snapshot::Snapshot;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::task::block_in_place;

use crate::export;
use crate::state::{snapshot_of, AppState, Inputs};

const NOT_CONNECTED: &str = "Not connected: supply the database credentials first.";

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReceipt {
    pub path: String,
    pub rows: usize,
}

/// What one export produced: the CSV rows written, the files they landed in,
/// and the branch + merge-request URL the operator must open. The database is
/// untouched — appointments appear only after the MR merges and intranet's CI
/// applies it.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReceipt {
    pub rows: usize,
    pub files: Vec<String>,
    pub branch: String,
    pub pr_url: String,
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

/// The active connection target, or a friendly error when none is set.
async fn spec_of(state: &State<'_, AppState>) -> Result<ConnSpec, String> {
    state.conn.lock().await.clone().ok_or_else(|| NOT_CONNECTED.to_string())
}

/// Load a complete, fresh input set: the corpus from the database (read-only)
/// and the preallocations from the local store. Blocking.
fn load_inputs(spec: &ConnSpec, store: &Path) -> Result<Inputs, String> {
    let pool = cupid::data::db::load(spec).map_err(|e| e.to_string())?;
    let records = preallocations::read_file(store).map_err(|e| e.to_string())?;
    let (preallocations, warnings) = preallocations::resolve(&records, &pool);
    Ok(Inputs {
        pool,
        records,
        preallocations,
        warnings,
        last_result: None,
        synced_at: now_rfc3339(),
    })
}

/// Point the app at a Supabase project. Verifies the credentials with a
/// probe query before storing them; on success any loaded corpus is dropped
/// (it belongs to the previous database) and the target description returned.
#[tauri::command]
pub async fn connect(
    state: State<'_, AppState>,
    project_ref: String,
    password: String,
    region: Option<String>,
) -> Result<String, String> {
    let spec = ConnSpec::supabase(&project_ref, &password, region.as_deref());

    block_in_place(|| -> Result<(), String> {
        let mut client = spec.connect().map_err(|e| e.to_string())?;
        client.batch_execute("SELECT 1").map_err(|e| e.to_string())?;
        Ok(())
    })?;

    let mut inputs = state.inputs.lock().await;
    *state.conn.lock().await = Some(spec.clone());
    *inputs = None;
    Ok(spec.describe())
}

/// Where the app currently points: `None` until credentials are supplied
/// (or DATABASE_URL seeded the target at startup). Never includes secrets.
#[tauri::command]
pub async fn connection_info(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.conn.lock().await.as_ref().map(ConnSpec::describe))
}

/// Reload the corpus from Postgres and the preallocations from the local
/// store. Invalidates any previous run: the returned snapshot has `run: None`.
#[tauri::command]
pub async fn sync(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let spec = spec_of(&state).await?;
    let store = state.store_path();
    let inputs = block_in_place(|| load_inputs(&spec, &store))?;
    *guard = Some(inputs);
    Ok(snapshot_of(guard.as_ref().expect("just set")))
}

/// Run the allocation (preallocations, then IA over BlockComm, then GS over
/// Main/Sub) against the loaded corpus. Stores the result and returns a
/// snapshot carrying the RunView.
#[tauri::command]
pub async fn run_matching(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;
    inputs.last_result = Some(block_in_place(|| {
        cupid::algorithm::run(&inputs.pool, &inputs.preallocations)
    }));
    Ok(snapshot_of(inputs))
}

/// Preallocate `(applicant, position)`: the pair holds the position outright
/// in the next run. Preallocations are operator state local to this machine:
/// the pair is written to the store file and the in-memory set, and any run
/// is invalidated (its result no longer reflects the preallocation set).
#[tauri::command]
pub async fn add_preallocation(
    state: State<'_, AppState>,
    applicant_id: i32,
    position_id: i32,
    note: Option<String>,
) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;
    inputs.pool.applicant(ApplicantIdx(applicant_id)).ok_or("Unknown applicant.")?;
    inputs.pool.position(PositionIdx(position_id)).ok_or("Unknown position.")?;

    let record = PreallocationRecord { user_id: applicant_id, position_id, note: note.clone() };
    match inputs
        .records
        .iter_mut()
        .find(|r| r.user_id == applicant_id && r.position_id == position_id)
    {
        Some(existing) => existing.note = record.note,
        None => inputs.records.push(record),
    }
    preallocations::write_file(&state.store_path(), &inputs.records)
        .map_err(|e| e.to_string())?;

    inputs.preallocations.grant_with_note(
        ApplicantIdx(applicant_id),
        PositionIdx(position_id),
        note,
    );
    inputs.last_result = None;
    Ok(snapshot_of(inputs))
}

/// Revoke a preallocation: remove it from the store file and the in-memory
/// set, and invalidate any run.
#[tauri::command]
pub async fn remove_preallocation(
    state: State<'_, AppState>,
    applicant_id: i32,
    position_id: i32,
) -> Result<Snapshot, String> {
    let mut guard = state.inputs.lock().await;
    let inputs = guard.as_mut().ok_or("Sync first: no corpus loaded.")?;

    inputs
        .records
        .retain(|r| !(r.user_id == applicant_id && r.position_id == position_id));
    preallocations::write_file(&state.store_path(), &inputs.records)
        .map_err(|e| e.to_string())?;

    inputs.preallocations.revoke(ApplicantIdx(applicant_id), PositionIdx(position_id));
    inputs.last_result = None;
    Ok(snapshot_of(inputs))
}

/// Probe SSH push access to the intranet repo without touching anything.
/// Resolves to a confirmation line; the error carries operator guidance
/// (missing key vs. missing push permission). The UI runs this as the first
/// half of the export step, before any clone or branch exists.
#[tauri::command]
pub async fn check_access() -> Result<String, String> {
    block_in_place(export::check_push_access)?;
    Ok("SSH push access to ShearesWeb/intranet verified.".to_string())
}

/// Resolve the operator's held-back position ids against the corpus. An id
/// the engine does not know means the console and the corpus have diverged;
/// silently ignoring it would export a position the operator held back, or
/// purge preferences they meant to keep.
fn excluded_set(pool: &Pool, excluded: &[i32]) -> Result<HashSet<PositionIdx>, String> {
    excluded
        .iter()
        .map(|&id| {
            pool.position(PositionIdx(id))
                .map(|_| PositionIdx(id))
                .ok_or_else(|| format!("Unknown position in the exclusion list: {id}."))
        })
        .collect()
}

/// The preallocations that survive a purge: those on positions the operator
/// held back. Every other record is an input to the cycle that just ended.
fn retained_preallocations(
    records: &[PreallocationRecord],
    excluded: &HashSet<PositionIdx>,
) -> Vec<PreallocationRecord> {
    records
        .iter()
        .filter(|r| excluded.contains(&PositionIdx(r.position_id)))
        .cloned()
        .collect()
}

/// Export the run's new allocations as CSVs and push them to the intranet
/// repo: every settled allocation — preallocated seats included — that is not
/// already an existing appointment becomes a row in cupid's per-CCA files
/// under `data/cca-appointment/allocation/`. Positions listed in `excluded`
/// are held back: their seats stay out of the export, and `purge` must be
/// given the same list so their preferences survive into the next cycle.
/// Nothing is written to the database; the receipt carries the merge-request
/// URL the operator must open to land the change. The run is kept:
/// appointments show up on the next sync after the MR merges.
#[tauri::command]
pub async fn commit(
    state: State<'_, AppState>,
    excluded: Vec<i32>,
) -> Result<ExportReceipt, String> {
    let guard = state.inputs.lock().await;
    let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
    let result = inputs.last_result.as_ref().ok_or("Run matching first: nothing to export.")?;
    let excluded = excluded_set(&inputs.pool, &excluded)?;
    let rows = cupid::export::rows_from(result, &inputs.pool, &excluded);
    if rows.is_empty() {
        return Err(if excluded.is_empty() {
            "Nothing to export: the run adds no new appointments.".to_string()
        } else {
            "Nothing to export: every new appointment is on an excluded position.".to_string()
        });
    }

    let row_count = rows.len();
    let export_root = state.export_root();
    let timestamp = now_rfc3339();
    let (files, branch, pr_url) =
        block_in_place(|| export::publish(&export_root, &timestamp, rows))?;
    Ok(ExportReceipt { rows: row_count, files, branch, pr_url })
}

pub fn archive_rows(pool: &cupid::models::Pool) -> usize {
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

/// Permanently delete the preference rows in the database AND the local
/// preallocations belonging to the allocation cycle that just ended, so
/// neither leaks into the next one. Positions listed in `excluded` are held
/// back: their preference rows, chair rankings and preallocations survive
/// untouched, which is what lets an operator re-run a single role later. Pass
/// the same list `commit` was given. This is cupid's one write path — it opens
/// the sole read-write connection. The corpus is reloaded afterwards so the
/// returned snapshot reflects the emptied market. Irreversible; the UI gates
/// this behind a completed export + archive and a typed confirmation.
#[tauri::command]
pub async fn purge(
    state: State<'_, AppState>,
    excluded: Vec<i32>,
) -> Result<PurgeReceipt, String> {
    let mut guard = state.inputs.lock().await;
    let spec = spec_of(&state).await?;
    let inputs = guard.as_ref().ok_or("Sync first: no corpus loaded.")?;
    let excluded = excluded_set(&inputs.pool, &excluded)?;
    let retained = retained_preallocations(&inputs.records, &excluded);

    // `<> ALL` rather than `= ANY` over the kept ids: an empty exclusion list
    // then still clears the table outright, orphan rows included.
    let held: Vec<i32> = excluded.iter().map(|p| p.0).collect();
    let deleted = block_in_place(|| -> Result<u64, String> {
        let mut client = spec.connect_read_write().map_err(|e| e.to_string())?;
        let mut tx = client.transaction().map_err(|e| e.to_string())?;
        let user_rows = tx
            .execute("DELETE FROM preferred_positions WHERE position_id <> ALL($1)", &[&held])
            .map_err(|e| e.to_string())?;
        let position_rows = tx
            .execute("DELETE FROM preferred_candidates WHERE position_id <> ALL($1)", &[&held])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(user_rows + position_rows)
    })?;

    let store = state.store_path();
    preallocations::write_file(&store, &retained).map_err(|e| e.to_string())?;

    let fresh = block_in_place(|| load_inputs(&spec, &store))?;
    *guard = Some(fresh);
    Ok(PurgeReceipt { deleted, snapshot: snapshot_of(guard.as_ref().expect("just set")) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cupid::models::*;

    fn two_position_pool() -> Pool {
        let positions = vec![
            Position::new(10, Cca::new(1, "C"), "Chair".into(), None, 1,
                PositionType::MainComm, vec![]),
            Position::new(20, Cca::new(1, "C"), "Vice".into(), None, 1,
                PositionType::MainComm, vec![]),
        ];
        Pool::new(vec![Applicant::new(1, "A".into(), "a@x".into(), vec![])], positions)
    }

    fn record(position_id: i32) -> PreallocationRecord {
        PreallocationRecord { user_id: 1, position_id, note: None }
    }

    #[test]
    fn excluded_set_resolves_known_positions() {
        let set = excluded_set(&two_position_pool(), &[20]).unwrap();
        assert_eq!(set, HashSet::from([PositionIdx(20)]));
    }

    #[test]
    fn excluded_set_rejects_a_position_outside_the_corpus() {
        let err = excluded_set(&two_position_pool(), &[99]).unwrap_err();
        assert!(err.contains("99"), "the error names the bad id: {err}");
    }

    #[test]
    fn purge_keeps_preallocations_on_excluded_positions_only() {
        let records = vec![record(10), record(20)];
        assert_eq!(
            retained_preallocations(&records, &HashSet::from([PositionIdx(20)])),
            vec![record(20)]
        );
    }

    #[test]
    fn purge_with_nothing_excluded_keeps_no_preallocations() {
        let records = vec![record(10), record(20)];
        assert_eq!(retained_preallocations(&records, &HashSet::new()), vec![]);
    }

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
}
