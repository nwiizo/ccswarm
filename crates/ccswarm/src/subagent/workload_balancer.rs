//! Workload Balancer for Subagent Distribution
//!
//! Provides intelligent distribution of tasks across available subagents
//! based on capabilities, load, and priority.

use super::{SubagentError, SubagentResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Strategy for balancing workload
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least loaded agent first
    #[default]
    LeastLoaded,
    /// Random selection
    Random,
    /// Capability-based matching
    CapabilityMatch,
    /// Priority-weighted selection
    PriorityWeighted,
    /// Sticky sessions (same agent for related tasks)
    Sticky,
}

/// Configuration for the workload balancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancerConfig {
    /// Primary balancing strategy
    pub strategy: BalancingStrategy,
    /// Fallback strategy if primary fails
    pub fallback_strategy: BalancingStrategy,
    /// Maximum load per agent (number of tasks)
    pub max_load_per_agent: usize,
    /// Enable capability matching
    pub enable_capability_matching: bool,
    /// Weight for recency (0.0-1.0)
    pub recency_weight: f64,
    /// Weight for success rate (0.0-1.0)
    pub success_weight: f64,
}

impl Default for BalancerConfig {
    fn default() -> Self {
        Self {
            strategy: BalancingStrategy::LeastLoaded,
            fallback_strategy: BalancingStrategy::RoundRobin,
            max_load_per_agent: 5,
            enable_capability_matching: true,
            recency_weight: 0.3,
            success_weight: 0.5,
        }
    }
}

/// Statistics about an agent's workload
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentWorkloadStats {
    /// Current number of active tasks
    pub active_tasks: usize,
    /// Total tasks completed
    pub total_completed: usize,
    /// Successful tasks
    pub successful_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Average task duration (ms)
    pub avg_duration_ms: u64,
    /// Last task completion time
    pub last_completion: Option<chrono::DateTime<chrono::Utc>>,
    /// Capabilities this agent has demonstrated
    pub demonstrated_capabilities: Vec<String>,
}

impl AgentWorkloadStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_completed == 0 {
            return 1.0; // Assume perfect if no history
        }
        self.successful_tasks as f64 / self.total_completed as f64
    }

    /// Calculate a score for this agent (higher = better choice)
    pub fn score(&self, config: &BalancerConfig) -> f64 {
        let mut score = 1.0;

        // Lower score for higher load
        score -= (self.active_tasks as f64 / config.max_load_per_agent as f64).min(1.0) * 0.4;

        // Higher score for better success rate
        score += self.success_rate() * config.success_weight;

        // Slight bonus for recently active (warmed up)
        if let Some(last) = self.last_completion {
            let age = chrono::Utc::now() - last;
            if age.num_minutes() < 5 {
                score += 0.1 * config.recency_weight;
            }
        }

        score.max(0.0)
    }

    /// Update stats after task completion
    pub fn record_completion(&mut self, success: bool, duration_ms: u64) {
        self.total_completed += 1;
        if success {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
        }

        // Update average duration
        let total_duration = self.avg_duration_ms * (self.total_completed - 1) as u64;
        self.avg_duration_ms = (total_duration + duration_ms) / self.total_completed as u64;

        self.last_completion = Some(chrono::Utc::now());
        self.active_tasks = self.active_tasks.saturating_sub(1);
    }
}

/// Assignment of a task to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Task ID
    pub task_id: String,
    /// Assigned agent instance ID
    pub agent_id: String,
    /// Assignment timestamp
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    /// Priority of the task
    pub priority: i32,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Workload balancer for distributing tasks
pub struct WorkloadBalancer {
    /// Configuration
    config: BalancerConfig,
    /// Agent workload statistics
    stats: Arc<RwLock<HashMap<String, AgentWorkloadStats>>>,
    /// Current assignments
    assignments: Arc<RwLock<HashMap<String, TaskAssignment>>>,
    /// Round-robin counter
    rr_counter: Arc<RwLock<usize>>,
    /// Sticky session mappings (context_key -> agent_id)
    sticky_sessions: Arc<RwLock<HashMap<String, String>>>,
}

impl WorkloadBalancer {
    /// Create a new workload balancer
    pub fn new(config: BalancerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
            rr_counter: Arc::new(RwLock::new(0)),
            sticky_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BalancerConfig::default())
    }

    /// Register an agent for load balancing
    pub async fn register_agent(&self, agent_id: &str) {
        let mut stats = self.stats.write().await;
        stats
            .entry(agent_id.to_string())
            .or_insert_with(AgentWorkloadStats::default);
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut stats = self.stats.write().await;
        stats.remove(agent_id);

        // Remove any sticky sessions for this agent
        let mut sticky = self.sticky_sessions.write().await;
        sticky.retain(|_, v| v != agent_id);
    }

    /// Select the best agent for a task
    pub async fn select_agent(
        &self,
        available_agents: &[String],
        required_capabilities: &[String],
        sticky_key: Option<&str>,
        priority: i32,
    ) -> SubagentResult<String> {
        if available_agents.is_empty() {
            return Err(SubagentError::Validation("No available agents".to_string()));
        }

        // Check sticky session first
        if let Some(key) = sticky_key {
            if self.config.strategy == BalancingStrategy::Sticky {
                let sticky = self.sticky_sessions.read().await;
                if let Some(agent_id) = sticky.get(key) {
                    if available_agents.contains(agent_id) {
                        return Ok(agent_id.clone());
                    }
                }
            }
        }

        let selected = match self.config.strategy {
            BalancingStrategy::RoundRobin => self.select_round_robin(available_agents).await,
            BalancingStrategy::LeastLoaded => self.select_least_loaded(available_agents).await,
            BalancingStrategy::Random => self.select_random(available_agents),
            BalancingStrategy::CapabilityMatch => {
                self.select_by_capability(available_agents, required_capabilities)
                    .await
            }
            BalancingStrategy::PriorityWeighted => {
                self.select_priority_weighted(available_agents, priority)
                    .await
            }
            BalancingStrategy::Sticky => {
                // Fallback to least loaded if no sticky session
                self.select_least_loaded(available_agents).await
            }
        };

        let agent_id = selected
            .or_else(|| {
                // Try fallback strategy
                match self.config.fallback_strategy {
                    BalancingStrategy::RoundRobin => available_agents.first().cloned(),
                    _ => available_agents.first().cloned(),
                }
            })
            .ok_or_else(|| SubagentError::Validation("Failed to select agent".to_string()))?;

        // Update sticky session if needed
        if let Some(key) = sticky_key {
            let mut sticky = self.sticky_sessions.write().await;
            sticky.insert(key.to_string(), agent_id.clone());
        }

        Ok(agent_id)
    }

    /// Round-robin selection
    async fn select_round_robin(&self, agents: &[String]) -> Option<String> {
        let mut counter = self.rr_counter.write().await;
        let idx = *counter % agents.len();
        *counter = counter.wrapping_add(1);
        agents.get(idx).cloned()
    }

    /// Select least loaded agent
    async fn select_least_loaded(&self, agents: &[String]) -> Option<String> {
        let stats = self.stats.read().await;

        agents
            .iter()
            .map(|id| {
                let load = stats.get(id).map(|s| s.active_tasks).unwrap_or(0);
                (id, load)
            })
            .min_by_key(|(_, load)| *load)
            .map(|(id, _)| id.clone())
    }

    /// Random selection
    fn select_random(&self, agents: &[String]) -> Option<String> {
        use rand::Rng;
        if agents.is_empty() {
            return None;
        }
        let idx = rand::rng().random_range(0..agents.len());
        agents.get(idx).cloned()
    }

    /// Select by capability match
    async fn select_by_capability(&self, agents: &[String], required: &[String]) -> Option<String> {
        if required.is_empty() {
            return self.select_least_loaded(agents).await;
        }

        let stats = self.stats.read().await;

        // Find agents with matching capabilities
        let matching: Vec<_> = agents
            .iter()
            .filter(|id| {
                if let Some(s) = stats.get(*id) {
                    required
                        .iter()
                        .all(|cap| s.demonstrated_capabilities.contains(cap))
                } else {
                    false
                }
            })
            .collect();

        if !matching.is_empty() {
            // Among matching, select least loaded
            matching
                .iter()
                .map(|id| {
                    let load = stats.get(*id).map(|s| s.active_tasks).unwrap_or(0);
                    (*id, load)
                })
                .min_by_key(|(_, load)| *load)
                .map(|(id, _)| (*id).clone())
        } else {
            // Fall back to least loaded
            drop(stats);
            self.select_least_loaded(agents).await
        }
    }

    /// Priority-weighted selection (higher priority tasks get better agents)
    async fn select_priority_weighted(&self, agents: &[String], priority: i32) -> Option<String> {
        let stats = self.stats.read().await;

        // Score each agent
        let mut scored: Vec<_> = agents
            .iter()
            .map(|id| {
                let base_score = stats.get(id).map(|s| s.score(&self.config)).unwrap_or(0.5);

                // Higher priority tasks get bonus for better agents
                let priority_factor = 1.0 + (priority as f64 / 100.0);
                (id, base_score * priority_factor)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.first().map(|(id, _)| (*id).clone())
    }

    /// Record task assignment
    pub async fn assign_task(
        &self,
        task_id: &str,
        agent_id: &str,
        priority: i32,
        capabilities: Vec<String>,
    ) {
        let assignment = TaskAssignment {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            assigned_at: chrono::Utc::now(),
            priority,
            required_capabilities: capabilities,
        };

        let mut assignments = self.assignments.write().await;
        assignments.insert(task_id.to_string(), assignment);

        // Update agent stats
        let mut stats = self.stats.write().await;
        if let Some(s) = stats.get_mut(agent_id) {
            s.active_tasks += 1;
        }
    }

    /// Record task completion
    pub async fn complete_task(&self, task_id: &str, success: bool, duration_ms: u64) {
        let assignment = {
            let mut assignments = self.assignments.write().await;
            assignments.remove(task_id)
        };

        if let Some(a) = assignment {
            let mut stats = self.stats.write().await;
            if let Some(s) = stats.get_mut(&a.agent_id) {
                s.record_completion(success, duration_ms);

                // Add demonstrated capabilities on success
                if success {
                    for cap in a.required_capabilities {
                        if !s.demonstrated_capabilities.contains(&cap) {
                            s.demonstrated_capabilities.push(cap);
                        }
                    }
                }
            }
        }
    }

    /// Get agent statistics
    pub async fn get_agent_stats(&self, agent_id: &str) -> Option<AgentWorkloadStats> {
        let stats = self.stats.read().await;
        stats.get(agent_id).cloned()
    }

    /// Get all statistics
    pub async fn get_all_stats(&self) -> HashMap<String, AgentWorkloadStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get current assignments
    pub async fn get_assignments(&self) -> Vec<TaskAssignment> {
        let assignments = self.assignments.read().await;
        assignments.values().cloned().collect()
    }
}

impl Clone for WorkloadBalancer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            assignments: Arc::clone(&self.assignments),
            rr_counter: Arc::clone(&self.rr_counter),
            sticky_sessions: Arc::clone(&self.sticky_sessions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_stats_score() {
        let config = BalancerConfig::default();
        let stats = AgentWorkloadStats {
            active_tasks: 1,
            total_completed: 10,
            successful_tasks: 9,
            failed_tasks: 1,
            avg_duration_ms: 1000,
            last_completion: Some(chrono::Utc::now()),
            demonstrated_capabilities: vec!["rust".to_string()],
        };

        let score = stats.score(&config);
        assert!(score > 0.5); // Should be above average with 90% success rate
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let balancer = WorkloadBalancer::with_defaults();
        let agents = vec!["a1".to_string(), "a2".to_string(), "a3".to_string()];

        let first = balancer.select_round_robin(&agents).await;
        let second = balancer.select_round_robin(&agents).await;
        let third = balancer.select_round_robin(&agents).await;
        let fourth = balancer.select_round_robin(&agents).await;

        assert_eq!(first, Some("a1".to_string()));
        assert_eq!(second, Some("a2".to_string()));
        assert_eq!(third, Some("a3".to_string()));
        assert_eq!(fourth, Some("a1".to_string())); // Wraps around
    }

    #[tokio::test]
    async fn test_least_loaded_selection() {
        let balancer = WorkloadBalancer::with_defaults();

        // Register agents with different loads
        balancer.register_agent("a1").await;
        balancer.register_agent("a2").await;

        // Assign tasks to a1
        balancer.assign_task("t1", "a1", 0, vec![]).await;
        balancer.assign_task("t2", "a1", 0, vec![]).await;

        let agents = vec!["a1".to_string(), "a2".to_string()];
        let selected = balancer.select_least_loaded(&agents).await;

        assert_eq!(selected, Some("a2".to_string())); // a2 has no tasks
    }
}
