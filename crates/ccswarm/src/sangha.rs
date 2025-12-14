use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::CCSwarmError;

/// Sangha - collective intelligence system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sangha {
    pub members: Vec<SanghaMember>,
    pub votes: HashMap<String, Vec<Vote>>,
    pub consensus_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaMember {
    pub id: String,
    pub role: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub member_id: String,
    pub value: VoteValue,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Sangha {
    pub fn new(consensus_threshold: f32) -> Self {
        Self {
            members: Vec::new(),
            votes: HashMap::new(),
            consensus_threshold,
        }
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
            total_weight / max_weight
        })
    }
}

impl Default for Sangha {
    fn default() -> Self {
        Self::new(0.66) // 2/3 majority by default
    }
}
