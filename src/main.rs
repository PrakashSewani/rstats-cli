use anyhow::Result;
use clap::Parser;
use rstats::{app::App, cli::Cli, config::Config};

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(cli.log_level.as_deref().unwrap_or("rstats=info"))
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_cli(&cli)?;
    App::run(config)
}
