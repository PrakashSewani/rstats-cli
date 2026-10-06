use std::time::{Duration, SystemTime};
use tempfile::tempdir;

use rstats::{
    app::AppState,
    config::Config,
    model::{CpuSnapshot, Snapshot},
    recording::Recorder,
};

fn config(directory: std::path::PathBuf) -> Config {
    Config {
        interval: Duration::from_millis(1000),
        history_capacity: 10,
        recording_directory: directory,
        no_color: false,
        bell: false,
        theme: Default::default(),
        alerts: Vec::new(),
    }
}

#[test]
fn state_records_snapshots_and_keeps_session_history_after_stop() {
    let directory = tempdir().unwrap();
    let mut state = AppState::new(config(directory.path().to_path_buf()));
    assert_eq!(state.recording_action(), "s start recording");
    state.toggle_recording().unwrap();
    assert_eq!(state.recording_action(), "s stop recording");
    assert!(state.recorder.is_some());
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
    assert_eq!(state.recording_action(), "s start recording");
    assert_eq!(state.saved_recordings.len(), 1);
    state.load_selected_recording().unwrap();
    assert_eq!(state.loaded_recording.as_ref().unwrap().cpu, vec![42.0]);
}

#[test]
fn changing_recording_selection_clears_loaded_session() {
    let directory = tempdir().unwrap();
    let mut first = Recorder::start(directory.path()).unwrap();
    first.record(&Snapshot::default()).unwrap();
    first.finish().unwrap();
    let mut second = Recorder::start(directory.path()).unwrap();
    second.record(&Snapshot::default()).unwrap();
    second.finish().unwrap();

    let mut state = AppState::new(config(directory.path().to_path_buf()));
    state.refresh_recordings().unwrap();
    state.load_selected_recording().unwrap();
    assert!(state.loaded_recording.is_some());
    state.move_recording_selection(1);
    assert!(state.loaded_recording.is_none());
}
