//! Task Converter Module
//!
//! Provides conversion between orchestrator Task types and subagent SpawnTask types.
//! This enables seamless integration between the proactive master's decision system
//! and the parallel executor's task execution system.

use crate::agent::{Priority, Task, TaskType};
use crate::subagent::SpawnTask;

/// Conversion from Task reference to SpawnTask
///
/// This enables the ProactiveMaster to convert its internal Task representation
/// to the SpawnTask format required by the ParallelExecutor.
impl From<&Task> for SpawnTask {
    fn from(task: &Task) -> Self {
        let priority = match task.priority {
            Priority::Critical => 100,
            Priority::High => 75,
            Priority::Medium => 50,
            Priority::Low => 25,
        };

        let mut spawn_task = SpawnTask::new(&task.description)
            .with_id(&task.id)
            .with_priority(priority);

        // Add task metadata as context
        spawn_task = spawn_task.with_context(
            "task_type",
            serde_json::json!(format!("{:?}", task.task_type)),
        );
        spawn_task = spawn_task.with_context(
            "priority",
            serde_json::json!(format!("{:?}", task.priority)),
        );

        if let Some(details) = &task.details {
            spawn_task = spawn_task.with_context("details", serde_json::json!(details));
        }

        if let Some(parent_id) = &task.parent_task_id {
            spawn_task = spawn_task.with_context("parent_task_id", serde_json::json!(parent_id));
        }

        spawn_task
    }
}

/// Conversion from owned Task to SpawnTask
impl From<Task> for SpawnTask {
    fn from(task: Task) -> Self {
        SpawnTask::from(&task)
    }
}

/// Convert priority to integer value for SpawnTask
pub fn priority_to_int(priority: &Priority) -> i32 {
    match priority {
        Priority::Critical => 100,
        Priority::High => 75,
        Priority::Medium => 50,
        Priority::Low => 25,
    }
}

/// Convert task type to string for context
pub fn task_type_to_string(task_type: &TaskType) -> &'static str {
    match task_type {
        TaskType::Development => "development",
        TaskType::Testing => "testing",
        TaskType::Documentation => "documentation",
        TaskType::Infrastructure => "infrastructure",
        TaskType::Coordination => "coordination",
        TaskType::Review => "review",
        TaskType::Bugfix | TaskType::Bug => "bugfix",
        TaskType::Feature => "feature",
        TaskType::Remediation => "remediation",
        TaskType::Assistance => "assistance",
        TaskType::Research => "research",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_to_spawn_task_conversion() {
        let task = Task::new(
            "test-task-1".to_string(),
            "Implement feature X".to_string(),
            Priority::High,
            TaskType::Development,
        )
        .with_details("Additional details here".to_string());

        let spawn_task: SpawnTask = (&task).into();

        assert_eq!(spawn_task.id, "test-task-1");
        assert_eq!(spawn_task.prompt, "Implement feature X");
        assert_eq!(spawn_task.priority, 75); // High priority = 75
        assert!(spawn_task.context.contains_key("task_type"));
        assert!(spawn_task.context.contains_key("details"));
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(priority_to_int(&Priority::Critical), 100);
        assert_eq!(priority_to_int(&Priority::High), 75);
        assert_eq!(priority_to_int(&Priority::Medium), 50);
        assert_eq!(priority_to_int(&Priority::Low), 25);
    }

    #[test]
    fn test_owned_task_conversion() {
        let task = Task::new(
            "owned-task".to_string(),
            "Test owned conversion".to_string(),
            Priority::Medium,
            TaskType::Testing,
        );

        let spawn_task: SpawnTask = task.into();
        assert_eq!(spawn_task.priority, 50);
    }
}
