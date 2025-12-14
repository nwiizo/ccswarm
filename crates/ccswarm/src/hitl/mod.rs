//! Human-in-the-Loop (HITL) Module
//!
//! Provides approval workflows for dangerous operations, enabling human oversight
//! before critical actions are executed by agents.

mod approval;
mod policy;
mod workflow;

pub use approval::{
    ApprovalChannel, ApprovalRequest, ApprovalResult, ApprovalStatus, PendingApproval,
};
pub use policy::{ActionType, ApprovalPolicy, PolicyRule, RiskLevel};
pub use workflow::{ApprovalWorkflow, WorkflowConfig, WorkflowState};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// HITL system for managing approval workflows
pub struct HitlSystem {
    /// Pending approvals
    pending: Arc<RwLock<HashMap<String, PendingApproval>>>,
    /// Completed approvals history
    history: Arc<RwLock<Vec<CompletedApproval>>>,
    /// Approval policies
    policies: Arc<RwLock<Vec<ApprovalPolicy>>>,
    /// Active workflows (reserved for future workflow-based approval)
    #[allow(dead_code)]
    workflows: Arc<RwLock<HashMap<String, ApprovalWorkflow>>>,
    /// Configuration
    config: HitlConfig,
    /// Channel for approval requests
    request_tx: mpsc::Sender<ApprovalRequest>,
    /// Channel for receiving approval requests (reserved for async processing)
    #[allow(dead_code)]
    request_rx: Arc<RwLock<mpsc::Receiver<ApprovalRequest>>>,
}

/// Configuration for the HITL system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlConfig {
    /// Whether HITL is enabled
    pub enabled: bool,
    /// Default timeout for approval requests in seconds
    pub default_timeout_secs: u64,
    /// Maximum pending approvals
    pub max_pending: usize,
    /// Auto-approve low-risk operations
    pub auto_approve_low_risk: bool,
    /// Approval channels to use
    pub channels: Vec<ApprovalChannel>,
    /// Whether to require reason for approvals
    pub require_reason: bool,
    /// History retention in hours
    pub history_retention_hours: u64,
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_timeout_secs: 300, // 5 minutes
            max_pending: 100,
            auto_approve_low_risk: false,
            channels: vec![ApprovalChannel::Cli],
            require_reason: false,
            history_retention_hours: 24,
        }
    }
}

/// A completed approval with full history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedApproval {
    /// Original request
    pub request: ApprovalRequest,
    /// Result of the approval
    pub result: ApprovalResult,
    /// Who approved/rejected
    pub approved_by: Option<String>,
    /// Reason for the decision
    pub reason: Option<String>,
    /// When the decision was made
    pub decided_at: DateTime<Utc>,
}

impl HitlSystem {
    /// Create a new HITL system
    pub fn new(config: HitlConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            policies: Arc::new(RwLock::new(Vec::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            config,
            request_tx: tx,
            request_rx: Arc::new(RwLock::new(rx)),
        }
    }

    /// Check if HITL is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Add an approval policy
    pub async fn add_policy(&self, policy: ApprovalPolicy) {
        let mut policies = self.policies.write().await;
        policies.push(policy);
    }

    /// Remove a policy by name
    pub async fn remove_policy(&self, name: &str) -> bool {
        let mut policies = self.policies.write().await;
        let initial_len = policies.len();
        policies.retain(|p| p.name != name);
        policies.len() < initial_len
    }

    /// Get all policies
    pub async fn get_policies(&self) -> Vec<ApprovalPolicy> {
        let policies = self.policies.read().await;
        policies.clone()
    }

    /// Request approval for an action
    pub async fn request_approval(&self, request: ApprovalRequest) -> Result<String, String> {
        if !self.config.enabled {
            return Err("HITL system is disabled".to_string());
        }

        // Check if approval is required based on policies
        let requires_approval = self.check_policies(&request).await;

        if !requires_approval
            && self.config.auto_approve_low_risk
            && request.risk_level == RiskLevel::Low
        {
            // Auto-approve low-risk operations
            return Ok(request.id.clone());
        }

        // Check max pending limit
        let pending = self.pending.read().await;
        if pending.len() >= self.config.max_pending {
            return Err("Maximum pending approvals reached".to_string());
        }
        drop(pending);

        // Create pending approval
        let pending_approval =
            PendingApproval::new(request.clone(), self.config.default_timeout_secs);

        // Store in pending
        let id = request.id.clone();
        let mut pending = self.pending.write().await;
        pending.insert(id.clone(), pending_approval);

        // Send to request channel
        self.request_tx
            .send(request)
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        Ok(id)
    }

    /// Check if any policy requires approval for this request
    async fn check_policies(&self, request: &ApprovalRequest) -> bool {
        let policies = self.policies.read().await;

        for policy in policies.iter() {
            if policy.matches(request) {
                return true;
            }
        }

        // Default: require approval for high-risk actions
        request.risk_level >= RiskLevel::High
    }

    /// Approve a pending request
    pub async fn approve(
        &self,
        request_id: &str,
        approved_by: Option<String>,
        reason: Option<String>,
    ) -> Result<ApprovalResult, String> {
        let mut pending = self.pending.write().await;

        let pending_approval = pending
            .remove(request_id)
            .ok_or_else(|| format!("Pending approval not found: {}", request_id))?;

        let result = ApprovalResult {
            request_id: request_id.to_string(),
            status: ApprovalStatus::Approved,
            approved_by: approved_by.clone(),
            reason: reason.clone(),
            timestamp: Utc::now(),
            modified_action: None,
        };

        // Add to history
        let completed = CompletedApproval {
            request: pending_approval.request,
            result: result.clone(),
            approved_by,
            reason,
            decided_at: Utc::now(),
        };

        let mut history = self.history.write().await;
        history.push(completed);

        Ok(result)
    }

    /// Reject a pending request
    pub async fn reject(
        &self,
        request_id: &str,
        rejected_by: Option<String>,
        reason: String,
    ) -> Result<ApprovalResult, String> {
        let mut pending = self.pending.write().await;

        let pending_approval = pending
            .remove(request_id)
            .ok_or_else(|| format!("Pending approval not found: {}", request_id))?;

        let result = ApprovalResult {
            request_id: request_id.to_string(),
            status: ApprovalStatus::Rejected,
            approved_by: rejected_by.clone(),
            reason: Some(reason.clone()),
            timestamp: Utc::now(),
            modified_action: None,
        };

        // Add to history
        let completed = CompletedApproval {
            request: pending_approval.request,
            result: result.clone(),
            approved_by: rejected_by,
            reason: Some(reason),
            decided_at: Utc::now(),
        };

        let mut history = self.history.write().await;
        history.push(completed);

        Ok(result)
    }

    /// Approve with modifications
    pub async fn approve_with_modifications(
        &self,
        request_id: &str,
        approved_by: Option<String>,
        modified_action: serde_json::Value,
        reason: Option<String>,
    ) -> Result<ApprovalResult, String> {
        let mut pending = self.pending.write().await;

        let pending_approval = pending
            .remove(request_id)
            .ok_or_else(|| format!("Pending approval not found: {}", request_id))?;

        let result = ApprovalResult {
            request_id: request_id.to_string(),
            status: ApprovalStatus::ApprovedWithModifications,
            approved_by: approved_by.clone(),
            reason: reason.clone(),
            timestamp: Utc::now(),
            modified_action: Some(modified_action),
        };

        // Add to history
        let completed = CompletedApproval {
            request: pending_approval.request,
            result: result.clone(),
            approved_by,
            reason,
            decided_at: Utc::now(),
        };

        let mut history = self.history.write().await;
        history.push(completed);

        Ok(result)
    }

    /// Get pending approval by ID
    pub async fn get_pending(&self, request_id: &str) -> Option<PendingApproval> {
        let pending = self.pending.read().await;
        pending.get(request_id).cloned()
    }

    /// Get all pending approvals
    pub async fn get_all_pending(&self) -> Vec<PendingApproval> {
        let pending = self.pending.read().await;
        pending.values().cloned().collect()
    }

    /// Get pending approvals for an agent
    pub async fn get_pending_by_agent(&self, agent_id: &str) -> Vec<PendingApproval> {
        let pending = self.pending.read().await;
        pending
            .values()
            .filter(|p| p.request.agent_id.as_deref() == Some(agent_id))
            .cloned()
            .collect()
    }

    /// Get approval history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<CompletedApproval> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Check for timed out approvals
    pub async fn check_timeouts(&self) -> Vec<String> {
        let mut pending = self.pending.write().await;
        let now = Utc::now();

        let timed_out: Vec<String> = pending
            .iter()
            .filter(|(_, p)| p.expires_at < now)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &timed_out {
            if let Some(approval) = pending.remove(id) {
                // Record as timed out
                let completed = CompletedApproval {
                    request: approval.request,
                    result: ApprovalResult {
                        request_id: id.clone(),
                        status: ApprovalStatus::Timeout,
                        approved_by: None,
                        reason: Some("Request timed out".to_string()),
                        timestamp: now,
                        modified_action: None,
                    },
                    approved_by: None,
                    reason: Some("Request timed out".to_string()),
                    decided_at: now,
                };

                let mut history = self.history.write().await;
                history.push(completed);
            }
        }

        timed_out
    }

    /// Wait for approval decision
    pub async fn wait_for_decision(&self, request_id: &str) -> Result<ApprovalResult, String> {
        let timeout = std::time::Duration::from_secs(self.config.default_timeout_secs);
        let start = std::time::Instant::now();

        loop {
            // Check if approved or rejected
            let pending = self.pending.read().await;
            if !pending.contains_key(request_id) {
                // Check history for result
                let history = self.history.read().await;
                if let Some(completed) = history.iter().find(|c| c.result.request_id == request_id)
                {
                    return Ok(completed.result.clone());
                }
                return Err("Request not found".to_string());
            }
            drop(pending);

            // Check timeout
            if start.elapsed() >= timeout {
                self.check_timeouts().await;
                return Err("Approval request timed out".to_string());
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Clean up old history entries
    pub async fn cleanup_history(&self) {
        let cutoff =
            Utc::now() - chrono::Duration::hours(self.config.history_retention_hours as i64);

        let mut history = self.history.write().await;
        history.retain(|c| c.decided_at > cutoff);
    }

    /// Get statistics
    pub async fn get_stats(&self) -> HitlStats {
        let pending = self.pending.read().await;
        let history = self.history.read().await;

        let approved = history
            .iter()
            .filter(|c| c.result.status == ApprovalStatus::Approved)
            .count();
        let rejected = history
            .iter()
            .filter(|c| c.result.status == ApprovalStatus::Rejected)
            .count();
        let timed_out = history
            .iter()
            .filter(|c| c.result.status == ApprovalStatus::Timeout)
            .count();

        HitlStats {
            pending_count: pending.len(),
            history_count: history.len(),
            approved_count: approved,
            rejected_count: rejected,
            timeout_count: timed_out,
            policy_count: 0, // Would need to count policies
        }
    }
}

impl Default for HitlSystem {
    fn default() -> Self {
        Self::new(HitlConfig::default())
    }
}

/// Statistics for the HITL system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlStats {
    /// Number of pending approvals
    pub pending_count: usize,
    /// Number of historical approvals
    pub history_count: usize,
    /// Number of approved requests
    pub approved_count: usize,
    /// Number of rejected requests
    pub rejected_count: usize,
    /// Number of timed out requests
    pub timeout_count: usize,
    /// Number of active policies
    pub policy_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hitl_system_creation() {
        let system = HitlSystem::new(HitlConfig::default());
        assert!(system.is_enabled());

        let pending = system.get_all_pending().await;
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_request_approval() {
        let system = HitlSystem::new(HitlConfig::default());

        let request = ApprovalRequest::new(
            "Delete all files",
            ActionType::FileDelete,
            RiskLevel::Critical,
        );

        let id = system.request_approval(request).await.unwrap();
        assert!(!id.is_empty());

        let pending = system.get_all_pending().await;
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_approve_request() {
        let system = HitlSystem::new(HitlConfig::default());

        let request = ApprovalRequest::new("Delete file", ActionType::FileDelete, RiskLevel::High);

        let id = system.request_approval(request).await.unwrap();

        let result = system
            .approve(&id, Some("admin".to_string()), Some("Approved".to_string()))
            .await
            .unwrap();

        assert_eq!(result.status, ApprovalStatus::Approved);
        assert!(system.get_pending(&id).await.is_none());

        let history = system.get_history(None).await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_reject_request() {
        let system = HitlSystem::new(HitlConfig::default());

        let request = ApprovalRequest::new(
            "Drop database",
            ActionType::DatabaseModify,
            RiskLevel::Critical,
        );

        let id = system.request_approval(request).await.unwrap();

        let result = system
            .reject(&id, Some("admin".to_string()), "Too dangerous".to_string())
            .await
            .unwrap();

        assert_eq!(result.status, ApprovalStatus::Rejected);
        assert_eq!(result.reason, Some("Too dangerous".to_string()));
    }

    #[tokio::test]
    async fn test_auto_approve_low_risk() {
        let config = HitlConfig {
            auto_approve_low_risk: true,
            ..Default::default()
        };
        let system = HitlSystem::new(config);

        let request = ApprovalRequest::new("Read file", ActionType::FileRead, RiskLevel::Low);

        // Low-risk should be auto-approved
        let id = system.request_approval(request).await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_policy_matching() {
        let system = HitlSystem::new(HitlConfig::default());

        // Add a policy
        let policy =
            ApprovalPolicy::new("block_production").with_rule(PolicyRule::RequireApproval {
                action_types: vec![ActionType::Deploy],
                environments: Some(vec!["production".to_string()]),
            });

        system.add_policy(policy).await;

        let policies = system.get_policies().await;
        assert_eq!(policies.len(), 1);
    }

    #[tokio::test]
    async fn test_timeout() {
        let config = HitlConfig {
            default_timeout_secs: 1,
            ..Default::default()
        };
        let system = HitlSystem::new(config);

        let request = ApprovalRequest::new(
            "Critical action",
            ActionType::SystemCommand,
            RiskLevel::Critical,
        );

        let id = system.request_approval(request).await.unwrap();

        // Wait for timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let timed_out = system.check_timeouts().await;
        assert!(timed_out.contains(&id));

        let history = system.get_history(None).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].result.status, ApprovalStatus::Timeout);
    }

    #[tokio::test]
    async fn test_stats() {
        let system = HitlSystem::new(HitlConfig::default());

        // Create and approve some requests
        let request1 = ApprovalRequest::new("Action 1", ActionType::FileWrite, RiskLevel::Medium);
        let id1 = system.request_approval(request1).await.unwrap();
        system.approve(&id1, None, None).await.unwrap();

        let request2 = ApprovalRequest::new("Action 2", ActionType::FileDelete, RiskLevel::High);
        let id2 = system.request_approval(request2).await.unwrap();
        system
            .reject(&id2, None, "Rejected".to_string())
            .await
            .unwrap();

        let stats = system.get_stats().await;
        assert_eq!(stats.pending_count, 0);
        assert_eq!(stats.history_count, 2);
        assert_eq!(stats.approved_count, 1);
        assert_eq!(stats.rejected_count, 1);
    }
}
