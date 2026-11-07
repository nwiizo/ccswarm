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
        Some(id) => resolve_run_id(&runs_dir, id),
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

/// Resolve a full run ID, or a unique prefix such as the 8-character ID that
/// pipeline output prints, to its directory under `runs_dir`.
///
/// The ID is validated by [`validate_run_id`] before it touches the
/// filesystem. A prefix that matches several runs fails and lists them.
pub(crate) fn resolve_run_id(runs_dir: &Path, id: &str) -> Result<PathBuf> {
    validate_run_id(id).context("invalid run ID")?;
    let exact = runs_dir.join(id);
    if exact.is_dir() {
        return Ok(exact);
    }

    let mut matches: Vec<String> = std::fs::read_dir(runs_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(id))
        .collect();
    matches.sort();

    match matches.as_slice() {
        [] => Err(anyhow!(
            "run '{}' not found under {}",
            id,
            runs_dir.display()
        )),
        [only] => Ok(runs_dir.join(only)),
        many => Err(anyhow!(
            "run ID prefix '{}' matches {} runs; use a longer prefix:\n  {}",
            id,
            many.len(),
            many.join("\n  ")
        )),
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

    fn runs_dir_with(ids: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for id in ids {
            std::fs::create_dir(dir.path().join(id)).expect("run dir");
        }
        dir
    }

    #[test]
    fn short_run_id_resolves_to_its_run() {
        let dir = runs_dir_with(&[
            "e87a38b7-8791-4cd1-af9e-ff731f42c505",
            "941451c1-3aa1-4eee-9e3a-9cb2bca30537",
        ]);
        let path = resolve_run_id(dir.path(), "e87a38b7").expect("unique prefix");
        assert_eq!(
            path,
            dir.path().join("e87a38b7-8791-4cd1-af9e-ff731f42c505")
        );
    }

    #[test]
    fn full_run_id_wins_over_longer_names_it_prefixes() {
        let dir = runs_dir_with(&["abc", "abcdef"]);
        assert_eq!(
            resolve_run_id(dir.path(), "abc").expect("exact"),
            dir.path().join("abc")
        );
    }

    #[test]
    fn ambiguous_prefix_lists_the_candidates() {
        let dir = runs_dir_with(&["e87a38b7-aaaa", "e87a38b7-bbbb", "11111111-cccc"]);
        let err = resolve_run_id(dir.path(), "e87a").expect_err("ambiguous");
        let message = err.to_string();
        assert!(message.contains("matches 2 runs"), "{message}");
        assert!(message.contains("e87a38b7-aaaa"), "{message}");
        assert!(message.contains("e87a38b7-bbbb"), "{message}");
        assert!(!message.contains("11111111-cccc"), "{message}");
    }

    #[test]
    fn unknown_or_unsafe_ids_are_rejected() {
        let dir = runs_dir_with(&["e87a38b7-aaaa"]);
        assert!(resolve_run_id(dir.path(), "ffff").is_err());
        assert!(resolve_run_id(dir.path(), "../e87a").is_err());
        assert!(resolve_run_id(&dir.path().join("missing"), "e87a").is_err());
    }
}
