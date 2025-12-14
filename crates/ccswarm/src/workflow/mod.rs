//! Graph-based Workflow Engine Module
//!
//! Enables defining agent workflows as directed graphs with nodes (tasks)
//! and edges (dependencies/transitions).

mod execution;
mod graph;
mod node;

pub use execution::{ExecutionContext, ExecutionResult, WorkflowExecutor};
pub use graph::{Workflow, WorkflowBuilder, WorkflowError};
pub use node::{NodeId, NodeStatus, NodeType, WorkflowNode};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Workflow registry for managing multiple workflows
pub struct WorkflowRegistry {
    /// Registered workflows
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    /// Active executions
    executions: Arc<RwLock<HashMap<String, WorkflowExecution>>>,
}

/// A running workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Execution ID
    pub id: String,
    /// Workflow ID being executed
    pub workflow_id: String,
    /// Current status
    pub status: WorkflowExecutionStatus,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Node execution states
    pub node_states: HashMap<String, NodeExecutionState>,
    /// Variables/context
    pub variables: HashMap<String, serde_json::Value>,
}

/// Status of a workflow execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowExecutionStatus {
    /// Pending start
    Pending,
    /// Currently running
    Running,
    /// Paused
    Paused,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled
    Cancelled,
}

/// State of a node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionState {
    /// Node ID
    pub node_id: String,
    /// Current status
    pub status: NodeStatus,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Output data
    pub output: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count
    pub retry_count: u32,
}

impl WorkflowRegistry {
    /// Create a new workflow registry
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a workflow
    pub async fn register(&self, workflow: Workflow) -> Result<(), WorkflowError> {
        let mut workflows = self.workflows.write().await;
        if workflows.contains_key(&workflow.id) {
            return Err(WorkflowError::DuplicateId(workflow.id));
        }
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Get a workflow by ID
    pub async fn get(&self, id: &str) -> Option<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.get(id).cloned()
    }

    /// Remove a workflow
    pub async fn remove(&self, id: &str) -> Option<Workflow> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(id)
    }

    /// List all workflows
    pub async fn list(&self) -> Vec<String> {
        let workflows = self.workflows.read().await;
        workflows.keys().cloned().collect()
    }

    /// Start executing a workflow
    pub async fn execute(&self, workflow_id: &str) -> Result<String, WorkflowError> {
        let workflow = self
            .get(workflow_id)
            .await
            .ok_or_else(|| WorkflowError::NotFound(workflow_id.to_string()))?;

        let execution_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let mut node_states = HashMap::new();
        for node in &workflow.nodes {
            node_states.insert(
                node.id.0.clone(),
                NodeExecutionState {
                    node_id: node.id.0.clone(),
                    status: NodeStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    output: None,
                    error: None,
                    retry_count: 0,
                },
            );
        }

        let execution = WorkflowExecution {
            id: execution_id.clone(),
            workflow_id: workflow_id.to_string(),
            status: WorkflowExecutionStatus::Pending,
            started_at: now,
            completed_at: None,
            node_states,
            variables: HashMap::new(),
        };

        let mut executions = self.executions.write().await;
        executions.insert(execution_id.clone(), execution);

        Ok(execution_id)
    }

    /// Get execution status
    pub async fn get_execution(&self, execution_id: &str) -> Option<WorkflowExecution> {
        let executions = self.executions.read().await;
        executions.get(execution_id).cloned()
    }

    /// Update execution status
    pub async fn update_execution(
        &self,
        execution: WorkflowExecution,
    ) -> Result<(), WorkflowError> {
        let mut executions = self.executions.write().await;
        if !executions.contains_key(&execution.id) {
            return Err(WorkflowError::NotFound(execution.id));
        }
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }

    /// Cancel an execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), WorkflowError> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(execution_id) {
            execution.status = WorkflowExecutionStatus::Cancelled;
            execution.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(WorkflowError::NotFound(execution_id.to_string()))
        }
    }

    /// Get workflow count
    pub async fn workflow_count(&self) -> usize {
        let workflows = self.workflows.read().await;
        workflows.len()
    }

    /// Get active execution count
    pub async fn active_execution_count(&self) -> usize {
        let executions = self.executions.read().await;
        executions
            .values()
            .filter(|e| e.status == WorkflowExecutionStatus::Running)
            .count()
    }
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = WorkflowRegistry::new();
        assert_eq!(registry.workflow_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_workflow() {
        let registry = WorkflowRegistry::new();
        let workflow = WorkflowBuilder::new("test-workflow")
            .name("Test Workflow")
            .build()
            .unwrap();

        registry.register(workflow).await.unwrap();
        assert_eq!(registry.workflow_count().await, 1);
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let registry = WorkflowRegistry::new();
        let workflow1 = WorkflowBuilder::new("test-workflow")
            .name("Test 1")
            .build()
            .unwrap();
        let workflow2 = WorkflowBuilder::new("test-workflow")
            .name("Test 2")
            .build()
            .unwrap();

        registry.register(workflow1).await.unwrap();
        let result = registry.register(workflow2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_workflow() {
        let registry = WorkflowRegistry::new();
        let workflow = WorkflowBuilder::new("my-workflow")
            .name("My Workflow")
            .build()
            .unwrap();

        registry.register(workflow).await.unwrap();

        let retrieved = registry.get("my-workflow").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "My Workflow");

        let missing = registry.get("nonexistent").await;
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_remove_workflow() {
        let registry = WorkflowRegistry::new();
        let workflow = WorkflowBuilder::new("to-remove")
            .name("To Remove")
            .build()
            .unwrap();

        registry.register(workflow).await.unwrap();
        assert_eq!(registry.workflow_count().await, 1);

        let removed = registry.remove("to-remove").await;
        assert!(removed.is_some());
        assert_eq!(registry.workflow_count().await, 0);
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let registry = WorkflowRegistry::new();

        for i in 0..3 {
            let workflow = WorkflowBuilder::new(&format!("workflow-{}", i))
                .name(&format!("Workflow {}", i))
                .build()
                .unwrap();
            registry.register(workflow).await.unwrap();
        }

        let list = registry.list().await;
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn test_execute_workflow() {
        let registry = WorkflowRegistry::new();
        let workflow = WorkflowBuilder::new("exec-test")
            .name("Execution Test")
            .build()
            .unwrap();

        registry.register(workflow).await.unwrap();

        let exec_id = registry.execute("exec-test").await.unwrap();
        assert!(!exec_id.is_empty());

        let execution = registry.get_execution(&exec_id).await;
        assert!(execution.is_some());
        assert_eq!(execution.unwrap().status, WorkflowExecutionStatus::Pending);
    }

    #[tokio::test]
    async fn test_execute_nonexistent() {
        let registry = WorkflowRegistry::new();
        let result = registry.execute("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_execution() {
        let registry = WorkflowRegistry::new();
        let workflow = WorkflowBuilder::new("cancel-test")
            .name("Cancel Test")
            .build()
            .unwrap();

        registry.register(workflow).await.unwrap();
        let exec_id = registry.execute("cancel-test").await.unwrap();

        registry.cancel_execution(&exec_id).await.unwrap();

        let execution = registry.get_execution(&exec_id).await.unwrap();
        assert_eq!(execution.status, WorkflowExecutionStatus::Cancelled);
        assert!(execution.completed_at.is_some());
    }
}
