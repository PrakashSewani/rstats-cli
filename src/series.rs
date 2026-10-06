use std::time::SystemTime;

use crate::model::{NetworkSnapshot, Snapshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetRates {
    pub received_per_sec: f64,
    pub transmitted_per_sec: f64,
}

pub fn net_rates(
    previous_time: SystemTime,
    previous_interfaces: &[NetworkSnapshot],
    current: &Snapshot,
) -> Option<NetRates> {
    let elapsed = current.timestamp.duration_since(previous_time).ok()?.as_secs_f64();
    if elapsed <= 0.0 {
        return None;
    }
    let (received, transmitted) = matched_deltas(previous_interfaces, current);
    Some(NetRates {
        received_per_sec: received as f64 / elapsed,
        transmitted_per_sec: transmitted as f64 / elapsed,
    })
}

pub fn between(previous: &Snapshot, current: &Snapshot) -> Option<NetRates> {
    net_rates(previous.timestamp, &previous.networks, current)
}

pub fn max_disk_used_percent(snapshot: &Snapshot) -> Option<f64> {
    snapshot
        .disks
        .iter()
        .map(|disk| disk.used_percent)
        .filter(|value| value.is_finite())
        .fold(None, |worst: Option<f64>, value| Some(worst.map_or(value, |worst| worst.max(value))))
}

fn matched_deltas(previous: &[NetworkSnapshot], current: &Snapshot) -> (u64, u64) {
    current.networks.iter().fold((0, 0), |(received, transmitted), interface| {
        let Some(previous_interface) =
            previous.iter().find(|candidate| candidate.name == interface.name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiskSnapshot, NetworkSnapshot};
    use std::time::{Duration, UNIX_EPOCH};

    fn snapshot_at(seconds: u64) -> Snapshot {
        Snapshot {
            timestamp: UNIX_EPOCH + Duration::from_secs(seconds),
            networks: vec![NetworkSnapshot {
                name: "en0".to_owned(),
                received_bytes: 4096,
                transmitted_bytes: 2048,
            }],
            ..Snapshot::default()
        }
    }

    fn disk(used_percent: f64) -> DiskSnapshot {
        DiskSnapshot {
            name: "disk".to_owned(),
            mount_point: "/".to_owned(),
            total_bytes: 100,
            available_bytes: 50,
            used_percent,
        }
    }

    #[test]
    fn computes_throughput_from_cumulative_counters() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(102);
        current.networks[0].received_bytes += 2048;
        current.networks[0].transmitted_bytes += 1024;
        let rates = between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 1024.0, transmitted_per_sec: 512.0 });
    }

    #[test]
    fn between_returns_none_for_stale_timestamps() {
        let previous = snapshot_at(100);
        let same_time = snapshot_at(100);
        assert!(between(&previous, &same_time).is_none());
    }

    #[test]
    fn floors_reset_counters_per_interface() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(101);
        current.networks[0].received_bytes = 1;
        current.networks[0].transmitted_bytes += 1024;
        let rates = between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 0.0, transmitted_per_sec: 1024.0 });
    }

    #[test]
    fn ignores_interfaces_without_history() {
        let previous = snapshot_at(100);
        let mut current = snapshot_at(101);
        current.networks.push(NetworkSnapshot {
            name: "veth0".to_owned(),
            received_bytes: 9_000_000,
            transmitted_bytes: 9_000_000,
        });
        let rates = between(&previous, &current).unwrap();
        assert_eq!(rates, NetRates { received_per_sec: 0.0, transmitted_per_sec: 0.0 });
    }

    #[test]
    fn finds_the_most_used_disk_and_ignores_invalid_values() {
        let mut snapshot = snapshot_at(0);
        snapshot.disks = vec![disk(50.0), disk(f64::NAN), disk(93.5)];
        assert_eq!(max_disk_used_percent(&snapshot), Some(93.5));

        snapshot.disks.clear();
        assert_eq!(max_disk_used_percent(&snapshot), None);
    }
}
