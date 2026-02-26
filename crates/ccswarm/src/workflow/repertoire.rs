//! Repertoire: External workflow piece package management
//!
//! Manages installation and loading of external piece packages from Git repositories.
//! Packages are stored in `~/.ccswarm/repertoire/<name>/`.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::piece::Piece;

/// Metadata about an installed repertoire package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepertoirePackage {
    /// Package name (derived from repo name)
    pub name: String,
    /// Source URL
    pub source_url: String,
    /// Local installation path
    pub install_path: PathBuf,
    /// Installed pieces
    pub pieces: Vec<String>,
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

        // Discover pieces in the cloned repo
        let pieces = self.discover_pieces(&install_path).await?;

        let package = RepertoirePackage {
            name,
            source_url: url.to_string(),
            install_path,
            pieces,
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
                    let pieces = self.discover_pieces(&path).await.unwrap_or_default();
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    packages.push(RepertoirePackage {
                        name,
                        source_url: String::new(),
                        install_path: path,
                        pieces,
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

    /// Load all pieces from installed repertoire packages
    pub async fn load_all_pieces(&self) -> Result<Vec<Piece>> {
        let mut all_pieces = Vec::new();
        let packages = self.list().await?;

        for package in &packages {
            match self.load_pieces_from_package(package).await {
                Ok(pieces) => all_pieces.extend(pieces),
                Err(e) => {
                    warn!(
                        "Failed to load pieces from package '{}': {}",
                        package.name, e
                    );
                }
            }
        }

        Ok(all_pieces)
    }

    /// Discover piece YAML files in a directory
    async fn discover_pieces(&self, dir: &Path) -> Result<Vec<String>> {
        let mut pieces = Vec::new();

        // Look for YAML files in the root and pieces/ subdirectory
        for search_dir in &[dir.to_path_buf(), dir.join("pieces")] {
            if !search_dir.exists() {
                continue;
            }

            let mut entries = tokio::fs::read_dir(search_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && (ext == "yaml" || ext == "yml")
                    && let Some(name) = path.file_stem()
                {
                    pieces.push(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(pieces)
    }

    /// Load pieces from a specific package
    async fn load_pieces_from_package(&self, package: &RepertoirePackage) -> Result<Vec<Piece>> {
        let mut pieces = Vec::new();

        for search_dir in &[
            package.install_path.clone(),
            package.install_path.join("pieces"),
        ] {
            if !search_dir.exists() {
                continue;
            }

            let mut entries = tokio::fs::read_dir(search_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && (ext == "yaml" || ext == "yml")
                {
                    let content = tokio::fs::read_to_string(&path).await?;
                    match serde_yaml::from_str::<Piece>(&content) {
                        Ok(piece) => pieces.push(piece),
                        Err(e) => {
                            warn!("Failed to parse piece at {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(pieces)
    }

    /// Save package metadata
    async fn save_metadata(&self, package: &RepertoirePackage) -> Result<()> {
        let metadata_path = package.install_path.join(".repertoire.json");
        let content = serde_json::to_string_pretty(package)?;
        tokio::fs::write(&metadata_path, content).await?;
        Ok(())
    }
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
            extract_package_name("https://github.com/user/my-pieces").unwrap(),
            "my-pieces"
        );
        assert_eq!(
            extract_package_name("https://github.com/user/my-pieces.git").unwrap(),
            "my-pieces"
        );
        assert_eq!(
            extract_package_name("git@github.com:user/workflow-pack.git").unwrap(),
            "workflow-pack"
        );
    }
}
