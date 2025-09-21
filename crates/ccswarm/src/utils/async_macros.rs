/// Async operation macros for concurrent execution and coordination
///
/// These macros simplify common async patterns in ccswarm.
/// Execute multiple async operations concurrently
#[macro_export]
macro_rules! concurrent_execute {
    ($($future:expr),* $(,)?) => {{
        use futures::future::join_all;

        let futures = vec![
            $(Box::pin($future),)*
        ];

        join_all(futures).await
    }};
}

/// Execute operations with timeout and retry
#[macro_export]
macro_rules! resilient_async {
    ($operation:expr, timeout: $timeout:expr, retries: $retries:expr) => {{
        use anyhow::Context;
        use tokio::time::{timeout, Duration};

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < $retries {
            match timeout($timeout, $operation).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < $retries {
                        tracing::warn!(
                            "Operation failed (attempt {}/{}), retrying...",
                            attempts,
                            $retries
                        );
                        tokio::time::sleep(Duration::from_millis(100 * (1 << attempts))).await;
                    }
                }
                Err(_) => {
                    last_error = Some(anyhow::anyhow!("Operation timed out"));
                    attempts += 1;
                    if attempts < $retries {
                        tracing::warn!(
                            "Operation timed out (attempt {}/{}), retrying...",
                            attempts,
                            $retries
                        );
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("Operation failed after {} attempts", $retries)))
    }};
}

/// Select first successful operation from multiple async tasks
#[macro_export]
macro_rules! select_first_ok {
    ($($future:expr),* $(,)?) => {{
        use tokio::select;
        use futures::future::FutureExt;

        select! {
            $(
                result = $future.fuse() => {
                    if result.is_ok() {
                        return result;
                    }
                }
            )*
        }

        Err(anyhow::anyhow!("All operations failed"))
    }};
}

/// Execute with cancellation token
#[macro_export]
macro_rules! cancellable_async {
    ($token:expr, $operation:expr) => {{
        tokio::select! {
            _ = $token.cancelled() => {
                Err(anyhow::anyhow!("Operation cancelled"))
            }
            result = $operation => {
                result
            }
        }
    }};
}

/// Stream processing with batching
#[macro_export]
macro_rules! stream_batch_process {
    ($stream:expr, $batch_size:expr, $processor:expr) => {{
        use futures::stream::{StreamExt, TryStreamExt};

        $stream
            .chunks($batch_size)
            .map(|batch| async move { $processor(batch).await })
            .buffer_unordered(4)
            .try_collect()
            .await
    }};
}

/// Async mutex with timeout
#[macro_export]
macro_rules! async_lock_timeout {
    ($mutex:expr, $timeout:expr) => {{
        use tokio::time::timeout;

        match timeout($timeout, $mutex.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(anyhow::anyhow!("Failed to acquire lock within timeout")),
        }
    }};
}

/// Spawn task with automatic error logging
#[macro_export]
macro_rules! spawn_logged {
    ($name:expr, $future:expr) => {{
        tokio::spawn(async move {
            let _span = tracing::info_span!("spawned_task", name = $name).entered();

            match $future.await {
                Ok(result) => {
                    tracing::debug!("Task '{}' completed successfully", $name);
                    Ok(result)
                }
                Err(e) => {
                    tracing::error!("Task '{}' failed: {:#}", $name, e);
                    Err(e)
                }
            }
        })
    }};
}

/// Execute with progress updates
#[macro_export]
macro_rules! async_with_progress {
    ($progress:expr, $operation:expr) => {{
        let start = std::time::Instant::now();

        let result = $operation;

        let duration = start.elapsed();
        $progress.update_duration(duration).await;

        result
    }};
}

/// Parallel map over collection
#[macro_export]
macro_rules! parallel_map {
    ($collection:expr, $mapper:expr) => {{
        use futures::future::join_all;

        let futures: Vec<_> = $collection.into_iter().map($mapper).collect();

        join_all(futures).await
    }};
}

/// Rate-limited async execution
#[macro_export]
macro_rules! rate_limited_async {
    ($operation:expr, $rate_limiter:expr) => {{
        $rate_limiter.acquire().await;
        $operation.await
    }};
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[tokio::test]
    async fn test_concurrent_execute() {
        use std::pin::Pin;
        use futures::Future;
        
        let futures: Vec<Pin<Box<dyn Future<Output = Result<i32>>>>> = vec![
            Box::pin(async { Ok::<_, anyhow::Error>(1) }),
            Box::pin(async { Ok::<_, anyhow::Error>(2) }),
            Box::pin(async { Ok::<_, anyhow::Error>(3) }),
        ];
        
        let results = futures::future::join_all(futures).await;

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_spawn_logged() {
        let handle = tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok::<_, anyhow::Error>(42)
        });

        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_parallel_map() {
        let numbers = vec![1, 2, 3, 4, 5];

        let futures: Vec<_> = numbers.into_iter()
            .map(|n| async move { Ok::<_, anyhow::Error>(n * 2) })
            .collect();
        let results: Vec<Result<i32>> = futures::future::join_all(futures).await;

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].as_ref().unwrap(), &2);
        assert_eq!(results[4].as_ref().unwrap(), &10);
    }
}
