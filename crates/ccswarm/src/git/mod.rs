pub mod shell;

pub use shell::{ShellWorktreeInfo as WorktreeInfo, ShellWorktreeManager as WorktreeManager};

/// Git utilities without libgit2 dependency
pub struct GitUtils;

impl GitUtils {
    /// Check if directory is a git repository
    pub async fn is_git_repo(path: &std::path::Path) -> bool {
        path.join(".git").exists()
    }

    /// Get current branch name
    pub async fn get_current_branch(repo_path: &std::path::Path) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(repo_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow::anyhow!("Failed to get current branch"))
        }
    }

    /// Get HEAD commit hash
    pub async fn get_head_commit(repo_path: &std::path::Path) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow::anyhow!("Failed to get HEAD commit"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_git_utils() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // 初期状態ではGitリポジトリではない
        assert!(!GitUtils::is_git_repo(repo_path).await);

        // Gitリポジトリを初期化
        shell::ShellWorktreeManager::init_if_needed(repo_path)
            .await
            .unwrap();

        // 初期化後はGitリポジトリ
        assert!(GitUtils::is_git_repo(repo_path).await);
    }
}
