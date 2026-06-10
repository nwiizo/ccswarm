use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ccswarm::cli::{Cli, CliRunner};

#[tokio::main]
async fn main() {
    let exit_code = run_main().await;
    // Flush buffered OTel spans before the process exits — the batch exporter
    // would otherwise silently drop them for short-lived invocations.
    shutdown_otel();
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

async fn run_main() -> i32 {
    // Check if first arg looks like a task (not a subcommand or flag)
    let args: Vec<String> = std::env::args().collect();
    let is_direct_task = args.len() >= 2
        && !args[1].starts_with('-')
        && !is_known_subcommand(&args[1])
        && !looks_like_subcommand(&args[1]);

    if args.len() == 1 {
        // Interactive mode: ccswarm (no args)
        init_logging(false, "text");
        if let Err(e) = run_interactive().await {
            display_error(&e, false);
            return 1;
        }
        return 0;
    }

    if is_direct_task {
        // Direct task mode: ccswarm "タスクを書くだけ"
        init_logging(false, "text");
        let task = args[1..].join(" ");
        if let Err(e) = run_direct_task(&task).await {
            display_error(&e, false);
            return 1;
        }
        return 0;
    }

    let cli = Cli::parse();

    init_logging(cli.verbose, &cli.log_format);

    if let Err(e) = run_cli(&cli).await {
        display_error(&e, cli.verbose);
        return 1;
    }
    0
}

fn is_known_subcommand(arg: &str) -> bool {
    matches!(
        arg,
        "doctor"
            | "help"
            | "init"
            | "task"
            | "agents"
            | "agent-gen"
            | "worktree"
            | "logs"
            | "config"
            | "interactive"
            | "pipeline"
            | "health"
            | "quickstart"
            | "flow"
            | "repertoire"
            | "lab"
            | "harness"
            | "approve"
            | "session"
            | "run"
            | "facets"
            | "queue"
            | "auto"
            | "undo"
            | "replay"
            | "cost"
            | "tail"
            | "scaffold"
    )
}

/// Detect if a single argument looks like a CLI subcommand attempt rather than
/// a natural language task description. Command-like strings are all-lowercase
/// with optional dashes/underscores (e.g., "invalid-command", "my_cmd").
/// Natural language tasks typically contain uppercase letters, spaces, or
/// non-ASCII characters (e.g., "Fix the bug", "Snakeゲームを作って").
fn looks_like_subcommand(arg: &str) -> bool {
    // Multiple args joined by shell means it's clearly a task description
    // (this function is only called for args[1], but we check anyway)
    if arg.contains(' ') {
        return false;
    }

    // If it contains uppercase, digits at start, or non-ASCII, it's likely a task
    // Command names are typically: lowercase letters, dashes, underscores
    !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_')
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
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(json_layer)
                .with(otel_layer())
                .init();
        }
        _ => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .compact();
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .with(otel_layer())
                .init();
        }
    }
}

/// OTLP span export layer. Built only with `--features otel`, and active only
/// when `OTEL_EXPORTER_OTLP_ENDPOINT` is set — so the feature can ship in a
/// binary without affecting runs that don't opt in.
/// The active tracer provider, kept so `shutdown_otel` can flush the batch
/// exporter at process exit.
#[cfg(feature = "otel")]
static OTEL_PROVIDER: std::sync::OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> =
    std::sync::OnceLock::new();

#[cfg(feature = "otel")]
fn otel_layer<S>() -> Option<impl tracing_subscriber::Layer<S>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    use opentelemetry::trace::TracerProvider as _;

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_err() {
        return None;
    }
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .map_err(|e| eprintln!("otel: failed to build OTLP exporter: {e}"))
        .ok()?;
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();
    let tracer = provider.tracer("ccswarm");
    opentelemetry::global::set_tracer_provider(provider.clone());
    let _ = OTEL_PROVIDER.set(provider);
    Some(tracing_opentelemetry::layer().with_tracer(tracer))
}

#[cfg(feature = "otel")]
fn shutdown_otel() {
    if let Some(provider) = OTEL_PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        eprintln!("otel: failed to flush spans on shutdown: {e}");
    }
}

#[cfg(not(feature = "otel"))]
fn otel_layer() -> Option<tracing_subscriber::layer::Identity> {
    None
}

#[cfg(not(feature = "otel"))]
fn shutdown_otel() {}

/// Interactive mode: `ccswarm` with no arguments
/// Asks what to build, then runs the full pipeline with OK/NG flow.
async fn run_interactive() -> anyhow::Result<()> {
    use colored::Colorize;

    eprintln!("{}", "ccswarm".bright_cyan().bold());
    eprintln!();
    eprint!("What do you want to build? > ");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    let mut task = String::new();
    std::io::stdin().read_line(&mut task)?;
    let task = task.trim();

    if task.is_empty() {
        eprintln!("No task specified. Exiting.");
        return Ok(());
    }

    run_direct_task(task).await
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
        eprintln!(
            "{}",
            colored::Colorize::bright_cyan(colored::Colorize::bold("ccswarm"))
        );
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
        eprintln!(
            "  {} Project initialized",
            colored::Colorize::bright_green("\u{2713}")
        );
    }

    // Build a minimal Cli for CliRunner
    let cli = Cli::parse_from([
        "ccswarm",
        "--repo",
        ".",
        "pipeline",
        "--task",
        task,
        "--flow",
        "default",
        "--timeout",
        "600",
    ]);
    let runner = CliRunner::new(&cli).await?;

    // Run pipeline with full OK/NG flow
    runner
        .handle_pipeline(
            task, "default", "text", 600, false, None, false, // isolate
            None,  // budget
            None,  // run_budget_tokens
            None,  // model
            false, // auto_commit (ask instead)
            false, // create_pr (ask instead)
            None,  // approval_gate (interactive ask is the gate)
        )
        .await
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
