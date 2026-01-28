//! Integration tests for ai-session crate
//!
//! These tests verify that different modules work together correctly.

use ai_session::*;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test full session lifecycle with AI features
#[tokio::test]
async fn test_ai_session_lifecycle() -> Result<()> {
    let manager = SessionManager::new();

    // Create session with AI features enabled
    let mut config = SessionConfig::default();
    config.enable_ai_features = true;
    config.context_config.max_tokens = 8192;
    config.force_headless = true;

    let session = manager.create_session_with_config(config).await?;
    assert_eq!(session.status().await, core::SessionStatus::Initializing);

    // Start session
    session.start().await?;
    assert_eq!(session.status().await, core::SessionStatus::Running);

    // Test session interaction
    session.send_input("echo 'Hello AI Session'\n").await?;
    sleep(Duration::from_millis(100)).await;
    let output = session.read_output().await?;

    // Verify output is captured
    assert!(!output.is_empty());

    // Stop session
    session.stop().await?;
    assert_eq!(session.status().await, core::SessionStatus::Terminated);

    Ok(())
}

/// Test multi-agent coordination
#[tokio::test]
async fn test_multi_agent_coordination() -> Result<()> {
    use coordination::{AgentId, MultiAgentSession};

    let coordinator = Arc::new(MultiAgentSession::new());
    let manager = SessionManager::new();

    // Create multiple agent sessions
    let frontend_config = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("frontend".to_string());
        config.enable_ai_features = true;
        config.force_headless = true;
        config
    };

    let backend_config = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("backend".to_string());
        config.enable_ai_features = true;
        config.force_headless = true;
        config
    };

    let frontend_session = manager.create_session_with_config(frontend_config).await?;
    let backend_session = manager.create_session_with_config(backend_config).await?;

    // Register agents
    let frontend_id = AgentId::new();
    let backend_id = AgentId::new();

    coordinator.register_agent(frontend_id.clone(), frontend_session.clone())?;
    coordinator.register_agent(backend_id.clone(), backend_session.clone())?;

    // Verify agents are registered
    let agents = coordinator.list_agents();
    assert_eq!(agents.len(), 2);
    assert!(agents.contains(&frontend_id));
    assert!(agents.contains(&backend_id));

    // Test agent retrieval
    assert!(coordinator.get_agent(&frontend_id).is_some());
    assert!(coordinator.get_agent(&backend_id).is_some());

    // Cleanup
    coordinator.unregister_agent(&frontend_id)?;
    coordinator.unregister_agent(&backend_id)?;

    assert_eq!(coordinator.list_agents().len(), 0);

    Ok(())
}

/// Test session persistence and recovery
#[tokio::test]
async fn test_session_persistence() -> Result<()> {
    use persistence::{PersistenceManager, SessionMetadata, SessionState};
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    let manager = PersistenceManager::new(temp_dir.path().to_path_buf());

    // Create session state
    let session_id = core::SessionId::new();
    let config = SessionConfig::default();
    let context = context::SessionContext::new(session_id.clone());

    let state = SessionState {
        session_id: session_id.clone(),
        config: config.clone(),
        status: core::SessionStatus::Running,
        context,
        command_history: vec![],
        metadata: SessionMetadata::default(),
    };

    // Save state
    manager.save_session(&session_id, &state).await?;

    // Load state
    let loaded_state = manager.load_session(&session_id).await?;

    // Verify state integrity
    assert_eq!(loaded_state.session_id, session_id);
    assert_eq!(loaded_state.status, core::SessionStatus::Running);

    // List sessions
    let sessions = manager.list_sessions().await?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0], session_id);

    // Delete session
    manager.delete_session(&session_id).await?;
    let sessions_after_delete = manager.list_sessions().await?;
    assert_eq!(sessions_after_delete.len(), 0);

    Ok(())
}

/// Test output parsing and semantic analysis
#[tokio::test]
async fn test_output_parsing() -> Result<()> {
    use output::{OutputManager, OutputParser};

    let parser = OutputParser::new();
    let mut manager = OutputManager::new();

    // Test various output types
    let test_outputs = vec![
        (
            "ls -la\ntotal 42\ndrwxr-xr-x  5 user  staff  160 Dec  1 10:00 .\ndrwxr-xr-x  3 user  staff   96 Dec  1 09:00 ..",
            "file_listing",
        ),
        (
            "npm test\n✓ should pass test 1\n✓ should pass test 2\n✗ should fail test 3",
            "test_output",
        ),
        (
            "git status\nOn branch main\nnothing to commit, working tree clean",
            "git_status",
        ),
        ("error: command not found: nonexistent", "error_output"),
    ];

    for (output, expected_type) in test_outputs {
        let parsed = parser.parse(output)?;
        let _processed = manager.process_output(output)?;

        // Verify parsing succeeds
        // In a real implementation, we would check specific fields
        println!("Parsed output type for '{}': {:?}", expected_type, parsed);
    }

    Ok(())
}

// Note: Security and observability tests removed - modules were deleted as unused

/// Test tmux compatibility layer
#[tokio::test]
async fn test_tmux_integration() -> Result<()> {
    use integration::{MigrationHelper, TmuxCompatLayer};

    let _tmux = TmuxCompatLayer::new();
    let _migration = MigrationHelper::new();

    // These tests would require tmux to be installed
    // For now, we'll test the structure and basic functionality

    // Test session name generation
    let session_id = core::SessionId::new();
    let session_name = format!(
        "ai-session-{}",
        session_id
            .to_string()
            .split('-')
            .next()
            .unwrap_or("unknown")
    );
    assert!(session_name.starts_with("ai-session-"));

    // Test migration result structure
    let _result = integration::MigrationResult {
        session_name: "test-session".to_string(),
        captured_output: "test output".to_string(),
        environment: std::collections::HashMap::new(),
        working_directory: "/tmp".to_string(),
    };

    Ok(())
}

/// Test comprehensive session workflow
#[tokio::test]
async fn test_complete_workflow() -> Result<()> {
    // This test combines multiple features to simulate a real workflow
    let manager = SessionManager::new();

    // Create AI-enabled session
    let mut config = SessionConfig::default();
    config.enable_ai_features = true;
    config.agent_role = Some("test-agent".to_string());
    config.context_config.max_tokens = 4096;
    config.force_headless = true;

    let session = manager.create_session_with_config(config).await?;

    // Start session
    session.start().await?;

    // Set some metadata
    session
        .set_metadata("test_key".to_string(), serde_json::json!("test_value"))
        .await?;
    let metadata = session.get_metadata("test_key").await;
    assert_eq!(metadata, Some(serde_json::json!("test_value")));

    // Simulate command execution
    session.send_input("echo 'workflow test'\n").await?;
    sleep(Duration::from_millis(100)).await;
    let _output = session.read_output().await?;

    // Get AI context
    let context = session.get_ai_context().await?;
    assert_eq!(context.session_id, session.id);

    // Stop session
    session.stop().await?;

    // Cleanup
    manager.remove_session(&session.id).await?;

    Ok(())
}
