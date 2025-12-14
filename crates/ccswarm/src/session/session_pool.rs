/// Session pooling and advanced management for persistent Claude Code agents
///
/// This module provides sophisticated session pooling, load balancing, and
/// resource management for optimal token efficiency and performance.
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::interval;
use uuid::Uuid;

use crate::agent::persistent::PersistentClaudeAgent;
use crate::agent::{Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::identity::AgentRole;
use crate::session::worktree_session::{WorktreeSessionConfig, WorktreeSessionManager};

/// Session pool entry with metadata
#[derive(Debug, Clone)]
pub struct PooledSession {
    /// The persistent agent
    pub agent: Arc<Mutex<PersistentClaudeAgent>>,

    /// Pool metadata
    pub metadata: SessionPoolMetadata,

    /// Current usage statistics
    pub stats: SessionUsageStats,

    /// Last health check timestamp
    pub last_health_check: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPoolMetadata {
    pub pool_id: String,
    pub agent_id: String,
    pub role: AgentRole,
    pub created_at: DateTime<Utc>,
    pub pool_generation: u64,
    pub priority_score: f64,
    pub max_concurrent_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUsageStats {
    pub total_tasks_executed: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub current_load: f64,
    pub peak_load: f64,
    pub uptime: Duration,
    pub last_activity: DateTime<Utc>,
}

impl Default for SessionUsageStats {
    fn default() -> Self {
        Self {
            total_tasks_executed: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            total_execution_time: Duration::ZERO,
            average_execution_time: Duration::ZERO,
            current_load: 0.0,
            peak_load: 0.0,
            uptime: Duration::ZERO,
            last_activity: Utc::now(),
        }
    }
}

/// Session pool configuration
#[derive(Debug, Clone)]
pub struct SessionPoolConfig {
    /// Minimum sessions to keep warm per role
    pub min_sessions_per_role: usize,

    /// Maximum sessions allowed per role
    pub max_sessions_per_role: usize,

    /// Maximum concurrent tasks per session
    pub max_concurrent_tasks_per_session: usize,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Session warmup strategy
    pub warmup_strategy: WarmupStrategy,

    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,

    /// Auto-scaling configuration
    pub auto_scaling: AutoScalingConfig,

    /// Performance monitoring
    pub enable_performance_monitoring: bool,
}

#[derive(Debug, Clone)]
pub enum WarmupStrategy {
    Lazy,       // Create sessions on demand
    Eager,      // Pre-create minimum sessions
    Predictive, // Create sessions based on usage patterns
}

#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,     // Simple round-robin
    LeastLoaded,    // Route to least loaded session
    WeightedRandom, // Random with priority weights
    Adaptive,       // Adapt based on performance metrics
}

#[derive(Debug, Clone)]
pub struct AutoScalingConfig {
    pub enabled: bool,
    pub scale_up_threshold: f64,   // Load threshold to scale up
    pub scale_down_threshold: f64, // Load threshold to scale down
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub target_load: f64,
}

impl Default for SessionPoolConfig {
    fn default() -> Self {
        Self {
            min_sessions_per_role: 1,
            max_sessions_per_role: 5,
            max_concurrent_tasks_per_session: 3,
            health_check_interval: Duration::from_secs(30),
            warmup_strategy: WarmupStrategy::Lazy,
            load_balancing: LoadBalancingStrategy::LeastLoaded,
            auto_scaling: AutoScalingConfig {
                enabled: true,
                scale_up_threshold: 0.8,
                scale_down_threshold: 0.3,
                scale_up_cooldown: Duration::from_secs(60),
                scale_down_cooldown: Duration::from_secs(300),
                target_load: 0.6,
            },
            enable_performance_monitoring: true,
        }
    }
}

/// Advanced session pool manager
#[derive(Debug)]
pub struct SessionPool {
    /// Session pools by role
    pools: Arc<RwLock<HashMap<String, Vec<PooledSession>>>>,

    /// Underlying worktree session manager
    worktree_manager: Arc<Mutex<WorktreeSessionManager>>,

    /// Pool configuration
    config: SessionPoolConfig,

    /// Load balancer state
    load_balancer_state: Arc<RwLock<HashMap<String, LoadBalancerState>>>,

    /// Performance metrics
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    /// Background task handles
    background_tasks: Vec<tokio::task::JoinHandle<()>>,

    /// Session creation semaphore (prevents overwhelming)
    creation_semaphore: Arc<Semaphore>,

    /// Auto-scaling state
    scaling_state: Arc<RwLock<HashMap<String, ScalingState>>>,
}

#[derive(Debug, Clone)]
struct LoadBalancerState {
    last_selected: usize,
    #[allow(dead_code)] // Will be used for advanced load balancing
    selection_weights: Vec<f64>,
    #[allow(dead_code)] // Will be used for performance-based routing
    performance_history: VecDeque<f64>,
}

#[derive(Debug, Default)]
struct PerformanceMetrics {
    total_tasks_processed: usize,
    total_execution_time: Duration,
    #[allow(dead_code)] // Will be used for utilization tracking
    session_utilization: HashMap<String, f64>,
    #[allow(dead_code)] // Will be used for error rate monitoring
    error_rates: HashMap<String, f64>,
    #[allow(dead_code)] // Will be used for throughput analysis
    throughput_history: VecDeque<(DateTime<Utc>, usize)>,
}

#[derive(Debug, Clone)]
struct ScalingState {
    #[allow(dead_code)] // Will be used for auto-scaling logic
    last_scale_up: Option<DateTime<Utc>>,
    #[allow(dead_code)] // Will be used for auto-scaling logic
    last_scale_down: Option<DateTime<Utc>>,
    #[allow(dead_code)] // Will be used for scaling operation tracking
    pending_scale_operations: usize,
}

impl SessionPool {
    /// Create a new session pool
    pub async fn new(
        worktree_config: WorktreeSessionConfig,
        pool_config: SessionPoolConfig,
    ) -> Result<Self> {
        let mut worktree_manager = WorktreeSessionManager::new(worktree_config)?;
        worktree_manager.start().await?;

        let pool = Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            worktree_manager: Arc::new(Mutex::new(worktree_manager)),
            config: pool_config.clone(),
            load_balancer_state: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            background_tasks: Vec::new(),
            creation_semaphore: Arc::new(Semaphore::new(pool_config.max_sessions_per_role * 4)),
            scaling_state: Arc::new(RwLock::new(HashMap::new())),
        };

        Ok(pool)
    }

    /// Start the session pool with background tasks
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting session pool");

        // Start health check task
        let health_check_task = self.start_health_check_task().await;
        self.background_tasks.push(health_check_task);

        // Start auto-scaling task if enabled
        if self.config.auto_scaling.enabled {
            let scaling_task = self.start_auto_scaling_task().await;
            self.background_tasks.push(scaling_task);
        }

        // Start performance monitoring task
        if self.config.enable_performance_monitoring {
            let monitoring_task = self.start_performance_monitoring_task().await;
            self.background_tasks.push(monitoring_task);
        }

        // Pre-warm sessions if using eager strategy
        if matches!(self.config.warmup_strategy, WarmupStrategy::Eager) {
            self.warmup_essential_sessions().await?;
        }

        tracing::info!("Session pool started successfully");
        Ok(())
    }

    /// Execute a task using the best available session
    pub async fn execute_task(
        &self,
        role: AgentRole,
        task: Task,
        claude_config: ClaudeConfig,
    ) -> Result<TaskResult> {
        let start_time = Instant::now();

        // Get optimal session for the task
        let session = self.get_optimal_session(&role, &claude_config).await?;

        // Execute task
        let result = {
            let mut agent = session.agent.lock().await;
            agent.execute_task(task.clone()).await?
        };

        // Update statistics
        self.update_session_stats(&session, &result, start_time.elapsed())
            .await;

        Ok(result)
    }

    /// Execute multiple tasks in batch with optimal session selection
    pub async fn execute_task_batch(
        &self,
        role: AgentRole,
        tasks: Vec<Task>,
        claude_config: ClaudeConfig,
    ) -> Result<Vec<TaskResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();

        // For batch operations, prefer a single session for maximum efficiency
        let session = self.get_optimal_session(&role, &claude_config).await?;

        // Execute batch
        let results = {
            let mut agent = session.agent.lock().await;
            agent.execute_task_batch(tasks).await?
        };

        // Update statistics for batch
        for result in &results {
            self.update_session_stats(
                &session,
                result,
                start_time.elapsed() / results.len() as u32,
            )
            .await;
        }

        tracing::info!("Batch of {} tasks completed in session pool", results.len());
        Ok(results)
    }

    /// Get optimal session for a role using load balancing strategy
    async fn get_optimal_session(
        &self,
        role: &AgentRole,
        claude_config: &ClaudeConfig,
    ) -> Result<Arc<PooledSession>> {
        let role_key = role.name().to_lowercase();

        // Check if we have available sessions
        let available_session = self.find_available_session(&role_key).await;

        if let Some(session) = available_session {
            return Ok(session);
        }

        // Need to create or wait for a session
        self.ensure_session_available(&role_key, role.clone(), claude_config.clone())
            .await
    }

    /// Find an available session using load balancing strategy
    async fn find_available_session(&self, role_key: &str) -> Option<Arc<PooledSession>> {
        let pools = self.pools.read().await;
        let sessions = pools.get(role_key)?;

        if sessions.is_empty() {
            return None;
        }

        match self.config.load_balancing {
            LoadBalancingStrategy::LeastLoaded => {
                // Find session with lowest current load
                sessions
                    .iter()
                    .filter(|s| s.stats.current_load < 1.0) // Not at max capacity
                    .min_by(|a, b| {
                        a.stats
                            .current_load
                            .partial_cmp(&b.stats.current_load)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|s| Arc::new((*s).clone()))
            }
            LoadBalancingStrategy::RoundRobin => {
                // Simple round-robin selection
                let mut lb_state = self.load_balancer_state.write().await;
                let state = lb_state
                    .entry(role_key.to_string())
                    .or_insert(LoadBalancerState {
                        last_selected: 0,
                        selection_weights: vec![1.0; sessions.len()],
                        performance_history: VecDeque::new(),
                    });

                let available_sessions: Vec<_> = sessions
                    .iter()
                    .filter(|s| s.stats.current_load < 1.0)
                    .collect();

                if available_sessions.is_empty() {
                    return None;
                }

                let selected_idx = state.last_selected % available_sessions.len();
                state.last_selected = (state.last_selected + 1) % available_sessions.len();

                Some(Arc::new((*available_sessions[selected_idx]).clone()))
            }
            LoadBalancingStrategy::Adaptive => {
                // Choose based on performance metrics
                sessions
                    .iter()
                    .filter(|s| s.stats.current_load < 1.0)
                    .max_by(|a, b| {
                        let score_a = self.calculate_session_score(a);
                        let score_b = self.calculate_session_score(b);
                        score_a
                            .partial_cmp(&score_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|s| Arc::new((*s).clone()))
            }
            LoadBalancingStrategy::WeightedRandom => {
                // Weighted random selection based on performance
                let available_sessions: Vec<_> = sessions
                    .iter()
                    .filter(|s| s.stats.current_load < 1.0)
                    .collect();

                if available_sessions.is_empty() {
                    return None;
                }

                // For simplicity, use round-robin for now
                // In real implementation, would use weighted random
                available_sessions.first().map(|s| Arc::new((**s).clone()))
            }
        }
    }

    /// Calculate performance score for session selection
    fn calculate_session_score(&self, session: &PooledSession) -> f64 {
        let load_factor = 1.0 - session.stats.current_load;
        let success_rate = if session.stats.total_tasks_executed > 0 {
            session.stats.successful_tasks as f64 / session.stats.total_tasks_executed as f64
        } else {
            1.0
        };
        let speed_factor = if session.stats.average_execution_time.as_millis() > 0 {
            1000.0 / session.stats.average_execution_time.as_millis() as f64
        } else {
            1.0
        };

        // Weighted score combining load, success rate, and speed
        (load_factor * 0.4) + (success_rate * 0.4) + (speed_factor.min(2.0) * 0.2)
    }

    /// Ensure a session is available for the role
    async fn ensure_session_available(
        &self,
        role_key: &str,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<PooledSession>> {
        // Check if we're at max capacity
        let current_count = {
            let pools = self.pools.read().await;
            pools.get(role_key).map(|p| p.len()).unwrap_or(0)
        };

        if current_count >= self.config.max_sessions_per_role {
            // Wait for an available session or fail
            tokio::time::timeout(
                Duration::from_secs(30),
                self.wait_for_available_session(role_key),
            )
            .await?
        } else {
            // Create a new session
            self.create_new_pooled_session(role_key, role, claude_config)
                .await
        }
    }

    /// Wait for an available session
    async fn wait_for_available_session(&self, role_key: &str) -> Result<Arc<PooledSession>> {
        let mut interval = interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            if let Some(session) = self.find_available_session(role_key).await {
                return Ok(session);
            }
        }
    }

    /// Create a new pooled session
    async fn create_new_pooled_session(
        &self,
        role_key: &str,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<PooledSession>> {
        // Acquire semaphore to limit concurrent creation
        let _permit = self.creation_semaphore.acquire().await?;

        tracing::info!("Creating new pooled session for role: {}", role_key);

        // Create session through worktree manager
        let agent = {
            let worktree_manager = self.worktree_manager.lock().await;
            worktree_manager
                .get_or_create_worktree_session(role.clone(), claude_config.clone())
                .await?
        };

        // Create pool metadata
        let metadata = SessionPoolMetadata {
            pool_id: Uuid::new_v4().to_string(),
            agent_id: {
                let agent_guard = agent.lock().await;
                agent_guard.identity.agent_id.clone()
            },
            role: role.clone(),
            created_at: Utc::now(),
            pool_generation: 1,
            priority_score: 1.0,
            max_concurrent_tasks: self.config.max_concurrent_tasks_per_session,
        };

        let pooled_session = PooledSession {
            agent,
            metadata,
            stats: SessionUsageStats::default(),
            last_health_check: Instant::now(),
        };

        let pooled_session = Arc::new(pooled_session);

        // Add to pool
        {
            let mut pools = self.pools.write().await;
            pools
                .entry(role_key.to_string())
                .or_insert_with(Vec::new)
                .push(pooled_session.as_ref().clone());
        }

        tracing::info!(
            "Created pooled session successfully: {}",
            pooled_session.metadata.agent_id
        );
        Ok(pooled_session)
    }

    /// Update session statistics after task execution
    async fn update_session_stats(
        &self,
        session: &Arc<PooledSession>,
        result: &TaskResult,
        execution_time: Duration,
    ) {
        // Note: In a real implementation, we would need mutable access to session stats
        // This is simplified for demonstration purposes
        tracing::debug!(
            "Task completed in session {}: success={}, duration={:?}",
            session.metadata.agent_id,
            result.success,
            execution_time
        );

        // Update global performance metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_tasks_processed += 1;
        metrics.total_execution_time += execution_time;
    }

    /// Start health check background task
    async fn start_health_check_task(&self) -> tokio::task::JoinHandle<()> {
        let pools = Arc::clone(&self.pools);
        let interval_duration = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Perform health checks on all sessions
                let pools_guard = pools.read().await;
                for (role_key, sessions) in pools_guard.iter() {
                    for session in sessions {
                        // Check session health
                        if let Err(e) = Self::health_check_session(session).await {
                            tracing::warn!(
                                "Health check failed for session {} in role {}: {}",
                                session.metadata.agent_id,
                                role_key,
                                e
                            );
                        }
                    }
                }
            }
        })
    }

    /// Perform health check on a session
    async fn health_check_session(session: &PooledSession) -> Result<()> {
        let agent = session.agent.lock().await;
        let stats = agent.get_session_stats().await;

        if !stats.is_active {
            return Err(anyhow::anyhow!("Session is not active"));
        }

        // Additional health checks could be added here
        Ok(())
    }

    /// Start auto-scaling background task
    async fn start_auto_scaling_task(&self) -> tokio::task::JoinHandle<()> {
        let pools = Arc::clone(&self.pools);
        let _scaling_state = Arc::clone(&self.scaling_state);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Check each role for scaling needs
                let pools_guard = pools.read().await;
                for (role_key, sessions) in pools_guard.iter() {
                    let current_load = Self::calculate_role_load(sessions);

                    // Implement scaling logic based on load
                    if current_load > config.auto_scaling.scale_up_threshold {
                        tracing::info!(
                            "High load detected for role {}: {:.2}",
                            role_key,
                            current_load
                        );
                        // Scale up logic would go here
                    } else if current_load < config.auto_scaling.scale_down_threshold {
                        tracing::debug!(
                            "Low load detected for role {}: {:.2}",
                            role_key,
                            current_load
                        );
                        // Scale down logic would go here
                    }
                }
            }
        })
    }

    /// Calculate average load for a role
    fn calculate_role_load(sessions: &[PooledSession]) -> f64 {
        if sessions.is_empty() {
            return 0.0;
        }

        let total_load: f64 = sessions.iter().map(|s| s.stats.current_load).sum();
        total_load / sessions.len() as f64
    }

    /// Start performance monitoring task
    async fn start_performance_monitoring_task(&self) -> tokio::task::JoinHandle<()> {
        let performance_metrics = Arc::clone(&self.performance_metrics);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let metrics = performance_metrics.read().await;
                tracing::info!(
                    "Performance metrics - Total tasks: {}, Avg execution time: {:?}",
                    metrics.total_tasks_processed,
                    if metrics.total_tasks_processed > 0 {
                        metrics.total_execution_time / metrics.total_tasks_processed as u32
                    } else {
                        Duration::ZERO
                    }
                );
            }
        })
    }

    /// Warmup essential sessions
    async fn warmup_essential_sessions(&self) -> Result<()> {
        use crate::identity::{default_backend_role, default_frontend_role};

        tracing::info!("Warming up essential sessions");

        let essential_roles = vec![default_frontend_role(), default_backend_role()];

        for role in essential_roles {
            for _ in 0..self.config.min_sessions_per_role {
                let claude_config = ClaudeConfig::default();
                if let Err(e) = self
                    .create_new_pooled_session(
                        &role.name().to_lowercase(),
                        role.clone(),
                        claude_config,
                    )
                    .await
                {
                    tracing::warn!("Failed to warmup session for role {}: {}", role.name(), e);
                }
            }
        }

        Ok(())
    }

    /// Get comprehensive pool statistics
    pub async fn get_pool_statistics(&self) -> PoolStatistics {
        let pools = self.pools.read().await;
        let performance_metrics = self.performance_metrics.read().await;

        let mut role_stats = HashMap::new();
        let mut total_sessions = 0;
        let mut active_sessions = 0;

        for (role_key, sessions) in pools.iter() {
            total_sessions += sessions.len();
            let role_active = sessions
                .iter()
                .filter(|s| s.stats.current_load > 0.0)
                .count();
            active_sessions += role_active;

            role_stats.insert(
                role_key.clone(),
                RoleStatistics {
                    total_sessions: sessions.len(),
                    active_sessions: role_active,
                    average_load: Self::calculate_role_load(sessions),
                    total_tasks: sessions.iter().map(|s| s.stats.total_tasks_executed).sum(),
                },
            );
        }

        PoolStatistics {
            total_sessions,
            active_sessions,
            role_statistics: role_stats,
            global_performance: GlobalPerformanceStats {
                total_tasks_processed: performance_metrics.total_tasks_processed,
                total_execution_time: performance_metrics.total_execution_time,
                average_execution_time: if performance_metrics.total_tasks_processed > 0 {
                    performance_metrics.total_execution_time
                        / performance_metrics.total_tasks_processed as u32
                } else {
                    Duration::ZERO
                },
            },
        }
    }

    /// Shutdown the session pool
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down session pool");

        // Cancel background tasks
        for handle in self.background_tasks.drain(..) {
            handle.abort();
        }

        // Shutdown worktree manager
        {
            let mut worktree_manager = self.worktree_manager.lock().await;
            worktree_manager.shutdown().await?;
        }

        // Clear pools
        {
            let mut pools = self.pools.write().await;
            pools.clear();
        }

        tracing::info!("Session pool shutdown complete");
        Ok(())
    }
}

/// Comprehensive pool statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub role_statistics: HashMap<String, RoleStatistics>,
    pub global_performance: GlobalPerformanceStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub average_load: f64,
    pub total_tasks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPerformanceStats {
    pub total_tasks_processed: usize,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
}
