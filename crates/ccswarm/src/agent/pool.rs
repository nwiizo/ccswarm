use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::agent::orchestrator::task_plan::{ParallelTask, ParallelTaskResult};
use crate::agent::orchestrator::{AgentOrchestrator, StepResult, StepType, TaskPlan, TaskStep};
use crate::agent::{AgentStatus, ClaudeCodeAgent, Task, TaskResult, TaskType};
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

#[async_trait]
impl AgentOrchestrator for AgentPool {
    async fn orchestrate_task(&self, task: &Task) -> Result<TaskResult> {
        info!("🎭 Pool orchestrating task: {}", task.description);

        // Analyze task to create execution plan
        let plan = self.analyze_task(task).await?;

        // Execute plan with parallel agent coordination
        let mut context = HashMap::new();
        context.insert("task_id".to_string(), task.id.clone());
        context.insert("pool_mode".to_string(), "true".to_string());

        for step in &plan.steps {
            let step_result = self.execute_step(step, &context).await?;

            // Update context with step results
            context.insert(
                format!("step_{}_result", step.id),
                serde_json::to_string(&step_result.outputs)?,
            );

            if !step_result.success {
                warn!("Pool step failed: {}", step.name);
                return Ok(TaskResult::failure(
                    format!("Pool orchestration failed at step: {}", step.name),
                    Duration::from_secs(0),
                ));
            }
        }

        Ok(TaskResult::success(
            serde_json::json!({"orchestration": "pool_success", "steps_completed": plan.steps.len()}),
            Duration::from_secs(10),
        ))
    }

    async fn analyze_task(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Pool-specific orchestration strategies
        match task.task_type {
            TaskType::Development => {
                // Multi-agent development workflow
                plan.add_step(
                    TaskStep::new(
                        "pool_agent_selection".to_string(),
                        "Select optimal agents for task".to_string(),
                        StepType::Analysis,
                    )
                    .with_description(
                        "Analyze available agents and select best ones for development task"
                            .to_string(),
                    ),
                );
                plan.add_step(
                    TaskStep::new(
                        "parallel_implementation".to_string(),
                        "Parallel implementation".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Execute implementation across multiple agents".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "integration_test".to_string(),
                        "Test agent integration".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Verify integration between agents".to_string()),
                );
            }
            TaskType::Testing => {
                // Distributed testing across agents
                plan.add_step(
                    TaskStep::new(
                        "test_planning".to_string(),
                        "Plan distributed test execution".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Plan test distribution across agents".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "parallel_testing".to_string(),
                        "Execute parallel tests".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Run tests in parallel across agents".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "test_aggregation".to_string(),
                        "Aggregate test results".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Collect and analyze test results".to_string()),
                );
            }
            TaskType::Infrastructure => {
                // DevOps coordination with other agents
                plan.add_step(
                    TaskStep::new(
                        "infra_planning".to_string(),
                        "Plan infrastructure changes".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Analyze infrastructure requirements".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "infra_implementation".to_string(),
                        "Deploy infrastructure".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Execute infrastructure deployment".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "infra_validation".to_string(),
                        "Validate deployment".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Verify infrastructure is working".to_string()),
                );
            }
            _ => {
                // Default pool workflow
                plan.add_step(
                    TaskStep::new(
                        "pool_analysis".to_string(),
                        "Analyze task for pool execution".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Determine best pool execution strategy".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "pool_execution".to_string(),
                        "Execute with best agent".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Execute task with selected agent".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "pool_validation".to_string(),
                        "Validate pool execution".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Verify task completion".to_string()),
                );
            }
        }

        Ok(plan)
    }

    async fn execute_step(
        &self,
        step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        info!("🎭 Pool executing step: {}", step.name);

        // Match on step name for custom pool behavior
        match step.name.as_str() {
            "Parallel implementation" | "Execute parallel tests" => {
                // Execute parallel tasks across multiple agents
                self.execute_parallel_step(step, context).await
            }
            "Select optimal agents for task" => {
                // Select best agents for the task
                self.execute_agent_selection_step(step, context).await
            }
            "Test agent integration" => {
                // Test integration between agents
                self.execute_integration_test_step(step, context).await
            }
            _ => {
                // Default execution - delegate to best agent
                self.execute_default_step(step, context).await
            }
        }
    }

    async fn review_and_adapt(
        &self,
        results: &[StepResult],
        current_plan: &mut TaskPlan,
    ) -> Result<TaskPlan> {
        info!(
            "🎭 Pool reviewing and adapting plan based on {} results",
            results.len()
        );

        // Check if any agents failed and need to be reassigned
        let failed_agents: Vec<_> = results.iter().filter(|r| !r.success).collect();

        if !failed_agents.is_empty() {
            warn!(
                "Pool detected {} failed agent executions",
                failed_agents.len()
            );

            // Add recovery steps for failed agents
            let recovery_step = TaskStep::new(
                "agent_recovery".to_string(),
                "Recover failed agents".to_string(),
                StepType::Execution,
            )
            .with_description("Restart and recover failed agents".to_string());

            let validation_step = TaskStep::new(
                "recovery_validation".to_string(),
                "Validate agent recovery".to_string(),
                StepType::Validation,
            )
            .with_description("Verify all agents are operational".to_string());

            current_plan.steps.push(recovery_step);
            current_plan.steps.push(validation_step);
        }

        Ok(current_plan.clone())
    }

    async fn execute_parallel_task(
        &self,
        task: &ParallelTask,
        _context: &HashMap<String, String>,
    ) -> Result<ParallelTaskResult> {
        // Execute parallel task across available agents
        let available_agents: Vec<_> = self
            .agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        if available_agents.is_empty() {
            return Ok(ParallelTaskResult {
                task_id: task.id.clone(),
                success: false,
                output: "No available agents".to_string(),
                error: Some("No agents available for parallel task execution".to_string()),
            });
        }

        // Use first available agent for the parallel task
        let agent_type = &available_agents[0];
        let execution_task = Task::new(
            task.id.clone(),
            task.name.clone(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        match self
            .execute_task_with_agent(agent_type, &execution_task)
            .await
        {
            Ok(result) => Ok(ParallelTaskResult {
                task_id: task.id.clone(),
                success: result.success,
                output: serde_json::to_string(&result.output).unwrap_or_default(),
                error: result.error,
            }),
            Err(e) => Ok(ParallelTaskResult {
                task_id: task.id.clone(),
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    async fn synthesize_results(
        &self,
        task: &Task,
        results: Vec<StepResult>,
    ) -> Result<TaskResult> {
        let total_steps = results.len();
        let successful_steps = results.iter().filter(|r| r.is_success()).count();
        let overall_success = successful_steps == total_steps;

        let summary = if overall_success {
            format!(
                "Pool orchestration completed successfully: {}/{} steps succeeded",
                successful_steps, total_steps
            )
        } else {
            format!(
                "Pool orchestration partially failed: {}/{} steps succeeded",
                successful_steps, total_steps
            )
        };

        let output = serde_json::json!({
            "orchestration_type": "pool",
            "task_id": task.id,
            "total_steps": total_steps,
            "successful_steps": successful_steps,
            "step_results": results.iter().map(|r| {
                serde_json::json!({
                    "step_id": r.step_id,
                    "success": r.success,
                    "summary": r.summary
                })
            }).collect::<Vec<_>>()
        });

        if overall_success {
            Ok(TaskResult::success(
                output,
                Duration::from_millis(results.iter().map(|r| r.duration_ms).sum()),
            ))
        } else {
            Ok(TaskResult::failure(
                summary,
                Duration::from_millis(results.iter().map(|r| r.duration_ms).sum()),
            ))
        }
    }
}

impl AgentPool {
    /// Execute parallel step across multiple agents
    async fn execute_parallel_step(
        &self,
        step: &TaskStep,
        _context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let mut results = Vec::new();

        // Get available agents
        let available_agents: Vec<_> = self
            .agents
            .iter()
            .map(|entry| {
                let agent_type = entry.key().clone();
                agent_type
            })
            .collect();

        if available_agents.is_empty() {
            return Ok(StepResult::new(step.id.clone())
                .failed("No available agents for parallel execution".to_string()));
        }

        // Execute on each available agent
        for agent_type in available_agents {
            let task = Task::new(
                format!("parallel_{}_{}", step.id, agent_type),
                format!("{} ({})", step.description, agent_type),
                crate::agent::Priority::Medium,
                TaskType::Development,
            );

            match self.execute_task_with_agent(&agent_type, &task).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Parallel execution failed for agent {}: {}", agent_type, e);
                    results.push(TaskResult::failure(e.to_string(), Duration::from_secs(0)));
                }
            }
        }

        let success_count = results.iter().filter(|r| r.success).count();
        let _success = success_count > 0; // At least one agent succeeded

        Ok(StepResult::new(step.id.clone())
            .with_summary(format!(
                "Parallel execution: {}/{} agents succeeded",
                success_count,
                results.len()
            ))
            .add_output("parallel_results".to_string(), results.len().to_string())
            .add_output("success_count".to_string(), success_count.to_string()))
    }

    /// Execute agent selection step
    async fn execute_agent_selection_step(
        &self,
        _step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task_id = context
            .get("task_id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        // Count available agents by type
        let mut agent_counts = HashMap::new();
        for entry in self.agents.iter() {
            let agent_type = entry.key().clone();
            *agent_counts.entry(agent_type).or_insert(0) += 1;
        }

        Ok(StepResult::new(_step.id.clone())
            .with_summary(format!("Selected agents for task {}", task_id))
            .add_output("task_id".to_string(), task_id)
            .add_output("total_agents".to_string(), self.agents.len().to_string()))
    }

    /// Execute integration test step
    async fn execute_integration_test_step(
        &self,
        _step: &TaskStep,
        _context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        // Test communication between agents
        let agent_types: Vec<_> = self
            .agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        let mut test_results = Vec::new();

        // Test inter-agent communication
        for i in 0..agent_types.len() {
            for j in (i + 1)..agent_types.len() {
                let from = &agent_types[i];
                let to = &agent_types[j];

                match self.send_message(from, to, "integration_test").await {
                    Ok(_) => test_results.push(format!("{} -> {} ✅", from, to)),
                    Err(e) => test_results.push(format!("{} -> {} ❌: {}", from, to, e)),
                }
            }
        }

        Ok(StepResult::new(_step.id.clone())
            .with_summary(format!(
                "Integration test: {} agent pairs tested",
                test_results.len()
            ))
            .add_output(
                "agent_pairs_tested".to_string(),
                test_results.len().to_string(),
            ))
    }

    /// Execute default step with best agent
    async fn execute_default_step(
        &self,
        step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task_id = context
            .get("task_id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        let task = Task::new(
            format!("pool_step_{}_{}", step.id, task_id),
            step.description.clone(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        // Get best agent for this task
        let best_agent = self.get_best_agent_for_task(&task).await?;
        let agent_type = {
            let agent_guard = best_agent.read().await;
            agent_guard.identity.specialization.name().to_lowercase()
        };

        // Execute with selected agent
        match self.execute_task_with_agent(&agent_type, &task).await {
            Ok(result) => Ok(StepResult::new(step.id.clone())
                .with_summary(format!(
                    "Task executed successfully by {} agent",
                    agent_type
                ))
                .add_output("agent_used".to_string(), agent_type)
                .add_output("task_success".to_string(), result.success.to_string())),
            Err(e) => {
                Ok(StepResult::new(step.id.clone())
                    .failed(format!("Default execution failed: {}", e)))
            }
        }
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

    #[tokio::test]
    async fn test_agent_pool_orchestration() {
        let pool = AgentPool::new().await.unwrap();

        let task = Task::new(
            "test_orchestration".to_string(),
            "Test pool orchestration".to_string(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        let plan = pool.analyze_task(&task).await.unwrap();
        assert!(!plan.steps.is_empty());
    }
}
