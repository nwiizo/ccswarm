use std::path::{Path, PathBuf};
use tokio::fs;
use crate::error::{CCSwarmError, Result};

/// Filesystem utilities
pub struct FsUtils;

impl FsUtils {
    /// Build a path from base and segments
    pub fn build_path(base: &Path, segments: &[&str]) -> PathBuf {
        let mut path = base.to_path_buf();
        for segment in segments {
            path.push(segment);
        }
        path
    }

    /// Ensure a directory exists
    pub async fn ensure_dir_exists(path: &Path, _context: &str) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).await
                .map_err(|e| CCSwarmError::Io(e))?;
        }
        Ok(())
    }

    /// Save JSON data to file
    pub async fn save_json<T: serde::Serialize>(
        data: &T,
        path: &Path,
        _name: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(data)
            .map_err(CCSwarmError::SerdeJson)?;
        Self::write_file(path, &content, _name).await
    }

    /// Load JSON data from file
    pub async fn load_json<T: serde::de::DeserializeOwned>(
        path: &Path,
        _name: &str,
    ) -> Result<T> {
        let content = fs::read_to_string(path).await
            .map_err(CCSwarmError::Io)?;
        serde_json::from_str(&content)
            .map_err(CCSwarmError::SerdeJson)
    }

    /// Write file with context
    pub async fn write_file(path: &Path, content: &str, _name: &str) -> Result<()> {
        fs::write(path, content).await
            .map_err(CCSwarmError::Io)
    }

    /// Remove directory and all contents
    pub async fn remove_dir_all(path: &Path, _name: &str) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path).await
                .map_err(CCSwarmError::Io)?;
        }
        Ok(())
    }
}
