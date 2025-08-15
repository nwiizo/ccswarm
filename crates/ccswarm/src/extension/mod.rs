//! Self-Extension functionality for agents and system
//!
//! This module implements the ability for agents to extend their own capabilities
//! and for the system itself to evolve through collective decision-making.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// pub mod agent_extension; // Module not found
// pub mod autonomous_agent_extension; // Module not found
pub mod meta_learning;
// pub mod propagation; // Module not found
// pub mod system_extension; // Module not found

use crate::identity::AgentRole;
// Remove imports - these types don't exist in extension_stub
// use crate::extension_stub::sangha::{Proposal, ProposalType, Sangha};

/// Types of extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionType {
    /// Capability extension (new skills, tools, patterns)
    Capability,
    /// Cognitive extension (reasoning, memory, learning)
    Cognitive,
    /// Collaborative extension (protocols, synchronization)
    Collaborative,
    /// System-level extension (architecture, infrastructure)
    System,
}

/// Represents an extension proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionProposal {
    pub id: Uuid,
    pub proposer: String,
    pub extension_type: ExtensionType,
    pub title: String,
    pub description: String,
    pub current_state: CurrentState,
    pub proposed_state: ProposedState,
    pub implementation_plan: ImplementationPlan,
    pub risk_assessment: RiskAssessment,
    pub success_criteria: Vec<SuccessCriterion>,
    pub created_at: DateTime<Utc>,
    pub status: ExtensionStatus,
}

/// Current state before extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentState {
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub performance_metrics: HashMap<String, f64>,
}

/// Proposed state after extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedState {
    pub new_capabilities: Vec<String>,
    pub expected_improvements: Vec<String>,
    pub performance_targets: HashMap<String, f64>,
}

/// Implementation plan for the extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub phases: Vec<ImplementationPhase>,
    pub timeline: String,
    pub resources_required: Vec<String>,
    pub dependencies: Vec<String>,
}

/// A phase in the implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    pub name: String,
    pub description: String,
    pub tasks: Vec<String>,
    pub duration_estimate: String,
    pub validation_method: String,
}

/// Risk assessment for the extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risks: Vec<Risk>,
    pub mitigation_strategies: Vec<String>,
    pub rollback_plan: String,
    pub overall_risk_score: f64,
}

/// Individual risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub category: RiskCategory,
}

/// Categories of risks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Technical,
    Performance,
    Compatibility,
    Security,
    Operational,
}

/// Success criteria for the extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub description: String,
    pub metric: String,
    pub target_value: String,
    pub measurement_method: String,
}

/// Status of an extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionStatus {
    /// Initial proposal stage
    Proposed,
    /// Under review by Sangha
    UnderReview,
    /// Approved for implementation
    Approved,
    /// Currently being implemented
    Implementing,
    /// In testing phase
    Testing,
    /// Successfully deployed
    Deployed,
    /// Failed and rolled back
    Failed,
    /// Rejected by Sangha
    Rejected,
}

pub struct ExtensionManager {
    /// Active extensions
    extensions: Arc<RwLock<HashMap<Uuid, Extension>>>,
    /// Extension history
    history: Arc<RwLock<Vec<ExtensionRecord>>>,
    /// Extension templates
    templates: Arc<RwLock<HashMap<String, ExtensionTemplate>>>,
}

/// An active extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: Uuid,
    pub proposal: ExtensionProposal,
    pub implementation_status: ImplementationStatus,
    pub test_results: Vec<TestResult>,
    pub deployment_info: Option<DeploymentInfo>,
    pub metrics: ExtensionMetrics,
}

/// Implementation status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationStatus {
    pub current_phase: usize,
    pub phase_progress: f64,
    pub blockers: Vec<String>,
    pub completed_tasks: Vec<String>,
    pub pending_tasks: Vec<String>,
}

/// Test results for the extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployed_at: DateTime<Utc>,
    pub deployed_by: String,
    pub version: String,
    pub configuration: HashMap<String, String>,
}

/// Metrics for tracking extension performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetrics {
    pub adoption_rate: f64,
    pub performance_improvement: HashMap<String, f64>,
    pub error_rate: f64,
    pub user_satisfaction: f64,
}

/// Historical record of an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionRecord {
    pub extension_id: Uuid,
    pub agent_id: String,
    pub extension_type: ExtensionType,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: ExtensionOutcome,
    pub lessons_learned: Vec<String>,
}

/// Outcome of an extension attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionOutcome {
    Success {
        improvements: Vec<String>,
    },
    Failure {
        reasons: Vec<String>,
    },
    PartialSuccess {
        achievements: Vec<String>,
        issues: Vec<String>,
    },
}

/// Template for common extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionTemplate {
    pub name: String,
    pub description: String,
    pub extension_type: ExtensionType,
    pub applicable_roles: Vec<AgentRole>,
    pub prerequisites: Vec<String>,
    pub typical_timeline: String,
    pub success_rate: f64,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        let manager = Self {
            extensions: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            // sangha, // Temporarily removed
            templates: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize with common templates
        tokio::spawn({
            let templates = manager.templates.clone();
            async move {
                let _ = Self::initialize_templates(templates).await;
            }
        });

        manager
    }

    /// Initialize common extension templates
    async fn initialize_templates(
        templates: Arc<RwLock<HashMap<String, ExtensionTemplate>>>,
    ) -> Result<()> {
        let mut templates_map = templates.write().await;

        // React Server Components template
        templates_map.insert(
            "react-server-components".to_string(),
            ExtensionTemplate {
                name: "React Server Components".to_string(),
                description: "Add RSC support to frontend agents".to_string(),
                extension_type: ExtensionType::Capability,
                applicable_roles: vec![AgentRole::Frontend {
                    technologies: vec!["React".to_string()],
                    responsibilities: vec![],
                    boundaries: vec![],
                }],
                prerequisites: vec!["React".to_string(), "Node.js 18+".to_string()],
                typical_timeline: "2 weeks".to_string(),
                success_rate: 0.85,
            },
        );

        // Distributed Processing template
        templates_map.insert(
            "distributed-processing".to_string(),
            ExtensionTemplate {
                name: "Distributed Processing".to_string(),
                description: "Enable distributed task processing".to_string(),
                extension_type: ExtensionType::System,
                applicable_roles: vec![],
                prerequisites: vec!["Kubernetes".to_string(), "Message Queue".to_string()],
                typical_timeline: "1 month".to_string(),
                success_rate: 0.75,
            },
        );

        Ok(())
    }

    /// Propose a new extension
    pub async fn propose_extension(&self, proposal: ExtensionProposal) -> Result<Uuid> {
        // Validate the proposal
        self.validate_proposal(&proposal)?;

        // Create a Sangha proposal
        self.create_sangha_proposal(&proposal).await?;

        // Submit to Sangha for review
        // self.sangha.submit_proposal(sangha_proposal).await?; // Temporarily disabled

        // Store the extension
        let extension = Extension {
            id: proposal.id,
            proposal: proposal.clone(),
            implementation_status: ImplementationStatus {
                current_phase: 0,
                phase_progress: 0.0,
                blockers: vec![],
                completed_tasks: vec![],
                pending_tasks: vec![],
            },
            test_results: vec![],
            deployment_info: None,
            metrics: ExtensionMetrics {
                adoption_rate: 0.0,
                performance_improvement: HashMap::new(),
                error_rate: 0.0,
                user_satisfaction: 0.0,
            },
        };

        let mut extensions = self.extensions.write().await;
        extensions.insert(extension.id, extension);

        Ok(proposal.id)
    }

    /// Validate an extension proposal
    fn validate_proposal(&self, proposal: &ExtensionProposal) -> Result<()> {
        // Check risk score
        if proposal.risk_assessment.overall_risk_score > 0.8 {
            anyhow::bail!(
                "Risk score too high: {}",
                proposal.risk_assessment.overall_risk_score
            );
        }

        // Check implementation plan
        if proposal.implementation_plan.phases.is_empty() {
            anyhow::bail!("Implementation plan must have at least one phase");
        }

        // Check success criteria
        if proposal.success_criteria.is_empty() {
            anyhow::bail!("Must define at least one success criterion");
        }

        Ok(())
    }

    /// Create a Sangha proposal from an extension proposal
    async fn create_sangha_proposal(&self, _proposal: &ExtensionProposal) -> Result<()> {
        // Temporarily disabled - ProposalBuilder doesn't exist in extension_stub
        // let proposal_type = match proposal.extension_type {
        //     ExtensionType::System => ProposalType::SystemExtension,
        //     _ => ProposalType::AgentExtension,
        // };

        // let sangha_proposal = crate::extension_stub::sangha::proposal::ProposalBuilder::new(
        //     proposal.title.clone(),
        //     proposal.proposer.clone(),
        //     proposal_type,
        // )
        // .description(proposal.description.clone())
        // .data(serde_json::to_value(proposal)?)
        // .build();

        // Ok(sangha_proposal)
        Ok(())
    }

    /// Start implementing an approved extension
    pub async fn start_implementation(&self, extension_id: Uuid) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&extension_id)
            .context("Extension not found")?;

        if extension.proposal.status != ExtensionStatus::Approved {
            anyhow::bail!("Extension must be approved before implementation");
        }

        extension.proposal.status = ExtensionStatus::Implementing;

        Ok(())
    }

    /// Update implementation progress
    pub async fn update_progress(
        &self,
        extension_id: Uuid,
        phase_progress: f64,
        completed_tasks: Vec<String>,
    ) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&extension_id)
            .context("Extension not found")?;

        extension.implementation_status.phase_progress = phase_progress;
        extension
            .implementation_status
            .completed_tasks
            .extend(completed_tasks);

        // Check if phase is complete
        if phase_progress >= 1.0 {
            extension.implementation_status.current_phase += 1;
            extension.implementation_status.phase_progress = 0.0;
        }

        Ok(())
    }

    /// Record test results
    pub async fn record_test_result(
        &self,
        extension_id: Uuid,
        test_result: TestResult,
    ) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&extension_id)
            .context("Extension not found")?;

        extension.test_results.push(test_result);

        Ok(())
    }

    /// Deploy an extension
    pub async fn deploy_extension(
        &self,
        extension_id: Uuid,
        deployed_by: String,
        configuration: HashMap<String, String>,
    ) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&extension_id)
            .context("Extension not found")?;

        // Verify all tests passed
        let all_tests_passed = extension.test_results.iter().all(|t| t.passed);
        if !all_tests_passed {
            anyhow::bail!("Cannot deploy extension with failing tests");
        }

        extension.deployment_info = Some(DeploymentInfo {
            deployed_at: Utc::now(),
            deployed_by,
            version: "1.0.0".to_string(),
            configuration,
        });

        extension.proposal.status = ExtensionStatus::Deployed;

        Ok(())
    }

    /// Get extension history for an agent
    pub async fn get_agent_history(&self, agent_id: &str) -> Vec<ExtensionRecord> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|record| record.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get successful extension patterns
    pub async fn get_successful_patterns(&self) -> Vec<ExtensionPattern> {
        let history = self.history.read().await;

        // Analyze history for successful patterns
        let mut patterns = Vec::new();

        // Group by extension type and analyze success factors
        // This is a simplified implementation
        for record in history.iter() {
            if let ExtensionOutcome::Success { improvements } = &record.outcome {
                patterns.push(ExtensionPattern {
                    extension_type: record.extension_type,
                    success_factors: improvements.clone(),
                    typical_duration: "2 weeks".to_string(), // Would calculate from data
                    prerequisites: vec![],
                });
            }
        }

        patterns
    }

    /// Get extension manager statistics
    pub async fn get_stats(&self) -> ExtensionStats {
        let extensions = self.extensions.read().await;
        let history = self.history.read().await;

        let total_extensions = extensions.len();
        let active_extensions = extensions
            .values()
            .filter(|e| {
                matches!(
                    e.proposal.status,
                    ExtensionStatus::Implementing | ExtensionStatus::Testing
                )
            })
            .count();
        let pending_proposals = extensions
            .values()
            .filter(|e| matches!(e.proposal.status, ExtensionStatus::Proposed))
            .count();
        let successful_extensions = history
            .iter()
            .filter(|r| matches!(r.outcome, ExtensionOutcome::Success { .. }))
            .count();
        let failed_extensions = history
            .iter()
            .filter(|r| matches!(r.outcome, ExtensionOutcome::Failure { .. }))
            .count();

        ExtensionStats {
            total_extensions,
            active_extensions,
            pending_proposals,
            successful_extensions,
            failed_extensions,
        }
    }
}

/// Extension manager statistics
#[derive(Debug, Clone)]
pub struct ExtensionStats {
    pub total_extensions: usize,
    pub active_extensions: usize,
    pub pending_proposals: usize,
    pub successful_extensions: usize,
    pub failed_extensions: usize,
}

/// Pattern extracted from successful extensions
#[derive(Debug, Clone)]
pub struct ExtensionPattern {
    pub extension_type: ExtensionType,
    pub success_factors: Vec<String>,
    pub typical_duration: String,
    pub prerequisites: Vec<String>,
}

/// Auto-suggestion engine for extensions
pub struct ExtensionSuggestionEngine {
    #[allow(dead_code)]
    extension_manager: Arc<ExtensionManager>,
}

impl ExtensionSuggestionEngine {
    pub fn new(extension_manager: Arc<ExtensionManager>) -> Self {
        Self { extension_manager }
    }

    /// Generate extension suggestions based on triggers
    pub async fn generate_suggestions(
        &self,
        trigger: ExtensionTrigger,
    ) -> Vec<ExtensionSuggestion> {
        match trigger {
            ExtensionTrigger::RepeatedErrors {
                error_pattern,
                frequency,
            } => self.suggest_for_errors(&error_pattern, frequency).await,
            ExtensionTrigger::PerformanceBottleneck { component, metric } => {
                self.suggest_for_performance(&component, metric).await
            }
            ExtensionTrigger::PeerCapabilityGap {
                peer_capabilities,
                own_capabilities,
            } => {
                self.suggest_for_capability_gap(&peer_capabilities, &own_capabilities)
                    .await
            }
            ExtensionTrigger::IndustryTrend {
                trend,
                relevance_score,
            } => self.suggest_for_trend(&trend, relevance_score).await,
        }
    }

    async fn suggest_for_errors(
        &self,
        _error_pattern: &str,
        _frequency: u32,
    ) -> Vec<ExtensionSuggestion> {
        // Analyze error pattern and suggest relevant extensions
        vec![]
    }

    async fn suggest_for_performance(
        &self,
        _component: &str,
        _metric: f64,
    ) -> Vec<ExtensionSuggestion> {
        vec![]
    }

    async fn suggest_for_capability_gap(
        &self,
        _peer_capabilities: &[String],
        _own_capabilities: &[String],
    ) -> Vec<ExtensionSuggestion> {
        vec![]
    }

    async fn suggest_for_trend(
        &self,
        _trend: &str,
        _relevance_score: f64,
    ) -> Vec<ExtensionSuggestion> {
        vec![]
    }
}

/// Triggers for auto-suggesting extensions
#[derive(Debug, Clone)]
pub enum ExtensionTrigger {
    RepeatedErrors {
        error_pattern: String,
        frequency: u32,
    },
    PerformanceBottleneck {
        component: String,
        metric: f64,
    },
    PeerCapabilityGap {
        peer_capabilities: Vec<String>,
        own_capabilities: Vec<String>,
    },
    IndustryTrend {
        trend: String,
        relevance_score: f64,
    },
}

/// Suggested extension
#[derive(Debug, Clone)]
pub struct ExtensionSuggestion {
    pub title: String,
    pub description: String,
    pub extension_type: ExtensionType,
    pub rationale: String,
    pub estimated_benefit: f64,
    pub implementation_complexity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extension_proposal_validation() {
        let proposal = ExtensionProposal {
            id: Uuid::new_v4(),
            proposer: "test-agent".to_string(),
            extension_type: ExtensionType::Capability,
            title: "Test Extension".to_string(),
            description: "Test description".to_string(),
            current_state: CurrentState {
                capabilities: vec!["React".to_string()],
                limitations: vec!["No SSR".to_string()],
                performance_metrics: HashMap::new(),
            },
            proposed_state: ProposedState {
                new_capabilities: vec!["React Server Components".to_string()],
                expected_improvements: vec!["Better performance".to_string()],
                performance_targets: HashMap::new(),
            },
            implementation_plan: ImplementationPlan {
                phases: vec![ImplementationPhase {
                    name: "Research".to_string(),
                    description: "Research RSC".to_string(),
                    tasks: vec!["Read docs".to_string()],
                    duration_estimate: "1 week".to_string(),
                    validation_method: "Review".to_string(),
                }],
                timeline: "2 weeks".to_string(),
                resources_required: vec![],
                dependencies: vec![],
            },
            risk_assessment: RiskAssessment {
                risks: vec![],
                mitigation_strategies: vec![],
                rollback_plan: "Revert changes".to_string(),
                overall_risk_score: 0.3,
            },
            success_criteria: vec![SuccessCriterion {
                description: "Performance improvement".to_string(),
                metric: "Page load time".to_string(),
                target_value: "< 2s".to_string(),
                measurement_method: "Lighthouse".to_string(),
            }],
            created_at: Utc::now(),
            status: ExtensionStatus::Proposed,
        };

        // Validation should pass
        let manager = ExtensionManager::new();
        assert!(manager.validate_proposal(&proposal).is_ok());
    }
}
