//! Container configuration module
//!
//! Provides configuration structures for creating and managing containers.

use crate::container::{NetworkConfig, ResourceLimits, VolumeMapping};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Container configuration for agent containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// Base image to use
    pub image: String,
    /// Working directory inside container
    pub working_dir: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Volume mappings
    pub volumes: Vec<VolumeMapping>,
    /// Network configuration
    pub network: NetworkConfig,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Command to run (optional, uses image default if not specified)
    pub command: Option<Vec<String>>,
    /// Additional labels
    pub labels: HashMap<String, String>,
    /// Restart policy
    pub restart_policy: RestartPolicy,
    /// Security options
    pub security_opts: Vec<String>,
    /// Capabilities to add
    pub cap_add: Vec<String>,
    /// Capabilities to drop
    pub cap_drop: Vec<String>,
}

/// Restart policy for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    No,
    Always,
    OnFailure { max_retries: u32 },
    UnlessStopped,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::OnFailure { max_retries: 3 }
    }
}

impl ContainerConfig {
    /// Create a new container configuration with defaults
    pub fn new(image: String) -> Self {
        let mut env = HashMap::new();
        env.insert("RUST_LOG".to_string(), "info".to_string());

        let mut labels = HashMap::new();
        labels.insert("app".to_string(), "ccswarm".to_string());
        labels.insert("managed-by".to_string(), "ccswarm".to_string());

        Self {
            image,
            working_dir: "/workspace".to_string(),
            env,
            volumes: Vec::new(),
            network: NetworkConfig {
                mode: crate::container::NetworkMode::Bridge,
                network_name: Some("ccswarm-network".to_string()),
                ports: Vec::new(),
            },
            resources: ResourceLimits::default(),
            command: None,
            labels,
            restart_policy: RestartPolicy::default(),
            security_opts: vec!["no-new-privileges".to_string()],
            cap_add: Vec::new(),
            cap_drop: vec!["ALL".to_string()],
        }
    }

    /// Create configuration for a specific agent role
    pub fn for_agent(role: &str, agent_id: &str) -> Self {
        let mut config = Self::new(Self::default_image_for_role(role));

        // Add agent-specific environment variables
        config
            .env
            .insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        config
            .env
            .insert("CCSWARM_AGENT_ROLE".to_string(), role.to_string());

        // Add agent-specific labels
        config
            .labels
            .insert("agent-id".to_string(), agent_id.to_string());
        config
            .labels
            .insert("agent-role".to_string(), role.to_string());

        // Configure based on role
        match role.to_lowercase().as_str() {
            "frontend" => {
                config
                    .env
                    .insert("NODE_ENV".to_string(), "development".to_string());
                config.cap_add.push("NET_BIND_SERVICE".to_string());
            }
            "backend" => {
                config.env.insert(
                    "DATABASE_URL".to_string(),
                    "postgresql://localhost/ccswarm".to_string(),
                );
                config.cap_add.push("NET_BIND_SERVICE".to_string());
            }
            "devops" => {
                // DevOps agents need more capabilities for infrastructure management
                config.cap_add.push("SYS_ADMIN".to_string());
                config.cap_add.push("NET_ADMIN".to_string());
            }
            "qa" => {
                config
                    .env
                    .insert("TEST_ENV".to_string(), "true".to_string());
            }
            _ => {}
        }

        config
    }

    /// Get default image for a given role
    fn default_image_for_role(role: &str) -> String {
        match role.to_lowercase().as_str() {
            "frontend" => "node:20-alpine".to_string(),
            "backend" => "rust:1.75-alpine".to_string(),
            "devops" => "alpine:latest".to_string(),
            "qa" => "python:3.11-alpine".to_string(),
            _ => "ubuntu:22.04".to_string(),
        }
    }

    /// Add a volume mapping
    pub fn add_volume(&mut self, host_path: String, container_path: String, read_only: bool) {
        self.volumes.push(VolumeMapping {
            host_path,
            container_path,
            read_only,
        });
    }

    /// Add an environment variable
    pub fn add_env(&mut self, key: String, value: String) {
        self.env.insert(key, value);
    }

    /// Set resource limits
    pub fn with_resources(mut self, resources: ResourceLimits) -> Self {
        self.resources = resources;
        self
    }

    /// Set custom command
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = Some(command);
        self
    }

    /// Add a label
    pub fn add_label(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
    }
}

/// Builder pattern for ContainerConfig
pub struct ContainerConfigBuilder {
    config: ContainerConfig,
}

impl ContainerConfigBuilder {
    /// Create a new builder with base image
    pub fn new(image: String) -> Self {
        Self {
            config: ContainerConfig::new(image),
        }
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: String) -> Self {
        self.config.working_dir = dir;
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: String, value: String) -> Self {
        self.config.env.insert(key, value);
        self
    }

    /// Add volume mapping
    pub fn volume(mut self, host: String, container: String, read_only: bool) -> Self {
        self.config.add_volume(host, container, read_only);
        self
    }

    /// Set network configuration
    pub fn network(mut self, network: NetworkConfig) -> Self {
        self.config.network = network;
        self
    }

    /// Set resource limits
    pub fn resources(mut self, resources: ResourceLimits) -> Self {
        self.config.resources = resources;
        self
    }

    /// Set command
    pub fn command(mut self, command: Vec<String>) -> Self {
        self.config.command = Some(command);
        self
    }

    /// Add label
    pub fn label(mut self, key: String, value: String) -> Self {
        self.config.labels.insert(key, value);
        self
    }

    /// Set restart policy
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.config.restart_policy = policy;
        self
    }

    /// Build the configuration
    pub fn build(self) -> ContainerConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_config_new() {
        let config = ContainerConfig::new("rust:latest".to_string());
        assert_eq!(config.image, "rust:latest");
        assert_eq!(config.working_dir, "/workspace");
        assert!(config.env.contains_key("RUST_LOG"));
        assert!(config.labels.contains_key("app"));
    }

    #[test]
    fn test_container_config_for_agent() {
        let config = ContainerConfig::for_agent("frontend", "agent-123");
        assert_eq!(
            config.env.get("CCSWARM_AGENT_ID"),
            Some(&"agent-123".to_string())
        );
        assert_eq!(
            config.env.get("CCSWARM_AGENT_ROLE"),
            Some(&"frontend".to_string())
        );
        assert!(config.env.contains_key("NODE_ENV"));
        assert!(config.cap_add.contains(&"NET_BIND_SERVICE".to_string()));
    }

    #[test]
    fn test_container_config_builder() {
        let config = ContainerConfigBuilder::new("alpine:latest".to_string())
            .working_dir("/app".to_string())
            .env("TEST".to_string(), "value".to_string())
            .volume(
                "/host/path".to_string(),
                "/container/path".to_string(),
                false,
            )
            .label("test".to_string(), "label".to_string())
            .build();

        assert_eq!(config.image, "alpine:latest");
        assert_eq!(config.working_dir, "/app");
        assert_eq!(config.env.get("TEST"), Some(&"value".to_string()));
        assert_eq!(config.volumes.len(), 1);
        assert_eq!(config.labels.get("test"), Some(&"label".to_string()));
    }
}
