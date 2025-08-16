use ccswarm::config::{
    AgentConfig, CcswarmConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig,
    ProjectConfig, RepositoryConfig, ThinkMode,
};
use ccswarm::coordination::{AgentMessage, CoordinationBus};
use ccswarm::orchestrator::MasterClaude;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a test configuration
fn create_test_config() -> CcswarmConfig {
    let mut agents = HashMap::new();
    agents.insert(
        "frontend".to_string(),
        AgentConfig {
            specialization: "frontend".to_string(),
            worktree: "agents/frontend".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );
    agents.insert(
        "backend".to_string(),
        AgentConfig {
            specialization: "backend".to_string(),
            worktree: "agents/backend".to_string(),
            branch: "feature/backend".to_string(),
            claude_config: ClaudeConfig::for_agent("backend"),
            claude_md_template: "backend_specialist".to_string(),
        },
    );

    CcswarmConfig {
        project: ProjectConfig {
            name: "TestProject".to_string(),
            repository: RepositoryConfig {
                url: "https://github.com/test/repo".to_string(),
                main_branch: "main".to_string(),
            },
            master_claude: MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.9,
                think_mode: ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: ClaudeConfig::for_master(),
                enable_proactive_mode: false, // Disable for tests
                proactive_frequency: 300,
                high_frequency: 60,
            },
        },
        agents,
        coordination: CoordinationConfig {
            communication_method: "json_files".to_string(),
            sync_interval: 30,
            quality_gate_frequency: "on_commit".to_string(),
            master_review_trigger: "all_tasks_complete".to_string(),
        },
    }
}

#[tokio::test]
async fn test_master_claude_creation_and_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    ccswarm::git::WorktreeManager::init_if_needed(&repo_path)
        .await
        .unwrap();

    // Create MasterClaude instance
    let config = create_test_config();
    let master = MasterClaude::new();

    // Verify basic properties
    assert!(master.id.starts_with("master-claude-"));
    // assert_eq!(master.config.project.name, "TestProject");

    // Check initial state
    let state = master.state.read().await;
    // assert_eq!(
    //     state.status,
    //     ccswarm::orchestrator::OrchestratorStatus::Initializing
    // );
    assert_eq!(state.total_tasks_processed, 0);
    assert_eq!(state.successful_tasks, 0);
    assert_eq!(state.failed_tasks, 0);
}

#[tokio::test]
async fn test_coordination_bus_message_flow() {
    // Create coordination bus
    let bus = Arc::new(CoordinationBus::new().await.unwrap());

    // Send a task assignment message
    let task_msg = AgentMessage::TaskAssignment {
        task_id: "test-task-001".to_string(),
        agent_id: "frontend-agent".to_string(),
        task_data: json!({
            "description": "Create login component",
            "priority": "high"
        }),
    };

    bus.send_message(task_msg.clone()).await.unwrap();

    // Wait for message propagation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to receive the message
    match bus.receive_message().await {
        Ok(received_msg) => match received_msg {
            AgentMessage::TaskAssignment {
                task_id, agent_id, ..
            } => {
                assert_eq!(task_id, "test-task-001");
                assert_eq!(agent_id, "frontend-agent");
            }
            _ => panic!("Unexpected message type received"),
        },
        Err(e) => panic!("Failed to receive message: {}", e),
    }
}

#[tokio::test]
async fn test_master_claude_task_assignment() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    ccswarm::git::WorktreeManager::init_if_needed(&repo_path)
        .await
        .unwrap();

    // Create and initialize MasterClaude
    let config = create_test_config();
    let master = MasterClaude::new();

    // Create a test task
    use ccswarm::agent::{Priority, Task, TaskType};
    let task = Task::new(
        "Create user authentication system".to_string(),
        TaskType::Feature,
        Priority::High,
    );

    // Add task to queue
    master.add_task(task.clone()).await.unwrap();

    // Verify task was added to pending tasks
    let state = master.state.read().await;
    assert_eq!(state.pending_tasks.len(), 1);
    assert_eq!(state.pending_tasks[0].id, "test-task-001");
}

#[tokio::test]
async fn test_agent_message_handling() {
    // Create coordination bus
    let bus = Arc::new(CoordinationBus::new().await.unwrap());

    // Test different message types
    let messages = vec![
        AgentMessage::StatusUpdate {
            agent_id: "test-agent".to_string(),
            status: ccswarm::agent::AgentStatus::Available,
            metrics: json!({"cpu": 45.2, "memory": 1024}),
        },
        AgentMessage::TaskProgress {
            agent_id: "test-agent".to_string(),
            task_id: "task-001".to_string(),
            progress: 0.75,
            message: "75% complete".to_string(),
        },
        AgentMessage::HelpRequest {
            agent_id: "test-agent".to_string(),
            context: "Need help with API integration".to_string(),
            priority: ccswarm::coordination::MessagePriority::High,
        },
    ];

    // Send all messages
    for msg in &messages {
        bus.send_message(msg.clone()).await.unwrap();
    }

    // Wait for propagation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify we can receive messages
    for _ in 0..messages.len() {
        let result = bus.receive_message().await;
        assert!(result.is_ok(), "Failed to receive message");
    }
}

#[tokio::test]
async fn test_quality_review_flow() {
    use ccswarm::agent::{Priority, Task, TaskResult, TaskType};

    // Create a task result
    let task = Task::new(
        "Implement user login".to_string(),
        TaskType::Feature,
        Priority::High,
    );

    let result = TaskResult {
        task_id: task.id.clone(),
        success: true,
        output: Some(json!({
            "message": "Login component created successfully",
            "files_created": ["src/components/Login.tsx"],
            "files_modified": [],
            "files_deleted": []
        }).to_string()),
        error: None,
        duration: Some(Duration::from_secs(120)),
    };

    // Create quality issue message
    let quality_msg = AgentMessage::QualityIssue {
        agent_id: "frontend-agent".to_string(),
        task_id: task.id.clone(),
        issues: vec![
            "Low test coverage".to_string(),
            "Missing documentation".to_string(),
        ],
    };

    // Verify message structure
    match quality_msg {
        AgentMessage::QualityIssue {
            agent_id,
            task_id,
            issues,
        } => {
            assert_eq!(agent_id, "frontend-agent");
            assert_eq!(task_id, "test-task-001");
            assert_eq!(issues.len(), 2);
        }
        _ => panic!("Unexpected message type"),
    }
}

#[tokio::test]
async fn test_master_claude_with_ai_session_integration() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    ccswarm::git::WorktreeManager::init_if_needed(&repo_path)
        .await
        .unwrap();

    // Create MasterClaude with AI-session enabled
    let mut config = create_test_config();
    config.project.master_claude.enable_proactive_mode = false; // Disable for testing

    let master = MasterClaude::new();

    // Create a task that would use AI-session
    use ccswarm::agent::{Priority, Task, TaskType};
    let task = Task::new(
        "Create API endpoint with AI-session support".to_string(),
        TaskType::Development,
        Priority::High,
    );

    // Add metadata indicating AI-session usage
    let enhanced_task = task.clone();
    // enhanced_task.metadata = Some(serde_json::Map::new());
    // if let Some(metadata) = enhanced_task.metadata.as_mut() {
    //     metadata.insert("ai_session_enabled".to_string(), json!(true));
    //     metadata.insert("context_compression".to_string(), json!("enabled"));
    // }

    // Add task
    master.add_task(enhanced_task).await.unwrap();

    // Verify task has AI-session metadata
    let state = master.state.read().await;
    assert_eq!(state.pending_tasks.len(), 1);
    let pending_task = &state.pending_tasks[0];
    assert!(pending_task.metadata.is_some());

    let metadata = pending_task.metadata.as_ref().unwrap();
    assert_eq!(metadata.get("ai_session_enabled"), Some(&json!(true)));
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    // Test coordination bus error handling
    let bus = Arc::new(CoordinationBus::new().await.unwrap());

    // Send message with empty agent_id (should still work)
    let invalid_msg = AgentMessage::StatusUpdate {
        agent_id: "".to_string(),
        status: ccswarm::agent::AgentStatus::Available,
        metrics: json!({}),
    };

    let send_result = bus.send_message(invalid_msg).await;
    assert!(
        send_result.is_ok(),
        "Should handle empty agent_id gracefully"
    );
}

#[tokio::test]
async fn test_task_result_creation() {
    use ccswarm::agent::TaskResult;

    // Test successful result
    let success_result = TaskResult::success(
        "test-task-id".to_string(),
        json!({
            "message": "Task completed successfully",
            "details": {
                "files_created": 5,
                "tests_passed": 10
            }
        }).to_string(),
    );

    assert!(success_result.success);
    assert!(success_result.error.is_none());
    // assert_eq!(success_result.duration, Some(Duration::from_secs(60)));

    // Test failure result
    let failure_result = TaskResult::failure(
        "test-task-id".to_string(),
        "Failed to connect to database".to_string(),
    );

    assert!(!failure_result.success);
    assert_eq!(
        failure_result.error,
        Some("Failed to connect to database".to_string())
    );
    // assert_eq!(failure_result.duration, Some(Duration::from_secs(30)));
}

#[tokio::test]
async fn test_coordination_message_types() {
    let bus = Arc::new(CoordinationBus::new().await.unwrap());

    // Test TaskGenerated message
    let task_gen_msg = AgentMessage::TaskGenerated {
        task_id: "gen-task-001".to_string(),
        description: "Implement caching layer".to_string(),
        reasoning: "Performance optimization needed".to_string(),
    };

    bus.send_message(task_gen_msg).await.unwrap();

    // Test RequestAssistance message
    let assist_msg = AgentMessage::RequestAssistance {
        agent_id: "backend-agent".to_string(),
        task_id: "task-002".to_string(),
        reason: "Need expertise on GraphQL implementation".to_string(),
    };

    bus.send_message(assist_msg).await.unwrap();

    // Test Custom message
    let custom_msg = AgentMessage::Custom {
        message_type: "performance_metric".to_string(),
        data: json!({
            "agent_id": "frontend-agent",
            "metric": "render_time",
            "value": 250,
            "unit": "ms"
        }),
    };

    bus.send_message(custom_msg).await.unwrap();

    // Wait and verify all messages were sent
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Receive and verify messages
    for _ in 0..3 {
        let result = bus.receive_message().await;
        assert!(result.is_ok(), "Should receive all message types");
    }
}
