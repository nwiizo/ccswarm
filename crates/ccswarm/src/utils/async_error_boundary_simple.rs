use std::future::Future;

use crate::error::CCSwarmError;

/// Simple async error boundary without Pin complexity
pub async fn catch_async<F, T>(f: F) -> Result<T, CCSwarmError>
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    f.await
}

/// Run async function with default error handling
pub async fn run_safe<F, T>(f: F) -> Option<T>
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    match f.await {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Error in async boundary: {}", err);
            None
        }
    }
}

/// Simple boundary wrapper
pub async fn boundary<F, T>(f: F) -> Result<T, CCSwarmError>
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    f.await
}

/// Boundary with fallback value
pub async fn boundary_with_fallback<F, T>(f: F, fallback: T) -> T
where
    F: Future<Output = Result<T, CCSwarmError>>,
{
    match f.await {
        Ok(val) => val,
        Err(_) => fallback,
    }
}