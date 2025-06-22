//! Demo of container isolation mode
//!
//! This example demonstrates how to use container isolation for agents.

use ccswarm::agent::{ClaudeCodeAgent, IsolationMode};
use ccswarm::config::ClaudeConfig;
use ccswarm::identity::AgentRole;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 ccswarm Container Isolation Demo");
    println!("=====================================\n");

    // Check Docker availability
    match ccswarm::container::docker::DockerContainerProvider::new().await {
        Ok(_) => println!("✅ Docker is available"),
        Err(e) => {
            println!("❌ Docker is not available: {}", e);
            println!("   Please ensure Docker is installed and running.");
            return Ok(());
        }
    }

    // Show isolation modes
    println!("\n📋 Available Isolation Modes:");
    println!("1. GitWorktree - File system isolation via git worktrees (default)");
    println!("2. Container - Full process isolation via Docker containers");
    println!("3. Hybrid - Container with worktree fallback\n");

    // Demonstrate each isolation mode
    let modes = vec![
        ("GitWorktree", IsolationMode::GitWorktree),
        ("Container", IsolationMode::Container),
        ("Hybrid", IsolationMode::Hybrid),
    ];

    for (name, mode) in modes {
        println!("🔧 Testing {} isolation mode:", name);

        let role = AgentRole::Backend {
            technologies: vec!["rust".to_string()],
            responsibilities: vec!["API development".to_string()],
            boundaries: vec!["Backend only".to_string()],
        };

        let agent = ClaudeCodeAgent::new_with_isolation(
            role,
            &PathBuf::from("./demo-workspace"),
            "demo",
            ClaudeConfig::default(),
            mode,
        )
        .await?;

        println!("   Agent ID: {}", agent.identity.agent_id);
        println!("   Isolation: {:?}", agent.isolation_mode);
        println!(
            "   Requires Docker: {}",
            agent.isolation_mode.requires_docker()
        );
        println!("   Uses Worktree: {}", agent.isolation_mode.uses_worktree());
        println!();
    }

    println!("✨ Demo complete!");
    println!("\nTo use container isolation in your project:");
    println!("   cargo run -- start --isolation container");
    println!("   cargo run -- start --isolation hybrid");

    Ok(())
}
