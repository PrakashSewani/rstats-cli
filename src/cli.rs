use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rstats", version, about = "Cross-platform terminal system resource monitor")]
pub struct Cli {
    #[arg(long, value_name = "MILLISECONDS")]
    pub interval_ms: Option<u64>,
    #[arg(long, value_name = "SECONDS")]
    pub history_seconds: Option<u64>,
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "DIRECTORY")]
    pub recording_dir: Option<PathBuf>,
    #[arg(long)]
    pub no_color: bool,
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,
}
