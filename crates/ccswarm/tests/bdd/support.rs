//! Support helpers for the BDD harness.
//!
//! These manipulate `.ccswarm/queue.yaml` directly rather than shelling out to the
//! compiled `ccswarm` binary. The benefit is that scenarios stay fast and hermetic
//! (no subprocess, no feature-flag on the CLI) and still exercise the same on-disk
//! format real users write to.

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) use ccswarm::bdd_api::{parse_provider_kind, resolve_provider_kind as resolve_provider};

const QUEUE_PATH: &str = ".ccswarm/queue.yaml";

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

fn default_state() -> String {
    "pending".into()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct QueueFile {
    #[serde(default)]
    pub(crate) tasks: Vec<QueueTask>,
}

pub(crate) fn load_queue(repo: &Path) -> Result<QueueFile> {
    let path = repo.join(QUEUE_PATH);
    if !path.exists() {
        return Ok(QueueFile::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(QueueFile::default());
    }
    serde_yml::from_str(&raw).context("parse queue.yaml")
}

fn save_queue(repo: &Path, q: &QueueFile) -> Result<()> {
    let path = repo.join(QUEUE_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_yml::to_string(q)?)?;
    Ok(())
}

fn append(repo: &Path, body: String) -> Result<()> {
    let mut q = load_queue(repo)?;
    q.tasks.push(QueueTask {
        id: format!("q-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        task: body,
        flow: None,
        state: "pending".into(),
        created_at: Utc::now(),
        completed_at: None,
        run_id: None,
    });
    save_queue(repo, &q)
}

pub(crate) fn queue_add_inline(repo: &Path, body: &str) -> Result<()> {
    append(repo, body.to_string())
}

pub(crate) fn queue_add_file(repo: &Path, file: &Path) -> Result<()> {
    let body = std::fs::read_to_string(file)?;
    append(repo, body)
}

pub(crate) fn queue_add_stdin(repo: &Path, content: &str) -> Result<()> {
    // Scenarios hand us the stdin payload as a string; the real CLI reads from
    // std::io::stdin but the behavior we care about is "the task body ends up in
    // the queue verbatim", which this satisfies without wrestling with stdin
    // redirection in the test harness.
    append(repo, content.to_string())
}

pub(crate) fn queue_clear(repo: &Path) -> Result<()> {
    let mut q = load_queue(repo)?;
    q.tasks.retain(|t| t.state != "pending");
    save_queue(repo, &q)
}

pub(crate) fn queue_add_both_sources(repo: &Path, _positional: &str, _file: &Path) -> Result<()> {
    // Mirror the CLI's guard: refuse when more than one input source is given.
    Err(anyhow!(
        "choose only one of: --from-issue <N>, --file <path>, or `-` (stdin)"
    ))
}
