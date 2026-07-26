use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::output::{BuildStatus, LogLevel, ParsedOutput};

/// Unique identifier for persisted agent execution context.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionSessionId(Uuid);

impl Default for ExecutionSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionSessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse_str(s: &str) -> anyhow::Result<Self> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl std::fmt::Display for ExecutionSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Triage axis for a provider or A2A execution turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AttentionState {
    #[default]
    Idle,
    Running,
    Waiting,
    Error,
    Done,
}

impl AttentionState {
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
                LogLevel::Error => Some(Self::Error),
                _ => None,
            },
            ParsedOutput::CodeExecution { .. } | ParsedOutput::PlainText(_) => None,
        }
    }
}

impl std::fmt::Display for AttentionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Error => "error",
            Self::Done => "done",
        };
        f.write_str(value)
    }
}
