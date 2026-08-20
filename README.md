# rstats

`rstats` is a cross-platform Rust terminal dashboard for live system resources, processes, alerts, and recording history. It supports macOS, Linux, and Windows through a native executable and is distributed for end users through the `rstats-cli` npm package.

## Install with npm

```sh
npm install --global rstats-cli
rstats --monitor
```

Or run it without a global install:

```sh
npx rstats-cli --monitor
```

The npm package installs only the optional native package matching the current operating system and CPU architecture. It does not download binaries during installation or execution.

Supported npm targets:

| Operating system | Architectures |
| --- | --- |
| macOS | x64, arm64 |
| Linux | x64, arm64 |
| Windows | x64 |

## CLI commands

Plain `rstats` and `rstats --monitor` both launch the interactive terminal monitor. `--monitor` is the explicit form for scripts and documentation.

```sh
rstats
rstats --monitor
rstats --help
rstats --version
rstats --open-recordings
rstats --open-recordings --recording-dir ./my-recordings
```

`--open-recordings` creates and opens the configured recordings directory in the platform file manager. macOS uses `open`, Windows uses `explorer`, and Linux uses `xdg-open`; Linux users need `xdg-open` installed.

The recording directory resolves in this order:

1. `--recording-dir DIRECTORY`
2. `recording_dir` in the TOML configuration file
3. `recordings` in the current working directory

The CLI modes are mutually exclusive: `--monitor --open-recordings` is rejected.

## Monitor controls

| Key | Action |
| --- | --- |
| `1`, `2`, `3`, `4` | Dashboard, processes, alerts, recording history |
| Up/Down | Select a saved recording on screen `4` |
| Enter | Load the selected recording into the charts |
| `s` | Start/stop recording |
| `q`, Ctrl-C | Quit and finalize any active recording |
| Space | Pause/resume updates |
| `R` | Reset live history |
| `c`, `m` | Sort processes by CPU or memory |
| `r` | Reverse process sort |
| `?` | Toggle help |

## Features

- CPU usage with per-core data
- Memory and swap utilization
- Disk usage and network counters
- Load average where the platform provides it
- Sortable and filterable process table
- Bounded CPU, memory, swap, and load history with sparklines
- Sustained threshold alerts with severity, cooldown, and recovery thresholds
- Start/stop recording sessions to portable JSONL files
- Built-in recording catalog and CPU/memory visualizer
- Interrupted recordings remain loadable when valid samples were flushed

## Configuration

CLI arguments override values from the TOML file:

```toml
interval_ms = 1000
history_seconds = 300
recording_dir = "recordings"
no_color = false
bell = false

[[alerts]]
name = "high_cpu"
metric = "cpu.total"
operator = "greater_than"
threshold = 90.0
duration_seconds = 30
severity = "warning"
cooldown_seconds = 300
recovery_threshold = 85.0
```

Use a configuration file with:

```sh
rstats --monitor --config ~/.config/rstats/config.toml
```

Supported alert metrics are `cpu.total`, `memory.used_percent`, `swap.used_percent`, and `load.average`. Supported operators are `greater_than`, `greater_than_or_equal`, `less_than`, and `less_than_or_equal`.

## Recording format

Press `s` to start or stop a session. Each session creates `rstats-<epoch-milliseconds>.jsonl` with one JSON object per line: a header, each collected sample, and a footer containing the stop time and sample count. The file is flushed after every sample.

Open screen `4` to browse saved sessions. Use Up/Down to select a file and Enter to load its CPU and memory charts directly inside the TUI. You do not need to open the JSONL file manually.

## Development

Run the Rust application from source:

```sh
cargo run --release -- --monitor
cargo run --release -- --open-recordings --recording-dir ./tmp-recordings
```

Run the quality checks:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
npm run check-version
npm run validate-packages
npm test
```

## Publishing

`Cargo.toml` is the authoritative version source. Synchronize npm manifests before a release:

```sh
npm run sync-version
npm run check-version
npm run validate-packages
```

Create a version tag such as `v0.1.0` after the Rust and npm versions match. The release workflow builds native binaries for all supported targets, stages the five platform packages, validates `npm pack` contents, publishes platform packages first, and publishes `rstats-cli` last. Native binaries and runtime recordings are generated artifacts and are not committed to the repository.

## License

MIT
