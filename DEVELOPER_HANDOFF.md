# Cupid CCA directory: developer handoff

This guide describes the Rust work already completed and the remaining work for
the CCA homepage and appointment workflow. The current branch is
`feat/cca-directory`. Check `git status --short` before starting; the work is
not committed.

## 1. System overview

Cupid has two model layers because it has two jobs:

```text
PostgreSQL
    │
    ├── all users, CCAs, positions, appointments
    │       └── Directory ──► DirectorySnapshot ──► future CCA homepage
    │
    └── preferences + chair rankings
            └── adapter ──► Pool ──► matching ──► AllocationSnapshot
```

`Directory` is the complete current CCA state. It includes all eight database
position types: `lead`, `vice`, `blockcomm`, `maincomm`, `subcomm`,
`team-manager`, `member`, and `resident`.

`Pool` is only the allocation input. It contains blockcomm, maincomm and
subcomm positions with known capacity. The existing allocation algorithms and
quota rules operate on `Pool` and must remain unchanged.

`DirectorySnapshot` is the future homepage JSON model. `AllocationSnapshot` is
the existing allocation-screen JSON model. Do not combine them: their fields
and position universes differ.

## 2. Current database loading flow

```text
sync
  │
  ├─ read-only RepeatableRead transaction
  ├─ load users, CCAs, positions, appointments
  ├─ load preferences and chair rankings
  ├─ construct Directory
  ├─ derive Pool from Directory
  ├─ cache both in Tauri Inputs
  └─ clear the old allocation run
```

Relevant files:

- `crates/cupid-core/src/data/directory.rs` loads the complete directory.
- `crates/cupid-core/src/data/db.rs::load_all` loads both views together.
- `crates/cupid-core/src/data/resolve.rs::from_directory` derives `Pool`.
- `src-tauri/src/state.rs` caches `Directory` and `Pool`.
- `src-tauri/src/commands.rs::directory_snapshot` returns the cached directory.

`directory_snapshot` does not query the database. A new `sync` refreshes both
models. `Directory` and `Pool` are separate owned values, so changing one does
not update the other automatically.

## 3. Rust directory model

Definitions: `crates/cupid-core/src/directory/`.

```rust
User { id, name, email }

Cca {
    id, name, kind, tier, cca_type, description, image_url
}

CcaPosition {
    id, cca_id, reporting_position_id,
    position_type, name, description, capacity
}

CcaAppointment {
    user_id, position_id, commitment_period,
    points, team_status, created_at
}
```

An appointment is identified by `(user_id, position_id)`; there is no separate
appointment ID. `capacity: None` means unlimited and `Some(0)` means closed.
`reporting_position_id` is the reporting link and may point across CCAs.

`PositionKind::allocation_type()` maps only block/main/sub positions into the
old allocation `PositionType`. `PositionKind::can_manage_appointments()` is a
backend policy: lead, vice, blockcomm, maincomm, subcomm and team-manager are
editable; member and resident are read-only.

`Directory` validates unknown users/positions, duplicate pairs, protected
roles, per-semester position capacity and overlapping non-resident CCA
appointments. Failed edits are atomic. Period updates preserve points, team
status and creation time.

The current methods are:

```rust
add_appointment(user_id, position_id, period)
remove_appointment(user_id, position_id)
update_appointment_period(user_id, position_id, period)
```

These methods currently change only an in-memory `Directory`. They do not write
PostgreSQL, edit CSVs, rebuild `Pool`, or clear a matching result.

## 4. Effective data versus pending edits

`Directory` is the effective state read from PostgreSQL. Do not use it as a UI
draft buffer. Add a separate pending-change model:

```text
effective Directory + pending changes
              │
              ▼
       DirectorySnapshot
```

Suggested shape:

```rust
enum CcaAppointmentChange {
    Add { appointment: CcaAppointment },
    Remove { user_id: i32, position_id: i32 },
    ChangePeriod {
        user_id: i32,
        position_id: i32,
        from: CommitmentPeriod,
        to: CommitmentPeriod,
    },
}

struct CcaAppointmentChangeSet {
    base_sync: String,
    changes: Vec<CcaAppointmentChange>,
}
```

Validate the complete proposed state against a cloned directory before keeping
a change. This prevents a failed move from losing the original appointment.
Pending changes must not affect allocation `Pool`.

For display, keep status out of `CcaAppointment` and add a snapshot-only view:

```rust
enum CcaAppointmentStatus {
    Existing,
    Added,    // green
    Modified, // mutation color
    Removed,  // mutation color
}

struct CcaAppointmentView {
    appointment: CcaAppointment,
    status: CcaAppointmentStatus,
}
```

`DirectorySnapshot.appointments` should eventually contain these view rows.
Keep removed rows visible so the operator can review them. A remote refresh may
replace the effective directory while pending changes remain visible; revalidate
pending changes against the refreshed directory before publication.

## 5. Work remaining

### A. Read-only homepage

Add TypeScript types and an API wrapper for `directory_snapshot`. Add CCA list
and detail screens showing:

- every CCA, including empty CCAs;
- CCA metadata;
- every position and position type;
- reporting relationships;
- capacity (`unlimited` for null, `closed` for zero);
- current holders and commitment periods;
- member/resident rows without edit controls.

Use `AllocationSnapshot` for existing allocation screens. Do not reuse its
`block | main | sub` type union for directory positions.

### B. Appointment editing

Add Tauri commands for add, remove and period update. Commands must validate in
Rust, store changes in a `CcaAppointmentChangeSet`, and return useful errors.
Add a review panel with discard support. Points and team status are displayed
but are not editable in this feature.

### C. Appointment publication

The existing allocation `commit` command publishes allocation CSVs and must stay
unchanged. Appointment changes need a separate publisher.

The recommended path is the intranet CSV/PR workflow:

```text
change set → review → update appointment CSVs → branch/PR → remote refresh
```

The publisher must handle additions, removals and period changes across all
appointment CSV directories, preserve unrelated rows and periods, detect stale
or conflicting edits, and never emit resident rows. A PR URL means “published
for review,” not “effective in the directory.”

Direct PostgreSQL writes are an alternative, but require a decision about CSV
authority and a concurrency strategy. That decision is still open.

### D. Refresh behavior

When effective remote data changes, refresh `Directory` and `Pool` together,
re-resolve preallocations and clear the old allocation result. No separate
“sync succeeded” flag is needed.

## 6. Completion checklist

- All CCAs and all eight position types are visible.
- Added, modified and removed appointment rows have display status.
- Member/resident edits fail in Rust and are hidden from edit controls.
- Capacity and semester CCA-conflict rules are enforced.
- Pending changes are separate from effective `Directory` and allocation `Pool`.
- Appointment publication handles add, remove and period update.
- Allocation commit behavior remains unchanged.
- Refresh rebuilds both models and clears stale matching results.
- Rust, UI and desktop checks pass with the required Tauri dependencies.

## 7. Verification already completed

The core Rust suite passed: 131 tests passed and 2 database tests were ignored.
The directory PostgreSQL loader test passed against disposable PostgreSQL 17.
Core Clippy passed with warnings denied. Full workspace verification was blocked
in the original environment by missing `pkg-config` and Linux Tauri libraries.

Run after further changes:

```sh
cargo test -p cupid
cargo clippy -p cupid --all-targets -- -D warnings
cargo test --workspace
npm run test -w ui
npm run lint -w ui
npm run build -w ui
git diff --check
```

Schema references: `../intranet/db/changelogs/core/01.enum.core.sql`,
`12.table.ccas.sql`, `13.table.cca-positions.sql`,
`14.table.cca-appointments.sql`, and the appointment validation triggers.
