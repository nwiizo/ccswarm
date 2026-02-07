//! Proposal lifecycle management

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::algorithms::{
    BftConsensus, ConsensusAlgorithm, ProofOfStakeConsensus, SimpleMajorityConsensus,
};
use super::{Vote, VoteValue};

/// Status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Voting,
    Approved,
    Rejected,
    Expired,
}

/// A proposal submitted to the Sangha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: String,
    pub status: ProposalStatus,
    pub proposer_id: String,
    pub votes: Vec<Vote>,
    pub created_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
    pub quorum: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Proposal {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        proposal_type: impl Into<String>,
        proposer_id: impl Into<String>,
        voting_duration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: description.into(),
            proposal_type: proposal_type.into(),
            status: ProposalStatus::Pending,
            proposer_id: proposer_id.into(),
            votes: Vec::new(),
            created_at: now,
            voting_deadline: now + Duration::hours(voting_duration_hours),
            quorum: 0.5,
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.voting_deadline
    }

    pub fn add_vote(&mut self, vote: Vote) -> Result<(), String> {
        if self.status != ProposalStatus::Voting {
            return Err(format!(
                "Proposal is not in voting status: {:?}",
                self.status
            ));
        }
        if self.is_expired() {
            return Err("Voting deadline has passed".to_string());
        }
        // Check for duplicate votes
        if self.votes.iter().any(|v| v.member_id == vote.member_id) {
            return Err(format!("Member {} has already voted", vote.member_id));
        }
        self.votes.push(vote);
        Ok(())
    }

    pub fn approval_ratio(&self, member_weights: &HashMap<String, f32>) -> f32 {
        let mut approve_weight = 0.0f32;
        let mut total_weight = 0.0f32;

        for vote in &self.votes {
            let weight = member_weights.get(&vote.member_id).copied().unwrap_or(1.0);
            total_weight += weight;
            if vote.value == VoteValue::Approve {
                approve_weight += weight * vote.confidence;
            }
        }

        if total_weight > 0.0 {
            approve_weight / total_weight
        } else {
            0.0
        }
    }
}

/// Manages proposal lifecycle
pub struct ProposalManager {
    proposals: HashMap<String, Proposal>,
    persistence_dir: Option<std::path::PathBuf>,
}

impl ProposalManager {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            persistence_dir: None,
        }
    }

    pub fn with_persistence(mut self, dir: std::path::PathBuf) -> Self {
        self.persistence_dir = Some(dir);
        self
    }

    /// Get the persistence directory if configured
    pub fn persistence_dir(&self) -> Option<&std::path::Path> {
        self.persistence_dir.as_deref()
    }

    pub fn submit(&mut self, mut proposal: Proposal) -> String {
        proposal.status = ProposalStatus::Voting;
        let id = proposal.id.clone();
        self.proposals.insert(id.clone(), proposal);
        id
    }

    pub fn get(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_mut(&mut self, proposal_id: &str) -> Option<&mut Proposal> {
        self.proposals.get_mut(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: &str, vote: Vote) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| format!("Proposal {} not found", proposal_id))?;
        proposal.add_vote(vote)
    }

    pub fn check_and_finalize(
        &mut self,
        proposal_id: &str,
        member_weights: &HashMap<String, f32>,
        threshold: f32,
    ) -> Option<ProposalStatus> {
        self.check_and_finalize_with_algorithm(
            proposal_id,
            member_weights,
            &ConsensusAlgorithm::SimpleMajority,
            threshold,
            &HashMap::new(),
        )
    }

    /// Finalize a proposal using the specified consensus algorithm
    pub fn check_and_finalize_with_algorithm(
        &mut self,
        proposal_id: &str,
        member_weights: &HashMap<String, f32>,
        algorithm: &ConsensusAlgorithm,
        threshold: f32,
        contribution_scores: &HashMap<String, f32>,
    ) -> Option<ProposalStatus> {
        let proposal = self.proposals.get_mut(proposal_id)?;

        let should_finalize = proposal.status == ProposalStatus::Voting
            && (proposal.is_expired() || proposal.votes.len() >= member_weights.len());

        if !should_finalize {
            return None;
        }

        let approved = match algorithm {
            ConsensusAlgorithm::SimpleMajority => {
                SimpleMajorityConsensus::check(&proposal.votes, member_weights)
            }
            ConsensusAlgorithm::Bft => BftConsensus::check(&proposal.votes, member_weights),
            ConsensusAlgorithm::ProofOfStake => ProofOfStakeConsensus::check(
                &proposal.votes,
                member_weights,
                contribution_scores,
                threshold,
            ),
        };

        proposal.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };
        Some(proposal.status.clone())
    }

    pub fn list_active(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| matches!(p.status, ProposalStatus::Pending | ProposalStatus::Voting))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    pub fn expire_overdue(&mut self) -> Vec<String> {
        let mut expired = Vec::new();
        for (id, proposal) in &mut self.proposals {
            if proposal.is_expired() && proposal.status == ProposalStatus::Voting {
                proposal.status = ProposalStatus::Expired;
                expired.push(id.clone());
            }
        }
        expired
    }
}

impl Default for ProposalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_lifecycle() {
        let mut manager = ProposalManager::new();

        let proposal = Proposal::new(
            "Add GraphQL",
            "Add GraphQL support to backend",
            "feature",
            "agent-1",
            24,
        );

        let id = manager.submit(proposal);
        assert!(manager.get(&id).is_some());
        assert_eq!(manager.get(&id).unwrap().status, ProposalStatus::Voting);

        // Vote
        let vote = Vote::new("agent-1", VoteValue::Approve, 1.0);
        assert!(manager.vote(&id, vote).is_ok());

        // Check finalization
        let mut weights = HashMap::new();
        weights.insert("agent-1".to_string(), 1.0);
        let result = manager.check_and_finalize(&id, &weights, 0.5);
        assert_eq!(result, Some(ProposalStatus::Approved));
    }

    #[test]
    fn test_duplicate_vote_rejected() {
        let mut manager = ProposalManager::new();
        let proposal = Proposal::new("Test", "Desc", "feature", "agent-1", 24);
        let id = manager.submit(proposal);

        let vote1 = Vote::new("agent-1", VoteValue::Approve, 1.0);
        assert!(manager.vote(&id, vote1).is_ok());

        let vote2 = Vote::new("agent-1", VoteValue::Reject, 1.0);
        assert!(manager.vote(&id, vote2).is_err());
    }
}
