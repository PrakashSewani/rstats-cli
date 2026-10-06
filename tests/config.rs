use std::{fs, time::Duration};
use tempfile::tempdir;

use rstats::{cli::Cli, config::Config, tui::theme::ThemeName};

#[test]
fn cli_values_override_file_values() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("config.toml");
    fs::write(&path, "interval_ms = 5000\nhistory_seconds = 10\n").unwrap();
    let cli = Cli {
        interval_ms: Some(200),
        history_seconds: Some(20),
        config: Some(path),
        recording_dir: None,
        monitor: false,
        open_recordings: false,
        once: false,
        watch: false,
        json: false,
        theme: None,
        export: None,
        format: None,
        output: None,
        report: None,
        no_color: false,
        log_level: None,
    };
    let config = Config::from_cli(&cli).unwrap();
    assert_eq!(config.interval, Duration::from_millis(200));
    assert_eq!(config.history_capacity, 100);
    assert_eq!(config.recording_directory, std::path::PathBuf::from("recordings"));
}

#[test]
fn theme_reads_from_file_and_cli_overrides() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("config.toml");
    fs::write(&path, "theme = \"light\"\n").unwrap();
    let cli = Cli {
        interval_ms: None,
        history_seconds: None,
        config: Some(path),
        recording_dir: None,
        monitor: false,
        open_recordings: false,
        once: false,
        watch: false,
        json: false,
        theme: None,
        export: None,
        format: None,
        output: None,
        report: None,
        no_color: false,
        log_level: None,
    };
    let config = Config::from_cli(&cli).unwrap();
    assert_eq!(config.theme, ThemeName::Light);
    assert_eq!(config.theme.as_str(), "light");

    let cli = Cli { theme: Some(ThemeName::Mono), ..cli };
    let config = Config::from_cli(&cli).unwrap();
    assert_eq!(config.theme, ThemeName::Mono);
}

#[test]
fn rejects_unknown_theme_in_config_file() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("config.toml");
    fs::write(&path, "theme = \"banana\"\n").unwrap();
    let cli = Cli {
        interval_ms: None,
        history_seconds: None,
        config: Some(path),
        recording_dir: None,
        monitor: false,
        open_recordings: false,
        once: false,
        watch: false,
        json: false,
        theme: None,
        export: None,
        format: None,
        output: None,
        report: None,
        no_color: false,
        log_level: None,
    };
    assert!(Config::from_cli(&cli).is_err());
}
