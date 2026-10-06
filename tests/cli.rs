use clap::Parser;
use rstats::{cli::Cli, tui::theme::ThemeName};

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

#[test]
fn parses_headless_modes() {
    let once = Cli::try_parse_from(["rstats", "--once"]).unwrap();
    assert!(once.once);
    assert!(!once.watch);
    assert!(!once.json);

    let watch = Cli::try_parse_from(["rstats", "--watch"]).unwrap();
    assert!(watch.watch);
    assert!(!watch.once);

    let watch_json = Cli::try_parse_from(["rstats", "--watch", "--json"]).unwrap();
    assert!(watch_json.watch);
    assert!(watch_json.json);

    let once_json = Cli::try_parse_from(["rstats", "--once", "--json"]).unwrap();
    assert!(once_json.once);
    assert!(once_json.json);
}

#[test]
fn rejects_headless_modes_with_tui_modes() {
    assert!(Cli::try_parse_from(["rstats", "--monitor", "--once"]).is_err());
    assert!(Cli::try_parse_from(["rstats", "--monitor", "--watch"]).is_err());
    assert!(Cli::try_parse_from(["rstats", "--once", "--open-recordings"]).is_err());
    assert!(Cli::try_parse_from(["rstats", "--watch", "--open-recordings"]).is_err());
}

#[test]
fn rejects_once_and_watch_together() {
    assert!(Cli::try_parse_from(["rstats", "--once", "--watch"]).is_err());
}

#[test]
fn rejects_json_without_headless_mode() {
    assert!(Cli::try_parse_from(["rstats", "--json"]).is_err());
    assert!(Cli::try_parse_from(["rstats", "--monitor", "--json"]).is_err());
}

#[test]
fn parses_theme_option() {
    let cli = Cli::try_parse_from(["rstats", "--theme", "mono"]).unwrap();
    assert_eq!(cli.theme, Some(ThemeName::Mono));
    let cli = Cli::try_parse_from(["rstats", "--monitor", "--theme", "light"]).unwrap();
    assert_eq!(cli.theme, Some(ThemeName::Light));
    assert!(Cli::try_parse_from(["rstats", "--theme", "banana"]).is_err());
}
