//! Plan approval workflow
//!
//! Read-only plan mode with lead approval/rejection feedback loop.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Draft,
    PendingApproval,
    Approved,
    Rejected,
    Revised,
}

/// A plan submitted for approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<PlanStep>,
    pub status: PlanStatus,
    pub submitted_by: String,
    pub reviewed_by: Option<String>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub revision: u32,
}

/// A step in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub order: u32,
    pub description: String,
    pub agent: Option<String>,
    pub estimated_duration: Option<String>,
    pub files_affected: Vec<String>,
}

impl Plan {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        submitted_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: description.into(),
            steps: Vec::new(),
            status: PlanStatus::Draft,
            submitted_by: submitted_by.into(),
            reviewed_by: None,
            feedback: None,
            created_at: Utc::now(),
            reviewed_at: None,
            revision: 1,
        }
    }

    pub fn add_step(&mut self, description: impl Into<String>) -> &mut Self {
        let order = self.steps.len() as u32 + 1;
        self.steps.push(PlanStep {
            order,
            description: description.into(),
            agent: None,
            estimated_duration: None,
            files_affected: Vec::new(),
        });
        self
    }

    pub fn submit_for_approval(&mut self) {
        self.status = PlanStatus::PendingApproval;
    }

    pub fn approve(&mut self, reviewer: impl Into<String>) {
        self.status = PlanStatus::Approved;
        self.reviewed_by = Some(reviewer.into());
        self.reviewed_at = Some(Utc::now());
    }

    pub fn reject(&mut self, reviewer: impl Into<String>, feedback: impl Into<String>) {
        self.status = PlanStatus::Rejected;
        self.reviewed_by = Some(reviewer.into());
        self.feedback = Some(feedback.into());
        self.reviewed_at = Some(Utc::now());
    }

    pub fn revise(&mut self) {
        self.status = PlanStatus::Draft;
        self.revision += 1;
        self.reviewed_by = None;
        self.feedback = None;
        self.reviewed_at = None;
    }
}

/// Plan approval manager
pub struct PlanApprovalManager {
    plans: std::collections::HashMap<String, Plan>,
}

impl PlanApprovalManager {
    pub fn new() -> Self {
        Self {
            plans: std::collections::HashMap::new(),
        }
    }

    pub fn submit(&mut self, mut plan: Plan) -> String {
        plan.submit_for_approval();
        let id = plan.id.clone();
        self.plans.insert(id.clone(), plan);
        id
    }

    pub fn approve(&mut self, plan_id: &str, reviewer: impl Into<String>) -> Result<(), String> {
        let plan = self
            .plans
            .get_mut(plan_id)
            .ok_or_else(|| format!("Plan {} not found", plan_id))?;
        if plan.status != PlanStatus::PendingApproval {
            return Err("Plan is not pending approval".to_string());
        }
        plan.approve(reviewer);
        Ok(())
    }

    pub fn reject(
        &mut self,
        plan_id: &str,
        reviewer: impl Into<String>,
        feedback: impl Into<String>,
    ) -> Result<(), String> {
        let plan = self
            .plans
            .get_mut(plan_id)
            .ok_or_else(|| format!("Plan {} not found", plan_id))?;
        if plan.status != PlanStatus::PendingApproval {
            return Err("Plan is not pending approval".to_string());
        }
        plan.reject(reviewer, feedback);
        Ok(())
    }

    pub fn get(&self, plan_id: &str) -> Option<&Plan> {
        self.plans.get(plan_id)
    }

    pub fn pending(&self) -> Vec<&Plan> {
        self.plans
            .values()
            .filter(|p| p.status == PlanStatus::PendingApproval)
            .collect()
    }
}

impl Default for PlanApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_approval_flow() {
        let mut manager = PlanApprovalManager::new();

        let mut plan = Plan::new("Auth System", "Implement auth", "backend-agent");
        plan.add_step("Create user model");
        plan.add_step("Add JWT middleware");

        let id = manager.submit(plan);
        assert_eq!(
            manager.get(&id).unwrap().status,
            PlanStatus::PendingApproval
        );

        assert!(manager.approve(&id, "lead").is_ok());
        assert_eq!(manager.get(&id).unwrap().status, PlanStatus::Approved);
    }

    #[test]
    fn test_plan_rejection_and_revision() {
        let mut manager = PlanApprovalManager::new();

        let plan = Plan::new("Feature", "Desc", "agent");
        let id = manager.submit(plan);

        assert!(manager.reject(&id, "lead", "Needs more detail").is_ok());
        assert_eq!(manager.get(&id).unwrap().status, PlanStatus::Rejected);
    }
}
