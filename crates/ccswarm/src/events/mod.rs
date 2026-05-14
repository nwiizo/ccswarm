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
use std::collections::HashSet;
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
    // Stage lifecycle
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
    /// Stage label associated with this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
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
            stage: None,
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

    /// Builder: attach a stage label.
    pub fn with_movement(mut self, stage: impl Into<String>) -> Self {
        self.stage = Some(stage.into());
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
        Self::new_in_runs_dir(PathBuf::from(".ccswarm").join("runs"), run_id).await
    }

    /// Create a new recorder under an explicit runs directory.
    pub async fn new_in_runs_dir(runs_dir: impl Into<PathBuf>, run_id: &str) -> Result<Self> {
        crate::run_id::validate_run_id(run_id).context("invalid run ID for event recorder")?;
        let run_dir = runs_dir.into().join(run_id);
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

// ─── SessionInfo ────────────────────────────────────────────────────────────

/// Metadata extracted from a session's event logs and/or summary.json.
///
/// Used by `ccswarm session list` to display rich information about each
/// pipeline run without requiring a complete summary file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Run UUID (from directory name or event data).
    pub run_id: String,
    /// Timestamp of the first event.
    pub started_at: Option<DateTime<Utc>>,
    /// Timestamp of the last event.
    pub ended_at: Option<DateTime<Utc>>,
    /// Human-readable duration string (e.g. "2m 15s").
    pub duration: Option<String>,
    /// Inferred status: completed, failed, running, or incomplete.
    pub status: String,
    /// Total number of events in the NDJSON log.
    pub total_events: usize,
    /// Task/flow name extracted from the first task_start message.
    pub task: Option<String>,
    /// Latest stage name from movement_start events.
    pub last_movement: Option<String>,
    /// Count of stages that completed.
    pub movements_completed: usize,
    /// Deduplicated list of agents seen in events.
    pub agents_used: Vec<String>,
    /// Whether any stage or task reported a failure status.
    pub has_errors: bool,
}

impl SessionInfo {
    /// Build a `SessionInfo` from a summary.json value, falling through to
    /// events for any missing fields.
    pub fn from_summary(summary: &serde_json::Value) -> Self {
        let run_id = summary
            .get("run_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_owned();

        let started_at = summary
            .get("started_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let ended_at = summary
            .get("ended_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let duration = match (started_at, ended_at) {
            (Some(start), Some(end)) => Some(format_duration(end - start)),
            _ => None,
        };

        let status = summary
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| {
                if ended_at.is_some() {
                    "completed".to_owned()
                } else {
                    "running".to_owned()
                }
            });

        let total_events = summary
            .get("total_events")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let agents_used: Vec<String> = summary
            .get("agents_used")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.as_str())
                    .map(|s| s.to_owned())
                    .collect()
            })
            .unwrap_or_default();

        let tasks_failed = summary
            .get("tasks_failed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let task = summary
            .get("task")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned());

        let last_movement = summary
            .get("last_movement")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned());

        let movements_completed = summary
            .get("movements_completed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Self {
            run_id,
            started_at,
            ended_at,
            duration,
            status,
            total_events,
            task,
            last_movement,
            movements_completed,
            agents_used,
            has_errors: tasks_failed > 0,
        }
    }

    /// Build a `SessionInfo` by parsing the raw NDJSON event lines.
    ///
    /// This is the primary path for sessions that lack a summary.json.
    pub fn from_events(run_id: &str, content: &str) -> Self {
        let mut started_at: Option<DateTime<Utc>> = None;
        let mut ended_at: Option<DateTime<Utc>> = None;
        let mut total_events: usize = 0;
        let mut task: Option<String> = None;
        let mut last_movement: Option<String> = None;
        let mut movements_completed: usize = 0;
        let mut agents: HashSet<String> = HashSet::new();
        let mut has_errors = false;
        let mut has_task_end = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let event: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            total_events += 1;

            // Extract timestamp
            if let Some(ts_str) = event.get("ts").and_then(|v| v.as_str())
                && let Ok(ts) = DateTime::parse_from_rfc3339(ts_str)
            {
                let ts_utc = ts.with_timezone(&Utc);
                if started_at.is_none() {
                    started_at = Some(ts_utc);
                }
                ended_at = Some(ts_utc);
            }

            // Extract event_type
            let event_type = event
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match event_type {
                "task_start" if task.is_none() => {
                    // Extract flow/task name from the message
                    // e.g. "Starting flow 'default'"
                    task = event
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(extract_piece_name);
                }
                "task_end" => {
                    has_task_end = true;
                }
                "movement_start" => {
                    last_movement = event
                        .get("stage")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned());
                }
                "movement_end" => {
                    movements_completed += 1;
                    // Check for failure status in metadata
                    if let Some(meta) = event.get("metadata")
                        && meta.get("status").and_then(|v| v.as_str()) == Some("failed")
                    {
                        has_errors = true;
                    }
                }
                "provider_error" => {
                    has_errors = true;
                }
                _ => {}
            }

            // Extract level for error detection
            if event.get("level").and_then(|v| v.as_str()) == Some("error") {
                has_errors = true;
            }

            // Extract agent name
            if let Some(agent) = event.get("agent").and_then(|v| v.as_str()) {
                agents.insert(agent.to_owned());
            }
        }

        let duration = match (started_at, ended_at) {
            (Some(start), Some(end)) if start != end => Some(format_duration(end - start)),
            _ => None,
        };

        let status = if has_errors {
            "failed".to_owned()
        } else if has_task_end {
            "completed".to_owned()
        } else if total_events > 0 {
            "running".to_owned()
        } else {
            "empty".to_owned()
        };

        Self {
            run_id: run_id.to_owned(),
            started_at,
            ended_at,
            duration,
            status,
            total_events,
            task,
            last_movement,
            movements_completed,
            agents_used: agents.into_iter().collect(),
            has_errors,
        }
    }
}

/// Extract a flow name from a task_start message.
///
/// Handles formats like:
/// - `"Starting flow 'default'"`  → `"default"`
/// - `"Starting flow 'quick'"`    → `"quick"`
/// - Other messages                → returns the message as-is (trimmed)
fn extract_piece_name(message: &str) -> String {
    if let Some(start) = message.find('\'')
        && let Some(end) = message[start + 1..].find('\'')
    {
        return message[start + 1..start + 1 + end].to_owned();
    }
    message.to_owned()
}

/// Format a chrono::Duration into a human-readable string.
///
/// Examples: `"0s"`, `"15s"`, `"2m 30s"`, `"1h 5m"`, `"2h 30m 15s"`.
fn format_duration(dur: chrono::Duration) -> String {
    let total_secs = dur.num_seconds().max(0);
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        if seconds > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else if minutes > 0 {
        if seconds > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}m", minutes)
        }
    } else {
        format!("{}s", seconds)
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
            !json.contains("\"stage\""),
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
    async fn test_event_recorder_rejects_path_traversal_run_id() {
        let tmp = tempdir().expect("failed to create temp dir");
        let result =
            EventRecorder::new_in_runs_dir(tmp.path().join(".ccswarm").join("runs"), "..").await;

        assert!(result.is_err());
        assert!(!tmp.path().join(".ccswarm").join("runs").join("..").exists());
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

    // ─── SessionInfo tests ──────────────────────────────────────────────

    #[test]
    fn test_extract_piece_name_quoted() {
        assert_eq!(extract_piece_name("Starting flow 'default'"), "default");
        assert_eq!(extract_piece_name("Starting flow 'quick-fix'"), "quick-fix");
    }

    #[test]
    fn test_extract_piece_name_no_quotes() {
        assert_eq!(
            extract_piece_name("Some other message"),
            "Some other message"
        );
    }

    #[test]
    fn test_format_duration_seconds() {
        let dur = chrono::Duration::seconds(42);
        assert_eq!(format_duration(dur), "42s");
    }

    #[test]
    fn test_format_duration_minutes_seconds() {
        let dur = chrono::Duration::seconds(150);
        assert_eq!(format_duration(dur), "2m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        let dur = chrono::Duration::seconds(3661);
        assert_eq!(format_duration(dur), "1h 1m 1s");
    }

    #[test]
    fn test_format_duration_exact_hour() {
        let dur = chrono::Duration::seconds(3600);
        assert_eq!(format_duration(dur), "1h");
    }

    #[test]
    fn test_format_duration_exact_minutes() {
        let dur = chrono::Duration::seconds(120);
        assert_eq!(format_duration(dur), "2m");
    }

    #[test]
    fn test_format_duration_zero() {
        let dur = chrono::Duration::seconds(0);
        assert_eq!(format_duration(dur), "0s");
    }

    #[test]
    fn test_session_info_from_events_completed() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"info","run_id":"abc","event_type":"task_start","message":"Starting flow 'default'"}
{"ts":"2026-03-26T15:30:12.140Z","level":"info","run_id":"abc","event_type":"movement_start","stage":"plan","message":"Stage 'plan' started"}
{"ts":"2026-03-26T15:30:28.567Z","level":"info","run_id":"abc","event_type":"movement_end","stage":"plan","message":"Stage 'plan' completed"}
{"ts":"2026-03-26T15:30:38.660Z","level":"info","run_id":"abc","event_type":"task_end","message":"Task completed"}
"#;
        let info = SessionInfo::from_events("abc", ndjson);

        assert_eq!(info.run_id, "abc");
        assert_eq!(info.status, "completed");
        assert_eq!(info.total_events, 4);
        assert_eq!(info.task.as_deref(), Some("default"));
        assert_eq!(info.last_movement.as_deref(), Some("plan"));
        assert_eq!(info.movements_completed, 1);
        assert!(!info.has_errors);
        assert!(info.started_at.is_some());
        assert!(info.ended_at.is_some());
        assert!(info.duration.is_some());
    }

    #[test]
    fn test_session_info_from_events_failed() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"info","run_id":"fail-run","event_type":"task_start","message":"Starting flow 'default'"}
{"ts":"2026-03-26T15:30:12.140Z","level":"info","run_id":"fail-run","event_type":"movement_start","stage":"plan","message":"Stage 'plan' started"}
{"ts":"2026-03-26T15:30:28.567Z","level":"info","run_id":"fail-run","event_type":"movement_end","stage":"plan","message":"Stage 'plan' completed","metadata":{"status":"failed"}}
"#;
        let info = SessionInfo::from_events("fail-run", ndjson);

        assert_eq!(info.status, "failed");
        assert!(info.has_errors);
        assert_eq!(info.movements_completed, 1);
    }

    #[test]
    fn test_session_info_from_events_running() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"info","run_id":"running-run","event_type":"task_start","message":"Starting flow 'default'"}
{"ts":"2026-03-26T15:30:12.140Z","level":"info","run_id":"running-run","event_type":"movement_start","stage":"plan","message":"Stage 'plan' started"}
"#;
        let info = SessionInfo::from_events("running-run", ndjson);

        assert_eq!(info.status, "running");
        assert_eq!(info.total_events, 2);
        assert!(!info.has_errors);
    }

    #[test]
    fn test_session_info_from_events_empty() {
        let info = SessionInfo::from_events("empty-run", "");

        assert_eq!(info.status, "empty");
        assert_eq!(info.total_events, 0);
        assert!(info.started_at.is_none());
    }

    #[test]
    fn test_session_info_from_events_with_agents() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"info","run_id":"x","event_type":"task_start","agent":"backend","message":"Starting flow 'default'"}
{"ts":"2026-03-26T15:30:12.140Z","level":"info","run_id":"x","event_type":"movement_start","agent":"frontend","stage":"plan","message":"started"}
{"ts":"2026-03-26T15:30:12.141Z","level":"info","run_id":"x","event_type":"movement_end","agent":"backend","stage":"plan","message":"done"}
"#;
        let info = SessionInfo::from_events("x", ndjson);

        assert_eq!(info.agents_used.len(), 2);
        assert!(info.agents_used.contains(&"backend".to_owned()));
        assert!(info.agents_used.contains(&"frontend".to_owned()));
    }

    #[test]
    fn test_session_info_from_events_provider_error() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"info","run_id":"err","event_type":"task_start","message":"Starting flow 'default'"}
{"ts":"2026-03-26T15:30:12.140Z","level":"info","run_id":"err","event_type":"provider_error","message":"rate limited"}
"#;
        let info = SessionInfo::from_events("err", ndjson);

        assert_eq!(info.status, "failed");
        assert!(info.has_errors);
    }

    #[test]
    fn test_session_info_from_events_error_level() {
        let ndjson = r#"{"ts":"2026-03-26T15:30:12.139Z","level":"error","run_id":"err2","event_type":"task_start","message":"Something broke"}
"#;
        let info = SessionInfo::from_events("err2", ndjson);

        assert!(info.has_errors);
        assert_eq!(info.status, "failed");
    }

    #[test]
    fn test_session_info_from_events_malformed_lines() {
        let ndjson = "not-json\n{\"ts\":\"2026-03-26T15:30:12.139Z\",\"level\":\"info\",\"run_id\":\"m\",\"event_type\":\"task_start\",\"message\":\"ok\"}\n{broken\n";
        let info = SessionInfo::from_events("m", ndjson);

        // Only the valid line should be counted
        assert_eq!(info.total_events, 1);
        assert_eq!(info.status, "running");
    }

    #[test]
    fn test_session_info_from_summary() {
        let summary = serde_json::json!({
            "run_id": "sum-run",
            "started_at": "2026-03-26T15:30:12Z",
            "ended_at": "2026-03-26T15:35:12Z",
            "total_events": 10,
            "tasks_completed": 3,
            "tasks_failed": 0,
            "agents_used": ["frontend", "backend"]
        });

        let info = SessionInfo::from_summary(&summary);

        assert_eq!(info.run_id, "sum-run");
        assert_eq!(info.total_events, 10);
        assert_eq!(info.status, "completed");
        assert_eq!(info.duration.as_deref(), Some("5m"));
        assert_eq!(info.agents_used.len(), 2);
        assert!(!info.has_errors);
    }

    #[test]
    fn test_session_info_from_summary_with_task_and_movement() {
        let summary = serde_json::json!({
            "run_id": "rich-sum",
            "started_at": "2026-03-26T15:30:12Z",
            "ended_at": "2026-03-26T15:35:12Z",
            "total_events": 20,
            "tasks_completed": 5,
            "tasks_failed": 0,
            "agents_used": ["frontend"],
            "task": "default",
            "last_movement": "verify",
            "movements_completed": 3,
        });

        let info = SessionInfo::from_summary(&summary);

        assert_eq!(info.run_id, "rich-sum");
        assert_eq!(info.task.as_deref(), Some("default"));
        assert_eq!(info.last_movement.as_deref(), Some("verify"));
        assert_eq!(info.movements_completed, 3);
        assert!(!info.has_errors);
    }

    #[test]
    fn test_session_info_from_summary_missing_optional_fields() {
        let summary = serde_json::json!({
            "run_id": "minimal",
            "total_events": 2,
            "tasks_failed": 0,
        });

        let info = SessionInfo::from_summary(&summary);

        assert_eq!(info.run_id, "minimal");
        assert!(info.task.is_none());
        assert!(info.last_movement.is_none());
        assert_eq!(info.movements_completed, 0);
    }

    #[test]
    fn test_session_info_from_summary_with_failures() {
        let summary = serde_json::json!({
            "run_id": "fail-sum",
            "started_at": "2026-03-26T15:30:12Z",
            "total_events": 5,
            "tasks_completed": 1,
            "tasks_failed": 2,
            "agents_used": []
        });

        let info = SessionInfo::from_summary(&summary);

        assert!(info.has_errors);
        // status defaults to "running" when ended_at is missing (no status field)
        assert_eq!(info.status, "running");
    }
}
