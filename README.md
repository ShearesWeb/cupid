# cupid

Two-sided matching engine and desktop console for Sheares CCA committee allocation.

Chairs rank candidates for their positions, applicants rank the positions they want, and cupid settles the market in three passes:

1. **Preallocations**: operator-fixed `(applicant, position)` pairs are seated outright. They consume the seat and the holder's quota, are never scored, and can never be displaced.
2. **Immediate Acceptance** over BlockComm positions: first valid claim wins, no bumping.
3. **Gale-Shapley (applicant-proposing deferred acceptance)** over MainComm and SubComm positions: chair-preferred applicants displace weaker tentative holders until the market is stable.

Every applicant is bound by two rules across all passes: the type quota (`maincomm + blockcomm <= 2`, `subcomm <= 3`, and never `maincomm >= 1` with `subcomm >= 2`), and one non-resident position per CCA (mirroring the database's exclusion constraint, with `member`-style roles outside the market counted too).
Existing appointments are the system of record: they pre-occupy seats, count toward quota and the CCA rule, and are never modified by a run.

## Workspace

| Crate / dir | Role |
|---|---|
| `crates/cupid-core` | The engine: domain model, preallocation + IA + GS passes, and the `snapshot` read model (one immutable, serializable projection serving every UI query as a lookup). Also builds the `cupid` CLI binary. |
| `src-tauri` | Tauri 2 shell. Commands: `connect`, `connection_info`, `sync`, `run_matching`, `add_preallocation`, `remove_preallocation`, `commit`, `archive`, `purge`. |
| `ui` | React 19 + TypeScript + Vite console. Dumb by design: all domain logic stays in Rust; the UI builds lookup maps from the snapshot and renders. |

## Connection

Supply the Supabase project ref and database password in the app (splash screen on first launch, "Switch database" in the sidebar afterwards).
Without a region, cupid connects directly to `db.<ref>.supabase.co:5432`; that host is IPv6-only unless the project has the IPv4 add-on.
With a region (e.g. `ap-southeast-1`), cupid routes through the session pooler at `aws-0-<region>.pooler.supabase.com:5432` as `postgres.<ref>`, which works on IPv4 networks.
The project ref and region persist locally between launches; the password lives in memory only.
`DATABASE_URL` still works as a startup fallback for development, and remains required for the CLI binary.
Every session connects with `default_transaction_read_only=on`; only the end-of-cycle purge opens a writable connection, so the role needs delete grants on the two preference tables and nothing more.

## Data

The corpus loads from Postgres: `preferred_positions`, `preferred_candidates`, and `cca_appointments`, joined against `users`, `ccas`, and `cca_positions`.
Schema changelogs live in the intranet repo under `db/changelogs/cca-allocation/`.
Preallocations are operator state local to the machine: they live in `preallocations.json` in the app data directory and never touch the database.

Cupid does not write appointments.
Committing a run exports it instead:

- **Export** turns the run's new allocations — preallocated seats included, already-existing appointments excluded — into per-CCA CSV files under `data/cca-appointment/allocation/` in a fresh shallow clone of the intranet repo, then pushes a `cupid/allocation-<timestamp>` branch and presents the merge-request URL. Merging the MR lets intranet's CI reconcile the rows into `cca_appointments`. Files merge adds-only: existing CSV rows are never removed, so the declarative pipeline never reads a cupid export as a delete. Requires an SSH key with push access to `ShearesWeb/intranet` and a git identity configured on the machine.
- **Preallocations** add/remove records in the local store; any change invalidates the current run.
- **Purge** is the sole database write: after the export and archive it deletes both preference tables and clears the local preallocation store.

## Development

Everything runs from the repo root.

```sh
npm install                          # once; postinstall pulls in ui/ too

npm run dev                          # run the desktop app (connect in-app)
DATABASE_URL=postgres://… npm run dev  # or pre-seed the connection
npm run build                        # bundle a release build

cargo test --workspace               # engine + command tests
npm --prefix ui run test             # UI index-layer tests
```

`npm run dev` starts Vite itself through `beforeDevCommand`. Run `npm --prefix ui run dev` alone only to work on styling, since `invoke()` has no host outside the app.

The live-DB smoke test is ignored by default: `cargo test -p cupid db_load_against_live_database -- --ignored`.

## Releases

`src-tauri/tauri.conf.json` holds the version; `src-tauri/Cargo.toml` and `Cargo.lock` follow it. `ui/package.json` is never versioned, because the UI asks the shell for the running version at runtime.

Every push to `main` ships a stable release. `.github/workflows/release.yml` derives the next version from the commit subject (`feat!:` or `BREAKING CHANGE` → major, `feat:` → minor, anything else → patch), commits the bump, then builds macOS arm64, Windows x64, and Linux (AppImage + deb) and publishes them as one GitHub release with an updater manifest at `latest.json`. Feature branches and pull requests run `.github/workflows/ci.yml` instead: oxlint, vitest, `npm run build`, `cargo test --workspace`, and `cargo clippy -D warnings`.

Cupid checks for updates at launch and offers a modal when one exists; failures there stay silent so a bad network never blocks the console. The sidebar footer shows the running version and checks on demand, reporting both outcomes. Dev builds skip the check entirely.

Two limits worth knowing: the `.deb` cannot self-update (only the AppImage carries updater artifacts), and the binaries are unsigned, so the first install trips Gatekeeper on macOS and SmartScreen on Windows. Updates after that first launch install normally.

### One-time setup

Releases need a signing keypair. The private key must never enter the repo.

```sh
npm run tauri -- signer generate -w ~/.tauri/cupid.key
```

Add two repository secrets under Settings → Secrets and variables → Actions: `TAURI_SIGNING_PRIVATE_KEY` (the full contents of `~/.tauri/cupid.key`) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Paste the printed public key into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`, replacing the `REPLACE_WITH_TAURI_SIGNER_PUBLIC_KEY` placeholder. Until that placeholder is gone, published builds cannot be verified and the in-app updater rejects them.

To build a local installable copy without any of that:

```sh
npm run build -- --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
```
