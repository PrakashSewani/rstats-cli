# rstats-cli agent guide

Read this file before changing source code, tests, packaging, or workflows. It is the short operational guide; use the linked documents for deeper context.

## Project identity

`rstats-cli` is a Rust 2021 cross-platform terminal system monitor named `rstats`, distributed directly as a native binary and through an npm launcher with optional platform-specific native packages.

- Rust minimum: `1.75`
- UI: Ratatui + Crossterm
- Metrics: `sysinfo`
- Serialization/configuration: Serde, JSONL, TOML
- Package version source: `Cargo.toml`
- License: MIT

## First-read map

- [Architecture](docs/ARCHITECTURE.md) — modules, state/data flow, TUI, recording format, and known gaps.
- [Development](docs/DEVELOPMENT.md) — prerequisites, commands, tests, npm packaging, CI, and releases.
- [Maintainer skill](.commandcode/skills/rstats-cli-maintainer/SKILL.md) — task routing and agent workflow.
- `README.md` — end-user commands and controls.
- `CONTRIBUTING.md` — contributor checks and release summary.

## Source map

- `src/main.rs` — parses CLI, initializes logging, builds config, opens recordings or starts the monitor.
- `src/cli.rs` — Clap CLI definition.
- `src/config.rs` — TOML + CLI merge, defaults, bounds, and alert configuration.
- `src/model/` — `Snapshot`, metric identifiers, process view, and alert domain types.
- `src/collector/` — `Collector` trait and `SysinfoCollector` implementation.
- `src/app/runner.rs` — sampler thread, event loop, rendering dispatch, and command handling.
- `src/app/state.rs` — mutable application state, live histories, recording lifecycle, catalog selection, and alerts.
- `src/app/command.rs` — Crossterm key-to-command mapping. Only `KeyEventKind::Press` may trigger actions.
- `src/history/` — bounded `History` ring buffer used by `AppState`.
- `src/alerts/` — sustained-threshold alert evaluator.
- `src/recording.rs` — flushed JSONL recorder, loader, summaries, and catalog discovery.
- `src/tui/layout.rs` — dashboard layout, including conditional dynamic storage space.
- `src/tui/screens/` — Dashboard, Processes, Alerts, and recording History screens.
- `src/tui/widgets/` — reusable header, overview, process, alert, sparkline, and Storage widgets.
- `tests/` — integration tests for CLI, config, collector, domain, history, recording, state, and dashboard rendering.
- `npm/` — launcher package and five platform package manifests.
- `scripts/` — version sync, package validation, launcher tests, and release staging.
- `.github/workflows/` — CI and tag-driven release automation.

## Runtime flow

```text
SysinfoCollector
  -> sampler thread
  -> bounded crossbeam channel
  -> app::runner::run_loop
  -> AppState::apply_snapshot
  -> live histories / active Recorder / alerts
  -> Ratatui screen renderer
```

`Snapshot` is the central object. It contains timestamp, CPU, memory, swap, load average, dynamic disks, networks, and processes. `Snapshot.disks` is a `Vec<DiskSnapshot>` and must remain dynamic: support zero, one, and many drives without fixed drive slots.

## Non-negotiable invariants

- Treat `Cargo.toml` as the authoritative version source and keep npm manifests synchronized.
- Do not commit `target/`, runtime `recordings/`, native package binaries, `dist/`, release archives, or credentials.
- Keep recording files JSONL and flush samples as they are written; interrupted files with valid sample lines should remain loadable.
- Preserve header/sample/footer recording records and do not trust a footer count over parsed sample records when valid samples exist.
- Only key presses trigger commands. Repeat/release events must not toggle recording or move selections.
- Recordings use `s`: the active state is `AppState.recorder.is_some()`. The UI should show an active marker and contextual start/stop action.
- Disk UI must use the collected `Snapshot.disks` dynamically, display per-drive usage percentage and capacity, and remain readable on narrow terminals.
- Bound live histories and avoid adding unbounded state to the event loop without a deliberate design.
- Use existing project patterns and styles. Do not add comments unless the logic truly needs one.
- Validate external/system boundaries, but do not add speculative fallbacks or compatibility hacks.

## Common commands

```sh
cargo run -- --monitor
cargo run --release -- --monitor
cargo test --all-targets
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features -- -D warnings
npm run check-version
npm run validate-packages
npm test
```

On Windows, Rust builds use the MSVC toolchain and require the Visual Studio C++ Build Tools linker. The default recording directory is `recordings` relative to the process working directory; use `--recording-dir` when a deterministic location matters.

## Before changing code

1. Read the relevant section of `docs/ARCHITECTURE.md` and `docs/DEVELOPMENT.md`.
2. Inspect the current implementation and nearby tests before proposing a change.
3. Keep UI changes dynamic and test empty, normal, narrow, and overflow states.
4. Keep recording/key-event changes covered by regression tests.
5. Run the narrowest relevant test first, then the complete verification sequence before finishing.
6. Review `git diff --check` and `git status --short` for unintended generated files.

If behavior is intentionally changed, update the appropriate user-facing or maintainer documentation in the same change.
