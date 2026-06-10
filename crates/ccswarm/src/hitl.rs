//! Human-in-the-loop approval gating.
//!
//! `ApprovalStore` is the single owner of `.ccswarm/approvals/*.json`,
//! shared by the `ccswarm approve` CLI (writes decisions) and the
//! pipeline's commit gate (writes pending requests, polls for decisions).
//! Records are keyed by run ID so an approval ties back to
//! `.ccswarm/runs/<id>/`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Which operation an approval guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gate {
    Plan,
    RiskyEdit,
    Deploy,
    Merge,
    /// Commit (and optional PR) of an unattended pipeline run's output.
    Commit,
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Gate::Plan => "plan",
            Gate::RiskyEdit => "risky-edit",
            Gate::Deploy => "deploy",
            Gate::Merge => "merge",
            Gate::Commit => "commit",
        };
        f.write_str(s)
    }
}

/// Lifecycle of an approval record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// One approval record, stored as `.ccswarm/approvals/{id}.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: String,
    pub gate: Gate,
    pub status: ApprovalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Truncated task summary, set by the requesting pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    /// Who created the pending request ("auto" | "drain").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
}

/// Outcome of waiting on a gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOutcome {
    Approved,
    Rejected(Option<String>),
    TimedOut,
}

/// File-backed store for approval records.
pub struct ApprovalStore {
    dir: PathBuf,
}

impl ApprovalStore {
    /// Store rooted at `<repo_root>/.ccswarm/approvals`.
    pub fn new(repo_root: &Path) -> Self {
        Self {
            dir: repo_root.join(".ccswarm").join("approvals"),
        }
    }

    /// Create a pending approval request for `id`.
    pub async fn request(
        &self,
        id: &str,
        gate: Gate,
        task: &str,
        requested_by: &str,
    ) -> Result<ApprovalRecord> {
        crate::run_id::validate_run_id(id).context("invalid approval ID")?;
        let record = ApprovalRecord {
            id: id.to_string(),
            gate,
            status: ApprovalStatus::Pending,
            reason: None,
            task: Some(task.chars().take(200).collect()),
            requested_at: Some(chrono::Utc::now().to_rfc3339()),
            requested_by: Some(requested_by.to_string()),
            decided_at: None,
            decided_by: None,
        };
        self.write(&record).await?;
        Ok(record)
    }

    /// Record a decision for `id`. Merges onto an existing pending record when
    /// present (preserving request context); otherwise upserts a fresh record —
    /// the CLI has always allowed approving an ID with no prior request.
    pub async fn decide(
        &self,
        id: &str,
        gate: Gate,
        approve: bool,
        reason: Option<&str>,
    ) -> Result<ApprovalRecord> {
        crate::run_id::validate_run_id(id).context("invalid approval ID")?;
        let mut record = self.get(id).await?.unwrap_or(ApprovalRecord {
            id: id.to_string(),
            gate,
            status: ApprovalStatus::Pending,
            reason: None,
            task: None,
            requested_at: None,
            requested_by: None,
            decided_at: None,
            decided_by: None,
        });
        record.gate = gate;
        record.status = if approve {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        record.reason = reason.map(str::to_string);
        record.decided_at = Some(chrono::Utc::now().to_rfc3339());
        record.decided_by = Some("cli".to_string());
        self.write(&record).await?;
        Ok(record)
    }

    /// Read the record for `id`, if present and parsable.
    pub async fn get(&self, id: &str) -> Result<Option<ApprovalRecord>> {
        crate::run_id::validate_run_id(id).context("invalid approval ID")?;
        let path = self.dir.join(format!("{}.json", id));
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => Ok(serde_json::from_str(&content).ok()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("failed to read {:?}", path)),
        }
    }

    /// List all parsable records.
    pub async fn list(&self) -> Result<Vec<ApprovalRecord>> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let mut entries = tokio::fs::read_dir(&self.dir).await?;
        let mut records = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(content) = tokio::fs::read_to_string(&path).await
                && let Ok(record) = serde_json::from_str::<ApprovalRecord>(&content)
            {
                records.push(record);
            }
        }
        records.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(records)
    }

    /// Poll until `id` is decided for `expected_gate` or `timeout` elapses.
    /// A missing or corrupt record counts as still-pending (tolerates a
    /// decision write racing the read), as does a decision recorded for a
    /// *different* gate — `ccswarm approve plan --id <run-id>` must not
    /// release a waiting commit gate.
    pub async fn wait_for_decision(
        &self,
        id: &str,
        expected_gate: Gate,
        poll: Duration,
        timeout: Duration,
    ) -> Result<GateOutcome> {
        crate::run_id::validate_run_id(id).context("invalid approval ID")?;
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(record) = self.get(id).await?
                && record.gate == expected_gate
            {
                match record.status {
                    ApprovalStatus::Approved => return Ok(GateOutcome::Approved),
                    ApprovalStatus::Rejected => return Ok(GateOutcome::Rejected(record.reason)),
                    ApprovalStatus::Pending => {}
                }
            }
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Ok(GateOutcome::TimedOut);
            }
            // Cap the sleep at the remaining budget so the effective wait
            // never overshoots `timeout` by a full poll interval.
            tokio::time::sleep(poll.min(deadline - now)).await;
        }
    }

    /// Atomic write: temp file + rename, so a polling reader never sees a
    /// half-written record.
    async fn write(&self, record: &ApprovalRecord) -> Result<()> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let path = self.dir.join(format!("{}.json", record.id));
        let tmp = self.dir.join(format!("{}.json.tmp", record.id));
        let content = serde_json::to_string_pretty(record)?;
        tokio::fs::write(&tmp, content)
            .await
            .with_context(|| format!("failed to write {:?}", tmp))?;
        tokio::fs::rename(&tmp, &path)
            .await
            .with_context(|| format!("failed to rename {:?} -> {:?}", tmp, path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store() -> (TempDir, ApprovalStore) {
        let dir = TempDir::new().expect("tempdir");
        let store = ApprovalStore::new(dir.path());
        (dir, store)
    }

    #[tokio::test]
    async fn request_writes_pending_record() {
        let (_dir, store) = store();
        let record = store
            .request("run-1", Gate::Commit, "add login form", "auto")
            .await
            .expect("request");
        assert_eq!(record.status, ApprovalStatus::Pending);
        assert_eq!(record.gate, Gate::Commit);

        let read = store.get("run-1").await.expect("get").expect("record");
        assert_eq!(read.status, ApprovalStatus::Pending);
        assert_eq!(read.task.as_deref(), Some("add login form"));
        assert!(read.requested_at.is_some());
        assert_eq!(read.requested_by.as_deref(), Some("auto"));
    }

    #[tokio::test]
    async fn decide_flips_pending_and_preserves_request_context() {
        let (_dir, store) = store();
        store
            .request("run-2", Gate::Commit, "task", "drain")
            .await
            .expect("request");
        let decided = store
            .decide("run-2", Gate::Commit, true, None)
            .await
            .expect("decide");
        assert_eq!(decided.status, ApprovalStatus::Approved);
        assert!(decided.requested_at.is_some(), "request context preserved");
        assert!(decided.decided_at.is_some());
    }

    #[tokio::test]
    async fn decide_upserts_without_prior_request() {
        // Backward compat: `ccswarm approve plan --id x` with no pending record.
        let (_dir, store) = store();
        let decided = store
            .decide("run-3", Gate::Plan, false, Some("not ready"))
            .await
            .expect("decide");
        assert_eq!(decided.status, ApprovalStatus::Rejected);
        assert_eq!(decided.reason.as_deref(), Some("not ready"));
        assert!(decided.requested_at.is_none());
    }

    #[tokio::test]
    async fn rejects_path_traversal_ids() {
        let (_dir, store) = store();
        assert!(
            store
                .request("../escape", Gate::Commit, "t", "auto")
                .await
                .is_err()
        );
        assert!(store.decide("a/b", Gate::Plan, true, None).await.is_err());
        assert!(store.get("..").await.is_err());
    }

    #[tokio::test]
    async fn wait_for_decision_returns_approved() {
        let (dir, store) = store();
        store
            .request("run-4", Gate::Commit, "t", "auto")
            .await
            .expect("request");

        let decider = ApprovalStore::new(dir.path());
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            decider
                .decide("run-4", Gate::Commit, true, None)
                .await
                .expect("decide");
        });

        let outcome = store
            .wait_for_decision(
                "run-4",
                Gate::Commit,
                Duration::from_millis(10),
                Duration::from_secs(5),
            )
            .await
            .expect("wait");
        assert_eq!(outcome, GateOutcome::Approved);
    }

    #[tokio::test]
    async fn wait_for_decision_ignores_decisions_for_other_gates() {
        // `approve plan --id <run-id>` must not release a waiting commit gate.
        let (dir, store) = store();
        store
            .request("run-7", Gate::Commit, "t", "auto")
            .await
            .expect("request");

        let decider = ApprovalStore::new(dir.path());
        decider
            .decide("run-7", Gate::Plan, true, None)
            .await
            .expect("decide");

        let outcome = store
            .wait_for_decision(
                "run-7",
                Gate::Commit,
                Duration::from_millis(10),
                Duration::from_millis(60),
            )
            .await
            .expect("wait");
        assert_eq!(outcome, GateOutcome::TimedOut);
    }

    #[tokio::test]
    async fn wait_for_decision_carries_rejection_reason() {
        let (dir, store) = store();
        store
            .request("run-5", Gate::Commit, "t", "auto")
            .await
            .expect("request");

        let decider = ApprovalStore::new(dir.path());
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            decider
                .decide("run-5", Gate::Commit, false, Some("no"))
                .await
                .expect("decide");
        });

        let outcome = store
            .wait_for_decision(
                "run-5",
                Gate::Commit,
                Duration::from_millis(10),
                Duration::from_secs(5),
            )
            .await
            .expect("wait");
        assert_eq!(outcome, GateOutcome::Rejected(Some("no".to_string())));
    }

    #[tokio::test]
    async fn wait_for_decision_times_out_and_tolerates_corrupt_json() {
        let (dir, store) = store();
        store
            .request("run-6", Gate::Commit, "t", "auto")
            .await
            .expect("request");
        // Corrupt the record mid-wait: polling must keep going, not error.
        tokio::fs::write(
            dir.path().join(".ccswarm/approvals/run-6.json"),
            "{not json",
        )
        .await
        .expect("corrupt");

        let outcome = store
            .wait_for_decision(
                "run-6",
                Gate::Commit,
                Duration::from_millis(10),
                Duration::from_millis(60),
            )
            .await
            .expect("wait");
        assert_eq!(outcome, GateOutcome::TimedOut);
    }

    #[tokio::test]
    async fn list_returns_all_records_sorted() {
        let (_dir, store) = store();
        store
            .request("run-b", Gate::Commit, "t", "auto")
            .await
            .expect("request");
        store
            .decide("run-a", Gate::Plan, true, None)
            .await
            .expect("decide");
        let records = store.list().await.expect("list");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "run-a");
        assert_eq!(records[1].id, "run-b");
    }
}
