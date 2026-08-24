# Development

## Prerequisites

- Rust stable with minimum supported Rust version `1.75`.
- `rustfmt` and `clippy` components.
- Node.js `18+`; CI uses Node.js 20.
- Windows: MSVC Rust toolchain and Visual Studio C++ Build Tools/linker.
- Linux `--open-recordings`: `xdg-open` available in the desktop environment.

The project uses Rust 2021, Ratatui, Crossterm, `sysinfo`, Serde, and a committed `Cargo.lock`. Rust commands should normally use the lockfile for CI parity.

## Run locally

Start the monitor in debug mode:

```sh
cargo run -- --monitor
```

Start a release build:

```sh
cargo run --release -- --monitor
```

Plain `rstats` and `rstats --monitor` are equivalent. Useful commands:

```sh
rstats --help
rstats --version
rstats --open-recordings
rstats --open-recordings --recording-dir ./my-recordings
```

`--open-recordings` creates and opens the configured recording directory in the host file manager. It is an interactive desktop operation, not a headless test command. The modes `--monitor` and `--open-recordings` conflict.

The recording directory is resolved in this order:

1. `--recording-dir DIRECTORY`;
2. `recording_dir` in the TOML configuration file;
3. `recordings` relative to the current working directory.

Use an explicit directory in scripts and tests when the working directory could vary:

```sh
cargo run -- --monitor --recording-dir ./tmp-recordings
```

## Standard checks

Format source files:

```sh
cargo fmt --all
```

Run the check-only form before committing:

```sh
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
npm run check-version
npm run validate-packages
npm test
```

For a faster iteration loop, run the narrowest relevant test first, then run the complete sequence. UI changes should test empty, normal, narrow, and overflow states. Recording and key-event changes need regression tests.

`npm test` is Node’s built-in test runner. The workspace has no development npm dependencies; scripts use Node built-ins.

## Tests

Rust unit and integration tests cover:

- CLI parsing: `tests/cli.rs`;
- config precedence/bounds: `tests/config.rs`;
- system collection invariants: `tests/collector.rs`;
- process and alert domain behavior: `tests/domain.rs`;
- bounded history: `tests/history.rs`;
- JSONL recording/catalog behavior: `tests/recording.rs`;
- AppState recording lifecycle: `tests/recording_state.rs`;
- dashboard/storage rendering and layout: `tests/dashboard.rs`.

When adding a rendering feature, prefer Ratatui `TestBackend` buffer assertions and synthetic `Snapshot` data. Never assert a fixed disk count in collector tests because host hardware varies. When testing recordings, preserve header/sample/footer JSONL semantics and verify interrupted files with valid sample lines remain loadable.

## Project conventions

- `Snapshot` is the central data object and `Snapshot.disks` is always dynamic.
- `AppState.recorder.is_some()` is the authoritative active-recording state.
- Only `KeyEventKind::Press` may trigger commands; repeat/release events must be ignored.
- `s` starts/stops recording. The UI should show an active marker, sample count, and contextual start/stop hint.
- Disk indicators belong on the live Dashboard and must support zero, one, and many drives without fixed drive slots.
- Keep live histories bounded and use existing theme/layout helpers.
- Do not add comments unless the logic genuinely requires one.
- Validate external boundaries, but do not add speculative fallbacks or compatibility shims.

## npm packages

The root `package.json` is a private development workspace. `npm/rstats-cli/` is the publishable launcher. Five optional native packages contain platform-specific metadata and receive binaries during release staging:

| Rust target | npm package | Runtime |
| --- | --- | --- |
| `x86_64-apple-darwin` | `rstats-cli-darwin-x64` | macOS x64 |
| `aarch64-apple-darwin` | `rstats-cli-darwin-arm64` | macOS arm64 |
| `x86_64-unknown-linux-gnu` | `rstats-cli-linux-x64` | Linux x64 |
| `aarch64-unknown-linux-gnu` | `rstats-cli-linux-arm64` | Linux arm64 |
| `x86_64-pc-windows-msvc` | `rstats-cli-win32-x64` | Windows x64 |

The launcher resolves the matching optional package and spawns its packaged executable. It must not download binaries during installation or execution, and it must not build shell command strings.

## Version synchronization

`Cargo.toml` is authoritative. After changing its package version, run:

```sh
npm run sync-version
npm run check-version
```

The sync script updates the root package, the main launcher, all five native manifests, and optional dependency versions. `npm run check-version` must pass before a release or pull request.

## Package validation and staging

Validate source package metadata without requiring native binaries:

```sh
npm run validate-packages
```

The validator checks all six package manifests, names, versions, optional dependencies, exact `files` allowlists, binary mappings, OS/CPU restrictions, and required source files.

Release staging maps target binaries with:

```sh
node scripts/stage-native-package.mjs <rust-target> <binary-path>
```

Then require all native binaries:

```sh
node scripts/validate-packages.mjs --require-binaries
```

The release job dry-runs `npm pack --dry-run --json` for every package and creates `dist/SHA256SUMS`.

## CI

`.github/workflows/ci.yml` runs on Ubuntu, macOS, and Windows for pushes and pull requests. It uses Node 20 and stable Rust with `rustfmt` and `clippy`, then runs the locked Rust checks and npm checks.

CI-equivalent commands are:

```sh
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
node scripts/sync-version.mjs --check
node scripts/validate-packages.mjs
npm test
```

## Releases

`.github/workflows/release.yml` starts for pushed tags matching `v*`, but validation requires the exact tag `v<Cargo.toml version>`.

Release sequence:

1. Update `Cargo.toml` version.
2. Run `npm run sync-version`.
3. Run all Rust and npm checks.
4. Push the matching tag, for example `v0.1.0`.
5. The validate job checks the tag/version and all quality checks.
6. The build matrix produces five native targets; Linux arm64 uses `cross` 0.2.5.
7. The package job stages binaries, validates with `--require-binaries`, dry-runs packages, and creates checksums.
8. The publish job publishes platform packages first, then `rstats-cli`, using npm trusted publishing/OIDC and provenance.
9. The GitHub release job attaches native binaries and checksums.

Do not push a tag until `npm run check-version` passes and the tag exactly matches the Cargo version.

## Generated and ignored artifacts

Do not commit:

- `target/` Rust build output;
- runtime data under `recordings/`;
- native binaries under `npm/*/bin/`;
- `dist/`, `release/`, npm staging output, archives, and `.tgz` files;
- npm debug logs, caches, credentials, or temporary files.

The `.gitignore` contains the repository’s authoritative ignore patterns. Runtime recordings may be present locally for testing, but they are generated data and should not be added to source control.

## Common pitfalls

- Changing Cargo’s version without syncing npm manifests causes CI/release failure.
- The default recording directory depends on the process working directory.
- Optional native npm dependencies may be omitted by a package manager; the launcher does not download a fallback binary.
- Windows supports only x64 in the npm distribution and needs the MSVC linker for local builds.
- Linux arm64 is cross-built with `cross`, not ordinary host Cargo.
- `--open-recordings` requires a desktop file opener.
- Native package file allowlists are intentionally exact; changing them requires updating validation logic and release expectations.
- The TUI’s `/` filter command is not a complete input editor yet; `no_color`, `bell`, network charts, historical disk charts, and sampler thread joining are also incomplete areas documented in `docs/ARCHITECTURE.md`.

## Documentation maintenance

When changing module boundaries, runtime flow, recording schema, keyboard invariants, dashboard layout, supported targets, or release commands, update `AGENTS.md` and the relevant section of `docs/ARCHITECTURE.md` or `docs/DEVELOPMENT.md` in the same change. Keep `README.md` focused on end-user behavior and `CONTRIBUTING.md` focused on contributor workflow.
