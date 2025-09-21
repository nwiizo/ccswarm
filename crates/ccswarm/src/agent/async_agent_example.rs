/// Example of using async error boundaries in agent operations
///
/// This module demonstrates best practices for error handling in async agent contexts.

use anyhow::Result;
use crate::utils::{boundary, with_retry, AsyncCircuitBreaker, ConcurrentBoundary};
use std::sync::Arc;
use tokio::time::Duration;

/// Example agent with error boundary protection
pub struct ResilientAgent {
    name: String,
    circuit_breaker: Arc<AsyncCircuitBreaker>,
}

impl ResilientAgent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            circuit_breaker: Arc::new(AsyncCircuitBreaker::new(
                "agent_operations",
                3,  // failure threshold
                2,  // success threshold  
                Duration::from_secs(30),  // timeout
            )),
        }
    }

    /// Execute task with error boundary protection
    pub async fn execute_task_safely(&self, task: &str) -> Result<String> {
        // Use error boundary with fallback
        boundary(self.execute_task_internal(task), format!("{}_task", self.name))
            .with_fallback(|| async move {
                tracing::warn!("Executing fallback for task");
                Ok(())
            })
            .execute()
            .await
    }

    /// Execute with circuit breaker protection
    pub async fn execute_with_circuit_breaker(&self, task: &str) -> Result<String> {
        self.circuit_breaker
            .execute(self.execute_task_internal(task))
            .await
    }

    /// Execute with retry logic
    pub async fn execute_with_retry(&self, task: &str) -> Result<String> {
        with_retry(
            || Box::pin(self.execute_task_internal(task)),
            3,  // max retries
            format!("{}_retry", self.name),
        )
        .await
    }

    /// Internal task execution (may fail)
    async fn execute_task_internal(&self, task: &str) -> Result<String> {
        // Simulate task execution that might fail
        tracing::info!("Agent {} executing task: {}", self.name, task);
        
        // Simulate potential failure scenarios
        if task.contains("fail") {
            return Err(anyhow::anyhow!("Task execution failed"));
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("Task '{}' completed by {}", task, self.name))
    }
}

/// Example of concurrent agent operations with boundaries
pub async fn execute_multi_agent_task(agents: Vec<ResilientAgent>, task: &str) -> Result<Vec<Result<String>>> {
    let task = task.to_string();
    
    // Create concurrent boundary for all agent operations
    let mut boundary = ConcurrentBoundary::new("multi_agent_execution")
        .max_failures(2);  // Allow up to 2 failures
    
    // Add each agent's operation to the boundary
    for agent in agents {
        let task_clone = task.clone();
        boundary = boundary.add_operation(async move {
            agent.execute_task_safely(&task_clone).await.map(|_| ())
        });
    }
    
    // Execute all operations with boundary protection
    boundary.execute().await
}

/// Example of complex async workflow with multiple boundaries
pub async fn complex_agent_workflow() -> Result<()> {
    use crate::concurrent_execute;
    
    // Create agents
    let frontend_agent = ResilientAgent::new("frontend");
    let backend_agent = ResilientAgent::new("backend");
    let devops_agent = ResilientAgent::new("devops");
    
    // Phase 1: Initial setup with concurrent execution
    let setup_results = concurrent_execute!(
        boundary(frontend_agent.execute_task_safely("setup UI"), "frontend_setup").execute(),
        boundary(backend_agent.execute_task_safely("setup API"), "backend_setup").execute(),
        boundary(devops_agent.execute_task_safely("setup infra"), "devops_setup").execute()
    );
    
    // Check setup results
    for (i, result) in setup_results.iter().enumerate() {
        if let Err(e) = result {
            tracing::error!("Setup phase {} failed: {:#}", i, e);
        }
    }
    
    // Phase 2: Main operations with circuit breaker
    let main_ops = ConcurrentBoundary::new("main_operations")
        .fail_fast()  // Stop on first failure
        .add_operation(async {
            frontend_agent.execute_with_circuit_breaker("build components").await.map(|_| ())
        })
        .add_operation(async {
            backend_agent.execute_with_circuit_breaker("deploy services").await.map(|_| ())
        })
        .execute()
        .await?;
    
    // Phase 3: Cleanup with retry
    let cleanup = with_retry(
        || Box::pin(async {
            devops_agent.execute_task_safely("cleanup resources").await
        }),
        5,  // More retries for critical cleanup
        "cleanup_phase",
    )
    .await?;
    
    tracing::info!("Complex workflow completed successfully");
    Ok(())
}

