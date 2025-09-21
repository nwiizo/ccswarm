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
    async fn check_api_endpoints(&self) -> Result<HashMap<String, EndpointStatus>> {
        // In a real implementation, this would check actual endpoints
        // For now, return simulated data
        let mut endpoints = HashMap::new();

        let common_endpoints = vec![
            ("/api/health", "GET"),
            ("/api/users", "GET"),
            ("/api/users", "POST"),
            ("/api/auth/login", "POST"),
            ("/api/auth/logout", "POST"),
        ];

        for (path, method) in common_endpoints {
            endpoints.insert(
                format!("{}-{}", method, path),
                EndpointStatus {
                    path: path.to_string(),
                    method: method.to_string(),
                    is_healthy: true,
                    response_time_ms: Some(rand::random::<f64>() * 100.0),
                    last_checked: Utc::now(),
                },
            );
        }

        Ok(endpoints)
    }

    /// Check database connection status
    async fn check_database_status(&self) -> Result<DatabaseStatus> {
        // Simulated database status
        Ok(DatabaseStatus {
            is_connected: true,
            database_type: "PostgreSQL".to_string(),
            connection_pool_size: 10,
            active_connections: 3,
            last_migration: Some("20240315_add_user_table".to_string()),
        })
    }

    /// Gather server performance metrics
    async fn gather_server_metrics(&self) -> Result<ServerMetrics> {
        // Simulated server metrics
        Ok(ServerMetrics {
            uptime_seconds: 86400, // 1 day
            memory_usage_mb: 256.5,
            cpu_usage_percent: 15.3,
            request_count: 1234,
            error_rate: 0.02, // 2% error rate
        })
    }

    /// List active backend services
    async fn list_active_services(&self) -> Result<Vec<ServiceInfo>> {
        // Simulated service list
        Ok(vec![
            ServiceInfo {
                name: "express-server".to_string(),
                status: "running".to_string(),
                port: Some(3000),
                dependencies: vec!["postgresql".to_string(), "redis".to_string()],
            },
            ServiceInfo {
                name: "websocket-server".to_string(),
                status: "running".to_string(),
                port: Some(3001),
                dependencies: vec!["redis".to_string()],
            },
            ServiceInfo {
                name: "background-worker".to_string(),
                status: "running".to_string(),
                port: None,
                dependencies: vec!["postgresql".to_string(), "redis".to_string()],
            },
        ])
    }

    /// Get recent API calls
    async fn get_recent_api_calls(&self) -> Result<Vec<ApiCallInfo>> {
        // Return last 10 API calls (simulated)
        let mut calls = Vec::new();
        for i in 0..10 {
            calls.push(ApiCallInfo {
                timestamp: Utc::now() - chrono::Duration::minutes(i as i64),
                endpoint: match i % 4 {
                    0 => "/api/users".to_string(),
                    1 => "/api/auth/login".to_string(),
                    2 => "/api/products".to_string(),
                    _ => "/api/health".to_string(),
                },
                method: if i % 3 == 0 { "POST" } else { "GET" }.to_string(),
                status_code: if i == 7 { 500 } else { 200 },
                response_time_ms: rand::random::<f64>() * 200.0,
            });
        }
        Ok(calls)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClaudeConfig;
    use crate::identity::default_backend_role;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_backend_status_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ClaudeConfig::for_agent("backend");

        let agent = ClaudeCodeAgent::new(default_backend_role(), temp_dir.path(), "test", config)
            .await
            .unwrap();

        let status = agent.generate_backend_status().await.unwrap();

        assert!(!status.api_endpoints.is_empty());
        assert!(status.database_status.is_connected);
        assert!(!status.active_services.is_empty());
    }

    #[test]
    fn test_backend_status_formatting() {
        let status = BackendStatus {
            api_endpoints: HashMap::new(),
            database_status: DatabaseStatus {
                is_connected: true,
                database_type: "PostgreSQL".to_string(),
                connection_pool_size: 10,
                active_connections: 3,
                last_migration: None,
            },
            server_metrics: ServerMetrics {
                uptime_seconds: 3600,
                memory_usage_mb: 512.0,
                cpu_usage_percent: 25.0,
                request_count: 1000,
                error_rate: 0.01,
            },
            active_services: vec![],
            recent_api_calls: vec![],
        };

        let formatted = status.format_backend_status();
        assert!(formatted.contains("Backend Status Summary"));
        assert!(formatted.contains("PostgreSQL"));
    }
}
