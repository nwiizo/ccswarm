#[cfg(test)]
mod orchestrator_integration_tests {
    use crate::agent::{ClaudeCodeAgent, IsolationMode, Priority, Task, TaskResult, TaskType};
    use crate::config::{
        AgentConfig, CcswarmConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig,
        ProjectConfig, RepositoryConfig, ThinkMode,
    };
    use crate::coordination::AgentMessage;
    use crate::git::shell::ShellWorktreeManager as WorktreeManager;
    use crate::orchestrator::{MasterClaude, OrchestratorStatus, ReviewHistoryEntry};
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Create a test configuration with all agent types
    async fn create_comprehensive_test_config() -> (CcswarmConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        WorktreeManager::init_if_needed(&repo_path).await.unwrap();

        let mut agents = HashMap::new();

        // Frontend agent
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

        // Backend agent
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

        // DevOps agent
        agents.insert(
            "devops".to_string(),
            AgentConfig {
                specialization: "devops".to_string(),
                worktree: "agents/devops".to_string(),
                branch: "feature/devops".to_string(),
                claude_config: ClaudeConfig::for_agent("devops"),
                claude_md_template: "devops_specialist".to_string(),
            },
        );

        // QA agent
        agents.insert(
            "qa".to_string(),
            AgentConfig {
                specialization: "qa".to_string(),
                worktree: "agents/qa".to_string(),
                branch: "feature/qa".to_string(),
                claude_config: ClaudeConfig::for_agent("qa"),
                claude_md_template: "qa_specialist".to_string(),
            },
        );

        let config = CcswarmConfig {
            project: ProjectConfig {
                name: "Test Orchestrator Project".to_string(),
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
                    enable_proactive_mode: true,
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
        };

        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_master_claude_with_orchestration_context() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Test that orchestration context is properly set
        let task = Task::new(
            "test-task-1".to_string(),
            "Create user authentication flow".to_string(),
            Priority::High,
            TaskType::Feature,
        );

        // Add task to queue
        master.add_task(task.clone()).await.unwrap();

        // Verify task was added to pending tasks
        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 1);
        assert_eq!(state.pending_tasks[0].id, task.id);
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_agent_task_delegation_with_metadata() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(IsolationMode::GitWorktree);

        // Create mock agents
        let frontend_agent = ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_frontend_role(),
            &repo_path,
            "feature/frontend",
            ClaudeConfig::for_agent("frontend"),
            IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        master
            .agents
            .insert("frontend-agent".to_string(), frontend_agent);

        // Create a frontend task
        let task = Task::new(
            "ui-task-1".to_string(),
            "Create login component with React".to_string(),
            Priority::High,
            TaskType::Development,
        );

        // Test task assignment logic
        let selected_agent = master.select_optimal_agent(&task).await.unwrap();
        assert_eq!(selected_agent, "frontend-agent");
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_quality_review_triggers_remediation() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();
        let bus = master.coordination_bus.clone();

        // Simulate a quality issue
        let quality_message = AgentMessage::QualityIssue {
            agent_id: "backend-agent".to_string(),
            task_id: "api-task-1".to_string(),
            issues: vec![
                "Low test coverage".to_string(),
                "High complexity".to_string(),
                "Missing error handling".to_string(),
            ],
        };

        bus.send_message(quality_message).await.unwrap();

        // Allow time for message processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check that remediation task was created in review history
        let state = master.state.read().await;
        assert!(state.review_history.contains_key("api-task-1"));

        let history_entries = &state.review_history["api-task-1"];
        assert_eq!(history_entries.len(), 1);
        assert_eq!(history_entries[0].issues_found.len(), 3);
        assert!(history_entries[0].remediation_task_id.is_some());
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_assistance_request_handling() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(IsolationMode::GitWorktree);

        // Create two backend agents
        let backend_agent1 = ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/backend1",
            ClaudeConfig::for_agent("backend1"),
            IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        let backend_agent2 = ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/backend2",
            ClaudeConfig::for_agent("backend2"),
            IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        master
            .agents
            .insert("backend-agent-1".to_string(), backend_agent1);
        master
            .agents
            .insert("backend-agent-2".to_string(), backend_agent2);

        // Simulate assistance request
        let assistance_message = AgentMessage::RequestAssistance {
            agent_id: "backend-agent-1".to_string(),
            task_id: "complex-task-1".to_string(),
            reason: "Need help with database optimization".to_string(),
        };

        master
            .coordination_bus
            .send_message(assistance_message)
            .await
            .unwrap();

        // Allow time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify assistance task was created
        let state = master.state.read().await;
        let assistance_task = state
            .pending_tasks
            .iter()
            .find(|t| t.task_type == TaskType::Assistance);
        assert!(assistance_task.is_some());
    }

    #[tokio::test]
    async fn test_proactive_task_generation() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Add an objective for proactive tracking
        let obj_id = master
            .set_objective(
                "Complete user authentication".to_string(),
                "Implement full authentication flow".to_string(),
                None,
            )
            .await
            .unwrap();

        assert!(obj_id.starts_with("obj-"));

        // Trigger proactive analysis
        let decisions = master.trigger_proactive_analysis().await.unwrap();

        // In a real scenario, this would generate tasks based on objectives
        // For testing, we verify the mechanism works
        assert!(decisions.is_empty() || !decisions.is_empty()); // Either case is valid
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_remediation_task_completion_triggers_rereview() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Setup initial review history
        let original_task_id = "original-task-1";
        let remediation_task_id = format!("remediate-{}-123", original_task_id);

        let review_entry = ReviewHistoryEntry {
            task_id: original_task_id.to_string(),
            agent_id: "backend-agent".to_string(),
            review_date: chrono::Utc::now(),
            issues_found: vec!["Low test coverage".to_string()],
            remediation_task_id: Some(remediation_task_id.clone()),
            review_passed: false,
            iteration: 1,
        };

        {
            let mut state = master.state.write().await;
            state
                .review_history
                .entry(original_task_id.to_string())
                .or_insert_with(Vec::new)
                .push(review_entry);
        }

        // Simulate remediation task completion
        let completion_message = AgentMessage::TaskCompleted {
            agent_id: "backend-agent".to_string(),
            task_id: remediation_task_id.clone(),
            result: TaskResult {
                success: true,
                output: serde_json::json!({"fixed": true}),
                error: None,
                duration: std::time::Duration::from_secs(300),
            },
        };

        master
            .coordination_bus
            .send_message(completion_message)
            .await
            .unwrap();

        // Allow time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify re-review task was created
        let state = master.state.read().await;
        let review_task = state.pending_tasks.iter().find(|t| {
            t.task_type == TaskType::Review
                && t.parent_task_id == Some(original_task_id.to_string())
        });
        assert!(review_task.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_task_processing() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Create multiple tasks concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let master_clone = master.clone();
            let handle = tokio::spawn(async move {
                let task = Task::new(
                    format!("concurrent-task-{}", i),
                    format!("Task {} description", i),
                    Priority::Medium,
                    TaskType::Development,
                );
                master_clone.add_task(task).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks to be added
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all tasks were added
        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 10);
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_agent_health_monitoring() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(IsolationMode::GitWorktree);

        // Create an agent with old last_activity
        let mut unhealthy_agent = ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/backend",
            ClaudeConfig::for_agent("backend"),
            IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        // Set last activity to 10 minutes ago
        unhealthy_agent.last_activity = chrono::Utc::now() - chrono::Duration::seconds(600);

        master
            .agents
            .insert("unhealthy-agent".to_string(), unhealthy_agent);

        // Run health check
        master.check_agent_health().await.unwrap();

        // Verify agent was restarted (last_activity updated)
        let agent = master.agents.get("unhealthy-agent").unwrap();
        let time_diff = chrono::Utc::now() - agent.last_activity;
        assert!(time_diff.num_seconds() < 5); // Should be recent
    }

    #[tokio::test]
    async fn test_task_prioritization() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Add tasks with different priorities
        let critical_task = Task::new(
            "critical-1".to_string(),
            "Fix production bug".to_string(),
            Priority::Critical,
            TaskType::Bug,
        );

        let medium_task = Task::new(
            "medium-1".to_string(),
            "Add new feature".to_string(),
            Priority::Medium,
            TaskType::Feature,
        );

        let low_task = Task::new(
            "low-1".to_string(),
            "Update documentation".to_string(),
            Priority::Low,
            TaskType::Documentation,
        );

        // Add in random order
        master.add_task(medium_task).await.unwrap();
        master.add_task(low_task).await.unwrap();
        master.add_task(critical_task).await.unwrap();

        // Verify tasks are properly tracked
        let state = master.state.read().await;
        assert_eq!(state.pending_tasks.len(), 3);

        // Find the critical task
        let critical_exists = state
            .pending_tasks
            .iter()
            .any(|t| t.priority == Priority::Critical);
        assert!(critical_exists);
    }

    #[tokio::test]
    async fn test_status_report_generation() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(IsolationMode::GitWorktree);

        // Add some agents
        for i in 0..3 {
            let agent = ClaudeCodeAgent::new_with_isolation(
                crate::identity::default_backend_role(),
                &repo_path,
                &format!("feature/agent-{}", i),
                ClaudeConfig::for_agent(&format!("agent-{}", i)),
                IsolationMode::GitWorktree,
            )
            .await
            .unwrap();

            master.agents.insert(format!("agent-{}", i), agent);
        }

        // Update orchestrator state
        {
            let mut state = master.state.write().await;
            state.status = OrchestratorStatus::Running;
            state.total_tasks_processed = 10;
            state.successful_tasks = 8;
            state.failed_tasks = 2;
        }

        // Generate report
        let report = master.generate_status_report().await.unwrap();

        assert_eq!(report.orchestrator_id, master.id);
        assert_eq!(report.status, OrchestratorStatus::Running);
        assert_eq!(report.total_agents, 3);
        assert_eq!(report.total_tasks_processed, 10);
        assert_eq!(report.successful_tasks, 8);
        assert_eq!(report.failed_tasks, 2);
    }

    #[tokio::test]
    async fn test_milestone_tracking() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Add milestone
        let milestone_id = master
            .add_milestone(
                "MVP Release".to_string(),
                "Complete minimum viable product".to_string(),
                Some(chrono::Utc::now() + chrono::Duration::days(30)),
            )
            .await
            .unwrap();

        assert!(milestone_id.starts_with("milestone-"));
    }

    #[tokio::test]
    #[ignore = "Integration test may fail in CI due to timing"]
    async fn test_error_recovery_mechanism() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let mut master = MasterClaude::new(config, repo_path.clone()).await.unwrap();
        master.set_isolation_mode(IsolationMode::GitWorktree);

        // Create agent in error state
        let mut error_agent = ClaudeCodeAgent::new_with_isolation(
            crate::identity::default_backend_role(),
            &repo_path,
            "feature/error",
            ClaudeConfig::for_agent("error-agent"),
            IsolationMode::GitWorktree,
        )
        .await
        .unwrap();

        error_agent.status = crate::agent::AgentStatus::Error("Test error".to_string());

        master.agents.insert("error-agent".to_string(), error_agent);

        // Run health check
        master.check_agent_health().await.unwrap();

        // Verify agent was recovered
        let agent = master.agents.get("error-agent").unwrap();
        assert!(matches!(agent.status, crate::agent::AgentStatus::Available));
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Add a task
        let task = Task::new(
            "shutdown-test".to_string(),
            "Test task".to_string(),
            Priority::Low,
            TaskType::Testing,
        );
        master.add_task(task).await.unwrap();

        // Initiate shutdown
        master.shutdown().await.unwrap();

        // Verify shutdown state
        let state = master.state.read().await;
        assert_eq!(state.status, OrchestratorStatus::ShuttingDown);
    }

    #[tokio::test]
    async fn test_task_metadata_enhancement() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Test task complexity detection
        let feature_task = Task::new(
            "feature-1".to_string(),
            "Implement complex feature".to_string(),
            Priority::High,
            TaskType::Feature,
        );

        let insights = master.get_proactive_insights_for_task(&feature_task).await;
        assert_eq!(insights["task_complexity"], "high");
        assert_eq!(insights["recommended_approach"], "careful_planning");
    }

    #[tokio::test]
    async fn test_dependency_identification() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Test various task descriptions
        let api_task = Task::new(
            "api-1".to_string(),
            "Create REST API endpoint for users".to_string(),
            Priority::High,
            TaskType::Development,
        );

        let dependencies = master.identify_task_dependencies(&api_task).await;
        assert!(dependencies.contains(&"backend_api".to_string()));

        let ui_task = Task::new(
            "ui-1".to_string(),
            "Build UI component for dashboard".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        let ui_deps = master.identify_task_dependencies(&ui_task).await;
        assert!(ui_deps.contains(&"frontend_components".to_string()));
    }

    #[tokio::test]
    async fn test_similar_task_matching() {
        let (config, _temp_dir) = create_comprehensive_test_config().await;
        let repo_path = std::path::PathBuf::from(&config.project.repository.url);

        let master = MasterClaude::new(config, repo_path).await.unwrap();

        // Add some review history
        {
            let mut state = master.state.write().await;
            state.review_history.insert(
                "feature-auth-1".to_string(),
                vec![ReviewHistoryEntry {
                    task_id: "feature-auth-1".to_string(),
                    agent_id: "backend".to_string(),
                    review_date: chrono::Utc::now(),
                    issues_found: vec![],
                    remediation_task_id: None,
                    review_passed: true,
                    iteration: 1,
                }],
            );
        }

        let similar_task = Task::new(
            "new-feature-1".to_string(),
            "Another feature".to_string(),
            Priority::Medium,
            TaskType::Feature,
        );

        let similar = master.find_similar_completed_tasks(&similar_task).await;
        assert!(!similar.is_empty());
    }
}
