/// Async error boundaries for graceful error handling in concurrent operations
///
/// This module provides error boundaries that prevent panic propagation
/// and ensure graceful degradation in async contexts.
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use tracing::{error, warn};

/// Error boundary for async operations
#[allow(dead_code)]
pub struct AsyncErrorBoundary<F> {
    future: F,
    name: String,
    #[allow(clippy::type_complexity)]
    fallback: Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send>>,
}

impl<F> AsyncErrorBoundary<F>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    /// Create a new error boundary
    pub fn new(future: F, name: impl Into<String>) -> Self {
        Self {
            future,
            name: name.into(),
            fallback: None,
        }
    }

    /// Add a fallback operation
    pub fn with_fallback<FB, FBF>(mut self, fallback: FB) -> Self
    where
        FB: Fn() -> FBF + Send + 'static,
        FBF: Future<Output = Result<()>> + Send + 'static,
    {
        self.fallback = Some(Box::new(move || Box::pin(fallback())));
        self
    }

    // Note: execute method moved to async_error_boundary_simple.rs for simpler implementation
    // Use boundary() or boundary_with_fallback() functions instead
}

/// Boundary for multiple concurrent operations
pub struct ConcurrentBoundary {
    operations: Vec<Pin<Box<dyn Future<Output = Result<()>> + Send>>>,
    name: String,
    fail_fast: bool,
    max_failures: Option<usize>,
}

impl ConcurrentBoundary {
    /// Create a new concurrent boundary
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            operations: Vec::new(),
            name: name.into(),
            fail_fast: false,
            max_failures: None,
        }
    }

    /// Add an operation to the boundary
    pub fn add_operation<F>(mut self, operation: F) -> Self
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.operations.push(Box::pin(operation));
        self
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    /// Set maximum allowed failures
    pub fn max_failures(mut self, max: usize) -> Self {
        self.max_failures = Some(max);
        self
    }

    /// Execute all operations within the boundary
    pub async fn execute(self) -> Result<Vec<Result<()>>> {
        use futures::future::{join_all, select_all};

        if self.fail_fast {
            // Execute with fail-fast behavior
            let mut futures = self.operations;
            let mut results = Vec::new();

            while !futures.is_empty() {
                let (result, _index, remaining) = select_all(futures).await;

                match result {
                    Ok(_) => results.push(Ok(())),
                    Err(e) => {
                        error!("Operation failed in '{}': {:#}", self.name, e);
                        results.push(Err(e));

                        // Cancel remaining operations
                        for _ in remaining {
                            results.push(Err(anyhow::anyhow!("Cancelled due to fail-fast")));
                        }
                        break;
                    }
                }

                futures = remaining;
            }

            Ok(results)
        } else {
            // Execute all operations regardless of failures
            let results = join_all(self.operations).await;

            let failure_count = results.iter().filter(|r| r.is_err()).count();

            if let Some(max) = self.max_failures {
                if failure_count > max {
                    return Err(anyhow::anyhow!(
                        "Too many failures in '{}': {} > {}",
                        self.name,
                        failure_count,
                        max
                    ));
                }
            }

            Ok(results)
        }
    }
}

/// Circuit breaker for async operations
pub struct AsyncCircuitBreaker {
    name: String,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: tokio::time::Duration,
    state: tokio::sync::RwLock<CircuitState>,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed { failure_count: usize },
    Open { opened_at: tokio::time::Instant },
    HalfOpen { success_count: usize },
}

impl AsyncCircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(
        name: impl Into<String>,
        failure_threshold: usize,
        success_threshold: usize,
        timeout: tokio::time::Duration,
    ) -> Self {
        Self {
            name: name.into(),
            failure_threshold,
            success_threshold,
            timeout,
            state: tokio::sync::RwLock::new(CircuitState::Closed { failure_count: 0 }),
        }
    }

    /// Execute operation through circuit breaker
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.timeout {
                    // Transition to half-open
                    *self.state.write().await = CircuitState::HalfOpen { success_count: 0 };
                    self.execute_in_half_open(operation).await
                } else {
                    Err(anyhow::anyhow!("Circuit breaker '{}' is open", self.name))
                }
            }
            CircuitState::Closed { .. } => self.execute_in_closed(operation).await,
            CircuitState::HalfOpen { .. } => self.execute_in_half_open(operation).await,
        }
    }

    async fn execute_in_closed<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match operation.await {
            Ok(result) => {
                *self.state.write().await = CircuitState::Closed { failure_count: 0 };
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.write().await;

                if let CircuitState::Closed { failure_count } = &mut *state {
                    *failure_count += 1;

                    if *failure_count >= self.failure_threshold {
                        warn!("Circuit breaker '{}' opening due to failures", self.name);
                        *state = CircuitState::Open {
                            opened_at: tokio::time::Instant::now(),
                        };
                    }
                }

                Err(e)
            }
        }
    }

    async fn execute_in_half_open<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match operation.await {
            Ok(result) => {
                let mut state = self.state.write().await;

                if let CircuitState::HalfOpen { success_count } = &mut *state {
                    *success_count += 1;

                    if *success_count >= self.success_threshold {
                        warn!("Circuit breaker '{}' closing after recovery", self.name);
                        *state = CircuitState::Closed { failure_count: 0 };
                    }
                }

                Ok(result)
            }
            Err(e) => {
                warn!(
                    "Circuit breaker '{}' reopening due to failure in half-open state",
                    self.name
                );
                *self.state.write().await = CircuitState::Open {
                    opened_at: tokio::time::Instant::now(),
                };
                Err(e)
            }
        }
    }
}

/// Create an error boundary for async operations
pub fn create_boundary<F>(future: F, name: impl Into<String>) -> AsyncErrorBoundary<F>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    AsyncErrorBoundary::new(future, name)
}

/// Execute with automatic retry and backoff
pub async fn with_retry<F, T>(
    operation: F,
    max_retries: usize,
    name: impl Into<String>,
) -> Result<T>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>>,
{
    let name = name.into();
    let mut attempts = 0;
    let mut delay = tokio::time::Duration::from_millis(100);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {:#}",
                    name, attempts, max_retries, e
                );
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => {
                error!("Operation '{}' failed after {} attempts", name, max_retries);
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: execute() method moved to async_error_boundary_simple.rs
    // Use boundary() function instead
    #[tokio::test]
    async fn test_async_error_boundary() {
        use crate::utils::boundary;
        
        let result = boundary(async { Ok::<_, anyhow::Error>(42) }, "test_boundary").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = AsyncCircuitBreaker::new(
            "test_breaker",
            2, // failure threshold
            2, // success threshold
            tokio::time::Duration::from_millis(100),
        );

        // First call should succeed
        let result = breaker.execute(async { Ok::<_, anyhow::Error>(1) }).await;
        assert!(result.is_ok());

        // Two failures should open the circuit
        let _ = breaker
            .execute(async { Err::<i32, _>(anyhow::anyhow!("fail")) })
            .await;
        let _ = breaker
            .execute(async { Err::<i32, _>(anyhow::anyhow!("fail")) })
            .await;

        // Circuit should be open
        let result = breaker.execute(async { Ok::<_, anyhow::Error>(2) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_boundary() {
        let boundary = ConcurrentBoundary::new("test_concurrent")
            .add_operation(async { Ok(()) })
            .add_operation(async { Ok(()) })
            .add_operation(async { Err(anyhow::anyhow!("test error")) })
            .max_failures(1);

        let results = boundary.execute().await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 2);
        assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
    }
}
