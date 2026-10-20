use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};
use tempfile::tempdir;

use rstats::{
    export::{render_csv, render_json, render_report, run_export, ExportFormat},
    model::{CpuSnapshot, DiskSnapshot, MemorySnapshot, NetworkSnapshot, Snapshot},
    recording::{CaptureScope, RecordedSession, Recorder},
};

fn sample(
    seconds: u64,
    cpu: f64,
    memory: f64,
    received: u64,
    transmitted: u64,
    used: f64,
    load: Option<f64>,
) -> Snapshot {
    Snapshot {
        timestamp: UNIX_EPOCH + Duration::from_secs(seconds),
        cpu: CpuSnapshot { total_usage: cpu, per_core: Vec::new() },
        memory: MemorySnapshot { used_percent: memory, ..MemorySnapshot::default() },
        load_average: load,
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
    }
}

fn write_session(directory: &Path) -> PathBuf {
    let mut recorder = Recorder::start(directory, CaptureScope::Standard).unwrap();
    recorder.record(&sample(10, 42.0, 64.0, 1_000, 2_000, 60.0, Some(1.5))).unwrap();
    recorder.record(&sample(12, 50.0, 70.0, 5_000, 4_000, 80.0, Some(2.5))).unwrap();
    recorder.finish().unwrap().path
}

#[test]
fn csv_export_writes_header_and_rows() {
    let directory = tempdir().unwrap();
    let path = write_session(directory.path());
    let output = directory.path().join("export.csv");

    run_export(&path, ExportFormat::Csv, Some(&output)).unwrap();

    let contents = fs::read_to_string(&output).unwrap();
    let mut lines = contents.lines();
    assert_eq!(
        lines.next().unwrap(),
        "timestamp_ms,cpu_total_pct,memory_used_pct,swap_used_pct,load_average,net_received_bps,net_transmitted_bps,disk_max_used_pct"
    );
    assert_eq!(lines.next().unwrap(), "10000,42.00,64.00,0.00,1.50,0,0,60.00");
    assert_eq!(lines.next().unwrap(), "12000,50.00,70.00,0.00,2.50,2000,1000,80.00");
    assert!(lines.next().is_none());
}

#[test]
fn json_export_matches_recorded_values() {
    let directory = tempdir().unwrap();
    let path = write_session(directory.path());
    let session = RecordedSession::load(&path).unwrap();

    let rendered = render_json(&session).unwrap();
    let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["timestamp_ms"], 10000);
    assert_eq!(rows[1]["cpu_total_pct"].as_f64(), Some(50.0));
    assert_eq!(rows[1]["net_received_bps"].as_f64(), Some(2000.0));
    assert_eq!(rows[1]["net_transmitted_bps"].as_f64(), Some(1000.0));
    assert_eq!(rows[1]["load_average"].as_f64(), Some(2.5));
    assert_eq!(rows[1]["disk_max_used_pct"].as_f64(), Some(80.0));
}

#[test]
fn report_summarizes_a_session() {
    let directory = tempdir().unwrap();
    let path = write_session(directory.path());
    let session = RecordedSession::load(&path).unwrap();

    let report = render_report(&session);
    assert!(report.contains("rstats recording report"));
    assert!(report.contains("samples     2"));
    assert!(report.contains("cpu %"));
    assert!(report.contains("peak"));
    assert!(report.contains("(+2s)"));
    assert!(report.contains("net rx"));
    assert!(report.contains("disk %"));
}

#[test]
fn export_rejects_sessions_without_samples() {
    let directory = tempdir().unwrap();
    let recorder = Recorder::start(directory.path(), CaptureScope::Standard).unwrap();
    let summary = recorder.finish().unwrap();

    assert!(run_export(&summary.path, ExportFormat::Csv, None).is_err());

    let session = RecordedSession::load(&summary.path).unwrap();
    assert_eq!(render_csv(&session).lines().count(), 1);
}
