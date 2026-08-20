# cupid

Two-sided matching engine and desktop console for Sheares CCA committee allocation.
Chairs rank candidates, applicants rank positions, and cupid settles the market and exports the result.

## Installation

On **Linux and macOS**:

```sh
curl -fsSL https://raw.githubusercontent.com/ShearesWeb/cupid/main/install.sh | sh
```

The script resolves the latest release and installs the build for your machine:

| Platform | Asset | Lands in |
|---|---|---|
| macOS (Apple Silicon) | `Cupid_aarch64.app.tar.gz` | `/Applications/Cupid.app`, or `~/Applications` if the former is not writable |
| Linux (x86_64) | `Cupid_<version>_amd64.AppImage` | `~/.local/lib/cupid/`, with a `cupid` symlink in `~/.local/bin` and a desktop entry |

Intel Macs and arm64 Linux have no published build; those have to build from source.
On Windows, run the `.exe` installer from the [releases page](https://github.com/ShearesWeb/cupid/releases/latest).

Two knobs: `CUPID_VERSION=0.2.1` pins a version instead of taking the latest, and `CUPID_PREFIX` moves the Linux install prefix off `~/.local`.
Cupid checks for updates at launch and installs them itself, so the script is only needed once; re-running it replaces the existing install in place.
Uninstalling is `rm -rf /Applications/Cupid.app` on macOS, or removing `~/.local/lib/cupid`, `~/.local/bin/cupid`, `~/.local/share/applications/cupid.desktop`, and `~/.local/share/pixmaps/cupid.png` on Linux.

The binaries are unsigned, so a browser-downloaded copy trips Gatekeeper on macOS and SmartScreen on Windows — the install script sidesteps that.
The AppImage needs FUSE; install `libfuse2` if it refuses to start.
Note that the `.deb` published alongside the AppImage cannot self-update.

## Overview

A Rust engine wrapped in a Tauri desktop shell, with a React console on top.
All domain logic stays in Rust — the UI is a renderer over one snapshot the engine hands it.

| Crate / dir | Role | Stack |
|---|---|---|
| `crates/cupid-core` | The engine: domain model, the preallocation, immediate-acceptance and Gale-Shapley passes, and the `snapshot` read model that serves every UI query as a lookup. Also builds the `cupid` CLI binary. | Rust 2024, `postgres` + `rustls` |
| `src-tauri` | Desktop shell and command surface: `connect`, `sync`, `run_matching`, preallocation add/remove, `commit`, `archive`, `purge`. | Tauri 2, `tokio` |
| `ui` | The console. Builds lookup maps from the snapshot and renders; no domain logic. | React 19, TypeScript, Vite 8 |

Data loads read-only from a Supabase Postgres instance, configured in-app.
Committing a run exports per-CCA CSVs as a merge request against the intranet repo rather than writing appointments back.

## Development

Everything runs from the repo root.

```sh
npm install                            # once; the ui workspace comes along

npm run dev                            # run the desktop app (connect in-app)
DATABASE_URL=postgres://… npm run dev  # or pre-seed the connection
npm run build                          # bundle a release build

cargo test --workspace                 # engine + command tests
npm run test -w ui                     # UI index-layer tests
npm run lint -w ui                     # oxlint
cargo clippy --workspace --all-targets -- -D warnings
```

`npm run dev` starts Vite itself through `beforeDevCommand`.
Run `npm run dev -w ui` alone only to work on styling, since `invoke()` has no host outside the app.

The live-DB smoke test is ignored by default: `cargo test -p cupid db_load_against_live_database -- --ignored`.

To build a local installable copy without release signing keys:

```sh
npm run build -- --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
```
