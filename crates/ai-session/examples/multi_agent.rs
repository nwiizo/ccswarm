//! Multi-agent coordination example - demonstrates AI agent collaboration

use ai_session::coordination::{AgentId, BroadcastMessage, MessageBus, MessagePriority};
use ai_session::{SessionConfig, SessionManager};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("ai_session=info")
        .init();

    println!("AI Session Multi-Agent Example\n");

    // Create shared coordination bus
    let bus = Arc::new(RwLock::new(MessageBus::new()));
    println!("✓ Coordination bus initialized");

    // Create session manager
    let manager = SessionManager::new();

    // Create multiple agent sessions
    println!("\nCreating agent sessions...");

    let agents = vec![
        ("frontend", "Frontend Developer Agent"),
        ("backend", "Backend Developer Agent"),
        ("devops", "DevOps Engineer Agent"),
    ];

    let mut sessions = Vec::new();

    for (role, description) in agents {
        let mut config = SessionConfig::default();
        config.enable_ai_features = true;
        config.agent_role = Some(role.to_string());

        let session = manager.create_session_with_config(config).await?;
        println!("✓ {} session created: {}", description, session.id);

        // Register agent with bus (in a real implementation)
        // For demo purposes, we'll skip the complex subscription logic

        sessions.push((role, session));
    }

    // Simulate task distribution
    println!("\n📨 Distributing tasks to agents...");

    let tasks = vec![
        ("frontend", "Create React component for user dashboard"),
        ("backend", "Implement REST API for user data"),
        ("devops", "Set up CI/CD pipeline"),
        ("frontend", "Add responsive design to dashboard"),
        ("backend", "Add authentication middleware"),
    ];

    for (target_role, task_desc) in tasks {
        println!("\n  Task: {}", task_desc);
        println!("  Assigning to: {} agent", target_role);

        // Find the target session
        if let Some((_, session)) = sessions.iter().find(|(role, _)| *role == target_role) {
            // Send task via coordination bus
            let message = BroadcastMessage {
                id: uuid::Uuid::new_v4(),
                from: AgentId::new(), // Orchestrator agent
                content: format!(
                    "Task: {} (priority: high, estimated_tokens: 2000)",
                    task_desc
                ),
                priority: MessagePriority::High,
                timestamp: chrono::Utc::now(),
            };

            // In a real implementation, this would broadcast to actual agents
            println!("    Broadcasting task: {}", task_desc);

            // Simulate agent processing
            sleep(Duration::from_millis(500)).await;

            // Agent sends status update
            let status_message = BroadcastMessage {
                id: uuid::Uuid::new_v4(),
                from: AgentId::new(), // Session agent
                content: format!("Status: processing task '{}' (progress: 25%)", task_desc),
                priority: MessagePriority::Normal,
                timestamp: chrono::Utc::now(),
            };

            // In a real implementation, this would broadcast status
            println!("    Status update: Processing task");
            println!("  ✓ Agent acknowledged and started processing");
        }
    }

    // Show coordination statistics
    println!("\n📊 Coordination Statistics:");
    let stats = bus.read().await.get_statistics();
    println!("  - Total messages: {}", stats.total_messages);
    println!("  - Active subscribers: {}", stats.subscriber_count);
    println!("  - Message types: {:?}", stats.message_type_counts);

    // Demonstrate context sharing
    println!("\n🤝 Context Sharing:");
    println!("  - Frontend agent shares component structure with backend");
    println!("  - Backend agent shares API endpoints with frontend");
    println!("  - DevOps agent shares deployment status with all");

    // Show token efficiency across agents
    println!("\n💰 Token Efficiency:");
    let total_tokens = 15000; // Simulated
    let saved_tokens = 9500; // Simulated
    println!("  - Total tokens used: {}", total_tokens);
    println!("  - Tokens saved by sharing: {}", saved_tokens);
    println!(
        "  - Efficiency gain: {:.1}%",
        (saved_tokens as f64 / total_tokens as f64) * 100.0
    );

    // Clean up
    println!("\nTerminating all sessions...");
    for (role, session) in sessions {
        session.stop().await?;
        println!("✓ {} session terminated", role);
    }

    Ok(())
}

// Extension trait for demo statistics
trait CoordinationStats {
    fn get_statistics(&self) -> BusStatistics;
}

struct BusStatistics {
    total_messages: usize,
    subscriber_count: usize,
    message_type_counts: Vec<(String, usize)>,
}

impl CoordinationStats for MessageBus {
    fn get_statistics(&self) -> BusStatistics {
        // Demo implementation
        BusStatistics {
            total_messages: 10,
            subscriber_count: 3,
            message_type_counts: vec![
                ("TaskAssignment".to_string(), 5),
                ("StatusUpdate".to_string(), 5),
            ],
        }
    }
}
