/// Backend Agent Status Reporting Module
///
/// This module implements status reporting functionality specific to backend agents,
/// providing detailed information about API endpoints, database connections,
/// and server health.
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::agent::ClaudeCodeAgent;
use crate::coordination::StatusTracker;
use crate::identity::AgentRole;

/// Backend-specific status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    /// API endpoint health status
    pub api_endpoints: HashMap<String, EndpointStatus>,
    /// Database connection status
    pub database_status: DatabaseStatus,
    /// Server metrics
    pub server_metrics: ServerMetrics,
    /// Active services
    pub active_services: Vec<ServiceInfo>,
    /// Recent API calls
    pub recent_api_calls: Vec<ApiCallInfo>,
}

/// Status of an API endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStatus {
    pub path: String,
    pub method: String,
    pub is_healthy: bool,
    pub response_time_ms: Option<f64>,
    pub last_checked: chrono::DateTime<Utc>,
}

/// Database connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub is_connected: bool,
    pub database_type: String,
    pub connection_pool_size: usize,
    pub active_connections: usize,
    pub last_migration: Option<String>,
}

/// Server performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub request_count: u64,
    pub error_rate: f64,
}

/// Information about active services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub port: Option<u16>,
    pub dependencies: Vec<String>,
}

/// Recent API call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallInfo {
    pub timestamp: chrono::DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub status_code: u16,
    pub response_time_ms: f64,
}

impl ClaudeCodeAgent {
    /// Generate backend-specific status report
    pub async fn generate_backend_status(&self) -> Result<BackendStatus> {
        // Check if this is a backend agent
        if !matches!(self.identity.specialization, AgentRole::Backend { .. }) {
            return Err(anyhow::anyhow!(
                "This method is only available for backend agents"
            ));
        }

        // Simulate gathering backend-specific metrics
        let api_endpoints = self.check_api_endpoints().await?;
        let database_status = self.check_database_status().await?;
        let server_metrics = self.gather_server_metrics().await?;
        let active_services = self.list_active_services().await?;
        let recent_api_calls = self.get_recent_api_calls().await?;

        Ok(BackendStatus {
            api_endpoints,
            database_status,
            server_metrics,
            active_services,
            recent_api_calls,
        })
    }

    /// Report detailed backend status to coordination system
    pub async fn report_backend_status(&self) -> Result<()> {
        let backend_status = self.generate_backend_status().await?;

        let detailed_status = json!({
            "agent_id": self.identity.agent_id,
            "role": "Backend",
            "status": self.status,
            "timestamp": Utc::now(),
            "worktree": self.worktree_path.to_string_lossy(),
            "branch": self.branch_name,
            "backend_specific": {
                "api_health": backend_status.api_endpoints.values()
                    .filter(|e| e.is_healthy)
                    .count() as f64 / backend_status.api_endpoints.len().max(1) as f64,
                "database": backend_status.database_status,
                "server": backend_status.server_metrics,
                "services": backend_status.active_services,
                "recent_activity": backend_status.recent_api_calls.len(),
            },
            "capabilities": self.identity.specialization.technologies(),
            "current_task": self.current_task.as_ref().map(|t| json!({
                "id": t.id,
                "description": t.description,
                "type": t.task_type,
                "priority": t.priority,
            })),
            "task_history_summary": json!({
                "total": self.task_history.len(),
                "successful": self.task_history.iter().filter(|(_, r)| r.success).count(),
                "failed": self.task_history.iter().filter(|(_, r)| !r.success).count(),
            }),
        });

        // Write to status tracker
        let status_tracker = StatusTracker::new().await?;
        status_tracker
            .update_status(
                &self.identity.agent_id,
                &self.status,
                detailed_status.clone(),
            )
            .await?;

        // Also send via coordination bus if needed
        tracing::info!(
            "Backend agent {} reported detailed status",
            self.identity.agent_id
        );

        Ok(())
    }

    /// Check API endpoints health
    ///
    /// Returns empty map when no backend agent is connected.
    /// Will be populated when the backend agent reports real endpoint data.
    async fn check_api_endpoints(&self) -> Result<HashMap<String, EndpointStatus>> {
        // No data available until a backend agent reports real endpoint health
        Ok(HashMap::new())
    }

    /// Check database connection status
    ///
    /// Returns disconnected status when no backend agent is connected.
    async fn check_database_status(&self) -> Result<DatabaseStatus> {
        Ok(DatabaseStatus {
            is_connected: false,
            database_type: "unknown".to_string(),
            connection_pool_size: 0,
            active_connections: 0,
            last_migration: None,
        })
    }

    /// Gather server performance metrics
    ///
    /// Returns zero metrics when no backend agent is connected.
    async fn gather_server_metrics(&self) -> Result<ServerMetrics> {
        Ok(ServerMetrics {
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            request_count: 0,
            error_rate: 0.0,
        })
    }

    /// List active backend services
    ///
    /// Returns empty list when no backend agent is connected.
    async fn list_active_services(&self) -> Result<Vec<ServiceInfo>> {
        Ok(Vec::new())
    }

    /// Get recent API calls
    ///
    /// Returns empty list when no backend agent is connected.
    async fn get_recent_api_calls(&self) -> Result<Vec<ApiCallInfo>> {
        Ok(Vec::new())
    }
}

/// Extension trait for backend-specific status operations
pub trait BackendStatusExt {
    /// Get a formatted status summary for display
    fn format_backend_status(&self) -> String;

    /// Check if the backend is healthy
    fn is_backend_healthy(&self) -> bool;
}

impl BackendStatusExt for BackendStatus {
    fn format_backend_status(&self) -> String {
        let api_health = self.api_endpoints.values().filter(|e| e.is_healthy).count() as f64
            / self.api_endpoints.len().max(1) as f64
            * 100.0;

        format!(
            "Backend Status Summary:\n\
             ├─ API Health: {:.1}% ({}/{} endpoints)\n\
             ├─ Database: {} ({})\n\
             ├─ Server: {:.1}MB RAM, {:.1}% CPU\n\
             ├─ Active Services: {}\n\
             └─ Recent Activity: {} API calls",
            api_health,
            self.api_endpoints.values().filter(|e| e.is_healthy).count(),
            self.api_endpoints.len(),
            if self.database_status.is_connected {
                "Connected"
            } else {
                "Disconnected"
            },
            self.database_status.database_type,
            self.server_metrics.memory_usage_mb,
            self.server_metrics.cpu_usage_percent,
            self.active_services.len(),
            self.recent_api_calls.len()
        )
    }

    fn is_backend_healthy(&self) -> bool {
        // Backend is healthy if:
        // - Database is connected
        // - At least 80% of API endpoints are healthy
        // - Error rate is below 5%
        let api_health_ratio = self.api_endpoints.values().filter(|e| e.is_healthy).count() as f64
            / self.api_endpoints.len().max(1) as f64;

        self.database_status.is_connected
            && api_health_ratio >= 0.8
            && self.server_metrics.error_rate < 0.05
    }
}
