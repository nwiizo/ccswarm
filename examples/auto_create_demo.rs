use anyhow::Result;
use ccswarm::cli::{Cli, Commands};
use clap::Parser;
use std::path::PathBuf;

/// Demonstrates the auto-create functionality of ccswarm
///
/// This example shows how to use the `ccswarm auto-create` command
/// to automatically generate a complete application with multiple agents
/// working together.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 ccswarm Auto-Create Demo");
    println!("==========================\n");

    // Create CLI args for auto-create command
    let args = vec![
        "ccswarm",
        "auto-create",
        "Create a TODO application with React frontend and Express backend",
        "--output",
        "./auto_todo_demo",
    ];

    // Parse CLI arguments
    let cli = Cli::parse_from(args);

    // Create CLI runner
    let runner = ccswarm::cli::CliRunner::new(&cli).await?;

    // Execute the auto-create command
    match &cli.command {
        Commands::AutoCreate { .. } => {
            println!("📋 Starting auto-create process...\n");
            runner.run(&cli.command).await?;
        }
        _ => unreachable!(),
    }

    println!("\n✅ Auto-create demo completed!");
    println!("📂 Check the ./auto_todo_demo directory for the generated application");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_auto_create_workflow() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_app");

        // Create test args
        let args = vec![
            "ccswarm",
            "auto-create",
            "Create a simple TODO app",
            "--output",
            output_path.to_str().unwrap(),
        ];

        // Parse and execute
        let cli = Cli::parse_from(args);
        let runner = ccswarm::cli::CliRunner::new(&cli).await.unwrap();

        // Should execute without errors
        runner.run(&cli.command).await.unwrap();
    }
}
