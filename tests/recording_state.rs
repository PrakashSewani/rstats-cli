use std::{
    fs,
    time::{Duration, SystemTime},
};
use tempfile::tempdir;

use rstats::{
    app::{AppState, DeepCaptureIntent},
    config::Config,
    model::{CpuSnapshot, DiskSnapshot, MetricKind, NetworkSnapshot, ProcessSnapshot, Snapshot},
    recording::{CaptureScope, Recorder},
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
        record_scope: CaptureScope::Standard,
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
    let mut first = Recorder::start(directory.path(), CaptureScope::Standard).unwrap();
    first.record(&Snapshot::default()).unwrap();
    first.finish().unwrap();
    let mut second = Recorder::start(directory.path(), CaptureScope::Standard).unwrap();
    second.record(&Snapshot::default()).unwrap();
    second.finish().unwrap();

    let mut state = AppState::new(config(directory.path().to_path_buf()));
    state.refresh_recordings().unwrap();
    state.load_selected_recording().unwrap();
    assert!(state.loaded_recording.is_some());
    state.move_recording_selection(1);
    assert!(state.loaded_recording.is_none());
}

#[test]
fn state_tracks_network_and_disk_histories() {
    let directory = tempdir().unwrap();
    let mut state = AppState::new(config(directory.path().to_path_buf()));
    let sample = |seconds: u64, received: u64, transmitted: u64, used: f64| Snapshot {
        timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(seconds),
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
    state.apply_snapshot(sample(10, 1_000, 2_000, 60.0));
    state.apply_snapshot(sample(12, 3_000, 4_000, 80.0));

    let receive = state.histories.get(&MetricKind::NetworkReceive).unwrap();
    assert_eq!(receive.as_vec(), vec![0.0, 1_000.0]);
    let transmit = state.histories.get(&MetricKind::NetworkTransmit).unwrap();
    assert_eq!(transmit.as_vec(), vec![0.0, 1_000.0]);
    let disk = state.histories.get(&MetricKind::Disk).unwrap();
    assert_eq!(disk.as_vec(), vec![60.0, 80.0]);
}

#[test]
fn deep_capture_toggle_requires_confirmation_before_collecting() {
    let directory = tempdir().unwrap();
    let mut state = AppState::new(config(directory.path().to_path_buf()));

    state.toggle_record_scope();
    assert_eq!(state.deep_capture_dialog, Some(DeepCaptureIntent::Enable));
    assert_eq!(state.record_scope, CaptureScope::Standard);
    assert!(state.recorder.is_none());

    state.confirm_deep_capture();
    assert_eq!(state.record_scope, CaptureScope::Deep);
    assert!(state.deep_capture_confirmed);
    assert!(state.deep_capture_dialog.is_none());

    state.toggle_recording().unwrap();
    assert!(state.recorder.is_some());
    state.apply_snapshot(Snapshot {
        timestamp: SystemTime::UNIX_EPOCH,
        processes: vec![ProcessSnapshot { pid: 7, ..ProcessSnapshot::default() }],
        ..Snapshot::default()
    });
    let summary = state.stop_recording().unwrap().unwrap();
    let contents = fs::read_to_string(&summary.path).unwrap();
    let sample: serde_json::Value = serde_json::from_str(contents.lines().nth(1).unwrap()).unwrap();
    assert_eq!(sample["snapshot"]["processes"][0]["pid"], 7);
}

#[test]
fn deep_scope_from_config_waits_for_confirmation_to_start() {
    let directory = tempdir().unwrap();
    let mut config = config(directory.path().to_path_buf());
    config.record_scope = CaptureScope::Deep;
    let mut state = AppState::new(config);

    state.toggle_recording().unwrap();
    assert!(state.recorder.is_none());
    assert_eq!(state.deep_capture_dialog, Some(DeepCaptureIntent::Start));

    state.cancel_deep_capture();
    assert!(state.recorder.is_none());
    assert!(state.deep_capture_dialog.is_none());

    state.toggle_recording().unwrap();
    state.confirm_deep_capture();
    assert!(state.recorder.is_some());
    state.stop_recording().unwrap();
}

#[test]
fn capture_scope_cannot_change_while_recording() {
    let directory = tempdir().unwrap();
    let mut state = AppState::new(config(directory.path().to_path_buf()));
    state.toggle_recording().unwrap();
    state.toggle_record_scope();
    assert_eq!(state.record_scope, CaptureScope::Standard);
    assert!(state.recording_error.is_some());
    state.stop_recording().unwrap();
}
