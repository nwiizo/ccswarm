//! Task builder utilities

use super::{Priority, Task, TaskType};
use uuid::Uuid;

/// Builder for creating tasks with validation
pub struct TaskBuilder {
    description: String,
    priority: Option<Priority>,
    task_type: Option<TaskType>,
    details: Option<String>,
    depends_on: Vec<String>,
    estimated_duration: Option<u64>,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new(description: String) -> Self {
        Self {
            description,
            priority: None,
            task_type: None,
            details: None,
            depends_on: Vec::new(),
            estimated_duration: None,
        }
    }

    /// Parse task description with modifiers
    /// Format: "Task description [priority] [type]"
    pub fn parse(input: &str) -> Self {
        let (desc, priority, task_type) = Self::parse_modifiers(input);

        let mut builder = Self::new(desc);
        builder.priority = Some(priority);
        builder.task_type = Some(task_type);

        builder
    }

    /// Parse modifiers from task description
    pub fn parse_modifiers(desc: &str) -> (String, Priority, TaskType) {
        let mut description = desc.to_string();
        let mut priority = Priority::Medium;
        let mut task_type = TaskType::Development;

        // Parse priority modifier [high], [medium], [low]
        if let Some(start) = description.find('[') {
            if let Some(end) = description[start..].find(']') {
                let modifier = &description[start + 1..start + end].to_lowercase();

                // Try to parse as priority
                if let Ok(p) = modifier.parse::<Priority>() {
                    priority = p;
                    description = format!(
                        "{}{}",
                        &description[..start],
                        &description[start + end + 1..]
                    )
                    .trim()
                    .to_string();
                }

                // Try to parse as task type
                if let Ok(t) = modifier.parse::<TaskType>() {
                    task_type = t;
                    description = format!(
                        "{}{}",
                        &description[..start],
                        &description[start + end + 1..]
                    )
                    .trim()
                    .to_string();
                }
            }
        }

        // Check for second modifier
        if let Some(start) = description.find('[') {
            if let Some(end) = description[start..].find(']') {
                let modifier = &description[start + 1..start + end].to_lowercase();

                // Try to parse as task type if we haven't found one yet
                if let Ok(t) = modifier.parse::<TaskType>() {
                    task_type = t;
                    description = format!(
                        "{}{}",
                        &description[..start],
                        &description[start + end + 1..]
                    )
                    .trim()
                    .to_string();
                }
            }
        }

        (description.trim().to_string(), priority, task_type)
    }

    /// Set priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set task type
    pub fn task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Set details
    pub fn details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, task_id: String) -> Self {
        self.depends_on.push(task_id);
        self
    }

    /// Set estimated duration
    pub fn estimated_duration(mut self, minutes: u64) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: self.description,
            priority: self.priority.unwrap_or(Priority::Medium),
            task_type: self.task_type.unwrap_or(TaskType::Development),
            details: self.details,
            estimated_duration: self.estimated_duration.map(|d| d as u32),
            assigned_to: None,
            parent_task_id: None,
            quality_issues: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifiers() {
        let (desc, priority, task_type) = TaskBuilder::parse_modifiers("Fix bug [high] [bug]");
        assert_eq!(desc, "Fix bug");
        assert_eq!(priority, Priority::High);
        assert_eq!(task_type, TaskType::Bugfix);

        let (desc, priority, _) = TaskBuilder::parse_modifiers("Add feature [low]");
        assert_eq!(desc, "Add feature");
        assert_eq!(priority, Priority::Low);

        let (desc, _, task_type) = TaskBuilder::parse_modifiers("Write tests [test]");
        assert_eq!(desc, "Write tests");
        assert_eq!(task_type, TaskType::Testing);
    }

    #[test]
    fn test_builder() {
        let task = TaskBuilder::new("Implement feature".to_string())
            .priority(Priority::High)
            .task_type(TaskType::Feature)
            .details("Add user authentication".to_string())
            .estimated_duration(120)
            .build();

        assert_eq!(task.description, "Implement feature");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.task_type, TaskType::Feature);
        assert_eq!(task.details, Some("Add user authentication".to_string()));
        assert_eq!(task.estimated_duration, Some(120));
    }
}
