# Architecture

`rstats-cli` is a native Rust terminal monitor. It collects system metrics in a background sampler, applies each `Snapshot` to mutable application state, renders the selected Ratatui screen, and optionally writes every accepted snapshot to a JSONL recording.

## Repository shape

```text
src/main.rs                 startup and CLI dispatch
src/cli.rs                  Clap arguments
src/config.rs               TOML + CLI configuration
src/export.rs               recording export (CSV/JSON) and reports
src/format.rs               shared byte, rate, and uptime formatting
src/headless.rs             --once and --watch headless output
src/model/                  serializable domain types
src/collector/              system metric collection
src/app/                    event loop, commands, and mutable state
src/history/                bounded live history ring buffer
src/alerts/                 sustained alert evaluation
src/recording.rs             JSONL recording (capture scopes) and catalog loading
src/series.rs                derived series helpers (network rates, disk usage)
src/tui/                    terminal lifecycle, layout, screens, widgets
tests/                      Rust integration coverage
viewer/                     static web viewer for recordings (Vite + TypeScript)
npm/                        launcher and native package manifests
scripts/                    version, package, and release helpers
.github/workflows/          CI and tagged release automation
```

`Cargo.toml` is the authoritative Rust package metadata and version source. The npm package version must match it.

## Startup and configuration flow

`src/main.rs` performs this sequence:

1. Parse `Cli` from `src/cli.rs` with Clap.
2. Initialize tracing.
3. Build `Config` through `Config::from_cli` in `src/config.rs`.
4. If `--open-recordings` is set, create/open the configured directory through `src/open_recordings.rs`.
5. If `--export` or `--report` is set, emit recording output through `src/export.rs`.
6. If `--once` or `--watch` is set, emit headless output through `src/headless.rs`.
7. Otherwise call `App::run(config)` in `src/app/runner.rs`.

Plain `rstats` and `rstats --monitor` both start the interactive monitor. `--monitor` and `--open-recordings` conflict. `--once` and `--watch` are headless output modes: they conflict with both TUI modes and with each other, and `--json` requires one of them. `--export` and `--report` are recording-analysis modes: they conflict with the TUI and headless modes and with each other, and `--format`/`--output` require `--export`.

`--once` collects a single snapshot after a warm-up pause and prints it (human-readable summary, or pretty JSON with `--json`). `--watch` repeats collection on the configured interval and writes one plain-text line per sample (compact JSON Lines with `--json`), flushing every line; it exits cleanly on a closed stdout pipe. Sample clocks in watch lines are UTC.

Configuration precedence is CLI value, TOML value, then built-in default. The recording directory resolves as:

1. `--recording-dir`
2. `recording_dir` in the TOML file
3. `recordings` relative to the current working directory

The default interval is 1,000 ms, with a 100 ms minimum. Requested live history is bounded to at most 3,600 samples. Alert configuration supports CPU, memory, swap, and load average metrics.

## Central data model

`src/model/snapshot.rs` defines the serializable `Snapshot` passed through the whole runtime:

- `timestamp: SystemTime`, serialized as Unix epoch milliseconds
- optional hostname and OS
- uptime
- `CpuSnapshot`, including total and per-core usage
- `MemorySnapshot`, including total, used, available bytes, and used percentage
- `SwapSnapshot`
- optional load average
- `Vec<DiskSnapshot>`
- cumulative network interface byte counters (`received_bytes`/`transmitted_bytes` are totals since interface creation, not per-refresh deltas)
- process snapshots

`Snapshot.disks` must remain a dynamic vector. It safely represents zero, one, or many OS-reported filesystems. `DiskSnapshot` includes name, mount point, total bytes, available bytes, and used percentage.

The collector and recording format already retain full disk snapshots. `RecordedSession` derives a worst-disk usage series (`disk_max`) and per-interval network receive/transmit rates from recorded samples, so disk and network charts work both live and from recordings.

## Collector and runtime data flow

`src/collector/mod.rs` defines the `Collector` trait with `warm_up` and `collect`. `SysinfoCollector` in `src/collector/sysinfo_collector.rs` owns a `sysinfo::System` and calls `refresh_all()` before building each snapshot.

The runtime path is:

```text
SysinfoCollector::collect()
  -> Snapshot
  -> AppEvent::Snapshot(Box<Snapshot>)
  -> bounded crossbeam channel
  -> App::run_loop()
  -> AppState::apply_snapshot()
  -> histories, Recorder, alerts
  -> screen/widget renderers
```

`start_sampler` launches the collector on a thread. The channel capacity is four. The event loop drains queued events before each draw, then polls one terminal event with Crossterm.

Collection normalization includes:

- non-finite CPU values become `0.0`;
- zero-denominator percentages become `0.0`;
- non-finite load average becomes `None`;
- disk used bytes use saturating subtraction;
- disk percentages are expected to be finite and within `0..=100`.

## AppState responsibilities

`src/app/state.rs` owns mutable runtime state:

- current `Snapshot`;
- live bounded `History` values for CPU, memory, swap, load average, network receive/transmit rates, and worst-disk usage;
- the previous network sample (`net_previous`) used to compute the next interval's rates;
- current recording histories, bounded to 86,400 samples;
- alert events and evaluator state;
- screen, pause, help, and error state;
- capture scope and deep-capture confirmation state (dialog and session flag);
- active `Recorder`;
- last recording summary;
- saved recording catalog, selection index, and explicitly loaded session.

`apply_snapshot` ignores the snapshot entirely when paused. Otherwise it updates the current snapshot and live histories, writes to the active recorder if present, updates recording histories after a successful write, evaluates alerts, and clears a prior collector error.

Recording state is authoritative in `AppState.recorder`:

- `Some(Recorder)` means recording is active;
- `None` means recording is inactive.

`toggle_recording` calls `start_recording` or `stop_recording`. Starting clears current recording histories, last summary, and loaded recording. Stopping finalizes the JSONL footer, stores the summary, and refreshes the catalog. Normal quit also finalizes an active recording.

`toggle_record_scope` switches between the standard and deep capture scopes while idle. Deep capture must not collect without consent: enabling it arms `deep_capture_dialog` with `DeepCaptureIntent::Enable`, and starting a deep recording arms it with `DeepCaptureIntent::Start` until `confirm_deep_capture` records the session confirmation; `cancel_deep_capture` dismisses the dialog without collecting.

`move_recording_selection` wraps around the catalog and clears stale `loaded_recording` when the selected entry changes. `load_selected_recording` reads the selected JSONL file explicitly.

## Keyboard commands

`src/app/command.rs` maps Crossterm `KeyEvent`s to commands. Only `KeyEventKind::Press` is accepted. Repeat and release events return `Command::None`; this is required so one physical key action cannot toggle recording twice or move selection twice.

Current controls:

| Key | Action |
| --- | --- |
| `1`–`4` | Dashboard, Processes, Alerts, History |
| `q`, Ctrl-C | Quit |
| Space | Pause/resume application updates |
| `s` | Start/stop recording |
| `S` | Toggle deep capture (confirmation required before collecting) |
| Up/Down, `k`/`j` | Select a saved recording on History |
| Enter | Load the selected recording |
| `R` | Reset live history |
| `?` | Toggle help |
| `c`, `m` | Sort processes by CPU or memory |
| `r` | Reverse process sort |
| `t` | Cycle color theme (dark, light, mono) |
| `/` | Maps to a filter command, but the input flow is incomplete |

## TUI structure

`src/tui/terminal.rs` enters raw mode, alternate-screen mode, and mouse capture, then restores them on normal return. The `run_loop` function in `src/app/runner.rs` dispatches to one of four screens:

- `src/tui/screens/dashboard.rs` — live gauges, dynamic Storage panel, a 2x2 CPU/memory/network/disk sparkline grid, footer, help overlay;
- `src/tui/screens/process_screen.rs` — process table and current filter label;
- `src/tui/screens/alerts_screen.rs` — active/pending alert table;
- `src/tui/screens/history_screen.rs` — saved recording catalog, status, and CPU/memory/network/disk charts;
- `src/tui/screens/deep_capture_dialog.rs` — centered deep-capture warning and confirmation overlay, rendered above any screen.

Reusable widgets are under `src/tui/widgets/`. Styles are centralized in `src/tui/theme.rs` as title, muted, gauge, warning, and critical styles resolved against the active palette. The palette is selected by `theme` in the config file or `--theme` on the command line and can be cycled at runtime with `t`; built-in palettes are dark, light, and mono.

### Dashboard

`src/tui/layout.rs::dashboard_chunks` returns a named dashboard layout with header, overview, optional storage, history, and footer rectangles. Without disks, the original four-region layout is retained. With disks, a Storage region is inserted below the four fixed CPU/Memory/Swap/Alerts gauges.

`src/tui/widgets/storage.rs::render_storage`:

1. sorts disks by mount point and then name;
2. chooses a mount point, name, or `disk` fallback label;
3. displays a usage bar, percentage, and used/total capacity;
4. uses green below 80%, yellow from 80% to below 90%, and red at 90% or above;
5. uses two columns on wide terminals and one compact column on narrow terminals;
6. truncates labels to preserve numeric details;
7. displays `+N more disks` when the available region cannot show every disk.

The widget handles an empty list with `No disks detected`, though the Dashboard omits the panel when the live snapshot has no disks.

### History visualizer

`src/tui/screens/history_screen.rs` shows the active recording histories while recording. When idle, it prefers an explicitly loaded session and otherwise previews the selected catalog entry. Empty recordings show a clear no-samples state. Charts render as a 2x2 grid: CPU, memory, network throughput (receive plus transmit rates), and worst-disk usage. `RecordedSession` retains CPU, memory, swap, and load-average series plus network receive/transmit rates, worst-disk usage, and per-sample timestamps derived at load time.

### History storage

`src/history/ring_buffer.rs::History` is the bounded `VecDeque<f64>` implementation used by `AppState`. `src/model/metric.rs` contains a separate `MetricHistory` implementation that is not wired into the application; check usage before modifying either type.

## Recording format

`src/recording.rs` writes one JSON object per line:

```json
{"type":"header","version":1,"scope":"standard","started_at_ms":0}
{"type":"sample","snapshot":{"timestamp":0}}
{"type":"footer","stopped_at_ms":1000,"samples":1}
```

`Recorder::start` takes a `CaptureScope` (`standard` by default, `deep` opt-in), records it in the header, creates a unique `rstats-<epoch-milliseconds>.jsonl` path, writes and flushes the header, and initializes the count. Standard-scope samples carry every metric except the process table; deep-scope samples contain the complete `Snapshot`, including processes, disks, and networks. Legacy recordings without a `scope` field load with `scope: None`. `Recorder::record` writes and flushes every sample. `Recorder::finish` writes and flushes the footer.

`RecordedSession::load` reads valid lines and extracts session metadata plus CPU, memory, swap, and load-average vectors, network receive/transmit rates, worst-disk usage, and per-sample timestamps. If valid sample records exist, their parsed count is authoritative; the footer count is only used when no valid samples were parsed. Invalid JSONL lines are skipped so interrupted files with valid lines remain loadable. `list_recordings` discovers `.jsonl` files and silently skips files that cannot be loaded.

Do not remove the header/sample/footer structure, per-sample flush behavior, or parsed-sample-count precedence without updating compatibility tests and documentation.

## Export and reports

`src/export.rs` reuses `RecordedSession::load`. `--export` writes one row per sample to stdout or `--output FILE`: CSV by default, JSON rows with `--format json`. Columns are `timestamp_ms`, `cpu_total_pct`, `memory_used_pct`, `swap_used_pct`, `load_average`, `net_received_bps`, `net_transmitted_bps`, and `disk_max_used_pct`. The `load_average` cell is written only when the loaded load-average vector is aligned with the sample count. `--report` prints per-metric averages and peaks with peak time offsets from the first sample. Both modes reject recordings with no samples.

## Web viewer

`viewer/` is a self-contained static site (Vite + TypeScript + uPlot) that parses recordings and exports entirely client-side — no backend, no uploads. The landing page (`/`) opens a file or demo and routes to `/view`, which renders the aligned timeline charts and, when the recording is deep scope, the process explorer. The parser (`viewer/src/lib/parse.ts`) auto-detects JSONL recordings, export JSON rows, and single snapshots, and tolerates missing `processes` (standard scope) and a missing `scope` header field (legacy files). CI typechecks and builds it in the `viewer` job; `.github/workflows/pages.yml` deploys `viewer/dist` to GitHub Pages.

## Alerts

`src/alerts/rules.rs` maps configured metrics to `Snapshot` values. `src/alerts/evaluator.rs` tracks rules through inactive, pending, firing, and cooldown states keyed by rule name. Alert duration uses sample timestamps. The UI displays active/pending alert events; inactive and cooldown history is not rendered.

## npm distribution

The npm layer contains:

- `npm/rstats-cli/` — main launcher package;
- five optional native packages for Darwin x64/arm64, Linux x64/arm64, and Windows x64.

`npm/rstats-cli/bin/rstats.js` maps `process.platform` and `process.arch`, resolves the matching optional package executable, and spawns it with inherited stdio. It does not download binaries or construct shell commands.

Native package binaries are release artifacts. They are staged by `scripts/stage-native-package.mjs` and must not be committed in normal source changes.

## Cross-platform caveats and known gaps

- `sysinfo` reports OS-visible filesystems, not necessarily physical drives; partitions, virtual filesystems, removable media, and network mounts can appear.
- Mount/name strings can be empty or long; storage rendering must retain safe fallbacks and truncation.
- Windows builds require the MSVC linker and Visual Studio C++ Build Tools.
- The default recording path is relative to the process working directory, so use `--recording-dir` for deterministic automation.
- `Config.no_color` and `Config.bell` are present but currently have no apparent runtime effect.
- `/` maps to `Command::Filter`, but `handle_key` does not provide an input editor path.
- Swap and load-average recording vectors are loaded but not visualized on the History screen.
- Deep-scope recordings carry the full process table and grow roughly 100x faster than standard ones (~15 MB/min at 1 s intervals), and `RecordedSession::load` reads the full file into memory.
- Live histories are bounded, but loaded chart vectors are not explicitly bounded.
- The sampler thread is signaled but not joined, and terminal restoration is not panic-safe.
- The alert count is displayed through a percentage-like gauge and clamps above 100.

When changing one of these areas, decide whether the behavior is intentional, add focused regression coverage, and update this document if the architecture changes.
