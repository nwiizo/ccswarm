//! Scaffold handler: create project + run pipeline in one command

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

/// Scaffold a new project: create directory, git init, run pipeline
pub async fn handle_scaffold(
    dir: &Path,
    task: &str,
    flow: &str,
    timeout: u64,
    default_provider: Option<crate::providers::ProviderKind>,
) -> Result<()> {
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
    tokio::fs::write(
        dir.join("package.json"),
        r#"{"scripts":{"test":"node -e \"console.log('No tests configured yet')\""}}
"#,
    )
    .await?;
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
        .args(pipeline_args(dir, task, flow, timeout, default_provider))
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
        let exit_code = output.status.code().unwrap_or(-1);
        eprintln!();
        eprintln!(
            "{} Pipeline exited with code {}",
            "\u{2717}".bright_red().bold(),
            exit_code
        );
        anyhow::bail!("pipeline failed during scaffold (exit code {})", exit_code);
    }

    Ok(())
}

fn pipeline_args(
    dir: &Path,
    task: &str,
    flow: &str,
    timeout: u64,
    default_provider: Option<crate::providers::ProviderKind>,
) -> Vec<String> {
    let mut args = vec!["--repo".to_string(), dir.to_string_lossy().into_owned()];

    if let Some(provider) = default_provider {
        args.push("--provider".to_string());
        args.push(provider.as_str().to_string());
    }

    args.extend([
        "pipeline".to_string(),
        "--task".to_string(),
        task.to_string(),
        "--flow".to_string(),
        flow.to_string(),
        "--timeout".to_string(),
        timeout.to_string(),
        "--verbose".to_string(),
    ]);

    args
}

#[cfg(test)]
mod tests {
    use super::pipeline_args;
    use crate::providers::ProviderKind;
    use std::path::Path;

    #[test]
    fn pipeline_args_forward_default_provider() {
        let args = pipeline_args(
            Path::new("/tmp/app"),
            "Create an app",
            "quick",
            45,
            Some(ProviderKind::Codex),
        );

        assert!(args.windows(2).any(|pair| pair == ["--provider", "codex"]));
        assert!(args.windows(2).any(|pair| pair == ["--repo", "/tmp/app"]));
    }
}
