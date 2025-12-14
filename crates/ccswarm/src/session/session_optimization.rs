/// Session optimization module for reducing duplication and improving efficiency
///
/// This module provides consolidated patterns for session management.
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use crate::identity::AgentRole;

/// Simplified session trait that is object-safe
pub trait SimpleSession: Send + Sync {
    /// Get session statistics
    fn get_stats(&self) -> SessionStats;
    /// Check if session is alive
    fn is_alive(&self) -> bool;
}

/// Session statistics
#[derive(Debug, Default, Clone)]
pub struct SessionStats {
    pub is_active: bool,
    pub commands_executed: usize,
    pub last_activity: Option<std::time::Instant>,
}
use crate::utils::async_error_boundary::{AsyncCircuitBreaker, with_retry};

/// Unified session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedSessionConfig {
    /// Maximum sessions per role
    pub max_sessions_per_role: usize,
    /// Session idle timeout
    pub idle_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Enable session compression
    pub enable_compression: bool,
    /// Compression threshold (0.0-1.0)
    pub compression_threshold: f64,
    /// Session reuse strategy
    pub reuse_strategy: ReuseStrategy,
    /// Performance optimization settings
    pub performance_settings: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReuseStrategy {
    /// Always reuse existing sessions
    Aggressive,
    /// Reuse based on load threshold
    LoadBased { threshold: f64 },
    /// Time-based reuse
    TimeBased { max_age: Duration },
    /// Hybrid approach
    Hybrid {
        load_threshold: f64,
        max_age: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable batch operations
    pub batch_operations: bool,
    /// Batch size
    pub batch_size: usize,
    /// Enable predictive scaling
    pub predictive_scaling: bool,
    /// Context caching
    pub context_caching: bool,
}

impl Default for OptimizedSessionConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_role: 5,
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            enable_compression: true,
            compression_threshold: 0.8,
            reuse_strategy: ReuseStrategy::Hybrid {
                load_threshold: 0.7,
                max_age: Duration::from_secs(3600),
            },
            performance_settings: PerformanceSettings {
                batch_operations: true,
                batch_size: 10,
                predictive_scaling: true,
                context_caching: true,
            },
        }
    }
}

/// Session lifecycle manager with optimization
pub struct OptimizedSessionManager {
    /// Configuration
    config: OptimizedSessionConfig,
    /// Session registry by role
    registry: Arc<RwLock<HashMap<String, Vec<Arc<OptimizedSession>>>>>,
    /// Circuit breakers per role
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<AsyncCircuitBreaker>>>>,
    /// Performance tracker
    performance_tracker: Arc<PerformanceTracker>,
    /// Session factory
    session_factory: Arc<dyn SessionFactory + Send + Sync>,
}

/// Optimized session wrapper
pub struct OptimizedSession {
    /// Unique session ID
    pub id: String,
    /// Associated role
    pub role: AgentRole,
    /// Underlying session
    pub session: Arc<Mutex<Box<dyn SimpleSession + Send>>>,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Performance metrics
    pub metrics: Arc<RwLock<SessionMetrics>>,
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub compression_enabled: bool,
    pub compression_ratio: f64,
    pub context_size: usize,
}

#[derive(Debug, Default)]
pub struct SessionMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_execution_time: Duration,
    pub average_response_time: Duration,
    pub token_savings: usize,
}

/// Factory trait for creating sessions
#[async_trait]
pub trait SessionFactory: Send + Sync {
    async fn create_session(&self, role: &AgentRole) -> Result<Box<dyn SimpleSession + Send>>;
    async fn validate_session(&self, session: &dyn SimpleSession) -> Result<bool>;
}

/// Performance tracking
pub struct PerformanceTracker {
    metrics: Arc<RwLock<GlobalMetrics>>,
    #[allow(dead_code)]
    predictions: Arc<RwLock<PredictionEngine>>,
}

#[derive(Debug, Default)]
struct GlobalMetrics {
    total_sessions_created: usize,
    total_sessions_reused: usize,
    #[allow(dead_code)]
    total_compression_savings: usize,
    #[allow(dead_code)]
    average_session_lifetime: Duration,
    #[allow(dead_code)]
    peak_concurrent_sessions: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
struct PredictionEngine {
    historical_load: Vec<(DateTime<Utc>, f64)>,
    predicted_load: HashMap<String, f64>,
}

impl OptimizedSessionManager {
    /// Create new optimized session manager
    pub fn new(
        config: OptimizedSessionConfig,
        session_factory: Arc<dyn SessionFactory + Send + Sync>,
    ) -> Self {
        Self {
            config,
            registry: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            performance_tracker: Arc::new(PerformanceTracker::new()),
            session_factory,
        }
    }

    /// Get or create optimized session
    pub async fn get_or_create_session(&self, role: &AgentRole) -> Result<Arc<OptimizedSession>> {
        let role_key = role.name().to_lowercase();

        // Try to reuse existing session
        if let Some(session) = self.find_reusable_session(&role_key).await? {
            self.performance_tracker.record_reuse().await;
            return Ok(session);
        }

        // Create new session with circuit breaker protection
        let _breaker = self.get_circuit_breaker(&role_key).await;

        // Create the session directly
        let session = self.create_optimized_session(role).await?;

        // Register the session
        self.register_session(&role_key, session.clone()).await?;
        self.performance_tracker.record_creation().await;

        Ok(session)
    }

    /// Find reusable session based on strategy
    async fn find_reusable_session(&self, role_key: &str) -> Result<Option<Arc<OptimizedSession>>> {
        let registry = self.registry.read().await;

        if let Some(sessions) = registry.get(role_key) {
            for session in sessions {
                if self.is_session_reusable(session).await? {
                    // Update last used time
                    // Note: metadata is cloned but not currently persisted back
                    // TODO: Persist metadata update to session registry
                    let _metadata = {
                        let mut m = session.metadata.clone();
                        m.last_used = Utc::now();
                        m
                    };

                    return Ok(Some(session.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Check if session is reusable based on strategy
    async fn is_session_reusable(&self, session: &OptimizedSession) -> Result<bool> {
        // Validate session is still alive
        let session_guard = session.session.lock().await;
        let is_valid = self
            .session_factory
            .validate_session(session_guard.as_ref())
            .await?;
        drop(session_guard);

        if !is_valid {
            return Ok(false);
        }

        // Apply reuse strategy
        match &self.config.reuse_strategy {
            ReuseStrategy::Aggressive => Ok(true),

            ReuseStrategy::LoadBased { threshold } => {
                let metrics = session.metrics.read().await;
                let current_load = self.calculate_session_load(&metrics);
                Ok(current_load < *threshold)
            }

            ReuseStrategy::TimeBased { max_age } => {
                let age = Utc::now().signed_duration_since(session.metadata.created_at);
                Ok(age.to_std().unwrap_or(Duration::MAX) < *max_age)
            }

            ReuseStrategy::Hybrid {
                load_threshold,
                max_age,
            } => {
                let metrics = session.metrics.read().await;
                let current_load = self.calculate_session_load(&metrics);
                let age = Utc::now().signed_duration_since(session.metadata.created_at);

                Ok(current_load < *load_threshold
                    && age.to_std().unwrap_or(Duration::MAX) < *max_age)
            }
        }
    }

    /// Calculate session load (0.0-1.0)
    fn calculate_session_load(&self, metrics: &SessionMetrics) -> f64 {
        // Simple load calculation based on recent operations
        let operation_rate = metrics.total_operations as f64 / 100.0;
        let failure_rate = if metrics.total_operations > 0 {
            metrics.failed_operations as f64 / metrics.total_operations as f64
        } else {
            0.0
        };

        (operation_rate + failure_rate * 2.0).min(1.0)
    }

    /// Create new optimized session
    async fn create_optimized_session(&self, role: &AgentRole) -> Result<Arc<OptimizedSession>> {
        // Create underlying session
        let role_clone = role.clone();
        let factory = self.session_factory.clone();
        let base_session = with_retry(
            move || {
                let factory = factory.clone();
                let role = role_clone.clone();
                Box::pin(async move {
                    factory
                        .create_session(&role)
                        .await
                        .map_err(|e| crate::error::CCSwarmError::session("unknown", e.to_string()))
                })
            },
            3,
        )
        .await?;

        // Create optimized wrapper
        let session = Arc::new(OptimizedSession {
            id: uuid::Uuid::new_v4().to_string(),
            role: role.clone(),
            session: Arc::new(Mutex::new(base_session)),
            metadata: SessionMetadata {
                created_at: Utc::now(),
                last_used: Utc::now(),
                compression_enabled: self.config.enable_compression,
                compression_ratio: 0.0,
                context_size: 0,
            },
            metrics: Arc::new(RwLock::new(SessionMetrics::default())),
        });

        // Initialize compression if enabled
        if self.config.enable_compression {
            self.initialize_compression(&session).await?;
        }

        Ok(session)
    }

    /// Initialize session compression
    async fn initialize_compression(&self, session: &OptimizedSession) -> Result<()> {
        // This would integrate with ai-session's compression features
        tracing::debug!("Initializing compression for session {}", session.id);
        Ok(())
    }

    /// Register session in the registry
    async fn register_session(&self, role_key: &str, session: Arc<OptimizedSession>) -> Result<()> {
        let mut registry = self.registry.write().await;

        let sessions = registry
            .entry(role_key.to_string())
            .or_insert_with(Vec::new);

        // Enforce max sessions per role
        if sessions.len() >= self.config.max_sessions_per_role {
            // Remove oldest or least used session
            sessions.sort_by_key(|s| s.metadata.last_used);
            if let Some(old_session) = sessions.first() {
                tracing::info!(
                    "Evicting old session {} for role {}",
                    old_session.id,
                    role_key
                );
                sessions.remove(0);
            }
        }

        sessions.push(session);
        Ok(())
    }

    /// Get or create circuit breaker for role
    async fn get_circuit_breaker(&self, role_key: &str) -> Arc<AsyncCircuitBreaker> {
        let mut breakers = self.circuit_breakers.write().await;

        breakers
            .entry(role_key.to_string())
            .or_insert_with(|| Arc::new(AsyncCircuitBreaker::new(3)))
            .clone()
    }

    /// Execute operation on session with optimization
    pub async fn execute_on_session<F, T>(
        &self,
        session: &OptimizedSession,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce(&dyn SimpleSession) -> futures::future::BoxFuture<'_, Result<T>> + Send,
        T: Send,
    {
        let start = Instant::now();

        // Lock session
        let session_guard = session.session.lock().await;

        // Execute operation
        let result = operation(session_guard.as_ref()).await;

        // Update metrics
        let execution_time = start.elapsed();
        let mut metrics = session.metrics.write().await;
        metrics.total_operations += 1;

        match &result {
            Ok(_) => metrics.successful_operations += 1,
            Err(_) => metrics.failed_operations += 1,
        }

        metrics.total_execution_time += execution_time;
        metrics.average_response_time =
            metrics.total_execution_time / metrics.total_operations as u32;

        result
    }

    /// Batch execute operations
    pub async fn batch_execute<F, T>(
        &self,
        role: &AgentRole,
        operations: Vec<F>,
    ) -> Result<Vec<Result<T>>>
    where
        F: FnOnce(&dyn SimpleSession) -> futures::future::BoxFuture<'_, Result<T>> + Send,
        T: Send,
    {
        if !self.config.performance_settings.batch_operations {
            // Execute individually if batching is disabled
            let mut results = Vec::new();
            for op in operations {
                let session = self.get_or_create_session(role).await?;
                results.push(self.execute_on_session(&session, op).await);
            }
            return Ok(results);
        }

        // Get session for batch
        let session = self.get_or_create_session(role).await?;

        // Execute in batches
        let mut results = Vec::new();
        let batch_size = self.config.performance_settings.batch_size;

        // Convert operations into an iterator to handle chunks properly
        let mut ops_iter = operations.into_iter();

        loop {
            let batch: Vec<_> = ops_iter.by_ref().take(batch_size).collect();
            if batch.is_empty() {
                break;
            }

            for op in batch {
                let session_clone = session.clone();
                results.push(self.execute_on_session(&session_clone, op).await);
            }
        }

        Ok(results)
    }

    /// Get session statistics
    pub async fn get_statistics(&self) -> SessionStatistics {
        let registry = self.registry.read().await;
        let mut stats = SessionStatistics::default();

        for (role, sessions) in registry.iter() {
            stats.sessions_by_role.insert(
                role.clone(),
                RoleSessionStats {
                    total_sessions: sessions.len(),
                    active_sessions: sessions
                        .iter()
                        .filter(|s| {
                            let age = Utc::now().signed_duration_since(s.metadata.last_used);
                            age.to_std().unwrap_or(Duration::MAX) < self.config.idle_timeout
                        })
                        .count(),
                    compression_ratio: sessions
                        .iter()
                        .map(|s| s.metadata.compression_ratio)
                        .sum::<f64>()
                        / sessions.len().max(1) as f64,
                },
            );

            stats.total_sessions += sessions.len();
        }

        stats
    }

    /// Cleanup idle sessions
    pub async fn cleanup_idle_sessions(&self) -> Result<usize> {
        let mut registry = self.registry.write().await;
        let mut cleaned = 0;

        for sessions in registry.values_mut() {
            let before = sessions.len();

            sessions.retain(|session| {
                let age = Utc::now().signed_duration_since(session.metadata.last_used);
                age.to_std().unwrap_or(Duration::MAX) < self.config.idle_timeout
            });

            cleaned += before - sessions.len();
        }

        tracing::info!("Cleaned up {} idle sessions", cleaned);
        Ok(cleaned)
    }
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(GlobalMetrics::default())),
            predictions: Arc::new(RwLock::new(PredictionEngine {
                historical_load: Vec::new(),
                predicted_load: HashMap::new(),
            })),
        }
    }

    async fn record_creation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_sessions_created += 1;
    }

    async fn record_reuse(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_sessions_reused += 1;
    }

    pub async fn get_reuse_ratio(&self) -> f64 {
        let metrics = self.metrics.read().await;
        let total = metrics.total_sessions_created + metrics.total_sessions_reused;
        if total > 0 {
            metrics.total_sessions_reused as f64 / total as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub sessions_by_role: HashMap<String, RoleSessionStats>,
}

#[derive(Debug, Serialize)]
pub struct RoleSessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub compression_ratio: f64,
}
