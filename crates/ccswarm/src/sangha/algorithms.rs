//! Consensus algorithms for Sangha voting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Vote, VoteValue};

/// Available consensus algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusAlgorithm {
    /// Simple majority: 51% approval
    SimpleMajority,
    /// Byzantine Fault Tolerant: 67% approval
    Bft,
    /// Proof of Stake: weighted by contribution score
    ProofOfStake,
}

/// Simple majority consensus (51%)
pub struct SimpleMajorityConsensus;

impl SimpleMajorityConsensus {
    pub fn check(votes: &[Vote], member_weights: &HashMap<String, f32>) -> bool {
        Self::approval_ratio(votes, member_weights) > 0.51
    }

    pub fn approval_ratio(votes: &[Vote], member_weights: &HashMap<String, f32>) -> f32 {
        let mut approve = 0.0f32;
        let mut total = 0.0f32;

        for vote in votes {
            let weight = member_weights.get(&vote.member_id).copied().unwrap_or(1.0);
            total += weight;
            match vote.value {
                VoteValue::Approve => approve += weight * vote.confidence,
                VoteValue::Reject => {}
                VoteValue::Abstain => {
                    total -= weight; // Abstentions don't count toward total
                }
            }
        }

        if total > 0.0 { approve / total } else { 0.0 }
    }
}

/// Byzantine Fault Tolerant consensus (67%)
pub struct BftConsensus;

impl BftConsensus {
    pub fn check(votes: &[Vote], member_weights: &HashMap<String, f32>) -> bool {
        Self::approval_ratio(votes, member_weights) >= 0.67
    }

    pub fn approval_ratio(votes: &[Vote], member_weights: &HashMap<String, f32>) -> f32 {
        let mut approve = 0.0f32;
        let mut total = 0.0f32;

        for vote in votes {
            let weight = member_weights.get(&vote.member_id).copied().unwrap_or(1.0);
            total += weight;
            if vote.value == VoteValue::Approve {
                approve += weight * vote.confidence;
            }
        }

        if total > 0.0 { approve / total } else { 0.0 }
    }
}

/// Proof of Stake consensus (weighted by contribution)
pub struct ProofOfStakeConsensus;

impl ProofOfStakeConsensus {
    pub fn check(
        votes: &[Vote],
        member_weights: &HashMap<String, f32>,
        contribution_scores: &HashMap<String, f32>,
        threshold: f32,
    ) -> bool {
        Self::approval_ratio(votes, member_weights, contribution_scores) >= threshold
    }

    pub fn approval_ratio(
        votes: &[Vote],
        member_weights: &HashMap<String, f32>,
        contribution_scores: &HashMap<String, f32>,
    ) -> f32 {
        let mut approve = 0.0f32;
        let mut total = 0.0f32;

        for vote in votes {
            let base_weight = member_weights.get(&vote.member_id).copied().unwrap_or(1.0);
            let contribution = contribution_scores
                .get(&vote.member_id)
                .copied()
                .unwrap_or(0.0);
            // Stake = base weight * (1 + contribution score)
            let stake = base_weight * (1.0 + contribution);
            total += stake;
            if vote.value == VoteValue::Approve {
                approve += stake * vote.confidence;
            }
        }

        if total > 0.0 { approve / total } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_weights() -> HashMap<String, f32> {
        let mut w = HashMap::new();
        w.insert("a1".to_string(), 1.0);
        w.insert("a2".to_string(), 1.0);
        w.insert("a3".to_string(), 1.0);
        w
    }

    #[test]
    fn test_simple_majority() {
        let weights = make_weights();
        let votes = vec![
            Vote::new("a1", VoteValue::Approve, 1.0),
            Vote::new("a2", VoteValue::Approve, 1.0),
            Vote::new("a3", VoteValue::Reject, 1.0),
        ];
        assert!(SimpleMajorityConsensus::check(&votes, &weights));
    }

    #[test]
    fn test_simple_majority_fails() {
        let weights = make_weights();
        let votes = vec![
            Vote::new("a1", VoteValue::Approve, 1.0),
            Vote::new("a2", VoteValue::Reject, 1.0),
            Vote::new("a3", VoteValue::Reject, 1.0),
        ];
        assert!(!SimpleMajorityConsensus::check(&votes, &weights));
    }

    #[test]
    fn test_bft_needs_supermajority() {
        let weights = make_weights();
        // 2/3 = 0.666 < 0.67, should fail
        let votes = vec![
            Vote::new("a1", VoteValue::Approve, 1.0),
            Vote::new("a2", VoteValue::Approve, 1.0),
            Vote::new("a3", VoteValue::Reject, 1.0),
        ];
        assert!(!BftConsensus::check(&votes, &weights));
    }

    #[test]
    fn test_bft_passes_with_all() {
        let weights = make_weights();
        let votes = vec![
            Vote::new("a1", VoteValue::Approve, 1.0),
            Vote::new("a2", VoteValue::Approve, 1.0),
            Vote::new("a3", VoteValue::Approve, 1.0),
        ];
        assert!(BftConsensus::check(&votes, &weights));
    }

    #[test]
    fn test_proof_of_stake() {
        let weights = make_weights();
        let mut contributions = HashMap::new();
        contributions.insert("a1".to_string(), 5.0); // High contributor
        contributions.insert("a2".to_string(), 0.0);
        contributions.insert("a3".to_string(), 0.0);

        // a1 has high stake, their approve should dominate
        let votes = vec![
            Vote::new("a1", VoteValue::Approve, 1.0),
            Vote::new("a2", VoteValue::Reject, 1.0),
            Vote::new("a3", VoteValue::Reject, 1.0),
        ];
        assert!(ProofOfStakeConsensus::check(
            &votes,
            &weights,
            &contributions,
            0.5
        ));
    }
}
