//! Approval workflow engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::approval::{ApprovalRequest, ApprovalResult, ApprovalStatus};
use super::policy::ApprovalPolicy;

/// Configuration for an approval workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow name
    pub name: String,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Required approval count
    pub required_approvals: usize,
    /// Whether to allow modifications
    pub allow_modifications: bool,
    /// Escalation settings
    pub escalation: Option<EscalationConfig>,
    /// Auto-actions based on timeout
    pub timeout_action: TimeoutAction,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            timeout_secs: 300,
            required_approvals: 1,
            allow_modifications: true,
            escalation: None,
            timeout_action: TimeoutAction::Reject,
        }
    }
}

/// Configuration for escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Time after which to escalate (seconds)
    pub escalate_after_secs: u64,
    /// Who to escalate to
    pub escalate_to: Vec<String>,
    /// Maximum escalation levels
    pub max_levels: u32,
}

/// Action to take on timeout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Reject the request
    Reject,
    /// Approve the request
    Approve,
    /// Escalate to next level
    Escalate,
    /// Keep pending (extend timeout)
    Extend,
}

/// State of an approval workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow ID
    pub id: String,
    /// Associated request ID
    pub request_id: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Collected approvals
    pub approvals: Vec<WorkflowApproval>,
    /// Collected rejections
    pub rejections: Vec<WorkflowRejection>,
    /// Comments/notes
    pub comments: Vec<WorkflowComment>,
    /// Current escalation level
    pub escalation_level: u32,
    /// When the workflow started
    pub started_at: DateTime<Utc>,
    /// When the workflow will expire
    pub expires_at: DateTime<Utc>,
    /// When the workflow completed (if done)
    pub completed_at: Option<DateTime<Utc>>,
    /// Final result (if completed)
    pub result: Option<ApprovalResult>,
}

impl WorkflowState {
    /// Create a new workflow state
    pub fn new(request: &ApprovalRequest, config: &WorkflowConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            request_id: request.id.clone(),
            status: WorkflowStatus::Pending,
            approvals: Vec::new(),
            rejections: Vec::new(),
            comments: Vec::new(),
            escalation_level: 0,
            started_at: now,
            expires_at: now + chrono::Duration::seconds(config.timeout_secs as i64),
            completed_at: None,
            result: None,
        }
    }

    /// Check if workflow has enough approvals
    pub fn has_enough_approvals(&self, required: usize) -> bool {
        self.approvals.len() >= required
    }

    /// Check if workflow is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if workflow is completed
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            WorkflowStatus::Approved | WorkflowStatus::Rejected | WorkflowStatus::Expired
        )
    }

    /// Add an approval
    pub fn add_approval(&mut self, approval: WorkflowApproval) {
        self.approvals.push(approval);
    }

    /// Add a rejection
    pub fn add_rejection(&mut self, rejection: WorkflowRejection) {
        self.rejections.push(rejection);
    }

    /// Add a comment
    pub fn add_comment(&mut self, comment: WorkflowComment) {
        self.comments.push(comment);
    }

    /// Complete the workflow
    pub fn complete(&mut self, status: WorkflowStatus, result: ApprovalResult) {
        self.status = status;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
    }
}

/// Status of a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Waiting for approvals
    Pending,
    /// Waiting for additional approvals
    PartiallyApproved,
    /// Fully approved
    Approved,
    /// Rejected
    Rejected,
    /// Timed out
    Expired,
    /// Escalated to higher level
    Escalated,
    /// Cancelled
    Cancelled,
}

/// An approval within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowApproval {
    /// Who approved
    pub approver: String,
    /// When approved
    pub timestamp: DateTime<Utc>,
    /// Optional comment
    pub comment: Option<String>,
    /// Conditions attached (if any)
    pub conditions: Vec<String>,
}

/// A rejection within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRejection {
    /// Who rejected
    pub rejector: String,
    /// When rejected
    pub timestamp: DateTime<Utc>,
    /// Reason for rejection
    pub reason: String,
    /// Suggested alternative (if any)
    pub alternative: Option<String>,
}

/// A comment within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowComment {
    /// Who commented
    pub author: String,
    /// When commented
    pub timestamp: DateTime<Utc>,
    /// Comment content
    pub content: String,
    /// Whether this is a system comment
    pub is_system: bool,
}

/// An approval workflow engine
pub struct ApprovalWorkflow {
    /// Workflow configuration
    config: WorkflowConfig,
    /// Current state
    state: WorkflowState,
    /// Policies to apply
    policies: Vec<ApprovalPolicy>,
    /// Callbacks for state changes
    callbacks: WorkflowCallbacks,
}

/// Callbacks for workflow events
#[derive(Default)]
pub struct WorkflowCallbacks {
    /// Called when workflow is approved
    pub on_approved: Option<Box<dyn Fn(&WorkflowState) + Send + Sync>>,
    /// Called when workflow is rejected
    pub on_rejected: Option<Box<dyn Fn(&WorkflowState) + Send + Sync>>,
    /// Called when workflow expires
    pub on_expired: Option<Box<dyn Fn(&WorkflowState) + Send + Sync>>,
    /// Called when workflow is escalated
    pub on_escalated: Option<Box<dyn Fn(&WorkflowState, u32) + Send + Sync>>,
}

impl ApprovalWorkflow {
    /// Create a new approval workflow
    pub fn new(
        request: &ApprovalRequest,
        config: WorkflowConfig,
        policies: Vec<ApprovalPolicy>,
    ) -> Self {
        let state = WorkflowState::new(request, &config);
        Self {
            config,
            state,
            policies,
            callbacks: WorkflowCallbacks::default(),
        }
    }

    /// Get workflow ID
    pub fn id(&self) -> &str {
        &self.state.id
    }

    /// Get current state
    pub fn state(&self) -> &WorkflowState {
        &self.state
    }

    /// Get configuration
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// Set approval callback
    pub fn on_approved<F>(mut self, callback: F) -> Self
    where
        F: Fn(&WorkflowState) + Send + Sync + 'static,
    {
        self.callbacks.on_approved = Some(Box::new(callback));
        self
    }

    /// Set rejection callback
    pub fn on_rejected<F>(mut self, callback: F) -> Self
    where
        F: Fn(&WorkflowState) + Send + Sync + 'static,
    {
        self.callbacks.on_rejected = Some(Box::new(callback));
        self
    }

    /// Record an approval
    pub fn approve(
        &mut self,
        approver: impl Into<String>,
        comment: Option<String>,
    ) -> Result<WorkflowStatus, String> {
        let approver = approver.into();

        // Check if allowed to approve
        for policy in &self.policies {
            if !policy.can_approve(&approver) {
                return Err(format!(
                    "Approver {} is not allowed by policy {}",
                    approver, policy.name
                ));
            }
        }

        let approval = WorkflowApproval {
            approver: approver.clone(),
            timestamp: Utc::now(),
            comment,
            conditions: Vec::new(),
        };

        self.state.add_approval(approval);

        // Add system comment
        self.state.add_comment(WorkflowComment {
            author: "system".to_string(),
            timestamp: Utc::now(),
            content: format!("{} approved the request", approver),
            is_system: true,
        });

        // Check if we have enough approvals
        if self
            .state
            .has_enough_approvals(self.config.required_approvals)
        {
            // Check required approvers
            let approvers: Vec<String> = self
                .state
                .approvals
                .iter()
                .map(|a| a.approver.clone())
                .collect();
            let all_required = self
                .policies
                .iter()
                .all(|p| p.all_required_approved(&approvers));

            if all_required {
                let result = ApprovalResult {
                    request_id: self.state.request_id.clone(),
                    status: ApprovalStatus::Approved,
                    approved_by: Some(approvers.join(", ")),
                    reason: None,
                    timestamp: Utc::now(),
                    modified_action: None,
                };
                self.state.complete(WorkflowStatus::Approved, result);

                if let Some(ref callback) = self.callbacks.on_approved {
                    callback(&self.state);
                }

                return Ok(WorkflowStatus::Approved);
            }
        }

        self.state.status = WorkflowStatus::PartiallyApproved;
        Ok(WorkflowStatus::PartiallyApproved)
    }

    /// Record a rejection
    pub fn reject(&mut self, rejector: impl Into<String>, reason: String) -> WorkflowStatus {
        let rejector = rejector.into();

        let rejection = WorkflowRejection {
            rejector: rejector.clone(),
            timestamp: Utc::now(),
            reason: reason.clone(),
            alternative: None,
        };

        self.state.add_rejection(rejection);

        let result = ApprovalResult {
            request_id: self.state.request_id.clone(),
            status: ApprovalStatus::Rejected,
            approved_by: Some(rejector.clone()),
            reason: Some(reason),
            timestamp: Utc::now(),
            modified_action: None,
        };

        self.state.complete(WorkflowStatus::Rejected, result);

        if let Some(ref callback) = self.callbacks.on_rejected {
            callback(&self.state);
        }

        WorkflowStatus::Rejected
    }

    /// Add a comment
    pub fn comment(&mut self, author: impl Into<String>, content: impl Into<String>) {
        self.state.add_comment(WorkflowComment {
            author: author.into(),
            timestamp: Utc::now(),
            content: content.into(),
            is_system: false,
        });
    }

    /// Check and handle expiration
    pub fn check_expiration(&mut self) -> Option<WorkflowStatus> {
        if !self.state.is_expired() || self.state.is_completed() {
            return None;
        }

        match self.config.timeout_action {
            TimeoutAction::Reject => {
                let result = ApprovalResult {
                    request_id: self.state.request_id.clone(),
                    status: ApprovalStatus::Timeout,
                    approved_by: None,
                    reason: Some("Request timed out".to_string()),
                    timestamp: Utc::now(),
                    modified_action: None,
                };
                self.state.complete(WorkflowStatus::Expired, result);

                if let Some(ref callback) = self.callbacks.on_expired {
                    callback(&self.state);
                }

                Some(WorkflowStatus::Expired)
            }
            TimeoutAction::Approve => {
                let result = ApprovalResult {
                    request_id: self.state.request_id.clone(),
                    status: ApprovalStatus::Approved,
                    approved_by: Some("system".to_string()),
                    reason: Some("Auto-approved after timeout".to_string()),
                    timestamp: Utc::now(),
                    modified_action: None,
                };
                self.state.complete(WorkflowStatus::Approved, result);

                if let Some(ref callback) = self.callbacks.on_approved {
                    callback(&self.state);
                }

                Some(WorkflowStatus::Approved)
            }
            TimeoutAction::Escalate => {
                self.escalate();
                Some(WorkflowStatus::Escalated)
            }
            TimeoutAction::Extend => {
                // Extend timeout
                self.state.expires_at =
                    Utc::now() + chrono::Duration::seconds(self.config.timeout_secs as i64);
                None
            }
        }
    }

    /// Escalate the workflow
    pub fn escalate(&mut self) -> bool {
        if let Some(ref escalation) = self.config.escalation
            && self.state.escalation_level < escalation.max_levels
        {
            self.state.escalation_level += 1;
            self.state.status = WorkflowStatus::Escalated;

            // Extend timeout
            self.state.expires_at =
                Utc::now() + chrono::Duration::seconds(escalation.escalate_after_secs as i64);

            self.state.add_comment(WorkflowComment {
                author: "system".to_string(),
                timestamp: Utc::now(),
                content: format!(
                    "Escalated to level {} ({})",
                    self.state.escalation_level,
                    escalation.escalate_to.join(", ")
                ),
                is_system: true,
            });

            if let Some(ref callback) = self.callbacks.on_escalated {
                callback(&self.state, self.state.escalation_level);
            }

            return true;
        }
        false
    }

    /// Get progress summary
    pub fn progress_summary(&self) -> WorkflowProgress {
        WorkflowProgress {
            workflow_id: self.state.id.clone(),
            status: self.state.status,
            approvals_received: self.state.approvals.len(),
            approvals_required: self.config.required_approvals,
            rejections_received: self.state.rejections.len(),
            time_remaining_secs: if self.state.is_expired() {
                0
            } else {
                (self.state.expires_at - Utc::now()).num_seconds().max(0)
            },
            escalation_level: self.state.escalation_level,
        }
    }
}

/// Progress summary for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Workflow ID
    pub workflow_id: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Number of approvals received
    pub approvals_received: usize,
    /// Number of approvals required
    pub approvals_required: usize,
    /// Number of rejections received
    pub rejections_received: usize,
    /// Time remaining in seconds
    pub time_remaining_secs: i64,
    /// Current escalation level
    pub escalation_level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hitl::policy::{ActionType, RiskLevel};

    fn create_test_request() -> ApprovalRequest {
        ApprovalRequest::new("Test action", ActionType::FileDelete, RiskLevel::High)
    }

    #[test]
    fn test_workflow_creation() {
        let request = create_test_request();
        let config = WorkflowConfig::default();
        let workflow = ApprovalWorkflow::new(&request, config, vec![]);

        assert_eq!(workflow.state().status, WorkflowStatus::Pending);
        assert!(!workflow.state().is_expired());
    }

    #[test]
    fn test_workflow_approval() {
        let request = create_test_request();
        let config = WorkflowConfig {
            required_approvals: 1,
            ..Default::default()
        };
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        let status = workflow
            .approve("admin", Some("Looks good".to_string()))
            .unwrap();
        assert_eq!(status, WorkflowStatus::Approved);
        assert!(workflow.state().is_completed());
    }

    #[test]
    fn test_workflow_multiple_approvals() {
        let request = create_test_request();
        let config = WorkflowConfig {
            required_approvals: 2,
            ..Default::default()
        };
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        // First approval
        let status = workflow.approve("admin1", None).unwrap();
        assert_eq!(status, WorkflowStatus::PartiallyApproved);
        assert!(!workflow.state().is_completed());

        // Second approval
        let status = workflow.approve("admin2", None).unwrap();
        assert_eq!(status, WorkflowStatus::Approved);
        assert!(workflow.state().is_completed());
    }

    #[test]
    fn test_workflow_rejection() {
        let request = create_test_request();
        let config = WorkflowConfig::default();
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        let status = workflow.reject("admin", "Too risky".to_string());
        assert_eq!(status, WorkflowStatus::Rejected);
        assert!(workflow.state().is_completed());

        let result = workflow.state().result.as_ref().unwrap();
        assert_eq!(result.status, ApprovalStatus::Rejected);
    }

    #[test]
    fn test_workflow_comments() {
        let request = create_test_request();
        let config = WorkflowConfig::default();
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        workflow.comment("reviewer", "Need more context");

        assert_eq!(workflow.state().comments.len(), 1);
        assert_eq!(workflow.state().comments[0].content, "Need more context");
    }

    #[test]
    fn test_workflow_progress() {
        let request = create_test_request();
        let config = WorkflowConfig {
            required_approvals: 3,
            timeout_secs: 600,
            ..Default::default()
        };
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        workflow.approve("admin1", None).unwrap();

        let progress = workflow.progress_summary();
        assert_eq!(progress.approvals_received, 1);
        assert_eq!(progress.approvals_required, 3);
        assert!(progress.time_remaining_secs > 0);
    }

    #[test]
    fn test_workflow_escalation() {
        let request = create_test_request();
        let config = WorkflowConfig {
            escalation: Some(EscalationConfig {
                escalate_after_secs: 300,
                escalate_to: vec!["manager".to_string()],
                max_levels: 2,
            }),
            ..Default::default()
        };
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        assert!(workflow.escalate());
        assert_eq!(workflow.state().escalation_level, 1);
        assert_eq!(workflow.state().status, WorkflowStatus::Escalated);

        assert!(workflow.escalate());
        assert_eq!(workflow.state().escalation_level, 2);

        // Max level reached
        assert!(!workflow.escalate());
        assert_eq!(workflow.state().escalation_level, 2);
    }

    #[test]
    fn test_timeout_action_reject() {
        let request = create_test_request();
        let config = WorkflowConfig {
            timeout_secs: 0, // Immediate timeout
            timeout_action: TimeoutAction::Reject,
            ..Default::default()
        };
        let mut workflow = ApprovalWorkflow::new(&request, config, vec![]);

        // Force expiration
        workflow.state.expires_at = Utc::now() - chrono::Duration::seconds(1);

        let result = workflow.check_expiration();
        assert_eq!(result, Some(WorkflowStatus::Expired));
    }

    #[test]
    fn test_policy_restrictions() {
        let request = create_test_request();
        let config = WorkflowConfig::default();
        let policy = ApprovalPolicy::new("restricted").with_allowed_approver("admin");

        let mut workflow = ApprovalWorkflow::new(&request, config, vec![policy]);

        // Unauthorized approver
        let result = workflow.approve("developer", None);
        assert!(result.is_err());

        // Authorized approver
        let result = workflow.approve("admin", None);
        assert!(result.is_ok());
    }
}
