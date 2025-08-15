/// Simplified agent pool module
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::agent::{Agent, AgentRole, AgentStatus, Task, TaskResult};

/// Simplified agent pool
pub struct AgentPool {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl AgentPool {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn add_agent(&self, agent: Agent) -> Result<()> {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), agent);
        Ok(())
    }
    
    pub async fn get_agent(&self, id: &str) -> Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.get(id).cloned())
    }
    
    pub async fn get_available_agent(&self, role: AgentRole) -> Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values()
            .find(|a| a.role == role && a.status == AgentStatus::Idle)
            .cloned())
    }
    
    pub async fn assign_task(&self, agent_id: &str, _task: Task) -> Result<TaskResult> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = AgentStatus::Working;
        }
        
        // Simulate task execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = AgentStatus::Idle;
        }
        
        Ok(TaskResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            success: true,
            output: Some("Task completed".to_string()),
            error: None,
            duration: Some(std::time::Duration::from_millis(100)),
        })
    }
    
    pub async fn get_agent_count(&self) -> usize {
        self.agents.read().await.len()
    }
    
    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }
    
    pub async fn spawn_agent(&self, role: AgentRole, _config: Option<serde_json::Value>) -> Result<()> {
        let agent = Agent {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{:?} Agent", role),
            role,
            status: AgentStatus::Idle,
            capabilities: vec![],
        };
        self.add_agent(agent).await
    }
    
    pub async fn orchestrate_task(&self, task: Task) -> Result<TaskResult> {
        // Find available agent for the task type
        let role = match task.task_type {
            crate::agent::TaskType::Feature | crate::agent::TaskType::Development => AgentRole::Frontend,
            crate::agent::TaskType::Bug | crate::agent::TaskType::Bugfix => AgentRole::Backend,
            crate::agent::TaskType::Infrastructure => AgentRole::DevOps,
            crate::agent::TaskType::Testing | crate::agent::TaskType::Review => AgentRole::QA,
            _ => AgentRole::Backend,
        };
        
        if let Some(agent) = self.get_available_agent(role).await? {
            self.assign_task(&agent.id, task).await
        } else {
            Ok(TaskResult {
                task_id: task.id,
                success: false,
                output: None,
                error: Some("No available agent".to_string()),
                duration: None,
            })
        }
    }
    
    pub async fn execute_task_with_agent(&self, agent_id: &str, task: Task) -> Result<TaskResult> {
        self.assign_task(agent_id, task).await
    }
}