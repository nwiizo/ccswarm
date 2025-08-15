/// Sangha semantic voting system - Optimized version
use super::common::{MetricsCollector, MetricType};
use crate::semantic::{
    analyzer::{SemanticAnalyzer, Symbol},
    memory::{Memory, MemoryType, ProjectMemory},
    refactoring_system::RefactoringProposal,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Define AgentRole locally for semantic voting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentRole {
    Frontend,
    Backend,
    DevOps,
    QA,
    Semantic,
}
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    SimpleMajority,
    Supermajority,
    Unanimous,
    WeightedConsensus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    Refactoring(RefactoringProposal),
    ArchitectureChange,
    PolicyUpdate,
    AgentAddition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
    RequestChanges,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    UnderReview,
    Voting,
    Approved,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub related_symbols: Vec<Symbol>,
    pub status: ProposalStatus,
    pub votes: Vec<Vote>,
    pub quorum_required: usize,
    pub voting_deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub voter_role: AgentRole,
    pub decision: VoteDecision,
    pub reasoning: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    pub proposal_id: String,
    pub final_decision: VoteDecision,
    pub approval_percentage: f64,
    pub consensus_achieved: bool,
    pub implementation_recommendations: Vec<String>,
}

/// Unified voting handler
struct VotingHandler {
    analyzer: Arc<SemanticAnalyzer>,
    memory: Arc<ProjectMemory>,
    metrics: Arc<RwLock<MetricsCollector>>,
}

impl VotingHandler {
    async fn process_vote(&self, proposal: &Proposal, vote: &Vote) -> Result<f64> {
        // Generic vote processing logic
        let weight = self.calculate_vote_weight(&vote.voter_role, &proposal.proposal_type);
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.update(MetricType::Analysis, 1);
        
        Ok(weight)
    }
    
    fn calculate_vote_weight(&self, role: &AgentRole, proposal_type: &ProposalType) -> f64 {
        // Unified weight calculation
        match (role, proposal_type) {
            (AgentRole::Frontend, ProposalType::ArchitectureChange) => 0.8,
            (AgentRole::Backend, ProposalType::ArchitectureChange) => 0.9,
            _ => 1.0,
        }
    }
    
    async fn store_result(&self, proposal: &Proposal, result: &VotingResult) -> Result<()> {
        let memory = Memory {
            id: format!("voting_{}", proposal.id),
            name: format!("Voting: {}", proposal.title),
            content: serde_json::to_string(result)?,
            memory_type: MemoryType::Decision,
            related_symbols: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };
        
        self.memory.store_memory(memory).await?;
        Ok(())
    }
}

pub struct SanghaSemanticVoting {
    proposals: Arc<RwLock<Vec<Proposal>>>,
    consensus_algorithm: ConsensusAlgorithm,
    handler: Arc<VotingHandler>,
}

impl SanghaSemanticVoting {
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        memory: Arc<ProjectMemory>,
        consensus_algorithm: ConsensusAlgorithm,
    ) -> Self {
        Self {
            proposals: Arc::new(RwLock::new(Vec::new())),
            consensus_algorithm,
            handler: Arc::new(VotingHandler {
                analyzer,
                memory,
                metrics: Arc::new(RwLock::new(MetricsCollector::default())),
            }),
        }
    }

    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        related_symbols: Vec<Symbol>,
    ) -> Result<Proposal> {
        let proposal = Proposal {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            proposal_type,
            related_symbols,
            status: ProposalStatus::Draft,
            votes: Vec::new(),
            quorum_required: self.calculate_quorum(),
            voting_deadline: Utc::now() + chrono::Duration::days(7),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let mut proposals = self.proposals.write().await;
        proposals.push(proposal.clone());
        
        Ok(proposal)
    }

    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter_id: String,
        voter_role: AgentRole,
        decision: VoteDecision,
        reasoning: String,
    ) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        
        if let Some(proposal) = proposals.iter_mut().find(|p| p.id == proposal_id) {
            let vote = Vote {
                voter_id,
                voter_role,
                decision,
                reasoning,
                confidence: 0.8,
                timestamp: Utc::now(),
            };
            
            self.handler.process_vote(proposal, &vote).await?;
            proposal.votes.push(vote);
            proposal.updated_at = Utc::now();
        }
        
        Ok(())
    }

    pub async fn evaluate_consensus(&self, proposal_id: &str) -> Result<VotingResult> {
        let proposals = self.proposals.read().await;
        let proposal = proposals
            .iter()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;
        
        let (approvals, total) = self.count_votes(&proposal.votes);
        let approval_percentage = (approvals as f64 / total as f64) * 100.0;
        
        let consensus_achieved = match self.consensus_algorithm {
            ConsensusAlgorithm::SimpleMajority => approval_percentage > 50.0,
            ConsensusAlgorithm::Supermajority => approval_percentage >= 66.7,
            ConsensusAlgorithm::Unanimous => approval_percentage == 100.0,
            ConsensusAlgorithm::WeightedConsensus => approval_percentage > 60.0,
        };
        
        let result = VotingResult {
            proposal_id: proposal_id.to_string(),
            final_decision: if consensus_achieved {
                VoteDecision::Approve
            } else {
                VoteDecision::Reject
            },
            approval_percentage,
            consensus_achieved,
            implementation_recommendations: vec![],
        };
        
        self.handler.store_result(proposal, &result).await?;
        
        Ok(result)
    }

    pub async fn get_active_proposals(&self) -> Result<Vec<Proposal>> {
        let proposals = self.proposals.read().await;
        Ok(proposals
            .iter()
            .filter(|p| matches!(p.status, ProposalStatus::Voting | ProposalStatus::UnderReview))
            .cloned()
            .collect())
    }

    pub async fn get_voting_history(&self) -> Result<Vec<VotingResult>> {
        // In real implementation, would fetch from memory
        Ok(vec![])
    }

    fn calculate_quorum(&self) -> usize {
        match self.consensus_algorithm {
            ConsensusAlgorithm::SimpleMajority => 3,
            ConsensusAlgorithm::Supermajority => 5,
            ConsensusAlgorithm::Unanimous => 7,
            ConsensusAlgorithm::WeightedConsensus => 4,
        }
    }

    fn count_votes(&self, votes: &[Vote]) -> (usize, usize) {
        let approvals = votes
            .iter()
            .filter(|v| matches!(v.decision, VoteDecision::Approve))
            .count();
        (approvals, votes.len())
    }
}