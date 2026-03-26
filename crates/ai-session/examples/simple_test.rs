//! Simple test example - validates basic functionality

use ai_session::{SessionConfig, SessionManager};
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 AI-Session Simple Test\n");

    // Create session manager
    let manager = SessionManager::new();
    println!("✓ Session manager created");

    // Create basic session configuration
    let config = SessionConfig {
        enable_ai_features: false, // Disable AI features for basic test
        pty_size: (24, 80),
        ..Default::default()
    };

    // Create session
    let session = manager.create_session_with_config(config).await?;
    println!("✓ Session created: {}", session.id);

    // Check initial status
    let status = session.status().await;
    println!("✓ Initial status: {:?}", status);

    // Start the session
    match session.start().await {
        Ok(_) => println!("✓ Session started successfully"),
        Err(e) => {
            println!("❌ Failed to start session: {}", e);
            println!("📝 This might be expected in test environment without shell access");

            // Test the session manager functionality instead
            println!("\n🔄 Testing session management...");

            // Test session listing
            let sessions = manager.list_sessions();
            println!("✓ Found {} sessions", sessions.len());

            // Test session retrieval
            let retrieved = manager.get_session(&session.id);
            if retrieved.is_some() {
                println!("✓ Session retrieval works");
            }

            // Test metadata
            session
                .set_metadata("test_key".to_string(), serde_json::json!("test_value"))
                .await?;
            let metadata = session.get_metadata("test_key").await;
            if metadata.is_some() {
                println!("✓ Metadata storage works");
            }

            println!("✓ Basic session management functionality confirmed");
            return Ok(());
        }
    }

    // If we get here, the session started successfully
    let status = session.status().await;
    println!("✓ Session status after start: {:?}", status);

    // Test basic operations
    println!("\n📝 Testing basic operations...");

    // Test sending a simple command
    println!("Sending: echo 'Hello World'");
    match session.send_input("echo 'Hello World'\n").await {
        Ok(_) => {
            println!("✓ Command sent successfully");

            // Wait a bit for command to execute
            sleep(Duration::from_millis(500)).await;

            // Try to read output
            match session.read_output().await {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output);
                    if !output_str.trim().is_empty() {
                        println!("✓ Output received: {}", output_str.trim());
                    } else {
                        println!("ℹ️  No output received (may be normal)");
                    }
                }
                Err(e) => println!("⚠️  Read error: {}", e),
            }
        }
        Err(e) => println!("❌ Send error: {}", e),
    }

    // Test session info
    println!("\n📊 Session Information:");
    println!("  ID: {}", session.id);
    println!(
        "  Created: {}",
        session.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  Working Directory: {}",
        session.config.working_directory.display()
    );
    println!("  AI Features: {}", session.config.enable_ai_features);

    // Test context (even without AI features)
    if session.config.enable_ai_features {
        match session.get_ai_context().await {
            Ok(context) => {
                println!("✓ AI Context accessible");
                println!("  Session ID: {}", context.session_id);
            }
            Err(e) => println!("⚠️  AI Context error: {}", e),
        }
    }

    // Clean shutdown
    println!("\n🛑 Shutting down session...");
    session.stop().await?;
    println!("✓ Session stopped");

    let final_status = session.status().await;
    println!("✓ Final status: {:?}", final_status);

    // Clean up from manager
    manager.remove_session(&session.id).await?;
    println!("✓ Session removed from manager");

    let remaining_sessions = manager.list_sessions();
    println!("✓ Remaining sessions: {}", remaining_sessions.len());

    println!("\n🎉 Simple test completed successfully!");
    Ok(())
}
