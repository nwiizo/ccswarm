/// TODOサービス作成テスト（Git不使用のシンプル版）
/// 
/// このプログラムはccswarmシステムを使用してTODOアプリケーションの開発を
/// シミュレートします。Gitに依存せず、ファイルシステムベースで動作します。

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

// ccswarmライブラリのインポート（Git機能を除く）
use ccswarm::agent::{Priority, Task, TaskType, AgentStatus};
use ccswarm::agent::simple::SimpleClaudeAgent;
use ccswarm::config::ClaudeConfig;
use ccswarm::coordination::{AgentMessage, CoordinationBus, StatusTracker, TaskQueue};
use ccswarm::identity::{default_frontend_role, default_backend_role, default_devops_role, default_qa_role};
use ccswarm::workspace::SimpleWorkspaceManager;

#[tokio::main]
async fn main() -> Result<()> {
    // ログ設定
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 ccswarm TODOサービス作成テスト開始");

    // テスト用プロジェクトディレクトリを作成
    let project_dir = PathBuf::from("./test_todo_project");
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir).await?;
    }
    fs::create_dir_all(&project_dir).await?;

    // ワークスペース管理を初期化
    let workspace_manager = SimpleWorkspaceManager::new(project_dir.clone());
    workspace_manager.init_if_needed().await?;

    // 調整システムを初期化
    let coordination_bus = CoordinationBus::new().await?;
    let task_queue = TaskQueue::new().await?;
    let status_tracker = StatusTracker::new().await?;

    info!("📋 TODOアプリケーションのタスクを定義中...");

    // TODOアプリケーション開発タスクを定義
    let todo_tasks = vec![
        Task::new(
            "todo-1".to_string(),
            "Create React TODO frontend with add/delete functionality".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Build a React component with state management for TODO items".to_string()),

        Task::new(
            "todo-2".to_string(),
            "Implement REST API for TODO backend with CRUD operations".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create Express.js API with endpoints for TODO management".to_string()),

        Task::new(
            "todo-3".to_string(),
            "Set up database schema for TODO storage".to_string(),
            Priority::Medium,
            TaskType::Development,
        ).with_details("Design and implement PostgreSQL schema for TODO items".to_string()),

        Task::new(
            "todo-4".to_string(),
            "Create Docker deployment configuration".to_string(),
            Priority::Medium,
            TaskType::Infrastructure,
        ).with_details("Setup containerization for frontend and backend services".to_string()),

        Task::new(
            "todo-5".to_string(),
            "Write comprehensive tests for TODO functionality".to_string(),
            Priority::Medium,
            TaskType::Testing,
        ).with_details("Create unit and integration tests for all TODO features".to_string()),
    ];

    // タスクをキューに追加
    for task in &todo_tasks {
        task_queue.add_task(task).await?;
        info!("📝 タスクを追加: {}", task.description);
    }

    info!("🤖 専門エージェントを作成中...");

    // 専門エージェントを作成
    let mut agents: Vec<(String, SimpleClaudeAgent)> = vec![];
    
    // フロントエンドエージェント
    let mut frontend_agent = SimpleClaudeAgent::new(
        default_frontend_role(),
        &project_dir,
        ClaudeConfig::for_agent("frontend"),
    ).await?;
    frontend_agent.initialize(&workspace_manager).await?;
    agents.push(("frontend".to_string(), frontend_agent));

    // バックエンドエージェント
    let mut backend_agent = SimpleClaudeAgent::new(
        default_backend_role(),
        &project_dir,
        ClaudeConfig::for_agent("backend"),
    ).await?;
    backend_agent.initialize(&workspace_manager).await?;
    agents.push(("backend".to_string(), backend_agent));

    // DevOpsエージェント
    let mut devops_agent = SimpleClaudeAgent::new(
        default_devops_role(),
        &project_dir,
        ClaudeConfig::for_agent("devops"),
    ).await?;
    devops_agent.initialize(&workspace_manager).await?;
    agents.push(("devops".to_string(), devops_agent));

    // QAエージェント
    let mut qa_agent = SimpleClaudeAgent::new(
        default_qa_role(),
        &project_dir,
        ClaudeConfig::for_agent("qa"),
    ).await?;
    qa_agent.initialize(&workspace_manager).await?;
    agents.push(("qa".to_string(), qa_agent));

    info!("✅ {} 個のエージェントを初期化完了", agents.len());

    // エージェントステータスを追跡システムに登録
    for (agent_type, agent) in &agents {
        status_tracker.update_status(
            &agent.identity.agent_id,
            &agent.status,
            json!({
                "agent_type": agent_type,
                "specialization": agent.identity.specialization.name(),
                "workspace": agent.workspace_path,
                "initialized_at": agent.last_activity
            })
        ).await?;
    }

    info!("🎯 TODOアプリケーション開発シミュレーション開始...");

    // タスク実行シミュレーション
    let pending_tasks = task_queue.get_pending_tasks().await?;
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    for task in pending_tasks {
        info!("📋 タスク実行中: {}", task.description);

        // 適切なエージェントを選択
        let agent_index = select_agent_for_task(&task, &agents);
        
        if let Some(index) = agent_index {
            let (agent_type, ref mut agent) = &mut agents[index];
            
            // タスク実行前の状態更新
            agent.update_status(AgentStatus::Working);
            status_tracker.update_status(
                &agent.identity.agent_id,
                &agent.status,
                json!({
                    "current_task": task.id,
                    "task_description": task.description,
                    "started_at": Utc::now()
                })
            ).await?;

            // 調整バスにメッセージ送信
            coordination_bus.send_message(AgentMessage::StatusUpdate {
                agent_id: agent.identity.agent_id.clone(),
                status: AgentStatus::Working,
            }).await?;

            // タスクを実行
            match agent.execute_task(task.clone()).await {
                Ok(result) => {
                    if result.success {
                        info!("✅ {} エージェントがタスクを完了: {}", agent_type, task.description);
                        completed_tasks += 1;

                        // 完了メッセージを送信
                        coordination_bus.send_message(AgentMessage::TaskCompleted {
                            agent_id: agent.identity.agent_id.clone(),
                            task_id: task.id.clone(),
                            result: result.clone(),
                        }).await?;

                        // タスクをキューから削除
                        task_queue.remove_task(&task.id).await?;
                    } else {
                        warn!("❌ タスク実行失敗: {}", result.error.unwrap_or_default());
                        failed_tasks += 1;
                    }
                }
                Err(e) => {
                    warn!("❌ タスク実行エラー: {}", e);
                    failed_tasks += 1;
                }
            }

            // エージェントステータスを利用可能に戻す
            agent.update_status(AgentStatus::Available);
            status_tracker.update_status(
                &agent.identity.agent_id,
                &agent.status,
                json!({
                    "last_task": task.id,
                    "completed_at": Utc::now()
                })
            ).await?;
        } else {
            warn!("⚠️ タスクに適したエージェントが見つかりません: {}", task.description);
            failed_tasks += 1;
        }

        // 少し待機
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    info!("📊 TODOアプリケーション開発完了!");
    info!("✅ 完了タスク: {}", completed_tasks);
    info!("❌ 失敗タスク: {}", failed_tasks);

    // 最終ステータスレポート
    let final_statuses = status_tracker.get_all_statuses().await?;
    info!("📈 最終エージェントステータス:");
    for status in final_statuses {
        info!("  {} - {}", status["agent_id"], status["status"]);
    }

    // ワークスペース情報を表示
    let workspaces = workspace_manager.list_workspaces().await?;
    info!("📁 作成されたワークスペース:");
    for workspace in workspaces {
        info!("  {} - {}", workspace.agent_id, workspace.path.display());
    }

    // 調整バスを閉じる
    coordination_bus.close().await?;

    info!("🎉 ccswarm TODOサービステスト完了!");
    
    Ok(())
}

/// タスクに適したエージェントを選択
fn select_agent_for_task(task: &Task, agents: &[(String, SimpleClaudeAgent)]) -> Option<usize> {
    let description = task.description.to_lowercase();
    
    for (index, (agent_type, _)) in agents.iter().enumerate() {
        let matches = match agent_type.as_str() {
            "frontend" => {
                description.contains("react") || 
                description.contains("frontend") || 
                description.contains("ui") ||
                description.contains("component")
            },
            "backend" => {
                description.contains("api") || 
                description.contains("backend") || 
                description.contains("server") ||
                description.contains("database") ||
                description.contains("schema")
            },
            "devops" => {
                description.contains("docker") || 
                description.contains("deploy") || 
                description.contains("infrastructure") ||
                description.contains("container")
            },
            "qa" => {
                description.contains("test") || 
                description.contains("quality") || 
                description.contains("validation")
            },
            _ => false,
        };
        
        if matches {
            return Some(index);
        }
    }
    
    None
}