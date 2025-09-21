//! Essential integration tests for ccswarm core functionality

use anyhow::Result;
use std::collections::HashMap;

// Import core modules
use ccswarm::agent::personality::AgentPersonality;
use ccswarm::coordination::{AgentMessage, CoordinationBus};
use ccswarm::identity::default_backend_role;
use ccswarm::session::{
    memory::{EpisodeOutcome, EpisodeType, SessionMemory, WorkingMemoryType},
    AgentSession,
};

#[tokio::test]
async fn test_basic_agent_creation() -> Result<()> {
    let personality = AgentPersonality::new("test-agent".to_string());
    assert_eq!(personality.style, "test-agent");
    assert!(!personality.approach.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_session_memory() -> Result<()> {
    let mut memory = SessionMemory::new("session-1".to_string(), "agent-1".to_string());

    memory.add_to_working_memory(
        "Test task".to_string(),
        WorkingMemoryType::TaskInstructions,
        0.8,
    );

    assert!(!memory.working_memory.current_items.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_coordination_bus() -> Result<()> {
    let bus = CoordinationBus::new().await?;

    let message = AgentMessage::StatusUpdate {
        agent_id: "test-agent".to_string(),
        status: ccswarm::agent::AgentStatus::Available,
        metrics: serde_json::json!({}),
    };

    bus.send_message(message).await?;
    let _received = bus.receive_message().await?;

    Ok(())
}

#[test]
fn test_basic_task_execution() {
    use ccswarm::agent::{Priority, Task, TaskType};
    let task = Task::new(
        "test-1".to_string(),
        "Test task".to_string(),
        Priority::Medium,
        TaskType::Development,
    );
    assert_eq!(task.id, "test-1");
    assert_eq!(task.priority, Priority::Medium);
}