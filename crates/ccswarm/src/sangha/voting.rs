//! Voting mechanism for Sangha proposals

use super::*;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

/// Manages voting on proposals
pub struct VotingManager {
    /// Votes indexed by proposal ID
    votes: Arc<Mutex<HashMap<Uuid, Vec<Vote>>>>,
    /// Track who has voted on what to prevent double voting
    voter_records: Arc<Mutex<HashMap<Uuid, HashSet<String>>>>,
    /// Voting rules
    rules: VotingRules,
}

/// Rules governing the voting process
#[derive(Debug, Clone)]
pub struct VotingRules {
    /// Allow changing vote before deadline
    pub allow_vote_change: bool,
    /// Allow proxy voting
    pub allow_proxy: bool,
    /// Allow anonymous voting
    pub allow_anonymous: bool,
    /// Minimum reputation to vote
    pub minimum_reputation: f64,
    /// Whether abstentions count toward quorum
    pub abstentions_count_for_quorum: bool,
}

impl Default for VotingRules {
    fn default() -> Self {
        Self {
            allow_vote_change: true,
            allow_proxy: false,
            allow_anonymous: false,
            minimum_reputation: 0.0,
            abstentions_count_for_quorum: true,
        }
    }
}

impl VotingManager {
    pub fn new(rules: VotingRules) -> Self {
        Self {
            votes: Arc::new(Mutex::new(HashMap::new())),
            voter_records: Arc::new(Mutex::new(HashMap::new())),
            rules,
        }
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(&self, vote: Vote, voter: &SanghaMember) -> Result<()> {
        // Validate voter eligibility
        if voter.reputation < self.rules.minimum_reputation {
            anyhow::bail!(
                "Voter reputation {} is below minimum required {}",
                voter.reputation,
                self.rules.minimum_reputation
            );
        }

        if !voter.is_active {
            anyhow::bail!("Inactive members cannot vote");
        }

        let mut votes = self.votes.lock().await;
        let mut voter_records = self.voter_records.lock().await;

        // Check if voter has already voted
        let proposal_voters = voter_records
            .entry(vote.proposal_id)
            .or_insert_with(HashSet::new);

        if proposal_voters.contains(&vote.voter_id) && !self.rules.allow_vote_change {
            anyhow::bail!("Voter has already voted and vote changes are not allowed");
        }

        // If changing vote, remove the old one
        if proposal_voters.contains(&vote.voter_id) {
            let proposal_votes = votes.get_mut(&vote.proposal_id);
            if let Some(votes_list) = proposal_votes {
                votes_list.retain(|v| v.voter_id != vote.voter_id);
            }
        }

        // Record the vote
        proposal_voters.insert(vote.voter_id.clone());
        votes
            .entry(vote.proposal_id)
            .or_insert_with(Vec::new)
            .push(vote);

        Ok(())
    }

    /// Get all votes for a proposal
    pub async fn get_votes(&self, proposal_id: Uuid) -> Vec<Vote> {
        let votes = self.votes.lock().await;
        votes.get(&proposal_id).cloned().unwrap_or_default()
    }

    /// Get voting statistics for a proposal
    pub async fn get_voting_stats(&self, proposal_id: Uuid) -> VotingStats {
        let votes = self.get_votes(proposal_id).await;

        let mut stats = VotingStats {
            total_votes: votes.len(),
            aye_votes: 0,
            nay_votes: 0,
            abstain_votes: 0,
            veto_votes: 0,
            total_weight: 0.0,
            aye_weight: 0.0,
            nay_weight: 0.0,
            abstain_weight: 0.0,
            veto_weight: 0.0,
            participation_rate: 0.0,
        };

        for vote in &votes {
            stats.total_weight += vote.weight;
            match vote.choice {
                VoteChoice::Aye => {
                    stats.aye_votes += 1;
                    stats.aye_weight += vote.weight;
                }
                VoteChoice::Nay => {
                    stats.nay_votes += 1;
                    stats.nay_weight += vote.weight;
                }
                VoteChoice::Abstain => {
                    stats.abstain_votes += 1;
                    stats.abstain_weight += vote.weight;
                }
                VoteChoice::Veto => {
                    stats.veto_votes += 1;
                    stats.veto_weight += vote.weight;
                }
            }
        }

        stats
    }

    /// Check if quorum is met for a proposal
    pub async fn check_quorum(
        &self,
        proposal_id: Uuid,
        _total_members: usize,
        quorum_threshold: usize,
    ) -> bool {
        let voter_records = self.voter_records.lock().await;
        let voters = voter_records.get(&proposal_id);

        let vote_count = voters.map(|v| v.len()).unwrap_or(0);

        if self.rules.abstentions_count_for_quorum {
            vote_count >= quorum_threshold
        } else {
            // Need to check actual votes
            let votes = self.get_votes(proposal_id).await;
            let decisive_votes = votes
                .iter()
                .filter(|v| !matches!(v.choice, VoteChoice::Abstain))
                .count();
            decisive_votes >= quorum_threshold
        }
    }

    /// Get a summary of all votes with reasons
    pub async fn get_vote_summary(&self, proposal_id: Uuid) -> VoteSummary {
        let votes = self.get_votes(proposal_id).await;

        let mut reasons_by_choice: HashMap<VoteChoice, Vec<String>> = HashMap::new();

        for vote in &votes {
            if let Some(reason) = &vote.reason {
                reasons_by_choice
                    .entry(vote.choice)
                    .or_insert_with(Vec::new)
                    .push(reason.clone());
            }
        }

        VoteSummary {
            proposal_id,
            total_votes: votes.len(),
            votes_by_choice: votes.iter().map(|v| (v.choice, v.clone())).fold(
                HashMap::new(),
                |mut acc, (choice, vote)| {
                    acc.entry(choice).or_insert_with(Vec::new).push(vote);
                    acc
                },
            ),
            reasons_by_choice,
            voting_complete: false, // Would need proposal status to determine
        }
    }

    /// Delegate voting power (proxy voting)
    pub async fn delegate_vote(&self, _from: &str, _to: &str, _proposal_id: Uuid) -> Result<()> {
        if !self.rules.allow_proxy {
            anyhow::bail!("Proxy voting is not allowed");
        }

        // Implementation would involve tracking delegations
        // and applying them when calculating results

        Ok(())
    }
}

/// Statistics about voting on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStats {
    pub total_votes: usize,
    pub aye_votes: usize,
    pub nay_votes: usize,
    pub abstain_votes: usize,
    pub veto_votes: usize,
    pub total_weight: f64,
    pub aye_weight: f64,
    pub nay_weight: f64,
    pub abstain_weight: f64,
    pub veto_weight: f64,
    pub participation_rate: f64,
}

/// Summary of all votes on a proposal
#[derive(Debug, Clone)]
pub struct VoteSummary {
    pub proposal_id: Uuid,
    pub total_votes: usize,
    pub votes_by_choice: HashMap<VoteChoice, Vec<Vote>>,
    pub reasons_by_choice: HashMap<VoteChoice, Vec<String>>,
    pub voting_complete: bool,
}

/// Ballot for batch voting on multiple proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ballot {
    pub voter_id: String,
    pub votes: Vec<BallotEntry>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotEntry {
    pub proposal_id: Uuid,
    pub choice: VoteChoice,
    pub reason: Option<String>,
}

impl VotingManager {
    /// Submit a ballot with multiple votes
    pub async fn submit_ballot(
        &self,
        ballot: Ballot,
        voter: &SanghaMember,
    ) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();

        for entry in ballot.votes {
            let vote = Vote {
                voter_id: ballot.voter_id.clone(),
                proposal_id: entry.proposal_id,
                choice: entry.choice,
                reason: entry.reason,
                cast_at: ballot.submitted_at,
                weight: voter.voting_power,
            };

            results.push(self.cast_vote(vote, voter).await);
        }

        Ok(results)
    }
}

/// Voting power calculator
pub struct VotingPowerCalculator {
    base_power: f64,
    reputation_multiplier: f64,
    tenure_multiplier: f64,
}

impl VotingPowerCalculator {
    pub fn new() -> Self {
        Self {
            base_power: 1.0,
            reputation_multiplier: 0.5,
            tenure_multiplier: 0.1,
        }
    }

    /// Calculate voting power for a member
    pub fn calculate(&self, member: &SanghaMember) -> f64 {
        let tenure_days = (Utc::now() - member.joined_at).num_days() as f64;
        let tenure_bonus = (tenure_days / 30.0) * self.tenure_multiplier; // Bonus per month

        self.base_power + (member.reputation * self.reputation_multiplier) + tenure_bonus.min(1.0)
        // Cap tenure bonus at 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_voting_manager() {
        let rules = VotingRules::default();
        let manager = VotingManager::new(rules);

        let member = SanghaMember {
            agent_id: "test-agent".to_string(),
            role: AgentRole::Backend {
                technologies: vec!["Rust".to_string()],
                responsibilities: vec!["API".to_string()],
                boundaries: vec!["Server-side".to_string()],
            },
            joined_at: Utc::now(),
            voting_power: 1.0,
            is_active: true,
            reputation: 1.0,
        };

        let vote = Vote {
            voter_id: member.agent_id.clone(),
            proposal_id: Uuid::new_v4(),
            choice: VoteChoice::Aye,
            reason: Some("Good proposal".to_string()),
            cast_at: Utc::now(),
            weight: member.voting_power,
        };

        manager.cast_vote(vote.clone(), &member).await.unwrap();

        let votes = manager.get_votes(vote.proposal_id).await;
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].choice, VoteChoice::Aye);
    }

    #[tokio::test]
    async fn test_voting_stats() {
        let rules = VotingRules::default();
        let manager = VotingManager::new(rules);

        let proposal_id = Uuid::new_v4();
        let votes = vec![
            (VoteChoice::Aye, 1.0),
            (VoteChoice::Aye, 1.5),
            (VoteChoice::Nay, 1.0),
            (VoteChoice::Abstain, 0.5),
        ];

        for (i, (choice, weight)) in votes.iter().enumerate() {
            let member = SanghaMember {
                agent_id: format!("agent-{}", i),
                role: AgentRole::Backend {
                    technologies: vec!["Rust".to_string()],
                    responsibilities: vec!["API".to_string()],
                    boundaries: vec!["Server-side".to_string()],
                },
                joined_at: Utc::now(),
                voting_power: *weight,
                is_active: true,
                reputation: 1.0,
            };

            let vote = Vote {
                voter_id: member.agent_id.clone(),
                proposal_id,
                choice: *choice,
                reason: None,
                cast_at: Utc::now(),
                weight: *weight,
            };

            manager.cast_vote(vote, &member).await.unwrap();
        }

        let stats = manager.get_voting_stats(proposal_id).await;
        assert_eq!(stats.total_votes, 4);
        assert_eq!(stats.aye_votes, 2);
        assert_eq!(stats.nay_votes, 1);
        assert_eq!(stats.abstain_votes, 1);
        assert_eq!(stats.aye_weight, 2.5);
    }
}
