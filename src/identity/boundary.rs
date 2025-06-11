use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::Task;
use crate::identity::AgentRole;

/// Result of task boundary evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvaluation {
    /// Task is within agent's boundaries - proceed
    Accept { reason: String },
    /// Task should be delegated to another agent
    Delegate {
        reason: String,
        target_agent: String,
        suggestion: String,
    },
    /// Task needs clarification before proceeding
    Clarify {
        reason: String,
        questions: Vec<String>,
    },
    /// Task is explicitly rejected
    Reject { reason: String },
}

/// Task boundary checker for enforcing agent specialization
#[derive(Debug)]
pub struct TaskBoundaryChecker {
    role: AgentRole,
    allowed_patterns: Vec<Regex>,
    forbidden_patterns: Vec<Regex>,
    delegation_targets: HashMap<String, String>,
}

impl TaskBoundaryChecker {
    /// Create a new boundary checker for the given role
    pub fn new(role: AgentRole) -> Self {
        let (allowed, forbidden) = Self::create_patterns_for_role(&role);
        let delegation_targets = Self::create_delegation_map();

        Self {
            role,
            allowed_patterns: allowed,
            forbidden_patterns: forbidden,
            delegation_targets,
        }
    }

    /// Evaluate whether a task is within boundaries
    pub async fn evaluate_task(&self, task: &Task) -> TaskEvaluation {
        // Check if explicitly allowed
        if self.is_explicitly_allowed(task) {
            return TaskEvaluation::Accept {
                reason: "Task is within my specialization".to_string(),
            };
        }

        // Check if explicitly forbidden
        if self.is_explicitly_forbidden(task) {
            let target_agent = self.determine_correct_agent(task);
            return TaskEvaluation::Delegate {
                reason: "Task is outside my specialization".to_string(),
                target_agent: target_agent.clone(),
                suggestion: self.generate_delegation_message(task, &target_agent),
            };
        }

        // Ambiguous case - request clarification
        TaskEvaluation::Clarify {
            reason: "Task scope is unclear".to_string(),
            questions: self.generate_clarification_questions(task),
        }
    }

    /// Check if task matches allowed patterns
    fn is_explicitly_allowed(&self, task: &Task) -> bool {
        let task_text = format!(
            "{} {}",
            task.description,
            task.details.as_deref().unwrap_or("")
        );

        self.allowed_patterns
            .iter()
            .any(|pattern| pattern.is_match(&task_text))
    }

    /// Check if task matches forbidden patterns
    fn is_explicitly_forbidden(&self, task: &Task) -> bool {
        let task_text = format!(
            "{} {}",
            task.description,
            task.details.as_deref().unwrap_or("")
        );

        self.forbidden_patterns
            .iter()
            .any(|pattern| pattern.is_match(&task_text))
    }

    /// Determine which agent should handle this task
    fn determine_correct_agent(&self, task: &Task) -> String {
        let task_text = format!(
            "{} {}",
            task.description,
            task.details.as_deref().unwrap_or("")
        );

        // Check backend patterns
        if self.check_patterns(
            &task_text,
            &[
                r"(?i)(api|backend|server|database|sql|auth|endpoint)",
                r"(?i)(rest|graphql|microservice|grpc)",
            ],
        ) {
            return self
                .delegation_targets
                .get("backend")
                .cloned()
                .unwrap_or_else(|| "backend-agent".to_string());
        }

        // Check frontend patterns
        if self.check_patterns(
            &task_text,
            &[
                r"(?i)(ui|frontend|component|react|vue|angular)",
                r"(?i)(css|styling|tailwind|sass|layout)",
            ],
        ) {
            return self
                .delegation_targets
                .get("frontend")
                .cloned()
                .unwrap_or_else(|| "frontend-agent".to_string());
        }

        // Check DevOps patterns
        if self.check_patterns(
            &task_text,
            &[
                r"(?i)(docker|kubernetes|k8s|container)",
                r"(?i)(deploy|infrastructure|terraform|aws|gcp|azure)",
                r"(?i)(ci/cd|pipeline|jenkins|github.actions)",
            ],
        ) {
            return self
                .delegation_targets
                .get("devops")
                .cloned()
                .unwrap_or_else(|| "devops-agent".to_string());
        }

        // Check QA patterns
        if self.check_patterns(
            &task_text,
            &[
                r"(?i)(test|testing|qa|quality|spec)",
                r"(?i)(cypress|jest|playwright|selenium)",
                r"(?i)(coverage|automation|e2e|integration)",
            ],
        ) {
            return self
                .delegation_targets
                .get("qa")
                .cloned()
                .unwrap_or_else(|| "qa-agent".to_string());
        }

        // Default to master for unclear cases
        "master-claude".to_string()
    }

    /// Check if text matches any of the given patterns
    fn check_patterns(&self, text: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| {
            Regex::new(pattern)
                .map(|re| re.is_match(text))
                .unwrap_or(false)
        })
    }

    /// Generate delegation message
    fn generate_delegation_message(&self, task: &Task, target_agent: &str) -> String {
        format!(
            "Task '{}' appears to be {} work based on the content. \
             Recommending delegation to {} for proper handling.",
            task.description,
            self.categorize_task_type(task),
            target_agent
        )
    }

    /// Categorize task type for clearer communication
    fn categorize_task_type(&self, task: &Task) -> &str {
        let task_text = &task.description.to_lowercase();

        if task_text.contains("api") || task_text.contains("backend") {
            "backend API"
        } else if task_text.contains("ui") || task_text.contains("component") {
            "frontend UI"
        } else if task_text.contains("deploy") || task_text.contains("infrastructure") {
            "DevOps/infrastructure"
        } else if task_text.contains("test") || task_text.contains("qa") {
            "QA/testing"
        } else {
            "specialized"
        }
    }

    /// Generate clarification questions for ambiguous tasks
    fn generate_clarification_questions(&self, _task: &Task) -> Vec<String> {
        vec![
            format!("Is this task specifically related to {}?", self.role.name()),
            "What components or systems will this task modify?".to_string(),
            "Are there any API, database, or infrastructure changes involved?".to_string(),
            format!(
                "Should this be handled by a {} specialist?",
                self.role.name()
            ),
        ]
    }

    /// Create allowed and forbidden patterns for each role
    fn create_patterns_for_role(role: &AgentRole) -> (Vec<Regex>, Vec<Regex>) {
        match role {
            AgentRole::Frontend { .. } => {
                let allowed = vec![
                    r"(?i)(react|vue|angular|svelte)",
                    r"(?i)(component|jsx|tsx|ui)",
                    r"(?i)(css|scss|sass|tailwind|styled)",
                    r"(?i)(frontend|client.?side)",
                    r"(?i)(state.?management|redux|zustand|mobx)",
                    r"(?i)(webpack|vite|rollup|parcel)",
                    r"(?i)(jest.*component|testing.?library)",
                ];

                let forbidden = vec![
                    r"(?i)(api|endpoint|rest|graphql)",
                    r"(?i)(database|sql|orm|prisma|typeorm)",
                    r"(?i)(server|backend|node.*api)",
                    r"(?i)(docker|kubernetes|terraform)",
                    r"(?i)(auth.*server|jwt.*generate)",
                ];

                (
                    Self::compile_patterns(allowed),
                    Self::compile_patterns(forbidden),
                )
            }

            AgentRole::Backend { .. } => {
                let allowed = vec![
                    r"(?i)(api|endpoint|rest|graphql)",
                    r"(?i)(server|backend|microservice)",
                    r"(?i)(database|sql|orm|query)",
                    r"(?i)(auth|jwt|session|oauth)",
                    r"(?i)(express|fastify|nest|koa)",
                    r"(?i)(prisma|typeorm|sequelize)",
                ];

                let forbidden = vec![
                    r"(?i)(react|vue|angular|component)",
                    r"(?i)(css|scss|tailwind|styling)",
                    r"(?i)(ui|user.?interface|frontend)",
                    r"(?i)(docker|kubernetes|helm)",
                    r"(?i)(terraform|cloudformation)",
                ];

                (
                    Self::compile_patterns(allowed),
                    Self::compile_patterns(forbidden),
                )
            }

            AgentRole::DevOps { .. } => {
                let allowed = vec![
                    r"(?i)(docker|container|kubernetes)",
                    r"(?i)(deploy|deployment|release)",
                    r"(?i)(ci/cd|pipeline|jenkins)",
                    r"(?i)(terraform|ansible|cloudformation)",
                    r"(?i)(aws|gcp|azure|cloud)",
                    r"(?i)(monitoring|logging|metrics)",
                ];

                let forbidden = vec![
                    r"(?i)(business.?logic|feature|functionality)",
                    r"(?i)(component|ui|frontend.*code)",
                    r"(?i)(api.*implementation|endpoint.*logic)",
                    r"(?i)(database.*schema|migration.*create)",
                ];

                (
                    Self::compile_patterns(allowed),
                    Self::compile_patterns(forbidden),
                )
            }

            AgentRole::QA { .. } => {
                let allowed = vec![
                    r"(?i)(test|testing|spec|suite)",
                    r"(?i)(qa|quality|verification)",
                    r"(?i)(jest|cypress|playwright|selenium)",
                    r"(?i)(coverage|automation|e2e)",
                    r"(?i)(performance.*test|load.*test)",
                    r"(?i)(security.*test|penetration)",
                ];

                let forbidden = vec![
                    r"(?i)(implement.*feature|add.*functionality)",
                    r"(?i)(fix.*bug.*in.*code|patch.*issue)",
                    r"(?i)(deploy|release|infrastructure)",
                    r"(?i)(design.*api|create.*endpoint)",
                ];

                (
                    Self::compile_patterns(allowed),
                    Self::compile_patterns(forbidden),
                )
            }

            AgentRole::Master { .. } => {
                // Master has different patterns - focuses on coordination
                let allowed = vec![
                    r"(?i)(coordinate|orchestrate|manage)",
                    r"(?i)(review|quality|standard)",
                    r"(?i)(architecture|design|planning)",
                ];

                let forbidden = vec![
                    r"(?i)(implement|code|develop)",
                    r"(?i)(fix.*bug|patch|hotfix)",
                ];

                (
                    Self::compile_patterns(allowed),
                    Self::compile_patterns(forbidden),
                )
            }
        }
    }

    /// Compile string patterns into Regex objects
    fn compile_patterns(patterns: Vec<&str>) -> Vec<Regex> {
        patterns
            .into_iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    }

    /// Create delegation mapping
    fn create_delegation_map() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("frontend".to_string(), "frontend-agent".to_string());
        map.insert("backend".to_string(), "backend-agent".to_string());
        map.insert("devops".to_string(), "devops-agent".to_string());
        map.insert("qa".to_string(), "qa-agent".to_string());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Priority, TaskType};
    use crate::identity::{default_backend_role, default_frontend_role};

    #[tokio::test]
    async fn test_frontend_accepts_ui_task() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());
        let task = Task {
            id: "1".to_string(),
            description: "Create a React component for user profile".to_string(),
            details: Some("Using TypeScript and Tailwind CSS".to_string()),
            priority: Priority::Medium,
            task_type: TaskType::Development,
            estimated_duration: None,
        };

        let result = checker.evaluate_task(&task).await;

        match result {
            TaskEvaluation::Accept { .. } => {}
            _ => panic!("Frontend should accept UI tasks"),
        }
    }

    #[tokio::test]
    async fn test_frontend_delegates_backend_task() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());
        let task = Task {
            id: "2".to_string(),
            description: "Create REST API endpoint for authentication".to_string(),
            details: None,
            priority: Priority::High,
            task_type: TaskType::Development,
            estimated_duration: None,
        };

        let result = checker.evaluate_task(&task).await;

        match result {
            TaskEvaluation::Delegate { target_agent, .. } => {
                assert_eq!(target_agent, "backend-agent");
            }
            _ => panic!("Frontend should delegate backend tasks"),
        }
    }

    #[tokio::test]
    async fn test_unclear_task_triggers_clarification() {
        let checker = TaskBoundaryChecker::new(default_backend_role());
        let task = Task {
            id: "3".to_string(),
            description: "Update the user system".to_string(),
            details: None,
            priority: Priority::Medium,
            task_type: TaskType::Development,
            estimated_duration: None,
        };

        let result = checker.evaluate_task(&task).await;

        match result {
            TaskEvaluation::Clarify { questions, .. } => {
                assert!(!questions.is_empty());
            }
            _ => panic!("Unclear tasks should trigger clarification"),
        }
    }
}
