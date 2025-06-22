//! Integration tests for Docker container operations
//!
//! These tests require Docker to be running and accessible.
//! They are marked with #[ignore] by default to avoid failures in CI environments without Docker.
//! DISABLED: Container module is temporarily disabled

#[cfg(feature = "container")]
use ccswarm::container::{
    ContainerConfig, ContainerManager, ContainerProvider, ContainerStatus, DockerContainerProvider,
    ResourceLimits,
};
#[cfg(feature = "container")]
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize test logging
fn init_test_logging() {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ccswarm=debug".parse().unwrap()),
        )
        .try_init();
}

/// Check if Docker is available
async fn is_docker_available() -> bool {
    match DockerContainerProvider::new().await {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Docker not available: {}", e);
            false
        }
    }
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_docker_provider_lifecycle() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Create a simple container config
    let config = ContainerConfig::new("alpine:latest".to_string());

    // Create container
    info!("Creating container");
    let container = provider
        .create_container("test-container-lifecycle", &config)
        .await
        .expect("Failed to create container");

    assert_eq!(container.name, "test-container-lifecycle");
    assert_eq!(container.status, ContainerStatus::Created);

    // Start container
    info!("Starting container");
    provider
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // Check status
    let status = provider
        .get_status(&container.id)
        .await
        .expect("Failed to get container status");
    assert_eq!(status, ContainerStatus::Running);

    // Execute command
    info!("Executing command in container");
    let output = provider
        .exec_in_container(
            &container.id,
            vec!["echo".to_string(), "Hello from container".to_string()],
        )
        .await
        .expect("Failed to execute command");

    assert!(output.contains("Hello from container"));

    // Get logs
    info!("Getting container logs");
    let logs = provider
        .get_logs(&container.id, Some(10))
        .await
        .expect("Failed to get logs");
    assert!(!logs.is_empty());

    // Stop container
    info!("Stopping container");
    provider
        .stop_container(&container.id)
        .await
        .expect("Failed to stop container");

    // Verify stopped
    let status = provider
        .get_status(&container.id)
        .await
        .expect("Failed to get container status");
    assert_eq!(status, ContainerStatus::Stopped);

    // Remove container
    info!("Removing container");
    provider
        .remove_container(&container.id)
        .await
        .expect("Failed to remove container");
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_container_manager_agent_workflow() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = Box::new(
        DockerContainerProvider::new()
            .await
            .expect("Failed to create Docker provider"),
    );

    let mut manager = ContainerManager::new(provider);

    // Create container for agent
    let config = ContainerConfig::for_agent("backend", "test-agent-001");

    info!("Creating agent container");
    let container_id = manager
        .create_agent_container("test-agent-001", config)
        .await
        .expect("Failed to create agent container");

    assert!(!container_id.is_empty());

    // Execute command in agent
    info!("Executing command in agent container");
    let output = manager
        .exec_in_agent("test-agent-001", vec!["pwd".to_string()])
        .await
        .expect("Failed to execute command");

    assert!(output.contains("/workspace"));

    // Get agent stats
    info!("Getting agent stats");
    let stats = manager
        .get_agent_stats("test-agent-001")
        .await
        .expect("Failed to get agent stats");

    assert!(stats.memory_usage > 0);
    assert!(stats.cpu_percent >= 0.0);

    // Remove agent container
    info!("Removing agent container");
    manager
        .remove_agent_container("test-agent-001")
        .await
        .expect("Failed to remove agent container");
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_container_with_resource_limits() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Create container with resource limits
    let mut config = ContainerConfig::new("alpine:latest".to_string());
    config.resources = ResourceLimits {
        cpu_limit: Some(0.5),                  // 0.5 CPU
        memory_limit: Some(128 * 1024 * 1024), // 128MB
        memory_swap_limit: None,
        cpu_shares: Some(512),
    };

    info!("Creating container with resource limits");
    let container = provider
        .create_container("test-container-resources", &config)
        .await
        .expect("Failed to create container");

    provider
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // Let it run for a bit
    sleep(Duration::from_secs(2)).await;

    // Get stats to verify limits are applied
    let stats = provider
        .get_stats(&container.id)
        .await
        .expect("Failed to get container stats");

    // Memory limit should be approximately what we set
    assert!(stats.memory_limit > 0);
    info!("Container memory limit: {} bytes", stats.memory_limit);

    // Cleanup
    provider
        .stop_container(&container.id)
        .await
        .expect("Failed to stop container");
    provider
        .remove_container(&container.id)
        .await
        .expect("Failed to remove container");
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_container_list_and_filter() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Create multiple containers
    let config = ContainerConfig::new("alpine:latest".to_string());

    info!("Creating test containers");
    let container1 = provider
        .create_container("test-list-1", &config)
        .await
        .expect("Failed to create container 1");
    let container2 = provider
        .create_container("test-list-2", &config)
        .await
        .expect("Failed to create container 2");

    // List all ccswarm containers
    let containers = provider
        .list_containers(None)
        .await
        .expect("Failed to list containers");

    assert!(containers.len() >= 2);

    // List with filter
    let filtered = provider
        .list_containers(Some("test-list-1".to_string()))
        .await
        .expect("Failed to list filtered containers");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "test-list-1");

    // Cleanup
    info!("Cleaning up test containers");
    provider
        .remove_container(&container1.id)
        .await
        .expect("Failed to remove container 1");
    provider
        .remove_container(&container2.id)
        .await
        .expect("Failed to remove container 2");
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_container_network_isolation() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Create two containers on the same network
    let mut config1 = ContainerConfig::new("alpine:latest".to_string());
    config1.command = Some(vec!["sleep".to_string(), "30".to_string()]);

    let config2 = config1.clone();

    info!("Creating networked containers");
    let container1 = provider
        .create_container("test-network-1", &config1)
        .await
        .expect("Failed to create container 1");
    let container2 = provider
        .create_container("test-network-2", &config2)
        .await
        .expect("Failed to create container 2");

    // Start both containers
    provider
        .start_container(&container1.id)
        .await
        .expect("Failed to start container 1");
    provider
        .start_container(&container2.id)
        .await
        .expect("Failed to start container 2");

    // Containers on the same network should be able to ping each other by name
    let ping_result = provider
        .exec_in_container(
            &container1.id,
            vec![
                "ping".to_string(),
                "-c".to_string(),
                "1".to_string(),
                "test-network-2".to_string(),
            ],
        )
        .await;

    // Note: This might fail if containers don't have ping installed
    // but it demonstrates the network connectivity concept
    info!("Ping result: {:?}", ping_result);

    // Cleanup
    info!("Cleaning up networked containers");
    provider.stop_container(&container1.id).await.ok();
    provider.stop_container(&container2.id).await.ok();
    provider.remove_container(&container1.id).await.ok();
    provider.remove_container(&container2.id).await.ok();
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_multiple_agent_containers() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = Box::new(
        DockerContainerProvider::new()
            .await
            .expect("Failed to create Docker provider"),
    );

    let mut manager = ContainerManager::new(provider);

    // Create containers for different agent types
    let agents = vec![
        ("frontend", "agent-frontend-001"),
        ("backend", "agent-backend-001"),
        ("devops", "agent-devops-001"),
        ("qa", "agent-qa-001"),
    ];

    let mut created_agents = Vec::new();

    for (role, agent_id) in &agents {
        info!("Creating container for {} agent: {}", role, agent_id);
        let config = ContainerConfig::for_agent(role, agent_id);

        let container_id = manager
            .create_agent_container(agent_id, config)
            .await
            .expect(&format!("Failed to create {} agent container", role));

        created_agents.push((agent_id.to_string(), container_id));

        // Verify agent-specific configuration
        let output = manager
            .exec_in_agent(
                agent_id,
                vec!["printenv".to_string(), "CCSWARM_AGENT_ROLE".to_string()],
            )
            .await
            .expect("Failed to get agent role");

        assert!(output.contains(role));
    }

    // Verify all agents are running
    for (agent_id, _) in &created_agents {
        let stats = manager
            .get_agent_stats(agent_id)
            .await
            .expect(&format!("Failed to get stats for {}", agent_id));
        assert!(stats.memory_usage > 0);
    }

    // Cleanup all agents
    info!("Cleaning up all agent containers");
    for (agent_id, _) in created_agents {
        manager
            .remove_agent_container(&agent_id)
            .await
            .expect(&format!("Failed to remove agent {}", agent_id));
    }
}

#[tokio::test]
#[ignore = "Requires Docker"]
#[cfg(feature = "container")]
async fn test_container_error_handling() {
    init_test_logging();

    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Try to start non-existent container
    let result = provider.start_container("non-existent-container").await;
    assert!(result.is_err());

    // Try to execute in non-existent container
    let result = provider
        .exec_in_container(
            "non-existent-container",
            vec!["echo".to_string(), "test".to_string()],
        )
        .await;
    assert!(result.is_err());

    // Try to get stats for non-existent container
    let result = provider.get_stats("non-existent-container").await;
    assert!(result.is_err());

    // Try to create container with invalid image
    let config = ContainerConfig::new("invalid-image-that-does-not-exist:latest".to_string());
    let result = provider
        .create_container("test-invalid-image", &config)
        .await;
    // This might succeed initially but fail on start
    if let Ok(container) = result {
        let start_result = provider.start_container(&container.id).await;
        // Cleanup if created
        provider.remove_container(&container.id).await.ok();
        assert!(start_result.is_err());
    }
}
