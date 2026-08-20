use anyhow::Result;
use clap::Parser;
use rstats::{app::App, cli::Cli, config::Config, open_recordings::open_recordings};

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(cli.log_level.as_deref().unwrap_or("rstats=info"))
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_cli(&cli)?;
    if cli.open_recordings {
        return open_recordings(&config.recording_directory);
    }
    App::run(config)
}
