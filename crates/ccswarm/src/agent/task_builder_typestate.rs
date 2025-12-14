//! Type-state pattern for TaskBuilder ensuring compile-time validation
//!
//! This module implements a type-state pattern for TaskBuilder that guarantees
//! required fields are set at compile-time, preventing incomplete tasks from
//! being built.
//!
//! ## State Transition Diagram
//! ```text
//! NoDescription → HasDescription → HasPriority → Complete
//!                       ↓              ↓           ↓
//!                    [build*]      [build*]     [build]
//!
//! * Can only build if has default values
//! ```

use super::{Priority, Task, TaskType};
use std::marker::PhantomData;
use uuid::Uuid;

// ============================================================================
// Type States - Zero-sized types for compile-time validation
// ============================================================================

/// Initial state - no description set
pub struct NoDescription;

/// Has description, but missing priority
pub struct HasDescription;

/// Has description and priority, but missing task type
pub struct HasPriority;

/// All required fields are set, ready to build
pub struct Complete;

// ============================================================================
// TaskBuilder with Type-State Pattern
// ============================================================================

/// Type-safe task builder that enforces required fields at compile time
///
/// ## Example
/// ```rust
/// // This won't compile - can't build without description
/// // let task = TypedTaskBuilder::new().build(); // ERROR!
///
/// // Correct usage - must follow state transitions
/// let task = TypedTaskBuilder::new()
///     .description("Implement user authentication")  // Required
///     .priority(Priority::High)                      // Required
///     .task_type(TaskType::Development)              // Required
///     .details("Add JWT-based auth")                 // Optional
///     .build();                                      // Now we can build!
/// ```
pub struct TypedTaskBuilder<State> {
    description: Option<String>,
    priority: Option<Priority>,
    task_type: Option<TaskType>,
    details: Option<String>,
    depends_on: Vec<String>,
    estimated_duration: Option<u64>,
    _state: PhantomData<State>,
}

// ============================================================================
// State: NoDescription - Initial state
// ============================================================================

impl TypedTaskBuilder<NoDescription> {
    /// Create a new type-safe task builder
    pub fn new() -> Self {
        Self {
            description: None,
            priority: None,
            task_type: None,
            details: None,
            depends_on: Vec::new(),
            estimated_duration: None,
            _state: PhantomData,
        }
    }

    /// Set the task description (required first step)
    pub fn description(mut self, desc: impl Into<String>) -> TypedTaskBuilder<HasDescription> {
        self.description = Some(desc.into());

        TypedTaskBuilder {
            description: self.description,
            priority: self.priority,
            task_type: self.task_type,
            details: self.details,
            depends_on: self.depends_on,
            estimated_duration: self.estimated_duration,
            _state: PhantomData,
        }
    }

    /// Parse task description with modifiers and transition to HasDescription
    /// Format: "Task description [priority] [type]"
    pub fn parse(input: &str) -> TypedTaskBuilder<Complete> {
        let (desc, priority, task_type) = Self::parse_modifiers(input);

        TypedTaskBuilder {
            description: Some(desc),
            priority: Some(priority),
            task_type: Some(task_type),
            details: None,
            depends_on: Vec::new(),
            estimated_duration: None,
            _state: PhantomData,
        }
    }

    fn parse_modifiers(desc: &str) -> (String, Priority, TaskType) {
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
}

impl Default for TypedTaskBuilder<NoDescription> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// State: HasDescription - Description is set
// ============================================================================

impl TypedTaskBuilder<HasDescription> {
    /// Set the priority (required second step)
    pub fn priority(mut self, priority: Priority) -> TypedTaskBuilder<HasPriority> {
        self.priority = Some(priority);

        TypedTaskBuilder {
            description: self.description,
            priority: self.priority,
            task_type: self.task_type,
            details: self.details,
            depends_on: self.depends_on,
            estimated_duration: self.estimated_duration,
            _state: PhantomData,
        }
    }

    /// Build with default priority and task type
    pub fn build_with_defaults(self) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: self
                .description
                .expect("Description must be set in HasDescription state"),
            priority: Priority::Medium,
            task_type: TaskType::Development,
            details: self.details,
            estimated_duration: self.estimated_duration.map(|d| d as u32),
            assigned_to: None,
            parent_task_id: None,
            quality_issues: None,
            metadata: None,
        }
    }
}

// ============================================================================
// State: HasPriority - Description and Priority are set
// ============================================================================

impl TypedTaskBuilder<HasPriority> {
    /// Set the task type (required third step)
    pub fn task_type(mut self, task_type: TaskType) -> TypedTaskBuilder<Complete> {
        self.task_type = Some(task_type);

        TypedTaskBuilder {
            description: self.description,
            priority: self.priority,
            task_type: self.task_type,
            details: self.details,
            depends_on: self.depends_on,
            estimated_duration: self.estimated_duration,
            _state: PhantomData,
        }
    }

    /// Build with default task type
    pub fn build_with_default_type(self) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: self.description.expect("Description must be set"),
            priority: self.priority.expect("Priority must be set"),
            task_type: TaskType::Development,
            details: self.details,
            estimated_duration: self.estimated_duration.map(|d| d as u32),
            assigned_to: None,
            parent_task_id: None,
            quality_issues: None,
            metadata: None,
        }
    }
}

// ============================================================================
// State: Complete - All required fields are set
// ============================================================================

impl TypedTaskBuilder<Complete> {
    /// Build the task - only available when all required fields are set
    pub fn build(self) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: self
                .description
                .expect("Description must be set in Complete state"),
            priority: self
                .priority
                .expect("Priority must be set in Complete state"),
            task_type: self
                .task_type
                .expect("TaskType must be set in Complete state"),
            details: self.details,
            estimated_duration: self.estimated_duration.map(|d| d as u32),
            assigned_to: None,
            parent_task_id: None,
            quality_issues: None,
            metadata: None,
        }
    }
}

// ============================================================================
// Optional methods available in multiple states
// ============================================================================

/// Optional configuration methods available after description is set
pub trait OptionalTaskConfig {
    /// Set task details
    fn details(self, details: impl Into<String>) -> Self;

    /// Add a dependency
    fn depends_on(self, task_id: impl Into<String>) -> Self;

    /// Set estimated duration in minutes
    fn estimated_duration(self, minutes: u64) -> Self;
}

impl OptionalTaskConfig for TypedTaskBuilder<HasDescription> {
    fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    fn estimated_duration(mut self, minutes: u64) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }
}

impl OptionalTaskConfig for TypedTaskBuilder<HasPriority> {
    fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    fn estimated_duration(mut self, minutes: u64) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }
}

impl OptionalTaskConfig for TypedTaskBuilder<Complete> {
    fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    fn estimated_duration(mut self, minutes: u64) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_safe_task_builder() {
        // This demonstrates the compile-time safety
        let task = TypedTaskBuilder::new()
            .description("Implement feature")
            .priority(Priority::High)
            .task_type(TaskType::Development)
            .details("Additional context")
            .estimated_duration(60)
            .build();

        assert_eq!(task.description, "Implement feature");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.task_type, TaskType::Development);
        assert_eq!(task.details, Some("Additional context".to_string()));
        assert_eq!(task.estimated_duration, Some(60));
    }

    #[test]
    fn test_parse_with_modifiers() {
        let task = TypedTaskBuilder::<NoDescription>::parse("Fix bug [high] [bugfix]").build();

        assert_eq!(task.description, "Fix bug");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.task_type, TaskType::Bugfix);
    }

    #[test]
    fn test_build_with_defaults() {
        let task = TypedTaskBuilder::new()
            .description("Quick task")
            .build_with_defaults();

        assert_eq!(task.description, "Quick task");
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.task_type, TaskType::Development);
    }

    // The following would not compile - demonstrating type safety:
    // #[test]
    // fn test_cannot_build_without_description() {
    //     let task = TypedTaskBuilder::new().build(); // Compilation error!
    // }

    // #[test]
    // fn test_cannot_build_without_priority() {
    //     let task = TypedTaskBuilder::new()
    //         .description("Task")
    //         .build(); // Compilation error!
    // }
}
