//! Cycle and loop detection for Piece/Movement workflows.
//!
//! Provides takt-style cycle detection to prevent infinite loops in movement routing:
//! - **Static analysis**: Detect structural cycles before execution
//! - **Runtime tracking**: Monitor movement visits during execution
//! - **Loop strategies**: Configurable breakout behaviors (abort, skip, force-advance)
//!
//! Example usage:
//! ```rust,no_run
//! use ccswarm::workflow::cycle::{CycleDetector, LoopStrategy};
//!
//! let detector = CycleDetector::new(LoopStrategy::AllowN(3));
//! // let analysis = detector.analyze_piece(&piece);
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, warn};

use super::piece::Piece;

/// Strategy for handling loops in workflow execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopStrategy {
    /// Abort execution when loop detected
    Abort,
    /// Skip the looping movement and continue
    Skip,
    /// Force advance to next movement (ignore rules)
    ForceAdvance,
    /// Allow up to N iterations before taking action
    AllowN(u32),
}

impl Default for LoopStrategy {
    fn default() -> Self {
        LoopStrategy::AllowN(3)
    }
}

/// Result of static cycle analysis on a piece
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleAnalysis {
    /// Whether the workflow contains any cycles
    pub has_cycles: bool,
    /// List of detected cycle paths (each path is a sequence of movement IDs)
    pub cycle_paths: Vec<Vec<String>>,
    /// Maximum depth of the workflow graph
    pub max_depth: usize,
    /// Movements that are part of cycles
    pub cyclic_movements: HashSet<String>,
}

/// Cycle detector analyzes piece workflows for cycles
pub struct CycleDetector {
    /// Loop handling strategy
    strategy: LoopStrategy,
    /// Maximum iterations per movement before triggering strategy
    max_iterations: u32,
}

impl CycleDetector {
    /// Create a new cycle detector with the given strategy
    pub fn new(strategy: LoopStrategy) -> Self {
        let max_iterations = match strategy {
            LoopStrategy::AllowN(n) => n,
            _ => 1,
        };
        Self {
            strategy,
            max_iterations,
        }
    }

    /// Perform static analysis on a piece to detect cycles in the movement graph
    pub fn analyze_piece(&self, piece: &Piece) -> Result<CycleAnalysis> {
        debug!("Starting static cycle analysis for piece '{}'", piece.name);

        // Build adjacency list for the movement graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for movement in &piece.movements {
            let edges: Vec<String> = movement.rules.iter().map(|r| r.next.clone()).collect();
            graph.insert(movement.id.clone(), edges);
        }

        // Detect cycles using DFS with path tracking
        let mut cycle_paths = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for movement in &piece.movements {
            if !visited.contains(&movement.id) {
                self.dfs_detect_cycles(
                    &movement.id,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycle_paths,
                )?;
            }
        }

        // Calculate max depth
        let max_depth = self.calculate_max_depth(piece)?;

        // Collect all movements that are part of cycles
        let mut cyclic_movements = HashSet::new();
        for cycle_path in &cycle_paths {
            for movement_id in cycle_path {
                cyclic_movements.insert(movement_id.clone());
            }
        }

        let has_cycles = !cycle_paths.is_empty();

        if has_cycles {
            warn!(
                "Piece '{}' contains {} cycle(s): max_depth={}, cyclic_movements={}",
                piece.name,
                cycle_paths.len(),
                max_depth,
                cyclic_movements.len()
            );
        } else {
            info!("Piece '{}' is acyclic: max_depth={}", piece.name, max_depth);
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
        &self,
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
                    self.dfs_detect_cycles(neighbor, graph, visited, rec_stack, path, cycle_paths)?;
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
    fn calculate_max_depth(&self, piece: &Piece) -> Result<usize> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for movement in &piece.movements {
            let edges: Vec<String> = movement.rules.iter().map(|r| r.next.clone()).collect();
            graph.insert(movement.id.clone(), edges);
        }

        let mut max_depth = 0;
        let mut queue = VecDeque::new();
        queue.push_back((piece.initial_movement.clone(), 0));

        let mut visited = HashSet::new();
        visited.insert(piece.initial_movement.clone());

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

        Ok(max_depth)
    }

    /// Create a runtime loop tracker for execution monitoring
    pub fn create_tracker(&self) -> LoopTracker {
        LoopTracker::new(self.strategy, self.max_iterations)
    }

    /// Get the configured strategy
    pub fn strategy(&self) -> LoopStrategy {
        self.strategy
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new(LoopStrategy::default())
    }
}

/// Runtime tracker for monitoring movement visits during execution
#[derive(Debug, Clone)]
pub struct LoopTracker {
    /// Visit count per movement ID
    visit_counts: HashMap<String, u32>,
    /// History of movement transitions (for pattern detection)
    transition_history: Vec<String>,
    /// Loop handling strategy
    strategy: LoopStrategy,
    /// Maximum iterations before triggering strategy
    max_iterations: u32,
}

impl LoopTracker {
    /// Create a new loop tracker
    pub fn new(strategy: LoopStrategy, max_iterations: u32) -> Self {
        Self {
            visit_counts: HashMap::new(),
            transition_history: Vec::new(),
            strategy,
            max_iterations,
        }
    }

    /// Record a visit to a movement, returns whether the loop threshold was exceeded
    pub fn record_visit(&mut self, movement_id: &str) -> bool {
        let count = self
            .visit_counts
            .entry(movement_id.to_string())
            .or_insert(0);
        *count += 1;

        self.transition_history.push(movement_id.to_string());

        let exceeded = *count > self.max_iterations;

        if exceeded {
            warn!(
                "Loop threshold exceeded for movement '{}': {} visits (max: {})",
                movement_id, count, self.max_iterations
            );
        }

        exceeded
    }

    /// Get the visit count for a movement
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

    /// Get the current strategy
    pub fn strategy(&self) -> LoopStrategy {
        self.strategy
    }

    /// Get the transition history
    pub fn history(&self) -> &[String] {
        &self.transition_history
    }

    /// Reset the tracker state
    pub fn reset(&mut self) {
        self.visit_counts.clear();
        self.transition_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::piece::{Movement, MovementPermission, MovementRule, RuleCondition};

    fn make_movement(id: &str, rules: Vec<(&str, &str)>) -> Movement {
        Movement {
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
                })
                .collect(),
            parallel: false,
            sub_movements: vec![],
            output_contract: None,
            timeout: None,
            max_retries: 0,
        }
    }

    #[test]
    fn test_simple_cycle_detection() {
        // A -> B -> A
        let piece = Piece {
            name: "simple-cycle".to_string(),
            description: "Simple A-B cycle".to_string(),
            max_movements: 10,
            initial_movement: "A".to_string(),
            movements: vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "A")]),
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        };

        let detector = CycleDetector::default();
        let analysis = detector.analyze_piece(&piece).unwrap();

        assert!(analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 1);
        assert!(analysis.cyclic_movements.contains("A"));
        assert!(analysis.cyclic_movements.contains("B"));
    }

    #[test]
    fn test_complex_cycle_detection() {
        // A -> B -> C -> A
        let piece = Piece {
            name: "complex-cycle".to_string(),
            description: "A-B-C cycle".to_string(),
            max_movements: 10,
            initial_movement: "A".to_string(),
            movements: vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "C")]),
                make_movement("C", vec![("success", "A")]),
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        };

        let detector = CycleDetector::default();
        let analysis = detector.analyze_piece(&piece).unwrap();

        assert!(analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 1);
        assert_eq!(analysis.cyclic_movements.len(), 3);
    }

    #[test]
    fn test_no_cycle_linear_workflow() {
        // A -> B -> C (no cycles)
        let piece = Piece {
            name: "linear".to_string(),
            description: "Linear workflow".to_string(),
            max_movements: 10,
            initial_movement: "A".to_string(),
            movements: vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "C")]),
                make_movement("C", vec![]),
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        };

        let detector = CycleDetector::default();
        let analysis = detector.analyze_piece(&piece).unwrap();

        assert!(!analysis.has_cycles);
        assert_eq!(analysis.cycle_paths.len(), 0);
        assert_eq!(analysis.cyclic_movements.len(), 0);
        assert_eq!(analysis.max_depth, 2);
    }

    #[test]
    fn test_runtime_loop_tracking() {
        let mut tracker = LoopTracker::new(LoopStrategy::AllowN(3), 3);

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
        let mut tracker = LoopTracker::new(LoopStrategy::AllowN(5), 5);

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
        let mut tracker = LoopTracker::new(LoopStrategy::AllowN(5), 5);

        tracker.record_visit("A");
        tracker.record_visit("B");
        tracker.record_visit("C");
        tracker.record_visit("D");

        let pattern = tracker.detect_pattern();
        assert!(pattern.is_none());
    }

    #[test]
    fn test_max_iterations_enforcement() {
        let detector = CycleDetector::new(LoopStrategy::AllowN(2));
        let mut tracker = detector.create_tracker();

        assert!(!tracker.record_visit("loop"));
        assert!(!tracker.record_visit("loop"));
        assert!(tracker.record_visit("loop")); // Exceeds max_iterations=2
    }

    #[test]
    fn test_loop_strategy_abort() {
        let detector = CycleDetector::new(LoopStrategy::Abort);
        assert_eq!(detector.strategy(), LoopStrategy::Abort);

        let tracker = detector.create_tracker();
        assert_eq!(tracker.strategy(), LoopStrategy::Abort);
    }

    #[test]
    fn test_loop_strategy_skip() {
        let detector = CycleDetector::new(LoopStrategy::Skip);
        assert_eq!(detector.strategy(), LoopStrategy::Skip);
    }

    #[test]
    fn test_loop_strategy_force_advance() {
        let detector = CycleDetector::new(LoopStrategy::ForceAdvance);
        assert_eq!(detector.strategy(), LoopStrategy::ForceAdvance);
    }

    #[test]
    fn test_tracker_reset() {
        let mut tracker = LoopTracker::new(LoopStrategy::AllowN(3), 3);

        tracker.record_visit("A");
        tracker.record_visit("B");
        assert_eq!(tracker.visit_count("A"), 1);
        assert_eq!(tracker.history().len(), 2);

        tracker.reset();
        assert_eq!(tracker.visit_count("A"), 0);
        assert_eq!(tracker.history().len(), 0);
    }

    #[test]
    fn test_multiple_cycles() {
        // A -> B -> A and C -> D -> C
        let piece = Piece {
            name: "multi-cycle".to_string(),
            description: "Multiple independent cycles".to_string(),
            max_movements: 20,
            initial_movement: "A".to_string(),
            movements: vec![
                make_movement("A", vec![("success", "B")]),
                make_movement("B", vec![("success", "A"), ("alt", "C")]),
                make_movement("C", vec![("success", "D")]),
                make_movement("D", vec![("success", "C")]),
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        };

        let detector = CycleDetector::default();
        let analysis = detector.analyze_piece(&piece).unwrap();

        assert!(analysis.has_cycles);
        // Should detect at least one cycle (may detect both)
        assert!(!analysis.cycle_paths.is_empty());
    }
}
