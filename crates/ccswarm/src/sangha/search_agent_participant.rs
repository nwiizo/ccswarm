//! Search Agent Sangha Participant
//!
//! This module implements autonomous Sangha participation for the Search Agent,
//! enabling it to monitor proposals, conduct research, and cast informed votes.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{ConsensusType, Proposal, ProposalStatus, ProposalType, Sangha, Vote, VoteChoice};
use crate::agent::search_agent::{SearchAgent, SearchFilters, SearchRequest, SearchResult};
use crate::coordination::CoordinationBus;
use crate::identity::AgentRole;

/// Trait for agents that can participate in Sangha decisions
#[async_trait]
pub trait SanghaParticipant: Send + Sync {
    /// Get the participant's agent ID
    fn agent_id(&self) -> &str;

    /// Get the participant's role
    fn role(&self) -> &AgentRole;

    /// Monitor and respond to new proposals
    async fn monitor_proposals(&mut self, sangha: Arc<Sangha>) -> Result<()>;

    /// Analyze a proposal and determine voting position
    async fn analyze_proposal(&mut self, proposal: &Proposal) -> Result<VotingDecision>;

    /// Submit evidence or research findings for a proposal
    async fn submit_evidence(
        &mut self,
        proposal_id: Uuid,
        evidence: Evidence,
        sangha: Arc<Sangha>,
    ) -> Result<()>;

    /// Cast a vote with reasoning
    async fn cast_informed_vote(
        &mut self,
        proposal: &Proposal,
        decision: VotingDecision,
        sangha: Arc<Sangha>,
    ) -> Result<()>;

    /// Propose new initiatives based on identified needs
    async fn propose_initiative(
        &mut self,
        need: IdentifiedNeed,
        sangha: Arc<Sangha>,
    ) -> Result<Uuid>;
}

/// Voting decision with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingDecision {
    pub vote: VoteChoice,
    pub confidence: f64,
    pub reasoning: String,
    pub supporting_evidence: Vec<String>,
    pub concerns: Vec<String>,
}

/// Evidence submitted to support or oppose a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub summary: String,
    pub details: serde_json::Value,
    pub relevance_score: f64,
    pub sources: Vec<String>,
    pub submitted_at: DateTime<Utc>,
}

/// Types of evidence that can be submitted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Research(ResearchEvidence),
    Technical(TechnicalEvidence),
    Historical(HistoricalEvidence),
    Comparative(ComparativeEvidence),
}

/// Research-based evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEvidence {
    pub findings: Vec<String>,
    pub methodology: String,
    pub confidence_level: f64,
}

/// Technical analysis evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalEvidence {
    pub analysis_type: String,
    pub key_points: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Historical precedent evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvidence {
    pub precedents: Vec<String>,
    pub outcomes: Vec<String>,
    pub lessons_learned: Vec<String>,
}

/// Comparative analysis evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeEvidence {
    pub alternatives: Vec<String>,
    pub comparison_criteria: Vec<String>,
    pub recommendation: String,
}

/// Identified need that may require a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedNeed {
    pub need_type: NeedType,
    pub description: String,
    pub urgency: Urgency,
    pub supporting_data: serde_json::Value,
}

/// Types of needs that can be identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeedType {
    KnowledgeGap,
    CapabilityGap,
    ProcessImprovement,
    TechnicalDebt,
    PerformanceOptimization,
    SecurityEnhancement,
}

/// Urgency levels for needs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Urgency {
    Critical,
    High,
    Medium,
    Low,
}

/// Search Agent implementation of SanghaParticipant
pub struct SearchAgentSanghaParticipant {
    search_agent: SearchAgent,
    active_research: Arc<RwLock<HashMap<Uuid, ResearchTask>>>,
    evidence_cache: Arc<RwLock<HashMap<Uuid, Vec<Evidence>>>>,
    monitoring_interval: tokio::time::Duration,
}

/// Active research task for a proposal
#[derive(Debug, Clone)]
struct ResearchTask {
    _proposal_id: Uuid,
    queries: Vec<String>,
    results: Vec<SearchResult>,
    started_at: DateTime<Utc>,
    completed: bool,
}

impl SearchAgentSanghaParticipant {
    /// Create a new Search Agent Sangha participant
    pub fn new(agent_id: String, coordination_bus: Arc<CoordinationBus>) -> Self {
        Self {
            search_agent: SearchAgent::new(agent_id, coordination_bus),
            active_research: Arc::new(RwLock::new(HashMap::new())),
            evidence_cache: Arc::new(RwLock::new(HashMap::new())),
            monitoring_interval: tokio::time::Duration::from_secs(30),
        }
    }

    /// Extract search queries from a proposal
    fn extract_search_queries(&self, proposal: &Proposal) -> Vec<String> {
        let mut queries = Vec::new();

        // Extract from title and description
        queries.push(proposal.title.clone());

        // Extract key terms from description
        let key_terms = self.extract_key_terms(&proposal.description);
        queries.extend(key_terms);

        // Add proposal-type specific queries
        match proposal.proposal_type {
            ProposalType::AgentExtension => {
                queries.push(format!("{} agent capabilities", proposal.title));
                queries.push(format!("{} implementation patterns", proposal.title));
            }
            ProposalType::SystemExtension => {
                queries.push(format!("{} system architecture", proposal.title));
                queries.push(format!("{} best practices", proposal.title));
            }
            ProposalType::Doctrine => {
                queries.push(format!("{} principles", proposal.title));
                queries.push(format!("{} governance models", proposal.title));
            }
            ProposalType::TaskDelegation => {
                queries.push(format!("{} task automation", proposal.title));
                queries.push(format!("{} workflow optimization", proposal.title));
            }
            ProposalType::ResourceAllocation => {
                queries.push(format!("{} resource management", proposal.title));
                queries.push(format!("{} allocation strategies", proposal.title));
            }
            ProposalType::Emergency => {
                queries.push(format!("{} crisis response", proposal.title));
                queries.push(format!("{} emergency procedures", proposal.title));
            }
        }

        queries
    }

    /// Extract key terms from text
    fn extract_key_terms(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction - in production, use NLP
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut terms = Vec::new();

        // Look for technical terms (capitalized words, acronyms)
        for word in &words {
            if (word.chars().all(|c| c.is_uppercase()) && word.len() > 2)
                || (word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    && word.len() > 4)
            {
                terms.push(word.to_string());
            }
        }

        // Look for compound terms
        for window in words.windows(2) {
            if window[0]
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && window[1]
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                terms.push(format!("{} {}", window[0], window[1]));
            }
        }

        terms.dedup();
        terms
    }

    /// Conduct research for a proposal
    async fn conduct_research(&mut self, proposal: &Proposal) -> Result<Vec<SearchResult>> {
        let queries = self.extract_search_queries(proposal);
        let mut all_results = Vec::new();

        info!(
            "Conducting research for proposal '{}' with {} queries",
            proposal.title,
            queries.len()
        );

        // Create research task
        let research_task = ResearchTask {
            _proposal_id: proposal.id,
            queries: queries.clone(),
            results: Vec::new(),
            started_at: Utc::now(),
            completed: false,
        };

        {
            let mut active = self.active_research.write().await;
            active.insert(proposal.id, research_task);
        }

        // Execute searches
        for query in queries {
            let request = SearchRequest {
                requesting_agent: self.search_agent.agent_id.clone(),
                query: query.clone(),
                max_results: Some(5),
                filters: Some(SearchFilters {
                    domains: None,
                    date_range: Some("past year".to_string()),
                    language: Some("en".to_string()),
                    file_type: None,
                }),
                context: Some(format!("Sangha proposal research: {}", proposal.title)),
            };

            match self.search_agent.execute_search(&request).await {
                Ok(results) => {
                    info!("Found {} results for query: {}", results.len(), query);
                    all_results.extend(results);
                }
                Err(e) => {
                    warn!("Search failed for query '{}': {}", query, e);
                }
            }
        }

        // Update research task
        {
            let mut active = self.active_research.write().await;
            if let Some(task) = active.get_mut(&proposal.id) {
                task.results = all_results.clone();
                task.completed = true;
            }
        }

        Ok(all_results)
    }

    /// Analyze search results to form an opinion
    fn analyze_search_results(
        &self,
        proposal: &Proposal,
        results: &[SearchResult],
    ) -> VotingDecision {
        let mut supporting_evidence = Vec::new();
        let mut concerns = Vec::new();
        let mut confidence = 0.5; // Start neutral

        // Get research task to check timing and query relevance
        let research_info = futures::executor::block_on(async {
            self.active_research.read().await.get(&proposal.id).cloned()
        });

        // Count positive and negative indicators
        let mut positive_count = 0;
        let mut negative_count = 0;

        for result in results {
            let snippet_lower = result.snippet.to_lowercase();
            let title_lower = result.title.to_lowercase();

            // Check relevance based on original queries
            let _relevance_score = if let Some(ref task) = research_info {
                task.queries
                    .iter()
                    .filter(|q| {
                        title_lower.contains(&q.to_lowercase())
                            || snippet_lower.contains(&q.to_lowercase())
                    })
                    .count() as f64
                    / task.queries.len() as f64
            } else {
                0.5
            };

            // Look for positive indicators
            if snippet_lower.contains("success")
                || snippet_lower.contains("effective")
                || snippet_lower.contains("improved")
                || snippet_lower.contains("recommended")
            {
                positive_count += 1;
                supporting_evidence.push(format!(
                    "Positive evidence from {}: {}",
                    result.title,
                    result.snippet.chars().take(100).collect::<String>()
                ));
            }

            // Look for negative indicators
            if snippet_lower.contains("failure")
                || snippet_lower.contains("problem")
                || snippet_lower.contains("issue")
                || snippet_lower.contains("risk")
                || snippet_lower.contains("concern")
            {
                negative_count += 1;
                concerns.push(format!(
                    "Concern from {}: {}",
                    result.title,
                    result.snippet.chars().take(100).collect::<String>()
                ));
            }

            // Boost confidence if highly relevant
            if let Some(score) = result.relevance_score {
                if score > 0.8 {
                    confidence += 0.1;
                }
            }
        }

        // Calculate vote based on evidence
        let evidence_ratio = if positive_count + negative_count > 0 {
            positive_count as f64 / (positive_count + negative_count) as f64
        } else {
            0.5
        };

        let vote = if evidence_ratio > 0.7 {
            confidence = f64::min(confidence + 0.2, 0.9);
            VoteChoice::Aye
        } else if evidence_ratio < 0.3 {
            confidence = f64::min(confidence + 0.2, 0.9);
            VoteChoice::Nay
        } else if results.is_empty() {
            confidence = 0.3;
            VoteChoice::Abstain
        } else {
            confidence = 0.5;
            VoteChoice::Abstain
        };

        // Generate reasoning
        let reasoning = format!(
            "Based on {} search results, found {} supporting indicators and {} concerns. \
             Evidence ratio: {:.0}%. Research confidence: {:.0}%",
            results.len(),
            positive_count,
            negative_count,
            evidence_ratio * 100.0,
            confidence * 100.0
        );

        VotingDecision {
            vote,
            confidence,
            reasoning,
            supporting_evidence,
            concerns,
        }
    }

    /// Generate evidence from search results
    fn generate_evidence(&self, proposal: &Proposal, results: &[SearchResult]) -> Evidence {
        let findings: Vec<String> = results
            .iter()
            .take(5)
            .map(|r| {
                format!(
                    "{}: {}",
                    r.title,
                    r.snippet.chars().take(150).collect::<String>()
                )
            })
            .collect();

        let sources: Vec<String> = results.iter().take(5).map(|r| r.url.clone()).collect();

        let relevance_score = results
            .iter()
            .filter_map(|r| r.relevance_score)
            .sum::<f32>()
            / results.len().max(1) as f32;

        Evidence {
            evidence_type: EvidenceType::Research(ResearchEvidence {
                findings: findings.clone(),
                methodology: "Web search and content analysis".to_string(),
                confidence_level: relevance_score as f64,
            }),
            summary: format!(
                "Research findings from {} sources regarding '{}'",
                results.len(),
                proposal.title
            ),
            details: serde_json::json!({
                "total_results": results.len(),
                "top_findings": findings,
                "average_relevance": relevance_score,
                "search_timestamp": Utc::now(),
            }),
            relevance_score: relevance_score as f64,
            sources,
            submitted_at: Utc::now(),
        }
    }

    /// Detect knowledge gaps from monitoring
    async fn detect_knowledge_gaps(&self, sangha: Arc<Sangha>) -> Vec<IdentifiedNeed> {
        let mut needs = Vec::new();

        // Check active proposals for topics without sufficient research
        let proposals = {
            let proposals = sangha.proposals.read().await;
            proposals
                .values()
                .filter(|p| p.status == ProposalStatus::Voting)
                .cloned()
                .collect::<Vec<_>>()
        };

        let evidence_cache = self.evidence_cache.read().await;

        for proposal in proposals {
            let evidence_count = evidence_cache
                .get(&proposal.id)
                .map(|e| e.len())
                .unwrap_or(0);

            if evidence_count < 3 {
                needs.push(IdentifiedNeed {
                    need_type: NeedType::KnowledgeGap,
                    description: format!(
                        "Insufficient research for proposal '{}'. Only {} evidence items found.",
                        proposal.title, evidence_count
                    ),
                    urgency: Urgency::High,
                    supporting_data: serde_json::json!({
                        "proposal_id": proposal.id,
                        "proposal_title": proposal.title,
                        "current_evidence_count": evidence_count,
                        "recommended_minimum": 3,
                    }),
                });
            }
        }

        needs
    }

    /// Get research timing information for a proposal
    pub async fn get_research_timing(&self, proposal_id: Uuid) -> Option<(DateTime<Utc>, bool)> {
        self.active_research
            .read()
            .await
            .get(&proposal_id)
            .map(|task| (task.started_at, task.completed))
    }

    /// Get all queries used for a proposal research
    pub async fn get_research_queries(&self, proposal_id: Uuid) -> Option<Vec<String>> {
        self.active_research
            .read()
            .await
            .get(&proposal_id)
            .map(|task| task.queries.clone())
    }
}

#[async_trait]
impl SanghaParticipant for SearchAgentSanghaParticipant {
    fn agent_id(&self) -> &str {
        &self.search_agent.agent_id
    }

    fn role(&self) -> &AgentRole {
        // Return a static reference to avoid lifetime issues
        // In a real implementation, this would be stored as a field
        static SEARCH_ROLE: std::sync::OnceLock<AgentRole> = std::sync::OnceLock::new();
        SEARCH_ROLE.get_or_init(|| AgentRole::Search {
            technologies: vec![
                "Web Search".to_string(),
                "Information Retrieval".to_string(),
                "Research Analysis".to_string(),
            ],
            responsibilities: vec![
                "Proposal Research".to_string(),
                "Evidence Gathering".to_string(),
                "Knowledge Gap Detection".to_string(),
            ],
            boundaries: vec![
                "Read-only research".to_string(),
                "No implementation".to_string(),
                "No direct voting influence".to_string(),
            ],
        })
    }

    async fn monitor_proposals(&mut self, sangha: Arc<Sangha>) -> Result<()> {
        info!("Search agent starting Sangha proposal monitoring");

        loop {
            // Get active proposals
            let active_proposals = {
                let proposals = sangha.proposals.read().await;
                proposals
                    .values()
                    .filter(|p| p.status == ProposalStatus::Voting)
                    .cloned()
                    .collect::<Vec<_>>()
            };

            info!(
                "Found {} active proposals to research",
                active_proposals.len()
            );

            // Research each proposal
            for proposal in active_proposals {
                // Check if we've already researched this
                let already_researched = {
                    let cache = self.evidence_cache.read().await;
                    cache.contains_key(&proposal.id)
                };

                if !already_researched {
                    info!("Researching proposal: {}", proposal.title);

                    // Conduct research
                    match self.conduct_research(&proposal).await {
                        Ok(results) => {
                            if !results.is_empty() {
                                // Generate and submit evidence
                                let evidence = self.generate_evidence(&proposal, &results);
                                if let Err(e) = self
                                    .submit_evidence(proposal.id, evidence, sangha.clone())
                                    .await
                                {
                                    error!("Failed to submit evidence: {}", e);
                                }

                                // Analyze and vote
                                let decision = self.analyze_search_results(&proposal, &results);
                                if let Err(e) = self
                                    .cast_informed_vote(&proposal, decision, sangha.clone())
                                    .await
                                {
                                    error!("Failed to cast vote: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Research failed for proposal '{}': {}", proposal.title, e);
                        }
                    }
                }
            }

            // Check for knowledge gaps
            let needs = self.detect_knowledge_gaps(sangha.clone()).await;
            for need in needs {
                if let Urgency::High | Urgency::Critical = need.urgency {
                    info!("Identified knowledge gap: {}", need.description);
                    if let Err(e) = self.propose_initiative(need, sangha.clone()).await {
                        error!("Failed to propose initiative: {}", e);
                    }
                }
            }

            // Wait before next monitoring cycle
            tokio::time::sleep(self.monitoring_interval).await;
        }
    }

    async fn analyze_proposal(&mut self, proposal: &Proposal) -> Result<VotingDecision> {
        // Conduct research if not already done
        let results = match self.conduct_research(proposal).await {
            Ok(results) => results,
            Err(e) => {
                warn!("Research failed, abstaining: {}", e);
                return Ok(VotingDecision {
                    vote: VoteChoice::Abstain,
                    confidence: 0.0,
                    reasoning: format!("Unable to conduct research: {}", e),
                    supporting_evidence: vec![],
                    concerns: vec!["Research failure".to_string()],
                });
            }
        };

        Ok(self.analyze_search_results(proposal, &results))
    }

    async fn submit_evidence(
        &mut self,
        proposal_id: Uuid,
        evidence: Evidence,
        _sangha: Arc<Sangha>,
    ) -> Result<()> {
        info!(
            "Submitting evidence for proposal {}: {}",
            proposal_id, evidence.summary
        );

        // Cache the evidence
        {
            let mut cache = self.evidence_cache.write().await;
            cache
                .entry(proposal_id)
                .or_insert_with(Vec::new)
                .push(evidence.clone());
        }

        // In a real implementation, this would submit to Sangha's evidence store
        // For now, we just log it
        info!(
            "Evidence submitted with relevance score: {:.2}",
            evidence.relevance_score
        );

        Ok(())
    }

    async fn cast_informed_vote(
        &mut self,
        proposal: &Proposal,
        decision: VotingDecision,
        sangha: Arc<Sangha>,
    ) -> Result<()> {
        info!(
            "Casting {:?} vote for '{}' with {:.0}% confidence",
            decision.vote,
            proposal.title,
            decision.confidence * 100.0
        );

        let vote = Vote {
            voter_id: self.agent_id().to_string(),
            proposal_id: proposal.id,
            choice: decision.vote,
            reason: Some(decision.reasoning),
            cast_at: Utc::now(),
            weight: decision.confidence, // Use confidence as weight
        };

        sangha
            .cast_vote(vote)
            .await
            .context("Failed to cast vote")?;

        Ok(())
    }

    async fn propose_initiative(
        &mut self,
        need: IdentifiedNeed,
        sangha: Arc<Sangha>,
    ) -> Result<Uuid> {
        let proposal = Proposal {
            id: Uuid::new_v4(),
            proposal_type: match need.need_type {
                NeedType::KnowledgeGap => ProposalType::AgentExtension,
                NeedType::CapabilityGap => ProposalType::SystemExtension,
                NeedType::ProcessImprovement => ProposalType::TaskDelegation,
                NeedType::TechnicalDebt => ProposalType::SystemExtension,
                NeedType::PerformanceOptimization => ProposalType::SystemExtension,
                NeedType::SecurityEnhancement => ProposalType::Emergency,
            },
            title: format!(
                "Address {}: {}",
                match need.need_type {
                    NeedType::KnowledgeGap => "Knowledge Gap",
                    NeedType::CapabilityGap => "Capability Gap",
                    NeedType::ProcessImprovement => "Process Improvement",
                    NeedType::TechnicalDebt => "Technical Debt",
                    NeedType::PerformanceOptimization => "Performance Issue",
                    NeedType::SecurityEnhancement => "Security Gap",
                },
                need.description.chars().take(50).collect::<String>()
            ),
            description: need.description,
            proposer: self.agent_id().to_string(),
            created_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::hours(24),
            status: ProposalStatus::Draft,
            required_consensus: match need.urgency {
                Urgency::Critical => ConsensusType::SimpleMajority,
                Urgency::High => ConsensusType::SimpleMajority,
                Urgency::Medium => ConsensusType::SuperMajority,
                Urgency::Low => ConsensusType::SuperMajority,
            },
            data: need.supporting_data,
        };

        let proposal_id = sangha
            .submit_proposal(proposal)
            .await
            .context("Failed to submit proposal")?;

        info!(
            "Submitted proposal {} to address {:?}",
            proposal_id, need.need_type
        );

        Ok(proposal_id)
    }
}

/// Factory function to create a Search Agent Sangha participant
pub fn create_search_agent_participant(
    agent_id: String,
    coordination_bus: Arc<CoordinationBus>,
) -> Box<dyn SanghaParticipant> {
    Box::new(SearchAgentSanghaParticipant::new(
        agent_id,
        coordination_bus,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_key_terms() {
        let coordination_bus = Arc::new(CoordinationBus::new().await.unwrap());
        let participant =
            SearchAgentSanghaParticipant::new("test-search".to_string(), coordination_bus);

        let text = "Implement React Hooks API for Frontend Development";
        let terms = participant.extract_key_terms(text);

        assert!(terms.contains(&"React".to_string()));
        assert!(terms.contains(&"Hooks".to_string()));
        assert!(terms.contains(&"API".to_string()));
        assert!(terms.contains(&"Frontend".to_string()));
        assert!(terms.contains(&"React Hooks".to_string()));
    }

    #[tokio::test]
    async fn test_voting_decision_logic() {
        let coordination_bus = Arc::new(CoordinationBus::new().await.unwrap());
        let participant =
            SearchAgentSanghaParticipant::new("test-search".to_string(), coordination_bus);

        // Test with positive results
        let positive_results = vec![
            SearchResult {
                title: "Success Story".to_string(),
                url: "http://example.com/1".to_string(),
                snippet: "This approach was highly effective and improved performance".to_string(),
                relevance_score: Some(0.9),
                metadata: None,
            },
            SearchResult {
                title: "Best Practices".to_string(),
                url: "http://example.com/2".to_string(),
                snippet: "Recommended approach with proven success".to_string(),
                relevance_score: Some(0.85),
                metadata: None,
            },
        ];

        let proposal = Proposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::AgentExtension,
            title: "Test Proposal".to_string(),
            description: "Test description".to_string(),
            proposer: "test".to_string(),
            created_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::hours(1),
            status: ProposalStatus::Voting,
            required_consensus: ConsensusType::SimpleMajority,
            data: serde_json::json!({}),
        };

        let decision = participant.analyze_search_results(&proposal, &positive_results);
        assert_eq!(decision.vote, VoteChoice::Aye);
        assert!(decision.confidence > 0.7);
    }
}
