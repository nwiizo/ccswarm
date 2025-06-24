use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::str::FromStr;

/// Task priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for Priority {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(anyhow::anyhow!("Unknown priority: {}", s)),
        }
    }
}

/// Types of tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Development,
    Testing,
    Documentation,
    Infrastructure,
    Coordination,
    Review,
    Bugfix,
    Feature,
    Remediation, // Task to fix quality issues
}

impl FromStr for TaskType {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(TaskType::Development),
            "testing" | "test" => Ok(TaskType::Testing),
            "documentation" | "docs" => Ok(TaskType::Documentation),
            "infrastructure" | "infra" => Ok(TaskType::Infrastructure),
            "coordination" | "coord" => Ok(TaskType::Coordination),
            "review" => Ok(TaskType::Review),
            "bugfix" | "bug" => Ok(TaskType::Bugfix),
            "feature" | "feat" => Ok(TaskType::Feature),
            "remediation" | "fix" => Ok(TaskType::Remediation),
            _ => Err(anyhow::anyhow!("Unknown task type: {}", s)),
        }
    }
}

/// A task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: String,

    /// Task description
    pub description: String,

    /// Additional task details
    pub details: Option<String>,

    /// Task priority
    pub priority: Priority,

    /// Type of task
    pub task_type: TaskType,

    /// Estimated duration in seconds
    pub estimated_duration: Option<u32>,

    /// Agent assigned to this task
    pub assigned_to: Option<String>,

    /// Parent task ID (for remediation tasks)
    pub parent_task_id: Option<String>,

    /// Quality issues to fix (for remediation tasks)
    pub quality_issues: Option<Vec<String>>,
}

impl Task {
    /// Create a new task
    pub fn new(id: String, description: String, priority: Priority, task_type: TaskType) -> Self {
        Self {
            id,
            description,
            details: None,
            priority,
            task_type,
            estimated_duration: None,
            assigned_to: None,
            parent_task_id: None,
            quality_issues: None,
        }
    }

    /// Add details to the task
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Set estimated duration
    pub fn with_duration(mut self, seconds: u32) -> Self {
        self.estimated_duration = Some(seconds);
        self
    }

    /// Assign task to a specific agent
    pub fn assign_to(mut self, agent_id: String) -> Self {
        self.assigned_to = Some(agent_id);
        self
    }

    /// Set parent task ID (for remediation tasks)
    pub fn with_parent_task(mut self, parent_id: String) -> Self {
        self.parent_task_id = Some(parent_id);
        self
    }

    /// Set quality issues to fix
    pub fn with_quality_issues(mut self, issues: Vec<String>) -> Self {
        self.quality_issues = Some(issues);
        self
    }
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether the task completed successfully
    pub success: bool,

    /// Task output (JSON value)
    pub output: serde_json::Value,

    /// Error message if task failed
    pub error: Option<String>,

    /// Actual duration of task execution
    pub duration: Duration,
}

impl TaskResult {
    /// Create a successful task result
    pub fn success(output: serde_json::Value, duration: Duration) -> Self {
        Self {
            success: true,
            output,
            error: None,
            duration,
        }
    }

    /// Create a failed task result
    pub fn failure(error: String, duration: Duration) -> Self {
        Self {
            success: false,
            output: serde_json::json!({}),
            error: Some(error),
            duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "test-1".to_string(),
            "Create user component".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        assert_eq!(task.id, "test-1");
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.details.is_none());
    }

    #[test]
    fn test_task_with_details() {
        let task = Task::new(
            "test-2".to_string(),
            "Fix API bug".to_string(),
            Priority::High,
            TaskType::Bugfix,
        )
        .with_details("Users getting 500 errors on login".to_string())
        .with_duration(3600);

        assert!(task.details.is_some());
        assert_eq!(task.estimated_duration, Some(3600));
    }

    #[test]
    fn test_task_result() {
        let success_result = TaskResult::success(
            serde_json::json!({"status": "completed"}),
            Duration::from_secs(120),
        );

        assert!(success_result.success);
        assert!(success_result.error.is_none());

        let failure_result =
            TaskResult::failure("Connection timeout".to_string(), Duration::from_secs(30));

        assert!(!failure_result.success);
        assert_eq!(failure_result.error, Some("Connection timeout".to_string()));
    }
}
