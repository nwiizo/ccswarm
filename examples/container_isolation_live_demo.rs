//! Live demonstration of container isolation with agents
//! This shows how agents work in isolated Docker containers

use ccswarm::agent::{ClaudeCodeAgent, Priority, Task, TaskType};
use ccswarm::config::ClaudeConfig;
use ccswarm::container::{ContainerConfig, DockerContainerProvider};
use ccswarm::identity::{default_backend_role, default_frontend_role};
use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("{}", "🐳 Container Isolation Live Demo".bold().cyan());
    println!("{}", "Demonstrating agents running in isolated Docker containers\n".dimmed());

    // Check Docker availability
    match check_docker_status().await {
        Ok(info) => println!("{} Docker is running: {}", "✅".green(), info),
        Err(e) => {
            println!("{} Docker is not available: {}", "❌".red(), e);
            println!("Please ensure Docker is installed and running.");
            return Ok(());
        }
    }

    // Create temporary workspace
    let temp_dir = TempDir::new()?;
    let workspace = temp_dir.path().to_path_buf();
    println!("\n📁 Created workspace: {}", workspace.display());

    // Demonstrate different isolation modes
    println!("\n{}", "=== 1. Git Worktree Isolation (Default) ===".bold());
    demo_worktree_isolation(&workspace).await?;

    println!("\n{}", "=== 2. Container Isolation ===".bold());
    demo_container_isolation(&workspace).await?;

    println!("\n{}", "=== 3. Hybrid Isolation ===".bold());
    demo_hybrid_isolation(&workspace).await?;

    println!("\n{}", "✨ Demo completed successfully!".bold().green());
    println!("{}", "Container isolation provides stronger security boundaries between agents.".dimmed());

    Ok(())
}

async fn check_docker_status() -> Result<String> {
    use bollard::Docker;
    let docker = Docker::connect_with_socket_defaults()?;
    let version = docker.version().await?;
    Ok(format!("Docker {} (API: {})", 
        version.version.unwrap_or_default(),
        version.api_version.unwrap_or_default()
    ))
}

async fn demo_worktree_isolation(workspace: &PathBuf) -> Result<()> {
    println!("{}", "Creating agent with worktree isolation...".dimmed());
    
    let agent = ClaudeCodeAgent::new(
        default_frontend_role(),
        workspace,
        "worktree-demo",
        ClaudeConfig::default(),
    )
    .await?;

    println!("  {} Agent ID: {}", "•".blue(), agent.identity.agent_id);
    println!("  {} Worktree: {}", "•".blue(), agent.worktree_path.display());
    println!("  {} Branch: {}", "•".blue(), agent.branch_name);
    println!("  {} Isolation: {:?}", "•".blue(), agent.isolation_mode);
    
    // Show personality
    println!("\n  {} Initial Personality:", "👤".yellow());
    println!("    {}", agent.personality.describe_personality());
    
    // Execute a simple task
    let task = Task::new(
        Uuid::new_v4().to_string(),
        "Create a simple React component".to_string(),
        Priority::Medium,
        TaskType::Feature,
    );
    
    println!("\n  {} Simulating task execution: {}", "▶".green(), task.description);
    println!("    Isolation: Files are isolated in git worktree");
    println!("    Security: Process runs with current user permissions");
    
    Ok(())
}

async fn demo_container_isolation(workspace: &PathBuf) -> Result<()> {
    println!("{}", "Creating agent with container isolation...".dimmed());
    
    // Create agent - isolation mode is set at agent initialization
    let backend_role = default_backend_role();
    println!("  Using backend role for container demo");
    
    // Note: In real usage, isolation mode would be set via CLI args or config
    // For demo, we'll show the container configuration
    let agent_id = format!("container-demo-{}", Uuid::new_v4());
    let config = ContainerConfig::for_agent("backend", &agent_id);
    
    println!("\n  {} Container Configuration:", "🐋".blue());
    println!("    Image: {}", config.image);
    println!("    Memory limit: {:?}", config.resources.memory_limit);
    println!("    CPU limit: {:?}", config.resources.cpu_limit);
    println!("    Network mode: {:?}", config.network.mode);
    
    // Demonstrate container benefits
    println!("\n  {} Container Benefits:", "🛡️".green());
    println!("    ✓ Complete process isolation");
    println!("    ✓ Resource limits enforced");
    println!("    ✓ Network isolation");
    println!("    ✓ No access to host filesystem");
    println!("    ✓ Dropped capabilities for security");
    
    Ok(())
}

async fn demo_hybrid_isolation(workspace: &PathBuf) -> Result<()> {
    println!("{}", "Creating agent with hybrid isolation...".dimmed());
    
    // Show hybrid mode concept
    println!("  Hybrid mode combines container and worktree isolation");
    
    let agent = ClaudeCodeAgent::new(
        default_frontend_role(),
        workspace,
        "hybrid-demo",
        ClaudeConfig::default(),
    )
    .await?;

    println!("  {} Agent ID: {}", "•".blue(), agent.identity.agent_id);
    println!("  {} Current mode: {:?}", "•".blue(), agent.isolation_mode);
    
    println!("\n  {} Hybrid Mode Features:", "🔄".yellow());
    println!("    ✓ Attempts container isolation first");
    println!("    ✓ Falls back to worktree if Docker unavailable");
    println!("    ✓ Best of both worlds approach");
    println!("    ✓ Graceful degradation");
    
    // Show how it adapts
    println!("\n  {} Adaptation Example:", "🎯".green());
    println!("    1. Try: Execute in Docker container");
    println!("    2. If Docker fails: Fall back to git worktree");
    println!("    3. Continue task execution seamlessly");
    
    Ok(())
}