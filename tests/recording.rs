use std::fs;
use tempfile::tempdir;

use rstats::{
    model::{CpuSnapshot, DiskSnapshot, MemorySnapshot, NetworkSnapshot, Snapshot},
    recording::{list_recordings, RecordedSession, Recorder},
};

#[test]
fn recording_file_is_streamable_jsonl() {
    let directory = tempdir().unwrap();
    let mut recorder = Recorder::start(directory.path()).unwrap();
    recorder.record(&Snapshot::default()).unwrap();
    let summary = recorder.finish().unwrap();
    let lines = fs::read_to_string(summary.path).unwrap();
    assert_eq!(lines.lines().count(), 3);
    assert!(lines.lines().all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));
}

#[test]
fn catalog_loads_recorded_chart_values() {
    let directory = tempdir().unwrap();
    let mut recorder = Recorder::start(directory.path()).unwrap();
    recorder
        .record(&Snapshot {
            cpu: CpuSnapshot { total_usage: 42.0, per_core: Vec::new() },
            memory: MemorySnapshot { used_percent: 64.0, ..MemorySnapshot::default() },
            ..Snapshot::default()
        })
        .unwrap();
    let summary = recorder.finish().unwrap();
    let sessions = list_recordings(directory.path()).unwrap();
    assert_eq!(sessions.len(), 1);
    let loaded = RecordedSession::load(&summary.path).unwrap();
    assert_eq!(loaded.samples, 1);
    assert_eq!(loaded.cpu, vec![42.0]);
    assert_eq!(loaded.memory, vec![64.0]);
}

#[test]
fn catalog_preserves_disk_data_from_recorded_samples() {
    let directory = tempdir().unwrap();
    let mut recorder = Recorder::start(directory.path()).unwrap();
    let disks = (0..6)
        .map(|index| DiskSnapshot {
            name: format!("disk-{index}"),
            mount_point: format!("/mnt/{index}"),
            total_bytes: 1_000,
            available_bytes: 400,
            used_percent: 60.0 + index as f64,
        })
        .collect::<Vec<_>>();
    let summary = {
        recorder.record(&Snapshot { disks, ..Snapshot::default() }).unwrap();
        recorder.finish().unwrap()
    };
    let loaded = RecordedSession::load(&summary.path).unwrap();
    let contents = fs::read_to_string(summary.path).unwrap();
    assert_eq!(contents.lines().count(), 3);
    assert_eq!(loaded.samples, 1);
    let sample: serde_json::Value = serde_json::from_str(contents.lines().nth(1).unwrap()).unwrap();
    assert_eq!(sample["snapshot"]["disks"].as_array().unwrap().len(), 6);
    assert_eq!(sample["snapshot"]["disks"][0]["mount_point"], "/mnt/0");
}

#[test]
fn catalog_extracts_network_rates_and_disk_series() {
    let directory = tempdir().unwrap();
    let mut recorder = Recorder::start(directory.path()).unwrap();
    let sample = |seconds: u64, received: u64, transmitted: u64, used: f64| Snapshot {
        timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds),
        networks: vec![NetworkSnapshot {
            name: "en0".to_owned(),
            received_bytes: received,
            transmitted_bytes: transmitted,
        }],
        disks: vec![DiskSnapshot {
            name: "disk".to_owned(),
            mount_point: "/".to_owned(),
            total_bytes: 100,
            available_bytes: 40,
            used_percent: used,
        }],
        ..Snapshot::default()
    };
    recorder.record(&sample(10, 1_000, 2_000, 60.0)).unwrap();
    recorder.record(&sample(12, 5_000, 4_000, 80.0)).unwrap();
    let summary = recorder.finish().unwrap();

    let loaded = RecordedSession::load(&summary.path).unwrap();
    assert_eq!(loaded.net_received, vec![0.0, 2_000.0]);
    assert_eq!(loaded.net_transmitted, vec![0.0, 1_000.0]);
    assert_eq!(loaded.disk_max, vec![60.0, 80.0]);

    let listed = list_recordings(directory.path()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].net_received, loaded.net_received);
}
