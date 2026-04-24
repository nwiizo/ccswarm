//! Repertoire: External workflow flow package management
//!
//! Manages installation and loading of external flow packages from Git repositories.
//! Packages are stored in `~/.ccswarm/repertoire/<name>/`.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::flow::Flow;

/// Metadata about an installed repertoire package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepertoirePackage {
    /// Package name (derived from repo name)
    pub name: String,
    /// Source URL
    pub source_url: String,
    /// Local installation path
    pub install_path: PathBuf,
    /// Installed flows
    pub flows: Vec<String>,
}

/// Manages repertoire packages
pub struct RepertoireManager {
    /// Base directory for repertoire packages (~/.ccswarm/repertoire/)
    base_dir: PathBuf,
}

impl RepertoireManager {
    /// Create a new repertoire manager
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?
            .join(".ccswarm")
            .join("repertoire");

        Ok(Self { base_dir })
    }

    /// Create with a custom base directory
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Add a repertoire package from a Git URL
    pub async fn add(&self, url: &str) -> Result<RepertoirePackage> {
        // Extract package name from URL
        let name = extract_package_name(url)?;

        let install_path = self.base_dir.join(&name);

        // Check if already installed
        if install_path.exists() {
            return Err(anyhow!(
                "Package '{}' is already installed at {}. Use 'repertoire remove {}' first.",
                name,
                install_path.display(),
                name
            ));
        }

        // Create base directory
        tokio::fs::create_dir_all(&self.base_dir)
            .await
            .context("Failed to create repertoire directory")?;

        // Clone the repository
        info!("Cloning {} into {}", url, install_path.display());
        let output = tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                url,
                &install_path.to_string_lossy(),
            ])
            .output()
            .await
            .context("Failed to run git clone")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git clone failed: {}", stderr));
        }

        // Discover flows in the cloned repo
        let flows = self.discover_pieces(&install_path).await?;

        let package = RepertoirePackage {
            name,
            source_url: url.to_string(),
            install_path,
            flows,
        };

        // Save package metadata
        self.save_metadata(&package).await?;

        Ok(package)
    }

    /// List all installed repertoire packages
    pub async fn list(&self) -> Result<Vec<RepertoirePackage>> {
        let mut packages = Vec::new();

        if !self.base_dir.exists() {
            return Ok(packages);
        }

        let mut entries = tokio::fs::read_dir(&self.base_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let metadata_path = path.join(".repertoire.json");
                if metadata_path.exists() {
                    let content = tokio::fs::read_to_string(&metadata_path).await?;
                    if let Ok(package) = serde_json::from_str::<RepertoirePackage>(&content) {
                        packages.push(package);
                    }
                } else {
                    // Try to reconstruct metadata from directory
                    let flows = self.discover_pieces(&path).await.unwrap_or_default();
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    packages.push(RepertoirePackage {
                        name,
                        source_url: String::new(),
                        install_path: path,
                        flows,
                    });
                }
            }
        }

        Ok(packages)
    }

    /// Remove a repertoire package
    pub async fn remove(&self, name: &str) -> Result<()> {
        let install_path = self.base_dir.join(name);

        if !install_path.exists() {
            return Err(anyhow!("Package '{}' is not installed", name));
        }

        tokio::fs::remove_dir_all(&install_path)
            .await
            .with_context(|| format!("Failed to remove package at {}", install_path.display()))?;

        info!("Removed repertoire package '{}'", name);
        Ok(())
    }

    /// Load all flows from installed repertoire packages
    pub async fn load_all_pieces(&self) -> Result<Vec<Flow>> {
        let mut all_pieces = Vec::new();
        let packages = self.list().await?;

        for package in &packages {
            match self.load_pieces_from_package(package).await {
                Ok(flows) => all_pieces.extend(flows),
                Err(e) => {
                    warn!(
                        "Failed to load flows from package '{}': {}",
                        package.name, e
                    );
                }
            }
        }

        Ok(all_pieces)
    }

    /// Discover flow YAML files in a directory (returns the file stems).
    async fn discover_pieces(&self, dir: &Path) -> Result<Vec<String>> {
        let mut flows = Vec::new();
        for_each_flow_yaml(dir, |path| {
            if let Some(name) = path.file_stem() {
                flows.push(name.to_string_lossy().to_string());
            }
            Ok(())
        })
        .await?;
        Ok(flows)
    }

    /// Load flows from a specific package.
    async fn load_pieces_from_package(&self, package: &RepertoirePackage) -> Result<Vec<Flow>> {
        let mut flows = Vec::new();
        for_each_flow_yaml(&package.install_path, |path| {
            // Best-effort: unparseable files are logged and skipped, not fatal.
            match std::fs::read_to_string(path) {
                Ok(content) => match serde_yml::from_str::<Flow>(&content) {
                    Ok(flow) => flows.push(flow),
                    Err(e) => warn!("Failed to parse flow at {}: {}", path.display(), e),
                },
                Err(e) => warn!("Failed to read flow at {}: {}", path.display(), e),
            }
            Ok(())
        })
        .await?;
        Ok(flows)
    }

    /// Save package metadata
    async fn save_metadata(&self, package: &RepertoirePackage) -> Result<()> {
        let metadata_path = package.install_path.join(".repertoire.json");
        let content = serde_json::to_string_pretty(package)?;
        tokio::fs::write(&metadata_path, content).await?;
        Ok(())
    }
}

/// Walk `dir` and `dir/flows/` (if either exists), calling `visit` with the path of
/// every `*.yaml` / `*.yml` file found. Used by both `discover_pieces` (needs just
/// names) and `load_pieces_from_package` (needs to parse each file) so the directory
/// traversal lives in exactly one place.
async fn for_each_flow_yaml(dir: &Path, mut visit: impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for search_dir in [dir.to_path_buf(), dir.join("flows")] {
        if !search_dir.exists() {
            continue;
        }
        let mut entries = tokio::fs::read_dir(&search_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                visit(&path)?;
            }
        }
    }
    Ok(())
}

/// Extract package name from a Git URL
fn extract_package_name(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    url.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Could not extract package name from URL: {}", url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        assert_eq!(
            extract_package_name("https://github.com/user/my-flows").unwrap(),
            "my-flows"
        );
        assert_eq!(
            extract_package_name("https://github.com/user/my-flows.git").unwrap(),
            "my-flows"
        );
        assert_eq!(
            extract_package_name("git@github.com:user/workflow-pack.git").unwrap(),
            "workflow-pack"
        );
    }
}
