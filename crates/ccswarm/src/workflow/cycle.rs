//! Cycle and loop detection for Flow/Stage workflows.
//!
//! Two complementary guards against runaway stage routing:
//! - **Static analysis** (`analyze_flow`): detect structural cycles in a
//!   flow's rule graph before execution (`ccswarm flow check`).
//! - **Runtime tracking** (`LoopTracker`): count per-stage visits during
//!   execution and signal when a stage exceeds `Flow::max_stage_visits`,
//!   so review→fix loops stay bounded even when `max_stages` is generous.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, warn};

use super::flow::Flow;

/// Result of static cycle analysis on a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleAnalysis {
    /// Whether the workflow contains any cycles
    pub has_cycles: bool,
    /// List of detected cycle paths (each path is a sequence of stage IDs)
    pub cycle_paths: Vec<Vec<String>>,
    /// Maximum depth of the workflow graph
    pub max_depth: usize,
    /// Stages that are part of cycles
    pub cyclic_movements: HashSet<String>,
}

/// Perform static analysis on a flow to detect cycles in the stage graph.
///
/// Cycles are not inherently errors (review→fix loops are cycles by design);
/// callers should surface them as informational warnings noting that runtime
/// execution bounds them via `max_stage_visits`.
pub fn analyze_flow(flow: &Flow) -> Result<CycleAnalysis> {
    debug!("Starting static cycle analysis for flow '{}'", flow.name);

    // Build adjacency list for the stage graph
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for stage in &flow.stages {
        let edges: Vec<String> = stage.rules.iter().map(|r| r.next.clone()).collect();
        graph.insert(stage.id.clone(), edges);
    }

    // Detect cycles using DFS with path tracking
    let mut cycle_paths = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for stage in &flow.stages {
        if !visited.contains(&stage.id) {
            dfs_detect_cycles(
                &stage.id,
                &graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycle_paths,
            )?;
        }
    }

    // Calculate max depth
    let max_depth = calculate_max_depth(flow);

    // Collect all stages that are part of cycles
    let mut cyclic_movements = HashSet::new();
    for cycle_path in &cycle_paths {
        for movement_id in cycle_path {
            cyclic_movements.insert(movement_id.clone());
        }
    }

    let has_cycles = !cycle_paths.is_empty();

    if has_cycles {
        info!(
            "Flow '{}' contains {} cycle(s): max_depth={}, cyclic_movements={}",
            flow.name,
            cycle_paths.len(),
            max_depth,
            cyclic_movements.len()
        );
    } else {
        info!("Flow '{}' is acyclic: max_depth={}", flow.name, max_depth);
    }

    Ok(CycleAnalysis {
        has_cycles,
        cycle_paths,
        max_depth,
        cyclic_movements,
    })
}

/// DFS-based cycle detection with path tracking
fn dfs_detect_cycles(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycle_paths: &mut Vec<Vec<String>>,
) -> Result<()> {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_detect_cycles(neighbor, graph, visited, rec_stack, path, cycle_paths)?;
            } else if rec_stack.contains(neighbor) {
                // Cycle detected - extract the cycle path
                let cycle_start = path
                    .iter()
                    .position(|n| n == neighbor)
                    .context("Cycle start not found in path")?;
                let cycle_path = path[cycle_start..].to_vec();
                debug!("Detected cycle: {:?}", cycle_path);
                cycle_paths.push(cycle_path);
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
    Ok(())
}

/// Calculate the maximum depth of the workflow graph using BFS
fn calculate_max_depth(flow: &Flow) -> usize {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for stage in &flow.stages {
        let edges: Vec<String> = stage.rules.iter().map(|r| r.next.clone()).collect();
        graph.insert(stage.id.clone(), edges);
    }

    let mut max_depth = 0;
    let mut queue = VecDeque::new();
    queue.push_back((flow.initial_movement.clone(), 0));

    let mut visited = HashSet::new();
    visited.insert(flow.initial_movement.clone());

    while let Some((node, depth)) = queue.pop_front() {
        max_depth = max_depth.max(depth);

        if let Some(neighbors) = graph.get(&node) {
            for neighbor in neighbors {
                // Only visit each node once to avoid infinite loops
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
    }

    max_depth
}

/// Runtime tracker for monitoring stage visits during execution.
///
/// `FlowEngine` records each stage visit; when a stage is visited more than
/// `max_visits` times the flow is aborted, with the repeating transition
/// pattern (if any) included in the diagnostics.
#[derive(Debug, Clone)]
pub struct LoopTracker {
    /// Visit count per stage ID
    visit_counts: HashMap<String, u32>,
    /// History of stage transitions (for pattern detection)
    transition_history: Vec<String>,
    /// Maximum visits per stage before the flow aborts
    max_visits: u32,
}

impl LoopTracker {
    /// Create a new loop tracker allowing up to `max_visits` visits per stage
    pub fn new(max_visits: u32) -> Self {
        Self {
            visit_counts: HashMap::new(),
            transition_history: Vec::new(),
            max_visits,
        }
    }

    /// Record a visit to a stage, returns whether the loop threshold was exceeded
    pub fn record_visit(&mut self, movement_id: &str) -> bool {
        let count = self
            .visit_counts
            .entry(movement_id.to_string())
            .or_insert(0);
        *count += 1;

        self.transition_history.push(movement_id.to_string());

        let exceeded = *count > self.max_visits;

        if exceeded {
            warn!(
                "Loop threshold exceeded for stage '{}': {} visits (max: {})",
                movement_id, count, self.max_visits
            );
        }

        exceeded
    }

    /// Get the visit count for a stage
    pub fn visit_count(&self, movement_id: &str) -> u32 {
        self.visit_counts.get(movement_id).copied().unwrap_or(0)
    }

    /// Detect repeating patterns in the transition history
    pub fn detect_pattern(&self) -> Option<Vec<String>> {
        if self.transition_history.len() < 4 {
            return None;
        }

        // Look for repeating sequences of length 2-10
        for pattern_len in 2..=10.min(self.transition_history.len() / 2) {
            if self.transition_history.len() < pattern_len * 2 {
                continue;
            }

            let recent = &self.transition_history[self.transition_history.len() - pattern_len..];
            let prev = &self.transition_history[self.transition_history.len() - pattern_len * 2
                ..self.transition_history.len() - pattern_len];

            if recent == prev {
                debug!("Detected repeating pattern: {:?}", recent);
                return Some(recent.to_vec());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::flow::{MovementPermission, MovementRule, RuleCondition, Stage};

    fn make_movement(id: &str, rules: Vec<(&str, &str)>) -> Stage {
        Stage {
            id: id.to_string(),
            persona: None,
            policy: None,
            knowledge: None,
            provider: None,
            model: None,
            instruction: format!("Instruction for {}", id),
            tools: vec![],
            permission: MovementPermission::Readonly,
            rules: rules
                .into_iter()
                .map(|(condition, next)| MovementRule {
                    condition: RuleCondition::Simple(condition.to_string()),
                    next: next.to_string(),
                    priority: 0,
                    interactive_only: false,
                    requires_user_input: false,
                })
                .collect(),
            parallel: false,
            sub_movements: vec![],
            output_contract: None,
            timeout: None,
            max_retries: 0,
            agent: None,
            working_dir: None,
            retry_delay_ms: 1000,
            pass_previous_response: true,
            call: None,
        }
    }

    fn make_flow(name: &str, initial: &str, stages: Vec<Stage>) -> Flow {
        Flow {
            name: name.to_string(),
            description: format!("{} test flow", name),
            max_stages: 10,
            max_stage_visits: 3,
            initial_movement: initial.to_string(),
            stages,
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        }
    }

    #[test]
    fn test_simple_cycle_detection() {
        // A -> B -> A
        let flow = make_flow(
            "simple-cycle",
            "A",
            vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "A")]),
            ],
        );

        let analysis = analyze_flow(&flow).unwrap();

        assert!(analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 1);
        assert!(analysis.cyclic_movements.contains("A"));
        assert!(analysis.cyclic_movements.contains("B"));
    }

    #[test]
    fn test_complex_cycle_detection() {
        // A -> B -> C -> A
        let flow = make_flow(
            "complex-cycle",
            "A",
            vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "C")]),
                make_movement("C", vec![("success", "A")]),
            ],
        );

        let analysis = analyze_flow(&flow).unwrap();

        assert!(analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 1);
        assert_eq!(analysis.cyclic_movements.len(), 3);
    }

    #[test]
    fn test_no_cycle_linear_workflow() {
        // A -> B -> C (no cycles)
        let flow = make_flow(
            "linear",
            "A",
            vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "C")]),
                make_movement("C", vec![]),
            ],
        );

        let analysis = analyze_flow(&flow).unwrap();

        assert!(!analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 0);
        assert_eq!(analysis.cyclic_movements.len(), 0);
        assert_eq!(analysis.max_depth, 2);
    }

    #[test]
    fn test_runtime_loop_tracking() {
        let mut tracker = LoopTracker::new(3);

        assert!(!tracker.record_visit("A"));
        assert_eq!(tracker.visit_count("A"), 1);

        assert!(!tracker.record_visit("B"));
        assert_eq!(tracker.visit_count("B"), 1);

        assert!(!tracker.record_visit("A"));
        assert_eq!(tracker.visit_count("A"), 2);

        assert!(!tracker.record_visit("A"));
        assert_eq!(tracker.visit_count("A"), 3);

        // Fourth visit should exceed threshold
        assert!(tracker.record_visit("A"));
        assert_eq!(tracker.visit_count("A"), 4);
    }

    #[test]
    fn test_pattern_detection() {
        let mut tracker = LoopTracker::new(5);

        // Create pattern: A -> B -> A -> B
        tracker.record_visit("A");
        tracker.record_visit("B");
        tracker.record_visit("A");
        tracker.record_visit("B");

        let pattern = tracker.detect_pattern();
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap(), vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn test_pattern_detection_no_pattern() {
        let mut tracker = LoopTracker::new(5);

        tracker.record_visit("A");
        tracker.record_visit("B");
        tracker.record_visit("C");
        tracker.record_visit("D");

        let pattern = tracker.detect_pattern();
        assert!(pattern.is_none());
    }

    #[test]
    fn test_multiple_cycles() {
        // A -> B -> A and C -> D -> C
        let flow = make_flow(
            "multi-cycle",
            "A",
            vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "A"), ("alt", "C")]),
                make_movement("C", vec![("success", "D")]),
                make_movement("D", vec![("success", "C")]),
            ],
        );

        let analysis = analyze_flow(&flow).unwrap();

        assert!(analysis.has_cycles);
        // Should detect at least one cycle (may detect both)
        assert!(!analysis.cycle_paths.is_empty());
    }
}
