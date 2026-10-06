use clap::{ArgGroup, Parser};
use std::path::PathBuf;

use crate::{export::ExportFormat, tui::theme::ThemeName};

#[derive(Debug, Parser)]
#[command(
    name = "rstats",
    version,
    about = "Cross-platform terminal system resource monitor",
    group(
        ArgGroup::new("headless")
            .args(["once", "watch"])
            .multiple(false)
            .conflicts_with_all(["monitor", "open_recordings"])
    )
)]
pub struct Cli {
    #[arg(long, value_name = "MILLISECONDS")]
    pub interval_ms: Option<u64>,
    #[arg(long, value_name = "SECONDS")]
    pub history_seconds: Option<u64>,
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "DIRECTORY")]
    pub recording_dir: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "open_recordings",
        help = "Explicitly launch the interactive terminal monitor"
    )]
    pub monitor: bool,
    #[arg(long, conflicts_with = "monitor", help = "Open the configured recordings directory")]
    pub open_recordings: bool,
    #[arg(long, help = "Collect a single snapshot, print it, and exit")]
    pub once: bool,
    #[arg(long, help = "Stream snapshots as plain text lines until interrupted")]
    pub watch: bool,
    #[arg(
        long,
        requires = "headless",
        help = "Emit JSON output instead of text (use with --once or --watch)"
    )]
    pub json: bool,
    #[arg(long, value_name = "NAME", help = "Color theme: dark, light, or mono")]
    pub theme: Option<ThemeName>,
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["monitor", "open_recordings", "once", "watch", "report"],
        help = "Export a recording to CSV or JSON and exit"
    )]
    pub export: Option<PathBuf>,
    #[arg(
        long,
        value_name = "FORMAT",
        requires = "export",
        help = "Export format: csv or json (default csv)"
    )]
    pub format: Option<ExportFormat>,
    #[arg(
        long,
        value_name = "FILE",
        requires = "export",
        help = "Write export output to FILE instead of stdout"
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["monitor", "open_recordings", "once", "watch"],
        help = "Print an averages and peaks summary for a recording and exit"
    )]
    pub report: Option<PathBuf>,
    #[arg(long)]
    pub no_color: bool,
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,
}
