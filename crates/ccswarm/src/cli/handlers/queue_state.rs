//! Shared persistent types for `.ccswarm/queue.yaml`.
//!
//! Single source of truth used by both `queue` (interactive) and `auto` (autonomous)
//! handlers. Previously duplicated between handlers/queue.rs and handlers/auto.rs —
//! keeping the definitions in one place prevents silent field-drift between the two
//! readers.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

#[cfg(unix)]
use nix::fcntl::{Flock, FlockArg};
#[cfg(unix)]
use std::fs::{File, OpenOptions};

pub(crate) const QUEUE_FILE: &str = ".ccswarm/queue.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClaimState {
    Unclaimed,
    Running { run_id: String, started_at_ms: u64 },
    RetryQueued { attempt: u32, due_at_ms: u64 },
    Released { reason: String },
}

#[allow(clippy::derivable_impls)]
impl Default for ClaimState {
    fn default() -> Self {
        Self::Unclaimed
    }
}

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
    #[serde(default)]
    pub claim: ClaimState,
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
        Some(parent) => parent.join(format!(
            ".queue.yaml.tmp.{}.{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        )),
        None => path.with_extension(format!("yaml.tmp.{}", uuid::Uuid::new_v4())),
    };
    tokio::fs::write(&tmp_path, body).await?;
    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct QueueState {
    path: PathBuf,
    task_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
    file_lock: Arc<Mutex<()>>,
}

impl QueueState {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let file_lock = global_file_lock(&path);
        Self {
            path,
            task_locks: global_task_locks(),
            file_lock,
        }
    }

    pub(crate) async fn load(&self) -> Result<QueueFile> {
        load_queue(&self.path).await
    }

    pub(crate) async fn try_claim(&self, id: &str, run_id: &str) -> Result<bool> {
        let task_lock = self.task_lock(id);
        let _task_guard = task_lock.lock().await;
        let _file_guard = self.file_lock.lock().await;
        let _process_guard = self.acquire_process_lock().await?;

        let mut queue = load_queue(&self.path).await?;
        let task = find_task_mut(&mut queue, id)?;

        match task.claim {
            ClaimState::Unclaimed | ClaimState::Released { .. } if task.state == "pending" => {
                task.claim = ClaimState::Running {
                    run_id: run_id.to_string(),
                    started_at_ms: now_ms(),
                };
                task.state = "running".to_string();
                task.completed_at = None;
                task.run_id = Some(run_id.to_string());
                save_queue(&self.path, &queue).await?;
                Ok(true)
            }
            ClaimState::Unclaimed
            | ClaimState::Released { .. }
            | ClaimState::Running { .. }
            | ClaimState::RetryQueued { .. } => Ok(false),
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn mark_retry_queued(
        &self,
        id: &str,
        attempt: u32,
        due_at_ms: u64,
    ) -> Result<()> {
        self.update_claim(id, |claim| match claim {
            ClaimState::Running { .. } => {
                *claim = ClaimState::RetryQueued { attempt, due_at_ms };
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Cannot mark task {id} retry queued unless it is running"
            )),
        })
        .await
    }

    pub(crate) async fn release(&self, id: &str, reason: &str) -> Result<()> {
        self.update_task(id, |task| {
            task.claim = ClaimState::Released {
                reason: reason.to_string(),
            };
            if task.state == "running" {
                task.state = default_state();
                task.completed_at = None;
                task.run_id = None;
            }
            Ok(())
        })
        .await
    }

    pub(crate) async fn release_terminal_if_running(
        &self,
        id: &str,
        observed_run_id: &str,
        observed_started_at_ms: u64,
        terminal_state: &str,
        reason: &str,
    ) -> Result<bool> {
        if !matches!(terminal_state, "completed" | "failed") {
            return Err(anyhow::anyhow!(
                "invalid terminal queue state for reconciliation: {terminal_state}"
            ));
        }

        let mut updated = false;
        self.update_task(id, |task| {
            if matches!(
                &task.claim,
                ClaimState::Running { run_id, started_at_ms }
                    if run_id == observed_run_id && *started_at_ms == observed_started_at_ms
            ) {
                task.claim = ClaimState::Released {
                    reason: reason.to_string(),
                };
                task.state = terminal_state.to_string();
                task.completed_at = Some(Utc::now());
                task.run_id = Some(observed_run_id.to_string());
                updated = true;
            }
            Ok(())
        })
        .await?;
        Ok(updated)
    }

    pub(crate) async fn make_retry_ready_if_due(
        &self,
        id: &str,
        observed_attempt: u32,
        observed_due_at_ms: u64,
        now_ms: u64,
    ) -> Result<bool> {
        let mut updated = false;
        self.update_task(id, |task| {
            if matches!(
                &task.claim,
                ClaimState::RetryQueued { attempt, due_at_ms }
                    if *attempt == observed_attempt
                        && *due_at_ms == observed_due_at_ms
                        && *due_at_ms <= now_ms
            ) {
                task.claim = ClaimState::Unclaimed;
                task.state = default_state();
                task.completed_at = None;
                updated = true;
            }
            Ok(())
        })
        .await?;
        Ok(updated)
    }

    pub(crate) async fn update_task<F>(&self, id: &str, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut QueueTask) -> Result<()>,
    {
        let task_lock = self.task_lock(id);
        let _task_guard = task_lock.lock().await;
        let _file_guard = self.file_lock.lock().await;
        let _process_guard = self.acquire_process_lock().await?;

        let mut queue = load_queue(&self.path).await?;
        let task = find_task_mut(&mut queue, id)?;
        mutate(task)?;
        save_queue(&self.path, &queue).await
    }

    pub(crate) async fn update_queue<F>(&self, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut QueueFile) -> Result<()>,
    {
        let _file_guard = self.file_lock.lock().await;
        let _process_guard = self.acquire_process_lock().await?;

        let mut queue = load_queue(&self.path).await?;
        mutate(&mut queue)?;
        save_queue(&self.path, &queue).await
    }

    async fn update_claim<F>(&self, id: &str, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut ClaimState) -> Result<()>,
    {
        self.update_task(id, |task| mutate(&mut task.claim)).await
    }

    fn task_lock(&self, id: &str) -> Arc<Mutex<()>> {
        let key = format!("{}:{id}", self.path.display());
        self.task_locks
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    async fn acquire_process_lock(&self) -> Result<QueueProcessLock> {
        let lock_path = self.lock_path();
        if let Some(parent) = lock_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::task::spawn_blocking(move || QueueProcessLock::acquire(&lock_path))
            .await
            .context("Queue process lock task panicked")?
    }

    fn lock_path(&self) -> PathBuf {
        self.path.with_extension("yaml.lock")
    }
}

type LockMap = DashMap<String, Arc<Mutex<()>>>;

fn global_task_locks() -> Arc<LockMap> {
    static TASK_LOCKS: OnceLock<Arc<LockMap>> = OnceLock::new();
    Arc::clone(TASK_LOCKS.get_or_init(|| Arc::new(DashMap::new())))
}

fn global_file_lock(path: &Path) -> Arc<Mutex<()>> {
    static FILE_LOCKS: OnceLock<LockMap> = OnceLock::new();
    FILE_LOCKS
        .get_or_init(DashMap::new)
        .entry(path.to_string_lossy().into_owned())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

#[cfg(unix)]
struct QueueProcessLock {
    _lock: Flock<File>,
}

#[cfg(unix)]
impl QueueProcessLock {
    fn acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("Failed to open queue lock file {}", path.display()))?;
        let lock = Flock::lock(file, FlockArg::LockExclusive)
            .map_err(|(_, err)| anyhow::anyhow!("Failed to lock queue file: {err}"))?;
        Ok(Self { _lock: lock })
    }
}

#[cfg(not(unix))]
struct QueueProcessLock;

#[cfg(not(unix))]
impl QueueProcessLock {
    fn acquire(_path: &Path) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Queue claim operations require process-level file locking, which is unsupported on this platform"
        ))
    }
}

fn find_task_mut<'a>(queue: &'a mut QueueFile, id: &str) -> Result<&'a mut QueueTask> {
    queue
        .tasks
        .iter_mut()
        .find(|task| task.id == id)
        .with_context(|| format!("Queue task not found: {id}"))
}

fn now_ms() -> u64 {
    Utc::now().timestamp_millis().max(0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_task(id: &str) -> QueueTask {
        QueueTask {
            id: id.to_string(),
            task: "test task".to_string(),
            flow: None,
            state: default_state(),
            created_at: Utc::now(),
            completed_at: None,
            run_id: None,
            claim: ClaimState::Unclaimed,
        }
    }

    async fn write_queue(path: &Path, tasks: Vec<QueueTask>) -> Result<()> {
        save_queue(path, &QueueFile { tasks }).await
    }

    #[test]
    fn test_claim_state_default_is_unclaimed() {
        assert_eq!(ClaimState::default(), ClaimState::Unclaimed);
    }

    #[test]
    fn test_claim_state_missing_field_deserializes_to_default() -> Result<()> {
        let raw = r#"{
            "id": "q-old",
            "task": "legacy task",
            "state": "pending",
            "created_at": "2026-05-08T00:00:00Z"
        }"#;

        let task: QueueTask = serde_json::from_str(raw)?;

        assert_eq!(task.claim, ClaimState::Unclaimed);
        let serialized = serde_json::to_string(&task)?;
        let round_tripped: QueueTask = serde_json::from_str(&serialized)?;
        assert_eq!(round_tripped.claim, ClaimState::Unclaimed);
        Ok(())
    }

    #[tokio::test]
    async fn test_try_claim_succeeds_then_blocks_second_caller() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1")]).await?;
        let state = QueueState::new(path);

        assert!(state.try_claim("q-1", "run-1").await?);
        let queue = state.load().await?;
        assert_eq!(queue.tasks[0].state, "running");
        assert_eq!(queue.tasks[0].run_id.as_deref(), Some("run-1"));
        assert!(!state.try_claim("q-1", "run-2").await?);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_try_claim_blocks_independent_queue_state_callers() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1")]).await?;

        let caller_count = 16;
        let barrier = Arc::new(tokio::sync::Barrier::new(caller_count));
        let mut handles = Vec::with_capacity(caller_count);

        for i in 0..caller_count {
            let state = QueueState::new(path.clone());
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                state.try_claim("q-1", &format!("run-{i}")).await
            }));
        }

        let mut successful_claims = 0usize;
        for handle in handles {
            if handle.await.context("claim task join failed")?? {
                successful_claims += 1;
            }
        }

        assert_eq!(successful_claims, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_release_resets_claim() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1")]).await?;
        let state = QueueState::new(path.clone());

        assert!(state.try_claim("q-1", "run-1").await?);
        state.release("q-1", "manual").await?;

        let queue = load_queue(&path).await?;
        assert_eq!(
            queue.tasks[0].claim,
            ClaimState::Released {
                reason: "manual".to_string()
            }
        );
        assert!(state.try_claim("q-1", "run-2").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_claim_blocks_released_completed_task() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        let mut task = queue_task("q-1");
        task.state = "completed".to_string();
        task.claim = ClaimState::Released {
            reason: "completed".to_string(),
        };
        write_queue(&path, vec![task]).await?;
        let state = QueueState::new(path);

        assert!(!state.try_claim("q-1", "run-2").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_retry_queued_only_from_running() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1"), queue_task("q-2")]).await?;
        let state = QueueState::new(path.clone());

        let unclaimed_result = state.mark_retry_queued("q-1", 1, 42).await;
        assert!(unclaimed_result.is_err());

        assert!(state.try_claim("q-2", "run-1").await?);
        state.mark_retry_queued("q-2", 2, 84).await?;

        let queue = load_queue(&path).await?;
        let retry_task = queue
            .tasks
            .iter()
            .find(|task| task.id == "q-2")
            .context("missing retry task")?;
        assert_eq!(
            retry_task.claim,
            ClaimState::RetryQueued {
                attempt: 2,
                due_at_ms: 84
            }
        );
        assert!(!state.try_claim("q-2", "run-2").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_release_terminal_if_running_updates_terminal_fields() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1")]).await?;
        let state = QueueState::new(path.clone());

        assert!(state.try_claim("q-1", "run-1").await?);
        let queue = load_queue(&path).await?;
        let ClaimState::Running { started_at_ms, .. } = queue.tasks[0].claim else {
            anyhow::bail!("expected running claim");
        };

        assert!(
            state
                .release_terminal_if_running(
                    "q-1",
                    "run-1",
                    started_at_ms,
                    "completed",
                    "terminal_state_reconciled",
                )
                .await?
        );

        let queue = load_queue(&path).await?;
        assert_eq!(queue.tasks[0].state, "completed");
        assert_eq!(queue.tasks[0].run_id.as_deref(), Some("run-1"));
        assert!(queue.tasks[0].completed_at.is_some());
        assert_eq!(
            queue.tasks[0].claim,
            ClaimState::Released {
                reason: "terminal_state_reconciled".to_string()
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_release_terminal_if_running_does_not_overwrite_newer_claim() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        write_queue(&path, vec![queue_task("q-1")]).await?;
        let state = QueueState::new(path.clone());

        assert!(state.try_claim("q-1", "run-new").await?);
        let released = state
            .release_terminal_if_running(
                "q-1",
                "run-old",
                1,
                "completed",
                "terminal_state_reconciled",
            )
            .await?;

        assert!(!released);
        let queue = load_queue(&path).await?;
        assert!(matches!(
            queue.tasks[0].claim,
            ClaimState::Running { ref run_id, .. } if run_id == "run-new"
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_make_retry_ready_if_due_restores_pending_state() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("queue.yaml");
        let mut task = queue_task("q-1");
        task.state = "running".to_string();
        task.claim = ClaimState::RetryQueued {
            attempt: 2,
            due_at_ms: 42,
        };
        write_queue(&path, vec![task]).await?;
        let state = QueueState::new(path.clone());

        assert!(state.make_retry_ready_if_due("q-1", 2, 42, 42).await?);

        let queue = load_queue(&path).await?;
        assert_eq!(queue.tasks[0].claim, ClaimState::Unclaimed);
        assert_eq!(queue.tasks[0].state, "pending");
        Ok(())
    }
}
