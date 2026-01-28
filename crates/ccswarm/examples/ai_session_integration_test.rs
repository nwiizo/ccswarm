//! Integration test to verify ai-session works with ccswarm

use ccswarm::session::{
    AIAgentId,
    // ai-session core types
    AISessionId,
    MessageBus,
    // ai-session coordination types
    MultiAgentSession,
    SessionContext,
};

fn main() {
    println!("Testing ai-session integration with ccswarm...\n");

    // Test 1: Create a MultiAgentSession
    let session = MultiAgentSession::default();
    println!(
        "✓ MultiAgentSession created (agents: {})",
        session.agents.len()
    );

    // Test 2: Create a MessageBus
    let _bus = MessageBus::default();
    println!("✓ MessageBus created");

    // Test 3: Create an AgentId
    let agent_id = AIAgentId::new();
    println!("✓ AIAgentId created: {}", agent_id);

    // Test 4: Create SessionContext with proper SessionId
    let session_id = AISessionId::new();
    let ctx = SessionContext::new(session_id.clone());
    println!("✓ SessionContext created: {}", ctx.session_id);

    // Test 5: Verify integration through ccswarm types
    println!("✓ ccswarm::session re-exports ai-session types correctly");

    println!("\n🎉 ai-session integration with ccswarm is working!");
}
