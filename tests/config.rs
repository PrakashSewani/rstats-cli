use std::{fs, time::Duration};
use tempfile::tempdir;

use rstats::{cli::Cli, config::Config};

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
        no_color: false,
        log_level: None,
    };
    let config = Config::from_cli(&cli).unwrap();
    assert_eq!(config.interval, Duration::from_millis(200));
    assert_eq!(config.history_capacity, 100);
    assert_eq!(config.recording_directory, std::path::PathBuf::from("recordings"));
}
