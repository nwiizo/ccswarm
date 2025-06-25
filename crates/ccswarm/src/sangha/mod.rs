//! Sangha - Collective decision-making mechanism for agents
//! 
//! This module implements the Sangha system, which enables agents to make
//! collective decisions through consensus mechanisms inspired by Buddhist
//! philosophical principles.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use uuid::Uuid;

pub mod consensus;
pub mod doctrine;
pub mod proposal;
pub mod session;
pub mod voting;

use crate::identity::AgentRole;

/// Represents the Sangha - the collective decision-making body
#[derive(Debug)]
pub struct Sangha {
    /// Unique identifier for this Sangha instance
    id: Uuid,
    /// Active members (agents) in the Sangha
    members: Arc<RwLock<HashMap<String, SanghaMember>>>,
    /// Current active proposals
    proposals: Arc<RwLock<HashMap<Uuid, Proposal>>>,
    /// Consensus mechanism
    consensus: Box<dyn ConsensusAlgorithm>,
    /// Sangha configuration
    config: SanghaConfig,
    /// Session manager for meetings
    session_manager: session::SessionManager,
    /// Vote storage (proposal_id -> votes)
    votes: Arc<RwLock<HashMap<Uuid, Vec<Vote>>>>,
    /// Storage path for persistence
    storage_path: std::path::PathBuf,
}

/// Configuration for the Sangha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaConfig {
    /// Minimum number of agents required for quorum
    pub quorum_threshold: usize,
    /// Percentage of votes needed for simple majority
    pub simple_majority: f64,
    /// Percentage of votes needed for supermajority
    pub super_majority: f64,
    /// Duration in seconds for voting rounds
    pub voting_duration_secs: u64,
    /// Whether to allow proxy voting
    pub allow_proxy_voting: bool,
    /// Maximum number of active proposals
    pub max_active_proposals: usize,
}

impl Default for SanghaConfig {
    fn default() -> Self {
        Self {
            quorum_threshold: 3,
            simple_majority: 0.51,
            super_majority: 0.67,
            voting_duration_secs: 300, // 5 minutes
            allow_proxy_voting: false,
            max_active_proposals: 10,
        }
    }
}

/// Represents a member of the Sangha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaMember {
    /// Agent ID
    pub agent_id: String,
    /// Agent's role
    pub role: AgentRole,
    /// When the member joined the Sangha
    pub joined_at: DateTime<Utc>,
    /// Voting power (default 1.0)
    pub voting_power: f64,
    /// Whether the member is currently active
    pub is_active: bool,
    /// Member's reputation score
    pub reputation: f64,
}

/// Trait for consensus algorithms
pub trait ConsensusAlgorithm: Send + Sync + std::fmt::Debug {
    /// Calculate consensus based on votes
    fn calculate_consensus(&self, votes: &[Vote]) -> ConsensusResult;
    
    /// Validate a proposal before voting
    fn validate_proposal(&self, proposal: &Proposal) -> Result<()>;
    
    /// Get the algorithm name
    fn name(&self) -> &str;
}

/// Result of consensus calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Whether consensus was reached
    pub reached: bool,
    /// Type of consensus (if reached)
    pub consensus_type: Option<ConsensusType>,
    /// Percentage of agreement
    pub agreement_percentage: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusType {
    /// Unanimous agreement
    Unanimous,
    /// Supermajority agreement
    SuperMajority,
    /// Simple majority
    SimpleMajority,
    /// No consensus reached
    NoConsensus,
}

/// Represents a proposal in the Sangha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: Uuid,
    /// Type of proposal
    pub proposal_type: ProposalType,
    /// Title of the proposal
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Agent who proposed
    pub proposer: String,
    /// When the proposal was created
    pub created_at: DateTime<Utc>,
    /// When voting ends
    pub voting_deadline: DateTime<Utc>,
    /// Current status
    pub status: ProposalStatus,
    /// Required consensus level
    pub required_consensus: ConsensusType,
    /// Additional data specific to proposal type
    pub data: serde_json::Value,
}

/// Types of proposals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    /// Doctrine change proposal
    Doctrine,
    /// Agent extension proposal
    AgentExtension,
    /// System extension proposal
    SystemExtension,
    /// Task delegation proposal
    TaskDelegation,
    /// Resource allocation proposal
    ResourceAllocation,
    /// Emergency action proposal
    Emergency,
}

/// Status of a proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is being drafted
    Draft,
    /// Open for voting
    Voting,
    /// Voting completed, calculating results
    Tallying,
    /// Proposal passed
    Passed,
    /// Proposal rejected
    Rejected,
    /// Proposal withdrawn
    Withdrawn,
    /// Proposal expired
    Expired,
}

/// Represents a vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter's agent ID
    pub voter_id: String,
    /// Proposal being voted on
    pub proposal_id: Uuid,
    /// The vote choice
    pub choice: VoteChoice,
    /// Optional reason for the vote
    pub reason: Option<String>,
    /// When the vote was cast
    pub cast_at: DateTime<Utc>,
    /// Vote weight (based on voting power)
    pub weight: f64,
}

/// Possible vote choices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteChoice {
    /// In favor
    Aye,
    /// Against
    Nay,
    /// Abstain from voting
    Abstain,
    /// Veto (if allowed)
    Veto,
}

impl Sangha {
    /// Create a new Sangha instance
    pub fn new(config: SanghaConfig) -> Result<Self> {
        let storage_path = std::path::PathBuf::from("sangha_storage");
        std::fs::create_dir_all(&storage_path)?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            members: Arc::new(RwLock::new(HashMap::new())),
            proposals: Arc::new(RwLock::new(HashMap::new())),
            consensus: Box::new(consensus::SimpleConsensus::new()),
            config,
            session_manager: session::SessionManager::new(),
            votes: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        })
    }

    /// Add a new member to the Sangha
    pub async fn add_member(&self, member: SanghaMember) -> Result<()> {
        let mut members = self.members.write().await;
        members.insert(member.agent_id.clone(), member);
        Ok(())
    }

    /// Remove a member from the Sangha
    pub async fn remove_member(&self, agent_id: &str) -> Result<()> {
        let mut members = self.members.write().await;
        members.remove(agent_id);
        Ok(())
    }

    /// Submit a new proposal
    pub async fn submit_proposal(&self, proposal: Proposal) -> Result<Uuid> {
        // Validate proposal
        self.consensus.validate_proposal(&proposal)
            .context("Failed to validate proposal")?;

        let mut proposals = self.proposals.write().await;
        
        // Check if we've reached max proposals
        let active_count = proposals.values()
            .filter(|p| matches!(p.status, ProposalStatus::Voting | ProposalStatus::Draft))
            .count();
            
        if active_count >= self.config.max_active_proposals {
            anyhow::bail!("Maximum number of active proposals reached");
        }

        let proposal_id = proposal.id;
        proposals.insert(proposal_id, proposal);
        
        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(&self, vote: Vote) -> Result<()> {
        let proposals = self.proposals.read().await;
        let proposal = proposals.get(&vote.proposal_id)
            .context("Proposal not found")?;
            
        if proposal.status != ProposalStatus::Voting {
            anyhow::bail!("Proposal is not open for voting");
        }
        
        if proposal.voting_deadline < Utc::now() {
            anyhow::bail!("Voting deadline has passed");
        }
        
        // Store vote 
        let vote = Vote {
            voter_id: voter_id.to_string(),
            proposal_id,
            choice,
            reason: reason.map(String::from),
            cast_at: Utc::now(),
            weight: self.get_voting_weight(voter_id).await,
        };
        
        // Store vote in memory
        let mut votes = self.votes.write().await;
        votes.entry(proposal_id)
            .or_insert_with(Vec::new)
            .push(vote.clone());
        
        // Persist vote to disk
        self.persist_vote(&vote).await?;
        
        tracing::info!("Vote cast by {} for proposal {}: {:?}", voter_id, proposal_id, choice);
        
        Ok(())
    }

    /// Calculate consensus for a proposal
    pub async fn calculate_consensus(&self, proposal_id: Uuid) -> Result<ConsensusResult> {
        // Get all votes for the proposal
        let votes = self.votes.read().await
            .get(&proposal_id)
            .cloned()
            .unwrap_or_default();
        
        let result = self.consensus.calculate_consensus(&votes);
        
        // Update proposal status based on result
        let mut proposals = self.proposals.write().await;
        if let Some(proposal) = proposals.get_mut(&proposal_id) {
            proposal.status = if result.reached {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected
            };
        }
        
        Ok(result)
    }

    /// Get voting weight for an agent
    async fn get_voting_weight(&self, voter_id: &str) -> f64 {
        // Base weight is 1.0, can be adjusted based on agent reputation, role, etc.
        let members = self.members.read().await;
        if let Some(member) = members.get(voter_id) {
            match member.role {
                AgentRole::Frontend | AgentRole::Backend | AgentRole::DevOps | AgentRole::QA => 1.0,
            }
        } else {
            0.5 // Non-member has reduced voting weight
        }
    }

    /// Persist vote to disk
    async fn persist_vote(&self, vote: &Vote) -> Result<()> {
        let vote_file = self.storage_path.join(format!("vote_{}.json", Uuid::new_v4()));
        let vote_json = serde_json::to_string_pretty(vote)?;
        fs::write(vote_file, vote_json).await?;
        Ok(())
    }

    /// Load all votes from disk
    pub async fn load_votes(&self) -> Result<()> {
        let mut votes = self.votes.write().await;
        
        let mut entries = fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name.starts_with("vote_") {
                        if let Ok(content) = fs::read_to_string(&path).await {
                            if let Ok(vote) = serde_json::from_str::<Vote>(&content) {
                                votes.entry(vote.proposal_id)
                                    .or_insert_with(Vec::new)
                                    .push(vote);
                            }
                        }
                    }
                }
            }
        }
        
        tracing::info!("Loaded {} proposal vote sets from storage", votes.len());
        Ok(())
    }

    /// Get all votes for a proposal
    pub async fn get_votes(&self, proposal_id: Uuid) -> Vec<Vote> {
        self.votes.read().await
            .get(&proposal_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get vote statistics for a proposal
    pub async fn get_vote_stats(&self, proposal_id: Uuid) -> VoteStats {
        let votes = self.get_votes(proposal_id).await;
        let mut stats = VoteStats::default();
        
        for vote in votes {
            match vote.choice {
                VoteChoice::Aye => stats.aye += 1,
                VoteChoice::Nay => stats.nay += 1,
                VoteChoice::Abstain => stats.abstain += 1,
                VoteChoice::Veto => stats.veto += 1,
            }
            stats.total_weight += vote.weight;
        }
        
        stats.total = stats.aye + stats.nay + stats.abstain + stats.veto;
        stats
    }

    /// Get current Sangha statistics
    pub async fn get_stats(&self) -> SanghaStats {
        let members = self.members.read().await;
        let proposals = self.proposals.read().await;
        
        SanghaStats {
            total_members: members.len(),
            active_members: members.values().filter(|m| m.is_active).count(),
            total_proposals: proposals.len(),
            active_proposals: proposals.values()
                .filter(|p| matches!(p.status, ProposalStatus::Voting | ProposalStatus::Draft))
                .count(),
            consensus_algorithm: self.consensus.name().to_string(),
        }
    }
}

/// Vote statistics for a proposal
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoteStats {
    pub aye: usize,
    pub nay: usize,
    pub abstain: usize,
    pub veto: usize,
    pub total: usize,
    pub total_weight: f64,
}

/// Statistics about the Sangha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaStats {
    pub total_members: usize,
    pub active_members: usize,
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub consensus_algorithm: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sangha_creation() {
        let config = SanghaConfig::default();
        let sangha = Sangha::new(config).unwrap();
        
        let stats = sangha.get_stats().await;
        assert_eq!(stats.total_members, 0);
        assert_eq!(stats.total_proposals, 0);
    }

    #[tokio::test]
    async fn test_member_management() {
        let sangha = Sangha::new(SanghaConfig::default()).unwrap();
        
        let member = SanghaMember {
            agent_id: "test-agent".to_string(),
            role: AgentRole::Frontend {
                technologies: vec!["React".to_string()],
                responsibilities: vec!["UI".to_string()],
                boundaries: vec!["Client-side".to_string()],
            },
            joined_at: Utc::now(),
            voting_power: 1.0,
            is_active: true,
            reputation: 1.0,
        };
        
        sangha.add_member(member.clone()).await.unwrap();
        
        let stats = sangha.get_stats().await;
        assert_eq!(stats.total_members, 1);
        assert_eq!(stats.active_members, 1);
        
        sangha.remove_member(&member.agent_id).await.unwrap();
        
        let stats = sangha.get_stats().await;
        assert_eq!(stats.total_members, 0);
    }
}