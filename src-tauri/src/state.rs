use std::sync::Mutex;

use cupid::models::{Appeals, MatchResult, Pool};
use cupid::snapshot::{self, Snapshot};

pub struct Inputs {
    pub pool: Pool,
    pub appeals: Appeals,
    pub last_result: Option<MatchResult>,
    pub synced_at: String,
}

#[derive(Default)]
pub struct AppState {
    pub inputs: Mutex<Option<Inputs>>,
}

/// Project the current inputs into the immutable read model served to the UI.
pub fn snapshot_of(inputs: &Inputs) -> Snapshot {
    snapshot::build(
        &inputs.pool,
        &inputs.appeals,
        inputs.last_result.as_ref(),
        inputs.synced_at.clone(),
    )
}
