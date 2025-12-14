//! Benchmark task definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of benchmark task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Implement new functionality
    Implementation,
    /// Fix a bug
    BugFix,
    /// Refactor existing code
    Refactoring,
    /// Add tests
    Testing,
    /// Improve documentation
    Documentation,
    /// Performance optimization
    Optimization,
    /// Code review
    Review,
}

impl TaskType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            TaskType::Implementation => "Implementation",
            TaskType::BugFix => "Bug Fix",
            TaskType::Refactoring => "Refactoring",
            TaskType::Testing => "Testing",
            TaskType::Documentation => "Documentation",
            TaskType::Optimization => "Optimization",
            TaskType::Review => "Code Review",
        }
    }
}

/// Difficulty level of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TaskDifficulty {
    /// Easy task
    Easy,
    /// Medium difficulty
    Medium,
    /// Hard task
    Hard,
    /// Expert level
    Expert,
}

impl TaskDifficulty {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            TaskDifficulty::Easy => "Easy",
            TaskDifficulty::Medium => "Medium",
            TaskDifficulty::Hard => "Hard",
            TaskDifficulty::Expert => "Expert",
        }
    }

    /// Get default point multiplier
    pub fn point_multiplier(&self) -> f64 {
        match self {
            TaskDifficulty::Easy => 1.0,
            TaskDifficulty::Medium => 1.5,
            TaskDifficulty::Hard => 2.0,
            TaskDifficulty::Expert => 3.0,
        }
    }

    /// Get expected time in minutes
    pub fn expected_minutes(&self) -> u32 {
        match self {
            TaskDifficulty::Easy => 5,
            TaskDifficulty::Medium => 15,
            TaskDifficulty::Hard => 30,
            TaskDifficulty::Expert => 60,
        }
    }
}

/// A single benchmark task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    /// Unique task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Detailed description
    pub description: Option<String>,
    /// Type of task
    pub task_type: TaskType,
    /// Difficulty level
    pub difficulty: TaskDifficulty,
    /// Points for completing this task
    pub points: u32,
    /// Time limit in seconds
    pub time_limit_secs: Option<u64>,
    /// Input files or context
    pub inputs: HashMap<String, String>,
    /// Expected output for validation
    pub expected_output: Option<String>,
    /// Validation script or command
    pub validation_command: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Hints available
    pub hints: Vec<String>,
    /// Prerequisites (other task IDs)
    pub prerequisites: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BenchmarkTask {
    /// Create a new benchmark task
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            task_type: TaskType::Implementation,
            difficulty: TaskDifficulty::Medium,
            points: 10,
            time_limit_secs: None,
            inputs: HashMap::new(),
            expected_output: None,
            validation_command: None,
            tags: Vec::new(),
            hints: Vec::new(),
            prerequisites: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set task type
    pub fn with_type(mut self, task_type: TaskType) -> Self {
        self.task_type = task_type;
        self
    }

    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: TaskDifficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Set points
    pub fn with_points(mut self, points: u32) -> Self {
        self.points = points;
        self
    }

    /// Set time limit
    pub fn with_time_limit(mut self, seconds: u64) -> Self {
        self.time_limit_secs = Some(seconds);
        self
    }

    /// Add input file
    pub fn with_input(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.inputs.insert(name.into(), content.into());
        self
    }

    /// Set expected output
    pub fn with_expected_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = Some(output.into());
        self
    }

    /// Set validation command
    pub fn with_validation(mut self, command: impl Into<String>) -> Self {
        self.validation_command = Some(command.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    /// Add a prerequisite
    pub fn with_prerequisite(mut self, task_id: impl Into<String>) -> Self {
        self.prerequisites.push(task_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get adjusted points based on difficulty
    pub fn adjusted_points(&self) -> u32 {
        (self.points as f64 * self.difficulty.point_multiplier()) as u32
    }

    /// Check if task has time limit
    pub fn has_time_limit(&self) -> bool {
        self.time_limit_secs.is_some()
    }

    /// Get default time limit based on difficulty
    pub fn default_time_limit(&self) -> u64 {
        self.time_limit_secs
            .unwrap_or_else(|| (self.difficulty.expected_minutes() as u64) * 60)
    }
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskContext {
    /// Working directory
    pub working_dir: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Available files
    pub files: HashMap<String, String>,
    /// Agent configuration
    pub agent_config: HashMap<String, serde_json::Value>,
}

impl TaskContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Add file
    pub fn with_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_display() {
        assert_eq!(TaskType::Implementation.display_name(), "Implementation");
        assert_eq!(TaskType::BugFix.display_name(), "Bug Fix");
    }

    #[test]
    fn test_difficulty_display() {
        assert_eq!(TaskDifficulty::Easy.display_name(), "Easy");
        assert_eq!(TaskDifficulty::Expert.display_name(), "Expert");
    }

    #[test]
    fn test_difficulty_multiplier() {
        assert_eq!(TaskDifficulty::Easy.point_multiplier(), 1.0);
        assert_eq!(TaskDifficulty::Hard.point_multiplier(), 2.0);
    }

    #[test]
    fn test_difficulty_expected_time() {
        assert_eq!(TaskDifficulty::Easy.expected_minutes(), 5);
        assert_eq!(TaskDifficulty::Expert.expected_minutes(), 60);
    }

    #[test]
    fn test_task_creation() {
        let task = BenchmarkTask::new("test-task", "Test Task");
        assert_eq!(task.id, "test-task");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.difficulty, TaskDifficulty::Medium);
    }

    #[test]
    fn test_task_builder() {
        let task = BenchmarkTask::new("complex", "Complex Task")
            .with_description("A complex task")
            .with_type(TaskType::BugFix)
            .with_difficulty(TaskDifficulty::Hard)
            .with_points(25)
            .with_time_limit(300)
            .with_tag("rust")
            .with_hint("Check the loop bounds");

        assert_eq!(task.task_type, TaskType::BugFix);
        assert_eq!(task.difficulty, TaskDifficulty::Hard);
        assert_eq!(task.points, 25);
        assert_eq!(task.time_limit_secs, Some(300));
        assert!(task.tags.contains(&"rust".to_string()));
        assert_eq!(task.hints.len(), 1);
    }

    #[test]
    fn test_task_with_inputs() {
        let task = BenchmarkTask::new("input-test", "Input Test")
            .with_input("main.rs", "fn main() {}")
            .with_input("lib.rs", "pub fn hello() {}");

        assert_eq!(task.inputs.len(), 2);
        assert!(task.inputs.contains_key("main.rs"));
    }

    #[test]
    fn test_task_adjusted_points() {
        let easy = BenchmarkTask::new("easy", "Easy")
            .with_difficulty(TaskDifficulty::Easy)
            .with_points(10);
        assert_eq!(easy.adjusted_points(), 10);

        let hard = BenchmarkTask::new("hard", "Hard")
            .with_difficulty(TaskDifficulty::Hard)
            .with_points(10);
        assert_eq!(hard.adjusted_points(), 20);
    }

    #[test]
    fn test_task_default_time_limit() {
        let task =
            BenchmarkTask::new("no-limit", "No Limit").with_difficulty(TaskDifficulty::Medium);

        assert!(!task.has_time_limit());
        assert_eq!(task.default_time_limit(), 15 * 60); // 15 minutes

        let with_limit = task.with_time_limit(120);
        assert!(with_limit.has_time_limit());
        assert_eq!(with_limit.default_time_limit(), 120);
    }

    #[test]
    fn test_task_prerequisites() {
        let task = BenchmarkTask::new("advanced", "Advanced")
            .with_prerequisite("basic-1")
            .with_prerequisite("basic-2");

        assert_eq!(task.prerequisites.len(), 2);
    }

    #[test]
    fn test_task_context() {
        let context = TaskContext::new()
            .with_working_dir("/tmp/test")
            .with_env("RUST_LOG", "debug")
            .with_file("test.rs", "fn test() {}");

        assert_eq!(context.working_dir, Some("/tmp/test".to_string()));
        assert!(context.env.contains_key("RUST_LOG"));
        assert!(context.files.contains_key("test.rs"));
    }

    #[test]
    fn test_difficulty_ordering() {
        assert!(TaskDifficulty::Easy < TaskDifficulty::Medium);
        assert!(TaskDifficulty::Medium < TaskDifficulty::Hard);
        assert!(TaskDifficulty::Hard < TaskDifficulty::Expert);
    }
}
