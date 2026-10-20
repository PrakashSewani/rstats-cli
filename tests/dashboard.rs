use ratatui::{backend::TestBackend, Terminal};
use rstats::{
    app::{AppState, DeepCaptureIntent},
    config::Config,
    model::{DiskSnapshot, Snapshot},
    recording::CaptureScope,
    tui::screens::{render_dashboard, render_deep_capture_dialog},
};
use std::time::Duration;

fn config() -> Config {
    Config {
        interval: Duration::from_millis(1000),
        history_capacity: 10,
        recording_directory: "recordings".into(),
        no_color: false,
        bell: false,
        theme: Default::default(),
        alerts: Vec::new(),
        record_scope: CaptureScope::Standard,
    }
}

fn disk(mount_point: &str, used_percent: f64) -> DiskSnapshot {
    DiskSnapshot {
        name: String::new(),
        mount_point: mount_point.to_owned(),
        total_bytes: 1024 * 1024 * 1024,
        available_bytes: 512 * 1024 * 1024,
        used_percent,
    }
}

fn rendered_snapshot(width: u16, height: u16, disks: Vec<DiskSnapshot>) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new(config());
    state.snapshot = Snapshot { disks, ..Snapshot::default() };
    terminal.draw(|frame| render_dashboard(frame, &state)).unwrap();
    terminal.backend().to_string()
}

#[test]
fn dashboard_shows_storage_for_one_disk() {
    let output = rendered_snapshot(120, 30, vec![disk("C:\\", 62.4)]);
    assert!(output.contains("Storage"));
    assert!(output.contains("C:\\"));
    assert!(output.contains("62.4%"));
    assert!(output.contains("512.0 MiB/1.0 GiB"));
}

#[test]
fn dashboard_shows_all_six_disks_on_wide_terminal() {
    let disks = (0..6).map(|index| disk(&format!("/disk{index}"), index as f64 * 10.0)).collect();
    let output = rendered_snapshot(120, 35, disks);
    for index in 0..6 {
        assert!(output.contains(&format!("/disk{index}")));
    }
    assert!(!output.contains("more disks"));
}

#[test]
fn dashboard_uses_compact_storage_on_narrow_terminal() {
    let disks = (0..4).map(|index| disk(&format!("/disk{index}"), 20.0)).collect();
    let output = rendered_snapshot(70, 30, disks);
    assert!(output.contains("Storage"));
    for index in 0..4 {
        assert!(output.contains(&format!("/disk{index}")));
    }
}

#[test]
fn dashboard_hides_storage_when_no_disks_are_reported() {
    let output = rendered_snapshot(120, 30, Vec::new());
    assert!(!output.contains("Storage"));
    assert!(!output.contains("No disks detected"));
}

#[test]
fn dashboard_summarizes_disks_when_storage_area_is_short() {
    let disks = (0..8).map(|index| disk(&format!("/disk{index}"), 20.0)).collect();
    let output = rendered_snapshot(70, 18, disks);
    assert!(output.contains("Storage"));
    assert!(output.contains("more disks"));
}

#[test]
fn dashboard_help_overlay_lists_theme_cycle() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new(config());
    state.help_visible = true;
    terminal.draw(|frame| render_dashboard(frame, &state)).unwrap();
    let output = terminal.backend().to_string();
    assert!(output.contains("Cycle color theme"));
}

#[test]
fn dashboard_renders_all_history_tiles() {
    let output = rendered_snapshot(120, 30, Vec::new());
    assert!(output.contains("CPU %"));
    assert!(output.contains("Memory %"));
    assert!(output.contains("Net I/O /s"));
    assert!(output.contains("Disk %"));
}

#[test]
fn deep_capture_dialog_renders_warning_and_keys() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| render_deep_capture_dialog(frame, DeepCaptureIntent::Enable)).unwrap();
    let output = terminal.backend().to_string();
    assert!(output.contains("Deep capture"));
    assert!(output.contains("process table"));
    assert!(output.contains("100x"));
    assert!(output.contains("[y] enable deep capture"));
}

#[test]
fn deep_capture_dialog_renders_on_small_terminals() {
    let backend = TestBackend::new(24, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| render_deep_capture_dialog(frame, DeepCaptureIntent::Start)).unwrap();
}
