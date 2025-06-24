use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use std::collections::HashMap;

use crate::agent::{
    orchestrator::{
        agent_orchestrator::OrchestrationBuilder,
        task_plan::{ParallelTask, ParallelTaskResult, StepResult, StepType, TaskPlan, TaskStep},
        AgentOrchestrator,
    },
    simple::SimpleClaudeAgent,
    Task, TaskResult,
};

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
        let last_result = results.last().unwrap();

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
    /// Create a quality check plan (test, lint, build)
    fn create_quality_check_plan(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Step 1: Initial Analysis
        let analysis = OrchestrationBuilder::analysis_step(
            "analysis",
            "Analyze Project Structure",
            vec![
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
        );
        plan.add_step(analysis);

        // Step 2: Run Quality Checks
        let mut quality_checks = TaskStep::new(
            "quality_checks".to_string(),
            "Run Quality Checks".to_string(),
            StepType::Validation,
        )
        .depends_on("analysis".to_string());

        quality_checks.add_parallel_task(OrchestrationBuilder::parallel_task(
            "run_tests",
            "Run tests",
            "cargo test",
            true,
        ));
        quality_checks.add_parallel_task(OrchestrationBuilder::parallel_task(
            "run_lint",
            "Run clippy",
            "cargo clippy -- -D warnings",
            true,
        ));
        quality_checks.add_parallel_task(OrchestrationBuilder::parallel_task(
            "check_fmt",
            "Check formatting",
            "cargo fmt -- --check",
            false,
        ));
        plan.add_step(quality_checks);

        // Step 3: Fix Issues (will be added adaptively if needed)

        // Step 4: Final Validation
        let validation =
            OrchestrationBuilder::validation_step("final_validation", "Final Validation")
                .depends_on("quality_checks".to_string());
        plan.add_step(validation);

        Ok(plan)
    }

    /// Create an implementation plan
    fn create_implementation_plan(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Step 1: Analysis and Planning
        let analysis = OrchestrationBuilder::analysis_step(
            "analysis",
            "Analyze Requirements",
            vec![
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
        );
        plan.add_step(analysis);

        // Step 2: Implementation
        let implementation = OrchestrationBuilder::execution_step(
            "implementation",
            "Implement Feature",
            vec!["analysis"],
        )
        .with_description(format!("Implement: {}", task.description));
        plan.add_step(implementation);

        // Step 3: Testing
        let mut testing = TaskStep::new(
            "testing".to_string(),
            "Test Implementation".to_string(),
            StepType::Validation,
        )
        .depends_on("implementation".to_string());

        testing.add_parallel_task(OrchestrationBuilder::parallel_task(
            "unit_tests",
            "Write unit tests",
            "cargo test",
            true,
        ));
        testing.add_parallel_task(OrchestrationBuilder::parallel_task(
            "integration_tests",
            "Run integration tests",
            "cargo test --test '*'",
            false,
        ));
        plan.add_step(testing);

        Ok(plan)
    }

    /// Create a refactoring plan
    fn create_refactoring_plan(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Step 1: Analysis
        let analysis = OrchestrationBuilder::analysis_step(
            "analysis",
            "Analyze Code Structure",
            vec![
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
        );
        plan.add_step(analysis);

        // Step 2: Refactor
        let refactor = OrchestrationBuilder::execution_step(
            "refactor",
            "Perform Refactoring",
            vec!["analysis"],
        )
        .with_description("Refactor code while maintaining functionality".to_string());
        plan.add_step(refactor);

        // Step 3: Validate
        let mut validate = TaskStep::new(
            "validate".to_string(),
            "Validate Refactoring".to_string(),
            StepType::Validation,
        )
        .depends_on("refactor".to_string());

        validate.add_parallel_task(OrchestrationBuilder::parallel_task(
            "test_all",
            "Run all tests",
            "cargo test --all",
            true,
        ));
        validate.add_parallel_task(OrchestrationBuilder::parallel_task(
            "bench",
            "Run benchmarks",
            "cargo bench",
            false,
        ));
        plan.add_step(validate);

        Ok(plan)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::TaskBuilder;
    use crate::config::ClaudeConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_simple_agent_orchestration() {
        let temp_dir = TempDir::new().unwrap();
        let config = ClaudeConfig::default();

        let agent = SimpleClaudeAgent::new(
            crate::identity::default_backend_role(),
            temp_dir.path(),
            config,
        )
        .await
        .unwrap();

        // Test quality check plan
        let task = TaskBuilder::new("Run tests and linting".to_string()).build();
        let plan = agent.analyze_task(&task).await.unwrap();

        assert!(plan.steps.len() >= 3);
        assert_eq!(plan.steps[0].name, "Analyze Project Structure");
        assert_eq!(plan.steps[1].name, "Run Quality Checks");
    }

    #[tokio::test]
    async fn test_implementation_plan() {
        let temp_dir = TempDir::new().unwrap();
        let config = ClaudeConfig::default();

        let agent = SimpleClaudeAgent::new(
            crate::identity::default_frontend_role(),
            temp_dir.path(),
            config,
        )
        .await
        .unwrap();

        let task = TaskBuilder::new("Implement new dashboard component".to_string()).build();
        let plan = agent.analyze_task(&task).await.unwrap();

        assert!(plan.steps.len() >= 3);
        assert!(plan.steps.iter().any(|s| s.name.contains("Implement")));
        assert!(plan.steps.iter().any(|s| s.name.contains("Test")));
    }
}
