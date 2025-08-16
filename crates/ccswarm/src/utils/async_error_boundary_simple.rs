/// Simple async error boundary implementation
use anyhow::Result;
use std::future::Future;
use tracing::{error, warn};

/// Simple boundary helper function
pub async fn boundary<F, T>(future: F, name: &str) -> Result<T>
where
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    // Use tokio spawn to isolate panics
    let name = name.to_string();
    let handle = tokio::task::spawn(future);

    match handle.await {
        Ok(result) => result,
        Err(join_error) => {
            error!("Task error in '{}': {:#}", name, join_error);
            Err(anyhow::anyhow!("Task failed: {}", join_error))
        }
    }
}

/// Boundary with fallback
pub async fn boundary_with_fallback<F, T, FB>(future: F, name: &str, fallback: FB) -> Result<T>
where
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
    FB: Future<Output = Result<T>> + Send + 'static,
{
    let name = name.to_string();
    let handle = tokio::task::spawn(future);

    match handle.await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => {
            error!("Error in '{}': {:#}", name, e);
            warn!("Executing fallback for '{}'", name);
            fallback.await
        }
        Err(join_error) => {
            error!("Task error in '{}': {:#}", name, join_error);
            warn!("Executing fallback for '{}'", name);
            fallback.await
        }
    }
}
