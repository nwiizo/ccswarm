use ai_session::coordination::{AgentId as AISessionAgentId, AgentMessage as AISessionMessage};
use ccswarm::agent::{AgentStatus, TaskResult};
/// Integration tests for message conversion between ccswarm and ai-session
use ccswarm::coordination::conversion::{
    AgentMappingRegistry, FromAISessionMessage, IntoAISessionMessage, UnifiedAgentInfo,
};
use ccswarm::coordination::AgentMessage as CCSwarmMessage;
use ccswarm::identity::{default_backend_role, default_frontend_role};
use chrono::Utc;

#[tokio::test]
async fn test_bidirectional_message_conversion() {
    // Create registry
    let registry = AgentMappingRegistry::new();

    // Register test agents
    let frontend_info = UnifiedAgentInfo {
        ccswarm_id: "frontend-001".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: default_frontend_role(),
        capabilities: vec!["Frontend".to_string(), "React".to_string()],
        metadata: serde_json::json!({
            "version": "1.0",
            "worktree": "/tmp/frontend"
        }),
    };

    let backend_info = UnifiedAgentInfo {
        ccswarm_id: "backend-001".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: default_backend_role(),
        capabilities: vec!["Backend".to_string(), "NodeJS".to_string()],
        metadata: serde_json::json!({
            "version": "1.0",
            "worktree": "/tmp/backend"
        }),
    };

    registry.register(frontend_info.clone()).await;
    registry.register(backend_info.clone()).await;

    // Test 1: TaskCompleted conversion
    let ccswarm_completed = CCSwarmMessage::TaskCompleted {
        agent_id: "frontend-001".to_string(),
        task_id: "task-123".to_string(),
        result: TaskResult {
            task_id: "task-123".to_string(),
            success: true,
            output: Some(serde_json::json!({
                "message": "Component created successfully",
                "artifacts": ["Button.tsx", "Button.css"],
                "metrics": {
                    "lines_of_code": 150,
                    "test_coverage": 95.5
                }
            }).to_string()),
            error: None,
            duration: Some(std::time::Duration::from_secs(5)),
        },
    };

    // Convert to ai-session
    let ai_msg = ccswarm_completed
        .clone()
        .into_ai_session(&registry)
        .await
        .unwrap();

    // Convert back to ccswarm
    let ccswarm_back = CCSwarmMessage::from_ai_session(ai_msg, &registry)
        .await
        .unwrap();

    // Verify roundtrip (note: task_id won't match due to UUID generation)
    match ccswarm_back {
        CCSwarmMessage::TaskCompleted {
            agent_id, result, ..
        } => {
            assert_eq!(agent_id, "frontend-001");
            assert_eq!(result.success, true);
            assert!(result.output.is_some());
        }
        _ => panic!("Expected TaskCompleted message"),
    }

    // Test 2: StatusUpdate conversion
    let ccswarm_status = CCSwarmMessage::StatusUpdate {
        agent_id: "backend-001".to_string(),
        status: AgentStatus::Working,
        metrics: serde_json::json!({"cpu": 25.5, "memory": 512}),
    };

    let ai_status = ccswarm_status.into_ai_session(&registry).await.unwrap();
    match ai_status {
        AISessionMessage::StatusUpdate {
            agent_id, status, ..
        } => {
            assert_eq!(agent_id, backend_info.ai_session_id);
            assert_eq!(status, "working");
        }
        _ => panic!("Expected StatusUpdate message"),
    }

    // Test 3: Help request conversion
    let ai_help = AISessionMessage::HelpRequest {
        agent_id: frontend_info.ai_session_id.clone(),
        context: "Cannot resolve React import".to_string(),
        priority: ai_session::coordination::MessagePriority::High,
    };

    let ccswarm_help = CCSwarmMessage::from_ai_session(ai_help, &registry)
        .await
        .unwrap();
    match ccswarm_help {
        CCSwarmMessage::RequestAssistance {
            agent_id, reason, ..
        } => {
            assert_eq!(agent_id, "frontend-001");
            assert_eq!(reason, "Cannot resolve React import");
        }
        _ => panic!("Expected RequestAssistance message"),
    }

    // Test 4: Custom/Unknown message handling
    let ccswarm_custom = CCSwarmMessage::QualityIssue {
        agent_id: "backend-001".to_string(),
        task_id: "task-456".to_string(),
        issues: vec!["Missing error handling".to_string()],
    };

    let ai_custom = ccswarm_custom.into_ai_session(&registry).await.unwrap();
    match ai_custom {
        AISessionMessage::Custom { message_type, data } => {
            assert_eq!(message_type, "ccswarm-message");
            assert!(data.get("QualityIssue").is_some());
        }
        _ => panic!("Expected Custom message"),
    }
}

#[tokio::test]
async fn test_agent_info_conversions() {
    // Test creating UnifiedAgentInfo from ccswarm agent
    let identity_role = default_frontend_role();
    let agent_role = ccswarm::agent::AgentRole::from_identity_role(&identity_role);
    let ccswarm_agent = ccswarm::agent::ClaudeCodeAgent::new(
        "frontend-test".to_string(),
        agent_role,
    );

    let unified_info = UnifiedAgentInfo::from_ccswarm_agent(&ccswarm_agent);
    assert_eq!(unified_info.role.name(), "Frontend");
    assert!(unified_info.capabilities.contains(&"Frontend".to_string()));
    assert_eq!(unified_info.ccswarm_id, ccswarm_agent.identity.agent_id);

    // Test creating UnifiedAgentInfo from ai-session registration
    let ai_agent_id = AISessionAgentId::new();
    let capabilities = vec!["Backend".to_string(), "API".to_string()];
    let metadata = serde_json::json!({
        "ccswarm_agent_id": "backend-test-123",
        "role": "Backend",
        "registered_at": Utc::now().to_rfc3339(),
    });

    let unified_from_ai = UnifiedAgentInfo::from_ai_session_registration(
        ai_agent_id.clone(),
        capabilities.clone(),
        metadata,
    )
    .unwrap();

    assert_eq!(unified_from_ai.ccswarm_id, "backend-test-123");
    assert_eq!(unified_from_ai.ai_session_id, ai_agent_id);
    assert_eq!(unified_from_ai.role.name(), "Backend");
    assert_eq!(unified_from_ai.capabilities, capabilities);
}

#[tokio::test]
async fn test_error_handling() {
    let registry = AgentMappingRegistry::new();

    // Test conversion with unregistered agent
    let unregistered_msg = CCSwarmMessage::StatusUpdate {
        agent_id: "unknown-agent".to_string(),
        status: AgentStatus::Available,
        metrics: serde_json::json!({}),
    };

    let result = unregistered_msg.into_ai_session(&registry).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("ai-session agent mapping"));

    // Test invalid role in from_ai_session_registration
    let invalid_metadata = serde_json::json!({
        "ccswarm_agent_id": "test-123",
        "role": "InvalidRole",
    });

    let result = UnifiedAgentInfo::from_ai_session_registration(
        AISessionAgentId::new(),
        vec!["test".to_string()],
        invalid_metadata,
    );

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid field value"));
}
