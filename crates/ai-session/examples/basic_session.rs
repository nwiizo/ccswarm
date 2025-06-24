//! Basic session example - demonstrates core AI session functionality

use ai_session::core::AISession;
use ai_session::{SessionConfig, SessionManager};
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("ai_session=debug")
        .init();

    println!("AI Session Basic Example\n");

    // Create session manager
    let manager = SessionManager::new();
    println!("✓ Session manager initialized");

    // Configure session with AI features
    let mut config = SessionConfig::default();
    config.enable_ai_features = true;
    config.context_config.max_tokens = 4096;
    config.context_config.compression_threshold = 0.8;

    println!("\nCreating AI-optimized session...");
    let session = manager.create_session_with_config(config).await?;
    println!("✓ Session created: {}", session.id);

    // Execute some commands
    println!("\nExecuting commands...");

    let commands = vec![
        "echo 'Hello from AI Session!'",
        "pwd",
        "ls -la",
        "echo 'Testing context management...'",
    ];

    for cmd in &commands {
        println!("  > {}", cmd);
        // Send command and read output
        session
            .send_input(&format!(
                "{}
",
                cmd
            ))
            .await?;
        tokio::time::sleep(Duration::from_millis(100)).await; // Give time for command execution
        let output = session.read_output().await?;
        let output_str = String::from_utf8_lossy(&output);
        if !output_str.trim().is_empty() {
            println!("    {}", output_str.trim());
        }
        sleep(Duration::from_millis(500)).await;
    }

    // Demonstrate AI context features
    println!("\n📊 AI Context Statistics:");
    let stats = session.get_context_stats().await?;
    println!("  - Total tokens: {}", stats.total_tokens);
    println!("  - Context efficiency: {:.1}%", stats.efficiency * 100.0);
    println!("  - Compression ratio: {:.2}x", stats.compression_ratio);

    // Show semantic understanding
    println!("\n🧠 Semantic Analysis:");
    let analysis = session.analyze_recent_output().await?;
    println!("  - Detected patterns: {:?}", analysis.patterns);
    println!("  - Key entities: {:?}", analysis.entities);
    println!("  - Suggested next actions: {:?}", analysis.suggestions);

    // Demonstrate multi-agent coordination
    println!("\n🤝 Multi-Agent Ready:");
    println!("  - Session can coordinate with other agents");
    println!("  - Shared context available via coordination bus");
    println!("  - Performance metrics tracked");

    // Clean up
    println!("\nTerminating session...");
    session.stop().await?;
    println!("✓ Session terminated cleanly");

    Ok(())
}

// Extension trait for demo purposes
trait SessionDemo {
    async fn get_context_stats(&self) -> Result<ContextStats>;
    async fn analyze_recent_output(&self) -> Result<SemanticAnalysis>;
}

struct ContextStats {
    total_tokens: usize,
    efficiency: f64,
    compression_ratio: f64,
}

struct SemanticAnalysis {
    patterns: Vec<String>,
    entities: Vec<String>,
    suggestions: Vec<String>,
}

impl SessionDemo for AISession {
    async fn get_context_stats(&self) -> Result<ContextStats> {
        // Demo implementation
        Ok(ContextStats {
            total_tokens: 1234,
            efficiency: 0.85,
            compression_ratio: 2.3,
        })
    }

    async fn analyze_recent_output(&self) -> Result<SemanticAnalysis> {
        // Demo implementation
        Ok(SemanticAnalysis {
            patterns: vec!["file_listing".to_string(), "echo_output".to_string()],
            entities: vec!["current_directory".to_string(), "files".to_string()],
            suggestions: vec!["analyze_code_files".to_string(), "run_tests".to_string()],
        })
    }
}
