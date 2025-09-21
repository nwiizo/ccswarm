use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

use crate::error::CCSwarmError;

/// Enhanced retry mechanism with exponential backoff
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            exponential_base: 2.0,
        }
    }
}

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, CCSwarmError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, CCSwarmError>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) if attempt >= config.max_attempts => {
                return Err(CCSwarmError::Other(format!(
                    "Operation failed after {} attempts: {}",
                    config.max_attempts, err
                )));
            }
            Err(_) => {
                sleep(delay).await;

                // Calculate next delay with exponential backoff
                let next_delay_ms = (delay.as_millis() as f32 * config.exponential_base) as u64;
                delay = Duration::from_millis(next_delay_ms.min(config.max_delay.as_millis() as u64));
            }
        }
    }
}