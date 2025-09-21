//! Demonstration of Search Agent autonomous Sangha participation
//!
//! This example shows how the Search Agent:
//! 1. Monitors Sangha proposals
//! 2. Conducts research automatically
//! 3. Submits evidence-based votes
//! 4. Proposes initiatives when knowledge gaps are detected

use anyhow::Result;
use ccswarm::agent::search_agent::{SearchAgent, SearchResult};
use ccswarm::coordination::CoordinationBus;
use ccswarm::identity::AgentRole;
use ccswarm::sangha::{
    ConsensusType, Proposal, ProposalStatus, ProposalType, Sangha, SanghaConfig, SanghaMember,
    VoteChoice,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;
use uuid::Uuid;

/// Mock search results for demonstration
fn create_mock_search_results(query: &str) -> Vec<SearchResult> {
    match query.to_lowercase().as_str() {
        q if q.contains("react") && q.contains("performance") => vec![
            SearchResult {
                title: "React Performance Optimization Guide".to_string(),
                url: "https://react.dev/learn/render-and-commit".to_string(),
                snippet: "React apps are fast by default, but you can optimize re-renders and improve performance significantly with these techniques.".to_string(),
                relevance_score: Some(0.95),
                metadata: None,
            },
            SearchResult {
                title: "Common React Performance Pitfalls".to_string(),
                url: "https://blog.example.com/react-performance".to_string(),
                snippet: "Avoid these common mistakes that can slow down your React application and cause unnecessary re-renders.".to_string(),
                relevance_score: Some(0.87),
                metadata: None,
            },
        ],
        q if q.contains("security") => vec![
            SearchResult {
                title: "OWASP Security Best Practices".to_string(),
                url: "https://owasp.org/www-project-top-ten/".to_string(),
                snippet: "Critical security vulnerabilities to watch out for in modern web applications.".to_string(),
                relevance_score: Some(0.92),
                metadata: None,
            },
        ],
        _ => vec![
            SearchResult {
                title: "General Documentation".to_string(),
                url: "https://example.com/docs".to_string(),
                snippet: "General information about the topic.".to_string(),
                relevance_score: Some(0.5),
                metadata: None,
            },
        ],
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Search Agent Sangha Demonstration");

    // Create infrastructure
    let coordination_bus = Arc::new(CoordinationBus::new().await?);
    let sangha = Arc::new(Sangha::new(SanghaConfig::default())?);

    // Create search agent with mocked search capability
    let mut search_agent =
        SearchAgent::new("search-agent-demo".to_string(), coordination_bus.clone());

    // Enable Sangha participation
    search_agent.enable_sangha_participation();

    // Add to Sangha
    let member = SanghaMember {
        agent_id: search_agent.agent_id.clone(),
        role: AgentRole::Search {
            technologies: vec!["Web Search".to_string(), "Research".to_string()],
            responsibilities: vec![
                "Proposal Research".to_string(),
                "Evidence Gathering".to_string(),
            ],
            boundaries: vec!["Read-only operations".to_string()],
        },
        joined_at: Utc::now(),
        voting_power: 1.0,
        is_active: true,
        reputation: 1.0,
    };
    sangha.add_member(member).await?;

    // Create some test proposals
    let proposals = vec![
        Proposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::SystemExtension,
            title: "Implement React Performance Monitoring".to_string(),
            description: "Add comprehensive performance monitoring to track React component render times and identify bottlenecks.".to_string(),
            proposer: "frontend-agent".to_string(),
            created_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::hours(2),
            status: ProposalStatus::Voting,
            required_consensus: ConsensusType::SimpleMajority,
            data: serde_json::json!({
                "impact": "medium",
                "effort": "1 week",
            }),
        },
        Proposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::Emergency,
            title: "Security Vulnerability Patch".to_string(),
            description: "Critical security vulnerability discovered in authentication system needs immediate patching.".to_string(),
            proposer: "security-monitor".to_string(),
            created_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::minutes(30),
            status: ProposalStatus::Voting,
            required_consensus: ConsensusType::SimpleMajority,
            data: serde_json::json!({
                "severity": "critical",
                "cve": "CVE-2024-XXXX",
            }),
        },
        Proposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::AgentExtension,
            title: "Add Machine Learning Capabilities".to_string(),
            description: "Extend backend agent with ML model integration for predictive analytics.".to_string(),
            proposer: "backend-agent".to_string(),
            created_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::hours(4),
            status: ProposalStatus::Voting,
            required_consensus: ConsensusType::SuperMajority,
            data: serde_json::json!({
                "technologies": ["TensorFlow", "PyTorch"],
                "use_cases": ["User behavior prediction", "Anomaly detection"],
            }),
        },
    ];

    // Submit proposals
    for proposal in &proposals {
        sangha.submit_proposal(proposal.clone()).await?;
        info!("Submitted proposal: {}", proposal.title);
    }

    // Simulate the search agent's autonomous behavior
    info!("\n=== Search Agent begins autonomous research ===\n");

    // For each proposal, the agent would:
    for proposal in &proposals {
        info!("Researching proposal: {}", proposal.title);

        // 1. Extract search queries
        let queries = vec![
            proposal.title.clone(),
            format!("{} best practices", proposal.title),
            format!("{} implementation", proposal.title),
        ];

        // 2. Conduct searches (using mock data for demo)
        let mut all_results = Vec::new();
        for query in &queries {
            info!("  Searching for: {}", query);
            let results = create_mock_search_results(query);
            all_results.extend(results);
        }

        // 3. Analyze results
        let positive_indicators = all_results
            .iter()
            .filter(|r| {
                r.snippet.contains("effective")
                    || r.snippet.contains("best")
                    || r.snippet.contains("critical")
            })
            .count();

        let confidence = (positive_indicators as f64 / all_results.len().max(1) as f64).min(0.9);

        // 4. Determine vote
        let vote_choice = if proposal.proposal_type == ProposalType::Emergency {
            VoteChoice::Aye // Always support emergency proposals
        } else if confidence > 0.6 {
            VoteChoice::Aye
        } else if confidence < 0.3 {
            VoteChoice::Nay
        } else {
            VoteChoice::Abstain
        };

        info!("  Analysis complete:");
        info!("    - Found {} search results", all_results.len());
        info!("    - Positive indicators: {}", positive_indicators);
        info!("    - Confidence: {:.0}%", confidence * 100.0);
        info!("    - Vote: {:?}", vote_choice);

        // 5. Submit evidence (in real implementation)
        info!("    - Evidence submitted to Sangha");

        // 6. Cast vote
        let vote = ccswarm::sangha::Vote {
            voter_id: search_agent.agent_id.clone(),
            proposal_id: proposal.id,
            choice: vote_choice,
            reason: Some(format!(
                "Based on {} search results with {:.0}% confidence",
                all_results.len(),
                confidence * 100.0
            )),
            cast_at: Utc::now(),
            weight: confidence,
        };

        sangha.cast_vote(vote).await?;
        info!("    - Vote cast successfully\n");
    }

    // Demonstrate knowledge gap detection
    info!("=== Search Agent detects knowledge gaps ===\n");

    info!("Knowledge gap detected: Insufficient research on 'Rust WebAssembly integration'");
    info!("Creating proposal to address this gap...");

    let knowledge_gap_proposal = Proposal {
        id: Uuid::new_v4(),
        proposal_type: ProposalType::AgentExtension,
        title: "Research Rust WebAssembly Integration Patterns".to_string(),
        description: "Conduct comprehensive research on best practices for integrating Rust with WebAssembly for high-performance web applications.".to_string(),
        proposer: search_agent.agent_id.clone(),
        created_at: Utc::now(),
        voting_deadline: Utc::now() + chrono::Duration::hours(6),
        status: ProposalStatus::Draft,
        required_consensus: ConsensusType::SimpleMajority,
        data: serde_json::json!({
            "knowledge_gap_type": "technical_implementation",
            "urgency": "medium",
            "affected_agents": ["frontend-agent", "backend-agent"],
        }),
    };

    sangha.submit_proposal(knowledge_gap_proposal).await?;
    info!("Knowledge gap proposal submitted successfully");

    // Show final statistics
    let stats = sangha.get_stats().await;
    info!("\n=== Final Sangha Statistics ===");
    info!("Total members: {}", stats.total_members);
    info!("Total proposals: {}", stats.total_proposals);
    info!("Active proposals: {}", stats.active_proposals);

    // Show vote summary for each proposal
    info!("\n=== Vote Summary ===");
    for proposal in &proposals {
        let vote_stats = sangha.get_vote_stats(proposal.id).await;
        info!(
            "{}: {} aye, {} nay, {} abstain",
            proposal.title, vote_stats.aye, vote_stats.nay, vote_stats.abstain
        );
    }

    info!("\nSearch Agent Sangha demonstration complete!");

    Ok(())
}
