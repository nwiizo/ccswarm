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

use super::{Priority, Task, TaskBuilder, TaskType};
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
/// use ccswarm::agent::{Priority, TaskType, TypedTaskBuilder};
/// use ccswarm::agent::task_builder_typestate::OptionalTaskConfig;
///
/// let task = TypedTaskBuilder::new()
///     .description("Implement user authentication")  // Required
///     .priority(Priority::High)                      // Required
///     .task_type(TaskType::Development)              // Required
///     .details("Add JWT-based auth")                 // Optional
///     .build();                                      // Now we can build!
/// ```
///
/// Building without a description is rejected:
/// ```compile_fail
/// use ccswarm::agent::TypedTaskBuilder;
/// let task = TypedTaskBuilder::new().build();
/// ```
///
/// An incomplete builder must explicitly opt into defaults:
/// ```compile_fail
/// use ccswarm::agent::TypedTaskBuilder;
/// let task = TypedTaskBuilder::new().description("Task").build();
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

    /// Parse task description with modifiers and transition to Complete
    /// Format: `Task description [priority] [type]`
    pub fn parse(input: &str) -> TypedTaskBuilder<Complete> {
        let (description, priority, task_type) = TaskBuilder::parse_modifiers(input);
        Self::new()
            .description(description)
            .priority(priority)
            .task_type(task_type)
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
        self.priority(Priority::Medium).build_with_default_type()
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
        self.task_type(TaskType::Development).build()
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
        let make_builder = || {
            TypedTaskBuilder::new()
                .description("Quick task")
                .details("Keep this context")
                .estimated_duration(25)
        };
        let tasks = [
            (make_builder().build_with_defaults(), Priority::Medium),
            (
                make_builder()
                    .priority(Priority::High)
                    .build_with_default_type(),
                Priority::High,
            ),
        ];

        for (task, priority) in tasks {
            assert_eq!(task.description, "Quick task");
            assert_eq!(task.priority, priority);
            assert_eq!(task.task_type, TaskType::Development);
            assert_eq!(task.details.as_deref(), Some("Keep this context"));
            assert_eq!(task.estimated_duration, Some(25));
            assert!(Uuid::parse_str(&task.id).is_ok());
        }
    }

    #[test]
    fn test_parsing_matches_task_builder() {
        for input in [
            "Fix bug [high] [bugfix]",
            "  Write docs [low] [documentation]  ",
            "Investigate [unknown] [high]",
            "Keep [unfinished",
            "Unicode 🦀 [HIGH] [bugfix]",
            "",
        ] {
            let typed = TypedTaskBuilder::parse(input).build();
            let untyped = super::super::TaskBuilder::parse(input).build();
            assert_eq!(typed.description, untyped.description, "{input}");
            assert_eq!(typed.priority, untyped.priority, "{input}");
            assert_eq!(typed.task_type, untyped.task_type, "{input}");
        }
    }
}
