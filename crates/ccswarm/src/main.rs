use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ccswarm::cli::{Cli, CliRunner};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .compact();

    let filter_layer =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    // Create and run CLI
    let runner = CliRunner::new(&cli).await?;
    runner.run(&cli.command).await?;

    Ok(())
}
