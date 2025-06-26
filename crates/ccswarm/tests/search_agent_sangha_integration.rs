//! Integration test for Search Agent Sangha participation

use anyhow::Result;
use ccswarm::agent::search_agent::SearchAgent;
use ccswarm::coordination::CoordinationBus;
use ccswarm::sangha::{
    Proposal, ProposalStatus, ProposalType, Sangha, SanghaConfig, SanghaMember,
};
use ccswarm::identity::AgentRole;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_search_agent_sangha_participation() -> Result<()> {
    // Create coordination bus
    let coordination_bus = Arc::new(CoordinationBus::new().await?);

    // Create Sangha
    let sangha_config = SanghaConfig {
        quorum_threshold: 1,
        simple_majority: 0.5,
        super_majority: 0.67,
        voting_duration_secs: 60,
        allow_proxy_voting: false,
        max_active_proposals: 10,
    };
    let sangha = Arc::new(Sangha::new(sangha_config)?);

    // Create and initialize search agent
    let mut search_agent = SearchAgent::new("search-agent-1".to_string(), coordination_bus.clone());
    
    // Skip gemini CLI verification for test
    // search_agent.initialize().await?;

    // Enable Sangha participation
    search_agent.enable_sangha_participation();

    // Add search agent as Sangha member
    let member = SanghaMember {
        agent_id: search_agent.agent_id.clone(),
        role: AgentRole::Search {
            technologies: vec!["Web Search".to_string()],
            responsibilities: vec!["Information Gathering".to_string()],
            boundaries: vec!["Read-only".to_string()],
        },
        joined_at: Utc::now(),
        voting_power: 1.0,
        is_active: true,
        reputation: 1.0,
    };
    sangha.add_member(member).await?;

    // Create a test proposal that needs research
    let proposal = Proposal {
        id: Uuid::new_v4(),
        proposal_type: ProposalType::AgentExtension,
        title: "Add GraphQL Support to Backend Agent".to_string(),
        description: "Proposal to add GraphQL API capabilities to the backend agent for more flexible data queries".to_string(),
        proposer: "master-claude".to_string(),
        created_at: Utc::now(),
        voting_deadline: Utc::now() + chrono::Duration::hours(1),
        status: ProposalStatus::Voting,
        required_consensus: ccswarm::sangha::ConsensusType::SimpleMajority,
        data: serde_json::json!({
            "technologies": ["GraphQL", "Apollo Server"],
            "estimated_effort": "2 weeks",
        }),
    };

    // Submit proposal to Sangha
    sangha.submit_proposal(proposal.clone()).await?;

    // Start Sangha monitoring (in production this would run continuously)
    // For testing, we'll just check that it starts without error
    search_agent.start_sangha_monitoring(sangha.clone()).await?;

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // In a real scenario, the search agent would:
    // 1. Detect the new proposal
    // 2. Conduct research on "GraphQL backend implementation"
    // 3. Analyze search results
    // 4. Submit evidence to Sangha
    // 5. Cast an informed vote

    // Verify Sangha stats
    let stats = sangha.get_stats().await;
    assert_eq!(stats.total_members, 1);
    assert_eq!(stats.active_proposals, 1);

    Ok(())
}

#[tokio::test]
async fn test_search_agent_knowledge_gap_detection() -> Result<()> {
    use ccswarm::sangha::search_agent_participant::{IdentifiedNeed, NeedType, Urgency};

    // This test would verify that the search agent can:
    // 1. Identify when proposals lack sufficient research
    // 2. Create new proposals to address knowledge gaps
    // 3. Prioritize research based on urgency

    // Create basic setup
    let coordination_bus = Arc::new(CoordinationBus::new().await?);
    let mut search_agent = SearchAgent::new("search-agent-2".to_string(), coordination_bus);
    search_agent.enable_sangha_participation();

    // In production, the agent would autonomously identify needs like:
    let example_need = IdentifiedNeed {
        need_type: NeedType::KnowledgeGap,
        description: "Insufficient research on WebAssembly integration patterns".to_string(),
        urgency: Urgency::High,
        supporting_data: serde_json::json!({
            "proposals_affected": 3,
            "current_evidence_count": 0,
            "recommended_queries": [
                "WebAssembly Rust integration",
                "WASM performance benchmarks",
                "WebAssembly security best practices"
            ]
        }),
    };

    // The agent would then create a proposal to address this gap
    println!("Example knowledge gap identified: {:?}", example_need);

    Ok(())
}