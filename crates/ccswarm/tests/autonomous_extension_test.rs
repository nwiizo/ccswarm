use anyhow::Result;
use async_trait::async_trait;
use ccswarm::extension::autonomous_agent_extension::*;
use ccswarm::extension::ExtensionProposal;
use ccswarm::identity::AgentRole;
use std::sync::Arc;

// Mock AI provider for testing
struct MockExtensionAIProvider;

#[async_trait]
impl ExtensionAIProvider for MockExtensionAIProvider {
    async fn send_message(&self, _message: &str) -> Result<String> {
        Ok("Mock AI response".to_string())
    }
}

// Mock Sangha client for testing
struct MockSanghaClient;

#[async_trait]
impl SanghaClient for MockSanghaClient {
    async fn propose_extension(&self, _proposal: &ExtensionProposal) -> Result<ProposalId> {
        Ok(ProposalId(uuid::Uuid::new_v4()))
    }

    async fn get_consensus(&self, _proposal_id: &ProposalId) -> Result<ConsensusResult> {
        Ok(ConsensusResult {
            approved: true,
            confidence: 0.85,
            feedback: vec![],
            conditions: vec![],
        })
    }

    async fn submit_evidence(&self, _proposal_id: &ProposalId, _evidence: &Evidence) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_autonomous_extension_manager_creation() {
    let provider = Arc::new(MockExtensionAIProvider);
    let sangha_client = Arc::new(MockSanghaClient);

    let manager = AutonomousAgentExtensionManager::new(
        "test-agent".to_string(),
        AgentRole::Frontend {
            technologies: vec!["React".to_string()],
            responsibilities: vec!["UI development".to_string()],
            boundaries: vec!["Frontend only".to_string()],
        },
        provider,
        sangha_client,
    );

    // Record multiple failure experiences to trigger pattern detection
    for i in 0..3 {
        manager
            .record_experience(
                "frontend_development".to_string(),
                format!("Building React components #{}", i),
                vec!["Attempted to create hooks".to_string()],
                TaskOutcome::Failure {
                    reason: "lack of React hooks knowledge".to_string(), // lowercase 'lack'
                    error_details: None,
                },
            )
            .await
            .unwrap();
    }

    // Test autonomous proposal generation
    let proposals = manager.propose_extensions().await.unwrap();

    // Should identify the need for React hooks knowledge
    assert!(
        !proposals.is_empty(),
        "Should generate at least one proposal"
    );
}

#[tokio::test]
async fn test_experience_analyzer() {
    let analyzer = ExperienceAnalyzer;

    let experiences = vec![
        Experience {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            task_type: "api_development".to_string(),
            context: "Building REST endpoints".to_string(),
            actions_taken: vec!["Created endpoint".to_string()],
            outcome: TaskOutcome::Success {
                metrics: std::collections::HashMap::new(),
            },
            insights: vec![],
        },
        Experience {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            task_type: "api_development".to_string(),
            context: "Building GraphQL endpoints".to_string(),
            actions_taken: vec!["Attempted GraphQL setup".to_string()],
            outcome: TaskOutcome::Failure {
                reason: "Lack of GraphQL knowledge".to_string(),
                error_details: None,
            },
            insights: vec![],
        },
        // Add third experience to meet pattern extraction threshold
        Experience {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            task_type: "api_development".to_string(),
            context: "Building WebSocket endpoints".to_string(),
            actions_taken: vec!["Attempted WebSocket setup".to_string()],
            outcome: TaskOutcome::Failure {
                reason: "Lack of GraphQL knowledge".to_string(), // Same reason to trigger common failure
                error_details: None,
            },
            insights: vec![],
        },
    ];

    let analysis = analyzer.analyze_experiences(&experiences).await.unwrap();

    assert!(!analysis.patterns.is_empty(), "Should identify patterns");
    assert!(
        !analysis.recurring_challenges.is_empty(),
        "Should identify challenges"
    );
}

#[tokio::test]
async fn test_capability_assessor() {
    let assessor = CapabilityAssessor;

    let mut capabilities = std::collections::HashMap::new();
    capabilities.insert(
        "react_basics".to_string(),
        CapabilityInfo {
            name: "react_basics".to_string(),
            description: "Basic React knowledge".to_string(),
            proficiency_level: 0.7,
            usage_count: 15,
            last_used: chrono::Utc::now(),
            effectiveness_score: 0.85,
        },
    );

    let recent_tasks = vec![Experience {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        task_type: "frontend".to_string(),
        context: "Advanced React patterns".to_string(),
        actions_taken: vec![],
        outcome: TaskOutcome::Failure {
            reason: "unable to implement React hooks".to_string(), // lowercase 'unable' to match
            error_details: None,
        },
        insights: vec![],
    }];

    let assessment = assessor
        .assess_capabilities(&capabilities, &recent_tasks)
        .await
        .unwrap();

    assert!(
        !assessment.strengths.is_empty(),
        "Should identify strengths"
    );
    assert!(
        !assessment.gaps.is_empty(),
        "Should identify capability gaps"
    );
}

#[tokio::test]
async fn test_autonomous_reasoning_full_flow() {
    // Create manager to test through public API
    let provider = Arc::new(MockExtensionAIProvider);
    let sangha_client = Arc::new(MockSanghaClient);
    let manager = AutonomousAgentExtensionManager::new(
        "test-agent".to_string(),
        AgentRole::Backend {
            technologies: vec!["Node.js".to_string()],
            responsibilities: vec!["API development".to_string()],
            boundaries: vec!["Backend only".to_string()],
        },
        provider,
        sangha_client,
    );

    // Add experiences showing repeated failures
    for i in 0..5 {
        manager
            .record_experience(
                "testing".to_string(),
                format!("Writing unit tests #{}", i),
                vec!["Attempted to write tests".to_string()],
                TaskOutcome::Failure {
                    reason: "Lack of testing framework knowledge".to_string(),
                    error_details: None,
                },
            )
            .await
            .unwrap();
    }

    // Add a capability that's underperforming
    manager
        .update_capability(
            "basic_testing".to_string(),
            0.2,  // effectiveness
            true, // used
        )
        .await
        .unwrap();

    // Test autonomous proposal generation
    let proposals = manager.propose_extensions().await.unwrap();

    // Should identify the need for testing framework knowledge
    assert!(
        !proposals.is_empty(),
        "Should generate at least one proposal"
    );
}
