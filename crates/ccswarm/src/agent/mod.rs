/// Compact agent module - Unified implementation
use crate::utils::generic_handler::{StateManager, ListManager, EventBus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: String,
    pub to: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Frontend,
    Backend,
    DevOps,
    QA,
    Search,
    Refactoring,
    Custom(String),
}

impl AgentRole {
    pub fn name(&self) -> &str {
        match self {
            AgentRole::Frontend => "Frontend",
            AgentRole::Backend => "Backend",
            AgentRole::DevOps => "DevOps",
            AgentRole::QA => "QA",
            AgentRole::Search => "Search",
            AgentRole::Refactoring => "Refactoring",
            AgentRole::Custom(name) => name,
        }
    }
    
    /// Convert from identity::AgentRole to agent::AgentRole
    pub fn from_identity_role(identity_role: &crate::identity::AgentRole) -> Self {
        match identity_role {
            crate::identity::AgentRole::Frontend { .. } => AgentRole::Frontend,
            crate::identity::AgentRole::Backend { .. } => AgentRole::Backend,
            crate::identity::AgentRole::DevOps { .. } => AgentRole::DevOps,
            crate::identity::AgentRole::QA { .. } => AgentRole::QA,
            crate::identity::AgentRole::Search { .. } => AgentRole::Search,
            crate::identity::AgentRole::Master { .. } => AgentRole::Custom("Master".to_string()),
        }
    }
    
    /// Convert to identity::AgentRole with default configurations
    pub fn to_identity_role(&self) -> crate::identity::AgentRole {
        match self {
            AgentRole::Frontend => crate::identity::default_frontend_role(),
            AgentRole::Backend => crate::identity::default_backend_role(),
            AgentRole::DevOps => crate::identity::default_devops_role(),
            AgentRole::QA => crate::identity::default_qa_role(),
            AgentRole::Search => crate::identity::default_search_role(),
            AgentRole::Refactoring => crate::identity::AgentRole::Frontend {
                technologies: vec!["Rust".to_string(), "AST".to_string()],
                responsibilities: vec!["Code Refactoring".to_string()],
                boundaries: vec!["No functional changes".to_string()],
            },
            AgentRole::Custom(name) if name == "Master" => crate::identity::AgentRole::Master {
                oversight_roles: vec!["Frontend".to_string(), "Backend".to_string(), "DevOps".to_string(), "QA".to_string()],
                quality_standards: crate::identity::QualityStandards::default(),
            },
            AgentRole::Custom(_) => crate::identity::default_frontend_role(), // Default fallback
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Available,
    Working,
    Suspended,
    Error(String),
    Initializing,
    WaitingForReview,   // 追加
    ShuttingDown,       // 追加
}


// Task-related types for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub status: TaskStatus,
    pub details: Option<String>,           // 追加
    pub estimated_duration: Option<u32>,   // 追加（秒単位）
}

impl Task {
    pub fn new(description: String, task_type: TaskType, priority: Priority) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            task_type,
            priority,
            status: TaskStatus::Pending,
            details: None,
            estimated_duration: None,
        }
    }

    // 4引数版のコンストラクタ（後方互換性のため）
    pub fn new_with_id(id: String, description: String, priority: Priority, task_type: TaskType) -> Self {
        Self {
            id,
            description,
            task_type,
            priority,
            status: TaskStatus::Pending,
            details: None,
            estimated_duration: None,
        }
    }

    pub fn with_duration(mut self, duration: u32) -> Self {
        self.estimated_duration = Some(duration);
        self
    }
    
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    Feature,
    Bug,
    Refactor,
    Documentation,
    Testing,
    Performance,
    Development,
    Infrastructure,
    Coordination,
    Research,   // 追加
    Bugfix,     // 追加
    Review,     // 追加
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration: Option<std::time::Duration>,
}

impl TaskResult {
    pub fn success(task_id: String, output: String) -> Self {
        Self {
            task_id,
            success: true,
            output: Some(output),
            error: None,
            duration: None,
        }
    }
    
    pub fn failure(task_id: String, error: String) -> Self {
        Self {
            task_id,
            success: false,
            output: None,
            error: Some(error),
            duration: None,
        }
    }
}

pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    pub fn new(description: String) -> Self {
        Self {
            task: Task {
                id: uuid::Uuid::new_v4().to_string(),
                description: description.clone(),
                task_type: TaskType::Feature,
                priority: Priority::Medium,
                status: TaskStatus::Pending,
                details: Some(description),
                estimated_duration: None,
            },
        }
    }
    
    pub fn with_type(mut self, task_type: TaskType) -> Self {
        self.task.task_type = task_type;
        self
    }
    
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.task.priority = priority;
        self
    }
    
    pub fn build(self) -> Task {
        self.task
    }
}

/// Unified agent manager using generic handlers
pub struct AgentManager {
    agents: ListManager<Agent>,
    state: StateManager<ManagerState>,
    events: Arc<EventBus<AgentEvent>>,
}

#[derive(Default)]
struct ManagerState {
    active_count: usize,
    total_tasks: usize,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Created(String),
    StatusChanged(String, AgentStatus),
    TaskCompleted(String),
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: ListManager::new(),
            state: StateManager::new(),
            events: Arc::new(EventBus::new()),
        }
    }
    
    pub async fn create_agent(&self, name: String, role: AgentRole) -> Result<Agent> {
        let agent = Agent {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            role: role.clone(),
            status: AgentStatus::Idle,
            capabilities: self.get_role_capabilities(&role),
        };
        
        self.agents.add(agent.clone()).await?;
        self.state.update(|s| {
            s.active_count += 1;
            Ok(())
        }).await?;
        
        self.events.publish(AgentEvent::Created(agent.id.clone())).await;
        
        Ok(agent)
    }
    
    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        self.agents.list().await
    }
    
    pub async fn get_agent(&self, id: &str) -> Result<Option<Agent>> {
        self.agents.find(|a| a.id == id).await
    }
    
    pub async fn update_status(&self, id: &str, status: AgentStatus) -> Result<()> {
        // In real implementation, would update in ListManager
        self.events.publish(AgentEvent::StatusChanged(id.to_string(), status)).await;
        Ok(())
    }
    
    pub async fn execute_task(&self, agent_id: &str, task: &str) -> Result<String> {
        self.update_status(agent_id, AgentStatus::Working).await?;
        
        // Simulate task execution
        let result = format!("Task '{}' completed by agent {}", task, agent_id);
        
        self.update_status(agent_id, AgentStatus::Idle).await?;
        self.state.update(|s| {
            s.total_tasks += 1;
            Ok(())
        }).await?;
        
        self.events.publish(AgentEvent::TaskCompleted(agent_id.to_string())).await;
        
        Ok(result)
    }
    
    fn get_role_capabilities(&self, role: &AgentRole) -> Vec<String> {
        match role {
            AgentRole::Frontend => vec![
                "React".to_string(),
                "TypeScript".to_string(),
                "CSS".to_string(),
            ],
            AgentRole::Backend => vec![
                "API Design".to_string(),
                "Database".to_string(),
                "Authentication".to_string(),
            ],
            AgentRole::DevOps => vec![
                "Docker".to_string(),
                "CI/CD".to_string(),
                "Infrastructure".to_string(),
            ],
            AgentRole::QA => vec![
                "Testing".to_string(),
                "Quality Assurance".to_string(),
                "Test Automation".to_string(),
            ],
            _ => vec!["General".to_string()],
        }
    }
    
    pub async fn get_statistics(&self) -> Result<AgentStatistics> {
        let agents = self.agents.list().await?;
        let state = self.state.read(|s| Ok((s.active_count, s.total_tasks))).await?;
        
        Ok(AgentStatistics {
            total_agents: agents.len(),
            active_agents: state.0,
            idle_agents: agents.iter().filter(|a| a.status == AgentStatus::Idle).count(),
            total_tasks_completed: state.1,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatistics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub total_tasks_completed: usize,
}



// Submodules
pub mod isolation;
pub mod orchestrator;
pub mod pool;
pub mod search_agent;
pub mod simple;
pub mod claude;

// Module for persistent agents
pub mod persistent {
    use super::*;
    
    #[derive(Debug, Clone)]
    pub struct PersistentClaudeAgent {
        pub agent: Agent,
        pub session_id: String,
    }
    
    impl PersistentClaudeAgent {
        pub fn new(name: String, role: AgentRole) -> Self {
            Self {
                agent: Agent {
                    id: uuid::Uuid::new_v4().to_string(),
                    name,
                    role,
                    status: AgentStatus::Idle,
                    capabilities: vec![],
                },
                session_id: uuid::Uuid::new_v4().to_string(),
            }
        }

        pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
            self.agent.status = AgentStatus::Working;
            
            // Simulate task execution
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            self.agent.status = AgentStatus::Idle;
            
            Ok(TaskResult {
                task_id: task.id,
                success: true,
                output: Some("Task completed".to_string()),
                error: None,
                duration: Some(std::time::Duration::from_millis(100)),
            })
        }

        pub async fn execute_task_batch(&mut self, tasks: Vec<Task>) -> Result<Vec<TaskResult>> {
            let mut results = Vec::new();
            for task in tasks {
                results.push(self.execute_task(task).await?);
            }
            Ok(results)
        }

        pub async fn get_session_stats(&self) -> SessionStats {
            SessionStats {
                token_count: 1000,  // Placeholder
                messages: vec!["Session started".to_string()],
            }
        }

        pub async fn establish_identity_once(&mut self) -> Result<()> {
            // Placeholder for identity establishment
            Ok(())
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct SessionStats {
        pub token_count: usize,
        pub messages: Vec<String>,
    }
}

// Re-exports
pub use isolation::IsolationMode;
pub use self::orchestrator::AgentOrchestrator;
pub use self::pool::AgentPool;

// Claude-specific types
pub struct ClaudeCodeAgent {
    pub agent: Agent,
    pub identity: crate::identity::AgentIdentity,
    pub status: AgentStatus,
    pub current_task: Option<Task>,
    pub last_activity: std::time::Instant,
    pub task_history: Vec<TaskResult>,
}

impl ClaudeCodeAgent {
    pub fn new(name: String, role: AgentRole) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let identity_role = match role {
            AgentRole::Frontend => crate::identity::AgentRole::Frontend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            AgentRole::Backend => crate::identity::AgentRole::Backend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            AgentRole::DevOps => crate::identity::AgentRole::DevOps {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            AgentRole::QA => crate::identity::AgentRole::QA {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            _ => crate::identity::AgentRole::Frontend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
        };
        Self {
            agent: Agent {
                id: id.clone(),
                name: name.clone(),
                role: role.clone(),
                status: AgentStatus::Idle,
                capabilities: vec![],
            },
            identity: crate::identity::AgentIdentity {
                agent_id: id.clone(),
                specialization: identity_role,
                workspace_path: std::path::PathBuf::new(),
                env_vars: std::collections::HashMap::new(),
                session_id: uuid::Uuid::new_v4().to_string(),
                parent_process_id: String::new(),
                initialized_at: chrono::Utc::now(),
            },
            status: AgentStatus::Idle,
            current_task: None,
            last_activity: std::time::Instant::now(),
            task_history: vec![],
        }
    }
    
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        self.status = AgentStatus::Working;
        self.current_task = Some(task.clone());
        self.last_activity = std::time::Instant::now();
        
        // Simulate task execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.status = AgentStatus::Idle;
        self.current_task = None;
        
        Ok(TaskResult {
            task_id: task.id,
            success: true,
            output: Some("Task completed".to_string()),
            error: None,
            duration: Some(self.last_activity.elapsed()),
        })
    }
}