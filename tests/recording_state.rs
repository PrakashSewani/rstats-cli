use std::time::{Duration, SystemTime};
use tempfile::tempdir;

use rstats::{
    app::AppState,
    config::Config,
    model::{CpuSnapshot, Snapshot},
};

fn config(directory: std::path::PathBuf) -> Config {
    Config {
        interval: Duration::from_millis(1000),
        history_capacity: 10,
        recording_directory: directory,
        no_color: false,
        bell: false,
        alerts: Vec::new(),
    }
}

#[test]
fn state_records_snapshots_and_keeps_session_history_after_stop() {
    let directory = tempdir().unwrap();
    let mut state = AppState::new(config(directory.path().to_path_buf()));
    state.toggle_recording().unwrap();
    state.apply_snapshot(Snapshot {
        timestamp: SystemTime::UNIX_EPOCH,
        cpu: CpuSnapshot { total_usage: 42.0, per_core: Vec::new() },
        ..Snapshot::default()
    });
    assert_eq!(state.recording_samples(), 1);
    assert_eq!(state.recording_histories[&rstats::model::MetricKind::Cpu].len(), 1);
    let summary = state.stop_recording().unwrap().unwrap();
    assert_eq!(summary.samples, 1);
    assert!(summary.path.exists());
    assert!(state.recorder.is_none());
}
