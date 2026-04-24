//! Scaffold handler: create project + run pipeline in one command

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

/// Scaffold a new project: create directory, git init, run pipeline
pub async fn handle_scaffold(dir: &Path, task: &str, flow: &str, timeout: u64) -> Result<()> {
    eprintln!("{} {}", "Scaffolding:".bright_cyan().bold(), dir.display());

    // 1. Create directory
    tokio::fs::create_dir_all(dir)
        .await
        .with_context(|| format!("Failed to create directory {:?}", dir))?;

    // 2. Initialize git
    let git_init = tokio::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .await
        .context("Failed to run git init")?;

    if !git_init.status.success() {
        anyhow::bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&git_init.stderr)
        );
    }

    // Configure git user if not set
    let _ = tokio::process::Command::new("git")
        .args(["config", "user.email", "ccswarm@local"])
        .current_dir(dir)
        .output()
        .await;
    let _ = tokio::process::Command::new("git")
        .args(["config", "user.name", "ccswarm"])
        .current_dir(dir)
        .output()
        .await;

    // 3. Create minimal project files
    tokio::fs::write(dir.join("package.json"), "{}\n").await?;
    tokio::fs::create_dir_all(dir.join("public")).await?;
    tokio::fs::create_dir_all(dir.join("e2e")).await?;

    // 4. Initial commit
    let _ = tokio::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .output()
        .await;
    let _ = tokio::process::Command::new("git")
        .args(["commit", "-m", "init scaffold"])
        .current_dir(dir)
        .output()
        .await;

    eprintln!("  {} Project initialized", "\u{2713}".bright_green());

    // 5. Run pipeline
    eprintln!(
        "  {} Running pipeline (flow: {}, timeout: {}s)...",
        "\u{25b6}".bright_blue(),
        flow,
        timeout
    );

    let ccswarm_bin = std::env::current_exe().unwrap_or_else(|_| "ccswarm".into());
    let output = tokio::process::Command::new(&ccswarm_bin)
        .args([
            "--repo",
            &dir.to_string_lossy(),
            "pipeline",
            "--task",
            task,
            "--flow",
            flow,
            "--timeout",
            &timeout.to_string(),
            "--verbose",
        ])
        .output()
        .await
        .context("Failed to run pipeline")?;

    // Display pipeline output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }
    if !stdout.is_empty() {
        print!("{}", stdout);
    }

    if output.status.success() {
        eprintln!();
        eprintln!(
            "{} Project ready at {}",
            "\u{2713}".bright_green().bold(),
            dir.display()
        );
    } else {
        eprintln!();
        eprintln!(
            "{} Pipeline exited with code {}",
            "\u{2717}".bright_red().bold(),
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(())
}
