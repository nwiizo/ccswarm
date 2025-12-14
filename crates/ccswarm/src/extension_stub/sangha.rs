use serde::{Deserialize, Serialize};

/// Vote structure for Sangha consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub proposal_id: String,
    pub vote_type: VoteType,
    pub reason: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    Approve,
    Reject,
    Abstain,
    NeedsChanges,
}

impl Vote {
    pub fn new(voter_id: String, proposal_id: String, vote_type: VoteType) -> Self {
        Self {
            voter_id,
            proposal_id,
            vote_type,
            reason: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}
