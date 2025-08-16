//! Demonstrates agent orchestrator functionality
//!
//! This example shows how agents can break down complex tasks into
//! sequential steps with parallel subtasks for efficient execution.

use anyhow::Result;
use ccswarm::{
    agent::{
        orchestrator::{StepType, TaskPlan, TaskStep, ParallelTask},
        simple::SimpleClaudeAgent,
        Priority, TaskBuilder, Task,
    },
    config::ClaudeConfig,
    identity::{default_backend_role, default_frontend_role},
};
use std::path::Path;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Agent Orchestrator Demo\n");

    // Create temporary workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Create agents with orchestrator capability
    let frontend_agent = create_agent(default_frontend_role(), workspace_path).await?;
    let backend_agent = create_agent(default_backend_role(), workspace_path).await?;

    // Demo 1: Frontend agent orchestrating a complex UI task
    println!("📋 Demo 1: Frontend Component Development\n");
    demo_frontend_orchestration(&frontend_agent).await?;

    println!("\n{}\n", "=".repeat(50));

    // Demo 2: Backend agent orchestrating API development
    println!("📋 Demo 2: Backend API Development\n");
    demo_backend_orchestration(&backend_agent).await?;

    println!("\n{}\n", "=".repeat(50));

    // Demo 3: Quality check orchestration
    println!("📋 Demo 3: Quality Check Workflow\n");
    demo_quality_check_orchestration(&backend_agent).await?;

    Ok(())
}

async fn create_agent(
    role: ccswarm::identity::AgentRole,
    workspace: &Path,
) -> Result<SimpleClaudeAgent> {
    let config = ClaudeConfig::for_agent(&role.name());

    SimpleClaudeAgent::new(role, workspace, config).await
}

async fn demo_frontend_orchestration(_agent: &SimpleClaudeAgent) -> Result<()> {
    let task = TaskBuilder::new(
        "Create a new dashboard component with real-time data updates".to_string(),
    )
    .with_priority(Priority::High)
    .build();

    println!("Task: {}", task.description);
    println!("Analyzing and planning orchestration...\n");

    // Create a simple plan for the task
    let mut plan = TaskPlan::new(task.id.clone());
    
    // Add steps to the plan
    let mut step1 = TaskStep::new(
        "step1".to_string(),
        "Set up React component structure".to_string(),
        StepType::Execution,
    );
    step1.add_parallel_task(ParallelTask {
        id: "create-component".to_string(),
        name: "Create component files".to_string(),
        command: "create component".to_string(),
        expected_duration_ms: 1000,
        critical: true,
        expect_failure: false,
    });
    plan.add_step(step1);
    
    let step2 = TaskStep::new(
        "step2".to_string(),
        "Implement real-time data fetching".to_string(),
        StepType::Execution,
    );
    plan.add_step(step2);
    
    let step3 = TaskStep::new(
        "step3".to_string(),
        "Add dashboard visualization".to_string(),
        StepType::Execution,
    );
    plan.add_step(step3);
    
    let step4 = TaskStep::new(
        "step4".to_string(),
        "Write tests".to_string(),
        StepType::Validation,
    );
    plan.add_step(step4);

    println!("📊 Orchestration Plan:");
    for (i, step) in plan.steps.iter().enumerate() {
        println!("  Step {}: {} ({:?})", i + 1, step.name, step.step_type);
        if !step.parallel_tasks.is_empty() {
            println!("    Parallel tasks:");
            for task in &step.parallel_tasks {
                println!(
                    "      - {} {}",
                    if task.critical { "⚠️" } else { "📌" },
                    task.name
                );
            }
        }
    }

    // In a real implementation, this would execute the plan
    println!("\n✅ Plan created successfully!");

    Ok(())
}

async fn demo_backend_orchestration(_agent: &SimpleClaudeAgent) -> Result<()> {
    let task =
        TaskBuilder::new("Implement user authentication API with JWT tokens".to_string()).build();

    println!("Task: {}", task.description);
    println!("Creating orchestration plan...\n");

    // Create orchestration plan manually since SimpleClaudeAgent doesn't have analyze_task
    let plan = create_backend_plan(&task);

    // Show the adaptive nature of planning
    println!("📊 Initial Plan:");
    print_plan_summary(&plan);

    // Simulate finding that we need database migrations
    println!("\n🔄 Adapting plan based on analysis...");
    println!("  → Discovered need for database migrations");
    println!("  → Adding migration step before implementation");

    Ok(())
}

async fn demo_quality_check_orchestration(_agent: &SimpleClaudeAgent) -> Result<()> {
    let task =
        TaskBuilder::new("Run comprehensive quality checks: test, lint, and format".to_string())
            .build();

    println!("Task: {}", task.description);
    println!("Setting up quality check orchestration...\n");

    // Create quality check plan manually
    let plan = create_quality_plan(&task);

    println!("📊 Quality Check Plan:");
    for step in &plan.steps {
        match step.step_type {
            StepType::Analysis => {
                println!("1️⃣ {} - Understand project structure", step.name);
            }
            StepType::Validation => {
                println!("2️⃣ {} - Run all checks in parallel", step.name);
                for task in &step.parallel_tasks {
                    println!("   ⚡ {}", task.name);
                }
            }
            StepType::Execution => {
                println!("3️⃣ {} - Fix any issues found", step.name);
            }
            _ => {}
        }
    }

    println!("\n💡 Benefits of orchestration:");
    println!("  • Parallel execution saves time");
    println!("  • Adaptive planning handles failures");
    println!("  • Clear progress tracking");

    Ok(())
}

fn print_plan_summary(plan: &TaskPlan) {
    for (i, step) in plan.steps.iter().enumerate() {
        println!(
            "  {}. {} ({} parallel tasks)",
            i + 1,
            step.name,
            step.parallel_tasks.len()
        );
    }
}

// Helper function to create backend plan
fn create_backend_plan(task: &Task) -> TaskPlan {
    let mut plan = TaskPlan::new(task.id.clone());
    
    let step1 = TaskStep::new(
        "analyze".to_string(),
        "Analyze requirements".to_string(),
        StepType::Analysis,
    );
    plan.add_step(step1);
    
    let mut step2 = TaskStep::new(
        "implement".to_string(),
        "Implement authentication".to_string(),
        StepType::Execution,
    );
    step2.add_parallel_task(ParallelTask {
        id: "jwt".to_string(),
        name: "Setup JWT".to_string(),
        command: "implement jwt".to_string(),
        expected_duration_ms: 2000,
        critical: true,
        expect_failure: false,
    });
    plan.add_step(step2);
    
    plan
}

// Helper function to create quality check plan
fn create_quality_plan(task: &Task) -> TaskPlan {
    let mut plan = TaskPlan::new(task.id.clone());
    
    let step1 = TaskStep::new(
        "analyze".to_string(),
        "Analyze project".to_string(),
        StepType::Analysis,
    );
    plan.add_step(step1);
    
    let mut step2 = TaskStep::new(
        "validate".to_string(),
        "Run quality checks".to_string(),
        StepType::Validation,
    );
    step2.add_parallel_task(ParallelTask {
        id: "test".to_string(),
        name: "Run tests".to_string(),
        command: "cargo test".to_string(),
        expected_duration_ms: 5000,
        critical: true,
        expect_failure: false,
    });
    step2.add_parallel_task(ParallelTask {
        id: "lint".to_string(),
        name: "Run linter".to_string(),
        command: "cargo clippy".to_string(),
        expected_duration_ms: 3000,
        critical: false,
        expect_failure: false,
    });
    plan.add_step(step2);
    
    let step3 = TaskStep::new(
        "fix".to_string(),
        "Fix issues".to_string(),
        StepType::Execution,
    );
    plan.add_step(step3);
    
    plan
}
