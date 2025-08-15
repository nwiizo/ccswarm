/// Compact agent module - Unified implementation
use crate::utils::generic_handler::{StateManager, ListManager, MessageProcessor, EventBus};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Suspended,
    Error(String),
}

// Task-related types for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    Feature,
    Bug,
    Refactor,
    Documentation,
    Testing,
    Performance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    pub fn new(description: String) -> Self {
        Self {
            task: Task {
                id: uuid::Uuid::new_v4().to_string(),
                description,
                task_type: TaskType::Feature,
                priority: Priority::Medium,
                status: TaskStatus::Pending,
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

/// Simplified persistent agent using generic handlers
pub struct PersistentAgent {
    base: Agent,
    session_manager: StateManager<SessionState>,
}

#[derive(Default)]
struct SessionState {
    session_id: Option<String>,
    context: Vec<String>,
    token_count: usize,
}

impl PersistentAgent {
    pub fn new(name: String, role: AgentRole) -> Self {
        Self {
            base: Agent {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                role,
                status: AgentStatus::Idle,
                capabilities: vec![],
            },
            session_manager: StateManager::new(),
        }
    }
    
    pub async fn start_session(&self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        self.session_manager.update(|s| {
            s.session_id = Some(session_id.clone());
            s.context.clear();
            s.token_count = 0;
            Ok(())
        }).await?;
        
        Ok(session_id)
    }
    
    pub async fn add_context(&self, context: String) -> Result<()> {
        self.session_manager.update(|s| {
            s.context.push(context);
            s.token_count += 100; // Simplified token counting
            Ok(())
        }).await
    }
    
    pub async fn get_efficiency(&self) -> Result<f64> {
        self.session_manager.read(|s| {
            if s.token_count == 0 {
                Ok(0.0)
            } else {
                Ok(s.context.len() as f64 / s.token_count as f64 * 100.0)
            }
        }).await
    }
}

/// Simplified pool agent manager
pub struct PoolAgentManager {
    pool: ListManager<Agent>,
    max_size: usize,
}

impl PoolAgentManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: ListManager::new(),
            max_size,
        }
    }
    
    pub async fn add_to_pool(&self, agent: Agent) -> Result<()> {
        let current = self.pool.list().await?;
        if current.len() < self.max_size {
            self.pool.add(agent).await
        } else {
            Err(anyhow::anyhow!("Pool is full"))
        }
    }
    
    pub async fn get_available(&self) -> Result<Option<Agent>> {
        self.pool.find(|a| a.status == AgentStatus::Idle).await
    }
    
    pub async fn scale(&self, target_size: usize) -> Result<()> {
        let current = self.pool.list().await?;
        
        if target_size > current.len() {
            // Add agents
            for i in current.len()..target_size {
                let agent = Agent {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: format!("pool-agent-{}", i),
                    role: AgentRole::Custom("Pool".to_string()),
                    status: AgentStatus::Idle,
                    capabilities: vec![],
                };
                self.pool.add(agent).await?;
            }
        } else if target_size < current.len() {
            // Remove agents
            let to_remove = current.len() - target_size;
            for _ in 0..to_remove {
                self.pool.remove(|a| a.status == AgentStatus::Idle).await?;
            }
        }
        
        Ok(())
    }
}

// Submodules for compatibility
pub mod orchestrator;
pub mod pool;
pub mod search_agent;

// Module for persistent agents
pub mod persistent {
    use super::*;
    
    pub struct PersistentClaudeAgent {
        pub agent: Agent,
    }
    
    pub struct SessionStats {
        pub token_count: usize,
        pub messages: Vec<String>,
    }
}

// Re-exports for backward compatibility
pub use self::orchestrator::AgentOrchestrator;
pub use self::pool::AgentPool;

// Claude-specific types
pub struct ClaudeCodeAgent {
    agent: Agent,
}

impl ClaudeCodeAgent {
    pub fn new(name: String, role: AgentRole) -> Self {
        Self {
            agent: Agent {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                role,
                status: AgentStatus::Idle,
                capabilities: vec![],
            },
        }
    }
}