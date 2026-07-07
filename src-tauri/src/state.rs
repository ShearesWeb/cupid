use cupid::models::{Appeals, MatchResult, Pool};
use cupid::snapshot::{self, Snapshot};
use tokio::sync::Mutex;

pub struct Inputs {
    pub pool: Pool,
    pub appeals: Appeals,
    /// Load-time anomalies (stale appeal rows etc.) carried into every snapshot.
    pub warnings: Vec<String>,
    pub last_result: Option<MatchResult>,
    pub synced_at: String,
}

#[derive(Default)]
pub struct AppState {
    /// One async lock serializes every command. Mutating commands hold it
    /// across their DB round-trip (via `block_in_place`), so the in-memory
    /// image can never diverge from the database, and an engine panic cannot
    /// poison it the way a `std::sync::Mutex` would.
    pub inputs: Mutex<Option<Inputs>>,
}

/// Project the current inputs into the immutable read model served to the UI.
pub fn snapshot_of(inputs: &Inputs) -> Snapshot {
    snapshot::build(
        &inputs.pool,
        &inputs.appeals,
        inputs.last_result.as_ref(),
        inputs.synced_at.clone(),
        inputs.warnings.clone(),
    )
}
