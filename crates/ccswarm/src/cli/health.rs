//! Health check commands for ccswarm system monitoring

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::time::Instant;

use crate::coordination::StatusTracker;
use crate::monitoring::MonitoringSystem;

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Everything is working perfectly
    Healthy,
    /// Minor issues that don't affect functionality
    Warning,
    /// Major issues affecting functionality
    Critical,
    /// Component is not responding
    Down,
}

impl HealthStatus {
    pub fn as_emoji(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️ ",
            HealthStatus::Critical => "🔴",
            HealthStatus::Down => "❌",
        }
    }

    pub fn as_colored_text(&self) -> String {
        match self {
            HealthStatus::Healthy => "HEALTHY".green().to_string(),
            HealthStatus::Warning => "WARNING".yellow().to_string(),
            HealthStatus::Critical => "CRITICAL".red().to_string(),
            HealthStatus::Down => "DOWN".red().bold().to_string(),
        }
    }
}

/// Health check result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub response_time_ms: u64,
}

impl fmt::Display for HealthCheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} - {} ({}ms)",
            self.status.as_emoji(),
            self.component,
            self.message,
            self.response_time_ms
        )
    }
}

/// System-wide health report
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub active_tasks: usize,
    pub session_count: usize,
}

impl SystemHealthReport {
    pub fn print_summary(&self) {
        println!("\n{}", "═══ ccswarm Health Report ═══".bold());
        println!(
            "Timestamp: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!(
            "Overall Status: {} {}",
            self.overall_status.as_emoji(),
            self.overall_status.as_colored_text()
        );
        println!();

        // Agent summary
        let agent_health_pct = if self.total_agents > 0 {
            (self.healthy_agents as f64 / self.total_agents as f64 * 100.0) as u32
        } else {
            0
        };
        println!(
            "Agents: {}/{} healthy ({}%)",
            self.healthy_agents, self.total_agents, agent_health_pct
        );
        println!("Active Tasks: {}", self.active_tasks);
        println!("AI Sessions: {}", self.session_count);
        println!();

        // Component checks
        println!("{}", "Component Health Checks:".bold());
        for check in &self.checks {
            println!("  {}", check);
            if let Some(details) = &check.details {
                if let Some(obj) = details.as_object() {
                    for (key, value) in obj {
                        println!("    - {}: {}", key.dimmed(), value);
                    }
                }
            }
        }
    }

    pub fn print_detailed(&self) {
        self.print_summary();

        println!("\n{}", "Detailed Component Information:".bold());

        // Group checks by component type
        let agent_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.component.starts_with("agent:"))
            .collect();
        let session_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.component.starts_with("session:"))
            .collect();
        let system_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| !c.component.starts_with("agent:") && !c.component.starts_with("session:"))
            .collect();

        if !agent_checks.is_empty() {
            println!("\n{}:", "Agent Health".underline());
            for check in agent_checks {
                println!("  {} {}", check.status.as_emoji(), check.component);
                println!("    Status: {}", check.status.as_colored_text());
                println!("    Message: {}", check.message);
                if let Some(details) = &check.details {
                    println!(
                        "    Details: {}",
                        serde_json::to_string_pretty(details)
                            .unwrap_or_else(|e| format!("Failed to serialize details: {}", e))
                    );
                }
            }
        }

        if !session_checks.is_empty() {
            println!("\n{}:", "AI-Session Health".underline());
            for check in session_checks {
                println!("  {} {}", check.status.as_emoji(), check.component);
                println!("    Status: {}", check.status.as_colored_text());
                println!("    Message: {}", check.message);
                if let Some(details) = &check.details {
                    println!(
                        "    Details: {}",
                        serde_json::to_string_pretty(details)
                            .unwrap_or_else(|e| format!("Failed to serialize details: {}", e))
                    );
                }
            }
        }

        if !system_checks.is_empty() {
            println!("\n{}:", "System Health".underline());
            for check in system_checks {
                println!("  {} {}", check.status.as_emoji(), check.component);
                println!("    Status: {}", check.status.as_colored_text());
                println!("    Message: {}", check.message);
                if let Some(details) = &check.details {
                    println!(
                        "    Details: {}",
                        serde_json::to_string_pretty(details)
                            .unwrap_or_else(|e| format!("Failed to serialize details: {}", e))
                    );
                }
            }
        }
    }
}

/// Health checker for the ccswarm system
pub struct HealthChecker {
    status_tracker: StatusTracker,
    monitoring_system: Option<MonitoringSystem>,
}

impl HealthChecker {
    pub fn new(status_tracker: StatusTracker) -> Self {
        Self {
            status_tracker,
            monitoring_system: None,
        }
    }

    /// Perform all health checks
    pub async fn check_all(&self) -> Result<SystemHealthReport> {
        let mut checks = Vec::new();
        let _start_time = Instant::now();

        // Check orchestrator status
        checks.push(self.check_orchestrator().await);

        // Check all agents
        let agent_checks = self.check_agents().await?;
        let healthy_agents = agent_checks
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();
        checks.extend(agent_checks);

        // Check AI sessions
        let session_checks = self.check_sessions().await?;
        let session_count = session_checks.len();
        checks.extend(session_checks);

        // Check monitoring system
        if self.monitoring_system.is_some() {
            checks.push(self.check_monitoring_system().await);
        }

        // Check coordination bus
        checks.push(self.check_coordination_bus().await);

        // Determine overall status
        let overall_status = if checks.iter().any(|c| matches!(c.status, HealthStatus::Down | HealthStatus::Critical)) {
            HealthStatus::Critical
        } else if checks.iter().any(|c| c.status == HealthStatus::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        // Get active task count
        let active_tasks = self.get_active_task_count().await;

        Ok(SystemHealthReport {
            timestamp: chrono::Utc::now(),
            overall_status,
            checks,
            total_agents: self.get_total_agent_count().await,
            healthy_agents,
            active_tasks,
            session_count,
        })
    }

    /// Check only agent health
    pub async fn check_agents_only(&self) -> Result<Vec<HealthCheckResult>> {
        self.check_agents().await
    }

    /// Check only session health
    pub async fn check_sessions_only(&self) -> Result<Vec<HealthCheckResult>> {
        self.check_sessions().await
    }

    // Private helper methods

    async fn check_orchestrator(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Check if orchestrator is running by checking coordination files
        match tokio::fs::read_to_string(".ccswarm/coordination/status.json").await {
            Ok(content) => {
                if let Ok(status) = serde_json::from_str::<serde_json::Value>(&content) {
                    let last_update = status
                        .get("last_update")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());

                    let status = if let Some(last_update) = last_update {
                        let age = chrono::Utc::now() - last_update.with_timezone(&chrono::Utc);
                        if age.num_seconds() < 60 {
                            HealthStatus::Healthy
                        } else if age.num_seconds() < 300 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Critical
                        }
                    } else {
                        HealthStatus::Warning
                    };

                    HealthCheckResult {
                        component: "orchestrator".to_string(),
                        status,
                        message: match status {
                            HealthStatus::Healthy => "Orchestrator is running normally".to_string(),
                            HealthStatus::Warning => "Orchestrator status is stale".to_string(),
                            _ => "Orchestrator may be unresponsive".to_string(),
                        },
                        details: Some(serde_json::json!({"status": status.as_colored_text()})),
                        response_time_ms: start.elapsed().as_millis() as u64,
                    }
                } else {
                    HealthCheckResult {
                        component: "orchestrator".to_string(),
                        status: HealthStatus::Down,
                        message: "Invalid status file".to_string(),
                        details: None,
                        response_time_ms: start.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(_) => HealthCheckResult {
                component: "orchestrator".to_string(),
                status: HealthStatus::Down,
                message: "Orchestrator not running".to_string(),
                details: None,
                response_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    async fn check_agents(&self) -> Result<Vec<HealthCheckResult>> {
        let mut results = Vec::new();

        // Get all agent statuses
        let statuses = self.status_tracker.get_all_statuses().await?;

        for status in statuses.into_iter() {
            let agent_name = status
                .get("agent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let start = Instant::now();

            // Check agent responsiveness
            let timestamp_str = status
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let last_update = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let age = chrono::Utc::now() - last_update;
            let health_status = if age.num_seconds() < 30 {
                HealthStatus::Healthy
            } else if age.num_seconds() < 120 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            };

            let mut details = serde_json::json!({
                "status": status.get("status").unwrap_or(&serde_json::Value::Null),
                "last_update": timestamp_str,
                "age_seconds": age.num_seconds(),
            });

            // Add agent-specific details
            if let Some(agent_details) = status.get("additional_info") {
                details["agent_details"] = agent_details.clone();
            }

            results.push(HealthCheckResult {
                component: format!("agent:{}", agent_name),
                status: health_status,
                message: match health_status {
                    HealthStatus::Healthy => format!("{} agent is responsive", agent_name),
                    HealthStatus::Warning => format!("{} agent response is delayed", agent_name),
                    _ => format!("{} agent is unresponsive", agent_name),
                },
                details: Some(details),
                response_time_ms: start.elapsed().as_millis() as u64,
            });
        }

        Ok(results)
    }

    async fn check_sessions(&self) -> Result<Vec<HealthCheckResult>> {
        let mut results = Vec::new();

        // Check for active sessions in .ccswarm/sessions directory
        let sessions_dir = std::path::Path::new(".ccswarm/sessions");
        if sessions_dir.exists() {
            let mut entries = tokio::fs::read_dir(sessions_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    let session_name = entry.file_name().to_string_lossy().to_string();
                    let start = Instant::now();

                    // Check session state file
                    let state_file = entry.path().join("state.json");
                    let health_status = if state_file.exists() {
                        match tokio::fs::read_to_string(&state_file).await {
                            Ok(content) => {
                                if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                                    HealthStatus::Healthy
                                } else {
                                    HealthStatus::Warning
                                }
                            }
                            Err(_) => HealthStatus::Warning,
                        }
                    } else {
                        HealthStatus::Critical
                    };

                    results.push(HealthCheckResult {
                        component: format!("session:{}", session_name),
                        status: health_status,
                        message: match health_status {
                            HealthStatus::Healthy => "Session is active and healthy".to_string(),
                            HealthStatus::Warning => "Session state is unclear".to_string(),
                            _ => "Session may be corrupted".to_string(),
                        },
                        details: Some(serde_json::json!({
                            "session_name": session_name,
                            "state_file_exists": state_file.exists(),
                        })),
                        response_time_ms: start.elapsed().as_millis() as u64,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn check_monitoring_system(&self) -> HealthCheckResult {
        let start = Instant::now();

        if let Some(monitoring) = &self.monitoring_system {
            let stats = monitoring.get_stats();

            HealthCheckResult {
                component: "monitoring_system".to_string(),
                status: HealthStatus::Healthy,
                message: "Monitoring system is active".to_string(),
                details: Some(serde_json::json!({
                    "total_entries": stats.total_entries,
                    "entries_per_agent": stats.entries_per_agent,
                    "entries_per_type": stats.entries_per_type,
                })),
                response_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            HealthCheckResult {
                component: "monitoring_system".to_string(),
                status: HealthStatus::Down,
                message: "Monitoring system not initialized".to_string(),
                details: None,
                response_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    async fn check_coordination_bus(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Check if coordination directory exists and is writable
        let coord_dir = std::path::Path::new(".ccswarm/coordination");
        if coord_dir.exists() && coord_dir.is_dir() {
            // Try to write a test file
            let test_file = coord_dir.join(".health_check");
            match tokio::fs::write(&test_file, b"health_check").await {
                Ok(_) => {
                    // Clean up test file
                    let _ = tokio::fs::remove_file(&test_file).await;

                    HealthCheckResult {
                        component: "coordination_bus".to_string(),
                        status: HealthStatus::Healthy,
                        message: "Coordination bus is operational".to_string(),
                        details: None,
                        response_time_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => HealthCheckResult {
                    component: "coordination_bus".to_string(),
                    status: HealthStatus::Critical,
                    message: format!("Coordination bus write failed: {}", e),
                    details: None,
                    response_time_ms: start.elapsed().as_millis() as u64,
                },
            }
        } else {
            HealthCheckResult {
                component: "coordination_bus".to_string(),
                status: HealthStatus::Down,
                message: "Coordination directory not found".to_string(),
                details: None,
                response_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    async fn get_active_task_count(&self) -> usize {
        // Read from task queue file if exists
        if let Ok(content) =
            tokio::fs::read_to_string(".ccswarm/coordination/task_queue.json").await
        {
            if let Ok(queue) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tasks) = queue.get("pending_tasks").and_then(|v| v.as_array()) {
                    return tasks.len();
                }
            }
        }
        0
    }

    async fn get_total_agent_count(&self) -> usize {
        self.status_tracker
            .get_all_statuses()
            .await
            .map(|statuses| statuses.len())
            .unwrap_or(0)
    }
}

/// Run health check diagnostics
pub async fn run_diagnostics(repo_path: &std::path::Path) -> Result<()> {
    println!("{}", "Running ccswarm diagnostics...".bold().cyan());

    // Check basic requirements
    println!("\n{}", "System Requirements:".underline());

    // Check Rust version
    let rust_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Not found".to_string());
    println!("  Rust: {}", rust_version);

    // Check Git version
    let git_version = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Not found".to_string());
    println!("  Git: {}", git_version);

    // Check environment variables
    println!("\n{}", "Environment Variables:".underline());
    let env_vars = vec![
        (
            "ANTHROPIC_API_KEY",
            std::env::var("ANTHROPIC_API_KEY").is_ok(),
        ),
        ("OPENAI_API_KEY", std::env::var("OPENAI_API_KEY").is_ok()),
        ("RUST_LOG", std::env::var("RUST_LOG").is_ok()),
    ];

    for (var, exists) in env_vars {
        println!(
            "  {}: {}",
            var,
            if exists {
                "Set ✓".green().to_string()
            } else {
                "Not set".yellow().to_string()
            }
        );
    }

    // Check ccswarm directories
    println!("\n{}", "ccswarm Directories:".underline());
    let dirs = vec![
        ".ccswarm",
        ".ccswarm/coordination",
        ".ccswarm/sessions",
        ".ccswarm/agents",
        ".ccswarm/worktrees",
    ];

    for dir in dirs {
        let path = repo_path.join(dir);
        println!(
            "  {}: {}",
            dir,
            if path.exists() {
                "Exists ✓".green().to_string()
            } else {
                "Missing".red().to_string()
            }
        );
    }

    // Check for running processes
    println!("\n{}", "Process Check:".underline());

    // Try to connect to orchestrator
    let orchestrator_running = tokio::fs::read_to_string(".ccswarm/coordination/status.json")
        .await
        .is_ok();

    println!(
        "  Orchestrator: {}",
        if orchestrator_running {
            "Running ✓".green().to_string()
        } else {
            "Not running".yellow().to_string()
        }
    );

    Ok(())
}
