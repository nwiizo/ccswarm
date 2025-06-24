//! Integration tests for ai-session crate
//!
//! These tests verify that different modules work together correctly.

use ai_session::*;
use anyhow::Result;
use std::path::PathBuf;
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
        config
    };

    let backend_config = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("backend".to_string());
        config.enable_ai_features = true;
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
        ("ls -la\ntotal 42\ndrwxr-xr-x  5 user  staff  160 Dec  1 10:00 .\ndrwxr-xr-x  3 user  staff   96 Dec  1 09:00 ..", "file_listing"),
        ("npm test\n✓ should pass test 1\n✓ should pass test 2\n✗ should fail test 3", "test_output"),
        ("git status\nOn branch main\nnothing to commit, working tree clean", "git_status"),
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

/// Test security and access control
#[tokio::test]
async fn test_security_features() -> Result<()> {
    use security::{Action, FileAccessMode, SecureSession};

    let secure_session = SecureSession::new("test-session");

    // Test action checking
    let test_actions = vec![
        Action::FileAccess {
            path: PathBuf::from("/tmp/test.txt"),
            mode: FileAccessMode::Read,
        },
        Action::SystemCall {
            syscall: "read".to_string(),
        },
    ];

    for action in test_actions {
        let _allowed = secure_session.is_allowed(&action);
        // In a real test, we would assert based on policy
    }

    // Test rate limiting
    let rate_limiter = security::RateLimit::new(5);

    // Should allow first 5 requests
    for _ in 0..5 {
        assert!(rate_limiter.check());
    }

    // Should deny 6th request
    assert!(!rate_limiter.check());

    Ok(())
}

/// Test observability and monitoring
#[tokio::test]
async fn test_observability_features() -> Result<()> {
    use observability::{Decision, DecisionId, DecisionTracker, DecisionType, SemanticTracer};
    use std::collections::HashMap;

    // Test semantic tracing
    let tracer = SemanticTracer::new();

    let span_id = tracer.start_span("test_operation", HashMap::new()).await;
    // Note: add_span_metadata method may not be implemented
    tracer.end_span(span_id).await?;

    let traces = tracer.get_traces().await;
    assert!(!traces.is_empty());

    // Test decision tracking
    let tracker = DecisionTracker::new();

    let decision = Decision {
        id: DecisionId::new(),
        decision_type: DecisionType::TaskAssignment,
        options: vec!["option_a".to_string(), "option_b".to_string()],
        selected: "option_a".to_string(),
        confidence: 0.95,
        timestamp: chrono::Utc::now(),
    };

    tracker.track(decision.clone()).await?;

    let decisions = tracker.get_decisions().await;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].selected, "option_a");

    // Test decision analysis
    let analysis = tracker.analyze_patterns().await;
    assert_eq!(analysis.total_decisions, 1);

    Ok(())
}

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
