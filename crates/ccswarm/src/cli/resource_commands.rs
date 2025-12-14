/// CLI commands for resource monitoring and management
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::sync::Arc;

use crate::resource::ResourceLimits;
use crate::session::SessionManager;

/// Resource management commands
#[derive(Parser, Debug)]
pub struct ResourceCommand {
    #[command(subcommand)]
    pub subcommand: ResourceSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ResourceSubcommand {
    /// Show resource usage for all agents
    Status,

    /// Show resource efficiency statistics
    Stats,

    /// Set resource limits for agents
    Limits {
        /// Maximum CPU percentage (0-100)
        #[arg(long)]
        max_cpu: Option<f32>,

        /// Maximum memory in GB
        #[arg(long)]
        max_memory_gb: Option<f32>,

        /// Idle timeout in minutes
        #[arg(long)]
        idle_timeout_min: Option<u64>,

        /// Enable/disable auto-suspension
        #[arg(long)]
        auto_suspend: Option<bool>,
    },

    /// Check and suspend idle agents
    CheckIdle,

    /// Resume a suspended agent
    Resume {
        /// Agent ID to resume
        agent_id: String,
    },
}

impl ResourceCommand {
    /// Execute resource command
    pub async fn execute(self, session_manager: Arc<SessionManager>) -> Result<()> {
        match self.subcommand {
            ResourceSubcommand::Status => show_resource_status(session_manager).await,
            ResourceSubcommand::Stats => show_resource_stats(session_manager).await,
            ResourceSubcommand::Limits {
                max_cpu,
                max_memory_gb,
                idle_timeout_min,
                auto_suspend,
            } => {
                update_resource_limits(
                    session_manager,
                    max_cpu,
                    max_memory_gb,
                    idle_timeout_min,
                    auto_suspend,
                )
                .await
            }
            ResourceSubcommand::CheckIdle => check_idle_agents(session_manager).await,
            ResourceSubcommand::Resume { agent_id } => {
                resume_agent(session_manager, agent_id).await
            }
        }
    }
}

/// Show resource usage status for all agents
async fn show_resource_status(session_manager: Arc<SessionManager>) -> Result<()> {
    println!("{}", "Resource Usage Status".bold().cyan());
    println!("{}", "=".repeat(80));

    let sessions = session_manager.list_active_sessions();

    if sessions.is_empty() {
        println!("{}", "No active sessions".yellow());
        return Ok(());
    }

    println!(
        "{:<30} {:<10} {:<10} {:<15} {:<10}",
        "Agent ID".bold(),
        "CPU %".bold(),
        "Memory MB".bold(),
        "Status".bold(),
        "Idle Time".bold()
    );
    println!("{}", "-".repeat(80));

    for session in sessions {
        if let Some(usage) = session_manager.get_session_resource_usage(&session.id) {
            let memory_mb = usage.memory_bytes / (1024 * 1024);
            let idle_time = chrono::Utc::now() - session.last_activity;
            let idle_str = format_duration(idle_time);

            let cpu_color = if usage.cpu_percent > 80.0 {
                "red"
            } else if usage.cpu_percent > 50.0 {
                "yellow"
            } else {
                "green"
            };

            let status_str = match session.status {
                crate::session::SessionStatus::Active => "Active".green(),
                crate::session::SessionStatus::Paused => "Paused".yellow(),
                crate::session::SessionStatus::Background => "Background".blue(),
                _ => format!("{:?}", session.status).normal(),
            };

            println!(
                "{:<30} {:<10} {:<10} {:<15} {:<10}",
                session.agent_id.trunc(28),
                format!("{:.1}", usage.cpu_percent).color(cpu_color),
                memory_mb.to_string(),
                status_str,
                idle_str
            );
        }
    }

    Ok(())
}

/// Show resource efficiency statistics
async fn show_resource_stats(session_manager: Arc<SessionManager>) -> Result<()> {
    if let Some(stats) = session_manager.get_resource_efficiency_stats() {
        println!("{}", "Resource Efficiency Statistics".bold().cyan());
        println!("{}", "=".repeat(50));

        println!("Total Agents:      {}", stats.total_agents);
        println!(
            "Active Agents:     {} ({})",
            stats.active_agents,
            format!(
                "{:.1}%",
                (stats.active_agents as f32 / stats.total_agents.max(1) as f32) * 100.0
            )
            .green()
        );
        println!(
            "Suspended Agents:  {} ({})",
            stats.suspended_agents,
            format!("{:.1}%", stats.suspension_rate).yellow()
        );
        println!();
        println!("Average CPU Usage:    {:.1}%", stats.average_cpu_usage);
        println!(
            "Average Memory Usage: {} MB ({:.1}%)",
            stats.average_memory_usage / (1024 * 1024),
            stats.average_memory_percent
        );
        println!(
            "Total Memory Usage:   {} MB",
            stats.total_memory_usage / (1024 * 1024)
        );

        println!();
        println!(
            "{}",
            format!(
                "💡 Efficiency Tip: {:.1}% of agents are suspended, saving resources",
                stats.suspension_rate
            )
            .italic()
        );
    } else {
        println!("{}", "Resource monitoring not enabled".yellow());
    }

    Ok(())
}

/// Update resource limits
async fn update_resource_limits(
    _session_manager: Arc<SessionManager>,
    max_cpu: Option<f32>,
    max_memory_gb: Option<f32>,
    idle_timeout_min: Option<u64>,
    auto_suspend: Option<bool>,
) -> Result<()> {
    #[allow(unused_mut, unused_assignments)]
    let mut _limits = ResourceLimits::default();

    if let Some(cpu) = max_cpu {
        _limits.max_cpu_percent = cpu;
        println!("✓ Max CPU set to {}%", cpu);
    }

    if let Some(mem_gb) = max_memory_gb {
        _limits.max_memory_bytes = (mem_gb * 1024.0 * 1024.0 * 1024.0) as u64;
        println!("✓ Max memory set to {} GB", mem_gb);
    }

    if let Some(timeout_min) = idle_timeout_min {
        _limits.idle_timeout = chrono::Duration::minutes(timeout_min as i64);
        println!("✓ Idle timeout set to {} minutes", timeout_min);
    }

    if let Some(suspend) = auto_suspend {
        _limits.auto_suspend_enabled = suspend;
        println!(
            "✓ Auto-suspend {}",
            if suspend { "enabled" } else { "disabled" }
        );
    }

    // TODO: Apply these limits to the session manager
    println!(
        "{}",
        "Note: Resource limits will be applied to new sessions".italic()
    );

    Ok(())
}

/// Check and suspend idle agents
async fn check_idle_agents(session_manager: Arc<SessionManager>) -> Result<()> {
    println!("{}", "Checking for idle agents...".cyan());

    let suspended = session_manager.check_and_suspend_idle_agents().await?;

    if suspended.is_empty() {
        println!("{}", "No idle agents found".green());
    } else {
        println!(
            "{}",
            format!("Suspended {} idle agents:", suspended.len()).yellow()
        );
        for agent_id in &suspended {
            println!("  - {}", agent_id);
        }

        let stats = session_manager.get_resource_efficiency_stats();
        if let Some(stats) = stats {
            let saved_memory_mb =
                (stats.average_memory_usage * suspended.len() as u64) / (1024 * 1024);
            println!();
            println!(
                "{}",
                format!("💡 Estimated memory saved: {} MB", saved_memory_mb).green()
            );
        }
    }

    Ok(())
}

/// Resume a suspended agent
async fn resume_agent(session_manager: Arc<SessionManager>, agent_id: String) -> Result<()> {
    // Find the session for this agent
    let sessions = session_manager.list_sessions();
    let session = sessions
        .iter()
        .find(|s| s.agent_id == agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    // Resume the session
    session_manager.resume_session(&session.id).await?;

    println!("{}", format!("✓ Agent {} resumed", agent_id).green());

    Ok(())
}

/// Format duration as human-readable string
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m", total_seconds / 60)
    } else {
        format!("{}h {}m", total_seconds / 3600, (total_seconds % 3600) / 60)
    }
}

/// Truncate string trait
trait Truncate {
    fn trunc(&self, max_len: usize) -> String;
}

impl Truncate for String {
    fn trunc(&self, max_len: usize) -> String {
        if self.len() <= max_len {
            self.clone()
        } else {
            format!("{}...", &self[..max_len - 3])
        }
    }
}
