use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, warn};

/// Git worktree information (shell command version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellWorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub head_commit: String,
    pub is_locked: bool,
    pub is_bare: bool,
}

/// Shell command-based Git worktree management
#[derive(Debug)]
pub struct ShellWorktreeManager {
    repo_path: PathBuf,
}

impl ShellWorktreeManager {
    /// Create a new worktree manager
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        Ok(Self { repo_path })
    }

    /// Check if git is available on the system
    pub fn is_git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Initialize repository (if needed)
    pub async fn init_if_needed(path: &Path) -> Result<()> {
        if !path.join(".git").exists() {
            info!("Initializing new Git repository at: {}", path.display());

            let output = Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
                .await
                .context("Failed to execute git init")?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to initialize git repository: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Create initial commit
            let readme_content =
                "# ccswarm Project\n\nThis repository is managed by ccswarm multi-agent system.\n";
            tokio::fs::write(path.join("README.md"), readme_content).await?;

            // git add . && git commit
            let add_output = Command::new("git")
                .args(["add", "."])
                .current_dir(path)
                .output()
                .await?;

            if add_output.status.success() {
                let commit_output = Command::new("git")
                    .args(["commit", "-m", "Initial commit by ccswarm"])
                    .current_dir(path)
                    .output()
                    .await?;

                if !commit_output.status.success() {
                    warn!("Failed to create initial commit, but continuing...");
                }
            }

            info!("Git repository initialized successfully");
        }

        Ok(())
    }

    /// Get list of worktrees
    pub async fn list_worktrees(&self) -> Result<Vec<ShellWorktreeInfo>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to execute git worktree list")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to list worktrees: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_worktree_list(&output_str).await
    }

    /// Clean up old worktrees
    pub async fn prune_worktrees(&self) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to execute git worktree prune")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to prune worktrees: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Pruned stale worktrees");
        Ok(())
    }

    /// Instance version of init_if_needed
    pub async fn init_repo_if_needed(&self) -> Result<()> {
        Self::init_if_needed(&self.repo_path).await
    }

    /// Simple version of create_worktree (with default parameters)
    pub async fn create_worktree(
        &self,
        worktree_path: &Path,
        branch_name: &str,
    ) -> Result<ShellWorktreeInfo> {
        self.create_worktree_full(worktree_path, branch_name, true)
            .await
    }

    /// Full parameter version of create_worktree
    pub async fn create_worktree_full(
        &self,
        worktree_path: &Path,
        branch_name: &str,
        create_new_branch: bool,
    ) -> Result<ShellWorktreeInfo> {
        // Check if branch exists
        let branch_exists = self.branch_exists(branch_name).await?;

        let mut args = vec!["worktree", "add"];

        if create_new_branch || !branch_exists {
            args.extend(["-b", branch_name]);
        }

        args.push(worktree_path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Invalid UTF-8 in worktree path: {:?}", worktree_path)
        })?);

        if !create_new_branch && branch_exists {
            args.push(branch_name);
        }

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to execute git worktree add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Get worktree information
        let head_commit = self.get_head_commit(worktree_path).await?;
        let is_locked = self.is_worktree_locked(worktree_path).await?;

        let info = ShellWorktreeInfo {
            path: worktree_path.to_path_buf(),
            branch: branch_name.to_string(),
            head_commit,
            is_locked,
            is_bare: false,
        };

        info!(
            "Created worktree: {} on branch {}",
            worktree_path.display(),
            branch_name
        );
        Ok(info)
    }

    /// Simple version of remove_worktree
    pub async fn remove_worktree(&self, worktree_path: &Path) -> Result<()> {
        self.remove_worktree_full(worktree_path, false).await
    }

    /// Full parameter version of remove_worktree
    pub async fn remove_worktree_full(&self, worktree_path: &Path, force: bool) -> Result<()> {
        let mut args = vec!["worktree", "remove"];

        if force {
            args.push("--force");
        }

        args.push(worktree_path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Invalid UTF-8 in worktree path: {:?}", worktree_path)
        })?);

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to execute git worktree remove")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to remove worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Removed worktree: {}", worktree_path.display());
        Ok(())
    }

    /// Commit changes in worktree
    pub async fn commit_worktree_changes(&self, worktree_path: &Path, message: &str) -> Result<()> {
        // First check if there are any changes
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(worktree_path)
            .output()
            .await
            .context("Failed to check git status")?;

        if !status_output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to check git status: {}",
                String::from_utf8_lossy(&status_output.stderr)
            ));
        }

        // Do nothing if there are no changes
        if status_output.stdout.is_empty() {
            info!(
                "No changes to commit in worktree: {}",
                worktree_path.display()
            );
            return Ok(());
        }

        // Stage changes
        let add_output = Command::new("git")
            .args(["add", "."])
            .current_dir(worktree_path)
            .output()
            .await
            .context("Failed to stage changes")?;

        if !add_output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to stage changes: {}",
                String::from_utf8_lossy(&add_output.stderr)
            ));
        }

        // Commit
        let commit_output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(worktree_path)
            .output()
            .await
            .context("Failed to commit changes")?;

        if !commit_output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to commit changes: {}",
                String::from_utf8_lossy(&commit_output.stderr)
            ));
        }

        info!("Committed changes in worktree: {}", worktree_path.display());
        Ok(())
    }

    /// Check if branch exists
    async fn branch_exists(&self, branch_name: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["branch", "--list", branch_name])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        Ok(output.status.success() && !output.stdout.is_empty())
    }

    /// Get HEAD commit
    async fn get_head_commit(&self, worktree_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Check if worktree is locked
    async fn is_worktree_locked(&self, _worktree_path: &Path) -> Result<bool> {
        // Simple version always returns false
        // Full implementation would check for .git/worktrees/<name>/locked file
        Ok(false)
    }

    /// Parse worktree list output
    async fn parse_worktree_list(&self, output: &str) -> Result<Vec<ShellWorktreeInfo>> {
        let mut worktrees = Vec::new();
        let mut current_worktree: Option<ShellWorktreeInfo> = None;

        for line in output.lines() {
            if line.starts_with("worktree ") {
                // Save previous worktree
                if let Some(wt) = current_worktree.take() {
                    worktrees.push(wt);
                }

                // Start new worktree
                let path_str = line
                    .strip_prefix("worktree ")
                    .ok_or_else(|| anyhow::anyhow!("Invalid worktree line format: {}", line))?;
                let path = PathBuf::from(path_str);
                current_worktree = Some(ShellWorktreeInfo {
                    path,
                    branch: String::new(),
                    head_commit: String::new(),
                    is_locked: false,
                    is_bare: false,
                });
            } else if let Some(ref mut wt) = current_worktree {
                if line.starts_with("HEAD ") {
                    wt.head_commit = line.strip_prefix("HEAD ").unwrap_or("unknown").to_string();
                } else if line.starts_with("branch ") {
                    wt.branch = line
                        .strip_prefix("branch refs/heads/")
                        .or_else(|| line.strip_prefix("branch "))
                        .unwrap_or("unknown")
                        .to_string();
                } else if line == "bare" {
                    wt.is_bare = true;
                } else if line == "locked" {
                    wt.is_locked = true;
                }
            }
        }

        // Add last worktree
        if let Some(wt) = current_worktree {
            worktrees.push(wt);
        }

        Ok(worktrees)
    }
}
