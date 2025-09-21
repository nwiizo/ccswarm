use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete task execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub steps: Vec<TaskStep>,
    pub context: HashMap<String, String>,
    pub adaptive: bool,
}

/// A single step in the task plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub step_type: StepType,
    pub parallel_tasks: Vec<ParallelTask>,
    pub dependencies: Vec<String>,
    pub required_context: Vec<String>,
}

/// Type of step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Analysis,
    Execution,
    Validation,
    Review,
}

/// A task that can be executed in parallel with others
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelTask {
    pub id: String,
    pub name: String,
    pub command: String,
    pub expected_duration_ms: u64,
    pub critical: bool,
    pub expect_failure: bool,
}

/// Result from executing a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub summary: String,
    pub outputs: HashMap<String, String>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
    pub parallel_results: Vec<ParallelTaskResult>,
}

/// Result from a parallel task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelTaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl TaskPlan {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            steps: Vec::new(),
            context: HashMap::new(),
            adaptive: true,
        }
    }

    pub fn add_step(&mut self, step: TaskStep) {
        self.steps.push(step);
    }

    pub fn update_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    pub fn remove_step(&mut self, step_id: &str) {
        self.steps.retain(|s| s.id != step_id);
    }

    pub fn insert_step_after(&mut self, after_id: &str, step: TaskStep) {
        if let Some(pos) = self.steps.iter().position(|s| s.id == after_id) {
            self.steps.insert(pos + 1, step);
        }
    }
}

impl TaskStep {
    pub fn new(id: String, name: String, step_type: StepType) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            step_type,
            parallel_tasks: Vec::new(),
            dependencies: Vec::new(),
            required_context: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn add_parallel_task(&mut self, task: ParallelTask) {
        self.parallel_tasks.push(task);
    }

    pub fn depends_on(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn requires_context(mut self, context_key: String) -> Self {
        self.required_context.push(context_key);
        self
    }
}

impl StepResult {
    pub fn new(step_id: String) -> Self {
        Self {
            step_id,
            success: true,
            summary: String::new(),
            outputs: HashMap::new(),
            errors: Vec::new(),
            duration_ms: 0,
            parallel_results: Vec::new(),
        }
    }

    pub fn failed(mut self, error: String) -> Self {
        self.success = false;
        self.errors.push(error);
        self
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = summary;
        self
    }

    pub fn add_output(mut self, key: String, value: String) -> Self {
        self.outputs.insert(key, value);
        self
    }

    pub fn is_success(&self) -> bool {
        self.success && self.parallel_results.iter().all(|r| r.success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_plan_creation() {
        let mut plan = TaskPlan::new("test-task".to_string());
        assert_eq!(plan.task_id, "test-task");
        assert!(plan.steps.is_empty());
        assert!(plan.adaptive);

        let step = TaskStep::new(
            "step1".to_string(),
            "Initial Analysis".to_string(),
            StepType::Analysis,
        );
        plan.add_step(step);
        assert_eq!(plan.steps.len(), 1);
    }

    #[test]
    fn test_task_step_builder() {
        let step = TaskStep::new(
            "step1".to_string(),
            "Test Step".to_string(),
            StepType::Execution,
        )
        .with_description("Test description".to_string())
        .depends_on("step0".to_string())
        .requires_context("previous_result".to_string());

        assert_eq!(step.description, "Test description");
        assert_eq!(step.dependencies.len(), 1);
        assert_eq!(step.required_context.len(), 1);
    }

    #[test]
    fn test_step_result() {
        let mut result = StepResult::new("step1".to_string());
        assert!(result.is_success());

        result = result
            .with_summary("Step completed successfully".to_string())
            .add_output("test_count".to_string(), "42".to_string());

        assert_eq!(result.outputs.get("test_count"), Some(&"42".to_string()));

        // Test failure
        let failed_result = StepResult::new("step2".to_string()).failed("Test failed".to_string());
        assert!(!failed_result.is_success());
    }
}
