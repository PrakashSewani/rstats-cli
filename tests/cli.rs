use clap::Parser;
use rstats::cli::Cli;

#[test]
fn parses_explicit_monitor_mode() {
    let cli = Cli::try_parse_from(["rstats", "--monitor"]).unwrap();
    assert!(cli.monitor);
    assert!(!cli.open_recordings);
}

#[test]
fn parses_open_recordings_mode() {
    let cli = Cli::try_parse_from(["rstats", "--open-recordings"]).unwrap();
    assert!(cli.open_recordings);
    assert!(!cli.monitor);
}

#[test]
fn rejects_conflicting_modes() {
    assert!(Cli::try_parse_from(["rstats", "--monitor", "--open-recordings"]).is_err());
}
