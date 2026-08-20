use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,
    pub hostname: Option<String>,
    pub os: Option<String>,
    pub uptime_secs: u64,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub swap: SwapSnapshot,
    pub load_average: Option<f64>,
    pub disks: Vec<DiskSnapshot>,
    pub networks: Vec<NetworkSnapshot>,
    pub processes: Vec<ProcessSnapshot>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::UNIX_EPOCH,
            hostname: None,
            os: None,
            uptime_secs: 0,
            cpu: CpuSnapshot::default(),
            memory: MemorySnapshot::default(),
            swap: SwapSnapshot::default(),
            load_average: None,
            disks: Vec::new(),
            networks: Vec::new(),
            processes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CpuSnapshot {
    pub total_usage: f64,
    pub per_core: Vec<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SwapSnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DiskSnapshot {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NetworkSnapshot {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = value.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        serializer.serialize_u64(millis)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        UNIX_EPOCH
            .checked_add(Duration::from_millis(millis))
            .ok_or_else(|| serde::de::Error::custom("timestamp is out of range"))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub command: String,
    pub cpu_usage: f64,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub runtime_secs: u64,
    pub status: String,
}
