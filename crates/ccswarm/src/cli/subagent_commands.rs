/// Subagent command handling module
use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;
use crate::execution::ExecutionEngine;
use crate::orchestrator::master_delegation::{DelegationStrategy, MasterDelegationEngine};

/// Subagent-specific commands
#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum SubagentCommand {
    /// Create a new subagent
    Create {
        /// Name of the subagent
        name: String,
        /// Role of the subagent
        role: String,
        /// Tools to enable
        #[arg(long)]
        tools: Vec<String>,
    },
    /// List all subagents
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    /// Delegate a task to a subagent
    Delegate {
        /// Subagent to delegate to
        subagent: String,
        /// Task description
        task: String,
    },
    /// Show subagent status
    Status {
        /// Subagent name
        name: String,
    },
}

/// Execute a subagent command
pub async fn execute_subagent_command(command: SubagentCommand) -> Result<()> {
    match command {
        SubagentCommand::Create { name, role, tools } => {
            // Validate role
            let valid_roles = ["frontend", "backend", "devops", "qa"];
            let role_lower = role.to_lowercase();
            if !valid_roles.contains(&role_lower.as_str()) {
                println!(
                    "{} Unknown role '{}'. Valid roles: {}",
                    "⚠️".bright_yellow(),
                    role,
                    valid_roles.join(", ")
                );
                return Ok(());
            }

            println!(
                "🚀 Creating subagent '{}' with role '{}'",
                name.bright_cyan(),
                role.bright_green()
            );

            if !tools.is_empty() {
                println!("   Tools: {}", tools.join(", "));
            }

            // Try to create via config
            let config_path = std::path::PathBuf::from("ccswarm.json");
            if config_path.exists() {
                let mut config = CcswarmConfig::from_file(config_path.clone())
                    .await
                    .context("Failed to load config")?;

                // Add agent to config
                let agent_config = crate::config::AgentConfig {
                    specialization: role.clone(),
                    worktree: format!(".worktrees/{}", name),
                    branch: format!("agent/{}", name),
                    claude_config: crate::config::ClaudeConfig::for_agent(&role_lower),
                    claude_md_template: String::new(),
                };

                config.agents.insert(name.clone(), agent_config);
                config
                    .to_file(config_path)
                    .await
                    .context("Failed to save config")?;

                println!("   ✅ Agent configuration saved");
            } else {
                println!(
                    "   ℹ️  No ccswarm.json found. Run 'ccswarm init' first for persistent config"
                );
            }

            println!("✅ Subagent '{}' created successfully", name.bright_green());
            Ok(())
        }
        SubagentCommand::List { detailed } => {
            // Load config to list agents
            let config_path = std::path::PathBuf::from("ccswarm.json");
            let config = if config_path.exists() {
                CcswarmConfig::from_file(config_path)
                    .await
                    .unwrap_or_default()
            } else {
                CcswarmConfig::default()
            };

            println!("{}", "Available subagents:".bright_cyan());
            println!();

            if config.agents.is_empty() {
                // Show default roles when no config exists
                let default_agents = [
                    ("frontend-specialist", "Frontend", "React, Vue, UI/UX, CSS"),
                    (
                        "backend-specialist",
                        "Backend",
                        "APIs, databases, server logic",
                    ),
                    (
                        "devops-specialist",
                        "DevOps",
                        "Docker, CI/CD, infrastructure",
                    ),
                    ("qa-specialist", "QA", "Testing, quality assurance"),
                ];

                for (name, role, desc) in &default_agents {
                    println!(
                        "  {} ({}) - {}",
                        name.bright_green(),
                        role.bright_blue(),
                        desc
                    );
                }

                if detailed {
                    println!();
                    println!("{}", "Note: Using default agent definitions".italic());
                    println!(
                        "{}",
                        "Run 'ccswarm init' to create a project configuration".italic()
                    );
                }
            } else {
                for (name, agent_config) in &config.agents {
                    println!(
                        "  {} ({})",
                        name.bright_green(),
                        agent_config.specialization.bright_blue()
                    );

                    if detailed {
                        println!("    Worktree: {}", agent_config.worktree);
                        println!("    Branch: {}", agent_config.branch);
                        println!("    Model: {}", agent_config.claude_config.model);
                        println!("    Output: {:?}", agent_config.claude_config.output_format);
                        println!();
                    }
                }
            }

            Ok(())
        }
        SubagentCommand::Delegate { subagent, task } => {
            println!("🎯 Delegating task to '{}'", subagent.bright_cyan());
            println!("   Task: {}", task);

            // Create task and use delegation engine to validate
            let mut engine = MasterDelegationEngine::new(DelegationStrategy::Hybrid);
            let task_obj = Task::new(
                uuid::Uuid::new_v4().to_string(),
                task.clone(),
                Priority::Medium,
                TaskType::Development,
            );

            match engine.delegate_task(task_obj.clone()) {
                Ok(decision) => {
                    let recommended = decision.target_agent.name();
                    println!(
                        "   Recommended agent: {} (confidence: {:.0}%)",
                        recommended.bright_green(),
                        decision.confidence * 100.0
                    );

                    // Check if requested subagent matches recommendation
                    if !subagent
                        .to_lowercase()
                        .contains(&recommended.to_lowercase())
                    {
                        println!(
                            "   ℹ️  Note: '{}' was specified, but '{}' may be better suited",
                            subagent, recommended
                        );
                    }

                    // Try to add to execution engine if available
                    let config_path = std::path::PathBuf::from("ccswarm.json");
                    if config_path.exists()
                        && let Ok(config) = CcswarmConfig::from_file(config_path).await
                        && let Ok(engine) = ExecutionEngine::new(&config).await
                    {
                        let assigned = task_obj.assign_to(subagent.clone());
                        let task_id = engine.get_executor().add_task(assigned).await;
                        println!("   📋 Task queued: {}", task_id.bright_cyan());
                    }

                    println!("   ✅ Delegation complete");
                }
                Err(e) => {
                    println!("   ⚠️ Delegation analysis failed: {}", e);
                    println!("   Task created for manual assignment");
                }
            }

            Ok(())
        }
        SubagentCommand::Status { name } => {
            println!("{}", format!("Subagent: {}", name).bright_cyan());
            println!();

            // Try to get status from config
            let config_path = std::path::PathBuf::from("ccswarm.json");
            if config_path.exists() {
                if let Ok(config) = CcswarmConfig::from_file(config_path).await {
                    if let Some(agent_config) = config.agents.get(&name) {
                        println!("  Role: {}", agent_config.specialization.bright_green());
                        println!("  Worktree: {}", agent_config.worktree);
                        println!("  Branch: {}", agent_config.branch);
                        println!("  Model: {}", agent_config.claude_config.model);

                        // Check if worktree exists
                        let worktree_path = std::path::Path::new(&agent_config.worktree);
                        let status = if worktree_path.exists() {
                            "Active (worktree exists)".bright_green()
                        } else {
                            "Inactive (no worktree)".bright_yellow()
                        };
                        println!("  Status: {}", status);

                        // Check execution stats if engine is available
                        if let Ok(engine) = ExecutionEngine::new(&config).await {
                            let history = engine.get_executor().get_execution_history(None).await;
                            let agent_tasks: Vec<_> = history
                                .iter()
                                .filter(|r| {
                                    r.agent_used.as_deref().is_some_and(|a| a.contains(&name))
                                })
                                .collect();

                            if !agent_tasks.is_empty() {
                                let completed = agent_tasks.iter().filter(|r| r.success).count();
                                let failed = agent_tasks.len() - completed;
                                println!("  Tasks completed: {}", completed);
                                println!("  Tasks failed: {}", failed);
                            }
                        }
                    } else {
                        println!(
                            "  {} Agent '{}' not found in configuration",
                            "⚠️".bright_yellow(),
                            name
                        );
                        println!(
                            "  Available agents: {}",
                            config
                                .agents
                                .keys()
                                .map(|k| k.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                } else {
                    println!("  Failed to load configuration");
                }
            } else {
                println!("  No ccswarm.json found. Run 'ccswarm init' to create a project");
            }

            Ok(())
        }
    }
}
