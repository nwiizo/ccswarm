//! Propagation mechanism for sharing successful extensions across agents

use super::*;
// Using string IDs instead of direct agent references
use chrono::Duration;
use std::collections::{HashMap, HashSet};

/// Manages the propagation of extensions across agents
pub struct PropagationManager {
    /// Propagation rules
    rules: PropagationRules,
    /// Propagation history
    history: Arc<RwLock<Vec<PropagationRecord>>>,
    /// Active propagations
    active_propagations: Arc<RwLock<HashMap<Uuid, ActivePropagation>>>,
}

/// Rules governing extension propagation
#[derive(Debug, Clone)]
pub struct PropagationRules {
    /// Mandatory extensions that all agents must adopt
    pub mandatory_types: HashSet<String>,
    /// Recommended extensions based on role
    pub role_recommendations: HashMap<AgentRole, Vec<String>>,
    /// Minimum success rate for propagation
    pub min_success_rate: f64,
    /// Minimum adoption count before propagation
    pub min_adoption_count: usize,
    /// Compatibility matrix
    pub compatibility_matrix: CompatibilityMatrix,
}

impl Default for PropagationRules {
    fn default() -> Self {
        let mut mandatory_types = HashSet::new();
        mandatory_types.insert("security_patches".to_string());
        mandatory_types.insert("critical_bug_fixes".to_string());
        mandatory_types.insert("protocol_updates".to_string());

        let mut role_recommendations = HashMap::new();
        role_recommendations.insert(
            crate::identity::default_frontend_role(),
            vec![
                "ui_frameworks".to_string(),
                "performance_tools".to_string(),
                "accessibility_features".to_string(),
            ],
        );
        role_recommendations.insert(
            crate::identity::default_backend_role(),
            vec![
                "api_frameworks".to_string(),
                "database_tools".to_string(),
                "caching_solutions".to_string(),
            ],
        );

        Self {
            mandatory_types,
            role_recommendations,
            min_success_rate: 0.8,
            min_adoption_count: 2,
            compatibility_matrix: CompatibilityMatrix::default(),
        }
    }
}

/// Compatibility matrix for extensions
#[derive(Debug, Clone, Default)]
pub struct CompatibilityMatrix {
    /// Extension compatibility rules
    pub compatibilities: HashMap<String, Vec<String>>,
    /// Conflicting extensions
    pub conflicts: HashMap<String, Vec<String>>,
    /// Required dependencies
    pub dependencies: HashMap<String, Vec<String>>,
}

/// Record of a propagation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationRecord {
    pub id: Uuid,
    pub extension_id: Uuid,
    pub source_agent: String,
    pub target_agents: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub propagation_type: PropagationType,
    pub results: HashMap<String, PropagationResult>,
}

/// Type of propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropagationType {
    Mandatory,
    Recommended,
    Optional,
    Experimental,
    Emergency,
}

/// Result of propagation to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationResult {
    Success {
        adoption_time: Duration,
        improvements: Vec<String>,
    },
    Failure {
        reason: String,
        retry_possible: bool,
    },
    Deferred {
        reason: String,
        retry_after: DateTime<Utc>,
    },
    Incompatible {
        conflicts: Vec<String>,
    },
}

/// Active propagation tracking
#[derive(Debug, Clone)]
pub struct ActivePropagation {
    pub extension: Extension,
    pub target_agents: Vec<String>,
    pub propagation_type: PropagationType,
    pub wave: PropagationWave,
    pub status: PropagationStatus,
}

/// Propagation wave for gradual rollout
#[derive(Debug, Clone)]
pub struct PropagationWave {
    pub wave_number: u32,
    pub agents_in_wave: Vec<String>,
    pub success_threshold: f64,
    pub rollback_on_failure: bool,
}

/// Status of propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationStatus {
    Planning,
    InProgress,
    Monitoring,
    Completed,
    Failed,
    RolledBack,
}

impl PropagationManager {
    pub fn new(rules: PropagationRules) -> Self {
        Self {
            rules,
            history: Arc::new(RwLock::new(Vec::new())),
            active_propagations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluate an extension for propagation
    pub async fn evaluate_for_propagation(
        &self,
        extension: &Extension,
        source_agent_id: &str,
        all_agent_ids: &[String],
    ) -> Result<PropagationPlan> {
        // Check if extension meets propagation criteria
        if !self.meets_propagation_criteria(extension).await? {
            return Ok(PropagationPlan::NoPropagation {
                reason: "Does not meet propagation criteria".to_string(),
            });
        }

        // Determine propagation type
        let propagation_type = self.determine_propagation_type(extension)?;

        // Identify target agents
        let target_agents = self.identify_target_agents(
            extension,
            source_agent_id,
            all_agent_ids,
            propagation_type,
        ).await?;

        if target_agents.is_empty() {
            return Ok(PropagationPlan::NoPropagation {
                reason: "No suitable target agents found".to_string(),
            });
        }

        // Check compatibility
        let compatibility_report = self.check_compatibility(extension, &target_agents).await?;

        // Create propagation waves
        let waves = self.create_propagation_waves(
            &target_agents,
            &compatibility_report,
            propagation_type,
        )?;

        Ok(PropagationPlan::Propagate {
            extension_id: extension.id,
            propagation_type,
            target_agents: target_agents.clone(),
            waves: waves.clone(),
            estimated_duration: self.estimate_duration(&waves),
            risk_assessment: self.assess_propagation_risk(extension, &target_agents),
        })
    }

    /// Start propagation of an extension
    pub async fn start_propagation(
        &self,
        plan: PropagationPlan,
        extension: Extension,
    ) -> Result<Uuid> {
        let propagation_id = Uuid::new_v4();

        match plan {
            PropagationPlan::NoPropagation { reason } => {
                anyhow::bail!("Cannot start propagation: {}", reason);
            }
            PropagationPlan::Propagate {
                propagation_type,
                target_agents,
                waves,
                ..
            } => {
                let active_propagation = ActivePropagation {
                    extension,
                    target_agents: target_agents.clone(),
                    propagation_type,
                    wave: waves[0].clone(),
                    status: PropagationStatus::Planning,
                };

                let mut active = self.active_propagations.write().await;
                active.insert(propagation_id, active_propagation);

                // Start first wave
                self.start_wave(propagation_id, 0).await?;

                Ok(propagation_id)
            }
        }
    }

    /// Monitor propagation progress
    pub async fn monitor_propagation(
        &self,
        propagation_id: Uuid,
    ) -> Result<PropagationProgress> {
        let active = self.active_propagations.read().await;
        let propagation = active.get(&propagation_id)
            .context("Propagation not found")?;

        // Get results from agents
        let results = self.collect_propagation_results(&propagation.wave.agents_in_wave).await?;

        let successful = results.values()
            .filter(|r| matches!(r, PropagationResult::Success { .. }))
            .count();
        let failed = results.values()
            .filter(|r| matches!(r, PropagationResult::Failure { .. }))
            .count();

        let success_rate = successful as f64 / propagation.wave.agents_in_wave.len() as f64;

        Ok(PropagationProgress {
            propagation_id,
            current_wave: propagation.wave.wave_number,
            agents_completed: successful + failed,
            agents_pending: propagation.wave.agents_in_wave.len() - successful - failed,
            success_rate,
            status: propagation.status,
            results,
        })
    }

    /// Advance to next propagation wave
    pub async fn advance_wave(&self, propagation_id: Uuid) -> Result<()> {
        let mut active = self.active_propagations.write().await;
        let propagation = active.get_mut(&propagation_id)
            .context("Propagation not found")?;

        propagation.wave.wave_number += 1;
        propagation.status = PropagationStatus::InProgress;

        Ok(())
    }

    /// Rollback a propagation
    pub async fn rollback_propagation(&self, propagation_id: Uuid) -> Result<()> {
        let mut active = self.active_propagations.write().await;
        let propagation = active.get_mut(&propagation_id)
            .context("Propagation not found")?;

        propagation.status = PropagationStatus::RolledBack;

        // Initiate rollback on affected agents
        self.initiate_rollback(&propagation.wave.agents_in_wave).await?;

        Ok(())
    }

    async fn meets_propagation_criteria(&self, extension: &Extension) -> Result<bool> {
        // Check success rate
        let success_rate = extension.metrics.user_satisfaction;
        if success_rate < self.rules.min_success_rate {
            return Ok(false);
        }

        // Check adoption count
        // This would check actual adoption metrics
        
        Ok(true)
    }

    fn determine_propagation_type(&self, extension: &Extension) -> Result<PropagationType> {
        let extension_category = match extension.proposal.extension_type {
            ExtensionType::System => "protocol_updates",
            ExtensionType::Capability => "capability_enhancement",
            ExtensionType::Cognitive => "cognitive_improvement",
            ExtensionType::Collaborative => "collaboration_enhancement",
        };

        if self.rules.mandatory_types.contains(extension_category) {
            Ok(PropagationType::Mandatory)
        } else {
            Ok(PropagationType::Recommended)
        }
    }

    async fn identify_target_agents(
        &self,
        extension: &Extension,
        source_agent_id: &str,
        all_agent_ids: &[String],
        propagation_type: PropagationType,
    ) -> Result<Vec<String>> {
        let mut targets = Vec::new();

        for agent_id in all_agent_ids {
            if agent_id == source_agent_id {
                continue;
            }

            let should_propagate = match propagation_type {
                PropagationType::Mandatory => true,
                PropagationType::Recommended => {
                    // TODO: Get agent role from config
                    true
                }
                PropagationType::Optional => {
                    self.agent_would_benefit(agent_id, extension).await
                }
                _ => false,
            };

            if should_propagate {
                targets.push(agent_id.clone());
            }
        }

        Ok(targets)
    }

    fn is_recommended_for_role(&self, role: &AgentRole, _extension: &Extension) -> bool {
        if let Some(_recommendations) = self.rules.role_recommendations.get(role) {
            // Check if extension matches any recommendation
            true // Simplified
        } else {
            false
        }
    }

    async fn agent_would_benefit(&self, _agent_id: &str, _extension: &Extension) -> bool {
        // Analyze if agent would benefit from extension
        true // Simplified
    }

    async fn check_compatibility(
        &self,
        _extension: &Extension,
        target_agents: &[String],
    ) -> Result<CompatibilityReport> {
        let mut report = CompatibilityReport {
            compatible_agents: vec![],
            incompatible_agents: HashMap::new(),
            warnings: HashMap::new(),
        };

        for agent_id in target_agents {
            // Check compatibility
            // This would check actual agent capabilities and conflicts
            report.compatible_agents.push(agent_id.clone());
        }

        Ok(report)
    }

    fn create_propagation_waves(
        &self,
        target_agents: &[String],
        compatibility_report: &CompatibilityReport,
        propagation_type: PropagationType,
    ) -> Result<Vec<PropagationWave>> {
        let wave_size = match propagation_type {
            PropagationType::Mandatory => target_agents.len(), // All at once
            PropagationType::Emergency => target_agents.len(), // All at once
            _ => 3, // Gradual rollout
        };

        let mut waves = Vec::new();
        let mut remaining_agents = compatibility_report.compatible_agents.clone();

        let mut wave_number = 1;
        while !remaining_agents.is_empty() {
            let agents_in_wave: Vec<_> = remaining_agents.drain(..wave_size.min(remaining_agents.len())).collect();
            
            waves.push(PropagationWave {
                wave_number,
                agents_in_wave,
                success_threshold: 0.8,
                rollback_on_failure: propagation_type != PropagationType::Experimental,
            });
            
            wave_number += 1;
        }

        Ok(waves)
    }

    fn estimate_duration(&self, waves: &[PropagationWave]) -> Duration {
        // Estimate based on wave count and typical adoption time
        Duration::days(waves.len() as i64 * 2)
    }

    fn assess_propagation_risk(
        &self,
        _extension: &Extension,
        _target_agents: &[String],
    ) -> RiskAssessment {
        // Assess risks of propagation
        RiskAssessment {
            risks: vec![],
            mitigation_strategies: vec![],
            rollback_plan: "Automated rollback on failure".to_string(),
            overall_risk_score: 0.3,
        }
    }

    async fn start_wave(&self, _propagation_id: Uuid, _wave_number: u32) -> Result<()> {
        // Start propagation for agents in wave
        Ok(())
    }

    async fn collect_propagation_results(
        &self,
        _agent_ids: &[String],
    ) -> Result<HashMap<String, PropagationResult>> {
        // Collect results from agents
        Ok(HashMap::new())
    }

    async fn initiate_rollback(&self, _agent_ids: &[String]) -> Result<()> {
        // Initiate rollback on agents
        Ok(())
    }
}

/// Propagation plan
#[derive(Debug, Clone)]
pub enum PropagationPlan {
    NoPropagation {
        reason: String,
    },
    Propagate {
        extension_id: Uuid,
        propagation_type: PropagationType,
        target_agents: Vec<String>,
        waves: Vec<PropagationWave>,
        estimated_duration: Duration,
        risk_assessment: RiskAssessment,
    },
}

/// Compatibility report
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub compatible_agents: Vec<String>,
    pub incompatible_agents: HashMap<String, Vec<String>>, // agent_id -> conflicts
    pub warnings: HashMap<String, Vec<String>>, // agent_id -> warnings
}

/// Propagation progress
#[derive(Debug, Clone)]
pub struct PropagationProgress {
    pub propagation_id: Uuid,
    pub current_wave: u32,
    pub agents_completed: usize,
    pub agents_pending: usize,
    pub success_rate: f64,
    pub status: PropagationStatus,
    pub results: HashMap<String, PropagationResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_rules_default() {
        let rules = PropagationRules::default();
        assert!(rules.mandatory_types.contains("security_patches"));
        assert_eq!(rules.min_success_rate, 0.8);
    }
}