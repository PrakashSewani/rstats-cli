# rstats

`rstats` is a cross-platform Rust terminal dashboard for live system resources and processes. It targets macOS, Linux, and Windows and keeps the terminal UI separate from a background metrics collector.

## Features

- CPU usage with per-core data
- Memory and swap utilization
- Disk usage and network counters
- Load average where the platform provides it
- Sortable/filterable process table
- Bounded CPU, memory, swap, and load history with sparklines
- Sustained threshold alerts with severity, cooldown, and recovery thresholds
- Configurable sampling interval and TOML alert rules
- Start/stop recording sessions to portable JSONL files
- Historic charts for the current recording session after it stops

## Run

```sh
cargo run --release
cargo run --release -- --interval-ms 500 --history-seconds 600
cargo run --release -- --config ~/.config/rstats/config.toml
```

The default interval is one second and the default in-memory history is five minutes. Intervals below 100 ms are clamped to 100 ms. Press `q` or Ctrl-C to exit. Recording files default to `recordings/`; override that with `--recording-dir` or `recording_dir` in TOML.

## Controls

| Key | Action |
| --- | --- |
| `1`, `2`, `3`, `4` | Dashboard, processes, alerts, recording history |
| `s` | Start/stop recording |
| `q`, Ctrl-C | Quit and finalize any active recording |
| Space | Pause/resume updates |
| `R` | Reset history |
| `c`, `m` | Sort processes by CPU or memory |
| `r` | Reverse process sort |
| `?` | Toggle help |

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

Supported alert metrics are `cpu.total`, `memory.used_percent`, `swap.used_percent`, and `load.average`. Supported operators are `greater_than`, `greater_than_or_equal`, `less_than`, and `less_than_or_equal`.

## Recording format

Press `s` to start or stop a session. Each session creates `rstats-<epoch-milliseconds>.jsonl`, with one JSON object per line: a `header`, each collected `sample`, and a final `footer` containing the stop time and sample count. The file is flushed after every sample, so it remains useful even if the process exits unexpectedly. The history screen (`4`) shows the CPU and memory charts captured during the current session.

## Development

```sh
cargo fmt --all
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Some metrics are platform-dependent. Missing values are represented internally as unavailable rather than fabricated zeroes. Process CPU percentages and process metadata follow the capabilities of the underlying operating system APIs exposed by `sysinfo`.
