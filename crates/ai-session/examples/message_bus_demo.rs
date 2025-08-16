//! Demonstrates the enhanced MessageBus functionality for ccswarm integration

use ai_session::{
    AgentId, AgentMessage, MessagePriority, MultiAgentSession, SessionConfig, SessionManager,
    TaskId,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== MessageBus Enhancement Demo ===\n");

    // Create the multi-agent session and session manager
    let multi_session = Arc::new(MultiAgentSession::new());
    let manager = SessionManager::new();

    // Create three agents
    let frontend_id = AgentId::new();
    let backend_id = AgentId::new();
    let devops_id = AgentId::new();

    println!("Creating agents:");
    println!("  Frontend: {}", frontend_id);
    println!("  Backend: {}", backend_id);
    println!("  DevOps: {}\n", devops_id);

    // Configure and register agents
    for (agent_id, role) in [
        (frontend_id.clone(), "frontend"),
        (backend_id.clone(), "backend"),
        (devops_id.clone(), "devops"),
    ] {
        let mut config = SessionConfig::default();
        config.agent_role = Some(role.to_string());
        config.enable_ai_features = true;

        let session = manager.create_session_with_config(config).await?;
        multi_session.register_agent(agent_id, session)?;
    }

    // Get the message bus
    let message_bus = &multi_session.message_bus;

    // Subscribe to all messages for monitoring
    let all_messages = message_bus.subscribe_all();

    // Spawn a monitoring task
    let _monitor_handle = tokio::spawn(async move {
        println!("Starting message monitor...\n");

        while let Ok(msg) = all_messages.recv() {
            match msg {
                AgentMessage::Registration {
                    agent_id,
                    capabilities,
                    ..
                } => {
                    println!(
                        "📝 Registration: Agent {} with capabilities: {:?}",
                        agent_id, capabilities
                    );
                }
                AgentMessage::TaskAssignment {
                    task_id, agent_id, ..
                } => {
                    println!("📋 Task {} assigned to agent {}", task_id, agent_id);
                }
                AgentMessage::TaskProgress {
                    agent_id,
                    progress,
                    message,
                    ..
                } => {
                    println!(
                        "📊 Progress from {}: {}% - {}",
                        agent_id,
                        (progress * 100.0) as u32,
                        message
                    );
                }
                AgentMessage::TaskCompleted {
                    agent_id, task_id, ..
                } => {
                    println!("✅ Task {} completed by agent {}", task_id, agent_id);
                }
                AgentMessage::HelpRequest {
                    agent_id,
                    context,
                    priority,
                } => {
                    println!(
                        "🆘 Help request from {} (priority: {:?}): {}",
                        agent_id, priority, context
                    );
                }
                AgentMessage::StatusUpdate {
                    agent_id, status, ..
                } => {
                    println!("📍 Status update from {}: {}", agent_id, status);
                }
                AgentMessage::Custom { message_type, .. } => {
                    println!("🔧 Custom message type: {}", message_type);
                }
            }
        }
    });

    // Simulate agent interactions
    println!("Simulating agent interactions...\n");

    // Frontend registers its capabilities
    message_bus
        .publish_to_agent(
            &frontend_id,
            AgentMessage::Registration {
                agent_id: frontend_id.clone(),
                capabilities: vec![
                    "react".to_string(),
                    "typescript".to_string(),
                    "css".to_string(),
                ],
                metadata: serde_json::json!({
                    "version": "1.0",
                    "experience_level": "senior"
                }),
            },
        )
        .await?;

    sleep(Duration::from_millis(100)).await;

    // Backend registers its capabilities
    message_bus
        .publish_to_agent(
            &backend_id,
            AgentMessage::Registration {
                agent_id: backend_id.clone(),
                capabilities: vec![
                    "rust".to_string(),
                    "api".to_string(),
                    "database".to_string(),
                ],
                metadata: serde_json::json!({
                    "version": "1.0",
                    "preferred_framework": "actix-web"
                }),
            },
        )
        .await?;

    sleep(Duration::from_millis(100)).await;

    // Master assigns a task to frontend
    let task_id = TaskId::new();
    message_bus
        .publish_to_agent(
            &frontend_id,
            AgentMessage::TaskAssignment {
                task_id: task_id.clone(),
                agent_id: frontend_id.clone(),
                task_data: serde_json::json!({
                    "type": "implement_ui",
                    "component": "UserDashboard",
                    "requirements": ["responsive", "accessible"]
                }),
            },
        )
        .await?;

    sleep(Duration::from_millis(200)).await;

    // Frontend reports progress
    for progress in [0.25, 0.5, 0.75, 1.0] {
        message_bus
            .publish_to_agent(
                &frontend_id,
                AgentMessage::TaskProgress {
                    agent_id: frontend_id.clone(),
                    task_id: task_id.clone(),
                    progress,
                    message: format!(
                        "Building component... {}% complete",
                        (progress * 100.0) as u32
                    ),
                },
            )
            .await?;
        sleep(Duration::from_millis(150)).await;
    }

    // Frontend completes the task
    message_bus
        .publish_to_agent(
            &frontend_id,
            AgentMessage::TaskCompleted {
                agent_id: frontend_id.clone(),
                task_id: task_id.clone(),
                result: serde_json::json!({
                    "success": true,
                    "files_created": ["UserDashboard.tsx", "UserDashboard.css"],
                    "tests_passed": 12
                }),
            },
        )
        .await?;

    sleep(Duration::from_millis(100)).await;

    // Frontend needs help with API integration
    message_bus
        .publish_to_agent(
            &backend_id,
            AgentMessage::HelpRequest {
                agent_id: frontend_id.clone(),
                context: "Need help with API endpoint for user data".to_string(),
                priority: MessagePriority::High,
            },
        )
        .await?;

    sleep(Duration::from_millis(100)).await;

    // Backend sends status update
    message_bus
        .publish_to_agent(
            &backend_id,
            AgentMessage::StatusUpdate {
                agent_id: backend_id.clone(),
                status: "ready".to_string(),
                metrics: serde_json::json!({
                    "cpu_usage": 15,
                    "memory_mb": 256,
                    "active_connections": 3
                }),
            },
        )
        .await?;

    sleep(Duration::from_millis(100)).await;

    // Custom message example
    message_bus
        .publish_to_agent(
            &devops_id,
            AgentMessage::Custom {
                message_type: "deployment_request".to_string(),
                data: serde_json::json!({
                    "environment": "staging",
                    "version": "1.2.3",
                    "components": ["frontend", "backend"]
                }),
            },
        )
        .await?;

    // Wait a bit for all messages to be processed
    sleep(Duration::from_millis(500)).await;

    println!("\n=== Demo Complete ===");

    Ok(())
}
