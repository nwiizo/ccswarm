use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::agent::{AgentStatus, Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::identity::{AgentIdentity, AgentRole};
use crate::workspace::SimpleWorkspaceManager;

// Orchestrator implementation removed - use main orchestrator module instead

/// Git不使用のシンプルなエージェント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleClaudeAgent {
    /// エージェントID
    pub identity: AgentIdentity,

    /// ワークスペースパス
    pub workspace_path: PathBuf,

    /// Claude設定
    pub claude_config: ClaudeConfig,

    /// 現在のステータス
    pub status: AgentStatus,

    /// 現在のタスク
    pub current_task: Option<Task>,

    /// タスク履歴
    pub task_history: Vec<(Task, TaskResult)>,

    /// 最終活動時刻
    pub last_activity: DateTime<Utc>,
}

impl SimpleClaudeAgent {
    /// 新しいシンプルエージェントを作成
    pub async fn new(
        role: AgentRole,
        workspace_root: &std::path::Path,
        claude_config: ClaudeConfig,
    ) -> Result<Self> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());
        let session_id = Uuid::new_v4().to_string();
        let workspace_path = workspace_root.join("agents").join(&agent_id);

        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role,
            workspace_path: workspace_path.clone(),
            env_vars: Self::create_env_vars(&agent_id, &session_id),
            session_id,
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        let agent = Self {
            identity,
            workspace_path,
            claude_config,
            status: AgentStatus::Initializing,
            current_task: None,
            task_history: Vec::new(),
            last_activity: Utc::now(),
        };

        Ok(agent)
    }

    /// 環境変数を作成
    fn create_env_vars(
        agent_id: &str,
        session_id: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("CCSWARM_SESSION_ID".to_string(), session_id.to_string());
        env_vars.insert(
            "CCSWARM_ROLE".to_string(),
            agent_id.split('-').next().unwrap_or("unknown").to_string(),
        );
        env_vars
    }

    /// エージェントを初期化
    pub async fn initialize(&mut self, workspace_manager: &SimpleWorkspaceManager) -> Result<()> {
        tracing::info!("Initializing simple agent: {}", self.identity.agent_id);

        // ワークスペースを作成
        workspace_manager
            .create_workspace(&self.identity.agent_id)
            .await?;

        // CLAUDE.mdを生成
        let claude_md_content = crate::agent::claude::generate_claude_md_content(&self.identity);
        workspace_manager
            .setup_claude_config(&self.identity.agent_id, &claude_md_content)
            .await?;

        self.status = AgentStatus::Available;
        self.last_activity = Utc::now();

        tracing::info!(
            "Simple agent {} initialized successfully",
            self.identity.agent_id
        );
        Ok(())
    }

    /// タスクを実行
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        tracing::info!(
            "Agent {} executing task: {}",
            self.identity.agent_id,
            task.id
        );

        self.status = AgentStatus::Working;
        self.current_task = Some(task.clone());
        self.last_activity = Utc::now();

        let start_time = std::time::Instant::now();

        // シミュレートされたタスク実行
        let result = self.simulate_task_execution(&task).await?;

        let duration = start_time.elapsed();
        let task_result = if result.is_success {
            TaskResult::success(
                serde_json::json!({
                    "agent_id": self.identity.agent_id,
                    "task_id": task.id,
                    "result": result.output,
                    "notes": result.notes
                }),
                duration,
            )
        } else {
            TaskResult::failure(result.error_message, duration)
        };

        // タスク履歴に追加
        self.task_history.push((task, task_result.clone()));
        self.current_task = None;
        self.status = AgentStatus::Available;
        self.last_activity = Utc::now();

        Ok(task_result)
    }

    /// タスク実行をシミュレート
    async fn simulate_task_execution(&self, task: &Task) -> Result<SimulatedResult> {
        // エージェントの専門性に基づいてタスクを評価
        let can_handle = self.can_handle_task(task);

        if !can_handle {
            return Ok(SimulatedResult {
                is_success: false,
                error_message: format!(
                    "Task '{}' is outside my specialization as a {} agent",
                    task.description,
                    self.identity.specialization.name()
                ),
                output: String::new(),
                notes: vec![
                    "Task delegation recommended".to_string(),
                    format!("Should be handled by appropriate specialist agent"),
                ],
            });
        }

        // 成功をシミュレート
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(SimulatedResult {
            is_success: true,
            error_message: String::new(),
            output: format!("Successfully completed: {}", task.description),
            notes: vec![
                format!("Processed by {} agent", self.identity.specialization.name()),
                "Task completed within agent boundaries".to_string(),
            ],
        })
    }

    /// タスクが処理可能かチェック
    fn can_handle_task(&self, task: &Task) -> bool {
        let role_name = self.identity.specialization.name().to_lowercase();
        let description = task.description.to_lowercase();
        let task_id = task.id.to_lowercase();

        match role_name.as_str() {
            "frontend" => {
                task_id.contains("frontend")
                    || description.contains("html")
                    || description.contains("css")
                    || description.contains("javascript")
                    || description.contains("ui")
                    || description.contains("react")
                    || description.contains("component")
                    || description.contains("frontend")
            }
            "backend" => {
                task_id.contains("backend")
                    || description.contains("node")
                    || description.contains("express")
                    || description.contains("package.json")
                    || description.contains("api")
                    || description.contains("server")
                    || description.contains("database")
                    || description.contains("backend")
            }
            "devops" => {
                task_id.contains("deploy")
                    || description.contains("script")
                    || description.contains("readme")
                    || description.contains("documentation")
                    || description.contains("deploy")
                    || description.contains("infrastructure")
                    || description.contains("ci/cd")
                    || description.contains("docker")
            }
            "qa" => {
                description.contains("test")
                    || description.contains("quality")
                    || description.contains("bug")
                    || description.contains("validation")
            }
            _ => true, // Masterエージェントは全て処理可能
        }
    }

    /// ステータスを更新
    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_activity = Utc::now();
    }
}

/// シミュレートされたタスク実行結果
#[derive(Debug)]
struct SimulatedResult {
    is_success: bool,
    error_message: String,
    output: String,
    notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Priority, TaskType};
    use crate::identity::default_frontend_role;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_simple_agent_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ClaudeConfig::for_agent("frontend");

        let agent = SimpleClaudeAgent::new(default_frontend_role(), temp_dir.path(), config)
            .await
            .unwrap();

        assert!(agent.identity.agent_id.contains("frontend"));
        assert_eq!(agent.status, AgentStatus::Initializing);
    }

    #[tokio::test]
    async fn test_task_handling() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_manager = SimpleWorkspaceManager::new(temp_dir.path().to_path_buf());
        workspace_manager.init_if_needed().await.unwrap();

        let config = ClaudeConfig::for_agent("frontend");
        let mut agent = SimpleClaudeAgent::new(default_frontend_role(), temp_dir.path(), config)
            .await
            .unwrap();

        agent.initialize(&workspace_manager).await.unwrap();

        let task = Task::new(
            "test-1".to_string(),
            "Create React component".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        let result = agent.execute_task(task).await.unwrap();
        assert!(result.success);
        assert_eq!(agent.task_history.len(), 1);
    }
}
