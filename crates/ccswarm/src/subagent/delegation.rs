/// Delegation system for Claude Code subagents
///
/// This module handles task delegation from Master Claude to subagents,
/// using Claude Code's native Task tool functionality.
use super::{SubagentError, SubagentResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a task that can be delegated to a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedTask {
    /// Unique identifier for this task
    pub id: String,

    /// Brief description of the task (3-5 words)
    pub description: String,

    /// Detailed prompt for the subagent
    pub prompt: String,

    /// Type of subagent to use
    pub subagent_type: String,

    /// Optional context to pass to the subagent
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,

    /// Priority level
    #[serde(default)]
    pub priority: TaskPriority,

    /// Expected outputs
    #[serde(default)]
    pub expected_outputs: Vec<String>,
}

/// Priority levels for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}


/// Result from a delegated task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID this result corresponds to
    pub task_id: String,

    /// Whether the task was successful
    pub success: bool,

    /// The actual result content
    pub result: serde_json::Value,

    /// Any error message if failed
    pub error: Option<String>,

    /// Metadata about the execution
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Manages task delegation to subagents
pub struct DelegationManager {
    /// Active delegated tasks
    active_tasks: HashMap<String, DelegatedTask>,

    /// Completed task results
    completed_tasks: HashMap<String, TaskResult>,
}

impl DelegationManager {
    /// Create a new delegation manager
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            completed_tasks: HashMap::new(),
        }
    }

    /// Create a task for delegation
    pub fn create_task(
        &mut self,
        description: String,
        prompt: String,
        subagent_type: String,
        priority: TaskPriority,
    ) -> DelegatedTask {
        let task_id = format!("task-{}", uuid::Uuid::new_v4());

        let task = DelegatedTask {
            id: task_id.clone(),
            description,
            prompt,
            subagent_type,
            context: HashMap::new(),
            priority,
            expected_outputs: Vec::new(),
        };

        self.active_tasks.insert(task_id, task.clone());
        task
    }

    /// Analyze a user request and determine delegation strategy
    pub fn analyze_for_delegation(&self, request: &str) -> SubagentResult<Vec<DelegatedTask>> {
        let mut tasks = Vec::new();

        // Analyze the request for different aspects
        let needs_frontend = self.detect_frontend_work(request);
        let needs_backend = self.detect_backend_work(request);
        let needs_devops = self.detect_devops_work(request);
        let needs_qa = self.detect_qa_work(request);
        let needs_semantic = self.detect_semantic_analysis(request);

        // Create appropriate delegation tasks
        if needs_frontend {
            tasks.push(self.create_frontend_task(request));
        }

        if needs_backend {
            tasks.push(self.create_backend_task(request));
        }

        if needs_devops {
            tasks.push(self.create_devops_task(request));
        }

        if needs_qa {
            tasks.push(self.create_qa_task(request));
        }

        if needs_semantic {
            tasks.push(self.create_semantic_task(request));
        }

        // If no specific needs detected, create a general task
        if tasks.is_empty() {
            tasks.push(self.create_general_task(request));
        }

        Ok(tasks)
    }

    /// Detect if frontend work is needed
    fn detect_frontend_work(&self, request: &str) -> bool {
        let frontend_keywords = [
            "ui",
            "frontend",
            "react",
            "vue",
            "component",
            "style",
            "css",
            "layout",
            "responsive",
            "design",
            "interface",
            "form",
            "button",
        ];

        let request_lower = request.to_lowercase();
        frontend_keywords
            .iter()
            .any(|keyword| request_lower.contains(keyword))
    }

    /// Detect if backend work is needed
    fn detect_backend_work(&self, request: &str) -> bool {
        let backend_keywords = [
            "api",
            "backend",
            "database",
            "server",
            "endpoint",
            "rest",
            "graphql",
            "authentication",
            "authorization",
            "model",
            "schema",
        ];

        let request_lower = request.to_lowercase();
        backend_keywords
            .iter()
            .any(|keyword| request_lower.contains(keyword))
    }

    /// Detect if DevOps work is needed
    fn detect_devops_work(&self, request: &str) -> bool {
        let devops_keywords = [
            "docker",
            "kubernetes",
            "deploy",
            "ci/cd",
            "pipeline",
            "container",
            "infrastructure",
            "aws",
            "azure",
            "gcp",
            "terraform",
            "ansible",
        ];

        let request_lower = request.to_lowercase();
        devops_keywords
            .iter()
            .any(|keyword| request_lower.contains(keyword))
    }

    /// Detect if QA work is needed
    fn detect_qa_work(&self, request: &str) -> bool {
        let qa_keywords = [
            "test",
            "testing",
            "qa",
            "quality",
            "coverage",
            "unit test",
            "integration test",
            "e2e",
            "bug",
            "validation",
            "verification",
        ];

        let request_lower = request.to_lowercase();
        qa_keywords
            .iter()
            .any(|keyword| request_lower.contains(keyword))
    }

    /// Detect if semantic analysis is needed
    fn detect_semantic_analysis(&self, request: &str) -> bool {
        let semantic_keywords = [
            "analyze",
            "understand",
            "refactor",
            "optimize",
            "pattern",
            "architecture",
            "structure",
            "dependency",
            "complexity",
            "metrics",
        ];

        let request_lower = request.to_lowercase();
        semantic_keywords
            .iter()
            .any(|keyword| request_lower.contains(keyword))
    }

    /// Create a frontend task
    fn create_frontend_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-frontend-{}", uuid::Uuid::new_v4()),
            description: "Frontend implementation".to_string(),
            prompt: format!(
                "As a frontend specialist, implement the following:\n\n{}\n\n\
                Focus on UI/UX, component architecture, and responsive design.",
                request
            ),
            subagent_type: "frontend-specialist".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::High,
            expected_outputs: vec![
                "Component implementation".to_string(),
                "Style definitions".to_string(),
                "State management".to_string(),
            ],
        }
    }

    /// Create a backend task
    fn create_backend_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-backend-{}", uuid::Uuid::new_v4()),
            description: "Backend development".to_string(),
            prompt: format!(
                "As a backend specialist, implement the following:\n\n{}\n\n\
                Focus on API design, database optimization, and business logic.",
                request
            ),
            subagent_type: "backend-specialist".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::High,
            expected_outputs: vec![
                "API endpoints".to_string(),
                "Database schemas".to_string(),
                "Business logic".to_string(),
            ],
        }
    }

    /// Create a DevOps task
    fn create_devops_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-devops-{}", uuid::Uuid::new_v4()),
            description: "DevOps configuration".to_string(),
            prompt: format!(
                "As a DevOps specialist, implement the following:\n\n{}\n\n\
                Focus on containerization, CI/CD, and infrastructure.",
                request
            ),
            subagent_type: "devops-specialist".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::Medium,
            expected_outputs: vec![
                "Docker configuration".to_string(),
                "CI/CD pipeline".to_string(),
                "Deployment scripts".to_string(),
            ],
        }
    }

    /// Create a QA task
    fn create_qa_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-qa-{}", uuid::Uuid::new_v4()),
            description: "Quality assurance".to_string(),
            prompt: format!(
                "As a QA specialist, implement the following:\n\n{}\n\n\
                Focus on comprehensive testing and quality metrics.",
                request
            ),
            subagent_type: "qa-specialist".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::Medium,
            expected_outputs: vec![
                "Test suites".to_string(),
                "Coverage reports".to_string(),
                "Quality metrics".to_string(),
            ],
        }
    }

    /// Create a semantic analysis task
    fn create_semantic_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-semantic-{}", uuid::Uuid::new_v4()),
            description: "Semantic analysis".to_string(),
            prompt: format!(
                "Perform semantic analysis:\n\n{}\n\n\
                Use semantic tools to understand code structure and patterns.",
                request
            ),
            subagent_type: "semantic-analyst".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::High,
            expected_outputs: vec![
                "Code analysis".to_string(),
                "Pattern identification".to_string(),
                "Improvement suggestions".to_string(),
            ],
        }
    }

    /// Create a general task
    fn create_general_task(&self, request: &str) -> DelegatedTask {
        DelegatedTask {
            id: format!("task-general-{}", uuid::Uuid::new_v4()),
            description: "General task".to_string(),
            prompt: request.to_string(),
            subagent_type: "general-purpose".to_string(),
            context: HashMap::new(),
            priority: TaskPriority::Medium,
            expected_outputs: vec!["Task completion".to_string()],
        }
    }

    /// Mark a task as completed
    pub fn complete_task(&mut self, task_id: &str, result: TaskResult) -> SubagentResult<()> {
        if self.active_tasks.remove(task_id).is_none() {
            return Err(SubagentError::NotFound(task_id.to_string()));
        }

        self.completed_tasks.insert(task_id.to_string(), result);
        Ok(())
    }

    /// Get the status of all tasks
    pub fn get_task_status(&self) -> (Vec<&DelegatedTask>, Vec<&TaskResult>) {
        let active: Vec<&DelegatedTask> = self.active_tasks.values().collect();
        let completed: Vec<&TaskResult> = self.completed_tasks.values().collect();
        (active, completed)
    }
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::new()
    }
}

