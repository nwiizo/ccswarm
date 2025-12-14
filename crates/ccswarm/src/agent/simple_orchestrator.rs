use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use std::collections::HashMap;

use crate::agent::{
    Task, TaskResult,
    orchestrator::{
        AgentOrchestrator,
        agent_orchestrator::OrchestrationBuilder,
        task_plan::{ParallelTask, ParallelTaskResult, StepResult, StepType, TaskPlan, TaskStep},
    },
    simple::SimpleClaudeAgent,
};

/// Configuration for unified plan creation (DRY pattern)
struct PlanConfig<'a> {
    /// Analysis step name
    analysis_name: &'a str,
    /// Analysis step description
    analysis_desc: &'a str,
    /// Analysis parallel tasks
    analysis_tasks: Vec<ParallelTask>,
    /// Main step name
    main_step_name: &'a str,
    /// Main step description
    main_step_desc: &'a str,
    /// Main step type
    main_step_type: StepType,
    /// Main step description content (optional)
    main_step_content: Option<String>,
    /// Validation step name
    validation_name: &'a str,
    /// Validation step description
    validation_desc: &'a str,
    /// Validation parallel tasks
    validation_tasks: Vec<ParallelTask>,
}

/// Orchestrator implementation for SimpleClaudeAgent
#[async_trait]
impl AgentOrchestrator for SimpleClaudeAgent {
    async fn analyze_task(&self, task: &Task) -> Result<TaskPlan> {
        info!("Analyzing task for orchestration: {}", task.description);

        let plan;

        // Determine task complexity and create appropriate steps
        let task_lower = task.description.to_lowercase();

        if task_lower.contains("test")
            || task_lower.contains("lint")
            || task_lower.contains("build")
        {
            // Quality check workflow
            plan = self.create_quality_check_plan(task)?;
        } else if task_lower.contains("implement")
            || task_lower.contains("create")
            || task_lower.contains("add")
        {
            // Implementation workflow
            plan = self.create_implementation_plan(task)?;
        } else if task_lower.contains("refactor") || task_lower.contains("optimize") {
            // Refactoring workflow
            plan = self.create_refactoring_plan(task)?;
        } else {
            // Default simple workflow
            plan = self.create_default_plan(task)?;
        }

        Ok(plan)
    }

    async fn execute_parallel_task(
        &self,
        task: &ParallelTask,
        _context: &HashMap<String, String>,
    ) -> Result<ParallelTaskResult> {
        debug!("Executing parallel task: {} - {}", task.id, task.command);

        // For simple agent, simulate execution
        // In a real implementation, this would use the provider system

        let start = std::time::Instant::now();
        let _duration = start.elapsed();

        // Simulate execution based on command
        let output = format!("Simulated output for: {}", task.command);

        Ok(ParallelTaskResult {
            task_id: task.id.clone(),
            success: true,
            output,
            error: None,
        })
    }

    async fn review_and_adapt(
        &self,
        results: &[StepResult],
        current_plan: &mut TaskPlan,
    ) -> Result<TaskPlan> {
        // Analyze results to determine if adaptation is needed
        let last_result = results
            .last()
            .ok_or_else(|| anyhow::anyhow!("No results available for adaptation"))?;

        // If tests failed, add a fix step
        if last_result.outputs.contains_key("test_failures") {
            let fix_step = TaskStep::new(
                format!("fix_{}", results.len()),
                "Fix Test Failures".to_string(),
                StepType::Execution,
            )
            .with_description("Fix the test failures identified in previous step".to_string())
            .depends_on(last_result.step_id.clone());

            current_plan.insert_step_after(&last_result.step_id, fix_step);
        }

        // If linting errors found, add a fix step
        if last_result.outputs.contains_key("lint_errors") {
            let fix_step = TaskStep::new(
                format!("fix_lint_{}", results.len()),
                "Fix Linting Errors".to_string(),
                StepType::Execution,
            )
            .with_description("Fix the linting errors identified".to_string());

            current_plan.insert_step_after(&last_result.step_id, fix_step);
        }

        Ok(current_plan.clone())
    }

    async fn synthesize_results(
        &self,
        task: &Task,
        results: Vec<StepResult>,
    ) -> Result<TaskResult> {
        let all_success = results.iter().all(|r| r.is_success());

        let mut summary = format!("Task '{}' orchestration complete.\n", task.description);
        summary.push_str(&format!("Executed {} steps:\n", results.len()));

        for (i, result) in results.iter().enumerate() {
            summary.push_str(&format!(
                "  {}. {} - {} ({}ms)\n",
                i + 1,
                result.step_id,
                if result.success { "✓" } else { "✗" },
                result.duration_ms
            ));
        }

        let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();
        summary.push_str(&format!("\nTotal duration: {}ms", total_duration));

        if all_success {
            Ok(TaskResult::success(
                serde_json::json!({
                    "task_id": task.id.clone(),
                    "summary": summary,
                    "total_duration_ms": total_duration
                }),
                std::time::Duration::from_millis(total_duration),
            ))
        } else {
            Ok(TaskResult::failure(
                summary,
                std::time::Duration::from_millis(total_duration),
            ))
        }
    }
}

impl SimpleClaudeAgent {
    /// Unified plan creation from configuration (DRY pattern)
    fn create_plan_from_config(&self, task: &Task, config: PlanConfig) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Step 1: Analysis
        let analysis = OrchestrationBuilder::analysis_step(
            config.analysis_name,
            config.analysis_desc,
            config.analysis_tasks,
        );
        plan.add_step(analysis);

        // Step 2: Main step (execution or validation)
        let mut main_step = TaskStep::new(
            config.main_step_name.to_string(),
            config.main_step_desc.to_string(),
            config.main_step_type,
        )
        .depends_on(config.analysis_name.to_string());

        if let Some(desc) = config.main_step_content {
            main_step = main_step.with_description(desc);
        }
        plan.add_step(main_step);

        // Step 3: Validation
        let mut validation = TaskStep::new(
            config.validation_name.to_string(),
            config.validation_desc.to_string(),
            StepType::Validation,
        )
        .depends_on(config.main_step_name.to_string());

        for task in config.validation_tasks {
            validation.add_parallel_task(task);
        }
        plan.add_step(validation);

        Ok(plan)
    }

    /// Create a quality check plan (test, lint, build)
    fn create_quality_check_plan(&self, task: &Task) -> Result<TaskPlan> {
        self.create_plan_from_config(
            task,
            PlanConfig {
                analysis_name: "analysis",
                analysis_desc: "Analyze Project Structure",
                analysis_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "check_cargo",
                        "Check Cargo.toml",
                        "cat Cargo.toml",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "check_src",
                        "Check source structure",
                        "ls -la src/",
                        false,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "check_tests",
                        "Check test structure",
                        "ls -la tests/",
                        false,
                    ),
                ],
                main_step_name: "quality_checks",
                main_step_desc: "Run Quality Checks",
                main_step_type: StepType::Validation,
                main_step_content: None,
                validation_name: "final_validation",
                validation_desc: "Final Validation",
                validation_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "run_tests",
                        "Run tests",
                        "cargo test",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "run_lint",
                        "Run clippy",
                        "cargo clippy -- -D warnings",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "check_fmt",
                        "Check formatting",
                        "cargo fmt -- --check",
                        false,
                    ),
                ],
            },
        )
    }

    /// Create an implementation plan
    fn create_implementation_plan(&self, task: &Task) -> Result<TaskPlan> {
        self.create_plan_from_config(
            task,
            PlanConfig {
                analysis_name: "analysis",
                analysis_desc: "Analyze Requirements",
                analysis_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "analyze_existing",
                        "Analyze existing code",
                        "grep -r 'impl' src/",
                        false,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "check_deps",
                        "Check dependencies",
                        "cargo tree",
                        false,
                    ),
                ],
                main_step_name: "implementation",
                main_step_desc: "Implement Feature",
                main_step_type: StepType::Execution,
                main_step_content: Some(format!("Implement: {}", task.description)),
                validation_name: "testing",
                validation_desc: "Test Implementation",
                validation_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "unit_tests",
                        "Write unit tests",
                        "cargo test",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "integration_tests",
                        "Run integration tests",
                        "cargo test --test '*'",
                        false,
                    ),
                ],
            },
        )
    }

    /// Create a refactoring plan
    fn create_refactoring_plan(&self, task: &Task) -> Result<TaskPlan> {
        self.create_plan_from_config(
            task,
            PlanConfig {
                analysis_name: "analysis",
                analysis_desc: "Analyze Code Structure",
                analysis_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "complexity",
                        "Check complexity",
                        "cargo clippy -- -W clippy::cognitive_complexity",
                        false,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "coverage",
                        "Check test coverage",
                        "cargo tarpaulin --print-summary",
                        false,
                    ),
                ],
                main_step_name: "refactor",
                main_step_desc: "Perform Refactoring",
                main_step_type: StepType::Execution,
                main_step_content: Some(
                    "Refactor code while maintaining functionality".to_string(),
                ),
                validation_name: "validate",
                validation_desc: "Validate Refactoring",
                validation_tasks: vec![
                    OrchestrationBuilder::parallel_task(
                        "test_all",
                        "Run all tests",
                        "cargo test --all",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "bench",
                        "Run benchmarks",
                        "cargo bench",
                        false,
                    ),
                ],
            },
        )
    }

    /// Create a default plan for simple tasks
    fn create_default_plan(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Single execution step
        let execution = TaskStep::new(
            "execution".to_string(),
            "Execute Task".to_string(),
            StepType::Execution,
        )
        .with_description(task.description.clone());

        plan.add_step(execution);

        Ok(plan)
    }
}
