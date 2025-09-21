use anyhow::Result;
use ccswarm::agent::{Priority, Task, TaskType};
use ccswarm::config::{CcswarmConfig, ClaudeConfig};
use ccswarm::orchestrator::MasterClaude;
use tempfile::TempDir;
use tracing::{info, Level};

/// Demonstrates the integration between agent-level and master-level orchestration
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting orchestration integration demo");

    // Create temporary workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path().to_path_buf();

    // Initialize git repository
    ccswarm::git::WorktreeManager::init_if_needed(&workspace_path).await?;

    // Create a complex task that will trigger agent orchestration
    let complex_task = Task::new(
        "task-complex-1".to_string(),
        "Implement user authentication system with JWT tokens and refresh mechanism".to_string(),
        Priority::High,
        TaskType::Feature,
    )
    .with_details(
        "Create a comprehensive authentication system including:\n\
         1. User registration endpoint\n\
         2. Login endpoint with JWT generation\n\
         3. Token refresh mechanism\n\
         4. Middleware for protected routes\n\
         5. User profile management\n\
         This requires multiple steps and careful planning."
            .to_string(),
    )
    .with_duration(7200); // 2 hours

    // Create a simple task for comparison
    let simple_task = Task::new(
        "task-simple-1".to_string(),
        "Fix typo in README".to_string(),
        Priority::Low,
        TaskType::Documentation,
    )
    .with_details("Fix spelling error in the installation section".to_string())
    .with_duration(300); // 5 minutes

    // Create configuration
    let config = create_test_config();

    // Initialize Master Claude orchestrator
    let master = MasterClaude::new(config, workspace_path.clone()).await?;
    master.initialize().await?;

    info!(
        "Master Claude initialized with {} agents",
        master.agents.len()
    );

    // Add tasks to the queue
    info!("Adding complex task to queue...");
    master.add_task(complex_task).await?;

    info!("Adding simple task to queue...");
    master.add_task(simple_task).await?;

    // Start coordination in background
    let coordination_handle = tokio::spawn({
        let master_clone = master.clone();
        async move {
            if let Err(e) = master_clone.start_coordination().await {
                eprintln!("Coordination error: {}", e);
            }
        }
    });

    // Wait for tasks to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Generate status report
    let status = master.generate_status_report().await?;
    info!(
        "Status Report:\n\
         - Total tasks processed: {}\n\
         - Successful: {}\n\
         - Failed: {}\n\
         - Pending: {}",
        status.total_tasks_processed,
        status.successful_tasks,
        status.failed_tasks,
        status.pending_tasks
    );

    // Check if complex task was orchestrated
    let state = master.state.read().await;
    for (task, result) in &state.review_history {
        info!("Task {} processing details:", task);
        if let Some(output) = result
            .get(0)
            .and_then(|r| r.review_passed.then(|| "orchestrated"))
        {
            info!("  - Used orchestration: {}", output);
        }
    }

    // Shutdown
    master.shutdown().await?;
    coordination_handle.abort();

    info!("Demo completed");
    Ok(())
}

fn create_test_config() -> CcswarmConfig {
    use ccswarm::config::{
        AgentConfig, CoordinationConfig, MasterClaudeConfig, ProjectConfig, RepositoryConfig,
        ThinkMode,
    };
    use std::collections::HashMap;

    let mut agents = HashMap::new();

    // Add backend agent that will handle the complex task
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
                quality_threshold: 0.9,
                think_mode: ThinkMode::Think,
                permission_level: "supervised".to_string(),
                claude_config: ClaudeConfig::for_master(),
                enable_proactive_mode: true,
                proactive_frequency: 300,
                high_frequency: 60,
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
