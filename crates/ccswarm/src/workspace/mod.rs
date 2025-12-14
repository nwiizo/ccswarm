use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

/// ワークスペース情報（Git不使用版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: PathBuf,
    pub agent_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

/// シンプルワークスペース管理（Git不使用）
#[derive(Debug)]
pub struct SimpleWorkspaceManager {
    base_path: PathBuf,
}

impl SimpleWorkspaceManager {
    /// 新しいワークスペース管理を作成
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// ベースディレクトリを初期化
    pub async fn init_if_needed(&self) -> Result<()> {
        if !self.base_path.exists() {
            info!("Creating workspace directory: {}", self.base_path.display());
            fs::create_dir_all(&self.base_path)
                .await
                .context("Failed to create workspace directory")?;
        }

        // agents ディレクトリを作成
        let agents_dir = self.base_path.join("agents");
        if !agents_dir.exists() {
            fs::create_dir_all(&agents_dir)
                .await
                .context("Failed to create agents directory")?;
        }

        Ok(())
    }

    /// エージェント用ワークスペースを作成
    pub async fn create_workspace(&self, agent_id: &str) -> Result<WorkspaceInfo> {
        let workspace_path = self.base_path.join("agents").join(agent_id);

        if workspace_path.exists() {
            warn!("Workspace already exists: {}", workspace_path.display());
        } else {
            fs::create_dir_all(&workspace_path)
                .await
                .context("Failed to create agent workspace")?;

            info!("Created workspace: {}", workspace_path.display());
        }

        let workspace_info = WorkspaceInfo {
            path: workspace_path,
            agent_id: agent_id.to_string(),
            created_at: chrono::Utc::now(),
            is_active: true,
        };

        // ワークスペース情報を保存
        self.save_workspace_info(&workspace_info).await?;

        Ok(workspace_info)
    }

    /// ワークスペース一覧を取得
    pub async fn list_workspaces(&self) -> Result<Vec<WorkspaceInfo>> {
        let agents_dir = self.base_path.join("agents");
        if !agents_dir.exists() {
            return Ok(Vec::new());
        }

        let mut workspaces = Vec::new();
        let mut entries = fs::read_dir(&agents_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.metadata().await?.is_dir() {
                let agent_id = entry.file_name().to_string_lossy().to_string();
                if let Ok(info) = self.load_workspace_info(&agent_id).await {
                    workspaces.push(info);
                } else {
                    // 情報ファイルがない場合は新規作成
                    let workspace_info = WorkspaceInfo {
                        path: entry.path(),
                        agent_id: agent_id.clone(),
                        created_at: chrono::Utc::now(),
                        is_active: true,
                    };
                    workspaces.push(workspace_info);
                }
            }
        }

        Ok(workspaces)
    }

    /// ワークスペースを削除
    pub async fn remove_workspace(&self, agent_id: &str) -> Result<()> {
        let workspace_path = self.base_path.join("agents").join(agent_id);

        if workspace_path.exists() {
            fs::remove_dir_all(&workspace_path)
                .await
                .context("Failed to remove workspace")?;
            info!("Removed workspace: {}", workspace_path.display());
        }

        Ok(())
    }

    /// ワークスペース情報を保存
    async fn save_workspace_info(&self, info: &WorkspaceInfo) -> Result<()> {
        let info_file = info.path.join(".workspace_info.json");
        let content = serde_json::to_string_pretty(info)?;
        fs::write(&info_file, content)
            .await
            .context("Failed to save workspace info")?;
        Ok(())
    }

    /// ワークスペース情報を読み込み
    async fn load_workspace_info(&self, agent_id: &str) -> Result<WorkspaceInfo> {
        let workspace_path = self.base_path.join("agents").join(agent_id);
        let info_file = workspace_path.join(".workspace_info.json");

        let content = fs::read_to_string(&info_file)
            .await
            .context("Failed to read workspace info")?;
        let info: WorkspaceInfo =
            serde_json::from_str(&content).context("Failed to parse workspace info")?;

        Ok(info)
    }

    /// CLAUDEの設定ファイルを配置
    pub async fn setup_claude_config(&self, agent_id: &str, claude_md_content: &str) -> Result<()> {
        let workspace_path = self.base_path.join("agents").join(agent_id);
        let claude_md_path = workspace_path.join("CLAUDE.md");

        fs::write(&claude_md_path, claude_md_content)
            .await
            .context("Failed to write CLAUDE.md")?;

        info!("CLAUDE.md created for agent: {}", agent_id);
        Ok(())
    }
}
