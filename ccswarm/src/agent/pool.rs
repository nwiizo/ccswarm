use anyhow::Result;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::agent::{AgentStatus, ClaudeCodeAgent, Task, TaskResult};
use crate::config::CcswarmConfig;
use crate::coordination::{AgentMessage, CoordinationBus};
use crate::identity::{
    default_backend_role, default_devops_role, default_frontend_role, default_qa_role,
};
use crate::orchestrator::master_delegation::MasterDelegationEngine;
use crate::session::claude_session::PersistentClaudeSession;

/// Agent pool for managing multiple agents
pub struct AgentPool {
    /// Active agents by type
    agents: Arc<DashMap<String, Arc<RwLock<ClaudeCodeAgent>>>>,

    /// Active sessions for agents
    sessions: Arc<DashMap<String, Arc<RwLock<PersistentClaudeSession>>>>,

    /// Coordination bus for inter-agent communication
    coordination_bus: Arc<CoordinationBus>,

    /// Task execution history
    execution_history: Arc<RwLock<Vec<TaskExecutionRecord>>>,
}

/// Record of task execution
#[derive(Debug, Clone)]
pub struct TaskExecutionRecord {
    pub task_id: String,
    pub agent_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<TaskResult>,
}

impl AgentPool {
    /// Create new agent pool
    pub async fn new() -> Result<Self> {
        let coordination_bus = Arc::new(CoordinationBus::new().await?);

        Ok(Self {
            agents: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            coordination_bus,
            execution_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Check if pool has agent of given type
    pub fn has_agent(&self, agent_type: &str) -> bool {
        self.agents.contains_key(agent_type)
    }

    /// Spawn a new agent
    pub async fn spawn_agent(&mut self, agent_type: &str, config: &CcswarmConfig) -> Result<()> {
        info!("🚀 Spawning {} agent", agent_type);

        // Get role for agent type
        let role = match agent_type.to_lowercase().as_str() {
            "frontend" => default_frontend_role(),
            "backend" => default_backend_role(),
            "devops" => default_devops_role(),
            "qa" => default_qa_role(),
            _ => return Err(anyhow::anyhow!("Unknown agent type: {}", agent_type)),
        };

        // Get agent config
        let agent_config = config
            .agents
            .get(agent_type)
            .ok_or_else(|| anyhow::anyhow!("No configuration for agent: {}", agent_type))?;

        // Create agent
        let mut agent = ClaudeCodeAgent::new(
            role,
            &PathBuf::from(&config.project.name),
            &agent_config.branch,
            agent_config.claude_config.clone(),
        )
        .await?;

        // Initialize agent
        agent.initialize().await?;

        // Create persistent session for agent
        let mut session = PersistentClaudeSession::new(
            agent.identity.clone(),
            agent.worktree_path.clone(),
            agent_config.claude_config.clone(),
        )
        .await?;

        // Initialize session
        session.initialize().await?;

        // Store agent and session
        self.agents
            .insert(agent_type.to_string(), Arc::new(RwLock::new(agent)));

        self.sessions
            .insert(agent_type.to_string(), Arc::new(RwLock::new(session)));

        info!("✅ {} agent spawned and initialized", agent_type);
        Ok(())
    }

    /// Get agent by type
    pub fn get_agent(&self, agent_type: &str) -> Result<Arc<RwLock<ClaudeCodeAgent>>> {
        self.agents
            .get(agent_type)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_type))
    }

    /// Get best agent for task
    pub async fn get_best_agent_for_task(
        &self,
        task: &Task,
    ) -> Result<Arc<RwLock<ClaudeCodeAgent>>> {
        let mut engine = MasterDelegationEngine::new(
            crate::orchestrator::master_delegation::DelegationStrategy::Hybrid,
        );

        let decision = engine.delegate_task(task.clone())?;
        let agent_type = decision.target_agent.name().to_lowercase();

        self.get_agent(&agent_type)
    }

    /// Execute task with agent
    pub async fn execute_task_with_agent(
        &self,
        agent_type: &str,
        task: &Task,
    ) -> Result<TaskResult> {
        info!(
            "📋 Executing task with {} agent: {}",
            agent_type, task.description
        );

        // Get agent and session
        let agent = self.get_agent(agent_type)?;
        let session = self
            .sessions
            .get(agent_type)
            .ok_or_else(|| anyhow::anyhow!("No session for agent: {}", agent_type))?;

        // Record execution start
        let record = TaskExecutionRecord {
            task_id: task.id.clone(),
            agent_id: agent_type.to_string(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            result: None,
        };

        self.execution_history.write().await.push(record.clone());

        // Update agent status
        {
            let mut agent_guard = agent.write().await;
            agent_guard.status = AgentStatus::Working;
            agent_guard.current_task = Some(task.clone());
            agent_guard.last_activity = chrono::Utc::now();
        }

        // Execute task with session
        let result = {
            let mut session_guard = session.write().await;
            session_guard.execute_task(task.clone()).await?
        };

        // Update execution record
        let mut history = self.execution_history.write().await;
        if let Some(record) = history.iter_mut().find(|r| r.task_id == task.id) {
            record.completed_at = Some(chrono::Utc::now());
            record.result = Some(result.clone());
        }

        // Update agent status
        {
            let mut agent_guard = agent.write().await;
            agent_guard.status = AgentStatus::Available;
            agent_guard.current_task = None;
            agent_guard.last_activity = chrono::Utc::now();

            // Add to task history
            agent_guard
                .task_history
                .push((task.clone(), result.clone()));
        }

        // Send completion message
        self.coordination_bus
            .send_message(AgentMessage::TaskCompleted {
                agent_id: agent_type.to_string(),
                task_id: task.id.clone(),
                result: result.clone(),
            })
            .await?;

        Ok(result)
    }

    /// Send message between agents
    pub async fn send_message(&self, from: &str, to: &str, message: &str) -> Result<String> {
        info!("💬 {} → {}: {}", from, to, message);

        // Create inter-agent message
        let msg = AgentMessage::InterAgentMessage {
            from_agent: from.to_string(),
            to_agent: to.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Send through coordination bus
        self.coordination_bus.send_message(msg).await?;

        // Simulate response (in real implementation, would wait for actual response)
        Ok(format!("Acknowledged: {}", message))
    }

    /// Broadcast message to all agents
    pub async fn broadcast_message(&self, from: &str, message: &str) -> Result<()> {
        info!("📢 {} → all: {}", from, message);

        for entry in self.agents.iter() {
            let to_agent = entry.key();
            if to_agent != from {
                self.send_message(from, to_agent, message).await?;
            }
        }

        Ok(())
    }

    /// Execute command with agent
    pub async fn execute_command_with_agent(
        &self,
        agent_type: &str,
        command: &str,
    ) -> Result<CommandResult> {
        info!("🔧 {} executing: {}", agent_type, command);

        // Get agent session
        let session = self
            .sessions
            .get(agent_type)
            .ok_or_else(|| anyhow::anyhow!("No session for agent: {}", agent_type))?;

        // Execute command through session
        let mut session_guard = session.write().await;
        let output = session_guard.execute_command(command).await?;

        // Parse output for test results if applicable
        let (passed_tests, total_tests) = if command.contains("npm test") {
            parse_test_results(&output)
        } else {
            (0, 0)
        };

        Ok(CommandResult {
            success: true,
            output,
            passed_tests,
            total_tests,
        })
    }

    /// Get execution history
    pub async fn get_execution_history(&self) -> Vec<TaskExecutionRecord> {
        self.execution_history.read().await.clone()
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub passed_tests: usize,
    pub total_tests: usize,
}

/// Parse test results from output
fn parse_test_results(output: &str) -> (usize, usize) {
    // Simple parsing - in real implementation would be more robust
    if output.contains("Tests:") {
        // Jest format: "Tests: 5 passed, 5 total"
        let passed = output.matches("passed").count();
        let total = output.matches("total").count();
        (passed, total)
    } else {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_pool_creation() {
        let pool = AgentPool::new().await.unwrap();
        assert!(!pool.has_agent("frontend"));
    }
}
