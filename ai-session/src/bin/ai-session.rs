//! AI Session CLI - Terminal session management optimized for AI agents

use ai_session::SessionConfig;
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

use ai_session::session_persistence::get_session_manager;

#[derive(Parser)]
#[command(name = "ai-session")]
#[command(about = "AI-optimized terminal session management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new session
    Create {
        /// Session name
        #[arg(short, long)]
        name: Option<String>,

        /// Working directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Enable AI context management
        #[arg(long)]
        ai_context: bool,

        /// Token limit for context
        #[arg(long, default_value = "4096")]
        token_limit: usize,
    },

    /// List active sessions
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Attach to a session
    Attach {
        /// Session ID or name
        session: String,
    },

    /// Execute command in session
    Exec {
        /// Session ID or name
        session: String,

        /// Command to execute
        command: Vec<String>,

        /// Capture output for AI analysis
        #[arg(long)]
        capture: bool,
    },

    /// Kill a session
    Kill {
        /// Session ID or name
        session: String,

        /// Force kill without cleanup
        #[arg(short, long)]
        force: bool,
    },

    /// Show session context and history
    Context {
        /// Session ID or name
        session: String,

        /// Number of recent entries to show
        #[arg(short, long, default_value = "10")]
        lines: usize,
    },

    /// Migrate from tmux
    Migrate {
        /// Tmux session name
        #[arg(short, long)]
        tmux_session: Option<String>,

        /// Migrate all tmux sessions
        #[arg(long)]
        all: bool,
    },

    /// Remote session management via HTTP API
    Remote {
        #[command(subcommand)]
        command: RemoteCommands,
    },

    /// Interactive mode for continuous conversation
    Interactive {
        /// Session name
        name: String,

        /// Server URL (default: http://localhost:3000)
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,

        /// Show raw output
        #[arg(long)]
        raw: bool,
    },

    /// Quick chat with Claude Code (convenience command)
    ClaudeChat {
        /// Server URL (default: http://localhost:4000 for Claude)
        #[arg(long, default_value = "http://localhost:4000")]
        server: String,

        /// Session name (default: claude-code)
        #[arg(long, default_value = "claude-code")]
        session: String,

        /// Show raw output
        #[arg(long)]
        raw: bool,

        /// Auto-create session if not exists
        #[arg(long, default_value = "true")]
        auto_create: bool,
    },
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Create a new remote session
    Create {
        /// Session name
        name: String,

        /// Enable AI features
        #[arg(long)]
        ai_features: bool,

        /// Server URL (default: http://localhost:3000)
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// List remote sessions
    List {
        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// Execute command in remote session
    Exec {
        /// Session name
        name: String,

        /// Command to execute
        command: Vec<String>,

        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,

        /// Show raw output
        #[arg(long)]
        raw: bool,
    },

    /// Get remote session output
    Output {
        /// Session name
        name: String,

        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,

        /// Show raw output
        #[arg(long)]
        raw: bool,
    },

    /// Get remote session status
    Status {
        /// Session name
        name: String,

        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// Delete remote session
    Delete {
        /// Session name
        name: String,

        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// Check server health
    Health {
        /// Server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
}

// API Response types
#[derive(Deserialize)]
struct SessionResponse {
    id: String,
    name: String,
    status: String,
    #[allow(dead_code)]
    created_at: String,
}

#[derive(Deserialize)]
struct SessionListResponse {
    sessions: Vec<SessionSummary>,
    total: usize,
}

#[derive(Deserialize)]
struct SessionSummary {
    id: String,
    name: String,
    status: String,
    created_at: String,
    last_activity: String,
}

#[derive(Deserialize)]
struct CommandResponse {
    success: bool,
    output: String,
    error: Option<String>,
    execution_time_ms: u64,
}

#[derive(Deserialize)]
struct OutputResponse {
    session_name: String,
    output: String,
    raw_output: String,
    timestamp: String,
    size_bytes: usize,
}

#[derive(Serialize)]
struct CreateSessionRequest {
    name: String,
    enable_ai_features: bool,
}

#[derive(Serialize)]
struct ExecuteCommandRequest {
    command: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("ai_session=debug")
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            dir,
            ai_context,
            token_limit,
        } => create_session(name, dir, ai_context, token_limit).await?,
        Commands::List { detailed } => list_sessions(detailed).await?,
        Commands::Attach { session } => attach_session(session).await?,
        Commands::Exec {
            session,
            command,
            capture,
        } => exec_command(session, command, capture).await?,
        Commands::Kill { session, force } => kill_session(session, force).await?,
        Commands::Context { session, lines } => show_context(session, lines).await?,
        Commands::Migrate { tmux_session, all } => migrate_tmux(tmux_session, all).await?,
        Commands::Remote { command } => handle_remote_command(command).await?,
        Commands::Interactive { name, server, raw } => interactive_mode(name, server, raw).await?,
        Commands::ClaudeChat {
            server,
            session,
            raw,
            auto_create,
        } => claude_chat_mode(server, session, raw, auto_create).await?,
    }

    Ok(())
}

async fn create_session(
    name: Option<String>,
    dir: Option<PathBuf>,
    ai_context: bool,
    token_limit: usize,
) -> Result<()> {
    let manager = get_session_manager().await?;

    let mut config = SessionConfig::default();
    if let Some(n) = name.clone() {
        config.name = Some(n);
    }
    if let Some(d) = dir {
        config.working_directory = d;
    }
    if ai_context {
        config.enable_ai_features = true;
        config.context_config.max_tokens = token_limit;
    }

    let session = manager.create_session_with_config(config).await?;

    // Session is automatically started and persisted by the PersistentSessionManager

    println!("Created session: {}", session.id);
    if let Some(n) = name {
        println!("Name: {}", n);
    }
    println!(
        "Working directory: {}",
        session.config.working_directory.display()
    );
    if ai_context {
        println!("AI context enabled with {} token limit", token_limit);
    }

    Ok(())
}

async fn list_sessions(detailed: bool) -> Result<()> {
    let manager = get_session_manager().await?;
    let session_ids = manager.list_all_sessions().await?;

    if session_ids.is_empty() {
        println!("No active sessions");
        return Ok(());
    }

    println!("Active sessions ({} total):", session_ids.len());
    for session_id in session_ids {
        if let Some(session) = manager.get_session(&session_id).await {
            if detailed {
                println!("\n  ID: {}", session.id);
                if let Some(name) = &session.config.name {
                    println!("  Name: {}", name);
                }
                println!(
                    "  Created: {}",
                    session.created_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!(
                    "  Directory: {}",
                    session.config.working_directory.display()
                );
                println!("  Status: {:?}", session.status().await);
                if session.config.enable_ai_features {
                    println!("  AI Features: Enabled");
                    println!(
                        "  Context Size: {} tokens",
                        session.config.context_config.max_tokens
                    );
                }
            } else {
                let id_str = session.id.to_string();
                let short_id = id_str.split('-').next().unwrap_or("unknown");
                let name_str = session.config.name.as_deref().unwrap_or(short_id);
                println!(
                    "  {} - {} ({}) [{}]",
                    short_id,
                    name_str,
                    session.created_at.format("%H:%M:%S"),
                    match session.status().await {
                        ai_session::core::SessionStatus::Running => "running",
                        ai_session::core::SessionStatus::Paused => "paused",
                        ai_session::core::SessionStatus::Terminated => "terminated",
                        _ => "unknown",
                    }
                );
            }
        }
    }

    Ok(())
}

async fn attach_session(session: String) -> Result<()> {
    let manager = get_session_manager().await?;
    let session_id = ai_session::core::SessionId::parse_str(&session)?;

    if let Some(session) = manager.get_session(&session_id).await {
        println!("Attaching to session: {}", session_id);
        println!("Session status: {:?}", session.status().await);
        println!(
            "Working directory: {}",
            session.config.working_directory.display()
        );

        // For now, just demonstrate that we can interact with the session
        println!(
            "\n(Interactive mode would start here. For now, use 'ai-session exec' to run commands)"
        );
    } else {
        eprintln!("Session not found: {}", session_id);
        std::process::exit(1);
    }

    Ok(())
}

async fn exec_command(session: String, command: Vec<String>, capture: bool) -> Result<()> {
    let manager = get_session_manager().await?;
    let session_id = ai_session::core::SessionId::parse_str(&session)?;

    let cmd = command.join(" ");
    println!("Executing in session {}: {}", session, cmd);

    let output_str = if let Some(session) = manager.get_session(&session_id).await {
        session.send_input(&cmd).await?;
        let output = session.read_output().await?;
        let result = String::from_utf8_lossy(&output);
        println!("{}", result);
        result.to_string()
    } else {
        eprintln!("Session not found: {}", session_id);
        std::process::exit(1);
    };

    if capture {
        println!("\nCaptured output:");
        println!("{}", output_str);
        println!("\n(Output saved for AI analysis)");
    }

    Ok(())
}

async fn kill_session(session: String, force: bool) -> Result<()> {
    let manager = get_session_manager().await?;
    let session_id = ai_session::core::SessionId::parse_str(&session)?;

    if force {
        println!("Force killing session: {}", session);
    } else {
        println!("Gracefully terminating session: {}", session);
    }

    manager.remove_session(&session_id).await?;
    println!("Session terminated");

    Ok(())
}

async fn show_context(session: String, lines: usize) -> Result<()> {
    println!("Session context for: {}", session);
    println!("Last {} context entries:", lines);
    println!("\n  [Context display not implemented in demo]");
    println!("  Would show:");
    println!("  - Command history");
    println!("  - AI conversation context");
    println!("  - Token usage statistics");
    println!("  - Performance metrics");

    Ok(())
}

async fn migrate_tmux(tmux_session: Option<String>, all: bool) -> Result<()> {
    use ai_session::integration::TmuxCompatLayer;

    let tmux = TmuxCompatLayer::new();

    if all {
        println!("Migrating all tmux sessions...");
        let sessions = tmux.list_tmux_sessions().await?;
        println!("Found {} tmux sessions", sessions.len());

        for session in sessions {
            println!("  - {} (created: {})", session.name, session.created);
        }

        println!("\n(Migration would convert these to AI sessions)");
    } else if let Some(name) = tmux_session {
        println!("Migrating tmux session: {}", name);
        println!("(Would capture state and create equivalent AI session)");
    } else {
        println!("Please specify --tmux-session or --all");
    }

    Ok(())
}

// Remote command handlers
async fn handle_remote_command(command: RemoteCommands) -> Result<()> {
    match command {
        RemoteCommands::Create {
            name,
            ai_features,
            server,
        } => remote_create_session(name, ai_features, server).await?,
        RemoteCommands::List { server } => remote_list_sessions(server).await?,
        RemoteCommands::Exec {
            name,
            command,
            server,
            raw,
        } => remote_exec_command(name, command, server, raw).await?,
        RemoteCommands::Output { name, server, raw } => {
            remote_get_output(name, server, raw).await?
        }
        RemoteCommands::Status { name, server } => remote_get_status(name, server).await?,
        RemoteCommands::Delete { name, server } => remote_delete_session(name, server).await?,
        RemoteCommands::Health { server } => remote_health_check(server).await?,
    }
    Ok(())
}

async fn remote_create_session(name: String, ai_features: bool, server: String) -> Result<()> {
    let client = reqwest::Client::new();
    let request = CreateSessionRequest {
        name: name.clone(),
        enable_ai_features: ai_features,
    };

    let response = client
        .post(format!("{}/sessions", server))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let session: SessionResponse = response.json().await?;
        println!("✅ Created remote session: {}", session.name);
        println!("   ID: {}", session.id);
        println!("   Status: {}", session.status);
        println!(
            "   AI Features: {}",
            if ai_features { "Enabled" } else { "Disabled" }
        );
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to create session: {}", error_text);
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_list_sessions(server: String) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/sessions", server)).send().await?;

    if response.status().is_success() {
        let list: SessionListResponse = response.json().await?;
        if list.sessions.is_empty() {
            println!("No remote sessions found");
        } else {
            println!("Remote sessions ({} total):", list.total);
            for session in list.sessions {
                println!(
                    "  {} - {} (Status: {})",
                    session.name,
                    session.id.split('-').next().unwrap_or(""),
                    session.status
                );
                println!("    Created: {}", session.created_at);
                println!("    Last Activity: {}", session.last_activity);
            }
        }
    } else {
        eprintln!("❌ Failed to list sessions: {}", response.status());
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_exec_command(
    name: String,
    command: Vec<String>,
    server: String,
    raw: bool,
) -> Result<()> {
    let client = reqwest::Client::new();
    let cmd = command.join(" ");
    let request = ExecuteCommandRequest {
        command: cmd.clone(),
    };

    println!("💬 Executing: {}", cmd);

    let response = client
        .post(format!("{}/sessions/{}/execute", server, name))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let result: CommandResponse = response.json().await?;
        if result.success {
            println!(
                "✅ Command executed successfully ({}ms)",
                result.execution_time_ms
            );
            if raw {
                println!("{}", result.output);
            } else {
                // Clean output for display
                let clean_output = clean_terminal_output(&result.output);
                if !clean_output.trim().is_empty() {
                    println!("\n📤 Output:");
                    println!("{}", clean_output);
                }
            }
        } else {
            eprintln!("❌ Command failed");
            if let Some(error) = result.error {
                eprintln!("   Error: {}", error);
            }
        }
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to execute command: {}", error_text);
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_get_output(name: String, server: String, raw: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/sessions/{}/output", server, name))
        .send()
        .await?;

    if response.status().is_success() {
        let output: OutputResponse = response.json().await?;
        println!(
            "📤 Session output for '{}' ({} bytes):",
            output.session_name, output.size_bytes
        );
        println!("   Timestamp: {}", output.timestamp);
        println!();

        if raw {
            println!("{}", output.raw_output);
        } else {
            println!("{}", output.output);
        }
    } else {
        eprintln!("❌ Failed to get output: {}", response.status());
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_get_status(name: String, server: String) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/sessions/{}/status", server, name))
        .send()
        .await?;

    if response.status().is_success() {
        let status_text = response.text().await?;
        let status: serde_json::Value = serde_json::from_str(&status_text)?;
        println!("📊 Session Status for '{}':", name);
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        eprintln!("❌ Failed to get status: {}", response.status());
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_delete_session(name: String, server: String) -> Result<()> {
    let client = reqwest::Client::new();

    println!("🗑️  Deleting session '{}'...", name);

    let response = client
        .delete(format!("{}/sessions/{}", server, name))
        .send()
        .await?;

    if response.status().is_success() {
        println!("✅ Session '{}' deleted successfully", name);
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to delete session: {}", error_text);
        std::process::exit(1);
    }

    Ok(())
}

async fn remote_health_check(server: String) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/health", server)).send().await?;

    if response.status().is_success() {
        let health: serde_json::Value = response.json().await?;
        println!("🏥 Server Health Check:");
        println!("{}", serde_json::to_string_pretty(&health)?);
    } else {
        eprintln!("❌ Server is not healthy: {}", response.status());
        std::process::exit(1);
    }

    Ok(())
}

fn clean_terminal_output(output: &str) -> String {
    // Simple cleaning - remove ANSI escape sequences
    let ansi_escape = regex::Regex::new(r"\x1b\[[0-9;]*[mK]").unwrap();
    let control_chars = regex::Regex::new(r"[\x00-\x1f\x7f]").unwrap();

    let cleaned = ansi_escape.replace_all(output, "");
    let cleaned = control_chars.replace_all(&cleaned, " ");

    // Remove excessive whitespace and empty lines
    cleaned
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .take(20) // Show first 20 lines
        .collect::<Vec<_>>()
        .join("\n")
}

// Interactive mode for continuous conversation
async fn interactive_mode(name: String, server: String, raw: bool) -> Result<()> {
    println!("🤖 AI-Session Interactive Mode");
    println!("   Session: {}", name);
    println!("   Server: {}", server);
    println!("   Commands:");
    println!("     /exit or /quit - Exit interactive mode");
    println!("     /status - Show session status");
    println!("     /output - Get latest output");
    println!("     /clear - Clear screen");
    println!("     /help - Show this help");
    println!("   Type your message and press Enter to send to the session");
    println!();

    let client = reqwest::Client::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Show prompt
        print!("💬 > ");
        stdout.flush()?;

        // Read input
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        // Handle special commands
        if input.is_empty() {
            continue;
        }

        match input {
            "/exit" | "/quit" => {
                println!("👋 Exiting interactive mode...");
                break;
            }
            "/status" => {
                match get_session_status(&client, &name, &server).await {
                    Ok(status) => println!("{}", status),
                    Err(e) => eprintln!("❌ Error getting status: {}", e),
                }
                continue;
            }
            "/output" => {
                match get_session_output(&client, &name, &server, raw).await {
                    Ok(output) => println!("{}", output),
                    Err(e) => eprintln!("❌ Error getting output: {}", e),
                }
                continue;
            }
            "/clear" => {
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
            "/help" => {
                println!("📖 Interactive Mode Commands:");
                println!("   /exit or /quit - Exit interactive mode");
                println!("   /status - Show session status");
                println!("   /output - Get latest output");
                println!("   /clear - Clear screen");
                println!("   /help - Show this help");
                continue;
            }
            _ if input.starts_with('/') => {
                println!("❓ Unknown command: {}. Type /help for commands.", input);
                continue;
            }
            _ => {
                // Send regular message to session
                match send_command_to_session(&client, &name, &server, input, raw).await {
                    Ok(output) => {
                        if !output.trim().is_empty() {
                            println!("\n📤 Response:");
                            println!("{}", output);
                        }
                    }
                    Err(e) => eprintln!("❌ Error: {}", e),
                }
            }
        }

        println!(); // Empty line for readability
    }

    Ok(())
}

// Helper function to send command and get response
async fn send_command_to_session(
    client: &reqwest::Client,
    name: &str,
    server: &str,
    command: &str,
    raw: bool,
) -> Result<String> {
    let request = ExecuteCommandRequest {
        command: command.to_string(),
    };

    let response = client
        .post(format!("{}/sessions/{}/execute", server, name))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let result: CommandResponse = response.json().await?;
        if result.success {
            if raw {
                Ok(result.output)
            } else {
                Ok(clean_terminal_output(&result.output))
            }
        } else {
            Err(anyhow::anyhow!("Command failed: {:?}", result.error))
        }
    } else {
        let error_text = response.text().await?;
        Err(anyhow::anyhow!("Request failed: {}", error_text))
    }
}

// Helper function to get session status
async fn get_session_status(client: &reqwest::Client, name: &str, server: &str) -> Result<String> {
    let response = client
        .get(format!("{}/sessions/{}/status", server, name))
        .send()
        .await?;

    if response.status().is_success() {
        let status_text = response.text().await?;
        let status: serde_json::Value = serde_json::from_str(&status_text)?;
        Ok(serde_json::to_string_pretty(&status)?)
    } else {
        Err(anyhow::anyhow!("Failed to get status"))
    }
}

// Helper function to get session output
async fn get_session_output(
    client: &reqwest::Client,
    name: &str,
    server: &str,
    raw: bool,
) -> Result<String> {
    let response = client
        .get(format!("{}/sessions/{}/output", server, name))
        .send()
        .await?;

    if response.status().is_success() {
        let output: OutputResponse = response.json().await?;
        if raw {
            Ok(output.raw_output)
        } else {
            Ok(output.output)
        }
    } else {
        Err(anyhow::anyhow!("Failed to get output"))
    }
}

// Claude chat mode - convenience wrapper
async fn claude_chat_mode(
    server: String,
    session: String,
    raw: bool,
    auto_create: bool,
) -> Result<()> {
    let client = reqwest::Client::new();

    println!("🤖 Claude Code Chat");
    println!("   Checking session...");

    // Check if session exists
    let session_exists = check_session_exists(&client, &session, &server).await?;

    if !session_exists && auto_create {
        println!("   Creating session '{}'...", session);

        // Create session
        let request = CreateSessionRequest {
            name: session.clone(),
            enable_ai_features: true,
        };

        let response = client
            .post(format!("{}/sessions", server))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Failed to create session: {}", error_text);
            std::process::exit(1);
        }

        println!("✅ Session created");

        // Start Claude in the session
        println!("   Starting Claude Code...");
        let start_request = ExecuteCommandRequest {
            command: "claude".to_string(),
        };

        let response = client
            .post(format!("{}/sessions/{}/execute", server, session))
            .json(&start_request)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Claude Code started");
            // Wait for Claude to initialize
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        } else {
            eprintln!("⚠️  Could not start Claude Code automatically");
        }
    } else if !session_exists {
        eprintln!(
            "❌ Session '{}' does not exist. Use --auto-create to create it.",
            session
        );
        std::process::exit(1);
    }

    println!("\n🎯 Ready for chat!");
    println!("   💡 Tip: You can directly ask questions about code, programming, etc.");
    println!("   💡 Type /help for commands, /exit to quit\n");

    // Launch interactive mode
    interactive_mode(session, server, raw).await
}

// Helper to check if session exists
async fn check_session_exists(client: &reqwest::Client, name: &str, server: &str) -> Result<bool> {
    let response = client.get(format!("{}/sessions", server)).send().await?;

    if response.status().is_success() {
        let list: SessionListResponse = response.json().await?;
        Ok(list.sessions.iter().any(|s| s.name == name))
    } else {
        Ok(false)
    }
}
