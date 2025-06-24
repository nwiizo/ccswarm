use anyhow::Result;
use chrono::Utc;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

use ccswarm::config::{CcswarmConfig, ProjectConfig, RepositoryConfig, MasterClaudeConfig, ThinkMode, ClaudeConfig, AgentConfig, CoordinationConfig};
use ccswarm::orchestrator::MasterClaude;
use ccswarm::agent::{Task, Priority, TaskType};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Testing Master Claude Orchestrator with Proactive Mode");
    
    // Create temporary directory for git repo
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    
    // Initialize git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()?;
    
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;
        
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;
    
    // Create test configuration with proactive mode enabled
    let config = create_test_config();
    
    // Initialize Master Claude
    let master_claude = MasterClaude::new(config, repo_path).await?;
    println!("✅ Master Claude initialized");
    
    // Initialize the orchestrator (this starts the proactive loops)
    master_claude.initialize().await?;
    println!("✅ Orchestrator initialized");
    
    // Set objectives and milestones
    let objective_id = master_claude.set_objective(
        "Build Real-time Chat Application".to_string(),
        "Create a WebSocket-based chat app with user authentication and message history".to_string(),
        Some(Utc::now() + chrono::Duration::days(30)),
    ).await?;
    println!("🎯 Set objective: {}", objective_id);
    
    let milestone_id = master_claude.add_milestone(
        "Authentication System".to_string(),
        "Implement JWT-based user authentication".to_string(),
        Some(Utc::now() + chrono::Duration::days(14)),
    ).await?;
    println!("🏁 Added milestone: {}", milestone_id);
    
    // Start coordination in background
    let coordination_handle = {
        let master_claude_clone = master_claude.clone();
        tokio::spawn(async move {
            if let Err(e) = master_claude_clone.start_coordination().await {
                eprintln!("Coordination error: {}", e);
            }
        })
    };
    
    println!("🔄 Started coordination loop");
    
    // Add some sample tasks to trigger proactive analysis
    let tasks = vec![
        Task::new(
            "auth-backend".to_string(),
            "Create JWT authentication API endpoints".to_string(),
            Priority::High,
            TaskType::Development,
        ),
        Task::new(
            "auth-frontend".to_string(),
            "Create login and registration components".to_string(),
            Priority::High,
            TaskType::Development,
        ),
        Task::new(
            "websocket-server".to_string(),
            "Implement WebSocket server for real-time messaging".to_string(),
            Priority::Medium,
            TaskType::Development,
        ),
    ];
    
    for task in tasks {
        master_claude.add_task(task.clone()).await?;
        println!("📝 Added task: {}", task.description);
    }
    
    // Enable high-frequency proactive mode
    master_claude.enable_proactive_mode().await?;
    println!("⚡ Enabled high-frequency proactive mode");
    
    // Wait for proactive analysis cycles
    println!("⏰ Waiting for proactive analysis cycles...");
    tokio::time::sleep(Duration::from_secs(20)).await;
    
    // Trigger immediate analysis
    let decisions = master_claude.trigger_proactive_analysis().await?;
    println!("🤖 Immediate analysis generated {} decisions", decisions.len());
    
    for (i, decision) in decisions.iter().enumerate() {
        println!("  Decision {}: {:?}", i + 1, decision.decision_type);
        println!("    Reasoning: {}", decision.reasoning);
        println!("    Confidence: {:.2}", decision.confidence);
        println!("    Risk: {:?}", decision.risk_assessment);
        for action in &decision.suggested_actions {
            println!("    Action: {} - {}", action.action_type, action.description);
        }
        println!();
    }
    
    // Generate final status report
    let status = master_claude.generate_status_report().await?;
    println!("📊 Final Status Report:");
    println!("   Orchestrator ID: {}", status.orchestrator_id);
    println!("   Status: {:?}", status.status);
    println!("   Uptime: {}s", status.uptime_seconds);
    println!("   Total agents: {}", status.total_agents);
    println!("   Active agents: {}", status.active_agents);
    println!("   Total tasks processed: {}", status.total_tasks_processed);
    println!("   Successful tasks: {}", status.successful_tasks);
    println!("   Failed tasks: {}", status.failed_tasks);
    println!("   Pending tasks: {}", status.pending_tasks);
    
    // Graceful shutdown
    println!("🛑 Shutting down...");
    master_claude.shutdown().await?;
    
    // Wait for coordination to finish
    if let Err(e) = timeout(Duration::from_secs(5), coordination_handle).await {
        println!("⚠️  Coordination shutdown timeout: {:?}", e);
    }
    
    println!("✅ Test completed successfully!");
    
    Ok(())
}

fn create_test_config() -> CcswarmConfig {
    let mut agents = std::collections::HashMap::new();
    
    agents.insert(
        "frontend".to_string(),
        AgentConfig {
            specialization: "react_typescript".to_string(),
            worktree: "agents/frontend".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );
    
    agents.insert(
        "backend".to_string(),
        AgentConfig {
            specialization: "node_microservices".to_string(),
            worktree: "agents/backend".to_string(),
            branch: "feature/backend".to_string(),
            claude_config: ClaudeConfig::for_agent("backend"),
            claude_md_template: "backend_specialist".to_string(),
        },
    );
    
    agents.insert(
        "qa".to_string(),
        AgentConfig {
            specialization: "qa".to_string(),
            worktree: "agents/qa".to_string(),
            branch: "feature/qa".to_string(),
            claude_config: ClaudeConfig::for_agent("qa"),
            claude_md_template: "qa_specialist".to_string(),
        },
    );
    
    CcswarmConfig {
        project: ProjectConfig {
            name: "Chat Application Test".to_string(),
            repository: RepositoryConfig {
                url: "https://github.com/test/chat-app".to_string(),
                main_branch: "main".to_string(),
            },
            master_claude: MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.85,
                think_mode: ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: ClaudeConfig::for_master(),
                enable_proactive_mode: true,  // プロアクティブモード有効
                proactive_frequency: 10,     // 10秒間隔（テスト用）
                high_frequency: 5,           // 高頻度5秒間隔（テスト用）
            },
        },
        agents,
        coordination: CoordinationConfig {
            communication_method: "json_files".to_string(),
            sync_interval: 10,
            quality_gate_frequency: "on_commit".to_string(),
            master_review_trigger: "all_tasks_complete".to_string(),
        },
    }
}