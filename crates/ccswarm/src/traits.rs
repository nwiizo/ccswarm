/// Common traits and patterns used throughout ccswarm
///
/// This module defines reusable traits that provide common functionality
/// across different components of the system, promoting code reuse and
/// consistent interfaces.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
// use uuid::Uuid; // Commented out unused import

use crate::error::{CCSwarmError, Result};

/// Unique identifier trait for entities in the system
pub trait Identifiable {
    /// Get the unique identifier for this entity
    fn id(&self) -> &str;

    /// Get a human-readable name for this entity
    fn name(&self) -> &str {
        self.id()
    }
}

/// Trait for entities that can be serialized and have metadata
pub trait Describable: Identifiable {
    /// Get a description of this entity
    fn description(&self) -> Option<&str> {
        None
    }

    /// Get metadata associated with this entity
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get the creation timestamp
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the last modified timestamp
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
}

/// Trait for entities that have a lifecycle state
pub trait Stateful {
    type State: Clone + PartialEq + std::fmt::Debug;

    /// Get the current state
    fn state(&self) -> &Self::State;

    /// Check if the entity is in a valid state for operations
    fn is_operational(&self) -> bool;

    /// Get state transition history if available
    fn state_history(&self) -> Vec<(Self::State, chrono::DateTime<chrono::Utc>)> {
        Vec::new()
    }
}

/// Trait for configurable entities
pub trait Configurable {
    type Config: Clone + Serialize + for<'de> Deserialize<'de>;

    /// Get the current configuration
    fn config(&self) -> &Self::Config;

    /// Update the configuration
    fn update_config(&mut self, config: Self::Config) -> Result<()>;

    /// Validate a configuration
    fn validate_config(config: &Self::Config) -> Result<()>;

    /// Get default configuration
    fn default_config() -> Self::Config;
}

/// Trait for entities that can be monitored for health and performance
#[async_trait]
pub trait Monitorable {
    type HealthStatus: Clone + std::fmt::Debug + Serialize;
    type Metrics: Clone + std::fmt::Debug + Serialize;

    /// Check the health of this entity
    async fn health_check(&self) -> Result<Self::HealthStatus>;

    /// Get current metrics
    async fn metrics(&self) -> Result<Self::Metrics>;

    /// Get historical metrics if available
    async fn historical_metrics(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<(chrono::DateTime<chrono::Utc>, Self::Metrics)>> {
        let _ = since;
        Ok(Vec::new())
    }
}

/// Trait for entities that can execute tasks asynchronously
#[async_trait]
pub trait Executable {
    type Input: Send + Sync;
    type Output: Send + Sync;
    type Context: Send + Sync;

    /// Execute a task with the given input and context
    async fn execute(&mut self, input: Self::Input, context: Self::Context) -> Result<Self::Output>;

    /// Check if this executor can handle the given input
    fn can_execute(&self, input: &Self::Input) -> bool;

    /// Get estimated execution time
    fn estimated_duration(&self, input: &Self::Input) -> Option<Duration> {
        let _ = input;
        None
    }
}

/// Trait for entities that can be paused and resumed
#[async_trait]
pub trait Pausable {
    /// Pause the entity
    async fn pause(&mut self) -> Result<()>;

    /// Resume the entity
    async fn resume(&mut self) -> Result<()>;

    /// Check if the entity is currently paused
    fn is_paused(&self) -> bool;
}

/// Trait for entities that support graceful shutdown
#[async_trait]
pub trait Shutdownable {
    /// Initiate graceful shutdown
    async fn shutdown(&mut self) -> Result<()>;

    /// Force immediate shutdown
    async fn force_shutdown(&mut self) -> Result<()> {
        self.shutdown().await
    }

    /// Check if shutdown is in progress
    fn is_shutting_down(&self) -> bool;

    /// Get shutdown timeout
    fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Trait for entities that can validate their internal state
pub trait Validatable {
    type ValidationResult: std::fmt::Debug;

    /// Validate the current state of the entity
    fn validate(&self) -> Result<Self::ValidationResult>;

    /// Auto-fix validation issues if possible
    fn auto_fix(&mut self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

/// Trait for entities that support event notification
#[async_trait]
pub trait EventEmitter {
    type Event: Clone + Send + Sync + std::fmt::Debug;

    /// Emit an event
    async fn emit_event(&self, event: Self::Event) -> Result<()>;

    /// Subscribe to events (returns a channel receiver)
    async fn subscribe(&self) -> Result<tokio::sync::mpsc::Receiver<Self::Event>>;
}

/// Trait for caching and memoization support
#[async_trait]
pub trait Cacheable {
    type Key: Clone + Eq + std::hash::Hash + Send + Sync;
    type Value: Clone + Send + Sync;

    /// Get value from cache
    async fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    /// Set value in cache
    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Result<()>;

    /// Remove value from cache
    async fn remove(&mut self, key: &Self::Key) -> Result<Option<Self::Value>>;

    /// Clear all cached values
    async fn clear(&mut self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            entries: 0,
            memory_usage: 0,
            hit_rate: 0.0,
        }
    }

    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for retry logic with exponential backoff
#[async_trait]
pub trait Retryable {
    type Operation: Send + Sync;
    type Result: Send + Sync;

    /// Execute operation with retry logic
    async fn retry_with_backoff<F, Fut>(
        &self,
        operation: F,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<Self::Result>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<Self::Result>> + Send;

    /// Get default retry configuration
    fn default_retry_config() -> RetryConfig {
        RetryConfig::default()
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Trait for resource cleanup
#[async_trait]
pub trait Cleanupable {
    /// Clean up resources
    async fn cleanup(&mut self) -> Result<CleanupReport>;

    /// Check if cleanup is needed
    fn needs_cleanup(&self) -> bool {
        false
    }

    /// Get cleanup schedule
    fn cleanup_schedule(&self) -> Option<Duration> {
        None
    }
}

/// Cleanup report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub items_cleaned: usize,
    pub bytes_freed: usize,
    pub duration: Duration,
    pub errors: Vec<String>,
}

impl CleanupReport {
    pub fn new() -> Self {
        Self {
            items_cleaned: 0,
            bytes_freed: 0,
            duration: Duration::from_millis(0),
            errors: Vec::new(),
        }
    }

    pub fn with_items(mut self, count: usize) -> Self {
        self.items_cleaned = count;
        self
    }

    pub fn with_bytes_freed(mut self, bytes: usize) -> Self {
        self.bytes_freed = bytes;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn add_error<S: Into<String>>(mut self, error: S) -> Self {
        self.errors.push(error.into());
        self
    }
}

impl Default for CleanupReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for serializable entities
pub trait Persistable: Serialize + for<'de> Deserialize<'de> {
    /// Serialize to bytes
    fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(CCSwarmError::from)
    }

    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).map_err(CCSwarmError::from)
    }

    /// Save to file
    async fn save_to_file<P: AsRef<std::path::Path> + Send>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        tokio::fs::write(path, bytes).await.map_err(CCSwarmError::from)
    }

    /// Load from file
    async fn load_from_file<P: AsRef<std::path::Path> + Send>(path: P) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes = tokio::fs::read(path).await.map_err(CCSwarmError::from)?;
        Self::from_bytes(&bytes)
    }
}

/// Automatically implement Persistable for types that implement Serialize + Deserialize
impl<T> Persistable for T where T: Serialize + for<'de> Deserialize<'de> {}

/// Helper macro for generating unique IDs
#[macro_export]
macro_rules! generate_id {
    ($prefix:expr) => {
        format!("{}-{}", $prefix, uuid::Uuid::new_v4())
    };
    () => {
        uuid::Uuid::new_v4().to_string()
    };
}

/// Helper macro for creating timestamped entities
#[macro_export]
macro_rules! with_timestamp {
    ($entity:expr) => {{
        let now = chrono::Utc::now();
        $entity.created_at = now;
        $entity.updated_at = now;
        $entity
    }};
}

/// Helper macro for implementing common trait combinations
#[macro_export]
macro_rules! impl_entity_traits {
    ($type:ty, $id_field:ident, $name_field:ident) => {
        impl crate::traits::Identifiable for $type {
            fn id(&self) -> &str {
                &self.$id_field
            }

            fn name(&self) -> &str {
                &self.$name_field
            }
        }
    };
    ($type:ty, $id_field:ident) => {
        impl crate::traits::Identifiable for $type {
            fn id(&self) -> &str {
                &self.$id_field
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEntity {
        id: String,
        name: String,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    impl_entity_traits!(TestEntity, id, name);

    impl Describable for TestEntity {
        fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
            self.created_at
        }

        fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
            self.updated_at
        }
    }

    #[test]
    fn test_identifiable() {
        let entity = TestEntity {
            id: "test-123".to_string(),
            name: "Test Entity".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(entity.id(), "test-123");
        assert_eq!(entity.name(), "Test Entity");
    }

    #[test]
    fn test_generate_id_macro() {
        let id1 = generate_id!("test");
        let id2 = generate_id!("test");

        assert!(id1.starts_with("test-"));
        assert!(id2.starts_with("test-"));
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_persistable() {
        let entity = TestEntity {
            id: "test-123".to_string(),
            name: "Test Entity".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test serialization
        let bytes = entity.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Test deserialization
        let restored: TestEntity = TestEntity::from_bytes(&bytes).unwrap();
        assert_eq!(restored.id, entity.id);
        assert_eq!(restored.name, entity.name);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.hits = 80;
        stats.misses = 20;
        stats.calculate_hit_rate();

        assert_eq!(stats.hit_rate, 0.8);
    }

    #[test]
    fn test_cleanup_report() {
        let report = CleanupReport::new()
            .with_items(10)
            .with_bytes_freed(1024)
            .with_duration(Duration::from_millis(500))
            .add_error("Test error");

        assert_eq!(report.items_cleaned, 10);
        assert_eq!(report.bytes_freed, 1024);
        assert_eq!(report.duration, Duration::from_millis(500));
        assert_eq!(report.errors.len(), 1);
    }
}