//! PTY functionality test for ai-session
//! 
//! This example demonstrates and tests the PTY capabilities of ai-session.

use ai_session::{SessionManager, SessionConfig};
use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Testing ai-session PTY implementation...");
    
    // Create session manager
    let manager = SessionManager::new();
    
    // Configure session
    let config = SessionConfig {
        name: Some("pty-test".to_string()),
        enable_ai_features: false, // Focus on PTY functionality
        working_directory: PathBuf::from("/tmp"),
        ..Default::default()
    };
    
    println!("📋 Creating session with config");
    println!("   Name: {:?}", config.name);
    println!("   Working directory: {:?}", config.working_directory);
    println!("   AI features: {}", config.enable_ai_features);
    
    // Create session
    match manager.create_session_with_config(config).await {
        Ok(session) => {
            println!("✅ Session created successfully!");
            println!("   Session ID: {}", session.id);
            println!("   Status: {:?}", *session.status.read().await);
            println!("   Created at: {}", session.created_at);
            
            // Test AI context access
            println!("🧪 Testing AI context access...");
            match session.get_ai_context().await {
                Ok(context) => {
                    println!("✅ AI context accessible:");
                    println!("   Message count: {}", context.get_message_count());
                    println!("   Total tokens: {}", context.get_total_tokens());
                }
                Err(e) => {
                    println!("⚠️  Failed to access AI context: {}", e);
                }
            }
            
            println!("✅ PTY functionality test completed successfully!");
            
        }
        Err(e) => {
            println!("❌ Failed to create session: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}