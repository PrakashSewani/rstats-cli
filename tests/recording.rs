use std::fs;
use tempfile::tempdir;

use rstats::{model::Snapshot, recording::Recorder};

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
