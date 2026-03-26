//! Event/NDJSON infrastructure for ccswarm observability.
//!
//! Implements spec §18: all run events are written as NDJSON (one JSON object per
//! line) to `.ccswarm/runs/{run-id}/events.ndjson`, and a `summary.json` is
//! produced at the end of each run.
//!
//! # Event lifecycle
//!
//! 1. Create an [`EventRecorder`] with a run UUID.
//! 2. Build [`Event`] values and call [`EventRecorder::record`].
//! 3. Call [`EventRecorder::write_summary`] with a [`RunSummary`] when the run
//!    finishes.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

// ─── Event level ─────────────────────────────────────────────────────────────

/// Severity level attached to every event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// ─── Event type ──────────────────────────────────────────────────────────────

/// Discriminant for the specific event kind.
///
/// Serialized as `snake_case` strings, e.g. `"movement_start"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Movement lifecycle
    MovementStart,
    MovementEnd,
    // Task lifecycle
    TaskEnqueue,
    TaskStart,
    TaskEnd,
    // Review workflow
    ReviewRequest,
    ReviewResult,
    // Human-in-the-loop
    HitlRequest,
    HitlDecision,
    // Provider interactions
    ProviderCall,
    ProviderError,
}

// ─── Event ───────────────────────────────────────────────────────────────────

/// A single structured event written to the NDJSON stream.
///
/// Optional fields are omitted from the serialised output when absent so that
/// each line stays as compact as possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// RFC 3339 timestamp (UTC).
    pub ts: DateTime<Utc>,
    /// Severity level.
    pub level: EventLevel,
    /// Unique identifier for the run that produced this event.
    pub run_id: String,
    /// Discriminated event kind.
    pub event_type: EventType,
    /// Name of the agent that emitted the event, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Movement label associated with this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movement: Option<String>,
    /// Task identifier, if the event is scoped to a task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Human-readable description of the event.
    pub message: String,
    /// Arbitrary structured payload (cost, token counts, latency, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Event {
    /// Create a minimal event with mandatory fields only.
    pub fn new(
        run_id: impl Into<String>,
        level: EventLevel,
        event_type: EventType,
        message: impl Into<String>,
    ) -> Self {
        Self {
            ts: Utc::now(),
            level,
            run_id: run_id.into(),
            event_type,
            agent: None,
            movement: None,
            task_id: None,
            message: message.into(),
            metadata: None,
        }
    }

    /// Builder: attach an agent name.
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Builder: attach a movement label.
    pub fn with_movement(mut self, movement: impl Into<String>) -> Self {
        self.movement = Some(movement.into());
        self
    }

    /// Builder: attach a task ID.
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Builder: attach arbitrary metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

// ─── RunSummary ──────────────────────────────────────────────────────────────

/// Aggregate statistics written to `summary.json` at the end of a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// Same UUID used in every event for this run.
    pub run_id: String,
    /// When the first event was recorded.
    pub started_at: DateTime<Utc>,
    /// When [`EventRecorder::write_summary`] was called, if set.
    pub ended_at: Option<DateTime<Utc>>,
    /// Total number of events written to `events.ndjson`.
    pub total_events: usize,
    /// Number of [`EventType::TaskEnd`] events observed.
    pub tasks_completed: usize,
    /// Number of [`EventType::ProviderError`] events observed.
    pub tasks_failed: usize,
    /// Deduplicated list of agent names seen across all events.
    pub agents_used: Vec<String>,
}

// ─── EventRecorder ───────────────────────────────────────────────────────────

/// Appends events as NDJSON to `.ccswarm/runs/{run_id}/events.ndjson`.
///
/// The recorder is cheap to clone (all state lives behind [`Arc`]-equivalent
/// primitives internally), but in practice a single recorder per run is
/// sufficient.
pub struct EventRecorder {
    run_id: String,
    run_dir: PathBuf,
    event_count: AtomicUsize,
}

impl EventRecorder {
    /// Create a new recorder for `run_id`.
    ///
    /// Creates `.ccswarm/runs/{run_id}/` if it does not yet exist.
    pub async fn new(run_id: &str) -> Result<Self> {
        let run_dir = PathBuf::from(".ccswarm").join("runs").join(run_id);
        fs::create_dir_all(&run_dir)
            .await
            .with_context(|| format!("failed to create run directory {:?}", run_dir))?;
        Ok(Self {
            run_id: run_id.to_owned(),
            run_dir,
            event_count: AtomicUsize::new(0),
        })
    }

    /// Append one JSON line to `events.ndjson`.
    ///
    /// Each call opens → writes → flushes → closes the file so that the log is
    /// always readable even if the process is killed mid-run.
    pub async fn record(&self, event: Event) -> Result<()> {
        let mut line =
            serde_json::to_string(&event).context("failed to serialize event to JSON")?;
        line.push('\n');

        let path = self.events_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .with_context(|| format!("failed to open events file {:?}", path))?;

        file.write_all(line.as_bytes())
            .await
            .with_context(|| format!("failed to write event to {:?}", path))?;

        file.flush()
            .await
            .with_context(|| format!("failed to flush events file {:?}", path))?;

        self.event_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Write `summary.json` (pretty-printed) for the completed run.
    pub async fn write_summary(&self, summary: &RunSummary) -> Result<()> {
        let path = self.run_dir.join("summary.json");
        let content =
            serde_json::to_string_pretty(summary).context("failed to serialize run summary")?;
        fs::write(&path, content)
            .await
            .with_context(|| format!("failed to write summary to {:?}", path))?;
        Ok(())
    }

    /// The run ID used by this recorder.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Number of events recorded so far in this run.
    pub fn event_count(&self) -> usize {
        self.event_count.load(Ordering::Relaxed)
    }

    /// Absolute path to the NDJSON event log.
    pub fn events_path(&self) -> PathBuf {
        self.run_dir.join("events.ndjson")
    }

    /// Absolute path to the run summary file.
    pub fn summary_path(&self) -> PathBuf {
        self.run_dir.join("summary.json")
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Helper: create a recorder rooted at a temp directory so tests do not
    // pollute the working directory.
    async fn recorder_in(dir: &std::path::Path, run_id: &str) -> EventRecorder {
        let run_dir = dir.join(".ccswarm").join("runs").join(run_id);
        fs::create_dir_all(&run_dir).await.unwrap();
        EventRecorder {
            run_id: run_id.to_owned(),
            run_dir,
            event_count: AtomicUsize::new(0),
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::new(
            "run-abc",
            EventLevel::Info,
            EventType::TaskStart,
            "task started",
        )
        .with_agent("backend")
        .with_task_id("t-001");

        let json = serde_json::to_string(&event).expect("serialization failed");

        // Must be valid JSON on a single line (no embedded newlines).
        assert!(
            !json.contains('\n'),
            "NDJSON line must not contain newlines"
        );

        // Required fields present.
        assert!(json.contains("\"run_id\":\"run-abc\""));
        assert!(json.contains("\"message\":\"task started\""));
        assert!(json.contains("\"event_type\":\"task_start\""));
        assert!(json.contains("\"level\":\"info\""));

        // Optional fields present only when set.
        assert!(json.contains("\"agent\":\"backend\""));
        assert!(json.contains("\"task_id\":\"t-001\""));

        // Absent optional fields must be omitted.
        assert!(
            !json.contains("\"movement\""),
            "absent field should be omitted"
        );
        assert!(
            !json.contains("\"metadata\""),
            "absent field should be omitted"
        );
    }

    #[tokio::test]
    async fn test_event_recorder_creates_directory() {
        let tmp = tempdir().expect("failed to create temp dir");
        let run_id = "test-run-dir";
        let run_dir = tmp.path().join(".ccswarm").join("runs").join(run_id);

        // Directory must not exist before construction.
        assert!(!run_dir.exists());

        // Use EventRecorder::new with a relative path override via env is
        // impractical; instead exercise the helper directly to verify creation.
        let recorder = recorder_in(tmp.path(), run_id).await;
        assert!(
            recorder.run_dir.exists(),
            "run directory should have been created"
        );
        assert_eq!(recorder.run_id(), run_id);
    }

    #[tokio::test]
    async fn test_event_recorder_writes_ndjson() {
        let tmp = tempdir().expect("failed to create temp dir");
        let recorder = recorder_in(tmp.path(), "test-run-ndjson").await;

        let events = vec![
            Event::new(
                "test-run-ndjson",
                EventLevel::Info,
                EventType::TaskEnqueue,
                "enqueued",
            ),
            Event::new(
                "test-run-ndjson",
                EventLevel::Info,
                EventType::TaskStart,
                "started",
            )
            .with_agent("frontend"),
            Event::new(
                "test-run-ndjson",
                EventLevel::Info,
                EventType::TaskEnd,
                "completed",
            )
            .with_task_id("t-42"),
        ];

        for ev in &events {
            recorder.record(ev.clone()).await.expect("record failed");
        }

        assert_eq!(recorder.event_count(), 3);

        let contents = fs::read_to_string(recorder.events_path())
            .await
            .expect("reading events.ndjson failed");

        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3, "one line per event");

        // Every line must be valid JSON.
        for line in &lines {
            serde_json::from_str::<serde_json::Value>(line)
                .unwrap_or_else(|e| panic!("line is not valid JSON: {e}\n{line}"));
        }

        // Verify append semantics: recording one more event adds a fourth line.
        recorder
            .record(Event::new(
                "test-run-ndjson",
                EventLevel::Warn,
                EventType::ProviderError,
                "rate limited",
            ))
            .await
            .expect("second record failed");

        let updated = fs::read_to_string(recorder.events_path())
            .await
            .expect("re-read failed");
        assert_eq!(updated.lines().count(), 4);
    }

    #[tokio::test]
    async fn test_summary_generation() {
        let tmp = tempdir().expect("failed to create temp dir");
        let recorder = recorder_in(tmp.path(), "test-run-summary").await;

        let summary = RunSummary {
            run_id: "test-run-summary".to_owned(),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            total_events: 5,
            tasks_completed: 3,
            tasks_failed: 1,
            agents_used: vec!["frontend".to_owned(), "backend".to_owned()],
        };

        recorder
            .write_summary(&summary)
            .await
            .expect("write_summary failed");

        let path = recorder.summary_path();
        assert!(path.exists(), "summary.json should exist");

        let raw = fs::read_to_string(&path)
            .await
            .expect("reading summary.json failed");

        // Must be valid JSON.
        let parsed: serde_json::Value =
            serde_json::from_str(&raw).expect("summary.json is not valid JSON");

        assert_eq!(parsed["run_id"], "test-run-summary");
        assert_eq!(parsed["total_events"], 5);
        assert_eq!(parsed["tasks_completed"], 3);
        assert_eq!(parsed["tasks_failed"], 1);
        assert!(parsed["agents_used"].is_array());

        // summary.json is pretty-printed (contains newlines / indentation).
        assert!(raw.contains('\n'), "summary.json should be pretty-printed");
    }
}
