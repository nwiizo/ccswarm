/// Integration test for resource monitoring functionality
use anyhow::Result;
use ccswarm::identity::default_frontend_role;
use ccswarm::resource::{ResourceLimits, ResourceMonitor};
use ccswarm::session::SessionManager;
use chrono::Duration;
use std::sync::Arc;

#[tokio::test]
async fn test_resource_monitoring_integration() -> Result<()> {
    // Create resource limits with short idle timeout for testing
    let mut limits = ResourceLimits::default();
    limits.idle_timeout = Duration::seconds(5);
    limits.auto_suspend_enabled = true;

    // Create session manager with resource monitoring
    let session_manager = SessionManager::with_resource_monitoring(limits).await?;

    // Create a test session
    let session = session_manager
        .create_session(
            "test-agent".to_string(),
            default_frontend_role(),
            "/tmp/test".to_string(),
            Some("Test session".to_string()),
            false,
        )
        .await?;

    // Check that resource monitoring was started
    let usage = session_manager.get_session_resource_usage(&session.id);
    assert!(usage.is_some(), "Resource usage should be available");

    // Get efficiency stats
    let stats = session_manager.get_resource_efficiency_stats();
    assert!(stats.is_some(), "Resource stats should be available");

    let stats = stats.unwrap();
    assert_eq!(stats.total_agents, 1);
    assert_eq!(stats.active_agents, 1);
    assert_eq!(stats.suspended_agents, 0);

    // Clean up
    session_manager.terminate_session(&session.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_idle_agent_suspension() -> Result<()> {
    // Create resource limits with very short idle timeout
    let mut limits = ResourceLimits::default();
    limits.idle_timeout = Duration::milliseconds(100); // Very short for testing
    limits.auto_suspend_enabled = true;

    // Create session manager
    let session_manager = Arc::new(SessionManager::with_resource_monitoring(limits).await?);

    // Create a test session
    let session = session_manager
        .create_session(
            "idle-test-agent".to_string(),
            default_frontend_role(),
            "/tmp/test".to_string(),
            Some("Idle test session".to_string()),
            false,
        )
        .await?;

    // Wait for idle timeout
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Check for idle agents
    let suspended = session_manager.check_and_suspend_idle_agents().await?;

    // Should have suspended our agent
    assert!(!suspended.is_empty(), "Should have suspended idle agent");
    assert!(suspended.contains(&"idle-test-agent".to_string()));

    // Verify session is paused
    let current_session = session_manager.get_session(&session.id);
    assert!(current_session.is_some());
    assert_eq!(
        current_session.unwrap().status,
        ccswarm::session::SessionStatus::Paused
    );

    // Clean up
    session_manager.terminate_session(&session.id).await?;

    Ok(())
}
