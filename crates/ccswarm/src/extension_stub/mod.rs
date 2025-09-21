pub mod agent_extension;
pub mod sangha;
pub mod meta_learning;

use crate::extension::{Extension, ExtensionState};
use crate::error::{CCSwarmError, Result};
use crate::traits::{Identifiable, Stateful, Validatable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stub implementation for extensions with improved error handling
///
/// This module provides a lightweight implementation of the extension system
/// for testing and development purposes. It manages extension registration,
/// validation, and basic lifecycle operations.
#[derive(Debug, Clone)]
pub struct ExtensionStub {
    extensions: HashMap<String, Extension>,
    registry_path: Option<std::path::PathBuf>,
}

impl ExtensionStub {
    /// Create a new extension stub
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            registry_path: None,
        }
    }

    /// Create extension stub with registry file path
    pub fn with_registry<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self {
            extensions: HashMap::new(),
            registry_path: Some(path.into()),
        }
    }

    /// Register a new extension
    pub fn register(&mut self, mut extension: Extension) -> Result<()> {
        // Validate extension before registration
        let validation_issues = extension.validate()?;
        if !validation_issues.is_empty() {
            // Try to auto-fix issues
            let fixes = extension.auto_fix()?;
            tracing::info!("Auto-fixed {} issues for extension {}: {:?}",
                         fixes.len(), extension.id(), fixes);

            // Re-validate after fixes
            let remaining_issues = extension.validate()?;
            if !remaining_issues.is_empty() {
                return Err(CCSwarmError::extension(
                    extension.id(),
                    format!("Extension validation failed: {:?}", remaining_issues)
                ));
            }
        }

        // Check for duplicate registration
        if self.extensions.contains_key(extension.id()) {
            return Err(CCSwarmError::extension(
                extension.id(),
                "Extension with this ID is already registered"
            ));
        }

        // Check dependencies
        let available_extensions: Vec<Extension> = self.extensions.values().cloned().collect();
        let missing_deps = extension.check_dependencies(&available_extensions)?;
        if !missing_deps.is_empty() {
            return Err(CCSwarmError::extension(
                extension.id(),
                format!("Missing dependencies: {:?}", missing_deps)
            ));
        }

        let extension_id = extension.id().to_string();
        self.extensions.insert(extension_id.clone(), extension);

        tracing::info!("Successfully registered extension: {}", extension_id);
        Ok(())
    }

    /// Unregister an extension
    pub fn unregister(&mut self, extension_id: &str) -> Result<Extension> {
        // Check if other extensions depend on this one
        let dependents: Vec<String> = self.extensions
            .values()
            .filter(|ext| {
                ext.dependencies.iter().any(|dep| dep.name == extension_id && !dep.optional)
            })
            .map(|ext| ext.id().to_string())
            .collect();

        if !dependents.is_empty() {
            return Err(CCSwarmError::extension(
                extension_id,
                format!("Cannot unregister extension: required by {:?}", dependents)
            ));
        }

        self.extensions.remove(extension_id)
            .ok_or_else(|| CCSwarmError::extension(extension_id, "Extension not found"))
    }

    /// Get an extension by ID
    pub fn get(&self, extension_id: &str) -> Option<&Extension> {
        self.extensions.get(extension_id)
    }

    /// Get a mutable reference to an extension
    pub fn get_mut(&mut self, extension_id: &str) -> Option<&mut Extension> {
        self.extensions.get_mut(extension_id)
    }

    /// List all extensions
    pub fn list(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    /// List extensions by state
    pub fn list_by_state(&self, state: &ExtensionState) -> Vec<&Extension> {
        self.extensions
            .values()
            .filter(|ext| ext.state() == state)
            .collect()
    }

    /// Get extension count
    pub fn count(&self) -> usize {
        self.extensions.len()
    }

    /// Check if an extension exists
    pub fn contains(&self, extension_id: &str) -> bool {
        self.extensions.contains_key(extension_id)
    }

    /// Load extensions from registry file
    pub async fn load_from_registry(&mut self) -> Result<usize> {
        let registry_path = self.registry_path.as_ref()
            .ok_or_else(|| CCSwarmError::config("No registry path configured"))?;

        if !registry_path.exists() {
            return Ok(0);
        }

        let content = tokio::fs::read_to_string(registry_path).await
            .map_err(|e| CCSwarmError::config(format!("Failed to read registry: {}", e)))?;

        let extensions: Vec<Extension> = serde_json::from_str(&content)
            .map_err(|e| CCSwarmError::config(format!("Failed to parse registry: {}", e)))?;

        let mut loaded_count = 0;
        for extension in extensions {
            if let Err(e) = self.register(extension) {
                tracing::warn!("Failed to load extension from registry: {}", e);
            } else {
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    /// Save extensions to registry file
    pub async fn save_to_registry(&self) -> Result<()> {
        let registry_path = self.registry_path.as_ref()
            .ok_or_else(|| CCSwarmError::config("No registry path configured"))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = registry_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| CCSwarmError::config(format!("Failed to create registry directory: {}", e)))?;
        }

        let extensions: Vec<Extension> = self.extensions.values().cloned().collect();
        let content = serde_json::to_string_pretty(&extensions)
            .map_err(|e| CCSwarmError::config(format!("Failed to serialize extensions: {}", e)))?;

        tokio::fs::write(registry_path, content).await
            .map_err(|e| CCSwarmError::config(format!("Failed to write registry: {}", e)))?;

        Ok(())
    }

    /// Validate all registered extensions
    pub fn validate_all(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();

        for extension in self.extensions.values() {
            let issues = extension.validate()?;
            if !issues.is_empty() {
                results.insert(extension.id().to_string(), issues);
            }
        }

        Ok(results)
    }

    /// Get extension statistics
    pub fn get_stats(&self) -> ExtensionStats {
        let total = self.extensions.len();
        let active = self.list_by_state(&ExtensionState::Active).len();
        let loaded = self.list_by_state(&ExtensionState::Loaded).len();
        let error_count = self.extensions.values()
            .filter(|ext| matches!(ext.state(), ExtensionState::Error(_)))
            .count();

        ExtensionStats {
            total_extensions: total,
            active_extensions: active,
            loaded_extensions: loaded,
            error_extensions: error_count,
            pending_proposals: 0, // Managed by ExtensionManager
            successful_extensions: active + loaded,
            failed_extensions: error_count,
        }
    }

    /// Find extensions by capability
    pub fn find_by_capability(&self, capability: &crate::extension::ExtensionCapability) -> Vec<&Extension> {
        self.extensions
            .values()
            .filter(|ext| ext.has_capability(capability))
            .collect()
    }

    /// Clear all extensions
    pub fn clear(&mut self) {
        self.extensions.clear();
    }
}

impl Default for ExtensionStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension manager for handling extension proposals and lifecycle
///
/// This manages the more complex aspects of extension management including
/// proposals, voting, and deployment of extensions in the system.
#[derive(Debug, Clone)]
pub struct ExtensionManager {
    pub proposals: HashMap<String, ExtensionProposal>,
    stats: ExtensionStats,
    auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionProposal {
    pub id: String,
    pub agent_id: String,
    pub extension_type: ExtensionType,
    pub status: ExtensionStatus,
    pub specification: String,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub current_state: CurrentState,
    pub proposed_state: ProposedState,
    pub implementation_plan: ImplementationPlan,
    pub risk_assessment: RiskAssessment,
    pub success_criteria: Vec<SuccessCriterion>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionType {
    Capability,
    Performance,
    Knowledge,
    Integration,
    System,
    Cognitive,
    Collaborative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionStatus {
    Proposed,
    UnderReview,
    Approved,
    Rejected,
    Implemented,
}

/// Statistics about the extension system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStats {
    pub total_extensions: usize,
    pub active_extensions: usize,
    pub loaded_extensions: usize,
    pub error_extensions: usize,
    pub pending_proposals: usize,
    pub successful_extensions: usize,
    pub failed_extensions: usize,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            stats: ExtensionStats {
                total_extensions: 0,
                active_extensions: 0,
                loaded_extensions: 0,
                error_extensions: 0,
                pending_proposals: 0,
                successful_extensions: 0,
                failed_extensions: 0,
            },
            auto_approve: false,
        }
    }

    /// Create extension manager with auto-approval enabled
    pub fn with_auto_approve() -> Self {
        Self {
            auto_approve: true,
            ..Self::new()
        }
    }

    /// Submit a new extension proposal
    pub async fn propose_extension(&mut self, mut proposal: ExtensionProposal) -> Result<String> {
        // Validate proposal
        self.validate_proposal(&proposal)?;

        // Auto-approve if enabled
        if self.auto_approve {
            proposal.status = ExtensionStatus::Approved;
        }

        let id = proposal.id.clone();
        self.proposals.insert(id.clone(), proposal);
        self.update_stats();

        tracing::info!("Extension proposal submitted: {}", id);
        Ok(id)
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&ExtensionProposal> {
        self.proposals.get(proposal_id)
    }

    /// Get mutable proposal by ID
    pub fn get_proposal_mut(&mut self, proposal_id: &str) -> Option<&mut ExtensionProposal> {
        self.proposals.get_mut(proposal_id)
    }

    /// List all proposals
    pub fn list_proposals(&self) -> Vec<&ExtensionProposal> {
        self.proposals.values().collect()
    }

    /// List proposals by status
    pub fn list_proposals_by_status(&self, status: &ExtensionStatus) -> Vec<&ExtensionProposal> {
        self.proposals
            .values()
            .filter(|p| &p.status == status)
            .collect()
    }

    /// Approve a proposal
    pub async fn approve_proposal(&mut self, proposal_id: &str) -> Result<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| CCSwarmError::extension(proposal_id, "Proposal not found"))?;

        if proposal.status != ExtensionStatus::UnderReview {
            return Err(CCSwarmError::extension(
                proposal_id,
                format!("Cannot approve proposal in status: {:?}", proposal.status)
            ));
        }

        proposal.status = ExtensionStatus::Approved;
        self.update_stats();

        tracing::info!("Extension proposal approved: {}", proposal_id);
        Ok(())
    }

    /// Reject a proposal
    pub async fn reject_proposal(&mut self, proposal_id: &str, reason: String) -> Result<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| CCSwarmError::extension(proposal_id, "Proposal not found"))?;

        proposal.status = ExtensionStatus::Rejected;
        self.update_stats();

        tracing::info!("Extension proposal rejected: {} - {}", proposal_id, reason);
        Ok(())
    }

    /// Mark proposal as implemented
    pub async fn mark_implemented(&mut self, proposal_id: &str) -> Result<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| CCSwarmError::extension(proposal_id, "Proposal not found"))?;

        if proposal.status != ExtensionStatus::Approved {
            return Err(CCSwarmError::extension(
                proposal_id,
                format!("Cannot implement proposal in status: {:?}", proposal.status)
            ));
        }

        proposal.status = ExtensionStatus::Implemented;
        self.update_stats();

        tracing::info!("Extension proposal implemented: {}", proposal_id);
        Ok(())
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> ExtensionStats {
        self.stats.clone()
    }

    /// Update internal statistics
    fn update_stats(&mut self) {
        let total = self.proposals.len();
        let pending = self.list_proposals_by_status(&ExtensionStatus::Proposed).len() +
                     self.list_proposals_by_status(&ExtensionStatus::UnderReview).len();
        let successful = self.list_proposals_by_status(&ExtensionStatus::Implemented).len();
        let failed = self.list_proposals_by_status(&ExtensionStatus::Rejected).len();

        self.stats.total_extensions = total;
        self.stats.pending_proposals = pending;
        self.stats.successful_extensions = successful;
        self.stats.failed_extensions = failed;
    }

    /// Validate a proposal before submission
    fn validate_proposal(&self, proposal: &ExtensionProposal) -> Result<()> {
        if proposal.title.trim().is_empty() {
            return Err(CCSwarmError::extension(&proposal.id, "Proposal title cannot be empty"));
        }

        if proposal.description.trim().is_empty() {
            return Err(CCSwarmError::extension(&proposal.id, "Proposal description cannot be empty"));
        }

        if proposal.proposer.trim().is_empty() {
            return Err(CCSwarmError::extension(&proposal.id, "Proposer cannot be empty"));
        }

        // Validate risk assessment
        if proposal.risk_assessment.overall_risk_score < 0.0 || proposal.risk_assessment.overall_risk_score > 1.0 {
            return Err(CCSwarmError::extension(&proposal.id, "Risk score must be between 0.0 and 1.0"));
        }

        Ok(())
    }

    /// Clean up old rejected proposals
    pub async fn cleanup_old_proposals(&mut self, max_age: chrono::Duration) -> Result<usize> {
        let cutoff = chrono::Utc::now() - max_age;
        let initial_count = self.proposals.len();

        self.proposals.retain(|_, proposal| {
            proposal.status != ExtensionStatus::Rejected || proposal.created_at > cutoff
        });

        let removed = initial_count - self.proposals.len();
        if removed > 0 {
            self.update_stats();
            tracing::info!("Cleaned up {} old proposals", removed);
        }

        Ok(removed)
    }
}

/// Current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentState {
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub performance_metrics: HashMap<String, f64>,
}

impl Default for CurrentState {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentState {
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            limitations: Vec::new(),
            performance_metrics: HashMap::new(),
        }
    }
}

/// Proposed state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedState {
    pub new_capabilities: Vec<String>,
    pub removed_limitations: Vec<String>,
    pub expected_improvements: HashMap<String, f64>,
    pub performance_targets: HashMap<String, f64>,
}

impl Default for ProposedState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProposedState {
    pub fn new() -> Self {
        Self {
            new_capabilities: Vec::new(),
            removed_limitations: Vec::new(),
            expected_improvements: HashMap::new(),
            performance_targets: HashMap::new(),
        }
    }
}

/// Implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub phases: Vec<ImplementationPhase>,
    pub total_duration: std::time::Duration,
    pub resources_required: Vec<String>,
    pub timeline: String,
    pub dependencies: Vec<String>,
}

/// Implementation phase with reduced redundancy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    pub name: String,
    pub description: String,
    pub estimated_duration: std::time::Duration,
    pub dependencies: Vec<String>,
    pub tasks: Vec<String>,
    pub validation_method: String,
    pub complexity: ComplexityLevel,
}

/// Complexity level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Risk assessment with improved structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: RiskLevel,
    pub potential_issues: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub rollback_plan: String,
    pub overall_risk_score: f32,
    pub categories: Vec<RiskCategory>,
}

/// Risk categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskCategory {
    Security,
    Performance,
    Compatibility,
    Stability,
    Maintenance,
    Compliance,
    UserExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Success criterion with reduced redundancy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub name: String,
    pub description: String,
    pub target_value: f64,
    pub measurement_method: String,
    pub measurable: bool,
}

