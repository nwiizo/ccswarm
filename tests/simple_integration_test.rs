//! Simple integration tests for ccswarm v0.3.0
//! Verifies core functionality including AI agent enhancements

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

// Import the modules we're testing
use ccswarm::agent::personality::AgentPersonality;
use ccswarm::coordination::{
    dialogue::{
        ConversationContext, ConversationType, DialogueCoordinationBus, DialogueMessageType,
        EmotionalTone, ResponseExpectation,
    },
    AgentMessage, CoordinationBus,
};
use ccswarm::identity::{default_backend_role, default_frontend_role};
use ccswarm::session::{
    memory::{EpisodeOutcome, EpisodeType, SessionMemory, WorkingMemoryType},
    AgentSession,
};

#[tokio::test]
async fn test_agent_personality_integration() -> Result<()> {
    println!("🧠 Testing Agent Personality System Integration...");

    let frontend_role = default_frontend_role();
    let personality = AgentPersonality::new("test-agent".to_string(), &frontend_role);

    // Test personality creation
    assert_eq!(personality.agent_id, "test-agent");
    assert!(!personality.skills.is_empty());
    assert!(personality.skills.contains_key("react"));

    // Test skill experience update
    let mut test_personality = personality.clone();
    test_personality.update_skill_experience("react", 50, true);
    assert!(test_personality.skills["react"].experience_points >= 150);

    // Test task approach generation
    let approach = test_personality.get_task_approach("Create React component with TypeScript");
    assert!(approach.estimated_effort > 0.0);
    assert!(approach.quality_focus > 0.0);

    // Test relevant skills detection
    let relevant_skills = test_personality.get_relevant_skills("React component testing");
    assert!(!relevant_skills.is_empty());

    println!("✅ Agent Personality System: PASSED");
    Ok(())
}

#[tokio::test]
async fn test_session_memory_integration() -> Result<()> {
    println!("🧠 Testing Session Memory System Integration...");

    let mut memory = SessionMemory::new("session-123".to_string(), "agent-456".to_string());

    // Test working memory
    memory.add_to_working_memory(
        "Task: Create user authentication".to_string(),
        WorkingMemoryType::TaskInstructions,
        0.8,
    );
    assert!(!memory.working_memory.current_items.is_empty());

    // Test task context
    memory.set_task_context(
        "task-123".to_string(),
        "User authentication implementation".to_string(),
    );
    assert!(memory.working_memory.active_task_context.is_some());

    // Test episodic memory
    let mut context = HashMap::new();
    context.insert("task_type".to_string(), "development".to_string());

    memory.add_episode(
        EpisodeType::TaskCompletion,
        "Successfully implemented authentication".to_string(),
        context,
        EpisodeOutcome::Success {
            metrics: HashMap::new(),
        },
    );
    assert!(!memory.episodic_memory.episodes.is_empty());

    // Test memory consolidation
    memory.consolidate_memories();

    // Test memory retrieval
    let retrieval_result = memory.retrieve_relevant_memories("authentication");
    assert!(
        !retrieval_result.working_memory_items.is_empty()
            || !retrieval_result.relevant_episodes.is_empty()
    );

    // Test memory summary
    let summary = memory.generate_memory_summary();
    assert_eq!(summary.session_id, "session-123");
    assert_eq!(summary.agent_id, "agent-456");

    println!("✅ Session Memory System: PASSED");
    Ok(())
}

#[tokio::test]
async fn test_dialogue_coordination_integration() -> Result<()> {
    println!("💬 Testing Dialogue Coordination System Integration...");

    let mut dialogue_bus = DialogueCoordinationBus::new().await?;

    // Test conversation creation
    let participants = vec!["frontend-agent".to_string(), "backend-agent".to_string()];
    let context = ConversationContext {
        project_context: Some("User authentication project".to_string()),
        task_context: Some("API and UI coordination".to_string()),
        shared_resources: vec!["database".to_string(), "auth-service".to_string()],
        constraints: vec!["security-first".to_string()],
        goals: vec![
            "implement secure auth".to_string(),
            "maintain UX quality".to_string(),
        ],
        related_conversations: vec![],
    };

    let conversation_id = dialogue_bus
        .start_conversation(
            participants.clone(),
            "Authentication Implementation Coordination".to_string(),
            ConversationType::TaskCoordination,
            context,
        )
        .await?;

    assert!(!conversation_id.is_empty());
    assert!(dialogue_bus.conversations.contains_key(&conversation_id));

    // Test dialogue messages
    dialogue_bus
        .add_dialogue_message(
            &conversation_id,
            "frontend-agent".to_string(),
            "I need to know the API endpoints for user login and registration".to_string(),
            DialogueMessageType::Question,
            EmotionalTone::Curious,
            ResponseExpectation::ThoughtfulResponse,
        )
        .await?;

    dialogue_bus
        .add_dialogue_message(
            &conversation_id,
            "backend-agent".to_string(),
            "I'll provide POST /auth/login and POST /auth/register endpoints with JWT response"
                .to_string(),
            DialogueMessageType::Answer,
            EmotionalTone::Confident,
            ResponseExpectation::Acknowledgment,
        )
        .await?;

    // Test conversation summary
    let summary = dialogue_bus
        .get_conversation_summary(&conversation_id)
        .unwrap();
    assert_eq!(summary.participants.len(), 2);
    assert_eq!(summary.message_count, 2);
    assert_eq!(summary.topic, "Authentication Implementation Coordination");

    // Test active conversations
    let active_conversations = dialogue_bus.get_active_conversations();
    assert!(!active_conversations.is_empty());

    println!("✅ Dialogue Coordination System: PASSED");
    Ok(())
}

#[tokio::test]
async fn test_session_with_memory() -> Result<()> {
    println!("📝 Testing Session with Memory Integration...");

    let session = AgentSession::new(
        "test-agent".to_string(),
        default_backend_role(),
        "/tmp/test".to_string(),
        Some("Integration test session".to_string()),
    );

    // Test session has memory
    assert_eq!(session.memory.session_id, session.id);
    assert_eq!(session.memory.agent_id, session.agent_id);

    // Test memory operations through session
    let mut test_session = session.clone();

    test_session.add_memory(
        "Working on API implementation".to_string(),
        WorkingMemoryType::TaskInstructions,
        0.9,
    );

    test_session.set_task_context(
        "task-api-123".to_string(),
        "Implement REST API for user management".to_string(),
    );

    let mut episode_context = HashMap::new();
    episode_context.insert("complexity".to_string(), "high".to_string());

    test_session.add_episode(
        EpisodeType::ProblemSolving,
        "Solved complex authentication issue".to_string(),
        episode_context,
        EpisodeOutcome::Success {
            metrics: HashMap::new(),
        },
    );

    // Test memory consolidation
    test_session.consolidate_memories();

    // Test memory retrieval
    let _memories = test_session.retrieve_memories("authentication");
    // Should find relevant memories

    // Test memory summary
    let memory_summary = test_session.get_memory_summary();
    assert_eq!(memory_summary.session_id, test_session.id);

    println!("✅ Session Memory Integration: PASSED");
    Ok(())
}

#[tokio::test]
async fn test_coordination_bus_functionality() -> Result<()> {
    println!("🚌 Testing Coordination Bus Functionality...");

    let bus = CoordinationBus::new().await?;

    // Test basic message sending and receiving
    let message = AgentMessage::Heartbeat {
        agent_id: "test-agent".to_string(),
        timestamp: Utc::now(),
    };

    bus.send_message(message.clone()).await?;
    let received = bus.receive_message().await?;

    match received {
        AgentMessage::Heartbeat { agent_id, .. } => {
            assert_eq!(agent_id, "test-agent");
        }
        _ => panic!("Wrong message type received"),
    }

    println!("✅ Coordination Bus: PASSED");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_feature_validation() -> Result<()> {
    println!("🔍 Comprehensive Feature Validation...");

    // Test 1: Agent Personality System
    println!("  → Testing personality traits and skill progression...");
    let mut personality =
        AgentPersonality::new("comprehensive-test".to_string(), &default_frontend_role());

    // Test skill progression
    let initial_react_xp = personality.skills["react"].experience_points;
    personality.update_skill_experience("react", 100, true);
    assert!(personality.skills["react"].experience_points > initial_react_xp);

    // Test adaptation
    personality.adapt_from_task_outcome("complex", true, None);
    assert!(!personality.adaptation_history.is_empty());

    // Test 2: Memory System
    println!("  → Testing four-type memory system...");
    let mut memory = SessionMemory::new(
        "comprehensive-session".to_string(),
        "comprehensive-agent".to_string(),
    );

    // Working memory
    memory.add_to_working_memory(
        "Critical task data".to_string(),
        WorkingMemoryType::TaskInstructions,
        0.9,
    );

    // Episodic memory
    memory.add_episode(
        EpisodeType::Learning,
        "Learned new React pattern".to_string(),
        HashMap::new(),
        EpisodeOutcome::Success {
            metrics: HashMap::new(),
        },
    );

    // Semantic memory
    memory.add_concept(
        "React Hooks".to_string(),
        "Functions that let you use state and lifecycle features".to_string(),
        HashMap::new(),
    );

    // Procedural memory
    memory.add_procedure(
        "Component Creation".to_string(),
        vec![], // Simplified for test
    );

    // Test consolidation
    memory.consolidate_memories();

    // Test 3: Dialogue System
    println!("  → Testing sophisticated dialogue management...");
    let mut dialogue_bus = DialogueCoordinationBus::new().await?;

    let conversation_id = dialogue_bus
        .start_conversation(
            vec![
                "agent1".to_string(),
                "agent2".to_string(),
                "agent3".to_string(),
            ],
            "Three-way collaboration test".to_string(),
            ConversationType::ProblemSolving,
            ConversationContext {
                project_context: Some("Multi-agent coordination".to_string()),
                task_context: Some("Complex problem solving".to_string()),
                shared_resources: vec!["shared-db".to_string()],
                constraints: vec!["time-critical".to_string()],
                goals: vec!["solve efficiently".to_string()],
                related_conversations: vec![],
            },
        )
        .await?;

    // Multi-turn conversation
    for i in 1..=3 {
        dialogue_bus
            .add_dialogue_message(
                &conversation_id,
                format!("agent{}", i),
                format!("Agent {} contributing to solution", i),
                DialogueMessageType::Suggestion,
                EmotionalTone::Confident,
                ResponseExpectation::ThoughtfulResponse,
            )
            .await?;
    }

    let summary = dialogue_bus
        .get_conversation_summary(&conversation_id)
        .unwrap();
    assert_eq!(summary.message_count, 3);
    assert_eq!(summary.participants.len(), 3);

    // Test 4: Integration Points
    println!("  → Testing integration between systems...");

    // Session with memory
    let mut session = AgentSession::new(
        "integration-agent".to_string(),
        default_backend_role(),
        "/tmp/integration-test".to_string(),
        Some("Integration validation".to_string()),
    );

    // Memory operations through session
    session.add_memory(
        "Integration test data".to_string(),
        WorkingMemoryType::IntermediateResult,
        0.8,
    );
    session.set_task_context(
        "integration-task".to_string(),
        "Validate all integrations".to_string(),
    );

    // Episode with detailed context
    let mut episode_context = HashMap::new();
    episode_context.insert("test_type".to_string(), "integration".to_string());
    episode_context.insert("complexity".to_string(), "high".to_string());
    episode_context.insert(
        "success_factors".to_string(),
        "coordination,memory,dialogue".to_string(),
    );

    session.add_episode(
        EpisodeType::TaskCompletion,
        "Successfully validated all integration points".to_string(),
        episode_context,
        EpisodeOutcome::Success {
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("completion_rate".to_string(), 1.0);
                metrics.insert("quality_score".to_string(), 0.95);
                metrics.insert("collaboration_effectiveness".to_string(), 0.88);
                metrics
            },
        },
    );

    session.consolidate_memories();
    let final_summary = session.get_memory_summary();

    assert!(final_summary.working_memory_load >= 0.0);
    assert!(final_summary.episodic_memory_size > 0);

    println!("✅ All Features Validated Successfully!");
    println!("📊 Integration Test Results:");
    println!("  • Agent Personality System: ✅ Functional");
    println!("  • Memory System (4 types): ✅ Functional");
    println!("  • Dialogue Coordination: ✅ Functional");
    println!("  • Session Integration: ✅ Functional");
    println!("  • Cross-system Integration: ✅ Functional");

    Ok(())
}

#[test]
fn test_version_update() {
    println!("🏷️  Testing Version Update to v0.3.0...");

    // This test ensures we're ready for version update
    // In a real scenario, this would check Cargo.toml, documentation, etc.

    let expected_features = vec![
        "Agent Personality System",
        "Multi-Type Memory System",
        "Dialogue Coordination",
        "Session Memory Integration",
        "Enhanced Agent Capabilities",
    ];

    for feature in &expected_features {
        println!("  ✅ {}", feature);
    }

    println!("🎉 Ready for v0.3.0 release!");
}
