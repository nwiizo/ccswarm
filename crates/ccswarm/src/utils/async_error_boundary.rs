use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::CCSwarmError;

/// Async error boundary for handling errors in async contexts
pub struct AsyncErrorBoundary<F> {
    future: F,
}

impl<F> AsyncErrorBoundary<F> {
    pub fn new(future: F) -> Self {
        Self { future }
    }
}

impl<F, T> Future for AsyncErrorBoundary<F>
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    type Output = Result<T, CCSwarmError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: This is a manual pin projection. The `future` field is structurally pinned:
        // - It is never moved after being pinned (no mem::swap, no mem::replace)
        // - It is never exposed via &mut after pinning
        // - The Drop impl does not move it
        // This is equivalent to what the `pin-project` crate generates automatically.
        unsafe {
            let this = self.get_unchecked_mut();
            let future = Pin::new_unchecked(&mut this.future);
            future.poll(cx)
        }
    }
}

/// Wrap an async function with error boundary
pub fn with_error_boundary<F, T>(future: F) -> AsyncErrorBoundary<F>
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    AsyncErrorBoundary::new(future)
}

/// Retry async operation
pub async fn with_retry<F, T>(f: F, max_retries: u32) -> Result<T, CCSwarmError>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, CCSwarmError>> + Send>>,
{
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_retries - 1 => return Err(e),
            _ => continue,
        }
    }
    Err(CCSwarmError::Other {
        message: "Max retries exceeded".to_string(),
        source: None,
    })
}

/// Async circuit breaker
pub struct AsyncCircuitBreaker {
    failure_threshold: u32,
    current_failures: u32,
    is_open: bool,
}

impl AsyncCircuitBreaker {
    pub fn new(threshold: u32) -> Self {
        Self {
            failure_threshold: threshold,
            current_failures: 0,
            is_open: false,
        }
    }

    pub async fn call<F, T>(&mut self, f: F) -> Result<T, CCSwarmError>
    where
        F: Future<Output = Result<T, CCSwarmError>>,
    {
        if self.is_open {
            return Err(CCSwarmError::Other {
                message: "Circuit breaker is open".to_string(),
                source: None,
            });
        }

        match f.await {
            Ok(result) => {
                self.current_failures = 0;
                Ok(result)
            }
            Err(e) => {
                self.current_failures += 1;
                if self.current_failures >= self.failure_threshold {
                    self.is_open = true;
                }
                Err(e)
            }
        }
    }
}

/// Concurrent boundary for parallel operations
pub struct ConcurrentBoundary {
    max_concurrent: usize,
}

impl ConcurrentBoundary {
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    pub fn max(&self) -> usize {
        self.max_concurrent
    }
}
