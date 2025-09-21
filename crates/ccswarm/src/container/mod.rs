//! Container management module for agent isolation
//!
//! This module provides Docker container support for running agents in isolated environments.
//! It offers complete process, network, and filesystem isolation compared to git worktrees.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

pub mod config;
pub mod docker;

pub use config::ContainerConfig;
pub use docker::DockerContainerProvider;

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Unique container ID
    pub id: String,
    /// Container name
    pub name: String,
    /// Current status (running, stopped, etc.)
    pub status: ContainerStatus,
    /// Container image
    pub image: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Volume mappings
    pub volumes: Vec<VolumeMapping>,
    /// Network configuration
    pub network: NetworkConfig,
    /// Resource limits
    pub resources: ResourceLimits,
}

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Removing,
    Error(String),
}

/// Volume mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    /// Host path
    pub host_path: String,
    /// Container path
    pub container_path: String,
    /// Read-only flag
    pub read_only: bool,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network mode (bridge, host, none, custom)
    pub mode: NetworkMode,
    /// Custom network name if applicable
    pub network_name: Option<String>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
}

/// Network mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Custom(String),
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Host port
    pub host_port: u16,
    /// Container port
    pub container_port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
}

/// Resource limits for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (number of CPUs)
    pub cpu_limit: Option<f64>,
    /// Memory limit in bytes
    pub memory_limit: Option<i64>,
    /// Memory + swap limit in bytes
    pub memory_swap_limit: Option<i64>,
    /// CPU shares (relative weight)
    pub cpu_shares: Option<i64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_limit: Some(1.0),                  // 1 CPU
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            memory_swap_limit: None,
            cpu_shares: Some(1024), // Default weight
        }
    }
}

/// Log entry from container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log message
    pub message: String,
    /// Stream type (stdout/stderr)
    pub stream: LogStream,
}

/// Log stream type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// Container provider trait for different container runtimes
#[async_trait]
pub trait ContainerProvider: Send + Sync {
    /// Create a new container
    async fn create_container(&self, name: &str, config: &ContainerConfig) -> Result<Container>;

    /// Start a container
    async fn start_container(&self, container_id: &str) -> Result<()>;

    /// Stop a container
    async fn stop_container(&self, container_id: &str) -> Result<()>;

    /// Remove a container
    async fn remove_container(&self, container_id: &str) -> Result<()>;

    /// Execute a command in a running container
    async fn exec_in_container(&self, container_id: &str, command: Vec<String>) -> Result<String>;

    /// Get container logs
    async fn get_logs(&self, container_id: &str, tail: Option<usize>) -> Result<Vec<LogEntry>>;

    /// Get container status
    async fn get_status(&self, container_id: &str) -> Result<ContainerStatus>;

    /// List all containers with optional filter
    async fn list_containers(&self, filter: Option<String>) -> Result<Vec<Container>>;

    /// Get container stats (CPU, memory usage)
    async fn get_stats(&self, container_id: &str) -> Result<ContainerStats>;

    /// Copy files to container
    async fn copy_to_container(
        &self,
        container_id: &str,
        src_path: &str,
        dest_path: &str,
    ) -> Result<()>;

    /// Copy files from container
    async fn copy_from_container(
        &self,
        container_id: &str,
        src_path: &str,
        dest_path: &str,
    ) -> Result<()>;
}

/// Container statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// Network I/O stats
    pub network_io: NetworkIO,
    /// Disk I/O stats
    pub disk_io: DiskIO,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
}

/// Disk I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIO {
    /// Bytes read
    pub read_bytes: u64,
    /// Bytes written
    pub write_bytes: u64,
}

/// Container manager for high-level operations
pub struct ContainerManager {
    provider: Box<dyn ContainerProvider>,
    containers: HashMap<String, Container>,
}

#[cfg(test)]
mod tests;

impl ContainerManager {
    /// Create a new container manager
    pub fn new(provider: Box<dyn ContainerProvider>) -> Self {
        Self {
            provider,
            containers: HashMap::new(),
        }
    }

    /// Create and start a container for an agent
    pub async fn create_agent_container(
        &mut self,
        agent_id: &str,
        config: ContainerConfig,
    ) -> Result<String> {
        let container_name = format!("ccswarm-agent-{}", agent_id);
        info!(
            "Creating container for agent: {} with name: {}",
            agent_id, container_name
        );

        // Create container
        debug!("Creating container with config: {:?}", config);
        let container = self
            .provider
            .create_container(&container_name, &config)
            .await?;
        let container_id = container.id.clone();

        // Store container info
        self.containers.insert(agent_id.to_string(), container);
        debug!("Container info stored for agent: {}", agent_id);

        // Start container
        info!("Starting container {} for agent {}", container_id, agent_id);
        self.provider.start_container(&container_id).await?;

        info!(
            "Container {} created and started successfully for agent {}",
            container_id, agent_id
        );
        Ok(container_id)
    }

    /// Stop and remove a container
    pub async fn remove_agent_container(&mut self, agent_id: &str) -> Result<()> {
        info!("Removing container for agent: {}", agent_id);

        if let Some(container) = self.containers.remove(agent_id) {
            let container_id = container.id.clone();
            info!(
                "Found container {} for agent {}, proceeding with removal",
                container_id, agent_id
            );

            // Stop container
            debug!("Stopping container: {}", container_id);
            self.provider.stop_container(&container_id).await?;

            // Remove container
            debug!("Removing container: {}", container_id);
            self.provider.remove_container(&container_id).await?;

            info!(
                "Container {} removed successfully for agent {}",
                container_id, agent_id
            );
        } else {
            info!("No container found for agent: {}", agent_id);
        }

        Ok(())
    }

    /// Execute command in agent container
    pub async fn exec_in_agent(&self, agent_id: &str, command: Vec<String>) -> Result<String> {
        debug!("Executing command for agent {}: {:?}", agent_id, command);

        let container = self.containers.get(agent_id).ok_or_else(|| {
            error!("Container not found for agent: {}", agent_id);
            anyhow::anyhow!("Container not found for agent {}", agent_id)
        })?;

        let result = self
            .provider
            .exec_in_container(&container.id, command)
            .await?;
        debug!("Command execution completed for agent {}", agent_id);

        Ok(result)
    }

    /// Get container stats for an agent
    pub async fn get_agent_stats(&self, agent_id: &str) -> Result<ContainerStats> {
        debug!("Getting stats for agent: {}", agent_id);

        let container = self.containers.get(agent_id).ok_or_else(|| {
            error!("Container not found for agent: {}", agent_id);
            anyhow::anyhow!("Container not found for agent {}", agent_id)
        })?;

        let stats = self.provider.get_stats(&container.id).await?;
        debug!(
            "Retrieved stats for agent {}: CPU={:.2}%, Memory={}MB",
            agent_id,
            stats.cpu_percent,
            stats.memory_usage / 1_048_576
        );

        Ok(stats)
    }
}
