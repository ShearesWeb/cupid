use std::path::PathBuf;

use cupid::models::{Appeals, Pool};
use cupid::snapshot::Snapshot;
use tauri::State;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::state::{snapshot_of, AppState, Inputs};

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
