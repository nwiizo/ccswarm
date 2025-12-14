use ccswarm::config::{
    AgentConfig, CcswarmConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig,
    ProjectConfig, RepositoryConfig, ThinkMode,
};
use ccswarm::coordination::{AgentMessage, CoordinationBus};
use ccswarm::orchestrator::{MasterClaude, ProactiveMaster};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper function to create a test configuration
fn create_test_config() -> CcswarmConfig {
    let mut agents = HashMap::new();
    agents.insert(
        "frontend".to_string(),
        AgentConfig {
            specialization: "frontend".to_string(),
            worktree: "agents/frontend".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );

    CcswarmConfig {
        project: ProjectConfig {
            name: "TestProject".to_string(),
            repository: RepositoryConfig {
                url: "https://github.com/test/repo".to_string(),
                main_branch: "main".to_string(),
            },
            master_claude: MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.9,
                think_mode: ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: ClaudeConfig::for_master(),
                enable_proactive_mode: false,
                proactive_frequency: 300,
                high_frequency: 60,
            },
        },
        agents,
        coordination: CoordinationConfig {
            communication_method: "json_files".to_string(),
            sync_interval: 30,
            quality_gate_frequency: "on_commit".to_string(),
            master_review_trigger: "all_tasks_complete".to_string(),
        },
    }
}

#[tokio::test]
async fn test_master_claude_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    ccswarm::git::WorktreeManager::init_if_needed(&repo_path)
        .await
        .unwrap();

    let config = create_test_config();
    let master = MasterClaude::new(config.clone(), repo_path.clone())
        .await
        .unwrap();

    assert!(master.id.starts_with("master-claude-"));
    assert_eq!(master.config.project.name, "TestProject");
}

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
