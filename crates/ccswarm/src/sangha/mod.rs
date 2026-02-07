//! Sangha - Collective Intelligence and Democratic Decision Making
//!
//! Implements consensus-based decision making for agent swarms.

mod algorithms;
mod persistence;
mod proposal;

pub use algorithms::{
    BftConsensus, ConsensusAlgorithm, ProofOfStakeConsensus, SimpleMajorityConsensus,
};
pub use persistence::SanghaPersistence;
pub use proposal::{Proposal, ProposalManager, ProposalStatus};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::CCSwarmError;

/// Sangha - collective intelligence system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sangha {
    pub members: Vec<SanghaMember>,
    pub votes: HashMap<String, Vec<Vote>>,
    pub consensus_threshold: f32,
    pub algorithm: ConsensusAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaMember {
    pub id: String,
    pub role: String,
    pub weight: f32,
    pub joined_at: DateTime<Utc>,
    pub contribution_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub member_id: String,
    pub value: VoteValue,
    pub confidence: f32,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteValue {
    Approve,
    Reject,
    Abstain,
}

#[async_trait]
pub trait SanghaConsensus {
    async fn propose(&mut self, proposal: &str) -> Result<String, CCSwarmError>;
    async fn vote(&mut self, proposal_id: &str, vote: Vote) -> Result<(), CCSwarmError>;
    async fn check_consensus(&self, proposal_id: &str) -> Result<bool, CCSwarmError>;
}

impl SanghaMember {
    pub fn new(id: impl Into<String>, role: impl Into<String>, weight: f32) -> Self {
        Self {
            id: id.into(),
            role: role.into(),
            weight,
            joined_at: Utc::now(),
            contribution_score: 0.0,
        }
    }
}

impl Vote {
    pub fn new(member_id: impl Into<String>, value: VoteValue, confidence: f32) -> Self {
        Self {
            member_id: member_id.into(),
            value,
            confidence,
            reason: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

impl Sangha {
    pub fn new(consensus_threshold: f32) -> Self {
        Self {
            members: Vec::new(),
            votes: HashMap::new(),
            consensus_threshold,
            algorithm: ConsensusAlgorithm::SimpleMajority,
        }
    }

    pub fn with_algorithm(mut self, algorithm: ConsensusAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn add_member(&mut self, member: SanghaMember) {
        self.members.push(member);
    }

    pub fn calculate_consensus(&self, proposal_id: &str) -> Option<f32> {
        self.votes.get(proposal_id).map(|votes| {
            let total_weight: f32 = votes
                .iter()
                .filter_map(|v| {
                    self.members
                        .iter()
                        .find(|m| m.id == v.member_id)
                        .map(|m| match v.value {
                            VoteValue::Approve => m.weight * v.confidence,
                            VoteValue::Reject => -m.weight * v.confidence,
                            VoteValue::Abstain => 0.0,
                        })
                })
                .sum();

            let max_weight: f32 = self.members.iter().map(|m| m.weight).sum();
            if max_weight > 0.0 {
                total_weight / max_weight
            } else {
                0.0
            }
        })
    }
}

impl Default for Sangha {
    fn default() -> Self {
        Self::new(0.66) // 2/3 majority by default
    }
}

#[async_trait]
impl SanghaConsensus for Sangha {
    async fn propose(&mut self, proposal_text: &str) -> Result<String, CCSwarmError> {
        let proposal = Proposal::new(
            proposal_text,
            proposal_text,
            "general",
            "system",
            24, // 24 hour voting window
        );
        let id = proposal.id.clone();
        self.votes.insert(id.clone(), Vec::new());
        Ok(id)
    }

    async fn vote(&mut self, proposal_id: &str, vote: Vote) -> Result<(), CCSwarmError> {
        let votes = self.votes.entry(proposal_id.to_string()).or_default();

        // Reject duplicate votes
        if votes.iter().any(|v| v.member_id == vote.member_id) {
            return Err(CCSwarmError::Agent {
                agent_id: vote.member_id.clone(),
                message: format!(
                    "Member {} has already voted on proposal {}",
                    vote.member_id, proposal_id
                ),
                source: None,
            });
        }

        votes.push(vote);
        Ok(())
    }

    async fn check_consensus(&self, proposal_id: &str) -> Result<bool, CCSwarmError> {
        let votes = self
            .votes
            .get(proposal_id)
            .ok_or_else(|| CCSwarmError::Agent {
                agent_id: "sangha".to_string(),
                message: format!("Proposal {} not found", proposal_id),
                source: None,
            })?;

        let member_weights: HashMap<String, f32> = self
            .members
            .iter()
            .map(|m| (m.id.clone(), m.weight))
            .collect();

        let contribution_scores: HashMap<String, f32> = self
            .members
            .iter()
            .map(|m| (m.id.clone(), m.contribution_score))
            .collect();

        let result = match self.algorithm {
            ConsensusAlgorithm::SimpleMajority => {
                SimpleMajorityConsensus::check(votes, &member_weights)
            }
            ConsensusAlgorithm::Bft => BftConsensus::check(votes, &member_weights),
            ConsensusAlgorithm::ProofOfStake => ProofOfStakeConsensus::check(
                votes,
                &member_weights,
                &contribution_scores,
                self.consensus_threshold,
            ),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sangha_default() {
        let sangha = Sangha::default();
        assert!((sangha.consensus_threshold - 0.66).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vote_with_reason() {
        let vote = Vote::new("agent-1", VoteValue::Approve, 0.9)
            .with_reason("Strong evidence supports this");
        assert_eq!(vote.member_id, "agent-1");
        assert!(vote.reason.is_some());
    }

    #[test]
    fn test_consensus_calculation() {
        let mut sangha = Sangha::new(0.5);
        sangha.add_member(SanghaMember::new("a1", "backend", 1.0));
        sangha.add_member(SanghaMember::new("a2", "frontend", 1.0));

        sangha.votes.insert(
            "prop-1".to_string(),
            vec![
                Vote::new("a1", VoteValue::Approve, 1.0),
                Vote::new("a2", VoteValue::Approve, 1.0),
            ],
        );

        let consensus = sangha.calculate_consensus("prop-1");
        assert!(consensus.is_some());
        assert!((consensus.unwrap() - 1.0).abs() < f32::EPSILON);
    }
}
