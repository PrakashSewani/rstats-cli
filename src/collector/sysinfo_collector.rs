use std::time::SystemTime;

use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, PidExt, ProcessExt, System, SystemExt};

use super::{Collector, CollectorError};
use crate::model::{
    CpuSnapshot, DiskSnapshot, MemorySnapshot, NetworkSnapshot, ProcessSnapshot, Snapshot,
    SwapSnapshot,
};

pub struct SysinfoCollector {
    system: System,
}

impl SysinfoCollector {
    pub fn new() -> Self {
        Self { system: System::new_all() }
    }
}

impl Default for SysinfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for SysinfoCollector {
    fn warm_up(&mut self) {
        self.system.refresh_all();
    }

    fn collect(&mut self) -> Result<Snapshot, CollectorError> {
        self.system.refresh_all();
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();
        let cpu = CpuSnapshot {
            total_usage: finite_or_zero(f64::from(self.system.global_cpu_info().cpu_usage())),
            per_core: self
                .system
                .cpus()
                .iter()
                .map(|cpu| finite_or_zero(f64::from(cpu.cpu_usage())))
                .collect(),
        };
        let memory = MemorySnapshot {
            total_bytes: total_memory,
            used_bytes: used_memory,
            available_bytes: self.system.available_memory(),
            used_percent: percent(used_memory, total_memory),
        };
        let swap = SwapSnapshot {
            total_bytes: total_swap,
            used_bytes: used_swap,
            used_percent: percent(used_swap, total_swap),
        };
        let disks = self
            .system
            .disks()
            .iter()
            .map(|disk| DiskSnapshot {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
                used_percent: percent(
                    disk.total_space().saturating_sub(disk.available_space()),
                    disk.total_space(),
                ),
            })
            .collect();
        let networks = self
            .system
            .networks()
            .iter()
            .map(|(name, network)| NetworkSnapshot {
                name: name.clone(),
                // received()/transmitted() are per-refresh deltas; store cumulative
                // totals so downstream rate math survives any sampling cadence.
                received_bytes: network.total_received(),
                transmitted_bytes: network.total_transmitted(),
            })
            .collect();
        let processes = self
            .system
            .processes()
            .values()
            .map(|process| ProcessSnapshot {
                pid: process.pid().as_u32(),
                name: process.name().to_owned(),
                user: process.user_id().map(|id| id.to_string()).unwrap_or_default(),
                command: process.cmd().join(" "),
                cpu_usage: f64::from(process.cpu_usage()),
                memory_bytes: process.memory(),
                virtual_memory_bytes: process.virtual_memory(),
                runtime_secs: process.run_time(),
                status: process.status().to_string(),
            })
            .collect();

        Ok(Snapshot {
            timestamp: SystemTime::now(),
            hostname: self.system.host_name(),
            os: self.system.long_os_version(),
            uptime_secs: self.system.uptime(),
            cpu,
            memory,
            swap,
            load_average: load_average(&self.system),
            disks,
            networks,
            processes,
        })
    }
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 * 100.0 / total as f64
    }
}

fn load_average(system: &System) -> Option<f64> {
    let average = system.load_average().one;
    if average.is_finite() {
        Some(average)
    } else {
        None
    }
}
