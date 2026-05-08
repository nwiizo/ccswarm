//! Attention state for triaging running sessions.
//!
//! Inspired by taskers' attention rail. Each session exposes a single
//! `AttentionState` that captures "what does this session need next?" — so a
//! caller running many sessions in parallel can quickly find the ones that
//! need a human, the ones that errored out, and the ones that finished cleanly.
//!
//! The state is observable through a [`tokio::sync::watch`] channel so UIs and
//! orchestrators can react without polling.

use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::output::{BuildStatus, ParsedOutput};

/// Triage axis for a session: what does it need next?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AttentionState {
    /// Session has not started or is otherwise dormant.
    #[default]
    Idle,
    /// Session is actively running with no special signal raised.
    Running,
    /// Session is blocked waiting for input or human review.
    Waiting,
    /// Session hit an error condition and stopped making progress.
    Error,
    /// Session finished its task successfully.
    Done,
}

impl AttentionState {
    /// Derive an attention update from a parsed output snapshot, if the
    /// snapshot is decisive enough to warrant a transition.
    ///
    /// Returns `None` for ambiguous output (e.g. plain text), in which case
    /// the caller should leave the existing state alone.
    pub fn from_parsed(parsed: &ParsedOutput) -> Option<Self> {
        match parsed {
            ParsedOutput::BuildOutput { status, .. } => match status {
                BuildStatus::Success => Some(Self::Done),
                BuildStatus::Failed(_) => Some(Self::Error),
                BuildStatus::Warning(_) => None,
                BuildStatus::InProgress => Some(Self::Running),
            },
            ParsedOutput::TestResults { failed, .. } => {
                if *failed > 0 {
                    Some(Self::Error)
                } else {
                    Some(Self::Done)
                }
            }
            ParsedOutput::StructuredLog { level, .. } => match level {
                crate::output::LogLevel::Error => Some(Self::Error),
                _ => None,
            },
            ParsedOutput::CodeExecution { .. } | ParsedOutput::PlainText(_) => None,
        }
    }
}

impl std::fmt::Display for AttentionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Error => "error",
            Self::Done => "done",
        };
        f.write_str(s)
    }
}

/// Watch-channel-backed tracker for a session's attention state.
///
/// Cheap to clone (sender side is internally `Arc`-like). Subscribers see the
/// current state on first read and every transition after.
#[derive(Debug)]
pub(crate) struct AttentionTracker {
    tx: watch::Sender<AttentionState>,
}

impl AttentionTracker {
    pub(crate) fn new(initial: AttentionState) -> Self {
        let (tx, _rx) = watch::channel(initial);
        Self { tx }
    }

    pub(crate) fn get(&self) -> AttentionState {
        *self.tx.borrow()
    }

    /// Update the state. Returns `true` if the state actually changed.
    pub(crate) fn set(&self, state: AttentionState) -> bool {
        let changed = *self.tx.borrow() != state;
        if changed {
            // Receiver count of zero is fine; watch::Sender::send only fails
            // when there are no receivers AND we want to know about it.
            // send_replace ignores that and always updates the slot.
            self.tx.send_replace(state);
        }
        changed
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<AttentionState> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{LogContext, LogLevel, TestDetails};

    #[test]
    fn build_success_maps_to_done() {
        let parsed = ParsedOutput::BuildOutput {
            status: BuildStatus::Success,
            artifacts: Vec::new(),
        };
        assert_eq!(
            AttentionState::from_parsed(&parsed),
            Some(AttentionState::Done)
        );
    }

    #[test]
    fn build_failure_maps_to_error() {
        let parsed = ParsedOutput::BuildOutput {
            status: BuildStatus::Failed("boom".into()),
            artifacts: Vec::new(),
        };
        assert_eq!(
            AttentionState::from_parsed(&parsed),
            Some(AttentionState::Error)
        );
    }

    #[test]
    fn passing_tests_map_to_done() {
        let parsed = ParsedOutput::TestResults {
            passed: 10,
            failed: 0,
            details: TestDetails::default(),
        };
        assert_eq!(
            AttentionState::from_parsed(&parsed),
            Some(AttentionState::Done)
        );
    }

    #[test]
    fn failing_tests_map_to_error() {
        let parsed = ParsedOutput::TestResults {
            passed: 5,
            failed: 2,
            details: TestDetails::default(),
        };
        assert_eq!(
            AttentionState::from_parsed(&parsed),
            Some(AttentionState::Error)
        );
    }

    #[test]
    fn plain_text_does_not_force_transition() {
        let parsed = ParsedOutput::PlainText("hello".into());
        assert!(AttentionState::from_parsed(&parsed).is_none());
    }

    #[test]
    fn warning_log_does_not_force_transition() {
        let parsed = ParsedOutput::StructuredLog {
            level: LogLevel::Warning,
            message: "deprecated API".into(),
            context: LogContext::default(),
        };
        assert!(AttentionState::from_parsed(&parsed).is_none());
    }

    #[test]
    fn tracker_set_returns_change_flag() {
        let tracker = AttentionTracker::new(AttentionState::Idle);
        assert!(tracker.set(AttentionState::Running));
        assert!(!tracker.set(AttentionState::Running));
        assert!(tracker.set(AttentionState::Done));
        assert_eq!(tracker.get(), AttentionState::Done);
    }

    #[tokio::test]
    async fn subscriber_observes_transitions() {
        let tracker = AttentionTracker::new(AttentionState::Idle);
        let mut rx = tracker.subscribe();
        assert_eq!(*rx.borrow(), AttentionState::Idle);

        tracker.set(AttentionState::Running);
        rx.changed().await.expect("sender alive");
        assert_eq!(*rx.borrow(), AttentionState::Running);

        tracker.set(AttentionState::Done);
        rx.changed().await.expect("sender alive");
        assert_eq!(*rx.borrow(), AttentionState::Done);
    }
}
