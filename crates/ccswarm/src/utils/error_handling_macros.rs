/// Error handling macros for consistent error management across ccswarm
///
/// These macros provide standardized patterns for error handling,
/// reducing boilerplate and ensuring consistency.
/// Convert Option to Result with context
#[macro_export]
macro_rules! ok_or_error {
    ($option:expr, $error:expr) => {
        $option.ok_or_else(|| anyhow::anyhow!($error))
    };
    ($option:expr, $error:literal, $($arg:tt)*) => {
        $option.ok_or_else(|| anyhow::anyhow!($error, $($arg)*))
    };
}

/// Try operation with custom error context
#[macro_export]
macro_rules! try_with_context {
    ($expr:expr, $context:expr) => {
        $expr.context($context)?
    };
    ($expr:expr, $context:literal, $($arg:tt)*) => {
        $expr.context(format!($context, $($arg)*))?
    };
}

/// Execute async operation with timeout
#[macro_export]
macro_rules! timeout_async {
    ($duration:expr, $future:expr) => {
        tokio::time::timeout($duration, $future)
            .await
            .context("Operation timed out")?
    };
    ($duration:expr, $future:expr, $context:expr) => {
        tokio::time::timeout($duration, $future)
            .await
            .context($context)?
    };
}

/// Retry operation with exponential backoff
#[macro_export]
macro_rules! retry_with_backoff {
    ($operation:expr, $max_retries:expr) => {{
        use tokio::time::{sleep, Duration};

        let mut retries = 0;
        let mut delay = Duration::from_millis(100);

        loop {
            match $operation {
                Ok(result) => break Ok(result),
                Err(e) if retries < $max_retries => {
                    tracing::warn!(
                        "Operation failed (attempt {}/{}): {}",
                        retries + 1,
                        $max_retries,
                        e
                    );
                    sleep(delay).await;
                    delay *= 2;
                    retries += 1;
                }
                Err(e) => break Err(e),
            }
        }
    }};
}

/// Log error and continue
#[macro_export]
macro_rules! log_error_continue {
    ($result:expr) => {
        if let Err(e) = $result {
            tracing::error!("Error occurred: {:#}", e);
        }
    };
    ($result:expr, $message:expr) => {
        if let Err(e) = $result {
            tracing::error!("{}: {:#}", $message, e);
        }
    };
}

/// Ensure cleanup on scope exit
#[macro_export]
macro_rules! defer {
    ($cleanup:expr) => {
        let _guard = $crate::utils::error_handling_macros::DeferGuard::new(|| $cleanup);
    };
}

/// Guard for deferred cleanup
pub struct DeferGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> DeferGuard<F> {
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }
}

impl<F: FnOnce()> Drop for DeferGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// Handle multiple errors gracefully
#[macro_export]
macro_rules! collect_errors {
    ($($operation:expr),* $(,)?) => {{
        let mut errors = Vec::new();
        $(
            if let Err(e) = $operation {
                errors.push(e);
            }
        )*

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Multiple errors occurred: {:?}", errors))
        }
    }};
}

/// Map error types with context
#[macro_export]
macro_rules! map_error {
    ($result:expr, $error_type:ty) => {
        $result.map_err(|e| <$error_type>::from(e))
    };
    ($result:expr, $error_type:ty, $context:expr) => {
        $result
            .map_err(|e| <$error_type>::from(e))
            .context($context)
    };
}

/// Validate input with custom error
#[macro_export]
macro_rules! validate {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err(anyhow::anyhow!($error));
        }
    };
    ($condition:expr, $error:literal, $($arg:tt)*) => {
        if !$condition {
            return Err(anyhow::anyhow!($error, $($arg)*));
        }
    };
}

/// Execute with resource acquisition
#[macro_export]
macro_rules! with_resource {
    ($resource:expr, $operation:expr) => {{
        let resource = $resource;
        let result = $operation(&resource);
        drop(resource);
        result
    }};
}

/// Chain multiple operations with early return on error
#[macro_export]
macro_rules! chain_ops {
    ($($op:expr),* $(,)?) => {{
        $(
            $op?;
        )*
        Ok(())
    }};
}

/// Create error with backtrace
#[macro_export]
macro_rules! error_with_backtrace {
    ($msg:expr) => {{
        let backtrace = std::backtrace::Backtrace::capture();
        anyhow::anyhow!("{}\nBacktrace:\n{}", $msg, backtrace)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        let backtrace = std::backtrace::Backtrace::capture();
        anyhow::anyhow!("{}\nBacktrace:\n{}", format!($fmt, $($arg)*), backtrace)
    }};
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[test]
    fn test_ok_or_error() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;

        assert!(ok_or_error!(some_value, "No value").is_ok());
        assert!(ok_or_error!(none_value, "No value").is_err());
    }

    #[tokio::test]
    async fn test_timeout_async() {
        use anyhow::Context;
        use tokio::time::Duration;

        let fast_op = async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<_, anyhow::Error>(42)
        };

        let result = tokio::time::timeout(Duration::from_secs(1), fast_op)
            .await
            .context("Operation timed out");
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    #[test]
    fn test_validate() {
        fn check_positive(n: i32) -> Result<()> {
            validate!(n > 0, "Number must be positive, got {}", n);
            Ok(())
        }

        assert!(check_positive(5).is_ok());
        assert!(check_positive(-5).is_err());
    }
}
