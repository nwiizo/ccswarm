//! AI-Session integration verification test
//! 
//! This test verifies that the ai-session integration is working correctly
//! with TMux replacement and native session management.

use anyhow::Result;
use ccswarm::identity::default_frontend_role;
use ccswarm::session::{SessionManager, SessionStatus};

#[tokio::test]
async fn test_ai_session_integration_basic() -> Result<()> {
    // Test 1: Session Manager Creation
    println!("🔧 Creating SessionManager with native session support...");
    let session_manager = SessionManager::new().await?;
    println!("✅ SessionManager created successfully");

    // Test 2: Agent Session Creation
    println!("🔧 Creating agent session...");
    let session = session_manager.create_session(
        "test-agent-001".to_string(),
        default_frontend_role(),
        "/tmp".to_string(),
        Some("AI-Session Integration Test".to_string()),
        false, // Don't auto-start for testing
    ).await?;
    
    println!("✅ Agent session created: {}", session.id);
    println!("   - Agent ID: {}", session.agent_id);
    println!("   - Role: {}", session.agent_role.name());
    println!("   - Status: {}", session.status);
    
    // Test 3: Session Status Management
    println!("🔧 Testing session status transitions...");
    assert_eq!(session.status, SessionStatus::Active);
    
    // Test pause functionality
    session_manager.pause_session(&session.id).await?;
    let paused_session = session_manager.get_session(&session.id).unwrap();
    assert_eq!(paused_session.status, SessionStatus::Paused);
    println!("✅ Session pause functionality works");
    
    // Test resume functionality
    session_manager.resume_session(&session.id).await?;
    let resumed_session = session_manager.get_session(&session.id).unwrap();
    assert_eq!(resumed_session.status, SessionStatus::Active);
    println!("✅ Session resume functionality works");
    
    // Test 4: Session Cleanup
    println!("🔧 Testing session termination...");
    session_manager.terminate_session(&session.id).await?;
    let terminated_session = session_manager.get_session(&session.id).unwrap();
    assert_eq!(terminated_session.status, SessionStatus::Terminated);
    println!("✅ Session termination works");
    
    // Test 5: Session Listing
    println!("🔧 Testing session listing functionality...");
    let all_sessions = session_manager.list_sessions();
    println!("✅ Found {} sessions", all_sessions.len());
    
    let active_sessions = session_manager.list_active_sessions();
    println!("✅ Found {} active sessions", active_sessions.len());
    
    println!("🎉 All AI-Session integration tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_multiple_sessions() -> Result<()> {
    println!("🔧 Testing multiple session management...");
    
    let session_manager = SessionManager::new().await?;
    
    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..3 {
        let session = session_manager.create_session(
            format!("multi-agent-{:03}", i),
            default_frontend_role(),
            "/tmp".to_string(),
            Some(format!("Multi-session test {}", i)),
            false,
        ).await?;
        session_ids.push(session.id);
    }
    
    println!("✅ Created {} sessions", session_ids.len());
    
    // Verify all sessions exist
    let all_sessions = session_manager.list_sessions();
    assert!(all_sessions.len() >= 3);
    
    // Clean up all sessions
    for session_id in &session_ids {
        session_manager.terminate_session(session_id).await?;
    }
    
    println!("✅ Multiple session management test passed!");
    Ok(())
}

#[tokio::test] 
async fn test_session_memory_integration() -> Result<()> {
    println!("🔧 Testing session memory integration...");
    
    let session_manager = SessionManager::new().await?;
    let session = session_manager.create_session(
        "memory-test-agent".to_string(),
        default_frontend_role(),
        "/tmp".to_string(),
        Some("Memory integration test".to_string()),
        false,
    ).await?;
    
    // Test memory operations
    let mut session_copy = session.clone();
    session_copy.add_memory(
        "Test memory content".to_string(),
        ccswarm::session::memory::WorkingMemoryType::TaskInstructions,
        0.8,
    );
    
    session_copy.set_task_context(
        "test-task-001".to_string(),
        "Testing memory functionality".to_string(),
    );
    
    let memory_summary = session_copy.get_memory_summary();
    println!("✅ Memory summary generated with working memory load: {:.2}", 
             memory_summary.working_memory_load);
    
    // Cleanup
    session_manager.terminate_session(&session.id).await?;
    
    println!("✅ Session memory integration test passed!");
    Ok(())
}

#[test]
fn test_session_status_display() {
    println!("🔧 Testing session status display...");
    
    assert_eq!(SessionStatus::Active.to_string(), "Active");
    assert_eq!(SessionStatus::Paused.to_string(), "Paused");
    assert_eq!(SessionStatus::Background.to_string(), "Background");
    assert_eq!(SessionStatus::Detached.to_string(), "Detached");
    assert_eq!(SessionStatus::Terminated.to_string(), "Terminated");
    assert_eq!(
        SessionStatus::Error("test error".to_string()).to_string(),
        "Error: test error"
    );
    
    println!("✅ Session status display test passed!");
}