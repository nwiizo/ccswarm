/// Git worktree integration with persistent Claude Code sessions
///
/// This module bridges the gap between git worktree management and persistent
/// Claude Code sessions, providing efficient workspace isolation while maintaining
/// session continuity for maximum token efficiency.
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::agent::persistent::PersistentClaudeAgent;
use crate::agent::{Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::git::shell::ShellWorktreeManager;
use crate::identity::{AgentIdentity, AgentRole};
use crate::session::persistent_session::{
    EfficiencyStats, PersistentSessionManager, PersistentSessionManagerConfig,
};

/// Worktree session information combining git and Claude persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSessionInfo {
    pub session_id: String,
    pub agent_id: String,
    pub agent_role: AgentRole,
    pub worktree_path: PathBuf,
    pub branch_name: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub tasks_completed: usize,
    pub status: WorktreeSessionStatus,
    pub git_status: GitWorktreeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorktreeSessionStatus {
    Creating,
    GitSetup,
    IdentityEstablishment,
    Active,
    Idle,
    Cleaning,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GitWorktreeStatus {
    NotCreated,
    Creating,
    Ready,
    Dirty,
    Locked,
    Error(String),
}

/// Configuration for worktree session management
#[derive(Debug, Clone)]
pub struct WorktreeSessionConfig {
    /// Base persistent session configuration
    pub persistent_config: PersistentSessionManagerConfig,

    /// Git repository root path
    pub repo_path: PathBuf,

    /// Branch prefix for agent worktrees
    pub branch_prefix: String,

    /// Whether to auto-commit changes
    pub auto_commit: bool,

    /// Whether to cleanup worktrees on session end
    pub cleanup_worktrees: bool,

    /// Maximum number of worktrees per role
    pub max_worktrees_per_role: usize,
}

impl Default for WorktreeSessionConfig {
    fn default() -> Self {
        Self {
            persistent_config: PersistentSessionManagerConfig::default(),
            repo_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            branch_prefix: "feature".to_string(),
            auto_commit: false,
            cleanup_worktrees: true,
            max_worktrees_per_role: 3,
        }
    }
}

/// Manages persistent Claude Code sessions with git worktree integration
#[derive(Debug)]
pub struct WorktreeSessionManager {
    /// Underlying persistent session manager
    persistent_manager: PersistentSessionManager,

    /// Git worktree manager
    git_manager: ShellWorktreeManager,

    /// Worktree session information
    worktree_sessions: Arc<RwLock<HashMap<String, WorktreeSessionInfo>>>,

    /// Configuration
    config: WorktreeSessionConfig,
}

impl WorktreeSessionManager {
    /// Create a new worktree session manager
    pub fn new(config: WorktreeSessionConfig) -> Result<Self> {
        let persistent_manager = PersistentSessionManager::new(
            config.repo_path.join("agents"),
            config.persistent_config.clone(),
        );

        let git_manager = ShellWorktreeManager::new(config.repo_path.clone())?;

        Ok(Self {
            persistent_manager,
            git_manager,
            worktree_sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Start the worktree session manager
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting worktree session manager");

        // Start the underlying persistent session manager
        self.persistent_manager.start().await?;

        // Initialize git repository if needed
        self.git_manager.init_repo_if_needed().await?;

        tracing::info!("Worktree session manager started successfully");
        Ok(())
    }

    /// Get or create a worktree session for an agent
    pub async fn get_or_create_worktree_session(
        &self,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<Mutex<PersistentClaudeAgent>>> {
        // Check if we can reuse an existing worktree session
        if let Some(existing) = self.find_reusable_worktree_session(&role).await {
            tracing::info!(
                "Reusing existing worktree session for role: {}",
                role.name()
            );
            return Ok(existing);
        }

        // Check limits
        let current_count = self.count_sessions_by_role(&role).await;
        if current_count >= self.config.max_worktrees_per_role {
            return Err(anyhow::anyhow!(
                "Maximum worktrees per role reached for {}: {}",
                role.name(),
                self.config.max_worktrees_per_role
            ));
        }

        // Create new worktree session
        self.create_new_worktree_session(role, claude_config).await
    }

    /// Find a reusable worktree session
    async fn find_reusable_worktree_session(
        &self,
        role: &AgentRole,
    ) -> Option<Arc<Mutex<PersistentClaudeAgent>>> {
        let sessions = self.worktree_sessions.read().await;

        for (_agent_id, info) in sessions.iter() {
            if info.agent_role.name() == role.name()
                && info.status == WorktreeSessionStatus::Idle
                && info.git_status == GitWorktreeStatus::Ready
            {
                // Get the persistent session
                return self
                    .persistent_manager
                    .get_or_create_session(role.clone(), ClaudeConfig::default())
                    .await
                    .ok();
            }
        }

        None
    }

    /// Count sessions by role
    async fn count_sessions_by_role(&self, role: &AgentRole) -> usize {
        let sessions = self.worktree_sessions.read().await;
        sessions
            .values()
            .filter(|info| info.agent_role.name() == role.name())
            .count()
    }

    /// Create a new worktree session
    async fn create_new_worktree_session(
        &self,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<Mutex<PersistentClaudeAgent>>> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());
        let branch_name = format!("{}/{}", self.config.branch_prefix, &agent_id);

        tracing::info!("Creating new worktree session for agent: {}", agent_id);

        // Create worktree session info
        let mut session_info = WorktreeSessionInfo {
            session_id: Uuid::new_v4().to_string(),
            agent_id: agent_id.clone(),
            agent_role: role.clone(),
            worktree_path: self.config.repo_path.join("agents").join(&agent_id),
            branch_name: branch_name.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            tasks_completed: 0,
            status: WorktreeSessionStatus::Creating,
            git_status: GitWorktreeStatus::NotCreated,
        };

        // Store session info early
        {
            let mut sessions = self.worktree_sessions.write().await;
            sessions.insert(agent_id.clone(), session_info.clone());
        }

        // Step 1: Setup git worktree
        session_info.status = WorktreeSessionStatus::GitSetup;
        session_info.git_status = GitWorktreeStatus::Creating;
        self.update_session_info(&agent_id, session_info.clone())
            .await;

        let _worktree_info = self
            .git_manager
            .create_worktree(&session_info.worktree_path, &branch_name)
            .await
            .context("Failed to create git worktree")?;

        session_info.git_status = GitWorktreeStatus::Ready;
        self.update_session_info(&agent_id, session_info.clone())
            .await;

        // Step 2: Create persistent Claude Code session
        session_info.status = WorktreeSessionStatus::IdentityEstablishment;
        self.update_session_info(&agent_id, session_info.clone())
            .await;

        // Create agent identity with worktree path
        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role,
            workspace_path: session_info.worktree_path.clone(),
            env_vars: Self::create_env_vars(&agent_id, &session_info.session_id),
            session_id: session_info.session_id.clone(),
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        // Create persistent agent with worktree workspace
        let agent_role = crate::agent::AgentRole::from_identity_role(&identity.specialization);
        let agent = PersistentClaudeAgent::new(identity.agent_id.clone(), agent_role);
        let agent = Arc::new(Mutex::new(agent));

        // Generate minimal CLAUDE.md in worktree
        self.setup_worktree_environment(&session_info, &agent)
            .await?;

        // Step 3: Establish identity once
        {
            let agent_guard = agent.lock().await;
            // Identity establishment no longer needed with simplified structure
        }

        session_info.status = WorktreeSessionStatus::Active;
        self.update_session_info(&agent_id, session_info).await;

        tracing::info!("Worktree session created successfully: {}", agent_id);
        Ok(agent)
    }

    /// Setup worktree environment
    async fn setup_worktree_environment(
        &self,
        session_info: &WorktreeSessionInfo,
        agent: &Arc<Mutex<PersistentClaudeAgent>>,
    ) -> Result<()> {
        // Create minimal CLAUDE.md in worktree
        let agent_guard = agent.lock().await;
        let compact_prompt = format!(
            r#"# CLAUDE.md - {} Agent Workspace

🤖 **AGENT**: {}
📁 **WORKSPACE**: {}
🎯 **SPECIALIZATION**: {}

## Quick Identity
You are a specialized {} agent working in an isolated git worktree.
Maintain strict role boundaries and provide focused responses.

## Response Format
Always include:
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: [Task assessment]
"#,
            agent_guard.agent.role.name(),
            agent_guard.agent.id,
            session_info
                .worktree_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            agent_guard.agent.role.name(),
            agent_guard.agent.role.name(),
            agent_guard.agent.role.name(),
            session_info
                .worktree_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
        );

        let claude_md_path = session_info.worktree_path.join("CLAUDE.md");
        tokio::fs::write(&claude_md_path, compact_prompt)
            .await
            .context("Failed to write CLAUDE.md")?;

        // Create .claude.json configuration
        let claude_config_path = session_info.worktree_path.join(".claude.json");
        let config_json = serde_json::json!({
            "dangerous_skip": true,
            "json_output": false,
            "think_mode": null
        });
        tokio::fs::write(
            &claude_config_path,
            serde_json::to_string_pretty(&config_json)?,
        )
        .await
        .context("Failed to write .claude.json")?;

        Ok(())
    }

    /// Execute a task in a worktree session
    pub async fn execute_task(
        &self,
        role: AgentRole,
        task: Task,
        claude_config: ClaudeConfig,
    ) -> Result<TaskResult> {
        let session = self
            .get_or_create_worktree_session(role, claude_config)
            .await?;

        // Update session status
        let agent_id = {
            let agent = session.lock().await;
            agent.agent.id.clone()
        };

        self.update_session_status(&agent_id, WorktreeSessionStatus::Active)
            .await;

        // Execute task
        let result = {
            let mut agent = session.lock().await;
            agent.execute_task(task).await?
        };

        // Auto-commit if enabled
        if self.config.auto_commit && result.success {
            if let Err(e) = self.auto_commit_changes(&agent_id).await {
                tracing::warn!("Auto-commit failed for agent {}: {}", agent_id, e);
            }
        }

        // Update session info
        self.update_session_activity(&agent_id).await;
        self.update_session_status(&agent_id, WorktreeSessionStatus::Idle)
            .await;

        Ok(result)
    }

    /// Execute multiple tasks in batch with worktree context
    pub async fn execute_task_batch(
        &self,
        role: AgentRole,
        tasks: Vec<Task>,
        claude_config: ClaudeConfig,
    ) -> Result<Vec<TaskResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        tracing::info!(
            "Executing batch of {} tasks in worktree for role: {}",
            tasks.len(),
            role.name()
        );

        let session = self
            .get_or_create_worktree_session(role, claude_config)
            .await?;

        let agent_id = {
            let agent = session.lock().await;
            agent.agent.id.clone()
        };

        self.update_session_status(&agent_id, WorktreeSessionStatus::Active)
            .await;

        // Execute batch
        let results = {
            let mut agent = session.lock().await;
            agent.execute_task_batch(tasks).await?
        };

        // Auto-commit batch results if enabled
        if self.config.auto_commit {
            if let Err(e) = self.auto_commit_changes(&agent_id).await {
                tracing::warn!("Auto-commit failed for agent {}: {}", agent_id, e);
            }
        }

        self.update_session_activity(&agent_id).await;
        self.update_session_status(&agent_id, WorktreeSessionStatus::Idle)
            .await;

        tracing::info!("Batch execution completed in worktree session");
        Ok(results)
    }

    /// Auto-commit changes in worktree
    async fn auto_commit_changes(&self, agent_id: &str) -> Result<()> {
        let session_info = {
            let sessions = self.worktree_sessions.read().await;
            sessions.get(agent_id).cloned()
        };

        if let Some(info) = session_info {
            let commit_message = format!(
                "Auto-commit from {} agent\n\nSession: {}\nTimestamp: {}",
                info.agent_role.name(),
                info.session_id,
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );

            // Use git manager to commit changes
            self.git_manager
                .commit_worktree_changes(&info.worktree_path, &commit_message)
                .await?;

            tracing::info!("Auto-committed changes for agent: {}", agent_id);
        }

        Ok(())
    }

    /// Update session info
    async fn update_session_info(&self, agent_id: &str, info: WorktreeSessionInfo) {
        let mut sessions = self.worktree_sessions.write().await;
        sessions.insert(agent_id.to_string(), info);
    }

    /// Update session status
    async fn update_session_status(&self, agent_id: &str, status: WorktreeSessionStatus) {
        let mut sessions = self.worktree_sessions.write().await;
        if let Some(info) = sessions.get_mut(agent_id) {
            info.status = status;
            info.last_activity = Utc::now();
        }
    }

    /// Update session activity
    async fn update_session_activity(&self, agent_id: &str) {
        let mut sessions = self.worktree_sessions.write().await;
        if let Some(info) = sessions.get_mut(agent_id) {
            info.last_activity = Utc::now();
            info.tasks_completed += 1;
        }
    }

    /// List all worktree sessions
    pub async fn list_worktree_sessions(&self) -> Vec<WorktreeSessionInfo> {
        let sessions = self.worktree_sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get sessions by role
    pub async fn get_worktree_sessions_by_role(
        &self,
        role: &AgentRole,
    ) -> Vec<WorktreeSessionInfo> {
        let sessions = self.worktree_sessions.read().await;
        sessions
            .values()
            .filter(|info| info.agent_role.name() == role.name())
            .cloned()
            .collect()
    }

    /// Get combined efficiency statistics
    pub async fn get_combined_efficiency_stats(&self) -> CombinedEfficiencyStats {
        let persistent_stats = self.persistent_manager.get_efficiency_stats().await;
        let worktree_sessions = self.worktree_sessions.read().await;

        let active_worktrees = worktree_sessions
            .values()
            .filter(|info| info.status == WorktreeSessionStatus::Active)
            .count();

        let idle_worktrees = worktree_sessions
            .values()
            .filter(|info| info.status == WorktreeSessionStatus::Idle)
            .count();

        CombinedEfficiencyStats {
            persistent_stats: persistent_stats.clone(),
            total_worktrees: worktree_sessions.len(),
            active_worktrees,
            idle_worktrees,
            worktree_reuse_rate: if persistent_stats.total_tasks_completed > worktree_sessions.len()
            {
                (persistent_stats.total_tasks_completed - worktree_sessions.len()) as f64
                    / persistent_stats.total_tasks_completed as f64
            } else {
                0.0
            },
        }
    }

    /// Cleanup worktree session
    pub async fn cleanup_worktree_session(&self, agent_id: &str) -> Result<()> {
        let session_info = {
            let mut sessions = self.worktree_sessions.write().await;
            sessions.remove(agent_id)
        };

        if let Some(info) = session_info {
            if self.config.cleanup_worktrees {
                // Remove git worktree
                if let Err(e) = self.git_manager.remove_worktree(&info.worktree_path).await {
                    tracing::warn!("Failed to remove worktree for {}: {}", agent_id, e);
                }
            }

            tracing::info!("Cleaned up worktree session: {}", agent_id);
        }

        Ok(())
    }

    /// Shutdown all worktree sessions
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down worktree session manager");

        // Get all session IDs
        let session_ids: Vec<String> = {
            let sessions = self.worktree_sessions.read().await;
            sessions.keys().cloned().collect()
        };

        // Cleanup all sessions
        for agent_id in session_ids {
            if let Err(e) = self.cleanup_worktree_session(&agent_id).await {
                tracing::error!("Failed to cleanup session {}: {}", agent_id, e);
            }
        }

        // Shutdown persistent manager
        self.persistent_manager.shutdown().await?;

        tracing::info!("Worktree session manager shutdown complete");
        Ok(())
    }

    /// Create environment variables
    fn create_env_vars(
        agent_id: &str,
        session_id: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("CCSWARM_SESSION_ID".to_string(), session_id.to_string());
        env_vars.insert("CCSWARM_WORKTREE_SESSION".to_string(), "true".to_string());
        env_vars.insert(
            "CCSWARM_ROLE".to_string(),
            agent_id.split('-').next().unwrap_or("unknown").to_string(),
        );
        env_vars
    }
}

/// Combined efficiency statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CombinedEfficiencyStats {
    pub persistent_stats: EfficiencyStats,
    pub total_worktrees: usize,
    pub active_worktrees: usize,
    pub idle_worktrees: usize,
    pub worktree_reuse_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[tokio::test]
    async fn test_worktree_session_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorktreeSessionConfig {
            repo_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = WorktreeSessionManager::new(config).unwrap();
        assert_eq!(manager.worktree_sessions.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_combined_efficiency_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorktreeSessionConfig {
            repo_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = WorktreeSessionManager::new(config).unwrap();
        let stats = manager.get_combined_efficiency_stats().await;

        assert_eq!(stats.total_worktrees, 0);
        assert_eq!(stats.active_worktrees, 0);
        assert_eq!(stats.idle_worktrees, 0);
    }
}
