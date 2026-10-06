use anyhow::Result;
use std::{
    io::{self, Write},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    collector::{Collector, SysinfoCollector},
    config::Config,
    format::{format_bytes, format_rate, format_uptime},
    model::Snapshot,
};

const WARM_UP_PAUSE: Duration = Duration::from_millis(250);
const TOP_PROCESS_LIMIT: usize = 5;
const LABEL_WIDTH: usize = 16;

pub fn run_once(json: bool) -> Result<()> {
    let mut collector = SysinfoCollector::new();
    collector.warm_up();
    thread::sleep(WARM_UP_PAUSE);
    let snapshot = collector.collect()?;
    let rendered = if json {
        let mut rendered = serde_json::to_string_pretty(&snapshot)?;
        rendered.push('\n');
        rendered
    } else {
        format!("{}\n", render_once(&snapshot))
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if let Err(error) = out.write_all(rendered.as_bytes()).and_then(|()| out.flush()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(error.into());
    }
    Ok(())
}

pub fn run_watch(config: &Config, json: bool) -> Result<()> {
    let mut collector = SysinfoCollector::new();
    collector.warm_up();
    thread::sleep(WARM_UP_PAUSE);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut previous: Option<Snapshot> = None;
    loop {
        let snapshot = collector.collect()?;
        let rates = previous.as_ref().and_then(|previous| NetRates::between(previous, &snapshot));
        if let Err(error) = emit(&mut out, &snapshot, rates.as_ref(), json) {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(error.into());
        }
        previous = Some(snapshot);
        thread::sleep(config.interval);
    }
}

fn emit(
    out: &mut impl Write,
    snapshot: &Snapshot,
    rates: Option<&NetRates>,
    json: bool,
) -> io::Result<()> {
    if json {
        serde_json::to_writer(&mut *out, snapshot).map_err(io::Error::other)?;
        writeln!(out)?;
    } else {
        writeln!(out, "{}", render_watch_line(snapshot, rates))?;
    }
    out.flush()
}

#[derive(Debug, PartialEq)]
pub struct NetRates {
    pub received_per_sec: f64,
    pub transmitted_per_sec: f64,
}

impl NetRates {
    fn between(previous: &Snapshot, current: &Snapshot) -> Option<Self> {
        let elapsed = current.timestamp.duration_since(previous.timestamp).ok()?.as_secs_f64();
        if elapsed <= 0.0 {
            return None;
        }
        let (received, transmitted) = matched_deltas(previous, current);
        Some(Self {
            received_per_sec: received as f64 / elapsed,
            transmitted_per_sec: transmitted as f64 / elapsed,
        })
    }
}

fn matched_deltas(previous: &Snapshot, current: &Snapshot) -> (u64, u64) {
    current.networks.iter().fold((0, 0), |(received, transmitted), interface| {
        let Some(previous_interface) =
            previous.networks.iter().find(|candidate| candidate.name == interface.name)
        else {
            return (received, transmitted);
        };
        (
            received.saturating_add(
                interface.received_bytes.saturating_sub(previous_interface.received_bytes),
            ),
            transmitted.saturating_add(
                interface.transmitted_bytes.saturating_sub(previous_interface.transmitted_bytes),
            ),
        )
    })
}

pub fn render_watch_line(snapshot: &Snapshot, rates: Option<&NetRates>) -> String {
    let net = match rates {
        Some(rates) => format!(
            "net ↓{} ↑{}",
            format_rate(rates.received_per_sec),
            format_rate(rates.transmitted_per_sec)
        ),
        None => "net ↓- ↑-".to_owned(),
    };
    let load = match snapshot.load_average {
        Some(value) => format!("{value:.2}"),
        None => "-".to_owned(),
    };
    let disk = snapshot
        .disks
        .iter()
        .map(|disk| disk.used_percent)
        .filter(|value| value.is_finite())
        .fold(f64::NEG_INFINITY, f64::max);
    let disk = if disk.is_finite() { format!("{disk:.1}%") } else { "-".to_owned() };
    format!(
        "{}  cpu {:.1}%  mem {:.1}%  swap {:.1}%  load {}  {}  disk {}",
        clock_utc(snapshot.timestamp),
        snapshot.cpu.total_usage,
        snapshot.memory.used_percent,
        snapshot.swap.used_percent,
        load,
        net,
        disk,
    )
}

pub fn render_once(snapshot: &Snapshot) -> String {
    let mut lines = Vec::new();
    let mut head = Vec::new();
    if let Some(hostname) = &snapshot.hostname {
        head.push(format!("host {hostname}"));
    }
    if let Some(os) = &snapshot.os {
        head.push(format!("os {os}"));
    }
    head.push(format!("uptime {}", format_uptime(snapshot.uptime_secs)));
    lines.push(head.join("  "));
    lines.push(String::new());
    lines.push(format!(
        "cpu     {:.1}%  ({} cores)",
        snapshot.cpu.total_usage,
        snapshot.cpu.per_core.len()
    ));
    lines.push(format!(
        "memory  {} / {}  ({:.1}%)",
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes),
        snapshot.memory.used_percent
    ));
    lines.push(format!(
        "swap    {} / {}  ({:.1}%)",
        format_bytes(snapshot.swap.used_bytes),
        format_bytes(snapshot.swap.total_bytes),
        snapshot.swap.used_percent
    ));
    if let Some(load) = snapshot.load_average {
        lines.push(format!("load    {load:.2}"));
    }
    for disk in &snapshot.disks {
        lines.push(format!(
            "disk    {}  {:.1}% used  ({} free of {})",
            disk.mount_point,
            disk.used_percent,
            format_bytes(disk.available_bytes),
            format_bytes(disk.total_bytes)
        ));
    }
    let mut networks: Vec<_> = snapshot
        .networks
        .iter()
        .filter(|interface| interface.received_bytes > 0 || interface.transmitted_bytes > 0)
        .collect();
    networks.sort_by(|left, right| {
        right
            .received_bytes
            .saturating_add(right.transmitted_bytes)
            .cmp(&left.received_bytes.saturating_add(left.transmitted_bytes))
            .then_with(|| left.name.cmp(&right.name))
    });
    for interface in networks {
        lines.push(format!(
            "net     {}  ↓ {}  ↑ {}",
            interface.name,
            format_bytes(interface.received_bytes),
            format_bytes(interface.transmitted_bytes)
        ));
    }
    lines.push(String::new());
    let mut processes: Vec<_> = snapshot.processes.iter().collect();
    processes.sort_by(|left, right| right.cpu_usage.total_cmp(&left.cpu_usage));
    lines.push(format!("top processes by cpu ({})", processes.len().min(TOP_PROCESS_LIMIT)));
    for process in processes.iter().take(TOP_PROCESS_LIMIT) {
        lines.push(format!(
            "  {:>7}  {}  {:>5.1}%  {}",
            process.pid,
            pad_label(&process.name),
            process.cpu_usage,
            format_bytes(process.memory_bytes)
        ));
    }
    lines.join("\n")
}

fn clock_utc(timestamp: SystemTime) -> String {
    let seconds = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let day = seconds % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3_600, (day % 3_600) / 60, day % 60)
}

fn pad_label(name: &str) -> String {
    let clipped: String = name.chars().take(LABEL_WIDTH).collect();
    format!("{clipped:<LABEL_WIDTH$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        CpuSnapshot, DiskSnapshot, MemorySnapshot, NetworkSnapshot, ProcessSnapshot, SwapSnapshot,
    };

    fn snapshot_at(seconds: u64) -> Snapshot {
        Snapshot {
            timestamp: UNIX_EPOCH + Duration::from_secs(seconds),
            hostname: Some("test-host".to_owned()),
            os: Some("test os".to_owned()),
            uptime_secs: 90_000,
            cpu: CpuSnapshot { total_usage: 12.3, per_core: vec![10.0, 14.6] },
            memory: MemorySnapshot {
                total_bytes: 16 * 1024 * 1024 * 1024,
                used_bytes: 9 * 1024 * 1024 * 1024,
                available_bytes: 7 * 1024 * 1024 * 1024,
                used_percent: 56.25,
            },
            swap: SwapSnapshot {
                total_bytes: 2 * 1024 * 1024 * 1024,
                used_bytes: 0,
                used_percent: 0.0,
            },
            load_average: Some(2.5),
            disks: vec![
                DiskSnapshot {
                    name: "disk1".to_owned(),
                    mount_point: "/".to_owned(),
                    total_bytes: 500 * 1024 * 1024 * 1024,
                    available_bytes: 200 * 1024 * 1024 * 1024,
                    used_percent: 60.0,
                },
                DiskSnapshot {
                    name: "disk2".to_owned(),
                    mount_point: "/data".to_owned(),
                    total_bytes: 100 * 1024 * 1024 * 1024,
                    available_bytes: 5 * 1024 * 1024 * 1024,
                    used_percent: 95.0,
                },
            ],
            networks: vec![NetworkSnapshot {
                name: "en0".to_owned(),
                received_bytes: 4096,
                transmitted_bytes: 2048,
            }],
            processes: vec![
                ProcessSnapshot {
                    pid: 42,
                    name: "alpha".to_owned(),
                    cpu_usage: 5.0,
                    memory_bytes: 1024,
                    ..ProcessSnapshot::default()
                },
                ProcessSnapshot {
                    pid: 7,
                    name: "beta".to_owned(),
                    cpu_usage: 40.0,
                    memory_bytes: 2048,
                    ..ProcessSnapshot::default()
                },
            ],
        }
    }

    #[test]
    fn once_summary_covers_sections() {
        let text = render_once(&snapshot_at(1));
        assert!(text.contains("host test-host"));
        assert!(text.contains("uptime 1d 1h 0m"));
        assert!(text.contains("cpu     12.3%  (2 cores)"));
        assert!(text.contains("memory  9.0 GiB / 16.0 GiB  (56.2%)"));
        assert!(text.contains("disk    /  60.0% used"));
        assert!(text.contains("net     en0  ↓ 4.0 KiB  ↑ 2.0 KiB"));
        assert!(text.contains("top processes by cpu (2)"));
        assert!(text.contains("beta"));
        assert!(text.contains("40.0%"));
    }

    #[test]
    fn once_summary_handles_missing_optional_data() {
        let mut snapshot = snapshot_at(1);
        snapshot.hostname = None;
        snapshot.os = None;
        snapshot.load_average = None;
        snapshot.disks.clear();
        snapshot.networks.clear();
        snapshot.processes.clear();
        let text = render_once(&snapshot);
        assert!(!text.contains("load"));
        assert!(!text.contains("disk"));
        assert!(!text.contains("net "));
        assert!(text.contains("top processes by cpu (0)"));
    }

    #[test]
    fn once_summary_skips_idle_interfaces_and_orders_by_activity() {
        let mut snapshot = snapshot_at(1);
        snapshot.networks.push(NetworkSnapshot {
            name: "idle0".to_owned(),
            received_bytes: 0,
            transmitted_bytes: 0,
        });
        snapshot.networks.push(NetworkSnapshot {
            name: "busy0".to_owned(),
            received_bytes: 1_000_000,
            transmitted_bytes: 1_000_000,
        });
        let text = render_once(&snapshot);
        assert!(!text.contains("idle0"));
        let busy_index = text.find("busy0").unwrap();
        let quiet_index = text.find("en0").unwrap();
        assert!(busy_index < quiet_index);
    }

    #[test]
    fn watch_line_contains_metrics() {
        let line = render_watch_line(&snapshot_at(0), None);
        assert!(line.starts_with("00:00:00  cpu 12.3%  mem 56.2%  swap 0.0%  load 2.50"));
        assert!(line.contains("net ↓- ↑-"));
        assert!(line.contains("disk 95.0%"));
    }

    #[test]
    fn watch_line_formats_rates() {
        let rates = NetRates { received_per_sec: 2048.0, transmitted_per_sec: 512.0 };
        let line = render_watch_line(&snapshot_at(0), Some(&rates));
        assert!(line.contains("net ↓2.0 KiB/s ↑512.0 B/s"));
    }

    #[test]
    fn watch_line_without_load_average() {
        let mut snapshot = snapshot_at(0);
        snapshot.load_average = None;
        let line = render_watch_line(&snapshot, None);
        assert!(line.contains("load -"));
    }

    #[test]
    fn net_rates_compute_throughput() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(102);
        current.networks[0].received_bytes += 2048;
        current.networks[0].transmitted_bytes += 1024;
        let rates = NetRates::between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 1024.0, transmitted_per_sec: 512.0 });
    }

    #[test]
    fn net_rates_none_for_stale_timestamps() {
        let previous = snapshot_at(100);
        let same_time = snapshot_at(100);
        assert!(NetRates::between(&previous, &same_time).is_none());
    }

    #[test]
    fn net_rates_floor_reset_counters_per_interface() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(101);
        current.networks[0].received_bytes = 1;
        current.networks[0].transmitted_bytes += 1024;
        let rates = NetRates::between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 0.0, transmitted_per_sec: 1024.0 });
    }

    #[test]
    fn net_rates_ignore_interfaces_without_history() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(101);
        current.networks.push(NetworkSnapshot {
            name: "veth0".to_owned(),
            received_bytes: 9_000_000,
            transmitted_bytes: 9_000_000,
        });
        let rates = NetRates::between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 0.0, transmitted_per_sec: 0.0 });
    }
}
