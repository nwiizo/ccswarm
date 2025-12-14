//! Workflow node definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a new node ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Type of workflow node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Start node - entry point
    Start,
    /// End node - termination point
    End,
    /// Task node - executes an agent task
    Task {
        /// Task description
        description: String,
        /// Agent role to assign
        agent_role: Option<String>,
        /// Timeout in seconds
        timeout_secs: Option<u64>,
    },
    /// Conditional branch
    Condition {
        /// Condition expression
        expression: String,
        /// Branch for true result
        true_branch: NodeId,
        /// Branch for false result
        false_branch: NodeId,
    },
    /// Parallel execution fork
    Parallel {
        /// Nodes to execute in parallel
        branches: Vec<NodeId>,
    },
    /// Join node - wait for parallel branches
    Join {
        /// Required branches to complete
        required: Vec<NodeId>,
    },
    /// Approval gate - requires human approval
    Approval {
        /// Approval message
        message: String,
        /// Required approvers
        approvers: Vec<String>,
    },
    /// Delay node
    Delay {
        /// Delay in seconds
        seconds: u64,
    },
    /// Loop node
    Loop {
        /// Condition to continue loop
        condition: String,
        /// Maximum iterations
        max_iterations: u32,
        /// Body node to execute
        body: NodeId,
    },
    /// Sub-workflow invocation
    SubWorkflow {
        /// Workflow ID to invoke
        workflow_id: String,
        /// Input mapping
        inputs: HashMap<String, String>,
    },
}

/// Status of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Not yet started
    Pending,
    /// Ready to execute
    Ready,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Skipped (branch not taken)
    Skipped,
    /// Waiting for input/approval
    Waiting,
    /// Cancelled
    Cancelled,
}

impl NodeStatus {
    /// Check if the node is terminal (no more transitions)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            NodeStatus::Completed
                | NodeStatus::Failed
                | NodeStatus::Skipped
                | NodeStatus::Cancelled
        )
    }

    /// Check if the node can be started
    pub fn can_start(&self) -> bool {
        matches!(self, NodeStatus::Pending | NodeStatus::Ready)
    }
}

/// A node in the workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Unique node ID
    pub id: NodeId,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Input edges (dependencies)
    pub inputs: Vec<NodeId>,
    /// Output edges (next nodes)
    pub outputs: Vec<NodeId>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Retry configuration for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries
    pub max_delay_ms: u64,
    /// Delay multiplier for exponential backoff
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        delay.min(self.max_delay_ms as f64) as u64
    }
}

impl WorkflowNode {
    /// Create a new node
    pub fn new(id: impl Into<NodeId>, name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: HashMap::new(),
            retry_config: None,
        }
    }

    /// Add an input edge
    pub fn with_input(mut self, input: impl Into<NodeId>) -> Self {
        self.inputs.push(input.into());
        self
    }

    /// Add an output edge
    pub fn with_output(mut self, output: impl Into<NodeId>) -> Self {
        self.outputs.push(output.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Check if this is a start node
    pub fn is_start(&self) -> bool {
        matches!(self.node_type, NodeType::Start)
    }

    /// Check if this is an end node
    pub fn is_end(&self) -> bool {
        matches!(self.node_type, NodeType::End)
    }

    /// Check if this node has no inputs (entry point)
    pub fn has_no_inputs(&self) -> bool {
        self.inputs.is_empty()
    }

    /// Check if this node has no outputs (exit point)
    pub fn has_no_outputs(&self) -> bool {
        self.outputs.is_empty()
    }
}

/// Node builder for fluent construction
#[allow(dead_code)]
pub struct NodeBuilder {
    id: NodeId,
    name: String,
    node_type: Option<NodeType>,
    inputs: Vec<NodeId>,
    outputs: Vec<NodeId>,
    metadata: HashMap<String, serde_json::Value>,
    retry_config: Option<RetryConfig>,
}

#[allow(dead_code)]
impl NodeBuilder {
    /// Create a new node builder
    pub fn new(id: impl Into<NodeId>) -> Self {
        let id = id.into();
        Self {
            name: id.0.clone(),
            id,
            node_type: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: HashMap::new(),
            retry_config: None,
        }
    }

    /// Set the node name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set as start node
    pub fn start(mut self) -> Self {
        self.node_type = Some(NodeType::Start);
        self
    }

    /// Set as end node
    pub fn end(mut self) -> Self {
        self.node_type = Some(NodeType::End);
        self
    }

    /// Set as task node
    pub fn task(mut self, description: impl Into<String>) -> Self {
        self.node_type = Some(NodeType::Task {
            description: description.into(),
            agent_role: None,
            timeout_secs: None,
        });
        self
    }

    /// Set as task with agent role
    pub fn task_with_agent(
        mut self,
        description: impl Into<String>,
        agent_role: impl Into<String>,
    ) -> Self {
        self.node_type = Some(NodeType::Task {
            description: description.into(),
            agent_role: Some(agent_role.into()),
            timeout_secs: None,
        });
        self
    }

    /// Set as delay node
    pub fn delay(mut self, seconds: u64) -> Self {
        self.node_type = Some(NodeType::Delay { seconds });
        self
    }

    /// Set as approval node
    pub fn approval(mut self, message: impl Into<String>) -> Self {
        self.node_type = Some(NodeType::Approval {
            message: message.into(),
            approvers: Vec::new(),
        });
        self
    }

    /// Add input dependency
    pub fn depends_on(mut self, node: impl Into<NodeId>) -> Self {
        self.inputs.push(node.into());
        self
    }

    /// Add output edge
    pub fn then(mut self, node: impl Into<NodeId>) -> Self {
        self.outputs.push(node.into());
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set retry configuration
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Build the node
    pub fn build(self) -> Result<WorkflowNode, String> {
        let node_type = self
            .node_type
            .ok_or_else(|| "Node type is required".to_string())?;

        Ok(WorkflowNode {
            id: self.id,
            name: self.name,
            node_type,
            inputs: self.inputs,
            outputs: self.outputs,
            metadata: self.metadata,
            retry_config: self.retry_config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new("test-node");
        assert_eq!(id.0, "test-node");
        assert_eq!(format!("{}", id), "test-node");
    }

    #[test]
    fn test_node_id_from() {
        let id: NodeId = "from-str".into();
        assert_eq!(id.0, "from-str");

        let id: NodeId = String::from("from-string").into();
        assert_eq!(id.0, "from-string");
    }

    #[test]
    fn test_node_status() {
        assert!(NodeStatus::Completed.is_terminal());
        assert!(NodeStatus::Failed.is_terminal());
        assert!(!NodeStatus::Running.is_terminal());
        assert!(!NodeStatus::Pending.is_terminal());

        assert!(NodeStatus::Pending.can_start());
        assert!(NodeStatus::Ready.can_start());
        assert!(!NodeStatus::Running.can_start());
    }

    #[test]
    fn test_workflow_node_creation() {
        let node = WorkflowNode::new("task-1", "First Task", NodeType::Start);

        assert_eq!(node.id.0, "task-1");
        assert_eq!(node.name, "First Task");
        assert!(node.is_start());
        assert!(!node.is_end());
    }

    #[test]
    fn test_node_with_edges() {
        let node = WorkflowNode::new("middle", "Middle Node", NodeType::Start)
            .with_input("start")
            .with_output("end");

        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert!(!node.has_no_inputs());
        assert!(!node.has_no_outputs());
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);

        let delay0 = config.delay_for_attempt(0);
        assert_eq!(delay0, 1000);

        let delay1 = config.delay_for_attempt(1);
        assert_eq!(delay1, 2000);

        let delay2 = config.delay_for_attempt(2);
        assert_eq!(delay2, 4000);
    }

    #[test]
    fn test_retry_config_max_delay() {
        let config = RetryConfig {
            max_delay_ms: 5000,
            ..Default::default()
        };

        let delay10 = config.delay_for_attempt(10);
        assert!(delay10 <= 5000);
    }

    #[test]
    fn test_node_builder_start() {
        let node = NodeBuilder::new("start")
            .name("Start Node")
            .start()
            .build()
            .unwrap();

        assert!(node.is_start());
        assert_eq!(node.name, "Start Node");
    }

    #[test]
    fn test_node_builder_task() {
        let node = NodeBuilder::new("task-1")
            .name("My Task")
            .task("Do something important")
            .depends_on("start")
            .then("end")
            .build()
            .unwrap();

        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        match node.node_type {
            NodeType::Task { description, .. } => {
                assert_eq!(description, "Do something important");
            }
            _ => panic!("Expected Task node type"),
        }
    }

    #[test]
    fn test_node_builder_with_agent() {
        let node = NodeBuilder::new("frontend-task")
            .task_with_agent("Create UI component", "frontend")
            .build()
            .unwrap();

        match node.node_type {
            NodeType::Task { agent_role, .. } => {
                assert_eq!(agent_role, Some("frontend".to_string()));
            }
            _ => panic!("Expected Task node type"),
        }
    }

    #[test]
    fn test_node_builder_delay() {
        let node = NodeBuilder::new("wait").delay(30).build().unwrap();

        match node.node_type {
            NodeType::Delay { seconds } => assert_eq!(seconds, 30),
            _ => panic!("Expected Delay node type"),
        }
    }

    #[test]
    fn test_node_builder_approval() {
        let node = NodeBuilder::new("approve")
            .approval("Please review this deployment")
            .build()
            .unwrap();

        match node.node_type {
            NodeType::Approval { message, .. } => {
                assert_eq!(message, "Please review this deployment");
            }
            _ => panic!("Expected Approval node type"),
        }
    }

    #[test]
    fn test_node_builder_missing_type() {
        let result = NodeBuilder::new("incomplete").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_node_with_metadata() {
        let node = WorkflowNode::new("meta-node", "Node with Metadata", NodeType::Start)
            .with_metadata("priority", "high");

        assert!(node.metadata.contains_key("priority"));
    }

    #[test]
    fn test_node_with_retry() {
        let node = WorkflowNode::new("retry-node", "Node with Retry", NodeType::Start).with_retry(
            RetryConfig {
                max_attempts: 5,
                ..Default::default()
            },
        );

        assert!(node.retry_config.is_some());
        assert_eq!(node.retry_config.unwrap().max_attempts, 5);
    }
}
