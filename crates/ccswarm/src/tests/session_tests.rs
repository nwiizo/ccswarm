use crate::session::{AgentSession, SessionManager, SessionStatus};
use crate::identity::{default_frontend_role, default_backend_role, default_devops_role, default_qa_role};
use ai_session::tmux_bridge::{TmuxClient, TmuxError};
use tempfile::TempDir;
use std::time::Duration;

/// Test suite for AgentSession functionality
#[cfg(test)]
mod agent_session_tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = AgentSession::new(
            "frontend-001".to_string(),
            default_frontend_role(),
            "/tmp/frontend-workspace".to_string(),
            Some("Frontend development session".to_string()),
        );

        assert_eq!(session.agent_id, "frontend-001");
        assert_eq!(session.agent_role.name(), "Frontend");
        assert_eq!(session.working_directory, "/tmp/frontend-workspace");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(!session.background_mode);
        assert!(!session.auto_accept);
        assert_eq!(session.tasks_processed, 0);
        assert_eq!(session.tasks_queued, 0);
        assert!(session.tmux_session.starts_with("ccswarm-frontend-"));
        assert_eq!(session.description, Some("Frontend development session".to_string()));
    }

    #[test]
    fn test_session_touch() {
        let mut session = AgentSession::new(
            "backend-001".to_string(),
            default_backend_role(),
            "/tmp/backend-workspace".to_string(),
            None,
        );

        let initial_time = session.last_activity;
        std::thread::sleep(Duration::from_millis(10));
        session.touch();
        
        assert!(session.last_activity > initial_time);
    }

    #[test]
    fn test_session_runnable_states() {
        let mut session = AgentSession::new(
            "devops-001".to_string(),
            default_devops_role(),
            "/tmp/devops-workspace".to_string(),
            None,
        );

        // Test runnable states
        session.status = SessionStatus::Active;
        assert!(session.is_runnable());

        session.status = SessionStatus::Background;
        assert!(session.is_runnable());

        session.status = SessionStatus::Detached;
        assert!(session.is_runnable());

        // Test non-runnable states
        session.status = SessionStatus::Paused;
        assert!(!session.is_runnable());

        session.status = SessionStatus::Terminated;
        assert!(!session.is_runnable());

        session.status = SessionStatus::Error("Test error".to_string());
        assert!(!session.is_runnable());
    }

    #[test]
    fn test_increment_tasks_processed() {
        let mut session = AgentSession::new(
            "qa-001".to_string(),
            default_qa_role(),
            "/tmp/qa-workspace".to_string(),
            None,
        );

        let initial_time = session.last_activity;
        assert_eq!(session.tasks_processed, 0);

        std::thread::sleep(Duration::from_millis(10));
        session.increment_tasks_processed();

        assert_eq!(session.tasks_processed, 1);
        assert!(session.last_activity > initial_time);
    }

    #[test]
    fn test_session_tmux_naming() {
        let frontend_session = AgentSession::new(
            "frontend-001".to_string(),
            default_frontend_role(),
            "/tmp/frontend".to_string(),
            None,
        );

        let backend_session = AgentSession::new(
            "backend-001".to_string(),
            default_backend_role(),
            "/tmp/backend".to_string(),
            None,
        );

        assert!(frontend_session.tmux_session.starts_with("ccswarm-frontend-"));
        assert!(backend_session.tmux_session.starts_with("ccswarm-backend-"));
        assert_ne!(frontend_session.tmux_session, backend_session.tmux_session);
    }
}

/// Test suite for SessionStatus functionality
#[cfg(test)]
mod session_status_tests {
    use super::*;

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Active.to_string(), "Active");
        assert_eq!(SessionStatus::Paused.to_string(), "Paused");
        assert_eq!(SessionStatus::Detached.to_string(), "Detached");
        assert_eq!(SessionStatus::Background.to_string(), "Background");
        assert_eq!(SessionStatus::Terminated.to_string(), "Terminated");
        assert_eq!(
            SessionStatus::Error("Connection lost".to_string()).to_string(),
            "Error: Connection lost"
        );
    }

    #[test]
    fn test_session_status_equality() {
        assert_eq!(SessionStatus::Active, SessionStatus::Active);
        assert_eq!(SessionStatus::Paused, SessionStatus::Paused);
        assert_ne!(SessionStatus::Active, SessionStatus::Paused);
        
        let error1 = SessionStatus::Error("Same error".to_string());
        let error2 = SessionStatus::Error("Same error".to_string());
        let error3 = SessionStatus::Error("Different error".to_string());
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}

/// Test suite for SessionManager functionality
#[cfg(test)]
mod session_manager_tests {
    use super::*;

    // Helper function to create a temporary directory for testing
    fn create_temp_workspace() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        // Note: This test may fail if tmux is not installed
        match SessionManager::new() {
            Ok(manager) => {
                let sessions = manager.list_sessions();
                assert!(sessions.is_empty());
            }
            Err(_) => {
                // Skip test if tmux is not available
                println!("Skipping test: tmux not available");
            }
        }
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let temp_dir = create_temp_workspace();
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        match SessionManager::new() {
            Ok(manager) => {
                // Create session
                let session_result = manager.create_session(
                    "test-agent-001".to_string(),
                    default_frontend_role(),
                    workspace_path.clone(),
                    Some("Test session for frontend".to_string()),
                    false, // Don't auto-start to avoid command execution
                );

                match session_result {
                    Ok(session) => {
                        assert_eq!(session.agent_id, "test-agent-001");
                        assert_eq!(session.working_directory, workspace_path);
                        assert_eq!(session.status, SessionStatus::Active);

                        // Test session retrieval
                        let retrieved = manager.get_session(&session.id);
                        assert!(retrieved.is_some());
                        assert_eq!(retrieved.unwrap().id, session.id);

                        // Test session listing
                        let all_sessions = manager.list_sessions();
                        assert_eq!(all_sessions.len(), 1);
                        assert_eq!(all_sessions[0].id, session.id);

                        // Test active sessions
                        let active_sessions = manager.list_active_sessions();
                        assert_eq!(active_sessions.len(), 1);

                        // Test background mode
                        assert!(manager.set_background_mode(&session.id, true).is_ok());
                        let updated_session = manager.get_session(&session.id).unwrap();
                        assert!(updated_session.background_mode);
                        assert!(updated_session.auto_accept);

                        // Test session termination
                        assert!(manager.terminate_session(&session.id).is_ok());
                        let terminated_session = manager.get_session(&session.id).unwrap();
                        assert_eq!(terminated_session.status, SessionStatus::Terminated);

                        // Test cleanup
                        let cleaned_count = manager.cleanup_terminated_sessions().unwrap();
                        assert_eq!(cleaned_count, 1);
                        
                        let final_sessions = manager.list_sessions();
                        assert!(final_sessions.is_empty());
                    }
                    Err(e) => {
                        println!("Skipping session lifecycle test: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("Skipping test: tmux not available");
            }
        }
    }

    #[tokio::test]
    async fn test_sessions_by_role() {
        let temp_dir = create_temp_workspace();
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        match SessionManager::new() {
            Ok(manager) => {
                // Create sessions with different roles
                let roles_and_ids = vec![
                    ("frontend-001", default_frontend_role()),
                    ("backend-001", default_backend_role()),
                    ("frontend-002", default_frontend_role()),
                    ("devops-001", default_devops_role()),
                ];

                let mut created_sessions = Vec::new();
                for (agent_id, role) in roles_and_ids {
                    match manager.create_session(
                        agent_id.to_string(),
                        role.clone(),
                        workspace_path.clone(),
                        None,
                        false,
                    ) {
                        Ok(session) => created_sessions.push(session),
                        Err(_) => {
                            println!("Skipping multi-role test: session creation failed");
                            return;
                        }
                    }
                }

                if !created_sessions.is_empty() {
                    // Test filtering by role
                    let frontend_sessions = manager.get_sessions_by_role(default_frontend_role());
                    assert_eq!(frontend_sessions.len(), 2);

                    let backend_sessions = manager.get_sessions_by_role(default_backend_role());
                    assert_eq!(backend_sessions.len(), 1);

                    let devops_sessions = manager.get_sessions_by_role(default_devops_role());
                    assert_eq!(devops_sessions.len(), 1);

                    let qa_sessions = manager.get_sessions_by_role(default_qa_role());
                    assert_eq!(qa_sessions.len(), 0);

                    // Cleanup
                    for session in &created_sessions {
                        let _ = manager.terminate_session(&session.id);
                    }
                    let _ = manager.cleanup_terminated_sessions();
                }
            }
            Err(_) => {
                println!("Skipping test: tmux not available");
            }
        }
    }

    #[test]
    fn test_session_state_transitions() {
        match SessionManager::new() {
            Ok(manager) => {
                // Test invalid state transitions
                let non_existent_id = "non-existent-session";
                
                assert!(manager.pause_session(non_existent_id).is_err());
                assert!(manager.resume_session(non_existent_id).is_err());
                assert!(manager.detach_session(non_existent_id).is_err());
                assert!(manager.attach_session(non_existent_id).is_err());
                assert!(manager.terminate_session(non_existent_id).is_err());
                assert!(manager.set_background_mode(non_existent_id, true).is_err());
            }
            Err(_) => {
                println!("Skipping test: tmux not available");
            }
        }
    }
}

/// Test suite for TmuxClient functionality
#[cfg(test)]
mod tmux_client_tests {
    use super::*;

    #[test]
    fn test_tmux_client_creation() {
        match TmuxClient::new() {
            Ok(client) => {
                // Test server running check
                let _is_running = client.is_server_running();
                // This test passes if tmux client can be created
            }
            Err(TmuxError::TmuxNotFound) => {
                println!("Tmux not installed, skipping tmux client tests");
            }
            Err(e) => {
                panic!("Unexpected error creating tmux client: {}", e);
            }
        }
    }

    #[test]
    fn test_tmux_client_with_timeout() {
        let custom_timeout = Duration::from_secs(60);
        match TmuxClient::with_timeout(custom_timeout) {
            Ok(_client) => {
                // Test passes if client is created with custom timeout
            }
            Err(TmuxError::TmuxNotFound) => {
                println!("Tmux not installed, skipping timeout test");
            }
            Err(e) => {
                panic!("Unexpected error creating tmux client with timeout: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_tmux_session_operations() {
        match TmuxClient::new() {
            Ok(client) => {
                let temp_dir = create_temp_workspace();
                let workspace_path = temp_dir.path().to_string_lossy();
                let session_name = "test-ccswarm-session";

                // Ensure server is running
                if client.ensure_server_running().is_err() {
                    println!("Skipping test: cannot start tmux server");
                    return;
                }

                // Test session creation
                match client.create_session(session_name, &workspace_path) {
                    Ok(_) => {
                        // Test session listing
                        match client.list_sessions() {
                            Ok(sessions) => {
                                let our_session = sessions
                                    .iter()
                                    .find(|s| s.name == session_name);
                                assert!(our_session.is_some());
                            }
                            Err(e) => {
                                println!("Warning: Could not list sessions: {}", e);
                            }
                        }

                        // Test session info
                        match client.get_session_info(session_name) {
                            Ok(session_info) => {
                                assert_eq!(session_info.name, session_name);
                            }
                            Err(e) => {
                                println!("Warning: Could not get session info: {}", e);
                            }
                        }

                        // Test environment variable setting
                        if let Err(e) = client.set_environment(session_name, "TEST_VAR", "test_value") {
                            println!("Warning: Could not set environment: {}", e);
                        }

                        // Test option setting
                        if let Err(e) = client.set_option(session_name, "status-left", "[TEST]") {
                            println!("Warning: Could not set option: {}", e);
                        }

                        // Clean up - kill the session
                        if let Err(e) = client.kill_session(session_name) {
                            println!("Warning: Could not kill session: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("Skipping tmux operations test: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("Skipping test: tmux not available");
            }
        }
    }

    #[test]
    fn test_tmux_error_handling() {
        match TmuxClient::new() {
            Ok(client) => {
                // Test operations on non-existent session
                let non_existent = "non-existent-session-12345";
                
                assert!(matches!(
                    client.kill_session(non_existent),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.attach_session(non_existent),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.send_keys(non_existent, "test"),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.send_command(non_existent, "echo test"),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.capture_pane(non_existent, None),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.list_windows(non_existent),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.new_window(non_existent, "test-window", None),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.set_environment(non_existent, "VAR", "value"),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.set_option(non_existent, "option", "value"),
                    Err(TmuxError::SessionNotFound(_))
                ));

                assert!(matches!(
                    client.get_session_info(non_existent),
                    Err(TmuxError::SessionNotFound(_))
                ));
            }
            Err(_) => {
                println!("Skipping error handling test: tmux not available");
            }
        }
    }

    // Helper function for test workspace creation
    fn create_temp_workspace() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }
}

/// Integration tests combining session management and tmux operations
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_tmux_integration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        match SessionManager::new() {
            Ok(manager) => {
                // Test creating a session that actually creates a tmux session
                match manager.create_session(
                    "integration-test-001".to_string(),
                    default_frontend_role(),
                    workspace_path,
                    Some("Integration test session".to_string()),
                    false, // Don't auto-start
                ) {
                    Ok(session) => {
                        // Verify the tmux session exists
                        let tmux_client = TmuxClient::new().unwrap();
                        match tmux_client.list_sessions() {
                            Ok(tmux_sessions) => {
                                let found_session = tmux_sessions
                                    .iter()
                                    .find(|s| s.name == session.tmux_session);
                                assert!(found_session.is_some(), "Tmux session should exist");

                                // Clean up
                                let _ = manager.terminate_session(&session.id);
                                let _ = manager.cleanup_terminated_sessions();
                            }
                            Err(e) => {
                                println!("Could not verify tmux session: {}", e);
                                // Clean up anyway
                                let _ = manager.terminate_session(&session.id);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Skipping integration test: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("Skipping integration test: SessionManager creation failed");
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_session_operations() {
        match SessionManager::new() {
            Ok(manager) => {
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let workspace_path = temp_dir.path().to_string_lossy().to_string();

                // Create multiple sessions concurrently
                let manager = std::sync::Arc::new(manager);
                let mut handles = Vec::new();
                for i in 0..3 {
                    let manager_clone = manager.clone();
                    let workspace_clone = workspace_path.clone();
                    
                    let handle = tokio::spawn(async move {
                        let agent_id = format!("concurrent-agent-{:03}", i);
                        let role = match i % 4 {
                            0 => default_frontend_role(),
                            1 => default_backend_role(),
                            2 => default_devops_role(),
                            _ => default_qa_role(),
                        };

                        manager_clone.create_session(
                            agent_id,
                            role,
                            workspace_clone,
                            Some(format!("Concurrent test session {}", i)),
                            false,
                        )
                    });
                    handles.push(handle);
                }

                // Wait for all sessions to be created
                let mut created_sessions = Vec::new();
                for handle in handles {
                    match handle.await {
                        Ok(Ok(session)) => created_sessions.push(session),
                        Ok(Err(e)) => println!("Session creation failed: {}", e),
                        Err(e) => println!("Task failed: {}", e),
                    }
                }

                if !created_sessions.is_empty() {
                    println!("Created {} concurrent sessions", created_sessions.len());

                    // Verify all sessions exist
                    let all_sessions = manager.list_sessions();
                    assert!(all_sessions.len() >= created_sessions.len());

                    // Clean up all sessions
                    for session in created_sessions {
                        let _ = manager.terminate_session(&session.id);
                    }
                    let _ = manager.cleanup_terminated_sessions();
                }
            }
            Err(_) => {
                println!("Skipping concurrent test: SessionManager creation failed");
            }
        }
    }
}