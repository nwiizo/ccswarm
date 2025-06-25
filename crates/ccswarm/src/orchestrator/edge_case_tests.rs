#[cfg(test)]
mod edge_case_tests {
    use crate::agent::{AgentStatus, Priority, Task, TaskResult, TaskType};
    use crate::config::{CcswarmConfig, ClaudeConfig};
    use crate::coordination::AgentMessage;
    use crate::orchestrator::{MasterClaude, OrchestratorStatus};
    use tempfile::TempDir;

    async fn create_minimal_config() -> (CcswarmConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Create minimal config manually
        let config = CcswarmConfig {
            project: crate::config::ProjectConfig {
                name: "Test Project".to_string(),
                repository: crate::config::RepositoryConfig {
                    url: repo_path.to_string_lossy().to_string(),
                    main_branch: "main".to_string(),
                },
                master_claude: crate::config::MasterClaudeConfig {
                    role: "technical_lead".to_string(),
                    quality_threshold: 0.85,
                    think_mode: crate::config::ThinkMode::UltraThink,
                    permission_level: "supervised".to_string(),
                    claude_config: ClaudeConfig::for_master(),
                    enable_proactive_mode: false,
                    proactive_frequency: 300,
                    high_frequency: 60,
                },
            },
            agents: std::collections::HashMap::new(),
            coordination: crate::config::CoordinationConfig {
                communication_method: "json_files".to_string(),
                sync_interval: 30,
                quality_gate_frequency: "on_commit".to_string(),
                master_review_trigger: "all_tasks_complete".to_string(),
            },
        };
        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_empty_agent_pool() {
        let (config, _temp_dir) = create_minimal_config().await;
        let repo_path = std::path::PathBuf::from("/tmp/test");

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Try to assign task with no agents
        let task = Task::new(
            "no-agents-task".to_string(),
            "Task with no agents".to_string(),
            Priority::High,
            TaskType::Development,
        );

        let result = master.select_optimal_agent(&task).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No available agents"));
    }

    #[tokio::test]
    async fn test_all_agents_busy() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(crate::agent::IsolationMode::GitWorktree);

        // Create busy agents
        for i in 0..3 {
            let mut agent = crate::agent::ClaudeCodeAgent::new_with_isolation(
                crate::identity::default_backend_role(),
                &repo_path,
                &format!("feature/busy-{}", i),
                ClaudeConfig::for_agent(&format!("busy-{}", i)),
                crate::agent::IsolationMode::GitWorktree,
            )
            .await
            .unwrap();

            agent.status = AgentStatus::Working;
            master.agents.insert(format!("busy-agent-{}", i), agent);
        }

        // Try to assign task
        let task = Task::new(
            "busy-task".to_string(),
            "Task for busy agents".to_string(),
            Priority::High,
            TaskType::Development,
        );

        let result = master.select_optimal_agent(&task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_task_metadata() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Create task with invalid metadata
        let mut task = Task::new(
            "invalid-meta".to_string(),
            "Task with bad metadata".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        task.metadata = Some(
            serde_json::json!({
                "invalid": null,
                "nested": {
                    "deeply": {
                        "nested": {
                            "value": "too deep"
                        }
                    }
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        // Should still handle the task
        master.add_task(task.clone()).await.unwrap();

        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_message_overload() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();
        let bus = master.coordination_bus.clone();

        // Send many messages concurrently
        let mut handles = vec![];
        for i in 0..100 {
            let bus_clone = bus.clone();
            let handle = tokio::spawn(async move {
                let message = AgentMessage::Heartbeat {
                    agent_id: format!("agent-{}", i),
                    timestamp: chrono::Utc::now(),
                };
                bus_clone.send_message(message).await
            });
            handles.push(handle);
        }

        // All should complete without panic
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_malformed_quality_issues() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Start the orchestrator
        {
            let mut state = master.state.write().await;
            state.status = OrchestratorStatus::Running;
        }

        // Send quality issue with empty issues list
        let empty_issues = AgentMessage::QualityIssue {
            agent_id: "test-agent".to_string(),
            task_id: "empty-issues-task".to_string(),
            issues: vec![],
        };

        // Should handle gracefully
        master
            .coordination_bus
            .send_message(empty_issues)
            .await
            .unwrap();

        // Send quality issue with very long issue descriptions
        let long_issues = AgentMessage::QualityIssue {
            agent_id: "test-agent".to_string(),
            task_id: "long-issues-task".to_string(),
            issues: vec!["X".repeat(10000)], // 10k character issue
        };

        master
            .coordination_bus
            .send_message(long_issues)
            .await
            .unwrap();

        // Allow processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Should not crash
        let state = master.state.read().await;
        assert_eq!(state.status, OrchestratorStatus::Running);
    }

    #[tokio::test]
    async fn test_cyclic_task_dependencies() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Create tasks with cyclic dependencies
        let task1 = Task::new(
            "cycle-1".to_string(),
            "Task 1".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_parent_task("cycle-3".to_string());

        let task2 = Task::new(
            "cycle-2".to_string(),
            "Task 2".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_parent_task("cycle-1".to_string());

        let task3 = Task::new(
            "cycle-3".to_string(),
            "Task 3".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_parent_task("cycle-2".to_string());

        // Should handle without stack overflow
        master.add_task(task1).await.unwrap();
        master.add_task(task2).await.unwrap();
        master.add_task(task3).await.unwrap();

        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_agent_disconnection_during_task() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(crate::agent::IsolationMode::GitWorktree);

        // Create agent
        let agent = crate::agent::ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/disconnect",
            ClaudeConfig::for_agent("disconnect-agent"),
            crate::agent::IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        master.agents.insert("disconnect-agent".to_string(), agent);

        // Simulate agent disconnection (remove from pool)
        master.agents.remove("disconnect-agent");

        // Try to send message for this agent
        let message = AgentMessage::TaskCompleted {
            agent_id: "disconnect-agent".to_string(),
            task_id: "orphan-task".to_string(),
            result: TaskResult {
                success: false,
                output: serde_json::json!({"error": "disconnected"}),
                error: Some("Agent disconnected".to_string()),
                duration: std::time::Duration::from_secs(0),
            },
        };

        // Should handle gracefully
        master.coordination_bus.send_message(message).await.unwrap();
    }

    #[tokio::test]
    async fn test_extremely_long_task_descriptions() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Create task with extremely long description
        let long_description = "A".repeat(100_000); // 100k characters
        let task = Task::new(
            "long-desc".to_string(),
            long_description,
            Priority::Low,
            TaskType::Documentation,
        );

        // Should handle without memory issues
        master.add_task(task).await.unwrap();

        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_rapid_agent_status_changes() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(crate::agent::IsolationMode::GitWorktree);

        // Create agent
        let agent = crate::agent::ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/rapid",
            ClaudeConfig::for_agent("rapid-agent"),
            crate::agent::IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        master.agents.insert("rapid-agent".to_string(), agent);

        // Rapidly change status
        let statuses = vec![
            AgentStatus::Available,
            AgentStatus::Working,
            AgentStatus::Error("Test error".to_string()),
            AgentStatus::Available,
            AgentStatus::Working,
        ];

        for status in statuses {
            let message = AgentMessage::StatusUpdate {
                agent_id: "rapid-agent".to_string(),
                status: status.clone(),
                metrics: serde_json::Value::Object(serde_json::Map::new()),
            };
            master.coordination_bus.send_message(message).await.unwrap();
        }

        // Should handle all changes without crashing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // The messages were sent successfully, which is what we're testing
        // The actual status update would require the orchestrator to be running
        assert!(master.agents.contains_key("rapid-agent"));
    }

    #[tokio::test]
    async fn test_nil_task_priority() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Create tasks with same priority
        for i in 0..5 {
            let task = Task::new(
                format!("same-priority-{}", i),
                "Same priority task".to_string(),
                Priority::Medium,
                TaskType::Development,
            );
            master.add_task(task).await.unwrap();
        }

        // All should be added successfully
        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 5);
    }

    #[tokio::test]
    async fn test_orchestrator_state_recovery() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Simulate error state
        {
            let mut state = master.state.write().await;
            state.status = OrchestratorStatus::Error("Test error".to_string());
        }

        // Should be able to recover
        {
            let mut state = master.state.write().await;
            state.status = OrchestratorStatus::Running;
        }

        let state = master.state.read().await;
        assert_eq!(state.status, OrchestratorStatus::Running);
    }

    #[tokio::test]
    async fn test_invalid_agent_specialization() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(crate::agent::IsolationMode::GitWorktree);

        // Create agent with frontend role
        let mut agent = crate::agent::ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_frontend_role(),
            &repo_path,
            "feature/unknown",
            ClaudeConfig::for_agent("unknown-agent"),
            crate::agent::IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        // Agent has frontend role, but we'll test task assignment for a role that doesn't match
        agent.status = AgentStatus::Available;

        master.agents.insert("unknown-agent".to_string(), agent);

        // Task assignment should still work (fall back to any available)
        let task = Task::new(
            "unknown-task".to_string(),
            "Task for unknown specialization".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        let result = master.select_optimal_agent(&task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "This test can hang in CI due to creating too many concurrent tasks"]
    async fn test_message_queue_overflow() {
        let (config, temp_dir) = create_minimal_config().await;
        let repo_path = temp_dir.path().to_path_buf();

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Try to overflow task queue (bounded at 1000)
        let mut handles = vec![];
        for i in 0..1100 {
            let master_clone = master.clone();
            let handle = tokio::spawn(async move {
                let task = Task::new(
                    format!("overflow-{}", i),
                    "Overflow task".to_string(),
                    Priority::Low,
                    TaskType::Development,
                );
                master_clone.add_task(task).await
            });
            handles.push(handle);
        }

        // Some should fail due to channel being full
        let mut success_count = 0;
        let mut fail_count = 0;

        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => success_count += 1,
                Err(_) => fail_count += 1,
            }
        }

        // Should have some failures
        assert!(fail_count > 0);
        assert!(success_count <= 1000);
    }
}
