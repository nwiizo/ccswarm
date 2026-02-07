use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

/// Workspace information (non-Git version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: PathBuf,
    pub agent_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

/// Simple workspace manager (without Git)
#[derive(Debug)]
pub struct SimpleWorkspaceManager {
    base_path: PathBuf,
}

impl SimpleWorkspaceManager {
    /// Create a new workspace manager
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Initialize the base directory
    pub async fn init_if_needed(&self) -> Result<()> {
        if !self.base_path.exists() {
            info!("Creating workspace directory: {}", self.base_path.display());
            fs::create_dir_all(&self.base_path)
                .await
                .context("Failed to create workspace directory")?;
        }

        // Create agents directory
        let agents_dir = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"));
        if !agents_dir.exists() {
            fs::create_dir_all(&agents_dir)
                .await
                .context("Failed to create agents directory")?;
        }

        Ok(())
    }

    /// Create a workspace for an agent
    pub async fn create_workspace(&self, agent_id: &str) -> Result<WorkspaceInfo> {
        let workspace_path = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"))
            .join(agent_id);

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

        // Save workspace info
        self.save_workspace_info(&workspace_info).await?;

        Ok(workspace_info)
    }

    /// List all workspaces
    pub async fn list_workspaces(&self) -> Result<Vec<WorkspaceInfo>> {
        let agents_dir = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"));
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
                    // Create new info if file doesn't exist
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

    /// Remove a workspace
    pub async fn remove_workspace(&self, agent_id: &str) -> Result<()> {
        let workspace_path = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"))
            .join(agent_id);

        if workspace_path.exists() {
            fs::remove_dir_all(&workspace_path)
                .await
                .context("Failed to remove workspace")?;
            info!("Removed workspace: {}", workspace_path.display());
        }

        Ok(())
    }

    /// Save workspace information
    async fn save_workspace_info(&self, info: &WorkspaceInfo) -> Result<()> {
        let info_file = info.path.join(".workspace_info.json");
        let content = serde_json::to_string_pretty(info)?;
        fs::write(&info_file, content)
            .await
            .context("Failed to save workspace info")?;
        Ok(())
    }

    /// Load workspace information
    async fn load_workspace_info(&self, agent_id: &str) -> Result<WorkspaceInfo> {
        let workspace_path = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"))
            .join(agent_id);
        let info_file = workspace_path.join(".workspace_info.json");

        let content = fs::read_to_string(&info_file)
            .await
            .context("Failed to read workspace info")?;
        let info: WorkspaceInfo =
            serde_json::from_str(&content).context("Failed to parse workspace info")?;

        Ok(info)
    }

    /// Set up Claude configuration files
    pub async fn setup_claude_config(&self, agent_id: &str, claude_md_content: &str) -> Result<()> {
        let workspace_path = self
            .base_path
            .parent()
            .map(|p| p.join("worktrees"))
            .unwrap_or_else(|| self.base_path.join(".worktrees"))
            .join(agent_id);
        let claude_md_path = workspace_path.join("CLAUDE.md");

        fs::write(&claude_md_path, claude_md_content)
            .await
            .context("Failed to write CLAUDE.md")?;

        info!("CLAUDE.md created for agent: {}", agent_id);
        Ok(())
    }
}
