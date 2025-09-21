//! Common utilities to reduce code duplication across the codebase

/// Utilities for working with timestamps
pub mod time {
    use chrono::{DateTime, Utc};

    /// Get current UTC timestamp
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }

    /// Calculate age from a timestamp to now
    pub fn age_since(timestamp: DateTime<Utc>) -> chrono::Duration {
        now().signed_duration_since(timestamp)
    }

    /// Check if a timestamp is older than the given duration
    pub fn is_older_than(timestamp: DateTime<Utc>, duration: chrono::Duration) -> bool {
        age_since(timestamp) > duration
    }
}

/// Utilities for working with collections
pub mod collections {
    use std::collections::HashMap;

    /// Create a new HashMap with better ergonomics
    pub fn new_hashmap<K, V>() -> HashMap<K, V> {
        HashMap::new()
    }

    /// Create a HashMap with initial capacity
    pub fn with_capacity<K, V>(capacity: usize) -> HashMap<K, V> {
        HashMap::with_capacity(capacity)
    }
}

/// Async utilities for common patterns
pub mod async_utils {
    use anyhow::Result;
    use std::future::Future;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Common pattern for acquiring multiple locks and performing operations
    pub async fn with_locks_2<T, U, R, F, Fut>(
        lock1: &Arc<RwLock<T>>,
        lock2: &Arc<RwLock<U>>,
        operation: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut T, &mut U) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        let mut guard1 = lock1.write().await;
        let mut guard2 = lock2.write().await;
        operation(&mut *guard1, &mut *guard2).await
    }

    /// Common pattern for acquiring three locks and performing operations
    pub async fn with_locks_3<T, U, V, R, F, Fut>(
        lock1: &Arc<RwLock<T>>,
        lock2: &Arc<RwLock<U>>,
        lock3: &Arc<RwLock<V>>,
        operation: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut T, &mut U, &mut V) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        let mut guard1 = lock1.write().await;
        let mut guard2 = lock2.write().await;
        let mut guard3 = lock3.write().await;
        operation(&mut *guard1, &mut *guard2, &mut *guard3).await
    }

    /// Update a timestamped record with current time
    pub trait Timestamped {
        fn update_timestamp(&mut self);
    }

    /// Helper to update any timestamped entity
    pub fn update_with_timestamp<T: Timestamped>(mut entity: T) -> T {
        entity.update_timestamp();
        entity
    }
}

/// Error handling utilities
pub mod errors {
    use anyhow::anyhow;

    /// Common pattern for "not found" errors
    pub fn not_found_error(entity_type: &str, id: &str) -> anyhow::Error {
        anyhow!("{} not found: {}", entity_type, id)
    }

    /// Common pattern for validation errors
    pub fn validation_error(field: &str, reason: &str) -> anyhow::Error {
        anyhow!("Validation failed for {}: {}", field, reason)
    }

    /// Common pattern for state transition errors
    pub fn invalid_state_error(current_state: &str, expected_state: &str) -> anyhow::Error {
        anyhow!(
            "Invalid state transition: expected {}, but was {}",
            expected_state,
            current_state
        )
    }
}

/// Logging utilities for common patterns
pub mod logging {
    /// Log metrics collection failure
    pub fn log_metrics_failure(id: &str, error: &dyn std::fmt::Display) {
        log::warn!("Failed to collect metrics for {}: {}", id, error);
    }

    /// Log monitoring update failure
    pub fn log_monitoring_failure(error: &dyn std::fmt::Display) {
        log::error!("Monitoring update failed: {}", error);
    }

    /// Log subagent operation
    pub fn log_subagent_created(name: &str, instance_id: &str) {
        log::info!("Created subagent '{}' with instance ID: {}", name, instance_id);
    }

    /// Log task delegation
    pub fn log_task_delegated(agent_id: &str, task: &str) {
        log::info!("Delegated task to {}: {}", agent_id, task);
    }

    /// Log initialization failure
    pub fn log_init_failure(component: &str, id: &str, error: &dyn std::fmt::Display) {
        log::error!("Failed to initialize {} {}: {}", component, id, error);
    }

    /// Log parsing warning
    pub fn log_parse_warning(file_path: &str, error: &dyn std::fmt::Display) {
        log::warn!("Failed to parse file {}: {}", file_path, error);
    }
}