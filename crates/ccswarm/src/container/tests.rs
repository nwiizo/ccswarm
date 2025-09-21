//! Unit tests for container module

use super::*;
use crate::container::config::{ContainerConfig, ContainerConfigBuilder, RestartPolicy};

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_container_config_default() {
        let config = ContainerConfig::new("rust:latest".to_string());

        assert_eq!(config.image, "rust:latest");
        assert_eq!(config.working_dir, "/workspace");
        assert!(config.env.contains_key("RUST_LOG"));
        assert_eq!(config.env.get("RUST_LOG"), Some(&"info".to_string()));
        assert!(config.labels.contains_key("app"));
        assert_eq!(config.labels.get("app"), Some(&"ccswarm".to_string()));
        assert!(config
            .security_opts
            .contains(&"no-new-privileges".to_string()));
        assert!(config.cap_drop.contains(&"ALL".to_string()));
    }

    #[test]
    fn test_container_config_for_frontend_agent() {
        let config = ContainerConfig::for_agent("frontend", "agent-123");

        assert_eq!(config.image, "node:20-alpine");
        assert_eq!(
            config.env.get("CCSWARM_AGENT_ID"),
            Some(&"agent-123".to_string())
        );
        assert_eq!(
            config.env.get("CCSWARM_AGENT_ROLE"),
            Some(&"frontend".to_string())
        );
        assert_eq!(config.env.get("NODE_ENV"), Some(&"development".to_string()));
        assert!(config.cap_add.contains(&"NET_BIND_SERVICE".to_string()));
        assert_eq!(
            config.labels.get("agent-id"),
            Some(&"agent-123".to_string())
        );
        assert_eq!(
            config.labels.get("agent-role"),
            Some(&"frontend".to_string())
        );
    }

    #[test]
    fn test_container_config_for_backend_agent() {
        let config = ContainerConfig::for_agent("backend", "agent-456");

        assert_eq!(config.image, "rust:1.75-alpine");
        assert_eq!(
            config.env.get("DATABASE_URL"),
            Some(&"postgresql://localhost/ccswarm".to_string())
        );
        assert!(config.cap_add.contains(&"NET_BIND_SERVICE".to_string()));
    }

    #[test]
    fn test_container_config_for_devops_agent() {
        let config = ContainerConfig::for_agent("devops", "agent-789");

        assert_eq!(config.image, "alpine:latest");
        assert!(config.cap_add.contains(&"SYS_ADMIN".to_string()));
        assert!(config.cap_add.contains(&"NET_ADMIN".to_string()));
    }

    #[test]
    fn test_container_config_for_qa_agent() {
        let config = ContainerConfig::for_agent("qa", "agent-101");

        assert_eq!(config.image, "python:3.11-alpine");
        assert_eq!(config.env.get("TEST_ENV"), Some(&"true".to_string()));
    }

    #[test]
    fn test_container_config_add_volume() {
        let mut config = ContainerConfig::new("alpine:latest".to_string());
        config.add_volume(
            "/host/path".to_string(),
            "/container/path".to_string(),
            false,
        );

        assert_eq!(config.volumes.len(), 1);
        assert_eq!(config.volumes[0].host_path, "/host/path");
        assert_eq!(config.volumes[0].container_path, "/container/path");
        assert_eq!(config.volumes[0].read_only, false);
    }

    #[test]
    fn test_container_config_add_env() {
        let mut config = ContainerConfig::new("alpine:latest".to_string());
        config.add_env("MY_VAR".to_string(), "my_value".to_string());

        assert_eq!(config.env.get("MY_VAR"), Some(&"my_value".to_string()));
    }

    #[test]
    fn test_container_config_builder() {
        let config = ContainerConfigBuilder::new("ubuntu:22.04".to_string())
            .working_dir("/app".to_string())
            .env("API_KEY".to_string(), "secret".to_string())
            .volume("/data".to_string(), "/app/data".to_string(), true)
            .label("version".to_string(), "1.0.0".to_string())
            .resources(ResourceLimits {
                cpu_limit: Some(2.0),
                memory_limit: Some(1024 * 1024 * 1024), // 1GB
                memory_swap_limit: None,
                cpu_shares: Some(2048),
            })
            .restart_policy(RestartPolicy::Always)
            .build();

        assert_eq!(config.image, "ubuntu:22.04");
        assert_eq!(config.working_dir, "/app");
        assert_eq!(config.env.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(config.volumes.len(), 1);
        assert_eq!(config.volumes[0].read_only, true);
        assert_eq!(config.labels.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(config.resources.cpu_limit, Some(2.0));
        assert_eq!(config.resources.memory_limit, Some(1024 * 1024 * 1024));
        matches!(config.restart_policy, RestartPolicy::Always);
    }

    #[test]
    fn test_restart_policy_default() {
        let policy = RestartPolicy::default();
        matches!(policy, RestartPolicy::OnFailure { max_retries: 3 });
    }
}

#[cfg(test)]
mod container_status_tests {
    use super::*;

    #[test]
    fn test_container_status_equality() {
        assert_eq!(ContainerStatus::Running, ContainerStatus::Running);
        assert_ne!(ContainerStatus::Running, ContainerStatus::Stopped);

        let error1 = ContainerStatus::Error("test error".to_string());
        let error2 = ContainerStatus::Error("test error".to_string());
        let error3 = ContainerStatus::Error("different error".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}

#[cfg(test)]
mod resource_limits_tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.cpu_limit, Some(1.0));
        assert_eq!(limits.memory_limit, Some(512 * 1024 * 1024)); // 512MB
        assert_eq!(limits.memory_swap_limit, None);
        assert_eq!(limits.cpu_shares, Some(1024));
    }

    #[test]
    fn test_custom_resource_limits() {
        let limits = ResourceLimits {
            cpu_limit: Some(4.0),
            memory_limit: Some(2 * 1024 * 1024 * 1024), // 2GB
            memory_swap_limit: Some(4 * 1024 * 1024 * 1024), // 4GB
            cpu_shares: Some(4096),
        };

        assert_eq!(limits.cpu_limit, Some(4.0));
        assert_eq!(limits.memory_limit, Some(2 * 1024 * 1024 * 1024));
        assert_eq!(limits.memory_swap_limit, Some(4 * 1024 * 1024 * 1024));
        assert_eq!(limits.cpu_shares, Some(4096));
    }
}

#[cfg(test)]
mod network_config_tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = ContainerConfig::new("alpine:latest".to_string());

        matches!(config.network.mode, NetworkMode::Bridge);
        assert_eq!(
            config.network.network_name,
            Some("ccswarm-network".to_string())
        );
        assert_eq!(config.network.ports.len(), 0);
    }

    #[test]
    fn test_port_mapping() {
        let port = PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: "tcp".to_string(),
        };

        assert_eq!(port.host_port, 8080);
        assert_eq!(port.container_port, 80);
        assert_eq!(port.protocol, "tcp");
    }
}

#[cfg(test)]
mod container_manager_tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    // Mock implementation of ContainerProvider
    mock! {
        Provider {}

        #[async_trait]
        impl ContainerProvider for Provider {
            async fn create_container(
                &self,
                name: &str,
                config: &ContainerConfig,
            ) -> Result<Container>;

            async fn start_container(&self, container_id: &str) -> Result<()>;
            async fn stop_container(&self, container_id: &str) -> Result<()>;
            async fn remove_container(&self, container_id: &str) -> Result<()>;

            async fn exec_in_container(
                &self,
                container_id: &str,
                command: Vec<String>,
            ) -> Result<String>;

            async fn get_logs(
                &self,
                container_id: &str,
                tail: Option<usize>,
            ) -> Result<Vec<LogEntry>>;

            async fn get_status(&self, container_id: &str) -> Result<ContainerStatus>;
            async fn list_containers(&self, filter: Option<String>) -> Result<Vec<Container>>;
            async fn get_stats(&self, container_id: &str) -> Result<ContainerStats>;

            async fn copy_to_container(
                &self,
                container_id: &str,
                src_path: &str,
                dest_path: &str,
            ) -> Result<()>;

            async fn copy_from_container(
                &self,
                container_id: &str,
                src_path: &str,
                dest_path: &str,
            ) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_container_manager_create_agent_container() {
        let mut mock_provider = MockProvider::new();

        let test_container = Container {
            id: "container-123".to_string(),
            name: "ccswarm-agent-test-agent".to_string(),
            status: ContainerStatus::Created,
            image: "alpine:latest".to_string(),
            env: HashMap::new(),
            volumes: Vec::new(),
            network: NetworkConfig {
                mode: NetworkMode::Bridge,
                network_name: None,
                ports: Vec::new(),
            },
            resources: ResourceLimits::default(),
        };

        let return_container = test_container.clone();
        mock_provider
            .expect_create_container()
            .with(eq("ccswarm-agent-test-agent"), always())
            .times(1)
            .return_once(move |_, _| Ok(return_container));

        mock_provider
            .expect_start_container()
            .with(eq("container-123"))
            .times(1)
            .return_once(|_| Ok(()));

        let mut manager = ContainerManager::new(Box::new(mock_provider));
        let config = ContainerConfig::new("alpine:latest".to_string());

        let container_id = manager
            .create_agent_container("test-agent", config)
            .await
            .unwrap();
        assert_eq!(container_id, "container-123");
        assert!(manager.containers.contains_key("test-agent"));
    }

    #[tokio::test]
    async fn test_container_manager_remove_agent_container() {
        let mut mock_provider = MockProvider::new();

        mock_provider
            .expect_stop_container()
            .with(eq("container-123"))
            .times(1)
            .return_once(|_| Ok(()));

        mock_provider
            .expect_remove_container()
            .with(eq("container-123"))
            .times(1)
            .return_once(|_| Ok(()));

        let mut manager = ContainerManager::new(Box::new(mock_provider));

        // Manually insert a container
        let container = Container {
            id: "container-123".to_string(),
            name: "ccswarm-agent-test-agent".to_string(),
            status: ContainerStatus::Running,
            image: "alpine:latest".to_string(),
            env: HashMap::new(),
            volumes: Vec::new(),
            network: NetworkConfig {
                mode: NetworkMode::Bridge,
                network_name: None,
                ports: Vec::new(),
            },
            resources: ResourceLimits::default(),
        };
        manager
            .containers
            .insert("test-agent".to_string(), container);

        manager.remove_agent_container("test-agent").await.unwrap();
        assert!(!manager.containers.contains_key("test-agent"));
    }

    #[tokio::test]
    async fn test_container_manager_exec_in_agent() {
        let mock_provider = MockProvider::new();
        let manager = ContainerManager::new(Box::new(mock_provider));

        // Try to execute in non-existent container
        let result = manager
            .exec_in_agent("non-existent", vec!["echo".to_string(), "test".to_string()])
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Container not found"));
    }

    #[tokio::test]
    async fn test_container_manager_get_agent_stats() {
        let mut mock_provider = MockProvider::new();

        let test_stats = ContainerStats {
            cpu_percent: 25.5,
            memory_usage: 256 * 1024 * 1024, // 256MB
            memory_limit: 512 * 1024 * 1024, // 512MB
            network_io: NetworkIO {
                rx_bytes: 1024,
                tx_bytes: 2048,
            },
            disk_io: DiskIO {
                read_bytes: 4096,
                write_bytes: 8192,
            },
        };

        mock_provider
            .expect_get_stats()
            .with(eq("container-123"))
            .times(1)
            .return_once(move |_| Ok(test_stats));

        let mut manager = ContainerManager::new(Box::new(mock_provider));

        // Manually insert a container
        let container = Container {
            id: "container-123".to_string(),
            name: "ccswarm-agent-test-agent".to_string(),
            status: ContainerStatus::Running,
            image: "alpine:latest".to_string(),
            env: HashMap::new(),
            volumes: Vec::new(),
            network: NetworkConfig {
                mode: NetworkMode::Bridge,
                network_name: None,
                ports: Vec::new(),
            },
            resources: ResourceLimits::default(),
        };
        manager
            .containers
            .insert("test-agent".to_string(), container);

        let stats = manager.get_agent_stats("test-agent").await.unwrap();
        assert_eq!(stats.cpu_percent, 25.5);
        assert_eq!(stats.memory_usage, 256 * 1024 * 1024);
    }
}

#[cfg(test)]
mod volume_mapping_tests {
    use super::*;

    #[test]
    fn test_volume_mapping() {
        let volume = VolumeMapping {
            host_path: "/host/data".to_string(),
            container_path: "/container/data".to_string(),
            read_only: true,
        };

        assert_eq!(volume.host_path, "/host/data");
        assert_eq!(volume.container_path, "/container/data");
        assert!(volume.read_only);
    }
}

#[cfg(test)]
mod log_entry_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_log_entry() {
        let now = Utc::now();
        let log = LogEntry {
            timestamp: now,
            message: "Test log message".to_string(),
            stream: LogStream::Stdout,
        };

        assert_eq!(log.timestamp, now);
        assert_eq!(log.message, "Test log message");
        matches!(log.stream, LogStream::Stdout);
    }
}
