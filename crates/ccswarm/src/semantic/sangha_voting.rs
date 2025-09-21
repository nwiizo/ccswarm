//! Sangha semantic voting system
//!
//! Democratic decision-making for code improvements using semantic analysis

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::analyzer::{ImpactAnalysis, SemanticAnalyzer, Symbol};
use super::memory::{Memory, MemoryType, ProjectMemory};
use super::refactoring_system::RefactoringProposal;
use super::subagent_integration::AgentRole;
use super::{SemanticError, SemanticResult};

/// Voting proposal with semantic context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub semantic_context: SemanticContext,
    pub impact_analysis: ImpactAnalysis,
    pub votes: HashMap<String, Vote>,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
    pub quorum_required: usize,
    pub approval_threshold: f64,
}

/// Type of proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Refactoring(RefactoringProposal),
    ArchitectureChange,
    PatternAdoption,
    DependencyUpdate,
    SecurityFix,
    PerformanceOptimization,
    TechnicalDebt,
    FeatureAddition,
}

/// Semantic context for informed voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub affected_symbols: Vec<Symbol>,
    pub code_metrics: CodeMetrics,
    pub historical_changes: Vec<HistoricalChange>,
    pub similar_past_proposals: Vec<String>,
    pub expert_analysis: HashMap<String, String>,
}

/// Code metrics for decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub complexity: usize,
    pub coupling: f64,
    pub cohesion: f64,
    pub test_coverage: f64,
    pub technical_debt_hours: f64,
    pub maintainability_index: f64,
}

/// Historical change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalChange {
    pub date: DateTime<Utc>,
    pub description: String,
    pub impact: String,
    pub success: bool,
}

/// Individual vote with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub voter_role: AgentRole,
    pub decision: VoteDecision,
    pub reasoning: String,
    pub confidence: f64,
    pub semantic_evidence: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Vote decision
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
    RequestChanges,
}

/// Proposal status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    UnderReview,
    Voting,
    Approved,
    Rejected,
    Implemented,
    Cancelled,
}

/// Consensus algorithm type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConsensusAlgorithm {
    SimpleMajority,   // > 50%
    Supermajority,    // > 66%
    UnanimousConsent, // 100%
    WeightedVoting,   // Based on expertise
    ByzantineFault,   // Byzantine fault tolerance
}

/// Voting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    pub proposal_id: String,
    pub total_votes: usize,
    pub approve_votes: usize,
    pub reject_votes: usize,
    pub abstain_votes: usize,
    pub request_changes_votes: usize,
    pub approval_percentage: f64,
    pub consensus_achieved: bool,
    pub final_decision: VoteDecision,
    pub implementation_recommendations: Vec<String>,
}

/// Agent expertise for weighted voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExpertise {
    pub agent_name: String,
    pub role: AgentRole,
    pub domain_expertise: HashMap<String, f64>,
    pub past_voting_accuracy: f64,
    pub contribution_score: f64,
}

/// Sangha semantic voting system
pub struct SanghaSemanticVoting {
    analyzer: Arc<SemanticAnalyzer>,
    memory: Arc<ProjectMemory>,
    proposals: Arc<RwLock<HashMap<String, SemanticProposal>>>,
    agent_expertise: Arc<RwLock<HashMap<String, AgentExpertise>>>,
    voting_history: Arc<RwLock<Vec<VotingResult>>>,
    consensus_algorithm: ConsensusAlgorithm,
}

impl SanghaSemanticVoting {
    /// Create a new Sangha voting system
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        memory: Arc<ProjectMemory>,
        consensus_algorithm: ConsensusAlgorithm,
    ) -> Self {
        Self {
            analyzer,
            memory,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            agent_expertise: Arc::new(RwLock::new(HashMap::new())),
            voting_history: Arc::new(RwLock::new(Vec::new())),
            consensus_algorithm,
        }
    }

    /// Create a new proposal with semantic analysis
    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        affected_symbols: Vec<Symbol>,
    ) -> SemanticResult<SemanticProposal> {
        // Analyze impact
        let impact_analysis = self
            .analyze_proposal_impact(&proposal_type, &affected_symbols)
            .await?;

        // Gather semantic context
        let semantic_context = self.gather_semantic_context(&affected_symbols).await?;

        // Set voting parameters based on proposal type
        let (quorum_required, approval_threshold) =
            self.determine_voting_parameters(&proposal_type);

        let proposal = SemanticProposal {
            id: format!("proposal_{}", Utc::now().timestamp()),
            title,
            description,
            proposal_type,
            semantic_context,
            impact_analysis,
            votes: HashMap::new(),
            status: ProposalStatus::Draft,
            created_at: Utc::now(),
            voting_deadline: Utc::now() + Duration::days(3),
            quorum_required,
            approval_threshold,
        };

        // Store proposal
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal.id.clone(), proposal.clone());

        // Store in memory
        self.store_proposal_in_memory(&proposal).await?;

        Ok(proposal)
    }

    /// Submit a vote with semantic reasoning
    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter: String,
        voter_role: AgentRole,
        decision: VoteDecision,
        reasoning: String,
    ) -> SemanticResult<()> {
        let mut proposals = self.proposals.write().await;

        if let Some(proposal) = proposals.get_mut(proposal_id) {
            // Check if voting is open
            if proposal.status != ProposalStatus::Voting {
                return Err(SemanticError::Other(
                    "Proposal not open for voting".to_string(),
                ));
            }

            // Check deadline
            if Utc::now() > proposal.voting_deadline {
                return Err(SemanticError::Other(
                    "Voting deadline has passed".to_string(),
                ));
            }

            // Generate semantic evidence
            let semantic_evidence = self.generate_semantic_evidence(proposal, &decision).await?;

            // Calculate confidence based on expertise
            let confidence = self
                .calculate_vote_confidence(&voter, &proposal.proposal_type)
                .await?;

            let vote = Vote {
                voter: voter.clone(),
                voter_role,
                decision,
                reasoning,
                confidence,
                semantic_evidence,
                timestamp: Utc::now(),
            };

            proposal.votes.insert(voter, vote);

            // Check if consensus is reached
            if proposal.votes.len() >= proposal.quorum_required {
                self.evaluate_consensus(proposal_id).await?;
            }

            Ok(())
        } else {
            Err(SemanticError::Other(format!(
                "Proposal {} not found",
                proposal_id
            )))
        }
    }

    /// Analyze proposal impact
    async fn analyze_proposal_impact(
        &self,
        proposal_type: &ProposalType,
        affected_symbols: &[Symbol],
    ) -> SemanticResult<ImpactAnalysis> {
        // Simplified impact analysis
        let severity = match proposal_type {
            ProposalType::SecurityFix => super::analyzer::ImpactSeverity::High,
            ProposalType::ArchitectureChange => super::analyzer::ImpactSeverity::High,
            ProposalType::PerformanceOptimization => super::analyzer::ImpactSeverity::Medium,
            _ => super::analyzer::ImpactSeverity::Low,
        };

        let suggested_actions = vec![
            "Review all affected code".to_string(),
            "Run comprehensive tests".to_string(),
            "Update documentation".to_string(),
        ];

        Ok(ImpactAnalysis {
            change: super::analyzer::SymbolChange {
                symbol: affected_symbols.first().cloned().unwrap_or_else(|| Symbol {
                    name: "unknown".to_string(),
                    path: "unknown".to_string(),
                    kind: super::analyzer::SymbolKind::Other("unknown".to_string()),
                    file_path: String::new(),
                    line: 0,
                    body: None,
                    references: Vec::new(),
                    metadata: HashMap::new(),
                }),
                change_type: super::analyzer::ChangeType::Modified,
                old_value: None,
                new_value: None,
            },
            affected_symbols: affected_symbols.to_vec(),
            severity,
            suggested_actions,
        })
    }

    /// Gather semantic context for informed voting
    async fn gather_semantic_context(
        &self,
        affected_symbols: &[Symbol],
    ) -> SemanticResult<SemanticContext> {
        // Calculate code metrics
        let code_metrics = self.calculate_code_metrics(affected_symbols).await?;

        // Get historical changes
        let historical_changes = self.get_historical_changes(affected_symbols).await?;

        // Find similar past proposals
        let similar_proposals = self.find_similar_proposals(affected_symbols).await?;

        // Generate expert analysis
        let expert_analysis = self
            .generate_expert_analysis(affected_symbols, &code_metrics)
            .await?;

        Ok(SemanticContext {
            affected_symbols: affected_symbols.to_vec(),
            code_metrics,
            historical_changes,
            similar_past_proposals: similar_proposals,
            expert_analysis,
        })
    }

    /// Calculate code metrics
    async fn calculate_code_metrics(&self, symbols: &[Symbol]) -> SemanticResult<CodeMetrics> {
        let mut total_complexity = 0;
        let mut total_lines = 0;

        for symbol in symbols {
            if let Some(ref body) = symbol.body {
                total_lines += body.lines().count();
                // Simplified complexity calculation
                total_complexity += body.matches("if ").count()
                    + body.matches("match ").count()
                    + body.matches("while ").count()
                    + body.matches("for ").count();
            }
        }

        Ok(CodeMetrics {
            complexity: total_complexity,
            coupling: 0.5,      // Simplified
            cohesion: 0.7,      // Simplified
            test_coverage: 0.8, // Simplified
            technical_debt_hours: (total_complexity as f64) * 0.5,
            maintainability_index: 100.0 - (total_complexity as f64),
        })
    }

    /// Get historical changes
    async fn get_historical_changes(
        &self,
        symbols: &[Symbol],
    ) -> SemanticResult<Vec<HistoricalChange>> {
        // In real implementation, would query version control
        Ok(vec![HistoricalChange {
            date: Utc::now() - Duration::days(30),
            description: "Previous refactoring".to_string(),
            impact: "Improved performance by 20%".to_string(),
            success: true,
        }])
    }

    /// Find similar past proposals
    async fn find_similar_proposals(&self, symbols: &[Symbol]) -> SemanticResult<Vec<String>> {
        let history = self.voting_history.read().await;
        let mut similar = Vec::new();

        // Find proposals that affected similar symbols
        for result in history.iter() {
            // Simplified similarity check
            similar.push(result.proposal_id.clone());
            if similar.len() >= 3 {
                break;
            }
        }

        Ok(similar)
    }

    /// Generate expert analysis
    async fn generate_expert_analysis(
        &self,
        symbols: &[Symbol],
        metrics: &CodeMetrics,
    ) -> SemanticResult<HashMap<String, String>> {
        let mut analysis = HashMap::new();

        if metrics.complexity > 20 {
            analysis.insert(
                "complexity".to_string(),
                "High complexity detected. Consider breaking down into smaller functions."
                    .to_string(),
            );
        }

        if metrics.test_coverage < 0.8 {
            analysis.insert(
                "testing".to_string(),
                "Test coverage below 80%. Additional tests recommended.".to_string(),
            );
        }

        if metrics.technical_debt_hours > 10.0 {
            analysis.insert(
                "debt".to_string(),
                format!(
                    "Technical debt estimated at {:.1} hours",
                    metrics.technical_debt_hours
                ),
            );
        }

        Ok(analysis)
    }

    /// Determine voting parameters
    fn determine_voting_parameters(&self, proposal_type: &ProposalType) -> (usize, f64) {
        match proposal_type {
            ProposalType::SecurityFix => (3, 0.51), // Lower threshold for security
            ProposalType::ArchitectureChange => (5, 0.75), // Higher threshold for architecture
            ProposalType::FeatureAddition => (4, 0.66),
            _ => (3, 0.66),
        }
    }

    /// Generate semantic evidence for a vote
    async fn generate_semantic_evidence(
        &self,
        proposal: &SemanticProposal,
        decision: &VoteDecision,
    ) -> SemanticResult<Vec<String>> {
        let mut evidence = Vec::new();

        match decision {
            VoteDecision::Approve => {
                if proposal.semantic_context.code_metrics.complexity > 15 {
                    evidence.push("Will reduce code complexity".to_string());
                }
                if proposal.semantic_context.code_metrics.test_coverage < 0.8 {
                    evidence.push("Improves testability".to_string());
                }
            }
            VoteDecision::Reject => {
                if proposal.semantic_context.code_metrics.maintainability_index > 80.0 {
                    evidence.push("Current code is already maintainable".to_string());
                }
                if proposal.impact_analysis.severity == super::analyzer::ImpactSeverity::High {
                    evidence.push("High risk with many affected components".to_string());
                }
            }
            _ => {}
        }

        Ok(evidence)
    }

    /// Calculate vote confidence based on expertise
    async fn calculate_vote_confidence(
        &self,
        voter: &str,
        proposal_type: &ProposalType,
    ) -> SemanticResult<f64> {
        let expertise = self.agent_expertise.read().await;

        if let Some(agent_expertise) = expertise.get(voter) {
            // Calculate confidence based on domain expertise
            let domain = match proposal_type {
                ProposalType::Refactoring(_) => "refactoring",
                ProposalType::SecurityFix => "security",
                ProposalType::PerformanceOptimization => "performance",
                _ => "general",
            };

            let domain_score = agent_expertise.domain_expertise.get(domain).unwrap_or(&0.5);
            let accuracy_score = agent_expertise.past_voting_accuracy;

            Ok((domain_score + accuracy_score) / 2.0)
        } else {
            Ok(0.5) // Default confidence
        }
    }

    /// Evaluate consensus
    async fn evaluate_consensus(&self, proposal_id: &str) -> SemanticResult<VotingResult> {
        let mut proposals = self.proposals.write().await;

        if let Some(proposal) = proposals.get_mut(proposal_id) {
            let total_votes = proposal.votes.len();
            let mut approve_votes = 0;
            let mut reject_votes = 0;
            let mut abstain_votes = 0;
            let mut request_changes_votes = 0;

            // Count votes with weighting if applicable
            for vote in proposal.votes.values() {
                let weight = if self.consensus_algorithm == ConsensusAlgorithm::WeightedVoting {
                    vote.confidence
                } else {
                    1.0
                };

                match vote.decision {
                    VoteDecision::Approve => approve_votes += 1,
                    VoteDecision::Reject => reject_votes += 1,
                    VoteDecision::Abstain => abstain_votes += 1,
                    VoteDecision::RequestChanges => request_changes_votes += 1,
                }
            }

            let approval_percentage = (approve_votes as f64) / (total_votes as f64);
            let consensus_achieved = approval_percentage >= proposal.approval_threshold;

            let final_decision = if consensus_achieved {
                VoteDecision::Approve
            } else if reject_votes > approve_votes {
                VoteDecision::Reject
            } else {
                VoteDecision::RequestChanges
            };

            // Update proposal status
            proposal.status = if consensus_achieved {
                ProposalStatus::Approved
            } else {
                ProposalStatus::Rejected
            };

            // Generate implementation recommendations
            let implementation_recommendations = self
                .generate_implementation_recommendations(proposal)
                .await?;

            let result = VotingResult {
                proposal_id: proposal_id.to_string(),
                total_votes,
                approve_votes,
                reject_votes,
                abstain_votes,
                request_changes_votes,
                approval_percentage: approval_percentage * 100.0,
                consensus_achieved,
                final_decision,
                implementation_recommendations,
            };

            // Store result
            let mut history = self.voting_history.write().await;
            history.push(result.clone());

            // Update agent expertise based on outcome
            self.update_agent_expertise(proposal, &result).await?;

            Ok(result)
        } else {
            Err(SemanticError::Other(format!(
                "Proposal {} not found",
                proposal_id
            )))
        }
    }

    /// Generate implementation recommendations
    async fn generate_implementation_recommendations(
        &self,
        proposal: &SemanticProposal,
    ) -> SemanticResult<Vec<String>> {
        let mut recommendations = Vec::new();

        // Add recommendations based on votes
        for vote in proposal.votes.values() {
            if vote.decision == VoteDecision::RequestChanges && !vote.reasoning.is_empty() {
                recommendations.push(format!("Consider: {}", vote.reasoning));
            }
        }

        // Add recommendations based on metrics
        if proposal.semantic_context.code_metrics.test_coverage < 0.8 {
            recommendations
                .push("Ensure comprehensive test coverage before implementation".to_string());
        }

        if proposal.impact_analysis.severity == super::analyzer::ImpactSeverity::High {
            recommendations.push("Implement in phases with monitoring".to_string());
        }

        Ok(recommendations)
    }

    /// Update agent expertise based on voting outcome
    async fn update_agent_expertise(
        &self,
        proposal: &SemanticProposal,
        result: &VotingResult,
    ) -> SemanticResult<()> {
        let mut expertise = self.agent_expertise.write().await;

        for (voter_name, vote) in &proposal.votes {
            let agent_expertise =
                expertise
                    .entry(voter_name.clone())
                    .or_insert_with(|| AgentExpertise {
                        agent_name: voter_name.clone(),
                        role: vote.voter_role.clone(),
                        domain_expertise: HashMap::new(),
                        past_voting_accuracy: 0.5,
                        contribution_score: 0.0,
                    });

            // Update accuracy based on whether they voted with consensus
            let voted_correctly = (vote.decision == VoteDecision::Approve
                && result.consensus_achieved)
                || (vote.decision == VoteDecision::Reject && !result.consensus_achieved);

            if voted_correctly {
                agent_expertise.past_voting_accuracy =
                    (agent_expertise.past_voting_accuracy * 0.9) + 0.1;
            } else {
                agent_expertise.past_voting_accuracy = agent_expertise.past_voting_accuracy * 0.9;
            }

            // Update contribution score
            agent_expertise.contribution_score += vote.confidence;
        }

        Ok(())
    }

    /// Store proposal in memory
    async fn store_proposal_in_memory(&self, proposal: &SemanticProposal) -> SemanticResult<()> {
        let memory = Memory {
            id: proposal.id.clone(),
            name: format!("Sangha Proposal: {}", proposal.title),
            content: serde_json::to_string(proposal)?,
            memory_type: MemoryType::Other("SanghaProposal".to_string()),
            related_symbols: proposal
                .semantic_context
                .affected_symbols
                .iter()
                .map(|s| s.path.clone())
                .collect(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), format!("{:?}", proposal.proposal_type));
                meta.insert("status".to_string(), format!("{:?}", proposal.status));
                meta
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.memory.store_memory(memory).await
    }

    /// Get active proposals
    pub async fn get_active_proposals(&self) -> SemanticResult<Vec<SemanticProposal>> {
        let proposals = self.proposals.read().await;
        Ok(proposals
            .values()
            .filter(|p| {
                p.status == ProposalStatus::Voting || p.status == ProposalStatus::UnderReview
            })
            .cloned()
            .collect())
    }

    /// Get voting history
    pub async fn get_voting_history(&self) -> SemanticResult<Vec<VotingResult>> {
        let history = self.voting_history.read().await;
        Ok(history.clone())
    }
}
