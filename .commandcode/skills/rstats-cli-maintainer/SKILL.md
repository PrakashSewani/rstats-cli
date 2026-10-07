---
name: rstats-cli-maintainer
description: Maintain the rstats-cli Rust terminal system monitor and its npm native-package distribution. Use when investigating or changing collectors, Snapshot/AppState data flow, Ratatui screens or dynamic storage indicators, recording/JSONL behavior, Crossterm commands, tests, packaging, CI, versioning, or releases.
license: MIT
compatibility: Requires Rust 1.75+ with rustfmt and clippy, Node.js 18+, and the platform linker needed for native builds.
metadata:
  repository: rstats-cli
  version: "1"
---

# Maintain rstats-cli

## Start here

1. Read the repository root `AGENTS.md`.
2. Read `docs/ARCHITECTURE.md` for runtime/data-flow or UI changes.
3. Read `docs/DEVELOPMENT.md` for tests, packaging, CI, versioning, or releases.
4. Inspect the current implementation and nearby tests before editing.
5. Keep changes focused and follow existing Rust/TUI patterns.

## Task routing

| Task | First files to inspect |
| --- | --- |
| CLI/config | `src/cli.rs`, `src/config.rs`, `src/main.rs`, `tests/cli.rs`, `tests/config.rs` |
| collection/model | `src/collector/`, `src/model/`, `tests/collector.rs`, `tests/domain.rs` |
| event loop/commands | `src/app/runner.rs`, `src/app/command.rs`, `src/app/event.rs` |
| state/recording | `src/app/state.rs`, `src/recording.rs`, `src/tui/screens/deep_capture_dialog.rs`, `tests/recording*.rs` |
| dashboard/TUI | `src/tui/layout.rs`, `src/tui/screens/`, `src/tui/widgets/`, `tests/dashboard.rs` |
| alerts/history | `src/alerts/`, `src/history/`, `tests/domain.rs`, `tests/history.rs` |
| npm/release | `package.json`, `npm/`, `scripts/`, `.github/workflows/`, `docs/DEVELOPMENT.md` |
| viewer | `viewer/`, `viewer/src/lib/parse.ts`, `.github/workflows/pages.yml` |

## Invariants

- `Snapshot` is the central runtime and recording object.
- Keep `Snapshot.disks: Vec<DiskSnapshot>` dynamic for zero, one, and many drives. Storage indicators belong on the live Dashboard and must remain readable in narrow terminals.
- `AppState.recorder.is_some()` is the authoritative active-recording state. `s` starts/stops recording; the UI shows an active marker and contextual action.
- Deep capture is opt-in: recordings default to the `standard` scope. `S` toggles the scope and a deep recording must not collect until `confirm_deep_capture` runs.
- The viewer is client-side only: never upload recordings or add backend calls under `viewer/`.
- Only `KeyEventKind::Press` triggers commands. Ignore repeat and release events so toggles and navigation happen once.
- Preserve JSONL header/sample/footer records, flush each sample, and prefer parsed valid sample counts over an untrusted footer count.
- Bound live histories. Do not add unbounded event-loop state without a deliberate design.
- `Cargo.toml` owns the version. Keep all npm manifests and optional dependency versions synchronized.
- Do not commit `target/`, `recordings/`, native package binaries, `dist/`, archives, credentials, or other generated artifacts.
- Do not modify `.commandcode/taste/`; it is managed by the taste system.

## UI rules

- Use existing theme helpers and Ratatui composition patterns.
- Test empty, normal, narrow, and overflow states for UI changes.
- For dynamic disks, use safe labels from mount point/name, deterministic ordering, clamped finite percentages, capacity text, and an overflow summary rather than fixed drive slots.
- For recording/history changes, cover active, stopped, empty, interrupted, selected, and loaded states as applicable.
- Keep user-facing documentation current when behavior or controls change.

## Verification

Run the narrowest relevant test first, then the full checks:

```sh
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
npm run check-version
npm run validate-packages
npm test
```

Before finishing, run `git diff --check` and `git status --short`. Confirm no generated artifacts or credentials were added. For release changes, also validate the exact `v<Cargo.toml version>` tag rule and use `node scripts/validate-packages.mjs --require-binaries` after staging native binaries.
