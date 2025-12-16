//! Integration tests for Master Claude orchestration
//!
//! These tests verify the coordination and delegation functionality.

use ccswarm::coordination::{AgentMessage, CoordinationBus};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_coordination_messages() {
    let bus = Arc::new(CoordinationBus::new().await.unwrap());

    let task_msg = AgentMessage::TaskAssignment {
        task_id: "test-task-001".to_string(),
        agent_id: "frontend-agent".to_string(),
        task_data: json!({
            "description": "Create login component",
            "priority": "high"
        }),
    };

    bus.send_message(task_msg.clone()).await.unwrap();
    let received_msg = bus.receive_message().await.unwrap();

    match received_msg {
        AgentMessage::TaskAssignment { task_id, .. } => {
            assert_eq!(task_id, "test-task-001");
        }
        _ => panic!("Unexpected message type received"),
    }
}
