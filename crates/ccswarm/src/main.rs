use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ccswarm::cli::{Cli, CliRunner};

#[tokio::main]
async fn main() {
    // Check if first arg looks like a task (not a subcommand or flag)
    let args: Vec<String> = std::env::args().collect();
    let is_direct_task = args.len() >= 2
        && !args[1].starts_with('-')
        && !is_known_subcommand(&args[1]);

    if is_direct_task {
        // Direct task mode: ccswarm "タスクを書くだけ"
        init_logging(false, "text");
        let task = args[1..].join(" ");
        if let Err(e) = run_direct_task(&task).await {
            display_error(&e, false);
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    init_logging(cli.verbose, &cli.log_format);

    if let Err(e) = run_cli(&cli).await {
        display_error(&e, cli.verbose);
        std::process::exit(1);
    }
}

fn is_known_subcommand(arg: &str) -> bool {
    matches!(
        arg,
        "runs" | "pieces" | "doctor" | "help"
            | "init" | "task" | "agents" | "agent-gen" | "worktree"
            | "logs" | "config" | "setup" | "tutorial" | "interactive"
            | "pipeline" | "help-topic" | "health" | "quickstart"
            | "piece" | "repertoire" | "sangha" | "extend" | "search"
            | "evolution" | "harness" | "approve" | "session" | "run" | "scaffold"
    )
}

fn init_logging(verbose: bool, format: &str) {
    let log_level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    let filter_layer =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into());

    match format {
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
}

/// Direct task mode: `ccswarm "Snakeゲームを作って"`
/// Auto-detects new project vs existing, runs full pipeline with OK/NG flow.
async fn run_direct_task(task: &str) -> anyhow::Result<()> {
    use ccswarm::cli::CliRunner;
    use std::path::PathBuf;

    let repo = PathBuf::from(".");
    let is_new_project = !repo.join("package.json").exists()
        && !repo.join("Cargo.toml").exists()
        && !repo.join("go.mod").exists()
        && !repo.join("pyproject.toml").exists()
        && !repo.join("index.html").exists();

    if is_new_project {
        // New project: scaffold first
        eprintln!("{}", colored::Colorize::bright_cyan(colored::Colorize::bold("ccswarm")));
        eprintln!();

        // Initialize project files
        if !repo.join(".git").exists() {
            let _ = tokio::process::Command::new("git")
                .arg("init")
                .output()
                .await;
            let _ = tokio::process::Command::new("git")
                .args(["config", "user.email", "ccswarm@local"])
                .output()
                .await;
            let _ = tokio::process::Command::new("git")
                .args(["config", "user.name", "ccswarm"])
                .output()
                .await;
        }
        tokio::fs::write("package.json", "{}\n").await?;
        let _ = tokio::fs::create_dir_all("public").await;
        let _ = tokio::fs::create_dir_all("e2e").await;
        let _ = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .output()
            .await;
        let _ = tokio::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .output()
            .await;
        eprintln!("  {} Project initialized", colored::Colorize::bright_green("\u{2713}"));
    }

    // Build a minimal Cli for CliRunner
    let cli = Cli::parse_from(["ccswarm", "--repo", ".", "pipeline", "--task", task, "--piece", "default", "--timeout", "600"]);
    let runner = CliRunner::new(&cli).await?;

    // Run pipeline with full OK/NG flow
    runner.handle_pipeline(
        task,
        "default",
        "text",
        600,
        false,
        None,
        false,   // isolate
        None,    // budget
        None,    // model
        false,   // auto_commit (ask instead)
        false,   // create_pr (ask instead)
    ).await
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
