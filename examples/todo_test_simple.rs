/// Enhanced TODO Service Creation Test with Session Management
///
/// This program uses the ccswarm system with session management, monitoring,
/// and multi-provider support to simulate TODO application development.
/// Filesystem-based operation without Git dependencies.
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

// ccswarm library imports with enhanced features
use ccswarm::agent::simple::SimpleClaudeAgent;
use ccswarm::agent::{AgentStatus, Priority, Task, TaskType};
use ccswarm::auto_accept::{AutoAcceptConfig, AutoAcceptEngine};
use ccswarm::config::ClaudeConfig;
use ccswarm::coordination::{AgentMessage, CoordinationBus, StatusTracker, TaskQueue};
use ccswarm::identity::{
    default_backend_role, default_devops_role, default_frontend_role, default_qa_role,
};
use ccswarm::monitoring::{MonitoringSystem, OutputType};
use ccswarm::session::{AgentSession, SessionManager};
use ccswarm::streaming::{StreamConfig, StreamingManager};
use ccswarm::workspace::SimpleWorkspaceManager;

#[tokio::main]
async fn main() -> Result<()> {
    // ログ設定
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🚀 ccswarm Enhanced TODO Service Test with Session Management");

    // テスト用プロジェクトディレクトリを作成
    let project_dir = PathBuf::from("./test_todo_project");
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir).await?;
    }
    fs::create_dir_all(&project_dir).await?;

    // ワークスペース管理を初期化
    let workspace_manager = SimpleWorkspaceManager::new(project_dir.clone());
    workspace_manager.init_if_needed().await?;

    // Initialize enhanced coordination systems
    let coordination_bus = CoordinationBus::new().await?;
    let task_queue = TaskQueue::new().await?;
    let status_tracker = StatusTracker::new().await?;
    let session_manager = SessionManager::new()?;
    let monitoring_system = std::sync::Arc::new(MonitoringSystem::new());
    let stream_config = StreamConfig {
        buffer_size: 1000,
        refresh_rate_ms: 100,
        max_line_length: 2000,
        enable_filtering: true,
        enable_highlighting: true,
    };
    let _streaming_manager = StreamingManager::new(monitoring_system.clone(), stream_config);
    let auto_accept_engine = AutoAcceptEngine::new(AutoAcceptConfig::default());

    info!("📋 Defining TODO application tasks with enhanced features...");

    // TODOアプリケーション開発タスクを定義
    let todo_tasks = vec![
        Task::new(
            "todo-1".to_string(),
            "Create React TODO frontend with add/delete functionality".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_details("Build a React component with state management for TODO items".to_string()),
        Task::new(
            "todo-2".to_string(),
            "Implement REST API for TODO backend with CRUD operations".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_details("Create Express.js API with endpoints for TODO management".to_string()),
        Task::new(
            "todo-3".to_string(),
            "Set up database schema for TODO storage".to_string(),
            Priority::Medium,
            TaskType::Development,
        )
        .with_details("Design and implement PostgreSQL schema for TODO items".to_string()),
        Task::new(
            "todo-4".to_string(),
            "Create Docker deployment configuration".to_string(),
            Priority::Medium,
            TaskType::Infrastructure,
        )
        .with_details("Setup containerization for frontend and backend services".to_string()),
        Task::new(
            "todo-5".to_string(),
            "Write comprehensive tests for TODO functionality".to_string(),
            Priority::Medium,
            TaskType::Testing,
        )
        .with_details("Create unit and integration tests for all TODO features".to_string()),
    ];

    // タスクをキューに追加
    for task in &todo_tasks {
        task_queue.add_task(task).await?;
        info!("📝 タスクを追加: {}", task.description);
    }

    info!("🤖 Creating specialized agents with session management...");

    // Create specialized agents with enhanced session management
    let mut agents: Vec<(String, SimpleClaudeAgent, AgentSession)> = vec![];

    // Frontend Agent with session
    let mut frontend_agent = SimpleClaudeAgent::new(
        default_frontend_role(),
        &project_dir,
        ClaudeConfig::for_agent("frontend"),
    )
    .await?;
    frontend_agent.initialize(&workspace_manager).await?;

    let frontend_session = session_manager.create_session(
        "frontend-test-001".to_string(),
        default_frontend_role(),
        project_dir.join("frontend").to_string_lossy().to_string(),
        Some("Frontend React development".to_string()),
        true,
    )?;
    agents.push(("frontend".to_string(), frontend_agent, frontend_session));

    // Backend Agent with session
    let mut backend_agent = SimpleClaudeAgent::new(
        default_backend_role(),
        &project_dir,
        ClaudeConfig::for_agent("backend"),
    )
    .await?;
    backend_agent.initialize(&workspace_manager).await?;

    let backend_session = session_manager.create_session(
        "backend-test-001".to_string(),
        default_backend_role(),
        project_dir.join("backend").to_string_lossy().to_string(),
        Some("Backend API development".to_string()),
        true,
    )?;
    agents.push(("backend".to_string(), backend_agent, backend_session));

    // DevOps Agent with session
    let mut devops_agent = SimpleClaudeAgent::new(
        default_devops_role(),
        &project_dir,
        ClaudeConfig::for_agent("devops"),
    )
    .await?;
    devops_agent.initialize(&workspace_manager).await?;

    let devops_session = session_manager.create_session(
        "devops-test-001".to_string(),
        default_devops_role(),
        project_dir.join("devops").to_string_lossy().to_string(),
        Some("DevOps containerization".to_string()),
        false, // background mode for DevOps
    )?;
    session_manager.set_background_mode(&devops_session.id, true)?;
    agents.push(("devops".to_string(), devops_agent, devops_session));

    // QA Agent with session and auto-accept enabled
    let mut qa_agent = SimpleClaudeAgent::new(
        default_qa_role(),
        &project_dir,
        ClaudeConfig::for_agent("qa"),
    )
    .await?;
    qa_agent.initialize(&workspace_manager).await?;

    let qa_session = session_manager.create_session(
        "qa-test-001".to_string(),
        default_qa_role(),
        project_dir.join("qa").to_string_lossy().to_string(),
        Some("QA testing and validation".to_string()),
        true,
    )?;
    agents.push(("qa".to_string(), qa_agent, qa_session));

    info!(
        "✅ {} agents initialized with sessions and monitoring",
        agents.len()
    );

    // Register agents with enhanced tracking and monitoring
    for (agent_type, agent, session) in &agents {
        status_tracker
            .update_status(
                &agent.identity.agent_id,
                &agent.status,
                json!({
                    "agent_type": agent_type,
                    "specialization": agent.identity.specialization.name(),
                    "workspace": agent.workspace_path,
                    "session_id": session.id,
                    "session_status": session.status,
                    "background_mode": session.background_mode,
                    "auto_accept": session.auto_accept,
                    "initialized_at": agent.last_activity
                }),
            )
            .await?;

        // Register with monitoring system
        monitoring_system
            .register_agent(session.id.clone())
            .unwrap();
    }

    info!("🎯 Starting enhanced TODO application development simulation...");

    // タスク実行シミュレーション
    let pending_tasks = task_queue.get_pending_tasks().await?;
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    for task in pending_tasks {
        info!("📋 タスク実行中: {}", task.description);

        // Select appropriate agent with session management
        let agent_index = select_agent_for_task(&task, &agents);

        if let Some(index) = agent_index {
            let (agent_type, ref mut agent, ref session) = &mut agents[index];

            // Check auto-accept capability
            let can_auto_accept = if session.auto_accept {
                // Create an operation for the task
                let operation = ccswarm::auto_accept::Operation {
                    operation_type: ccswarm::auto_accept::OperationType::EditFile,
                    description: task.description.clone(),
                    affected_files: vec![],
                    commands: vec![],
                    risk_level: 3, // Default medium risk
                    reversible: true,
                    task: Some(task.clone()),
                };

                match auto_accept_engine.should_auto_accept(&operation) {
                    Ok(ccswarm::auto_accept::AutoAcceptDecision::Accept(_)) => true,
                    _ => false,
                }
            } else {
                false
            };

            if can_auto_accept {
                info!("🤖 Auto-accepting task: {}", task.description);
            }

            // Log task start to monitoring
            monitoring_system
                .add_output(
                    session.id.clone(),
                    agent_type.clone(),
                    OutputType::Info,
                    format!(
                        "Starting task: {} ({})",
                        task.description,
                        if can_auto_accept { "AUTO" } else { "MANUAL" }
                    ),
                    Some(task.id.clone()),
                    session.id.clone(),
                )
                .unwrap();

            // Update status before execution
            agent.update_status(AgentStatus::Working);
            status_tracker
                .update_status(
                    &agent.identity.agent_id,
                    &agent.status,
                    json!({
                        "current_task": task.id,
                        "task_description": task.description,
                        "auto_accept": can_auto_accept,
                        "session_id": session.id,
                        "started_at": Utc::now()
                    }),
                )
                .await?;

            // Send coordination message
            coordination_bus
                .send_message(AgentMessage::StatusUpdate {
                    agent_id: agent.identity.agent_id.clone(),
                    status: AgentStatus::Working,
                })
                .await?;

            // Execute task with monitoring
            match agent.execute_task(task.clone()).await {
                Ok(result) => {
                    if result.success {
                        info!(
                            "✅ {} agent completed task: {}",
                            agent_type, task.description
                        );
                        completed_tasks += 1;

                        // Log success to monitoring
                        monitoring_system
                            .add_output(
                                session.id.clone(),
                                agent_type.clone(),
                                OutputType::Info,
                                format!("Task completed: {}", task.description),
                                Some(task.id.clone()),
                                session.id.clone(),
                            )
                            .unwrap();

                        // Send completion message
                        coordination_bus
                            .send_message(AgentMessage::TaskCompleted {
                                agent_id: agent.identity.agent_id.clone(),
                                task_id: task.id.clone(),
                                result: result.clone(),
                            })
                            .await?;

                        // Remove task from queue
                        task_queue.remove_task(&task.id).await?;
                    } else {
                        let error_msg = result.error.unwrap_or_default();
                        warn!("❌ Task execution failed: {}", error_msg);
                        failed_tasks += 1;

                        // Log error to monitoring
                        monitoring_system
                            .add_output(
                                session.id.clone(),
                                agent_type.clone(),
                                OutputType::Error,
                                format!("Task failed: {}", error_msg),
                                Some(task.id.clone()),
                                session.id.clone(),
                            )
                            .unwrap();
                    }
                }
                Err(e) => {
                    warn!("❌ Task execution error: {}", e);
                    failed_tasks += 1;

                    // Log error to monitoring
                    monitoring_system
                        .add_output(
                            session.id.clone(),
                            agent_type.clone(),
                            OutputType::Error,
                            format!("Execution error: {}", e),
                            Some(task.id.clone()),
                            session.id.clone(),
                        )
                        .unwrap();
                }
            }

            // Return agent status to available
            agent.update_status(AgentStatus::Available);
            status_tracker
                .update_status(
                    &agent.identity.agent_id,
                    &agent.status,
                    json!({
                        "last_task": task.id,
                        "completed_at": Utc::now(),
                        "session_id": session.id
                    }),
                )
                .await?;
        } else {
            warn!("⚠️ No suitable agent found for task: {}", task.description);
            failed_tasks += 1;
        }

        // 少し待機
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    info!("📊 Enhanced TODO application development completed!");
    info!("✅ Completed tasks: {}", completed_tasks);
    info!("❌ Failed tasks: {}", failed_tasks);

    // Display session statistics
    let session_list = session_manager.list_sessions();
    info!("🔄 Active sessions: {}", session_list.len());

    // Display monitoring statistics
    let stats = monitoring_system.get_stats();
    info!(
        "📈 Monitoring stats: {} total entries across {} agents",
        stats.total_entries, stats.active_streams
    );

    // Final status report with session information
    let final_statuses = status_tracker.get_all_statuses().await?;
    info!("📈 Final agent status with sessions:");
    for status in final_statuses {
        info!(
            "  {} - {} (Session: {})",
            status["agent_id"],
            status["status"],
            status.get("session_id").unwrap_or(&json!("N/A"))
        );
    }

    // ワークスペース情報を表示
    let workspaces = workspace_manager.list_workspaces().await?;
    info!("📁 作成されたワークスペース:");
    for workspace in workspaces {
        info!("  {} - {}", workspace.agent_id, workspace.path.display());
    }

    // 調整バスを閉じる
    coordination_bus.close().await?;

    info!("🎉 ccswarm Enhanced TODO Service Test completed!");

    Ok(())
}

/// Select appropriate agent for task with session awareness
fn select_agent_for_task(
    task: &Task,
    agents: &[(String, SimpleClaudeAgent, AgentSession)],
) -> Option<usize> {
    let description = task.description.to_lowercase();

    for (index, (agent_type, _, session)) in agents.iter().enumerate() {
        let matches = match agent_type.as_str() {
            "frontend" => {
                description.contains("react")
                    || description.contains("frontend")
                    || description.contains("ui")
                    || description.contains("component")
            }
            "backend" => {
                description.contains("api")
                    || description.contains("backend")
                    || description.contains("server")
                    || description.contains("database")
                    || description.contains("schema")
            }
            "devops" => {
                description.contains("docker")
                    || description.contains("deploy")
                    || description.contains("infrastructure")
                    || description.contains("container")
            }
            "qa" => {
                description.contains("test")
                    || description.contains("quality")
                    || description.contains("validation")
            }
            _ => false,
        };

        if matches && matches!(session.status, ccswarm::session::SessionStatus::Active) {
            return Some(index);
        }
    }

    None
}
