use super::task_plan::{
    ParallelTask, ParallelTaskResult, StepResult, StepType, TaskPlan, TaskStep,
};
use crate::agent::task::{Task, TaskResult};
use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use log::{debug, info};
use std::time::Instant;

/// Trait for agents to orchestrate complex tasks
#[async_trait]
pub trait AgentOrchestrator: Send + Sync {
    /// Orchestrate a complex task by breaking it into steps
    async fn orchestrate_task(&self, task: &Task) -> Result<TaskResult> {
        info!("Starting orchestration for task: {}", task.id);

        // Step 1: Initial Analysis
        let plan = self.analyze_task(task).await?;
        debug!("Created plan with {} steps", plan.steps.len());

        // Execute steps sequentially
        let mut all_results = Vec::new();
        let mut current_plan = plan;

        for (index, step) in current_plan.steps.clone().iter().enumerate() {
            info!(
                "Executing step {}/{}: {}",
                index + 1,
                current_plan.steps.len(),
                step.name
            );

            // Execute current step
            let step_result = self.execute_step(step, &current_plan.context).await?;

            // Update context with step outputs
            for (key, value) in &step_result.outputs {
                current_plan.update_context(key.clone(), value.clone());
            }

            all_results.push(step_result);

            // Review and adapt if this isn't the last step
            if index < current_plan.steps.len() - 1 && current_plan.adaptive {
                debug!("Reviewing progress and adapting plan...");
                if let Ok(adapted_plan) =
                    self.review_and_adapt(&all_results, &mut current_plan).await
                {
                    current_plan = adapted_plan;
                    info!(
                        "Plan adapted, now has {} remaining steps",
                        current_plan.steps.len() - index - 1
                    );
                }
            }
        }

        // Synthesize final result
        self.synthesize_results(task, all_results).await
    }

    /// Analyze task and create execution plan
    async fn analyze_task(&self, task: &Task) -> Result<TaskPlan>;

    /// Execute a single step with its parallel tasks
    async fn execute_step(
        &self,
        step: &TaskStep,
        context: &std::collections::HashMap<String, String>,
    ) -> Result<StepResult> {
        let start = Instant::now();
        let mut result = StepResult::new(step.id.clone());

        // Execute parallel tasks
        if !step.parallel_tasks.is_empty() {
            info!(
                "Executing {} parallel tasks for step: {}",
                step.parallel_tasks.len(),
                step.name
            );

            let futures: Vec<_> = step
                .parallel_tasks
                .iter()
                .map(|task| self.execute_parallel_task(task, context))
                .collect();

            let parallel_results = join_all(futures).await;

            // Collect results
            for (task, task_result) in step.parallel_tasks.iter().zip(parallel_results) {
                match task_result {
                    Ok(res) => {
                        if !res.success && task.critical {
                            result = result.failed(format!("Critical task '{}' failed", task.name));
                        }
                        result.parallel_results.push(res);
                    }
                    Err(e) => {
                        let error_msg = format!("Task '{}' error: {}", task.name, e);
                        if task.critical {
                            result = result.failed(error_msg);
                        } else {
                            result.errors.push(error_msg);
                        }
                    }
                }
            }
        }

        // Generate step summary
        let summary = self.generate_step_summary(step, &result).await?;
        result = result.with_summary(summary);

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Execute a single parallel task
    async fn execute_parallel_task(
        &self,
        task: &ParallelTask,
        context: &std::collections::HashMap<String, String>,
    ) -> Result<ParallelTaskResult>;

    /// Review results and adapt remaining plan
    async fn review_and_adapt(
        &self,
        _results: &[StepResult],
        current_plan: &mut TaskPlan,
    ) -> Result<TaskPlan> {
        // Default implementation: no adaptation
        Ok(current_plan.clone())
    }

    /// Synthesize all step results into final task result
    async fn synthesize_results(&self, task: &Task, results: Vec<StepResult>)
        -> Result<TaskResult>;

    /// Generate summary for a completed step
    async fn generate_step_summary(&self, step: &TaskStep, result: &StepResult) -> Result<String> {
        let success_count = result.parallel_results.iter().filter(|r| r.success).count();
        let total_count = result.parallel_results.len();

        Ok(format!(
            "Step '{}' completed: {}/{} tasks successful. Duration: {}ms",
            step.name, success_count, total_count, result.duration_ms
        ))
    }
}

/// Helper functions for building orchestration plans
pub struct OrchestrationBuilder;

impl OrchestrationBuilder {
    /// Create an analysis step
    pub fn analysis_step(id: &str, name: &str, tasks: Vec<ParallelTask>) -> TaskStep {
        let mut step = TaskStep::new(id.to_string(), name.to_string(), StepType::Analysis);
        for task in tasks {
            step.add_parallel_task(task);
        }
        step
    }

    /// Create an execution step
    pub fn execution_step(id: &str, name: &str, dependencies: Vec<&str>) -> TaskStep {
        let mut step = TaskStep::new(id.to_string(), name.to_string(), StepType::Execution);
        for dep in dependencies {
            step = step.depends_on(dep.to_string());
        }
        step
    }

    /// Create a validation step
    pub fn validation_step(id: &str, name: &str) -> TaskStep {
        TaskStep::new(id.to_string(), name.to_string(), StepType::Validation)
    }

    /// Create a parallel task
    pub fn parallel_task(id: &str, name: &str, command: &str, critical: bool) -> ParallelTask {
        ParallelTask {
            id: id.to_string(),
            name: name.to_string(),
            command: command.to_string(),
            expected_duration_ms: 1000,
            critical,
            expect_failure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::TaskBuilder;
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct TestOrchestrator;

    #[async_trait]
    impl AgentOrchestrator for TestOrchestrator {
        async fn analyze_task(&self, task: &Task) -> Result<TaskPlan> {
            let mut plan = TaskPlan::new(task.id.clone());

            // Add analysis step
            let analysis = OrchestrationBuilder::analysis_step(
                "step1",
                "Initial Analysis",
                vec![
                    OrchestrationBuilder::parallel_task(
                        "analyze_deps",
                        "Analyze Dependencies",
                        "cargo tree",
                        true,
                    ),
                    OrchestrationBuilder::parallel_task(
                        "check_files",
                        "Check Files",
                        "ls -la",
                        false,
                    ),
                ],
            );
            plan.add_step(analysis);

            Ok(plan)
        }

        async fn execute_parallel_task(
            &self,
            task: &ParallelTask,
            _context: &HashMap<String, String>,
        ) -> Result<ParallelTaskResult> {
            Ok(ParallelTaskResult {
                task_id: task.id.clone(),
                success: true,
                output: format!("Executed: {}", task.command),
                error: None,
            })
        }

        async fn synthesize_results(
            &self,
            task: &Task,
            results: Vec<StepResult>,
        ) -> Result<TaskResult> {
            let all_success = results.iter().all(|r| r.is_success());
            if all_success {
                Ok(TaskResult::success(
                    serde_json::json!({
                        "task_id": task.id.clone(),
                        "message": "Orchestration complete"
                    }),
                    std::time::Duration::from_secs(0),
                ))
            } else {
                Ok(TaskResult::failure(
                    "Orchestration failed".to_string(),
                    std::time::Duration::from_secs(0),
                ))
            }
        }
    }

    #[tokio::test]
    async fn test_orchestration() {
        let orchestrator = TestOrchestrator;
        let task = TaskBuilder::new("Test orchestration".to_string()).build();

        let result = orchestrator.orchestrate_task(&task).await.unwrap();
        assert!(result.success);
    }
}
