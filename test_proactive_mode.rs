use anyhow::Result;
use chrono::Utc;
use tempfile::TempDir;
use tokio::fs;

use ccswarm::config::{CcswarmConfig, ProjectConfig, RepositoryConfig, MasterClaudeConfig, ThinkMode, ClaudeConfig, AgentConfig, CoordinationConfig};
use ccswarm::orchestrator::{MasterClaude, proactive_master::{ProactiveMaster, Objective, Milestone}};
use ccswarm::agent::{Task, Priority, TaskType, TaskResult};
use ccswarm::security::SecurityAgent;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Testing ccswarm Proactive Mode & Security Agent");
    
    // Test 1: Proactive Master standalone
    test_proactive_master().await?;
    
    // Test 2: Security Agent
    test_security_agent().await?;
    
    // Test 3: Master Claude with Proactive Mode
    test_master_claude_proactive().await?;
    
    println!("✅ All tests completed successfully!");
    
    Ok(())
}

async fn test_proactive_master() -> Result<()> {
    println!("\n📋 Test 1: Proactive Master Standalone");
    
    let proactive_master = ProactiveMaster::new().await?;
    
    // Test setting an objective
    let objective = Objective {
        id: "test-obj-1".to_string(),
        title: "Complete TODO App MVP".to_string(),
        description: "Build a full-stack TODO application with authentication".to_string(),
        deadline: Some(Utc::now() + chrono::Duration::days(30)),
        progress: 0.0,
        key_results: vec![],
    };
    
    proactive_master.set_objective(objective).await?;
    println!("✅ Set project objective successfully");
    
    // Test adding a milestone
    let milestone = Milestone {
        id: "milestone-1".to_string(),
        name: "Frontend MVP".to_string(),
        description: "Complete basic React frontend".to_string(),
        deadline: Some(Utc::now() + chrono::Duration::days(14)),
        completion_percentage: 0.0,
        dependencies: vec![],
        critical_path: true,
    };
    
    proactive_master.add_milestone(milestone).await?;
    println!("✅ Added milestone successfully");
    
    // Test task completion context update
    let task = Task::new(
        "test-task-1".to_string(),
        "Create UserProfile component".to_string(),
        Priority::High,
        TaskType::Development,
    );
    
    let result = TaskResult::success(
        serde_json::json!({"component": "UserProfile", "files_created": 3}),
        std::time::Duration::from_secs(1800), // 30 minutes
    );
    
    proactive_master.update_context_from_completion(&task, &result).await?;
    println!("✅ Updated context from task completion");
    
    Ok(())
}

async fn test_security_agent() -> Result<()> {
    println!("\n🔒 Test 2: Security Agent");
    
    // Create a temporary directory with test files
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    
    // Create test JavaScript file with security issues
    let js_file = test_dir.join("test.js");
    let vulnerable_js = r#"
// SQL Injection vulnerability
const query = "SELECT * FROM users WHERE id = " + userId;

// Hardcoded password
const password = "mySecretPassword123";

// XSS vulnerability  
document.getElementById("content").innerHTML = userInput;

// Debug mode enabled
const debug = true;

// CORS allowing all origins
app.use(cors({ origin: "*" }));
"#;
    
    fs::write(&js_file, vulnerable_js).await?;
    
    // Create test package.json
    let package_json = test_dir.join("package.json");
    let package_content = r#"{
  "name": "test-app",
  "dependencies": {
    "lodash": "4.17.15",
    "express": "4.17.1"
  }
}"#;
    fs::write(&package_json, package_content).await?;
    
    // Initialize Security Agent
    let mut security_agent = SecurityAgent::new().await?;
    println!("✅ Security Agent initialized");
    
    // Test file scanning
    let file_violations = security_agent.scan_file(&js_file).await?;
    println!("🔍 Found {} security violations in test file", file_violations.len());
    
    for violation in &file_violations {
        println!("  ⚠️  {}: {} (Line {})", 
                format!("{:?}", violation.severity),
                violation.description,
                violation.line_number.unwrap_or(0));
    }
    
    // Test directory scanning
    let scan_result = security_agent.scan_directory(test_dir).await?;
    println!("📊 Security scan results:");
    println!("   Score: {:.2}/1.00", scan_result.security_score);
    println!("   Violations: {}", scan_result.violations.len());
    println!("   Vulnerabilities: {}", scan_result.vulnerabilities.len());
    println!("   Duration: {}ms", scan_result.duration_ms);
    
    // Generate security report
    let report = security_agent.generate_security_report();
    println!("📋 Security Report:");
    println!("   Total scans: {}", report.total_scans);
    println!("   Avg security score: {:.2}", report.average_security_score);
    println!("   Critical violations: {}", report.critical_violations);
    println!("   High violations: {}", report.high_violations);
    
    Ok(())
}

async fn test_master_claude_proactive() -> Result<()> {
    println!("\n🧠 Test 3: Master Claude with Proactive Mode");
    
    // Create temporary directory for git repo
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    
    // Initialize git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()?;
    
    // Create test ccswarm configuration
    let config = create_test_config();
    
    // Initialize Master Claude
    let master_claude = MasterClaude::new(config, repo_path).await?;
    println!("✅ Master Claude initialized");
    
    // Test setting objectives and milestones
    let objective_id = master_claude.set_objective(
        "Build Chat Application".to_string(),
        "Create a real-time chat app with WebSocket support".to_string(),
        Some(Utc::now() + chrono::Duration::days(60)),
    ).await?;
    println!("✅ Set objective: {}", objective_id);
    
    let milestone_id = master_claude.add_milestone(
        "WebSocket Integration".to_string(),
        "Implement real-time messaging with WebSocket".to_string(),
        Some(Utc::now() + chrono::Duration::days(30)),
    ).await?;
    println!("✅ Added milestone: {}", milestone_id);
    
    // Test triggering proactive analysis
    let decisions = master_claude.trigger_proactive_analysis().await?;
    println!("🤖 Proactive analysis generated {} decisions", decisions.len());
    
    for decision in &decisions {
        println!("  📝 Decision: {:?}", decision.decision_type);
        println!("     Reasoning: {}", decision.reasoning);
        println!("     Confidence: {:.2}", decision.confidence);
        println!("     Risk: {:?}", decision.risk_assessment);
    }
    
    // Test adding a task and simulating completion
    let test_task = Task::new(
        "chat-component".to_string(),
        "Create ChatMessage component".to_string(),
        Priority::High,
        TaskType::Development,
    );
    
    master_claude.add_task(test_task).await?;
    println!("✅ Added test task to queue");
    
    // Generate status report
    let status_report = master_claude.generate_status_report().await?;
    println!("📊 Master Claude Status:");
    println!("   Orchestrator ID: {}", status_report.orchestrator_id);
    println!("   Status: {:?}", status_report.status);
    println!("   Total agents: {}", status_report.total_agents);
    println!("   Active agents: {}", status_report.active_agents);
    println!("   Total tasks processed: {}", status_report.total_tasks_processed);
    println!("   Pending tasks: {}", status_report.pending_tasks);
    
    Ok(())
}

fn create_test_config() -> CcswarmConfig {
    let mut agents = std::collections::HashMap::new();
    
    agents.insert(
        "frontend".to_string(),
        AgentConfig {
            specialization: "frontend".to_string(),
            worktree: "agents/frontend".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );
    
    agents.insert(
        "backend".to_string(),
        AgentConfig {
            specialization: "backend".to_string(),
            worktree: "agents/backend".to_string(),
            branch: "feature/backend".to_string(),
            claude_config: ClaudeConfig::for_agent("backend"),
            claude_md_template: "backend_specialist".to_string(),
        },
    );
    
    CcswarmConfig {
        project: ProjectConfig {
            name: "Test Project".to_string(),
            repository: RepositoryConfig {
                url: "https://github.com/test/repo".to_string(),
                main_branch: "main".to_string(),
            },
            master_claude: MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.85,
                think_mode: ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: ClaudeConfig::for_master(),
                enable_proactive_mode: true,  // デフォルトで有効
                proactive_frequency: 30,     // 30秒間隔
                high_frequency: 15,          // 高頻度モード15秒間隔
            },
        },
        agents,
        coordination: CoordinationConfig {
            communication_method: "json_files".to_string(),
            sync_interval: 30,
            quality_gate_frequency: "on_commit".to_string(),
            master_review_trigger: "all_tasks_complete".to_string(),
        },
    }
}