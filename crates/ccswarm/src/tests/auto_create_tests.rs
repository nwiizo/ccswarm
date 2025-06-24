use tempfile::TempDir;

use crate::config::CcswarmConfig;
use crate::orchestrator::auto_create::AutoCreateEngine;

#[tokio::test]
async fn test_auto_create_engine_initialization() {
    let mut engine = AutoCreateEngine::new();

    // Should have templates for TODO app
    let tasks = engine
        .analyze_and_decompose("Create a TODO app")
        .await
        .unwrap();
    assert!(!tasks.is_empty());
    assert!(tasks.len() >= 4); // Frontend, Backend, DB, Tests
}

#[tokio::test]
async fn test_app_type_detection() {
    let mut engine = AutoCreateEngine::new();

    // Test TODO detection
    let tasks = engine
        .analyze_and_decompose("Create a TODO application")
        .await
        .unwrap();
    assert!(!tasks.is_empty());

    // Test Blog detection
    let tasks = engine
        .analyze_and_decompose("Create a blog website")
        .await
        .unwrap();
    assert!(!tasks.is_empty());

    // Test multiple app types are detected properly
    let tasks = engine
        .analyze_and_decompose("Create an online shop")
        .await
        .unwrap();
    assert!(!tasks.is_empty());
}

#[tokio::test]
async fn test_task_decomposition() {
    let mut engine = AutoCreateEngine::new();

    let tasks = engine
        .analyze_and_decompose("Create a TODO app with authentication")
        .await
        .unwrap();

    // Should have standard tasks
    assert!(tasks.iter().any(|t| t.description.contains("React")));
    assert!(tasks.iter().any(|t| t.description.contains("REST API")));

    // Should have authentication task
    assert!(tasks
        .iter()
        .any(|t| t.description.contains("authentication") || t.description.contains("JWT")));
}

#[tokio::test]
#[ignore = "Full auto create workflow test may fail in CI environment"]
async fn test_full_auto_create_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().to_path_buf();

    let mut engine = AutoCreateEngine::new();
    let config = create_test_config();

    // Execute auto-create
    engine
        .execute_auto_create("Create a TODO application", &config, &output_path)
        .await
        .unwrap();

    // Verify files were created
    assert!(output_path.join("index.html").exists());
    assert!(output_path.join("app.js").exists());
    assert!(output_path.join("styles.css").exists());
    assert!(output_path.join("server.js").exists());
    assert!(output_path.join("package.json").exists());
    assert!(output_path.join("Dockerfile").exists());
    assert!(output_path.join("docker-compose.yml").exists());
    assert!(output_path.join("app.test.js").exists());
    assert!(output_path.join("README.md").exists());
    assert!(output_path.join(".gitignore").exists());
}

#[tokio::test]
async fn test_responsive_design_customization() {
    let mut engine = AutoCreateEngine::new();

    let tasks = engine
        .analyze_and_decompose("Create a mobile-responsive TODO app")
        .await
        .unwrap();

    // Should have mobile/responsive mentioned in frontend task
    let frontend_task = tasks
        .iter()
        .find(|t| t.description.contains("React"))
        .expect("Should have frontend task");

    assert!(frontend_task.description.contains("mobile-responsive"));
}

#[tokio::test]
async fn test_realtime_features_customization() {
    let mut engine = AutoCreateEngine::new();

    let tasks = engine
        .analyze_and_decompose("Create a real-time chat application")
        .await
        .unwrap();

    // Should have WebSocket task
    assert!(tasks
        .iter()
        .any(|t| t.description.contains("real-time") || t.description.contains("WebSocket")));
}

fn create_test_config() -> CcswarmConfig {
    use crate::config::{
        AgentConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig, ProjectConfig,
        RepositoryConfig, ThinkMode,
    };
    use std::collections::HashMap;

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
            name: "Test Project".to_string(),
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
                enable_proactive_mode: true,
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
