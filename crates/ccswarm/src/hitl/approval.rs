//! Approval request and result types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::policy::{ActionType, RiskLevel};

/// A request for human approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub id: String,
    /// Description of the action
    pub description: String,
    /// Type of action being requested
    pub action_type: ActionType,
    /// Risk level of the action
    pub risk_level: RiskLevel,
    /// Agent requesting approval
    pub agent_id: Option<String>,
    /// Task associated with this request
    pub task_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Files that will be affected
    pub affected_files: Vec<String>,
    /// Commands that will be executed
    pub commands: Vec<String>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// Environment (production, staging, etc.)
    pub environment: Option<String>,
    /// Whether the action is reversible
    pub reversible: bool,
    /// Estimated impact description
    pub impact: Option<String>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        description: impl Into<String>,
        action_type: ActionType,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            action_type,
            risk_level,
            agent_id: None,
            task_id: None,
            session_id: None,
            affected_files: Vec::new(),
            commands: Vec::new(),
            context: HashMap::new(),
            created_at: Utc::now(),
            environment: None,
            reversible: true,
            impact: None,
        }
    }

    /// Set agent ID
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set task ID
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add affected files
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.affected_files = files;
        self
    }

    /// Add commands
    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        self.commands = commands;
        self
    }

    /// Add context
    pub fn with_context(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Set reversibility
    pub fn with_reversible(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }

    /// Set impact description
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = Some(impact.into());
        self
    }

    /// Check if this is a high-risk request
    pub fn is_high_risk(&self) -> bool {
        self.risk_level >= RiskLevel::High
    }

    /// Check if this affects production
    pub fn is_production(&self) -> bool {
        self.environment.as_deref() == Some("production")
    }

    /// Get a summary of the request
    pub fn summary(&self) -> String {
        format!(
            "[{:?}] {} (Risk: {:?})",
            self.action_type, self.description, self.risk_level
        )
    }
}

/// Result of an approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    /// Request ID
    pub request_id: String,
    /// Status of the approval
    pub status: ApprovalStatus,
    /// Who made the decision
    pub approved_by: Option<String>,
    /// Reason for the decision
    pub reason: Option<String>,
    /// When the decision was made
    pub timestamp: DateTime<Utc>,
    /// Modified action (if approved with modifications)
    pub modified_action: Option<serde_json::Value>,
}

impl ApprovalResult {
    /// Check if approved
    pub fn is_approved(&self) -> bool {
        matches!(
            self.status,
            ApprovalStatus::Approved | ApprovalStatus::ApprovedWithModifications
        )
    }

    /// Check if rejected
    pub fn is_rejected(&self) -> bool {
        self.status == ApprovalStatus::Rejected
    }

    /// Check if timed out
    pub fn is_timeout(&self) -> bool {
        self.status == ApprovalStatus::Timeout
    }
}

/// Status of an approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Pending human review
    Pending,
    /// Approved by human
    Approved,
    /// Approved with modifications
    ApprovedWithModifications,
    /// Rejected by human
    Rejected,
    /// Request timed out
    Timeout,
    /// Cancelled by requester
    Cancelled,
}

/// A pending approval waiting for decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    /// The approval request
    pub request: ApprovalRequest,
    /// When this approval expires
    pub expires_at: DateTime<Utc>,
    /// Number of reminders sent
    pub reminders_sent: u32,
    /// Notified channels
    pub notified_channels: Vec<ApprovalChannel>,
    /// Additional notes added during review
    pub notes: Vec<ApprovalNote>,
}

impl PendingApproval {
    /// Create a new pending approval
    pub fn new(request: ApprovalRequest, timeout_secs: u64) -> Self {
        let expires_at = Utc::now() + chrono::Duration::seconds(timeout_secs as i64);
        Self {
            request,
            expires_at,
            reminders_sent: 0,
            notified_channels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Check if this approval has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get time remaining in seconds
    pub fn time_remaining_secs(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }

    /// Add a note
    pub fn add_note(&mut self, author: impl Into<String>, content: impl Into<String>) {
        self.notes.push(ApprovalNote {
            author: author.into(),
            content: content.into(),
            timestamp: Utc::now(),
        });
    }
}

/// A note added during the approval review process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalNote {
    /// Who added the note
    pub author: String,
    /// Note content
    pub content: String,
    /// When the note was added
    pub timestamp: DateTime<Utc>,
}

/// Channels for sending approval notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalChannel {
    /// Command-line interface
    Cli,
    /// Slack notification
    Slack,
    /// Email notification
    Email,
    /// Discord notification
    Discord,
    /// Webhook
    Webhook,
    /// SMS
    Sms,
}

impl ApprovalChannel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ApprovalChannel::Cli => "CLI",
            ApprovalChannel::Slack => "Slack",
            ApprovalChannel::Email => "Email",
            ApprovalChannel::Discord => "Discord",
            ApprovalChannel::Webhook => "Webhook",
            ApprovalChannel::Sms => "SMS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_request_creation() {
        let request = ApprovalRequest::new(
            "Delete important file",
            ActionType::FileDelete,
            RiskLevel::High,
        );

        assert!(!request.id.is_empty());
        assert_eq!(request.description, "Delete important file");
        assert_eq!(request.action_type, ActionType::FileDelete);
        assert_eq!(request.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_approval_request_builder() {
        let request = ApprovalRequest::new("Deploy", ActionType::Deploy, RiskLevel::Critical)
            .with_agent("frontend")
            .with_task("task-123")
            .with_environment("production")
            .with_files(vec!["app.js".to_string()])
            .with_reversible(false)
            .with_impact("Will affect all users");

        assert_eq!(request.agent_id, Some("frontend".to_string()));
        assert_eq!(request.task_id, Some("task-123".to_string()));
        assert_eq!(request.environment, Some("production".to_string()));
        assert!(!request.reversible);
        assert!(request.is_production());
        assert!(request.is_high_risk());
    }

    #[test]
    fn test_pending_approval() {
        let request = ApprovalRequest::new("Test", ActionType::FileWrite, RiskLevel::Medium);
        let pending = PendingApproval::new(request, 300);

        assert!(!pending.is_expired());
        assert!(pending.time_remaining_secs() > 0);
        assert!(pending.time_remaining_secs() <= 300);
    }

    #[test]
    fn test_pending_approval_notes() {
        let request = ApprovalRequest::new("Test", ActionType::FileWrite, RiskLevel::Low);
        let mut pending = PendingApproval::new(request, 300);

        pending.add_note("reviewer", "Looks safe");
        assert_eq!(pending.notes.len(), 1);
        assert_eq!(pending.notes[0].content, "Looks safe");
    }

    #[test]
    fn test_approval_result() {
        let result = ApprovalResult {
            request_id: "test-123".to_string(),
            status: ApprovalStatus::Approved,
            approved_by: Some("admin".to_string()),
            reason: None,
            timestamp: Utc::now(),
            modified_action: None,
        };

        assert!(result.is_approved());
        assert!(!result.is_rejected());
    }

    #[test]
    fn test_request_summary() {
        let request = ApprovalRequest::new(
            "Delete database",
            ActionType::DatabaseModify,
            RiskLevel::Critical,
        );

        let summary = request.summary();
        assert!(summary.contains("Delete database"));
        assert!(summary.contains("Critical"));
    }
}
