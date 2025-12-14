//! Workflow execution engine

use super::graph::{Workflow, WorkflowError};
use super::node::{NodeId, NodeStatus, NodeType};
use super::{NodeExecutionState, WorkflowExecution, WorkflowExecutionStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Context for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    /// Variables available during execution
    pub variables: HashMap<String, serde_json::Value>,
    /// Agent assignments
    pub agent_assignments: HashMap<String, String>,
    /// Execution options
    pub options: ExecutionOptions,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable
    pub fn with_variable(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Assign an agent to a node
    pub fn with_agent_assignment(
        mut self,
        node_id: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Self {
        self.agent_assignments
            .insert(node_id.into(), agent_id.into());
        self
    }

    /// Get a variable
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Set a variable
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.variables.insert(key.into(), value.into());
    }
}

/// Execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    /// Maximum parallel nodes
    pub max_parallel: usize,
    /// Default timeout per node in seconds
    pub default_timeout_secs: u64,
    /// Whether to continue on failure
    pub continue_on_failure: bool,
    /// Whether to skip approval nodes
    pub skip_approvals: bool,
    /// Dry run mode (don't actually execute)
    pub dry_run: bool,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            default_timeout_secs: 300,
            continue_on_failure: false,
            skip_approvals: false,
            dry_run: false,
        }
    }
}

/// Result of a node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Node ID
    pub node_id: String,
    /// Whether successful
    pub success: bool,
    /// Output data
    pub output: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(node_id: impl Into<String>, output: serde_json::Value) -> Self {
        Self {
            node_id: node_id.into(),
            success: true,
            output: Some(output),
            error: None,
            duration_ms: 0,
        }
    }

    /// Create a failure result
    pub fn failure(node_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            success: false,
            output: None,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Workflow executor
pub struct WorkflowExecutor {
    /// The workflow being executed
    workflow: Workflow,
    /// Current execution state
    execution: Arc<RwLock<WorkflowExecution>>,
    /// Execution context
    context: Arc<RwLock<ExecutionContext>>,
    /// Completed nodes
    completed: Arc<RwLock<HashSet<NodeId>>>,
    /// Node execution handler
    handler: Arc<dyn NodeHandler + Send + Sync>,
}

/// Handler for node execution
#[async_trait::async_trait]
pub trait NodeHandler: Send + Sync {
    /// Execute a task node
    async fn execute_task(
        &self,
        node_id: &str,
        description: &str,
        agent_role: Option<&str>,
        context: &ExecutionContext,
    ) -> ExecutionResult;

    /// Handle an approval node
    async fn handle_approval(
        &self,
        node_id: &str,
        message: &str,
        approvers: &[String],
    ) -> ExecutionResult;

    /// Handle a delay node
    async fn handle_delay(&self, node_id: &str, seconds: u64) -> ExecutionResult;
}

/// Default node handler (for testing)
pub struct DefaultNodeHandler;

#[async_trait::async_trait]
impl NodeHandler for DefaultNodeHandler {
    async fn execute_task(
        &self,
        node_id: &str,
        description: &str,
        _agent_role: Option<&str>,
        _context: &ExecutionContext,
    ) -> ExecutionResult {
        // Default implementation just succeeds
        ExecutionResult::success(
            node_id,
            serde_json::json!({
                "description": description,
                "status": "completed"
            }),
        )
    }

    async fn handle_approval(
        &self,
        node_id: &str,
        message: &str,
        _approvers: &[String],
    ) -> ExecutionResult {
        // Default implementation auto-approves
        ExecutionResult::success(
            node_id,
            serde_json::json!({
                "message": message,
                "approved": true
            }),
        )
    }

    async fn handle_delay(&self, node_id: &str, seconds: u64) -> ExecutionResult {
        tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
        ExecutionResult::success(
            node_id,
            serde_json::json!({
                "delayed_seconds": seconds
            }),
        )
    }
}

impl WorkflowExecutor {
    /// Create a new executor
    pub fn new(workflow: Workflow, context: ExecutionContext) -> Self {
        Self::with_handler(workflow, context, Arc::new(DefaultNodeHandler))
    }

    /// Create with custom handler
    pub fn with_handler(
        workflow: Workflow,
        context: ExecutionContext,
        handler: Arc<dyn NodeHandler + Send + Sync>,
    ) -> Self {
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
            id: execution_id,
            workflow_id: workflow.id.clone(),
            status: WorkflowExecutionStatus::Pending,
            started_at: now,
            completed_at: None,
            node_states,
            variables: context.variables.clone(),
        };

        Self {
            workflow,
            execution: Arc::new(RwLock::new(execution)),
            context: Arc::new(RwLock::new(context)),
            completed: Arc::new(RwLock::new(HashSet::new())),
            handler,
        }
    }

    /// Get the execution ID
    pub async fn execution_id(&self) -> String {
        let execution = self.execution.read().await;
        execution.id.clone()
    }

    /// Get current execution status
    pub async fn status(&self) -> WorkflowExecutionStatus {
        let execution = self.execution.read().await;
        execution.status
    }

    /// Get execution snapshot
    pub async fn get_execution(&self) -> WorkflowExecution {
        let execution = self.execution.read().await;
        execution.clone()
    }

    /// Run the workflow to completion
    pub async fn run(&self) -> Result<WorkflowExecution, WorkflowError> {
        // Update status to running
        {
            let mut execution = self.execution.write().await;
            execution.status = WorkflowExecutionStatus::Running;
        }

        // Execute nodes in topological order
        loop {
            let ready_nodes = self.get_ready_nodes().await;

            if ready_nodes.is_empty() {
                break;
            }

            // Execute ready nodes (could be parallelized based on options)
            let context = self.context.read().await.clone();

            for node_id in ready_nodes {
                self.execute_node(&node_id, &context).await?;
            }
        }

        // Check if all nodes completed
        let execution = self.execution.read().await;
        let all_completed = execution
            .node_states
            .values()
            .all(|s| s.status.is_terminal());

        drop(execution);

        // Update final status
        {
            let mut execution = self.execution.write().await;
            execution.status = if all_completed {
                WorkflowExecutionStatus::Completed
            } else {
                WorkflowExecutionStatus::Failed
            };
            execution.completed_at = Some(Utc::now());
        }

        Ok(self.get_execution().await)
    }

    /// Get nodes that are ready to execute
    async fn get_ready_nodes(&self) -> Vec<NodeId> {
        let completed = self.completed.read().await;
        self.workflow
            .get_ready_nodes(&completed)
            .into_iter()
            .map(|n| n.id.clone())
            .collect()
    }

    /// Execute a single node
    async fn execute_node(
        &self,
        node_id: &NodeId,
        context: &ExecutionContext,
    ) -> Result<(), WorkflowError> {
        let node = self
            .workflow
            .get_node(node_id)
            .ok_or_else(|| WorkflowError::NodeNotFound(node_id.0.clone()))?;

        // Update node status to running
        {
            let mut execution = self.execution.write().await;
            if let Some(state) = execution.node_states.get_mut(&node_id.0) {
                state.status = NodeStatus::Running;
                state.started_at = Some(Utc::now());
            }
        }

        // Execute based on node type
        let result = match &node.node_type {
            NodeType::Start | NodeType::End => {
                ExecutionResult::success(&node_id.0, serde_json::json!({"type": "control"}))
            }
            NodeType::Task {
                description,
                agent_role,
                ..
            } => {
                if context.options.dry_run {
                    ExecutionResult::success(&node_id.0, serde_json::json!({"dry_run": true}))
                } else {
                    self.handler
                        .execute_task(&node_id.0, description, agent_role.as_deref(), context)
                        .await
                }
            }
            NodeType::Approval { message, approvers } => {
                if context.options.skip_approvals {
                    ExecutionResult::success(&node_id.0, serde_json::json!({"skipped": true}))
                } else {
                    self.handler
                        .handle_approval(&node_id.0, message, approvers)
                        .await
                }
            }
            NodeType::Delay { seconds } => {
                if context.options.dry_run {
                    ExecutionResult::success(&node_id.0, serde_json::json!({"dry_run": true}))
                } else {
                    self.handler.handle_delay(&node_id.0, *seconds).await
                }
            }
            NodeType::Condition {
                expression,
                true_branch,
                false_branch,
            } => {
                // Evaluate condition (simplified - just check if expression is "true")
                let result = expression == "true";
                let next = if result { true_branch } else { false_branch };
                ExecutionResult::success(
                    &node_id.0,
                    serde_json::json!({
                        "condition": expression,
                        "result": result,
                        "next": next.0
                    }),
                )
            }
            NodeType::Parallel { branches } => ExecutionResult::success(
                &node_id.0,
                serde_json::json!({
                    "branches": branches.iter().map(|b| &b.0).collect::<Vec<_>>()
                }),
            ),
            NodeType::Join { required } => ExecutionResult::success(
                &node_id.0,
                serde_json::json!({
                    "required": required.iter().map(|b| &b.0).collect::<Vec<_>>()
                }),
            ),
            NodeType::Loop {
                condition,
                max_iterations,
                body,
            } => ExecutionResult::success(
                &node_id.0,
                serde_json::json!({
                    "condition": condition,
                    "max_iterations": max_iterations,
                    "body": body.0
                }),
            ),
            NodeType::SubWorkflow {
                workflow_id,
                inputs,
            } => ExecutionResult::success(
                &node_id.0,
                serde_json::json!({
                    "sub_workflow": workflow_id,
                    "inputs": inputs
                }),
            ),
        };

        // Update node status
        {
            let mut execution = self.execution.write().await;
            if let Some(state) = execution.node_states.get_mut(&node_id.0) {
                state.status = if result.success {
                    NodeStatus::Completed
                } else {
                    NodeStatus::Failed
                };
                state.completed_at = Some(Utc::now());
                state.output = result.output.clone();
                state.error = result.error.clone();
            }
        }

        // Mark as completed
        if result.success {
            let mut completed = self.completed.write().await;
            completed.insert(node_id.clone());
        }

        Ok(())
    }

    /// Pause the workflow
    pub async fn pause(&self) {
        let mut execution = self.execution.write().await;
        if execution.status == WorkflowExecutionStatus::Running {
            execution.status = WorkflowExecutionStatus::Paused;
        }
    }

    /// Cancel the workflow
    pub async fn cancel(&self) {
        let mut execution = self.execution.write().await;
        execution.status = WorkflowExecutionStatus::Cancelled;
        execution.completed_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::graph::WorkflowBuilder;

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext::new()
            .with_variable("key", "value")
            .with_agent_assignment("task-1", "agent-1");

        assert!(context.get_variable("key").is_some());
        assert!(context.agent_assignments.contains_key("task-1"));
    }

    #[test]
    fn test_execution_options_default() {
        let options = ExecutionOptions::default();
        assert_eq!(options.max_parallel, 4);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success("node-1", serde_json::json!({"data": 123}));
        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::failure("node-1", "Something went wrong");
        assert!(!result.success);
        assert!(result.output.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_execution_result_with_duration() {
        let result = ExecutionResult::success("node-1", serde_json::json!({})).with_duration(1500);
        assert_eq!(result.duration_ms, 1500);
    }

    #[tokio::test]
    async fn test_workflow_executor_creation() {
        let workflow = WorkflowBuilder::new("test")
            .name("Test")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        assert_eq!(executor.status().await, WorkflowExecutionStatus::Pending);
    }

    #[tokio::test]
    async fn test_workflow_executor_run_simple() {
        let workflow = WorkflowBuilder::new("simple")
            .name("Simple Workflow")
            .start_node("start")
            .task_node("task", "Do something", None)
            .end_node("end")
            .connect("start", "task")
            .connect("task", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        let result = executor.run().await.unwrap();
        assert_eq!(result.status, WorkflowExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_workflow_executor_dry_run() {
        let workflow = WorkflowBuilder::new("dry-run")
            .name("Dry Run Test")
            .start_node("start")
            .task_node("task", "Heavy task", None)
            .end_node("end")
            .connect("start", "task")
            .connect("task", "end")
            .build()
            .unwrap();

        let context = ExecutionContext {
            options: ExecutionOptions {
                dry_run: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let executor = WorkflowExecutor::new(workflow, context);
        let result = executor.run().await.unwrap();
        assert_eq!(result.status, WorkflowExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_workflow_executor_parallel() {
        let workflow = WorkflowBuilder::new("parallel")
            .name("Parallel Workflow")
            .start_node("start")
            .task_node("task-a", "Task A", None)
            .task_node("task-b", "Task B", None)
            .end_node("end")
            .connect("start", "task-a")
            .connect("start", "task-b")
            .connect("task-a", "end")
            .connect("task-b", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        let result = executor.run().await.unwrap();
        assert_eq!(result.status, WorkflowExecutionStatus::Completed);
        assert_eq!(result.node_states.len(), 4);
    }

    #[tokio::test]
    async fn test_workflow_executor_cancel() {
        let workflow = WorkflowBuilder::new("cancel")
            .name("Cancel Test")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        executor.cancel().await;
        assert_eq!(executor.status().await, WorkflowExecutionStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_workflow_executor_pause() {
        let workflow = WorkflowBuilder::new("pause")
            .name("Pause Test")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        // Start running
        {
            let mut exec = executor.execution.write().await;
            exec.status = WorkflowExecutionStatus::Running;
        }

        executor.pause().await;
        assert_eq!(executor.status().await, WorkflowExecutionStatus::Paused);
    }

    #[tokio::test]
    async fn test_workflow_executor_with_variables() {
        let workflow = WorkflowBuilder::new("vars")
            .name("Variable Test")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new()
            .with_variable("input", "test-value")
            .with_variable("count", 42);

        let executor = WorkflowExecutor::new(workflow, context);
        let result = executor.run().await.unwrap();

        assert!(result.variables.contains_key("input"));
        assert_eq!(result.variables.get("count"), Some(&serde_json::json!(42)));
    }

    #[tokio::test]
    async fn test_workflow_executor_get_execution() {
        let workflow = WorkflowBuilder::new("snapshot")
            .name("Snapshot Test")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        let context = ExecutionContext::new();
        let executor = WorkflowExecutor::new(workflow, context);

        let execution = executor.get_execution().await;
        assert_eq!(execution.workflow_id, "snapshot");
        assert!(!execution.id.is_empty());
    }
}
