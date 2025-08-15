/// CLI commands for subagent management
/// 
/// This module provides command-line interface for managing Claude Code subagents

use crate::subagent::{
    converter::AgentConverter,
    manager::SubagentManager,
    parser::SubagentParser,
    SubagentConfig,
};
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Subagent management commands
#[derive(Debug, Args)]
pub struct SubagentCommands {
    #[command(subcommand)]
    pub command: SubagentCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SubagentCommand {
    /// List all available subagent definitions
    List,
    
    /// Show details of a specific subagent
    Show {
        /// Name of the subagent
        name: String,
    },
    
    /// Convert existing agents to subagent format
    Convert {
        /// Path to ccswarm.json configuration file
        #[arg(short, long, default_value = "ccswarm.json")]
        config: PathBuf,
        
        /// Output directory for subagent definitions
        #[arg(short, long, default_value = ".claude/agents")]
        output: PathBuf,
    },
    
    /// Create a new subagent instance
    Create {
        /// Name of the subagent type to create
        name: String,
    },
    
    /// Delegate a task to a subagent
    Delegate {
        /// Subagent instance ID
        instance: String,
        
        /// Task description
        task: String,
    },
    
    /// Show status of all active subagents
    Status,
    
    /// Validate subagent definitions
    Validate {
        /// Directory containing subagent definitions
        #[arg(short, long, default_value = ".claude/agents")]
        dir: PathBuf,
    },
}

/// Execute subagent commands
pub async fn execute_subagent_command(command: SubagentCommand) -> anyhow::Result<()> {
    match command {
        SubagentCommand::List => list_subagents().await,
        SubagentCommand::Show { name } => show_subagent(&name).await,
        SubagentCommand::Convert { config, output } => convert_agents(&config, &output).await,
        SubagentCommand::Create { name } => create_subagent(&name).await,
        SubagentCommand::Delegate { instance, task } => delegate_task(&instance, &task).await,
        SubagentCommand::Status => show_status().await,
        SubagentCommand::Validate { dir } => validate_definitions(&dir).await,
    }
}

/// List all available subagent definitions
async fn list_subagents() -> anyhow::Result<()> {
    let config = SubagentConfig::default();
    let manager = SubagentManager::new(config.clone());
    manager.initialize().await?;
    
    let definitions = manager.list_definitions().await;
    
    if definitions.is_empty() {
        println!("No subagent definitions found in {:?}", config.agents_dir);
        println!("Run 'ccswarm subagent convert' to convert existing agents.");
    } else {
        println!("Available subagent definitions:");
        for name in definitions {
            println!("  - {}", name);
        }
    }
    
    Ok(())
}

/// Show details of a specific subagent
async fn show_subagent(name: &str) -> anyhow::Result<()> {
    let config = SubagentConfig::default();
    let path = config.agents_dir.join(format!("{}.md", name));
    
    if !path.exists() {
        anyhow::bail!("Subagent '{}' not found", name);
    }
    
    let (definition, instructions) = SubagentParser::parse_file(&path)?;
    
    println!("Subagent: {}", definition.name);
    println!("Description: {}", definition.description);
    println!("\nTools:");
    
    if !definition.tools.standard.is_empty() {
        println!("  Standard: {}", definition.tools.standard.join(", "));
    }
    if !definition.tools.semantic.is_empty() {
        println!("  Semantic: {}", definition.tools.semantic.join(", "));
    }
    if !definition.tools.memory.is_empty() {
        println!("  Memory: {}", definition.tools.memory.join(", "));
    }
    if !definition.tools.custom.is_empty() {
        println!("  Custom: {}", definition.tools.custom.join(", "));
    }
    
    println!("\nCapabilities:");
    for cap in &definition.capabilities {
        println!("  - {}", cap);
    }
    
    println!("\nInstructions Preview:");
    let preview: String = instructions.lines().take(10).collect::<Vec<_>>().join("\n");
    println!("{}", preview);
    if instructions.lines().count() > 10 {
        println!("... (truncated)");
    }
    
    Ok(())
}

/// Convert existing agents to subagent format
async fn convert_agents(config_path: &PathBuf, output_dir: &PathBuf) -> anyhow::Result<()> {
    println!("Converting agents from {:?} to {:?}", config_path, output_dir);
    
    if !config_path.exists() {
        anyhow::bail!("Configuration file not found: {:?}", config_path);
    }
    
    let converted = AgentConverter::batch_convert_project(config_path, output_dir)?;
    
    if converted.is_empty() {
        println!("No agents found to convert.");
    } else {
        println!("Successfully converted {} agents:", converted.len());
        for name in converted {
            println!("  ✓ {}", name);
        }
        println!("\nSubagent definitions saved to {:?}", output_dir);
    }
    
    Ok(())
}

/// Create a new subagent instance
async fn create_subagent(name: &str) -> anyhow::Result<()> {
    let config = SubagentConfig::default();
    let manager = SubagentManager::new(config);
    manager.initialize().await?;
    
    println!("Creating subagent instance of type '{}'...", name);
    
    let instance_id = manager.create_subagent(name).await?;
    
    println!("✓ Created subagent instance: {}", instance_id);
    
    // Wait for initialization
    for _ in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Some(status) = manager.get_status(&instance_id).await {
            if matches!(status, crate::subagent::manager::SubagentStatus::Available) {
                println!("✓ Subagent ready for tasks");
                break;
            }
        }
    }
    
    Ok(())
}

/// Delegate a task to a subagent
async fn delegate_task(instance_id: &str, task: &str) -> anyhow::Result<()> {
    let config = SubagentConfig::default();
    let manager = SubagentManager::new(config);
    
    println!("Delegating task to subagent {}...", instance_id);
    
    let task_id = manager.delegate_task(instance_id, task).await?;
    
    println!("✓ Task delegated successfully");
    println!("  Task ID: {}", task_id);
    println!("  Description: {}", task);
    
    Ok(())
}

/// Show status of all active subagents
async fn show_status() -> anyhow::Result<()> {
    let config = SubagentConfig::default();
    let manager = SubagentManager::new(config);
    manager.initialize().await?;
    
    let instances = manager.list_instances().await;
    
    if instances.is_empty() {
        println!("No active subagent instances.");
    } else {
        println!("Active subagent instances:");
        println!("{:<40} Status", "Instance ID");
        println!("{}", "-".repeat(60));
        
        for (id, status) in instances {
            println!("{:<40} {:?}", id, status);
        }
    }
    
    Ok(())
}

/// Validate subagent definitions
async fn validate_definitions(dir: &PathBuf) -> anyhow::Result<()> {
    println!("Validating subagent definitions in {:?}...", dir);
    
    if !dir.exists() {
        anyhow::bail!("Directory not found: {:?}", dir);
    }
    
    let definitions = SubagentParser::parse_directory(dir)?;
    
    if definitions.is_empty() {
        println!("No subagent definitions found.");
    } else {
        println!("Validated {} subagent definitions:", definitions.len());
        for (def, _) in definitions {
            println!("  ✓ {} - {}", def.name, def.description);
        }
    }
    
    Ok(())
}