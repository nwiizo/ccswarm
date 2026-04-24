//! Shared persistent types for `.ccswarm/queue.yaml`.
//!
//! Single source of truth used by both `queue` (interactive) and `auto` (autonomous)
//! handlers. Previously duplicated between handlers/queue.rs and handlers/auto.rs —
//! keeping the definitions in one place prevents silent field-drift between the two
//! readers.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) const QUEUE_FILE: &str = ".ccswarm/queue.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueTask {
    pub(crate) id: String,
    pub(crate) task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) flow: Option<String>,
    #[serde(default = "default_state")]
    pub(crate) state: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
}

pub(crate) fn default_state() -> String {
    "pending".to_string()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct QueueFile {
    #[serde(default)]
    pub(crate) tasks: Vec<QueueTask>,
}

pub(crate) async fn load_queue(path: &Path) -> Result<QueueFile> {
    if !path.exists() {
        return Ok(QueueFile::default());
    }
    let raw = tokio::fs::read_to_string(path).await?;
    if raw.trim().is_empty() {
        return Ok(QueueFile::default());
    }
    serde_yml::from_str(&raw).context("Failed to parse queue file")
}

/// Save the queue atomically. Codex #2 fix: previous implementation did a plain
/// `tokio::fs::write` which is not atomic — a crash or concurrent writer could leave a
/// truncated / zero-byte `queue.yaml` and deadlock all future queue operations.
///
/// We now write to a same-directory temp file and `rename` into place. On POSIX the
/// rename is atomic, so concurrent readers either see the old file or the new one, never
/// a half-written state. This still doesn't protect against *lost updates* when
/// `queue drain` and `auto` run concurrently — both read → mutate → write — but at least
/// the file never becomes unparseable.
pub(crate) async fn save_queue(path: &Path, queue: &QueueFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let body = serde_yml::to_string(queue)?;

    // Same-directory temp file so `rename` stays within one filesystem.
    let tmp_path = match path.parent() {
        Some(parent) => parent.join(format!(".queue.yaml.tmp.{}", std::process::id())),
        None => path.with_extension("yaml.tmp"),
    };
    tokio::fs::write(&tmp_path, body).await?;
    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}
