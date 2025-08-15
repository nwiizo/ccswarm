//! Propagation mechanism for sharing successful extensions across agents

use super::*;
// Using string IDs instead of direct agent references
use anyhow::{anyhow, Result};
use chrono::TimeDelta;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Manages the propagation of extensions across agents
pub struct PropagationManager {
    /// Propagation rules
    rules: PropagationRules,
    /// Propagation history
    #[allow(dead_code)]
    history: Arc<RwLock<Vec<PropagationRecord>>>,
    /// Active propagations
    active_propagations: Arc<RwLock<HashMap<Uuid, ActivePropagation>>>,
    /// Propagation status tracking
    propagation_status: Arc<Mutex<HashMap<String, PropagationStatus>>>,
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
        adoption_time: TimeDelta,
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
    Propagating { started_at: DateTime<Utc> },
}

/// Plan for propagating an extension
#[derive(Debug, Clone)]
pub enum PropagationPlan {
    Immediate {
        target_agents: Vec<String>,
        propagation_type: PropagationType,
    },
    Gradual {
        waves: Vec<PropagationWave>,
        total_agents: usize,
    },
    NoPropagation {
        reason: String,
    },
}

impl PropagationManager {
    pub fn new(rules: PropagationRules) -> Self {
        Self {
            rules,
            history: Arc::new(RwLock::new(Vec::new())),
            active_propagations: Arc::new(RwLock::new(HashMap::new())),
            propagation_status: Arc::new(Mutex::new(HashMap::new())),
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
        let propagation_type = self.determine_propagation_type(extension);

        // Filter eligible agents
        let eligible_agents = self
            .filter_eligible_agents(extension, source_agent_id, all_agent_ids)
            .await?;

        if eligible_agents.is_empty() {
            return Ok(PropagationPlan::NoPropagation {
                reason: "No eligible agents for propagation".to_string(),
            });
        }

        // Create propagation plan based on type
        match propagation_type {
            PropagationType::Emergency => Ok(PropagationPlan::Immediate {
                target_agents: eligible_agents,
                propagation_type,
            }),
            PropagationType::Mandatory => {
                // Gradual rollout for mandatory updates
                let waves = self.create_propagation_waves(&eligible_agents, 3);
                Ok(PropagationPlan::Gradual {
                    waves,
                    total_agents: eligible_agents.len(),
                })
            }
            _ => {
                // Optional propagation to interested agents only
                let interested_agents = self
                    .filter_interested_agents(&eligible_agents, extension)
                    .await?;

                if interested_agents.is_empty() {
                    Ok(PropagationPlan::NoPropagation {
                        reason: "No agents interested in this extension".to_string(),
                    })
                } else {
                    Ok(PropagationPlan::Immediate {
                        target_agents: interested_agents,
                        propagation_type,
                    })
                }
            }
        }
    }

    /// Check if extension meets criteria for propagation
    async fn meets_propagation_criteria(&self, extension: &Extension) -> Result<bool> {
        // Check success rate from test results
        let success_count = extension.test_results.iter().filter(|r| r.passed).count();
        let total_tests = extension.test_results.len();

        if total_tests == 0 {
            return Ok(false);
        }

        let success_rate = success_count as f64 / total_tests as f64;
        if success_rate < self.rules.min_success_rate {
            return Ok(false);
        }

        // Check adoption metrics
        if extension.metrics.adoption_rate < 0.5 {
            return Ok(false);
        }

        // Check error rate
        if extension.metrics.error_rate > 0.1 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Determine the type of propagation based on extension characteristics
    fn determine_propagation_type(&self, extension: &Extension) -> PropagationType {
        // Check if it's a mandatory type
        let extension_type_str = format!("{:?}", extension.proposal.extension_type).to_lowercase();
        if self.rules.mandatory_types.contains(&extension_type_str) {
            return PropagationType::Mandatory;
        }

        // Check if it's recommended based on performance improvements
        if extension
            .metrics
            .performance_improvement
            .values()
            .any(|&v| v > 0.2)
        {
            return PropagationType::Recommended;
        }

        // Default to optional
        PropagationType::Optional
    }

    /// Filter agents eligible for receiving the extension
    async fn filter_eligible_agents(
        &self,
        extension: &Extension,
        source_agent_id: &str,
        all_agent_ids: &[String],
    ) -> Result<Vec<String>> {
        let mut eligible = Vec::new();

        for agent_id in all_agent_ids {
            if agent_id == source_agent_id {
                continue; // Skip source agent
            }

            // Check compatibility
            if self.is_compatible_with_agent(extension, agent_id).await? {
                eligible.push(agent_id.clone());
            }
        }

        Ok(eligible)
    }

    /// Check if extension is compatible with an agent
    async fn is_compatible_with_agent(
        &self,
        _extension: &Extension,
        _agent_id: &str,
    ) -> Result<bool> {
        // In a real implementation, this would check:
        // 1. Agent's current extensions
        // 2. Compatibility matrix
        // 3. Agent's role and capabilities
        // 4. Resource requirements

        // For now, assume all agents are compatible
        Ok(true)
    }

    /// Filter agents that would be interested in the extension
    async fn filter_interested_agents(
        &self,
        agent_ids: &[String],
        extension: &Extension,
    ) -> Result<Vec<String>> {
        let mut interested = Vec::new();

        for agent_id in agent_ids {
            if self.agent_would_benefit(agent_id, extension).await {
                interested.push(agent_id.clone());
            }
        }

        Ok(interested)
    }

    /// Check if an agent would benefit from the extension
    async fn agent_would_benefit(&self, _agent_id: &str, extension: &Extension) -> bool {
        // Determine benefit based on extension type and agent role
        let propagation_type = self.determine_propagation_type(extension);

        match propagation_type {
            PropagationType::Mandatory => true,
            PropagationType::Recommended => {
                // Check if extension type matches recommended types
                // Since we don't have agent_configs, use extension type
                matches!(
                    extension.proposal.extension_type,
                    ExtensionType::Capability | ExtensionType::Cognitive
                )
            }
            PropagationType::Optional => {
                // For optional extensions, check if it improves performance
                extension
                    .metrics
                    .performance_improvement
                    .values()
                    .any(|&v| v > 0.1)
            }
            _ => false,
        }
    }

    /// Create propagation waves for gradual rollout
    fn create_propagation_waves(
        &self,
        agent_ids: &[String],
        wave_count: usize,
    ) -> Vec<PropagationWave> {
        let agents_per_wave = agent_ids.len().div_ceil(wave_count);
        let mut waves = Vec::new();

        for (i, chunk) in agent_ids.chunks(agents_per_wave).enumerate() {
            waves.push(PropagationWave {
                wave_number: i as u32 + 1,
                agents_in_wave: chunk.to_vec(),
                success_threshold: 0.8,
                rollback_on_failure: i == 0, // Only rollback first wave failures
            });
        }

        waves
    }

    /// Execute a propagation plan
    pub async fn execute_propagation(
        &self,
        plan: PropagationPlan,
        extension: Extension,
    ) -> Result<PropagationRecord> {
        match plan {
            PropagationPlan::Immediate {
                target_agents,
                propagation_type,
            } => {
                self.execute_immediate_propagation(extension, target_agents, propagation_type)
                    .await
            }
            PropagationPlan::Gradual { waves, .. } => {
                self.execute_gradual_propagation(extension, waves).await
            }
            PropagationPlan::NoPropagation { reason } => Err(anyhow!("No propagation: {}", reason)),
        }
    }

    /// Execute immediate propagation to all target agents
    async fn execute_immediate_propagation(
        &self,
        extension: Extension,
        target_agents: Vec<String>,
        propagation_type: PropagationType,
    ) -> Result<PropagationRecord> {
        let propagation_id = Uuid::new_v4();
        let started_at = chrono::Utc::now();

        // Create active propagation
        let active_propagation = ActivePropagation {
            extension: extension.clone(),
            target_agents: target_agents.clone(),
            propagation_type,
            wave: PropagationWave {
                wave_number: 1,
                agents_in_wave: target_agents.clone(),
                success_threshold: 0.8,
                rollback_on_failure: false,
            },
            status: PropagationStatus::InProgress,
        };

        self.active_propagations
            .write()
            .await
            .insert(propagation_id, active_propagation);

        // Propagate to each agent
        let mut results = HashMap::new();
        for agent_id in &target_agents {
            let result = self.propagate_to_agent(&extension, agent_id).await?;
            results.insert(agent_id.clone(), result);
        }

        // Create propagation record
        let record = PropagationRecord {
            id: propagation_id,
            extension_id: extension.id,
            source_agent: "orchestrator".to_string(),
            target_agents,
            started_at,
            completed_at: Some(chrono::Utc::now()),
            propagation_type,
            results,
        };

        // Store in history
        self.history.write().await.push(record.clone());

        Ok(record)
    }

    /// Execute gradual propagation with waves
    async fn execute_gradual_propagation(
        &self,
        extension: Extension,
        waves: Vec<PropagationWave>,
    ) -> Result<PropagationRecord> {
        let propagation_id = Uuid::new_v4();
        let started_at = chrono::Utc::now();
        let mut all_results = HashMap::new();
        let mut all_target_agents = Vec::new();

        for wave in waves {
            tracing::info!("Executing propagation wave {}", wave.wave_number);

            // Create active propagation for this wave
            let active_propagation = ActivePropagation {
                extension: extension.clone(),
                target_agents: wave.agents_in_wave.clone(),
                propagation_type: PropagationType::Mandatory,
                wave: wave.clone(),
                status: PropagationStatus::InProgress,
            };

            self.active_propagations
                .write()
                .await
                .insert(propagation_id, active_propagation);

            // Propagate to agents in this wave
            let wave_results = self
                .propagate_wave(&extension, &wave.agents_in_wave)
                .await?;

            // Check success rate
            let success_count = wave_results
                .values()
                .filter(|r| matches!(r, PropagationResult::Success { .. }))
                .count();
            let success_rate = success_count as f64 / wave.agents_in_wave.len() as f64;

            if success_rate < wave.success_threshold && wave.rollback_on_failure {
                // Rollback and stop propagation
                tracing::warn!(
                    "Wave {} failed with success rate {:.2}%, rolling back",
                    wave.wave_number,
                    success_rate * 100.0
                );
                self.initiate_rollback(&wave.agents_in_wave).await?;

                return Err(anyhow!(
                    "Propagation failed at wave {} with success rate {:.2}%",
                    wave.wave_number,
                    success_rate * 100.0
                ));
            }

            all_results.extend(wave_results);
            all_target_agents.extend(wave.agents_in_wave);

            // Wait between waves
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        // Create propagation record
        let record = PropagationRecord {
            id: propagation_id,
            extension_id: extension.id,
            source_agent: "orchestrator".to_string(),
            target_agents: all_target_agents,
            started_at,
            completed_at: Some(chrono::Utc::now()),
            propagation_type: PropagationType::Mandatory,
            results: all_results,
        };

        // Store in history
        self.history.write().await.push(record.clone());

        Ok(record)
    }

    /// Propagate extension to a single agent
    async fn propagate_to_agent(
        &self,
        _extension: &Extension,
        agent_id: &str,
    ) -> Result<PropagationResult> {
        tracing::info!("Propagating extension to agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Send extension via coordination bus
        // 2. Monitor installation
        // 3. Verify successful adoption
        // 4. Collect metrics

        // Simulate propagation with status tracking

        // Mark as propagating
        self.propagation_status
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?
            .insert(
                agent_id.to_string(),
                PropagationStatus::Propagating {
                    started_at: chrono::Utc::now(),
                },
            );

        // Simulate async propagation (in real implementation, this would be actual agent communication)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // For simulation, return success
        Ok(PropagationResult::Success {
            adoption_time: TimeDelta::try_seconds(1).unwrap_or(TimeDelta::zero()),
            improvements: vec![
                "Extension successfully installed".to_string(),
                "Performance metrics improved".to_string(),
            ],
        })
    }

    /// Propagate to a wave of agents
    async fn propagate_wave(
        &self,
        _extension: &Extension,
        agent_ids: &[String],
    ) -> Result<HashMap<String, PropagationResult>> {
        let mut results = HashMap::new();

        // Start propagation for all agents in the wave
        for agent_id in agent_ids {
            tracing::info!("Starting propagation to agent: {}", agent_id);

            // Mark agent as having propagation in progress
            {
                self.propagation_status
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?
                    .insert(
                        agent_id.clone(),
                        PropagationStatus::Propagating {
                            started_at: chrono::Utc::now(),
                        },
                    );
            }

            // In a real implementation, this would start async propagation
            // For now, we'll simulate it
        }

        // Wait a bit for propagations to "complete"
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Collect results
        for agent_id in agent_ids {
            let status = self
                .propagation_status
                .lock()
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to acquire lock: {}", e);
                    e.into_inner()
                })
                .get(agent_id)
                .cloned();

            let result = match status {
                Some(PropagationStatus::Completed) => {
                    PropagationResult::Success {
                        adoption_time: TimeDelta::try_seconds(1).unwrap_or(TimeDelta::zero()), // Simulated
                        improvements: vec![
                            format!("Extension deployed to {}", agent_id),
                            "Metrics collection started".to_string(),
                        ],
                    }
                }
                Some(PropagationStatus::Failed) => PropagationResult::Failure {
                    reason: "Simulated failure".to_string(),
                    retry_possible: true,
                },
                Some(PropagationStatus::Propagating { started_at }) => {
                    // Still in progress - simulate completion based on time elapsed
                    let elapsed = chrono::Utc::now() - started_at;
                    if elapsed.num_seconds() > 30 {
                        // Consider it complete after 30 seconds for simulation

                        // Update status to completed
                        self.propagation_status
                            .lock()
                            .unwrap_or_else(|e| {
                                tracing::error!("Failed to acquire lock: {}", e);
                                e.into_inner()
                            })
                            .insert(agent_id.clone(), PropagationStatus::Completed);

                        PropagationResult::Success {
                            adoption_time: TimeDelta::try_seconds(1).unwrap_or(TimeDelta::zero()),
                            improvements: vec![
                                "Extension installed successfully".to_string(),
                                "Extension activated".to_string(),
                            ],
                        }
                    } else {
                        // Still in progress
                        PropagationResult::Deferred {
                            reason: "Still in progress".to_string(),
                            retry_after: chrono::Utc::now()
                                + TimeDelta::try_seconds(1).unwrap_or(TimeDelta::zero()),
                        }
                    }
                }
                Some(PropagationStatus::Planning)
                | Some(PropagationStatus::InProgress)
                | Some(PropagationStatus::Monitoring)
                | Some(PropagationStatus::RolledBack) => {
                    // Other statuses - treat as in progress
                    PropagationResult::Deferred {
                        reason: "Operation in progress".to_string(),
                        retry_after: chrono::Utc::now()
                            + TimeDelta::try_seconds(1).unwrap_or(TimeDelta::zero()),
                    }
                }
                None => {
                    // No status found - treat as not started
                    PropagationResult::Failure {
                        reason: "Propagation not started".to_string(),
                        retry_possible: true,
                    }
                }
            };

            results.insert(agent_id.clone(), result);
        }

        tracing::info!(
            "Collected propagation results for {} agents",
            agent_ids.len()
        );
        Ok(results)
    }

    async fn initiate_rollback(&self, agent_ids: &[String]) -> Result<()> {
        // Initiate rollback on agents
        tracing::warn!("Initiating rollback for {} agents", agent_ids.len());

        for agent_id in agent_ids {
            tracing::info!("Rolling back extension for agent: {}", agent_id);

            // Update agent status to indicate rollback
            // Note: This would need to be implemented with proper status tracking
            self.propagation_status
                .lock()
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to acquire lock: {}", e);
                    e.into_inner()
                })
                .insert(agent_id.clone(), PropagationStatus::RolledBack);
        }

        Ok(())
    }

    /// Monitor active propagations and update their status
    pub async fn monitor_propagations(&self) -> Result<()> {
        let active = self.active_propagations.read().await;

        for (id, propagation) in active.iter() {
            tracing::debug!(
                "Monitoring propagation {} with status {:?}",
                id,
                propagation.status
            );

            // In a real implementation, this would:
            // 1. Check agent health
            // 2. Verify extension is working
            // 3. Collect performance metrics
            // 4. Update status accordingly
        }

        Ok(())
    }

    /// Analyze propagation effectiveness
    pub async fn analyze_propagation_effectiveness(
        &self,
        _propagation_type: PropagationType,
        _all_agent_ids: &[String],
    ) -> Result<PropagationAnalysis> {
        // Analyze historical propagations
        let history = self.history.read().await;

        let total_propagations = history.len();
        let successful_propagations = history
            .iter()
            .filter(|r| {
                r.results
                    .values()
                    .filter(|res| matches!(res, PropagationResult::Success { .. }))
                    .count()
                    > r.results.len() / 2
            })
            .count();

        let average_adoption_time = if successful_propagations > 0 {
            let total_time: i64 = history
                .iter()
                .flat_map(|r| r.results.values())
                .filter_map(|res| match res {
                    PropagationResult::Success { adoption_time, .. } => {
                        Some(adoption_time.num_seconds())
                    }
                    _ => None,
                })
                .sum();

            TimeDelta::try_seconds(total_time / successful_propagations as i64)
                .unwrap_or(TimeDelta::zero())
        } else {
            TimeDelta::zero()
        };

        Ok(PropagationAnalysis {
            total_propagations,
            successful_propagations,
            success_rate: if total_propagations > 0 {
                successful_propagations as f64 / total_propagations as f64
            } else {
                0.0
            },
            average_adoption_time,
            common_failure_reasons: self.analyze_failure_reasons(&history),
            recommended_improvements: self.generate_recommendations(&history),
        })
    }

    fn analyze_failure_reasons(&self, history: &[PropagationRecord]) -> Vec<String> {
        let mut reasons = HashMap::new();

        for record in history {
            for result in record.results.values() {
                if let PropagationResult::Failure { reason, .. } = result {
                    *reasons.entry(reason.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut sorted_reasons: Vec<_> = reasons.into_iter().collect();
        sorted_reasons.sort_by_key(|(_, count)| -*count);

        sorted_reasons
            .into_iter()
            .take(5)
            .map(|(reason, _)| reason)
            .collect()
    }

    fn generate_recommendations(&self, history: &[PropagationRecord]) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze patterns and generate recommendations
        if history.len() > 10 {
            let recent_success_rate = history
                .iter()
                .rev()
                .take(5)
                .filter(|r| {
                    r.results
                        .values()
                        .filter(|res| matches!(res, PropagationResult::Success { .. }))
                        .count()
                        > r.results.len() / 2
                })
                .count() as f64
                / 5.0;

            if recent_success_rate < 0.6 {
                recommendations.push(
                    "Consider increasing testing requirements before propagation".to_string(),
                );
                recommendations
                    .push("Implement canary deployments for risky extensions".to_string());
            }
        }

        recommendations
    }

    /// Get propagation status for a specific agent
    pub fn get_propagation_status(&self, _agent_id: &str) -> Option<PropagationStatus> {
        // Implementation would check active propagations
        None
    }

    /// Get all propagation statuses
    pub async fn get_all_propagation_statuses(&self) -> HashMap<String, PropagationStatus> {
        self.propagation_status
            .lock()
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire lock: {}", e);
                e.into_inner()
            })
            .clone()
    }
}

/// Analysis of propagation effectiveness
#[derive(Debug, Clone)]
pub struct PropagationAnalysis {
    pub total_propagations: usize,
    pub successful_propagations: usize,
    pub success_rate: f64,
    pub average_adoption_time: TimeDelta,
    pub common_failure_reasons: Vec<String>,
    pub recommended_improvements: Vec<String>,
}

impl Default for PropagationManager {
    fn default() -> Self {
        Self::new(PropagationRules::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_propagation_evaluation() {
        let manager = PropagationManager::default();

        // Create a test extension
        let extension = create_test_extension();

        let plan = manager
            .evaluate_for_propagation(
                &extension,
                "source-agent",
                &["agent-1".to_string(), "agent-2".to_string()],
            )
            .await
            .unwrap();

        match plan {
            PropagationPlan::NoPropagation { reason } => {
                assert!(reason.contains("criteria"));
            }
            _ => panic!("Expected no propagation for test extension"),
        }
    }

    fn create_test_extension() -> Extension {
        Extension {
            id: Uuid::new_v4(),
            proposal: ExtensionProposal {
                id: Uuid::new_v4(),
                proposer: "test-agent".to_string(),
                extension_type: ExtensionType::Capability,
                title: "Test Extension".to_string(),
                description: "Test extension for unit tests".to_string(),
                current_state: CurrentState {
                    capabilities: vec![],
                    limitations: vec![],
                    performance_metrics: HashMap::new(),
                },
                proposed_state: ProposedState {
                    new_capabilities: vec![],
                    expected_improvements: vec![],
                    performance_targets: HashMap::new(),
                },
                implementation_plan: ImplementationPlan {
                    phases: vec![],
                    timeline: "1 week".to_string(),
                    resources_required: vec![],
                    dependencies: vec![],
                },
                risk_assessment: RiskAssessment {
                    risks: vec![],
                    mitigation_strategies: vec![],
                    rollback_plan: "Revert changes".to_string(),
                    overall_risk_score: 0.1,
                },
                success_criteria: vec![],
                created_at: chrono::Utc::now(),
                status: ExtensionStatus::Proposed,
            },
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
        }
    }
}
