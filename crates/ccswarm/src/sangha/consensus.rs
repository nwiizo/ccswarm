//! Consensus algorithms for Sangha decision-making

use super::*;
use std::collections::HashMap;

/// Simple majority consensus algorithm
#[derive(Debug)]
pub struct SimpleConsensus {
    config: ConsensusConfig,
}

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub simple_majority_threshold: f64,
    pub super_majority_threshold: f64,
    pub unanimous_threshold: f64,
    pub minimum_participation: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            simple_majority_threshold: 0.51,
            super_majority_threshold: 0.67,
            unanimous_threshold: 0.95,   // Allow for small margin
            minimum_participation: 0.50, // At least 50% must vote
        }
    }
}

impl SimpleConsensus {
    pub fn new() -> Self {
        Self {
            config: ConsensusConfig::default(),
        }
    }

    pub fn with_config(config: ConsensusConfig) -> Self {
        Self { config }
    }

    fn calculate_vote_percentages(&self, votes: &[Vote]) -> VotePercentages {
        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let mut counts = HashMap::new();

        for vote in votes {
            *counts.entry(vote.choice).or_insert(0.0) += vote.weight;
        }

        VotePercentages {
            aye: counts.get(&VoteChoice::Aye).copied().unwrap_or(0.0) / total_weight,
            nay: counts.get(&VoteChoice::Nay).copied().unwrap_or(0.0) / total_weight,
            abstain: counts.get(&VoteChoice::Abstain).copied().unwrap_or(0.0) / total_weight,
            veto: counts.get(&VoteChoice::Veto).copied().unwrap_or(0.0) / total_weight,
            total_weight,
        }
    }
}

impl ConsensusAlgorithm for SimpleConsensus {
    fn calculate_consensus(&self, votes: &[Vote]) -> ConsensusResult {
        if votes.is_empty() {
            return ConsensusResult {
                reached: false,
                consensus_type: Some(ConsensusType::NoConsensus),
                agreement_percentage: 0.0,
                metadata: HashMap::new(),
            };
        }

        let percentages = self.calculate_vote_percentages(votes);

        // Check for veto (if any veto exists, no consensus)
        if percentages.veto > 0.0 {
            return ConsensusResult {
                reached: false,
                consensus_type: Some(ConsensusType::NoConsensus),
                agreement_percentage: percentages.aye,
                metadata: [
                    ("reason".to_string(), "veto_exercised".to_string()),
                    (
                        "veto_percentage".to_string(),
                        format!("{:.2}%", percentages.veto * 100.0),
                    ),
                ]
                .into(),
            };
        }

        // Calculate agreement percentage (Aye / (Aye + Nay))
        let decisive_votes = percentages.aye + percentages.nay;
        if decisive_votes == 0.0 {
            // All abstained
            return ConsensusResult {
                reached: false,
                consensus_type: Some(ConsensusType::NoConsensus),
                agreement_percentage: 0.0,
                metadata: [("reason".to_string(), "all_abstained".to_string())].into(),
            };
        }

        let agreement_percentage = percentages.aye / decisive_votes;

        // Determine consensus type
        let consensus_type = if agreement_percentage >= self.config.unanimous_threshold {
            ConsensusType::Unanimous
        } else if agreement_percentage >= self.config.super_majority_threshold {
            ConsensusType::SuperMajority
        } else if agreement_percentage >= self.config.simple_majority_threshold {
            ConsensusType::SimpleMajority
        } else {
            ConsensusType::NoConsensus
        };

        let reached = matches!(
            consensus_type,
            ConsensusType::Unanimous | ConsensusType::SuperMajority | ConsensusType::SimpleMajority
        );

        ConsensusResult {
            reached,
            consensus_type: Some(consensus_type),
            agreement_percentage,
            metadata: [
                (
                    "aye_percentage".to_string(),
                    format!("{:.2}%", percentages.aye * 100.0),
                ),
                (
                    "nay_percentage".to_string(),
                    format!("{:.2}%", percentages.nay * 100.0),
                ),
                (
                    "abstain_percentage".to_string(),
                    format!("{:.2}%", percentages.abstain * 100.0),
                ),
                ("total_votes".to_string(), votes.len().to_string()),
            ]
            .into(),
        }
    }

    fn validate_proposal(&self, _proposal: &Proposal) -> Result<()> {
        // Basic validation
        Ok(())
    }

    fn name(&self) -> &str {
        "Simple Consensus"
    }
}

/// Byzantine Fault Tolerant consensus algorithm
#[derive(Debug)]
pub struct ByzantineConsensus {
    fault_tolerance: f64,
}

impl ByzantineConsensus {
    pub fn new(fault_tolerance: f64) -> Self {
        Self { fault_tolerance }
    }
}

impl ConsensusAlgorithm for ByzantineConsensus {
    fn calculate_consensus(&self, votes: &[Vote]) -> ConsensusResult {
        // Byzantine consensus requires > 2/3 agreement
        let required_percentage = 2.0 / 3.0;

        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let aye_weight: f64 = votes
            .iter()
            .filter(|v| v.choice == VoteChoice::Aye)
            .map(|v| v.weight)
            .sum();

        let agreement_percentage = if total_weight > 0.0 {
            aye_weight / total_weight
        } else {
            0.0
        };

        let reached = agreement_percentage > required_percentage;

        ConsensusResult {
            reached,
            consensus_type: Some(if reached {
                ConsensusType::SuperMajority
            } else {
                ConsensusType::NoConsensus
            }),
            agreement_percentage,
            metadata: [
                ("algorithm".to_string(), "byzantine".to_string()),
                (
                    "fault_tolerance".to_string(),
                    format!("{:.2}%", self.fault_tolerance * 100.0),
                ),
            ]
            .into(),
        }
    }

    fn validate_proposal(&self, _proposal: &Proposal) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Byzantine Fault Tolerant"
    }
}

/// Proof of Stake consensus algorithm
#[derive(Debug)]
pub struct ProofOfStakeConsensus {
    minimum_stake: f64,
}

impl ProofOfStakeConsensus {
    pub fn new(minimum_stake: f64) -> Self {
        Self { minimum_stake }
    }
}

impl ConsensusAlgorithm for ProofOfStakeConsensus {
    fn calculate_consensus(&self, votes: &[Vote]) -> ConsensusResult {
        // Filter votes by minimum stake (reputation)
        let valid_votes: Vec<&Vote> = votes
            .iter()
            .filter(|v| v.weight >= self.minimum_stake)
            .collect();

        if valid_votes.is_empty() {
            return ConsensusResult {
                reached: false,
                consensus_type: Some(ConsensusType::NoConsensus),
                agreement_percentage: 0.0,
                metadata: [("reason".to_string(), "no_valid_stakes".to_string())].into(),
            };
        }

        let total_stake: f64 = valid_votes.iter().map(|v| v.weight).sum();
        let aye_stake: f64 = valid_votes
            .iter()
            .filter(|v| v.choice == VoteChoice::Aye)
            .map(|v| v.weight)
            .sum();

        let agreement_percentage = aye_stake / total_stake;
        let reached = agreement_percentage > 0.5;

        ConsensusResult {
            reached,
            consensus_type: Some(if reached {
                ConsensusType::SimpleMajority
            } else {
                ConsensusType::NoConsensus
            }),
            agreement_percentage,
            metadata: [
                ("algorithm".to_string(), "proof_of_stake".to_string()),
                ("minimum_stake".to_string(), self.minimum_stake.to_string()),
                ("valid_votes".to_string(), valid_votes.len().to_string()),
            ]
            .into(),
        }
    }

    fn validate_proposal(&self, _proposal: &Proposal) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Proof of Stake"
    }
}

#[derive(Debug)]
struct VotePercentages {
    aye: f64,
    nay: f64,
    abstain: f64,
    veto: f64,
    total_weight: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_vote(choice: VoteChoice, weight: f64) -> Vote {
        Vote {
            voter_id: Uuid::new_v4().to_string(),
            proposal_id: Uuid::new_v4(),
            choice,
            reason: None,
            cast_at: Utc::now(),
            weight,
        }
    }

    #[test]
    fn test_simple_consensus_unanimous() {
        let consensus = SimpleConsensus::new();
        let votes = vec![
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Aye, 1.0),
        ];

        let result = consensus.calculate_consensus(&votes);
        assert!(result.reached);
        assert_eq!(result.consensus_type, Some(ConsensusType::Unanimous));
    }

    #[test]
    fn test_simple_consensus_majority() {
        let consensus = SimpleConsensus::new();
        let votes = vec![
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Nay, 1.0),
        ];

        let result = consensus.calculate_consensus(&votes);
        assert!(result.reached);
        // 2/3 = 66.67% which is less than 67% threshold, so it's SimpleMajority
        assert_eq!(result.consensus_type, Some(ConsensusType::SimpleMajority));
    }

    #[test]
    fn test_simple_consensus_veto() {
        let consensus = SimpleConsensus::new();
        let votes = vec![
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Aye, 1.0),
            create_vote(VoteChoice::Veto, 1.0),
        ];

        let result = consensus.calculate_consensus(&votes);
        assert!(!result.reached);
        assert_eq!(result.consensus_type, Some(ConsensusType::NoConsensus));
    }
}
