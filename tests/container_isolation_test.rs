//! Integration tests for container isolation mode
//! DISABLED: Container module is temporarily disabled

#[cfg(feature = "container")]
use ccswarm::agent::{ClaudeCodeAgent, IsolationMode, Priority, Task, TaskType};
#[cfg(feature = "container")]
use ccswarm::config::ClaudeConfig;
#[cfg(feature = "container")]
use ccswarm::identity::AgentRole;
#[cfg(feature = "container")]
use tempfile::TempDir;

/// Test container isolation mode for agents
#[tokio::test]
#[ignore] // Requires Docker
#[cfg(feature = "container")]
async fn test_agent_container_isolation() {
    // Create temporary directory for worktree
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create agent with container isolation
    let role = AgentRole::Backend {
        technologies: vec!["rust".to_string(), "actix".to_string()],
        responsibilities: vec!["API development".to_string()],
        boundaries: vec!["No frontend code".to_string()],
    };

    let claude_config = ClaudeConfig::default();

    let mut agent = ClaudeCodeAgent::new_with_isolation(
        role,
        workspace_root,
        "test",
        claude_config,
        IsolationMode::Container,
    )
    .await
    .expect("Failed to create agent");

    // Initialize agent (should create container)
    agent
        .initialize()
        .await
        .expect("Failed to initialize agent");

    // Verify container was created
    assert!(agent.container_id.is_some());
    println!(
        "Container created: {}",
        agent.container_id.as_ref().unwrap()
    );

    // Create a test task
    let task = Task::new(
        "test-task".to_string(),
        "Write a simple Rust function".to_string(),
        Priority::Medium,
        TaskType::Development,
    );

    // Execute task in container
    let result = agent.execute_task(task).await;

    // Task might fail due to missing Claude CLI in container, but that's expected
    // We're mainly testing that the container execution path works
    match result {
        Ok(result) => {
            println!("Task executed successfully: {:?}", result);
        }
        Err(e) => {
            println!("Task execution failed (expected): {}", e);
            // This is expected since we don't have Claude CLI in test containers
        }
    }

    // Shutdown agent (should cleanup container)
    agent.shutdown().await.expect("Failed to shutdown agent");
}

/// Test hybrid isolation mode fallback
#[tokio::test]
#[ignore] // Requires Docker
#[cfg(feature = "container")]
async fn test_hybrid_isolation_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    let role = AgentRole::Frontend {
        technologies: vec!["react".to_string(), "nextjs".to_string()],
        responsibilities: vec!["UI development".to_string()],
        boundaries: vec!["No backend code".to_string()],
    };

    let claude_config = ClaudeConfig::default();

    let mut agent = ClaudeCodeAgent::new_with_isolation(
        role,
        workspace_root,
        "test",
        claude_config,
        IsolationMode::Hybrid,
    )
    .await
    .expect("Failed to create agent");

    // Initialize agent
    agent
        .initialize()
        .await
        .expect("Failed to initialize agent");

    // In hybrid mode, container should be created
    assert!(agent.container_id.is_some());

    // Shutdown
    agent.shutdown().await.expect("Failed to shutdown agent");
}

/// Test file copy operations
#[tokio::test]
#[ignore] // Requires Docker
#[cfg(feature = "container")]
async fn test_container_file_operations() {
    use ccswarm::container::{ContainerConfig, ContainerProvider};
    use ccswarm::container::docker::DockerContainerProvider;
    use std::fs;

    // Create provider
    let provider = DockerContainerProvider::new()
        .await
        .expect("Failed to create Docker provider");

    // Create test container
    let config = ContainerConfig::new("alpine:latest".to_string());
    let container = provider
        .create_container("test-file-ops", &config)
        .await
        .expect("Failed to create container");

    provider
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // Create test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello from host!").expect("Failed to write test file");

    // Copy file to container
    provider
        .copy_to_container(&container.id, test_file.to_str().unwrap(), "/tmp/test.txt")
        .await
        .expect("Failed to copy to container");

    // Verify file exists in container
    let output = provider
        .exec_in_container(
            &container.id,
            vec!["cat".to_string(), "/tmp/test.txt".to_string()],
        )
        .await
        .expect("Failed to read file in container");

    assert_eq!(output.trim(), "Hello from host!");

    // Copy file back from container
    let output_file = temp_dir.path().join("output.txt");
    provider
        .copy_from_container(
            &container.id,
            "/tmp/test.txt",
            output_file.to_str().unwrap(),
        )
        .await
        .expect("Failed to copy from container");

    // Verify copied file
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert_eq!(content, "Hello from host!");

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
