//! Workflow graph definition and validation

use super::node::{NodeId, NodeType, WorkflowNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Error types for workflow operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// Duplicate workflow/node ID
    DuplicateId(String),
    /// Node not found
    NodeNotFound(String),
    /// Workflow not found
    NotFound(String),
    /// Invalid edge
    InvalidEdge { from: String, to: String },
    /// Cycle detected
    CycleDetected(Vec<String>),
    /// No start node
    NoStartNode,
    /// No end node
    NoEndNode,
    /// Multiple start nodes
    MultipleStartNodes,
    /// Unreachable nodes
    UnreachableNodes(Vec<String>),
    /// Validation error
    ValidationError(String),
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowError::DuplicateId(id) => write!(f, "Duplicate ID: {}", id),
            WorkflowError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            WorkflowError::NotFound(id) => write!(f, "Workflow not found: {}", id),
            WorkflowError::InvalidEdge { from, to } => {
                write!(f, "Invalid edge: {} -> {}", from, to)
            }
            WorkflowError::CycleDetected(path) => write!(f, "Cycle detected: {:?}", path),
            WorkflowError::NoStartNode => write!(f, "No start node defined"),
            WorkflowError::NoEndNode => write!(f, "No end node defined"),
            WorkflowError::MultipleStartNodes => write!(f, "Multiple start nodes defined"),
            WorkflowError::UnreachableNodes(nodes) => write!(f, "Unreachable nodes: {:?}", nodes),
            WorkflowError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for WorkflowError {}

/// A workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Nodes in the workflow
    pub nodes: Vec<WorkflowNode>,
    /// Version
    pub version: String,
    /// Tags
    pub tags: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Workflow {
    /// Get a node by ID
    pub fn get_node(&self, id: &NodeId) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| &n.id == id)
    }

    /// Get mutable node by ID
    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut WorkflowNode> {
        self.nodes.iter_mut().find(|n| &n.id == id)
    }

    /// Find start nodes
    pub fn start_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes.iter().filter(|n| n.is_start()).collect()
    }

    /// Find end nodes
    pub fn end_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes.iter().filter(|n| n.is_end()).collect()
    }

    /// Get entry nodes (no inputs)
    pub fn entry_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes.iter().filter(|n| n.has_no_inputs()).collect()
    }

    /// Get exit nodes (no outputs)
    pub fn exit_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes.iter().filter(|n| n.has_no_outputs()).collect()
    }

    /// Get successors of a node
    pub fn successors(&self, node_id: &NodeId) -> Vec<&WorkflowNode> {
        if let Some(node) = self.get_node(node_id) {
            node.outputs
                .iter()
                .filter_map(|id| self.get_node(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get predecessors of a node
    pub fn predecessors(&self, node_id: &NodeId) -> Vec<&WorkflowNode> {
        if let Some(node) = self.get_node(node_id) {
            node.inputs
                .iter()
                .filter_map(|id| self.get_node(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Validate the workflow
    pub fn validate(&self) -> Result<(), WorkflowError> {
        // Check for duplicate node IDs
        let mut seen_ids: HashSet<&str> = HashSet::new();
        for node in &self.nodes {
            if !seen_ids.insert(&node.id.0) {
                return Err(WorkflowError::DuplicateId(node.id.0.clone()));
            }
        }

        // Check all edges reference valid nodes
        for node in &self.nodes {
            for input in &node.inputs {
                if self.get_node(input).is_none() {
                    return Err(WorkflowError::NodeNotFound(input.0.clone()));
                }
            }
            for output in &node.outputs {
                if self.get_node(output).is_none() {
                    return Err(WorkflowError::NodeNotFound(output.0.clone()));
                }
            }
        }

        // Check for cycles using DFS
        if let Some(cycle) = self.find_cycle() {
            return Err(WorkflowError::CycleDetected(cycle));
        }

        Ok(())
    }

    /// Find a cycle in the graph using DFS
    fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();

        for node in &self.nodes {
            if self.dfs_find_cycle(&node.id.0, &mut visited, &mut rec_stack, &mut path) {
                return Some(path);
            }
        }

        None
    }

    fn dfs_find_cycle(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if rec_stack.contains(node_id) {
            path.push(node_id.to_string());
            return true;
        }

        if visited.contains(node_id) {
            return false;
        }

        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        path.push(node_id.to_string());

        if let Some(node) = self.nodes.iter().find(|n| n.id.0 == node_id) {
            for output_id in &node.outputs {
                if self.dfs_find_cycle(&output_id.0, visited, rec_stack, path) {
                    return true;
                }
            }
        }

        rec_stack.remove(node_id);
        path.pop();
        false
    }

    /// Get topological ordering of nodes
    pub fn topological_sort(&self) -> Result<Vec<&WorkflowNode>, WorkflowError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            in_degree.entry(&node.id.0).or_insert(0);
            for output in &node.outputs {
                *in_degree.entry(&output.0).or_insert(0) += 1;
            }
        }

        // Adjust: we need to count inputs, not outputs
        in_degree.clear();
        for node in &self.nodes {
            in_degree.insert(&node.id.0, node.inputs.len());
        }

        let mut queue: VecDeque<&WorkflowNode> = VecDeque::new();
        for node in &self.nodes {
            if node.inputs.is_empty() {
                queue.push_back(node);
            }
        }

        let mut result: Vec<&WorkflowNode> = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            for output_id in &node.outputs {
                if let Some(count) = in_degree.get_mut(output_id.0.as_str()) {
                    *count = count.saturating_sub(1);
                    if *count == 0
                        && let Some(next_node) = self.get_node(output_id)
                    {
                        queue.push_back(next_node);
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(WorkflowError::CycleDetected(vec![
                "Cycle in graph".to_string(),
            ]));
        }

        Ok(result)
    }

    /// Get ready nodes (all predecessors completed)
    pub fn get_ready_nodes(&self, completed: &HashSet<NodeId>) -> Vec<&WorkflowNode> {
        self.nodes
            .iter()
            .filter(|node| {
                !completed.contains(&node.id)
                    && node.inputs.iter().all(|input| completed.contains(input))
            })
            .collect()
    }
}

/// Builder for constructing workflows
pub struct WorkflowBuilder {
    id: String,
    name: String,
    description: Option<String>,
    nodes: Vec<WorkflowNode>,
    version: String,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            description: None,
            nodes: Vec::new(),
            version: "1.0.0".to_string(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set workflow name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add a node
    pub fn node(mut self, node: WorkflowNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add multiple nodes
    pub fn nodes(mut self, nodes: Vec<WorkflowNode>) -> Self {
        self.nodes.extend(nodes);
        self
    }

    /// Add a start node
    pub fn start_node(self, id: impl Into<NodeId>) -> Self {
        let id = id.into();
        self.node(WorkflowNode::new(
            id.clone(),
            format!("Start: {}", id.0),
            NodeType::Start,
        ))
    }

    /// Add an end node
    pub fn end_node(self, id: impl Into<NodeId>) -> Self {
        let id = id.into();
        self.node(WorkflowNode::new(
            id.clone(),
            format!("End: {}", id.0),
            NodeType::End,
        ))
    }

    /// Add a task node
    pub fn task_node(
        self,
        id: impl Into<NodeId>,
        description: impl Into<String>,
        agent_role: Option<String>,
    ) -> Self {
        let id = id.into();
        let desc = description.into();
        self.node(WorkflowNode::new(
            id.clone(),
            desc.clone(),
            NodeType::Task {
                description: desc,
                agent_role,
                timeout_secs: None,
            },
        ))
    }

    /// Connect two nodes
    pub fn connect(mut self, from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        let from_id = from.into();
        let to_id = to.into();

        // Find and update the source node
        for node in &mut self.nodes {
            if node.id == from_id && !node.outputs.contains(&to_id) {
                node.outputs.push(to_id.clone());
            }
            if node.id == to_id && !node.inputs.contains(&from_id) {
                node.inputs.push(from_id.clone());
            }
        }

        self
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        if self.name.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Workflow name is required".to_string(),
            ));
        }

        let workflow = Workflow {
            id: self.id,
            name: self.name,
            description: self.description,
            nodes: self.nodes,
            version: self.version,
            tags: self.tags,
            metadata: self.metadata,
        };

        workflow.validate()?;
        Ok(workflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder_basic() {
        let workflow = WorkflowBuilder::new("test-wf")
            .name("Test Workflow")
            .description("A test workflow")
            .version("2.0.0")
            .tag("test")
            .build()
            .unwrap();

        assert_eq!(workflow.id, "test-wf");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, Some("A test workflow".to_string()));
        assert_eq!(workflow.version, "2.0.0");
        assert!(workflow.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_workflow_with_nodes() {
        let workflow = WorkflowBuilder::new("node-wf")
            .name("Node Workflow")
            .start_node("start")
            .end_node("end")
            .connect("start", "end")
            .build()
            .unwrap();

        assert_eq!(workflow.nodes.len(), 2);
        assert_eq!(workflow.start_nodes().len(), 1);
        assert_eq!(workflow.end_nodes().len(), 1);
    }

    #[test]
    fn test_workflow_successors_predecessors() {
        let workflow = WorkflowBuilder::new("succ-pred")
            .name("Successor/Predecessor Test")
            .start_node("start")
            .task_node("middle", "Middle Task", None)
            .end_node("end")
            .connect("start", "middle")
            .connect("middle", "end")
            .build()
            .unwrap();

        let middle_id = NodeId::new("middle");
        let successors = workflow.successors(&middle_id);
        assert_eq!(successors.len(), 1);
        assert!(successors[0].is_end());

        let predecessors = workflow.predecessors(&middle_id);
        assert_eq!(predecessors.len(), 1);
        assert!(predecessors[0].is_start());
    }

    #[test]
    fn test_workflow_validation_duplicate_id() {
        let node1 = WorkflowNode::new("dup", "First", NodeType::Start);
        let node2 = WorkflowNode::new("dup", "Second", NodeType::End);

        let result = WorkflowBuilder::new("dup-test")
            .name("Duplicate Test")
            .node(node1)
            .node(node2)
            .build();

        assert!(matches!(result, Err(WorkflowError::DuplicateId(_))));
    }

    #[test]
    fn test_workflow_validation_invalid_edge() {
        let mut node = WorkflowNode::new("start", "Start", NodeType::Start);
        node.outputs.push(NodeId::new("nonexistent"));

        let result = WorkflowBuilder::new("invalid-edge")
            .name("Invalid Edge Test")
            .node(node)
            .build();

        assert!(matches!(result, Err(WorkflowError::NodeNotFound(_))));
    }

    #[test]
    fn test_workflow_topological_sort() {
        let workflow = WorkflowBuilder::new("topo-sort")
            .name("Topological Sort Test")
            .start_node("a")
            .task_node("b", "Task B", None)
            .task_node("c", "Task C", None)
            .end_node("d")
            .connect("a", "b")
            .connect("a", "c")
            .connect("b", "d")
            .connect("c", "d")
            .build()
            .unwrap();

        let sorted = workflow.topological_sort().unwrap();
        assert_eq!(sorted.len(), 4);

        // Start should be first
        assert_eq!(sorted[0].id.0, "a");
        // End should be last
        assert_eq!(sorted[3].id.0, "d");
    }

    #[test]
    fn test_workflow_get_ready_nodes() {
        let workflow = WorkflowBuilder::new("ready-test")
            .name("Ready Nodes Test")
            .start_node("start")
            .task_node("task1", "Task 1", None)
            .task_node("task2", "Task 2", None)
            .end_node("end")
            .connect("start", "task1")
            .connect("start", "task2")
            .connect("task1", "end")
            .connect("task2", "end")
            .build()
            .unwrap();

        // Initially, only start should be ready
        let ready = workflow.get_ready_nodes(&HashSet::new());
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id.0, "start");

        // After start is done, task1 and task2 should be ready
        let mut completed = HashSet::new();
        completed.insert(NodeId::new("start"));
        let ready = workflow.get_ready_nodes(&completed);
        assert_eq!(ready.len(), 2);

        // After task1 and task2 are done, end should be ready
        completed.insert(NodeId::new("task1"));
        completed.insert(NodeId::new("task2"));
        let ready = workflow.get_ready_nodes(&completed);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id.0, "end");
    }

    #[test]
    fn test_workflow_entry_exit_nodes() {
        let workflow = WorkflowBuilder::new("entry-exit")
            .name("Entry/Exit Test")
            .start_node("entry1")
            .start_node("entry2")
            .end_node("exit")
            .connect("entry1", "exit")
            .connect("entry2", "exit")
            .build()
            .unwrap();

        assert_eq!(workflow.entry_nodes().len(), 2);
        assert_eq!(workflow.exit_nodes().len(), 1);
    }

    #[test]
    fn test_workflow_error_display() {
        let err = WorkflowError::DuplicateId("test-id".to_string());
        assert!(err.to_string().contains("test-id"));

        let err = WorkflowError::CycleDetected(vec!["a".to_string(), "b".to_string()]);
        assert!(err.to_string().contains("Cycle"));
    }

    #[test]
    fn test_workflow_missing_name() {
        let result = WorkflowBuilder::new("no-name").build();
        assert!(matches!(result, Err(WorkflowError::ValidationError(_))));
    }

    #[test]
    fn test_workflow_with_task_and_agent() {
        let workflow = WorkflowBuilder::new("agent-wf")
            .name("Agent Workflow")
            .start_node("start")
            .task_node("frontend-task", "Create UI", Some("frontend".to_string()))
            .task_node("backend-task", "Create API", Some("backend".to_string()))
            .end_node("end")
            .connect("start", "frontend-task")
            .connect("start", "backend-task")
            .connect("frontend-task", "end")
            .connect("backend-task", "end")
            .build()
            .unwrap();

        assert_eq!(workflow.nodes.len(), 4);
    }
}
