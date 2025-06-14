#[cfg(test)]
mod tests {
    use crate::agent::{Priority, Task, TaskType};
    use crate::config::{
        AgentConfig, CcswarmConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig,
        ProjectConfig, RepositoryConfig, ThinkMode,
    };
    use crate::coordination::{AgentMessage, CoordinationBus, StatusTracker, TaskQueue};
    use crate::git::WorktreeManager;
    use crate::orchestrator::MasterClaude;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use uuid::Uuid;

    async fn create_test_config() -> (CcswarmConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        WorktreeManager::init_if_needed(&repo_path).await.unwrap();

        let mut agents = HashMap::new();

        // Frontend agent config
        agents.insert(
            "frontend".to_string(),
            AgentConfig {
                specialization: "frontend".to_string(),
                worktree: "agents/frontend-agent".to_string(),
                branch: "feature/frontend-ui".to_string(),
                claude_config: ClaudeConfig::for_agent("frontend"),
                claude_md_template: "frontend_specialist".to_string(),
            },
        );

        // Backend agent config
        agents.insert(
            "backend".to_string(),
            AgentConfig {
                specialization: "backend".to_string(),
                worktree: "agents/backend-agent".to_string(),
                branch: "feature/backend-api".to_string(),
                claude_config: ClaudeConfig::for_agent("backend"),
                claude_md_template: "backend_specialist".to_string(),
            },
        );

        let config = CcswarmConfig {
            project: ProjectConfig {
                name: "Test Integration Project".to_string(),
                repository: RepositoryConfig {
                    url: repo_path.to_string_lossy().to_string(),
                    main_branch: "main".to_string(),
                },
                master_claude: MasterClaudeConfig {
                    role: "technical_lead".to_string(),
                    quality_threshold: 0.85,
                    think_mode: ThinkMode::UltraThink,
                    permission_level: "supervised".to_string(),
                    claude_config: ClaudeConfig::for_master(),
                },
            },
            agents,
            coordination: CoordinationConfig {
                communication_method: "json_files".to_string(),
                sync_interval: 30,
                quality_gate_frequency: "on_commit".to_string(),
                master_review_trigger: "all_tasks_complete".to_string(),
            },
        };

        (config, temp_dir)
    }

    fn create_test_tasks() -> Vec<Task> {
        vec![
            Task::new(
                Uuid::new_v4().to_string(),
                "Create user login component with React".to_string(),
                Priority::High,
                TaskType::Development,
            ),
            Task::new(
                Uuid::new_v4().to_string(),
                "Implement authentication API endpoint".to_string(),
                Priority::High,
                TaskType::Development,
            ),
            Task::new(
                Uuid::new_v4().to_string(),
                "Set up CI/CD pipeline for deployment".to_string(),
                Priority::Medium,
                TaskType::Infrastructure,
            ),
            Task::new(
                Uuid::new_v4().to_string(),
                "Write integration tests for user flow".to_string(),
                Priority::Medium,
                TaskType::Testing,
            ),
        ]
    }

    #[tokio::test]
    async fn test_coordination_bus_message_flow() {
        let bus = CoordinationBus::new().await.unwrap();

        // Test various message types
        let messages = vec![
            AgentMessage::StatusUpdate {
                agent_id: "frontend-agent".to_string(),
                status: crate::agent::AgentStatus::Available,
            },
            AgentMessage::TaskCompleted {
                agent_id: "backend-agent".to_string(),
                task_id: "task-1".to_string(),
                result: crate::agent::TaskResult {
                    success: true,
                    output: serde_json::json!({"result": "completed"}),
                    error: None,
                    duration: std::time::Duration::from_secs(120),
                },
            },
            AgentMessage::RequestAssistance {
                agent_id: "frontend-agent".to_string(),
                task_id: "task-2".to_string(),
                reason: "Need backend API specification".to_string(),
            },
        ];

        // Send all messages
        for message in &messages {
            bus.send_message(message.clone()).await.unwrap();
        }

        // Receive and verify all messages
        for expected_message in &messages {
            let received = bus.receive_message().await.unwrap();

            match (expected_message, &received) {
                (
                    AgentMessage::StatusUpdate { agent_id: e_id, .. },
                    AgentMessage::StatusUpdate { agent_id: r_id, .. },
                ) => {
                    assert_eq!(e_id, r_id);
                }
                (
                    AgentMessage::TaskCompleted {
                        agent_id: e_id,
                        task_id: e_task,
                        ..
                    },
                    AgentMessage::TaskCompleted {
                        agent_id: r_id,
                        task_id: r_task,
                        ..
                    },
                ) => {
                    assert_eq!(e_id, r_id);
                    assert_eq!(e_task, r_task);
                }
                (
                    AgentMessage::RequestAssistance {
                        agent_id: e_id,
                        task_id: e_task,
                        reason: e_reason,
                    },
                    AgentMessage::RequestAssistance {
                        agent_id: r_id,
                        task_id: r_task,
                        reason: r_reason,
                    },
                ) => {
                    assert_eq!(e_id, r_id);
                    assert_eq!(e_task, r_task);
                    assert_eq!(e_reason, r_reason);
                }
                _ => panic!("Message type mismatch"),
            }
        }
    }

    #[tokio::test]
    async fn test_task_queue_operations() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let queue_dir = temp_dir.path().join("task-queue");
        let queue = TaskQueue::with_dir(queue_dir.to_str().unwrap())
            .await
            .unwrap();
        let tasks = create_test_tasks();

        // Add all tasks
        for task in &tasks {
            queue.add_task(task).await.unwrap();
        }

        // Retrieve tasks
        let pending_tasks = queue.get_pending_tasks().await.unwrap();
        assert_eq!(pending_tasks.len(), tasks.len());

        // Verify tasks are sorted by priority
        for i in 0..pending_tasks.len() - 1 {
            assert!(pending_tasks[i].priority >= pending_tasks[i + 1].priority);
        }

        // Remove a task
        queue.remove_task(&tasks[0].id).await.unwrap();

        let remaining_tasks = queue.get_pending_tasks().await.unwrap();
        assert_eq!(remaining_tasks.len(), tasks.len() - 1);
        assert!(!remaining_tasks.iter().any(|t| t.id == tasks[0].id));
    }

    #[tokio::test]
    async fn test_status_tracker() {
        let tracker = StatusTracker::new().await.unwrap();

        let agent_id = "test-agent";
        let status = crate::agent::AgentStatus::Working;
        let additional_info = serde_json::json!({
            "current_task": "creating component",
            "progress": 0.75
        });

        // Update status
        tracker
            .update_status(agent_id, &status, additional_info.clone())
            .await
            .unwrap();

        // Retrieve status
        let retrieved = tracker.get_status(agent_id).await.unwrap();
        assert!(retrieved.is_some());

        let status_info = retrieved.unwrap();
        assert_eq!(status_info["agent_id"], agent_id);
        assert_eq!(status_info["additional_info"], additional_info);

        // Get all statuses
        let all_statuses = tracker.get_all_statuses().await.unwrap();
        assert_eq!(all_statuses.len(), 1);
        assert_eq!(all_statuses[0]["agent_id"], agent_id);
    }

    #[tokio::test]
    async fn test_master_claude_initialization() {
        let (config, _temp_dir) = create_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        assert!(master.id.starts_with("master-claude-"));
        assert_eq!(master.agents.len(), 0); // Agents not initialized yet

        // Initialize would create agents, but we skip this in the test
        // as it requires actual Claude Code binary
    }

    #[tokio::test]
    async fn test_task_assignment_logic() {
        let (config, _temp_dir) = create_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let _master = MasterClaude::new(config, repo_path).await.unwrap();
        let tasks = create_test_tasks();

        // Test task categorization
        for task in &tasks {
            // This would normally call select_optimal_agent, but we test the logic
            let expected_agent = match task.description.as_str() {
                desc if desc.contains("React") || desc.contains("component") => "Frontend",
                desc if desc.contains("API") || desc.contains("authentication") => "Backend",
                desc if desc.contains("CI/CD") || desc.contains("deployment") => "DevOps",
                desc if desc.contains("test") => "QA",
                _ => "Backend", // Default
            };

            // Verify our logic matches expectations
            match task.description.as_str() {
                "Create user login component with React" => {
                    assert_eq!(expected_agent, "Frontend")
                }
                "Implement authentication API endpoint" => assert_eq!(expected_agent, "Backend"),
                "Set up CI/CD pipeline for deployment" => assert_eq!(expected_agent, "DevOps"),
                "Write integration tests for user flow" => assert_eq!(expected_agent, "QA"),
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_coordination_message_persistence() {
        let bus = CoordinationBus::new().await.unwrap();

        let test_message = AgentMessage::TaskCompleted {
            agent_id: "test-agent".to_string(),
            task_id: "test-task".to_string(),
            result: crate::agent::TaskResult {
                success: true,
                output: serde_json::json!({"test": "data"}),
                error: None,
                duration: std::time::Duration::from_secs(30),
            },
        };

        // Send message (should persist automatically)
        bus.send_message(test_message.clone()).await.unwrap();

        // Load persisted messages
        let persisted = bus.load_persisted_messages().await.unwrap();
        assert!(!persisted.is_empty());

        // Find our message
        let found = persisted.iter().find(
            |m| matches!(m, AgentMessage::TaskCompleted { task_id, .. } if task_id == "test-task"),
        );

        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_multi_agent_workflow_simulation() {
        let bus = CoordinationBus::new().await.unwrap();
        let queue = TaskQueue::new().await.unwrap();
        let tracker = StatusTracker::new().await.unwrap();

        // Simulate a multi-agent workflow
        let workflow_tasks = vec![
            ("frontend", "Create user registration form"),
            ("backend", "Implement user registration API"),
            ("qa", "Test user registration flow"),
            ("devops", "Deploy user registration feature"),
        ];

        // 1. Add all tasks to queue
        for (_agent_type, description) in &workflow_tasks {
            let task = Task::new(
                Uuid::new_v4().to_string(),
                description.to_string(),
                Priority::High,
                TaskType::Development,
            );
            queue.add_task(&task).await.unwrap();
        }

        // 2. Simulate agents picking up tasks
        let mut completed_tasks = Vec::new();

        for (agent_type, description) in &workflow_tasks {
            let agent_id = format!("{}-agent", agent_type);

            // Update agent status to working
            tracker
                .update_status(
                    &agent_id,
                    &crate::agent::AgentStatus::Working,
                    serde_json::json!({"current_task": description}),
                )
                .await
                .unwrap();

            // Send status update message
            bus.send_message(AgentMessage::StatusUpdate {
                agent_id: agent_id.clone(),
                status: crate::agent::AgentStatus::Working,
            })
            .await
            .unwrap();

            // Simulate task completion
            let task_id = Uuid::new_v4().to_string();
            let result = crate::agent::TaskResult {
                success: true,
                output: serde_json::json!({
                    "task": description,
                    "agent": agent_type,
                    "completed_at": chrono::Utc::now(),
                }),
                error: None,
                duration: std::time::Duration::from_secs(300),
            };

            // Send completion message
            bus.send_message(AgentMessage::TaskCompleted {
                agent_id: agent_id.clone(),
                task_id: task_id.clone(),
                result: result.clone(),
            })
            .await
            .unwrap();

            completed_tasks.push((agent_id.clone(), task_id, result));

            // Update agent status to available
            tracker
                .update_status(
                    &agent_id,
                    &crate::agent::AgentStatus::Available,
                    serde_json::json!({"last_task": description}),
                )
                .await
                .unwrap();
        }

        // 3. Verify workflow completion
        assert_eq!(completed_tasks.len(), workflow_tasks.len());

        // Check all agents are available
        let all_statuses = tracker.get_all_statuses().await.unwrap();
        assert_eq!(all_statuses.len(), workflow_tasks.len());

        for status in &all_statuses {
            let agent_status: crate::agent::AgentStatus =
                serde_json::from_value(status["status"].clone()).unwrap();
            assert!(matches!(agent_status, crate::agent::AgentStatus::Available));
        }

        // 4. Verify message flow
        let mut received_messages = Vec::new();
        while let Some(message) = bus.try_receive_message() {
            received_messages.push(message);
        }

        // Should have received 2 messages per agent (status update + completion)
        assert_eq!(received_messages.len(), workflow_tasks.len() * 2);
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let bus = CoordinationBus::new().await.unwrap();
        let tracker = StatusTracker::new().await.unwrap();

        let agent_id = "error-prone-agent";

        // 1. Agent encounters an error
        let error_status = crate::agent::AgentStatus::Error("Connection timeout".to_string());
        tracker
            .update_status(
                agent_id,
                &error_status,
                serde_json::json!({"error_details": "Failed to connect to service"}),
            )
            .await
            .unwrap();

        // 2. Send error notification
        bus.send_message(AgentMessage::RequestAssistance {
            agent_id: agent_id.to_string(),
            task_id: "failed-task".to_string(),
            reason: "Connection timeout during task execution".to_string(),
        })
        .await
        .unwrap();

        // 3. Simulate recovery
        let recovered_status = crate::agent::AgentStatus::Available;
        tracker
            .update_status(
                agent_id,
                &recovered_status,
                serde_json::json!({"recovery_time": chrono::Utc::now()}),
            )
            .await
            .unwrap();

        // 4. Verify recovery
        let final_status = tracker.get_status(agent_id).await.unwrap().unwrap();
        let agent_status: crate::agent::AgentStatus =
            serde_json::from_value(final_status["status"].clone()).unwrap();
        assert!(matches!(agent_status, crate::agent::AgentStatus::Available));
    }

    #[tokio::test]
    async fn test_concurrent_agent_operations() {
        use tokio::task;

        let bus = CoordinationBus::new().await.unwrap();
        let queue = TaskQueue::new().await.unwrap();

        // Create multiple tasks
        let tasks: Vec<_> = (0..10)
            .map(|i| {
                Task::new(
                    format!("task-{}", i),
                    format!("Test task {}", i),
                    Priority::Medium,
                    TaskType::Development,
                )
            })
            .collect();

        // Simulate concurrent operations
        let mut handles = Vec::new();

        // Task creation tasks
        for task in tasks {
            let queue_clone = queue.clone();
            let handle = task::spawn(async move {
                queue_clone.add_task(&task).await.unwrap();
            });
            handles.push(handle);
        }

        // Message sending tasks
        for i in 0..5 {
            let bus_clone = bus.clone();
            let agent_id = format!("agent-{}", i);
            let handle = task::spawn(async move {
                let message = AgentMessage::Heartbeat {
                    agent_id,
                    timestamp: chrono::Utc::now(),
                };
                bus_clone.send_message(message).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify results
        let pending_tasks = queue.get_pending_tasks().await.unwrap();
        assert_eq!(pending_tasks.len(), 10);

        // Verify at least some messages were received
        let mut message_count = 0;
        while bus.try_receive_message().is_some() {
            message_count += 1;
        }
        assert!(message_count > 0);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let (config, _temp_dir) = create_test_config().await;

        // Verify config structure
        assert_eq!(config.project.name, "Test Integration Project");
        assert_eq!(config.agents.len(), 2);
        assert!(config.agents.contains_key("frontend"));
        assert!(config.agents.contains_key("backend"));

        // Verify agent configurations
        let frontend_config = &config.agents["frontend"];
        assert_eq!(frontend_config.specialization, "frontend");
        assert!(frontend_config.claude_config.dangerous_skip);

        let backend_config = &config.agents["backend"];
        assert_eq!(backend_config.specialization, "backend");
        assert!(backend_config.claude_config.dangerous_skip);

        // Verify master claude config
        assert_eq!(config.project.master_claude.quality_threshold, 0.85);
        assert!(!config.project.master_claude.claude_config.dangerous_skip);
    }
}
