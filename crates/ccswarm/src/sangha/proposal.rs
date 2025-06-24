//! Proposal management for Sangha

use super::*;
// Temporarily removed extension dependency
// use crate::extension::ExtensionType;

/// Placeholder for ExtensionType since extension module is disabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionType {
    Capability,
    Tool,
    Integration,
}
use serde_json::json;

/// Builder for creating proposals
pub struct ProposalBuilder {
    title: String,
    description: String,
    proposer: String,
    proposal_type: ProposalType,
    required_consensus: ConsensusType,
    voting_duration_secs: Option<u64>,
    data: serde_json::Value,
}

impl ProposalBuilder {
    pub fn new(title: String, proposer: String, proposal_type: ProposalType) -> Self {
        Self {
            title,
            description: String::new(),
            proposer,
            proposal_type,
            required_consensus: ConsensusType::SimpleMajority,
            voting_duration_secs: None,
            data: json!({}),
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn required_consensus(mut self, consensus: ConsensusType) -> Self {
        self.required_consensus = consensus;
        self
    }

    pub fn voting_duration(mut self, secs: u64) -> Self {
        self.voting_duration_secs = Some(secs);
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Proposal {
        let now = Utc::now();
        let voting_duration = self.voting_duration_secs.unwrap_or(300); // 5 minutes default
        
        Proposal {
            id: Uuid::new_v4(),
            proposal_type: self.proposal_type,
            title: self.title,
            description: self.description,
            proposer: self.proposer,
            created_at: now,
            voting_deadline: now + chrono::Duration::seconds(voting_duration as i64),
            status: ProposalStatus::Draft,
            required_consensus: self.required_consensus,
            data: self.data,
        }
    }
}

/// Agent extension proposal details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExtensionProposalData {
    pub agent_id: String,
    pub extension_type: ExtensionType,
    pub current_capabilities: Vec<String>,
    pub proposed_capabilities: Vec<String>,
    pub implementation_plan: ImplementationPlan,
    pub risk_assessment: RiskAssessment,
    pub success_criteria: Vec<String>,
}

/// System extension proposal details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemExtensionProposalData {
    pub title: String,
    pub current_limitation: SystemLimitation,
    pub proposed_solution: ProposedSolution,
    pub impact_analysis: ImpactAnalysis,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimitation {
    pub description: String,
    pub bottlenecks: Vec<String>,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedSolution {
    pub architecture_changes: Vec<String>,
    pub new_components: Vec<ComponentSpec>,
    pub migration_strategy: MigrationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub name: String,
    pub language: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStrategy {
    pub phases: Vec<MigrationPhase>,
    pub rollback_plan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPhase {
    pub name: String,
    pub description: String,
    pub duration_estimate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub affected_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub development_time: String,
    pub required_expertise: Vec<String>,
    pub infrastructure: Vec<String>,
    pub estimated_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub phases: Vec<Phase>,
    pub timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub description: String,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risks: Vec<Risk>,
    pub overall_risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: RiskLevel,
    pub impact: RiskLevel,
    pub mitigation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Doctrine proposal details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineProposalData {
    pub category: DoctrineCategory,
    pub current_doctrine: Option<String>,
    pub proposed_doctrine: String,
    pub rationale: String,
    pub implications: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoctrineCategory {
    CorePrinciple,
    OperationalGuideline,
    TechnicalStandard,
    EthicalRule,
    ProcessDefinition,
}

/// Task delegation proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDelegationProposalData {
    pub task_id: String,
    pub task_description: String,
    pub proposed_agent: String,
    pub agent_role: AgentRole,
    pub rationale: String,
    pub estimated_effort: String,
    pub dependencies: Vec<String>,
}

impl Proposal {
    /// Create an agent extension proposal
    pub fn agent_extension(
        title: String,
        proposer: String,
        data: AgentExtensionProposalData,
    ) -> Result<Self> {
        let proposal = ProposalBuilder::new(title, proposer, ProposalType::AgentExtension)
            .description(format!(
                "Agent {} requests to extend capabilities with {:?}",
                data.agent_id, data.extension_type
            ))
            .required_consensus(ConsensusType::SimpleMajority)
            .data(serde_json::to_value(data)?)
            .build();
            
        Ok(proposal)
    }

    /// Create a system extension proposal
    pub fn system_extension(
        title: String,
        proposer: String,
        data: SystemExtensionProposalData,
    ) -> Result<Self> {
        let proposal = ProposalBuilder::new(title, proposer, ProposalType::SystemExtension)
            .description(format!(
                "System extension proposal: {}",
                data.current_limitation.description
            ))
            .required_consensus(ConsensusType::SuperMajority) // System changes need higher consensus
            .voting_duration(600) // 10 minutes for system changes
            .data(serde_json::to_value(data)?)
            .build();
            
        Ok(proposal)
    }

    /// Create a doctrine proposal
    pub fn doctrine(
        title: String,
        proposer: String,
        data: DoctrineProposalData,
    ) -> Result<Self> {
        let proposal = ProposalBuilder::new(title, proposer, ProposalType::Doctrine)
            .description(format!(
                "Doctrine change in {:?}: {}",
                data.category, data.rationale
            ))
            .required_consensus(match data.category {
                DoctrineCategory::CorePrinciple => ConsensusType::Unanimous,
                DoctrineCategory::EthicalRule => ConsensusType::SuperMajority,
                _ => ConsensusType::SimpleMajority,
            })
            .data(serde_json::to_value(data)?)
            .build();
            
        Ok(proposal)
    }

    /// Check if the proposal has expired
    pub fn is_expired(&self) -> bool {
        self.voting_deadline < Utc::now()
    }

    /// Check if the proposal is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProposalStatus::Voting) && !self.is_expired()
    }

    /// Transition the proposal to voting status
    pub fn start_voting(&mut self) -> Result<()> {
        if self.status != ProposalStatus::Draft {
            anyhow::bail!("Can only start voting on draft proposals");
        }
        
        self.status = ProposalStatus::Voting;
        Ok(())
    }

    /// Cancel the proposal
    pub fn withdraw(&mut self) -> Result<()> {
        if matches!(self.status, ProposalStatus::Passed | ProposalStatus::Rejected) {
            anyhow::bail!("Cannot withdraw completed proposals");
        }
        
        self.status = ProposalStatus::Withdrawn;
        Ok(())
    }
}

/// Manager for handling proposals
pub struct ProposalManager {
    proposals: Arc<RwLock<HashMap<Uuid, Proposal>>>,
}

impl ProposalManager {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submit a new proposal
    pub async fn submit(&self, mut proposal: Proposal) -> Result<Uuid> {
        proposal.start_voting()?;
        
        let mut proposals = self.proposals.write().await;
        let id = proposal.id;
        proposals.insert(id, proposal);
        
        Ok(id)
    }

    /// Get a proposal by ID
    pub async fn get(&self, id: Uuid) -> Option<Proposal> {
        let proposals = self.proposals.read().await;
        proposals.get(&id).cloned()
    }

    /// Get all active proposals
    pub async fn get_active(&self) -> Vec<Proposal> {
        let proposals = self.proposals.read().await;
        proposals.values()
            .filter(|p| p.is_active())
            .cloned()
            .collect()
    }

    /// Update expired proposals
    pub async fn update_expired(&self) -> Result<Vec<Uuid>> {
        let mut proposals = self.proposals.write().await;
        let mut expired = Vec::new();
        
        for (id, proposal) in proposals.iter_mut() {
            if proposal.status == ProposalStatus::Voting && proposal.is_expired() {
                proposal.status = ProposalStatus::Expired;
                expired.push(*id);
            }
        }
        
        Ok(expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_builder() {
        let proposal = ProposalBuilder::new(
            "Test Proposal".to_string(),
            "test-agent".to_string(),
            ProposalType::TaskDelegation,
        )
        .description("This is a test proposal".to_string())
        .required_consensus(ConsensusType::SuperMajority)
        .voting_duration(600)
        .build();
        
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.proposer, "test-agent");
        assert_eq!(proposal.required_consensus, ConsensusType::SuperMajority);
        assert_eq!(proposal.status, ProposalStatus::Draft);
    }

    #[test]
    fn test_agent_extension_proposal() {
        let data = AgentExtensionProposalData {
            agent_id: "frontend-agent".to_string(),
            extension_type: ExtensionType::Capability,
            current_capabilities: vec!["React".to_string()],
            proposed_capabilities: vec!["React Server Components".to_string()],
            implementation_plan: ImplementationPlan {
                phases: vec![],
                timeline: "2 weeks".to_string(),
            },
            risk_assessment: RiskAssessment {
                risks: vec![],
                overall_risk_level: RiskLevel::Low,
            },
            success_criteria: vec!["Performance improvement".to_string()],
        };
        
        let proposal = Proposal::agent_extension(
            "Add RSC support".to_string(),
            "frontend-agent".to_string(),
            data,
        ).unwrap();
        
        assert_eq!(proposal.proposal_type, ProposalType::AgentExtension);
        assert_eq!(proposal.required_consensus, ConsensusType::SimpleMajority);
    }
}