use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ccswarm::cli::{Cli, CliRunner};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let filter_layer =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into());

    match cli.log_format.as_str() {
        "ndjson" | "json" => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(json_layer)
                .init();
        }
        _ => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact();

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        }
    }

    // Create and run CLI with user-friendly error display
    if let Err(e) = run_cli(&cli).await {
        display_error(&e, cli.verbose);
        std::process::exit(1);
    }
}

async fn run_cli(cli: &Cli) -> anyhow::Result<()> {
    let runner = CliRunner::new(cli).await?;
    runner.run(&cli.command).await
}

/// Display errors in a user-friendly format with actionable suggestions
fn display_error(error: &anyhow::Error, verbose: bool) {
    // Try to downcast to CCSwarmError for rich error display
    if let Some(ccswarm_err) = error.downcast_ref::<ccswarm::error::CCSwarmError>() {
        ccswarm_err.to_user_error().display();
        if verbose {
            // Show full error chain in verbose mode
            let mut source = std::error::Error::source(ccswarm_err);
            if source.is_some() {
                eprintln!("  {}:", "Caused by".dimmed());
            }
            while let Some(cause) = source {
                eprintln!("    - {}", cause);
                source = std::error::Error::source(cause);
            }
        }
    } else {
        // Fallback for non-CCSwarmError anyhow errors
        use colored::Colorize;
        eprintln!("\n{} {}", "Error:".red().bold(), error);
        if verbose {
            for cause in error.chain().skip(1) {
                eprintln!("  {} {}", "Caused by:".yellow(), cause);
            }
        }
        eprintln!(
            "\n  {} {}",
            "Tip:".cyan().bold(),
            "Run with '--verbose' for more details, or 'ccswarm doctor' for diagnostics.".dimmed()
        );
    }
}

use colored::Colorize;
