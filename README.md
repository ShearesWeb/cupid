# cupid

Two-sided matching engine and desktop console for Sheares CCA committee allocation.

Chairs rank candidates for their positions, applicants rank the positions they want, and cupid settles the market in two passes:

1. **Immediate Acceptance** over BlockComm positions: first valid claim wins, no bumping.
2. **Gale-Shapley (applicant-proposing deferred acceptance)** over MainComm and SubComm positions: chair-preferred applicants displace weaker tentative holders until the market is stable.

Every applicant is bound by one quota rule across both passes: `maincomm + blockcomm <= 2`, `subcomm <= 3`, and never `maincomm >= 1` with `subcomm >= 2`.
Appeals are per-`(applicant, position)` exemptions that bypass the quota but still compete for seats.
Existing appointments are the system of record: they pre-occupy seats, count toward quota, and are never modified by a run.

## Workspace

| Crate / dir | Role |
|---|---|
| `crates/cupid-core` | The engine: domain model, IA + GS passes, and the `snapshot` read model (one immutable, serializable projection serving every UI query as a lookup). Also builds the `cupid` CLI binary. |
| `src-tauri` | Tauri 2 shell. Commands: `sync`, `run_matching`, `add_appeal`, `remove_appeal`, `commit`, `archive`, `purge`. |
| `ui` | React 19 + TypeScript + Vite console. Dumb by design: all domain logic stays in Rust; the UI builds lookup maps from the snapshot and renders. |

## Data

Everything loads from Postgres (`DATABASE_URL`): `cca_user_preferences`, `cca_position_preferences`, `cca_appointments`, and `cca_appeals`, joined against `users`, `ccas`, and `cca_positions`.
Schema changelogs live in the intranet repo under `db/changelogs/table/cca-allocation/`.

Writes are deliberately narrow:

- **Commit** inserts new appointments only (`ON CONFLICT DO NOTHING`, single transaction); existing rows are never updated or deleted.
- **Appeals** insert/delete rows in `cca_appeals`; any change invalidates the current run.
- **Purge** deletes both preference tables after the run is committed and archived.

## Development

```sh
cargo test --workspace          # engine + command tests
npm --prefix ui run test        # UI index-layer tests
npm --prefix ui run build       # typecheck + bundle

DATABASE_URL=postgres://… npx @tauri-apps/cli dev   # run the desktop app
```

The live-DB smoke test is ignored by default: `cargo test -p cupid db_load_against_live_database -- --ignored`.
