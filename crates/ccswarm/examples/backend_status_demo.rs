/// Example demonstrating backend agent status reporting
use anyhow::Result;
use ccswarm::agent::{BackendStatusExt, ClaudeCodeAgent};
use ccswarm::config::ClaudeConfig;
use ccswarm::identity::default_backend_role;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Backend Agent Status Demo");
    println!("============================\n");

    // Create temporary directory for agent workspace
    let temp_dir = TempDir::new()?;
    println!("📁 Created workspace at: {}", temp_dir.path().display());

    // Create backend agent configuration
    let config = ClaudeConfig::for_agent("backend");

    // Create backend agent
    let agent =
        ClaudeCodeAgent::new(default_backend_role(), temp_dir.path(), "demo", config).await?;

    println!("✅ Created backend agent: {}", agent.identity.agent_id);
    println!("🏷️  Role: {}", agent.identity.specialization.name());
    println!(
        "🔧 Technologies: {:?}",
        agent.identity.specialization.technologies()
    );

    // Generate backend status
    println!("\n📊 Generating backend status report...");
    let backend_status = agent.generate_backend_status().await?;

    // Display formatted status
    println!("\n{}", backend_status.format_backend_status());

    // Check health
    if backend_status.is_backend_healthy() {
        println!("\n✅ Backend is healthy!");
    } else {
        println!("\n⚠️  Backend has issues that need attention");
    }

    // Display detailed endpoint information
    println!("\n📡 API Endpoints:");
    for endpoint in backend_status.api_endpoints.values() {
        println!(
            "  {} {} - {} ({}ms)",
            endpoint.method,
            endpoint.path,
            if endpoint.is_healthy { "✓" } else { "✗" },
            endpoint.response_time_ms.unwrap_or(0.0) as u32
        );
    }

    // Display service information
    println!("\n🔧 Active Services:");
    for service in &backend_status.active_services {
        println!(
            "  {} - {} on port {:?}",
            service.name, service.status, service.port
        );
        if !service.dependencies.is_empty() {
            println!("    Dependencies: {}", service.dependencies.join(", "));
        }
    }

    // Report status to coordination system
    println!("\n📤 Reporting status to coordination system...");
    agent.report_backend_status().await?;
    println!("✅ Status reported successfully!");

    // Demonstrate JSON output for detailed status
    println!("\n📋 JSON Status (sample):");
    let json_sample = serde_json::json!({
        "agent_id": agent.identity.agent_id,
        "role": "Backend",
        "api_health": backend_status.api_endpoints.values()
            .filter(|e| e.is_healthy)
            .count() as f64 / backend_status.api_endpoints.len() as f64,
        "database_connected": backend_status.database_status.is_connected,
        "active_services": backend_status.active_services.len(),
    });
    println!("{}", serde_json::to_string_pretty(&json_sample)?);

    println!("\n✨ Demo completed successfully!");

    Ok(())
}
