use ai_session::coordination::{
    AgentId as AISessionAgentId, AgentMessage as AISessionMessage,
    MessagePriority as AISessionPriority,
};
/// Demo of ccswarm <-> ai-session message conversion
///
/// This example demonstrates how to use the conversion module to translate
/// messages between ccswarm and ai-session coordination systems.
use anyhow::Result;
use ccswarm::agent::{AgentStatus, TaskResult};
use ccswarm::coordination::conversion::{
    convert_from_ai_session, convert_to_ai_session, AgentMappingRegistry, UnifiedAgentInfo,
};
use ccswarm::coordination::AgentMessage as CCSwarmMessage;
use chrono::Utc;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CCSwarm <-> AI-Session Message Conversion Demo ===\n");

    // Step 1: Create the agent mapping registry
    let registry = AgentMappingRegistry::new();
    println!("✓ Created agent mapping registry");

    // Step 2: Register agents
    let frontend_info = UnifiedAgentInfo {
        ccswarm_id: "frontend-specialist".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: ccswarm::identity::default_frontend_role(),
        capabilities: vec!["React".to_string(), "Vue".to_string(), "UI/UX".to_string()],
        metadata: json!({
            "version": "1.0.0",
            "worktree": "/tmp/frontend-agent",
            "branch": "feature/ui-redesign",
        }),
    };

    let backend_info = UnifiedAgentInfo {
        ccswarm_id: "backend-engineer".to_string(),
        ai_session_id: AISessionAgentId::new(),
        role: ccswarm::identity::default_backend_role(),
        capabilities: vec![
            "API".to_string(),
            "Database".to_string(),
            "Auth".to_string(),
        ],
        metadata: json!({
            "version": "1.0.0",
            "worktree": "/tmp/backend-agent",
            "branch": "feature/api-v2",
        }),
    };

    registry.register(frontend_info.clone()).await;
    registry.register(backend_info.clone()).await;
    println!("✓ Registered 2 agents: frontend-specialist, backend-engineer\n");

    // Step 3: Demo ccswarm -> ai-session conversion
    println!("--- Converting CCSwarm Messages to AI-Session ---");

    // Status update message
    let ccswarm_status = CCSwarmMessage::StatusUpdate {
        agent_id: "frontend-specialist".to_string(),
        status: AgentStatus::Working,
        metrics: json!({
            "tasks_completed": 12,
            "uptime_hours": 2.5,
            "memory_usage_mb": 512,
        }),
    };

    let ai_status = convert_to_ai_session(ccswarm_status, &registry).await?;
    println!("✓ Converted StatusUpdate:");
    println!("  CCSwarm: frontend-specialist -> Working");
    match &ai_status {
        AISessionMessage::StatusUpdate {
            agent_id,
            status,
            metrics,
        } => {
            println!(
                "  AI-Session: {} -> {} (metrics: {:?})",
                agent_id, status, metrics
            );
        }
        _ => unreachable!(),
    }

    // Task completed message
    let ccswarm_completed = CCSwarmMessage::TaskCompleted {
        agent_id: "backend-engineer".to_string(),
        task_id: "task-123".to_string(),
        result: TaskResult {
            task_id: "task-123".to_string(),
            success: true,
            output: Some(json!({
                "message": "API endpoint created successfully",
                "endpoints": ["/api/v2/users", "/api/v2/auth"],
                "documentation": "https://docs.example.com/api/v2",
            }).to_string()),
            error: None,
            duration: Some(Duration::from_secs(45)),
        },
    };

    let ai_completed = convert_to_ai_session(ccswarm_completed, &registry).await?;
    println!("\n✓ Converted TaskCompleted:");
    println!("  CCSwarm: backend-engineer completed task-123");
    match &ai_completed {
        AISessionMessage::TaskCompleted {
            agent_id, result, ..
        } => {
            println!("  AI-Session: {} result: {}", agent_id, result);
        }
        _ => unreachable!(),
    }

    // Step 4: Demo ai-session -> ccswarm conversion
    println!("\n--- Converting AI-Session Messages to CCSwarm ---");

    // Help request message
    let ai_help = AISessionMessage::HelpRequest {
        agent_id: frontend_info.ai_session_id.clone(),
        context: "Need assistance with React state management in complex form".to_string(),
        priority: AISessionPriority::High,
    };

    let ccswarm_help = convert_from_ai_session(ai_help, &registry).await?;
    println!("✓ Converted HelpRequest:");
    match &ccswarm_help {
        CCSwarmMessage::RequestAssistance {
            agent_id, reason, ..
        } => {
            println!(
                "  AI-Session: {} (High priority)",
                frontend_info.ai_session_id
            );
            println!("  CCSwarm: {} - {}", agent_id, reason);
        }
        _ => unreachable!(),
    }

    // Task progress message
    let ai_progress = AISessionMessage::TaskProgress {
        agent_id: backend_info.ai_session_id.clone(),
        task_id: ai_session::coordination::TaskId::new(),
        progress: 0.75,
        message: "Database migration 75% complete".to_string(),
    };

    let ccswarm_progress = convert_from_ai_session(ai_progress, &registry).await?;
    println!("\n✓ Converted TaskProgress:");
    match &ccswarm_progress {
        CCSwarmMessage::InterAgentMessage {
            from_agent,
            message,
            ..
        } => {
            println!(
                "  AI-Session: {} progress update",
                backend_info.ai_session_id
            );
            println!("  CCSwarm: {} - {}", from_agent, message);
        }
        _ => unreachable!(),
    }

    // Step 5: Demo round-trip conversion
    println!("\n--- Round-Trip Conversion Test ---");

    let original = CCSwarmMessage::InterAgentMessage {
        from_agent: "frontend-specialist".to_string(),
        to_agent: "backend-engineer".to_string(),
        message: "UI is ready for the new API endpoints".to_string(),
        timestamp: Utc::now(),
    };

    println!("Original CCSwarm message:");
    println!("  From: frontend-specialist");
    println!("  To: backend-engineer");
    println!("  Message: UI is ready for the new API endpoints");

    // Convert to AI-Session and back
    let ai_converted = convert_to_ai_session(original.clone(), &registry).await?;
    let restored = convert_from_ai_session(ai_converted, &registry).await?;

    println!("\nAfter round-trip conversion:");
    match &restored {
        CCSwarmMessage::Coordination { payload, .. } => {
            let data = payload.get("data").unwrap();
            println!("  Type: Coordination (Custom)");
            println!("  Preserved data: {}", serde_json::to_string_pretty(data)?);
        }
        _ => println!("  Unexpected message type"),
    }

    // Step 6: Demonstrate unified agent info
    println!("\n--- Unified Agent Information ---");

    if let Some(info) = registry.get_agent_info("frontend-specialist").await {
        println!("Frontend Specialist:");
        println!("  CCSwarm ID: {}", info.ccswarm_id);
        println!("  AI-Session ID: {}", info.ai_session_id);
        println!("  Role: {}", info.role.name());
        println!("  Capabilities: {:?}", info.capabilities);
        println!(
            "  Metadata: {}",
            serde_json::to_string_pretty(&info.metadata)?
        );
    }

    println!("\n✅ Conversion demo completed successfully!");
    Ok(())
}
