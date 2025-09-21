//! Demonstrates agent orchestrator functionality
//!
//! This example shows how agents can break down complex tasks into
//! sequential steps with parallel subtasks for efficient execution.

use anyhow::Result;
use ccswarm::{
    agent::{
        orchestrator::{AgentOrchestrator, StepType, TaskPlan},
        simple::SimpleClaudeAgent,
        Priority, TaskBuilder,
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
    let config = ClaudeConfig::for_agent(role.name());

    SimpleClaudeAgent::new(role, workspace, config).await
}

async fn demo_frontend_orchestration(agent: &SimpleClaudeAgent) -> Result<()> {
    let task = TaskBuilder::new(
        "Create a new dashboard component with real-time data updates".to_string(),
    )
    .priority(Priority::High)
    .build();

    println!("Task: {}", task.description);
    println!("Analyzing and planning orchestration...\n");

    // Analyze task to create plan
    let plan = agent.analyze_task(&task).await?;

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

async fn demo_backend_orchestration(agent: &SimpleClaudeAgent) -> Result<()> {
    let task =
        TaskBuilder::new("Implement user authentication API with JWT tokens".to_string()).build();

    println!("Task: {}", task.description);
    println!("Creating orchestration plan...\n");

    let plan = agent.analyze_task(&task).await?;

    // Show the adaptive nature of planning
    println!("📊 Initial Plan:");
    print_plan_summary(&plan);

    // Simulate finding that we need database migrations
    println!("\n🔄 Adapting plan based on analysis...");
    println!("  → Discovered need for database migrations");
    println!("  → Adding migration step before implementation");

    Ok(())
}

async fn demo_quality_check_orchestration(agent: &SimpleClaudeAgent) -> Result<()> {
    let task =
        TaskBuilder::new("Run comprehensive quality checks: test, lint, and format".to_string())
            .build();

    println!("Task: {}", task.description);
    println!("Setting up quality check orchestration...\n");

    let plan = agent.analyze_task(&task).await?;

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
