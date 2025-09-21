use ai_session::coordination::{
    AgentId as AISessionAgentId, AgentMessage as AISessionMessage,
    MessagePriority as AISessionPriority,
};
use ccswarm::agent::{AgentStatus, ClaudeCodeAgent, Priority, TaskResult};
/// Example demonstrating the message conversion strategy between ccswarm and ai-session
///
/// This example shows how the type conversion layer enables MasterClaude integration
/// with ai-session's coordination system.

// Allow unused imports for demonstration purposes
#[allow(unused_imports)]
#[allow(dead_code)]
use ccswarm::coordination::conversion::{
    AgentMappingRegistry, ConversionError, FromAISessionMessage, IntoAISessionMessage,
    UnifiedAgentInfo,
};
use ccswarm::coordination::{AgentMessage as CCSwarmMessage, CoordinationType};
use ccswarm::identity::{default_backend_role, default_frontend_role};
use ccswarm::orchestrator::agent_access::AgentAttributeAccess;
use chrono::Utc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CCSwarm/AI-Session Message Conversion Demo ===\n");

    // 1. Create the agent mapping registry
    let registry = AgentMappingRegistry::new();
    println!("✓ Created agent mapping registry");

    // 2. Register some agents with unified info
    let frontend_agent = UnifiedAgentInfo {
        ccswarm_id: "frontend-specialist-001".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: default_frontend_role(),
        capabilities: vec![
            "Frontend".to_string(),
            "React".to_string(),
            "TypeScript".to_string(),
        ],
        metadata: serde_json::json!({
            "worktree": "/workspace/frontend",
            "branch": "feature/ui-redesign",
            "isolation_mode": "GitWorktree",
        }),
    };

    let backend_agent = UnifiedAgentInfo {
        ccswarm_id: "backend-specialist-001".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: default_backend_role(),
        capabilities: vec![
            "Backend".to_string(),
            "NodeJS".to_string(),
            "PostgreSQL".to_string(),
        ],
        metadata: serde_json::json!({
            "worktree": "/workspace/backend",
            "branch": "feature/api-v2",
            "isolation_mode": "GitWorktree",
        }),
    };

    registry.register(frontend_agent.clone()).await;
    registry.register(backend_agent.clone()).await;
    println!("✓ Registered 2 agents with unified information\n");

    // 3. Demonstrate agent attribute access
    println!("=== Agent Attribute Access ===");
    println!("Frontend agent:");
    println!("  - CCSwarm ID: {}", frontend_agent.ccswarm_id);
    println!("  - AI-Session ID: {}", frontend_agent.ai_session_id);
    println!("  - Role: {}", frontend_agent.role.name());
    println!("  - Capabilities: {:?}", frontend_agent.capabilities);
    println!();

    // 4. Demonstrate message conversions
    println!("=== Message Conversions ===\n");

    // Example 1: TaskCompleted conversion
    println!("1. TaskCompleted Message Conversion:");
    let ccswarm_task_completed = CCSwarmMessage::TaskCompleted {
        agent_id: "frontend-specialist-001".to_string(),
        task_id: "task-create-login-form".to_string(),
        result: TaskResult {
            success: true,
            output: serde_json::json!({
                "message": "Login form component created successfully",
                "files_created": ["LoginForm.tsx", "LoginForm.css", "LoginForm.test.tsx"],
                "lines_of_code": 250,
                "test_coverage": 98.5,
            }),
            error: None,
            duration: Duration::from_secs(120),
        },
    };

    // Convert to AI-Session message
    let ai_session_msg = ccswarm_task_completed
        .clone()
        .into_ai_session(&registry)
        .await?;
    println!("  ✓ Converted CCSwarm TaskCompleted → AI-Session message");

    // Convert back to CCSwarm
    let ccswarm_roundtrip = CCSwarmMessage::from_ai_session(ai_session_msg, &registry).await?;
    println!("  ✓ Converted AI-Session → CCSwarm TaskCompleted");

    match ccswarm_roundtrip {
        CCSwarmMessage::TaskCompleted {
            agent_id, result, ..
        } => {
            println!("  ✓ Roundtrip successful:");
            println!("    - Agent: {}", agent_id);
            println!("    - Success: {}", result.success);
            println!("    - Duration: {:?}", result.duration);
        }
        _ => println!("  ✗ Unexpected message type after roundtrip"),
    }
    println!();

    // Example 2: Help Request conversion
    println!("2. HelpRequest Message Conversion:");
    let ai_help_request = AISessionMessage::HelpRequest {
        agent_id: backend_agent.ai_session_id.clone(),
        context: "Cannot connect to PostgreSQL database - connection timeout".to_string(),
        priority: AISessionPriority::High,
    };

    let ccswarm_help = CCSwarmMessage::from_ai_session(ai_help_request, &registry).await?;
    println!("  ✓ Converted AI-Session HelpRequest → CCSwarm RequestAssistance");

    match ccswarm_help {
        CCSwarmMessage::RequestAssistance {
            agent_id, reason, ..
        } => {
            println!("  ✓ Conversion successful:");
            println!("    - Agent: {}", agent_id);
            println!("    - Reason: {}", reason);
        }
        _ => println!("  ✗ Unexpected message type"),
    }
    println!();

    // Example 3: Custom message handling
    println!("3. Custom Message Handling:");
    let ccswarm_quality_issue = CCSwarmMessage::QualityIssue {
        agent_id: "backend-specialist-001".to_string(),
        task_id: "task-implement-auth".to_string(),
        issues: vec![
            "Missing error handling in login endpoint".to_string(),
            "No rate limiting on authentication attempts".to_string(),
        ],
    };

    let ai_custom = ccswarm_quality_issue.into_ai_session(&registry).await?;
    println!("  ✓ Converted CCSwarm QualityIssue → AI-Session Custom message");

    match ai_custom {
        AISessionMessage::Custom { message_type, data } => {
            println!("  ✓ Custom message created:");
            println!("    - Type: {}", message_type);
            println!(
                "    - Contains QualityIssue data: {}",
                data.get("QualityIssue").is_some()
            );
        }
        _ => println!("  ✗ Unexpected message type"),
    }
    println!();

    // 5. Demonstrate MasterClaude integration pattern
    println!("=== MasterClaude Integration Pattern ===");
    println!("MasterClaude can now:");
    println!("  1. Register agents with unified CCSwarm/AI-Session identities");
    println!("  2. Send messages using either message format");
    println!("  3. Receive messages from AI-Session and convert to CCSwarm format");
    println!("  4. Access agent attributes using standardized methods");
    println!("  5. Maintain backward compatibility with existing CCSwarm code");
    println!();

    // 6. Show error handling
    println!("=== Error Handling ===");
    let unregistered_msg = CCSwarmMessage::StatusUpdate {
        agent_id: "unknown-agent".to_string(),
        status: AgentStatus::Available,
        metrics: serde_json::json!({}),
    };

    match unregistered_msg.into_ai_session(&registry).await {
        Ok(_) => println!("  ✗ Should have failed for unregistered agent"),
        Err(e) => println!("  ✓ Correctly caught error: {}", e),
    }

    println!("\n✓ Demo completed successfully!");

    Ok(())
}

/// Demonstrate standardized agent attribute access
fn demonstrate_agent_access(agent: &ClaudeCodeAgent) {
    println!("Agent Attributes via standardized access:");
    println!("  - ID: {}", agent.agent_id());
    println!("  - Role: {}", agent.role().name());
    println!("  - Specialization: {}", agent.specialization());
    println!("  - Capabilities: {:?}", agent.capabilities());
    println!(
        "  - Has Frontend capability: {}",
        agent.has_capability("Frontend")
    );
}
