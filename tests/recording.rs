use std::fs;
use tempfile::tempdir;

use rstats::{
    model::{CpuSnapshot, MemorySnapshot, Snapshot},
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
