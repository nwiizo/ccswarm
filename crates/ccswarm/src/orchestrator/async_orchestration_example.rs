/// Example of using async error boundaries in orchestration
///
/// This module demonstrates how to use error boundaries for reliable task orchestration.

use anyhow::Result;
use crate::utils::{boundary, boundary_with_fallback, with_retry, AsyncCircuitBreaker, ConcurrentBoundary};
use crate::task::Task;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

/// Orchestrator with built-in error resilience
pub struct ResilientOrchestrator {
    name: String,
    circuit_breakers: Arc<RwLock<std::collections::HashMap<String, Arc<AsyncCircuitBreaker>>>>,
}

impl ResilientOrchestrator {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            circuit_breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create circuit breaker for an agent
    async fn get_circuit_breaker(&self, agent_name: &str) -> Arc<AsyncCircuitBreaker> {
        let mut breakers = self.circuit_breakers.write().await;
        
        breakers.entry(agent_name.to_string())
            .or_insert_with(|| {
                Arc::new(AsyncCircuitBreaker::new(
                    format!("{}_breaker", agent_name),
                    5,  // failure threshold
                    3,  // success threshold
                    Duration::from_secs(60),  // reset timeout
                ))
            })
            .clone()
    }

    /// Delegate task with full error protection
    pub async fn delegate_task_safely(&self, task: &Task, agent_name: &str) -> Result<String> {
        let breaker = self.get_circuit_breaker(agent_name).await;
        
        // Execute with circuit breaker and error boundary
        let task_id = task.id.clone();
        let agent = agent_name.to_string();
        
        breaker.execute(
            boundary_with_fallback(
                self.delegate_task_internal(task, agent_name),
                &format!("delegate_task_{}", task_id),
                async move {
                    tracing::warn!("Fallback: Reassigning task {} from agent {}", task_id, agent);
                    // In real implementation, would reassign to another agent
                    Ok("fallback_result".to_string())
                }
            )
        ).await
    }

    /// Orchestrate multiple tasks concurrently
    pub async fn orchestrate_batch(&self, tasks: Vec<Task>) -> Result<Vec<Result<String>>> {
        let mut boundary = ConcurrentBoundary::new("batch_orchestration")
            .max_failures(tasks.len() / 3);  // Allow 1/3 failures
        
        for task in tasks {
            let orchestrator = self.clone();
            boundary = boundary.add_operation(async move {
                // Determine best agent for task
                let agent = orchestrator.select_agent_for_task(&task).await?;
                orchestrator.delegate_task_safely(&task, &agent).await.map(|_| ())
            });
        }
        
        boundary.execute().await
    }

    /// Execute critical path with multiple retry strategies
    pub async fn execute_critical_workflow(&self, tasks: Vec<Task>) -> Result<()> {
        // Phase 1: Preparation tasks (can fail individually)
        let prep_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.tags.contains(&"preparation".to_string()))
            .collect();
        
        let prep_boundary = ConcurrentBoundary::new("preparation_phase");
        let mut prep_ops = prep_boundary;
        
        for task in prep_tasks {
            let task_clone = task.clone();
            let orchestrator = self.clone();
            
            prep_ops = prep_ops.add_operation(async move {
                with_retry(
                    || Box::pin(async {
                        let agent = orchestrator.select_agent_for_task(&task_clone).await?;
                        orchestrator.delegate_task_safely(&task_clone, &agent).await
                    }),
                    3,
                    format!("prep_task_{}", task_clone.id),
                ).await.map(|_| ())
            });
        }
        
        let prep_results = prep_ops.execute().await?;
        let prep_failures = prep_results.iter().filter(|r| r.is_err()).count();
        
        if prep_failures > 0 {
            tracing::warn!("Preparation phase had {} failures", prep_failures);
        }
        
        // Phase 2: Critical tasks (must all succeed)
        let critical_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.priority == crate::task::Priority::Critical)
            .collect();
        
        for task in critical_tasks {
            // Critical tasks get more retries and dedicated handling
            let result = with_retry(
                || {
                    let task = task.clone();
                    let orchestrator = self.clone();
                    Box::pin(async move {
                        let agent = orchestrator.select_agent_for_task(&task).await?;
                        
                        // Try primary agent with circuit breaker
                        match orchestrator.delegate_task_safely(&task, &agent).await {
                            Ok(result) => Ok(result),
                            Err(e) => {
                                // Failover to backup agent
                                tracing::warn!("Primary agent failed, trying backup: {}", e);
                                let backup_agent = orchestrator.select_backup_agent(&agent).await?;
                                orchestrator.delegate_task_safely(&task, &backup_agent).await
                            }
                        }
                    })
                },
                5,  // More retries for critical tasks
                format!("critical_task_{}", task.id),
            ).await?;
            
            tracing::info!("Critical task {} completed: {}", task.id, result);
        }
        
        // Phase 3: Cleanup (best effort)
        let cleanup_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.tags.contains(&"cleanup".to_string()))
            .collect();
        
        let cleanup_boundary = ConcurrentBoundary::new("cleanup_phase");
        let mut cleanup_ops = cleanup_boundary;
        
        for task in cleanup_tasks {
            let task_clone = task.clone();
            let orchestrator = self.clone();
            
            cleanup_ops = cleanup_ops.add_operation(async move {
                // Cleanup is best-effort, use boundary without retry
                boundary(
                    async {
                        let agent = orchestrator.select_agent_for_task(&task_clone).await?;
                        orchestrator.delegate_task_safely(&task_clone, &agent).await
                    },
                    format!("cleanup_{}", task_clone.id),
                ).execute().await.map(|_| ())
            });
        }
        
        // Don't fail the whole workflow if cleanup fails
        if let Err(e) = cleanup_ops.execute().await {
            tracing::error!("Cleanup phase encountered errors: {:#}", e);
        }
        
        Ok(())
    }

    /// Internal task delegation
    async fn delegate_task_internal(&self, task: &Task, agent_name: &str) -> Result<String> {
        // Simulate task delegation
        tracing::info!("Delegating task {} to agent {}", task.id, agent_name);
        
        // Simulate potential failures
        if task.description.contains("unstable") {
            return Err(anyhow::anyhow!("Task delegation failed - unstable operation"));
        }
        
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(format!("Task {} delegated to {}", task.id, agent_name))
    }

    /// Select best agent for task
    async fn select_agent_for_task(&self, task: &Task) -> Result<String> {
        // Simplified agent selection logic
        match task.tags.first().map(|s| s.as_str()) {
            Some("frontend") => Ok("frontend-agent".to_string()),
            Some("backend") => Ok("backend-agent".to_string()),
            Some("devops") => Ok("devops-agent".to_string()),
            _ => Ok("general-agent".to_string()),
        }
    }

    /// Select backup agent
    async fn select_backup_agent(&self, primary_agent: &str) -> Result<String> {
        // Simple backup selection
        match primary_agent {
            "frontend-agent" => Ok("general-agent".to_string()),
            "backend-agent" => Ok("devops-agent".to_string()),
            _ => Ok("general-agent".to_string()),
        }
    }
}

// Implement Clone for the orchestrator
impl Clone for ResilientOrchestrator {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            circuit_breakers: self.circuit_breakers.clone(),
        }
    }
}

/// Example of monitoring orchestration health
pub struct OrchestrationMonitor {
    orchestrator: ResilientOrchestrator,
    health_check_interval: Duration,
}

impl OrchestrationMonitor {
    pub fn new(orchestrator: ResilientOrchestrator) -> Self {
        Self {
            orchestrator,
            health_check_interval: Duration::from_secs(30),
        }
    }

    /// Run continuous health monitoring with error boundaries
    pub async fn run_monitoring(&self) -> Result<()> {
        use crate::cancellable_async;
        use tokio_util::sync::CancellationToken;
        
        let token = CancellationToken::new();
        let monitor_token = token.clone();
        
        // Spawn monitoring task with error boundary
        let monitor_handle = crate::spawn_logged!(
            "orchestration_monitor",
            boundary(
                async move {
                    let mut interval = tokio::time::interval(self.health_check_interval);
                    
                    loop {
                        cancellable_async!(monitor_token.clone(), async {
                            interval.tick().await;
                            self.check_orchestration_health().await
                        })?;
                    }
                },
                "health_monitoring"
            ).execute()
        );
        
        // Wait for monitoring to complete or be cancelled
        tokio::select! {
            result = monitor_handle => {
                match result {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(e) => Err(anyhow::anyhow!("Monitor task failed: {}", e)),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Shutting down monitoring");
                token.cancel();
                Ok(())
            }
        }
    }

    async fn check_orchestration_health(&self) -> Result<()> {
        // Check circuit breaker states
        let breakers = self.orchestrator.circuit_breakers.read().await;
        
        for (agent, breaker) in breakers.iter() {
            tracing::debug!("Health check for agent {}", agent);
            // In real implementation, would check breaker state and metrics
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Priority, Status};
    use chrono::Utc;

    fn create_test_task(id: &str, tags: Vec<String>, priority: Priority) -> Task {
        Task {
            id: id.to_string(),
            description: "Test task".to_string(),
            priority,
            status: Status::Pending,
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_to: None,
            delegation_strategy: None,
            dependencies: vec![],
            parent_task_id: None,
            subtasks: vec![],
            result: None,
        }
    }

    #[tokio::test]
    async fn test_resilient_orchestration() {
        let orchestrator = ResilientOrchestrator::new("test_orchestrator");
        
        let task = create_test_task("test1", vec!["frontend".to_string()], Priority::High);
        let result = orchestrator.delegate_task_safely(&task, "frontend-agent").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_orchestration() {
        let orchestrator = ResilientOrchestrator::new("test_orchestrator");
        
        let tasks = vec![
            create_test_task("batch1", vec!["frontend".to_string()], Priority::Medium),
            create_test_task("batch2", vec!["backend".to_string()], Priority::High),
            create_test_task("batch3", vec!["devops".to_string()], Priority::Low),
        ];
        
        let results = orchestrator.orchestrate_batch(tasks).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_critical_workflow() {
        let orchestrator = ResilientOrchestrator::new("test_orchestrator");
        
        let tasks = vec![
            create_test_task("prep1", vec!["preparation".to_string()], Priority::Medium),
            create_test_task("critical1", vec!["backend".to_string()], Priority::Critical),
            create_test_task("cleanup1", vec!["cleanup".to_string()], Priority::Low),
        ];
        
        let result = orchestrator.execute_critical_workflow(tasks).await;
        assert!(result.is_ok());
    }
}