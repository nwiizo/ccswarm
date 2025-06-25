/// Demo of resource monitoring and efficiency features in ccswarm
///
/// This example demonstrates:
/// 1. Starting agents with resource monitoring
/// 2. Tracking CPU and memory usage
/// 3. Automatic suspension of idle agents
/// 4. Resource efficiency statistics
use anyhow::Result;
use ccswarm::identity::{default_backend_role, default_frontend_role};
use ccswarm::resource::{ResourceLimits, ResourceMonitor};
use ccswarm::session::SessionManager;
use chrono::Duration as ChronoDuration;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 ccswarm Resource Monitoring Demo");
    println!("===================================\n");

    // Create resource limits with aggressive settings for demo
    let mut resource_limits = ResourceLimits::default();
    resource_limits.idle_timeout = ChronoDuration::seconds(30); // 30 seconds for demo
    resource_limits.idle_cpu_threshold = 5.0;
    resource_limits.auto_suspend_enabled = true;
    resource_limits.max_cpu_percent = 50.0;
    resource_limits.max_memory_bytes = 1 * 1024 * 1024 * 1024; // 1GB

    println!("📊 Resource Limits Configuration:");
    println!("  - Max CPU: {}%", resource_limits.max_cpu_percent);
    println!(
        "  - Max Memory: {} MB",
        resource_limits.max_memory_bytes / (1024 * 1024)
    );
    println!("  - Idle Timeout: 30 seconds");
    println!("  - Auto-suspend: Enabled\n");

    // Create session manager with resource monitoring
    let session_manager =
        Arc::new(SessionManager::with_resource_monitoring(resource_limits).await?);

    // Create frontend agent session
    println!("🎨 Creating Frontend Agent Session...");
    let frontend_session = session_manager
        .create_session(
            "frontend-demo".to_string(),
            default_frontend_role(),
            "./workspace/frontend".to_string(),
            Some("Frontend development agent".to_string()),
            false, // Don't auto-start
        )
        .await?;
    println!("✅ Frontend session created: {}", frontend_session.id);

    // Create backend agent session
    println!("\n🔧 Creating Backend Agent Session...");
    let backend_session = session_manager
        .create_session(
            "backend-demo".to_string(),
            default_backend_role(),
            "./workspace/backend".to_string(),
            Some("Backend API agent".to_string()),
            false,
        )
        .await?;
    println!("✅ Backend session created: {}", backend_session.id);

    // Monitor resources for a while
    println!("\n📈 Monitoring Resources (60 seconds)...");
    println!("(Agents will be suspended after 30 seconds of idle time)\n");

    for i in 0..12 {
        sleep(Duration::from_secs(5)).await;

        println!("⏱️  Time: {} seconds", (i + 1) * 5);

        // Get resource usage for each session
        if let Some(frontend_usage) =
            session_manager.get_session_resource_usage(&frontend_session.id)
        {
            println!(
                "  Frontend: CPU {:.1}%, Memory {} MB",
                frontend_usage.cpu_percent,
                frontend_usage.memory_bytes / (1024 * 1024)
            );
        }

        if let Some(backend_usage) = session_manager.get_session_resource_usage(&backend_session.id)
        {
            println!(
                "  Backend: CPU {:.1}%, Memory {} MB",
                backend_usage.cpu_percent,
                backend_usage.memory_bytes / (1024 * 1024)
            );
        }

        // Check for suspended agents after 30 seconds
        if i == 6 {
            println!("\n🔍 Checking for idle agents...");
            let suspended = session_manager.check_and_suspend_idle_agents().await?;
            if !suspended.is_empty() {
                println!("⏸️  Suspended {} idle agents:", suspended.len());
                for agent in &suspended {
                    println!("    - {}", agent);
                }
            }
        }
    }

    // Show final efficiency statistics
    println!("\n📊 Resource Efficiency Statistics:");
    println!("==================================");

    if let Some(stats) = session_manager.get_resource_efficiency_stats() {
        println!("Total Agents:        {}", stats.total_agents);
        println!("Active Agents:       {}", stats.active_agents);
        println!(
            "Suspended Agents:    {} ({:.1}%)",
            stats.suspended_agents, stats.suspension_rate
        );
        println!("Avg CPU Usage:       {:.1}%", stats.average_cpu_usage);
        println!(
            "Avg Memory Usage:    {} MB",
            stats.average_memory_usage / (1024 * 1024)
        );
        println!(
            "Total Memory Saved:  ~{} MB",
            (stats.average_memory_usage * stats.suspended_agents as u64) / (1024 * 1024)
        );
    }

    // Demonstrate resuming an agent
    println!("\n🔄 Resuming suspended agents...");
    session_manager.resume_session(&frontend_session.id).await?;
    session_manager.resume_session(&backend_session.id).await?;
    println!("✅ Agents resumed");

    // Cleanup
    println!("\n🧹 Cleaning up...");
    session_manager
        .terminate_session(&frontend_session.id)
        .await?;
    session_manager
        .terminate_session(&backend_session.id)
        .await?;
    println!("✅ Sessions terminated");

    println!("\n✨ Demo completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("  - Resource monitoring tracks CPU and memory per agent");
    println!("  - Idle agents are automatically suspended to save resources");
    println!("  - Efficiency statistics help optimize system performance");
    println!("  - Resource limits prevent agents from consuming too much");

    Ok(())
}
