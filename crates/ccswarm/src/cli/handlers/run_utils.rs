//! Shared helpers for commands that operate on past runs under `.ccswarm/runs/<id>/`.
//!
//! Consolidates `resolve_run_path` (previously duplicated across introspect.rs and
//! replay.rs) and adds a single chokepoint for validating user-supplied run IDs against
//! path traversal.

use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};

use crate::run_id::validate_run_id;

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
