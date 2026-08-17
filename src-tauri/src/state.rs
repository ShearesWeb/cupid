use std::path::PathBuf;

use cupid::data::conn::ConnSpec;
use cupid::data::preallocations::PreallocationRecord;
use cupid::models::{MatchResult, Pool, Preallocations};
use cupid::snapshot::{self, Snapshot};
use tokio::sync::Mutex;

pub struct Inputs {
    pub pool: Pool,
    /// The local store's raw records, stale entries included: the store file
    /// is the source of truth and a save must never drop an entry merely
    /// because the current corpus cannot resolve it.
    pub records: Vec<PreallocationRecord>,
    /// The resolvable subset of `records`, as the algorithm consumes it.
    pub preallocations: Preallocations,
    /// Load-time anomalies (stale or conflicting preallocation records etc.)
    /// carried into every snapshot.
    pub warnings: Vec<String>,
    pub last_result: Option<MatchResult>,
    pub synced_at: String,
}

pub struct AppState {
    /// One async lock serializes every command. Mutating commands hold it
    /// across their DB round-trip (via `block_in_place`), so the in-memory
    /// image can never diverge from the database, and an engine panic cannot
    /// poison it the way a `std::sync::Mutex` would.
    pub inputs: Mutex<Option<Inputs>>,
    /// The active connection target. Seeded from DATABASE_URL when the app
    /// starts (dev convenience); replaced by the `connect` command.
    pub conn: Mutex<Option<ConnSpec>>,
    /// The app's data directory: home of the preallocation store and the
    /// intranet checkouts exports are pushed from.
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn from_env(data_dir: PathBuf) -> Self {
        AppState {
            inputs: Mutex::new(None),
            conn: Mutex::new(std::env::var("DATABASE_URL").ok().map(ConnSpec::Url)),
            data_dir,
        }
    }

    /// The local preallocation store file.
    pub fn store_path(&self) -> PathBuf {
        self.data_dir.join("preallocations.json")
    }

    /// Where exports clone the intranet repo.
    pub fn export_root(&self) -> PathBuf {
        self.data_dir.join("exports")
    }
}

/// Project the current inputs into the immutable read model served to the UI.
pub fn snapshot_of(inputs: &Inputs) -> Snapshot {
    snapshot::build(
        &inputs.pool,
        &inputs.preallocations,
        inputs.last_result.as_ref(),
        inputs.synced_at.clone(),
        inputs.warnings.clone(),
    )
}
