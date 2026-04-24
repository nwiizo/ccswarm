//! Shared helpers for commands that operate on past runs under `.ccswarm/runs/<id>/`.
//!
//! Consolidates `resolve_run_path` (previously duplicated across introspect.rs and
//! replay.rs) and adds a single chokepoint for validating user-supplied run IDs against
//! path traversal.

use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};

/// Validate that a run ID only contains characters allowed in UUIDs / short hex.
///
/// Rejects anything that could escape `.ccswarm/runs/` via `../`, embedded path
/// separators, or null bytes. This is the single chokepoint; every call site that
/// dereferences `.ccswarm/runs/<id>` must go through [`resolve_run_path`].
pub fn validate_run_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(anyhow!("run ID is empty"));
    }
    if id.len() > 128 {
        return Err(anyhow!("run ID is suspiciously long ({} bytes)", id.len()));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "run ID '{}' contains illegal characters (allowed: [A-Za-z0-9_-])",
            id
        ));
    }
    Ok(())
}

/// Resolve a run path. When `run_id` is `None`, returns the most recently created run.
///
/// All user-supplied IDs are validated by [`validate_run_id`] before being joined.
pub(crate) async fn resolve_run_path(repo_path: &Path, run_id: Option<&str>) -> Result<PathBuf> {
    let runs_dir = repo_path.join(".ccswarm").join("runs");
    if !runs_dir.exists() {
        return Err(anyhow!(
            "No runs directory. Run a pipeline first: ccswarm pipeline --task ..."
        ));
    }

    match run_id {
        Some(id) => {
            validate_run_id(id).context("invalid run ID")?;
            let path = runs_dir.join(id);
            if !path.is_dir() {
                return Err(anyhow!(
                    "run '{}' not found under {}",
                    id,
                    runs_dir.display()
                ));
            }
            Ok(path)
        }
        None => {
            let mut entries: Vec<PathBuf> = Vec::new();
            let mut rd = tokio::fs::read_dir(&runs_dir).await?;
            while let Some(entry) = rd.next_entry().await? {
                let p = entry.path();
                if p.is_dir() {
                    entries.push(p);
                }
            }
            entries.sort();
            entries
                .pop()
                .ok_or_else(|| anyhow!("no runs found in {}", runs_dir.display()))
        }
    }
}

/// Read and parse `summary.json` for a given run directory.
pub(crate) fn read_summary(run_path: &Path) -> Result<serde_json::Value> {
    let p = run_path.join("summary.json");
    if !p.exists() {
        return Err(anyhow!(
            "summary.json not found in {} — is this a finished run?",
            run_path.display()
        ));
    }
    let raw = std::fs::read_to_string(&p)?;
    serde_json::from_str(&raw).context("Failed to parse summary.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_run_id("../etc/passwd").is_err());
        assert!(validate_run_id("..").is_err());
        assert!(validate_run_id("a/b").is_err());
        assert!(validate_run_id("a\\b").is_err());
        assert!(validate_run_id("a\0b").is_err());
    }

    #[test]
    fn accepts_uuid_ish() {
        assert!(validate_run_id("97fc1ccd-ea1b-4119-9a3c-2c1a550d71ee").is_ok());
        assert!(validate_run_id("abc123").is_ok());
        assert!(validate_run_id("run_42").is_ok());
    }

    #[test]
    fn rejects_empty_and_long() {
        assert!(validate_run_id("").is_err());
        assert!(validate_run_id(&"a".repeat(200)).is_err());
    }
}
