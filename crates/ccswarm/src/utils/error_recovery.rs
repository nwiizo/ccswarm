use std::future::Future;
use tokio::time::{sleep, Duration};

use crate::error::CCSwarmError;

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, delay_ms: u64 },
    Fallback,
    CircuitBreaker { threshold: u32, reset_time: Duration },
    Ignore,
}

/// Error recovery handler
pub struct ErrorRecovery {
    strategy: RecoveryStrategy,
    failure_count: u32,
}

impl ErrorRecovery {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            failure_count: 0,
        }
    }

    pub async fn recover<F, Fut, T>(
        &mut self,
        operation: F,
        fallback: Option<T>,
    ) -> Result<T, CCSwarmError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, CCSwarmError>>,
    {
        match &self.strategy {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                for attempt in 0..*max_attempts {
                    match operation().await {
                        Ok(result) => {
                            self.failure_count = 0;
                            return Ok(result);
                        }
                        Err(_e) if attempt < max_attempts - 1 => {
                            self.failure_count += 1;
                            sleep(Duration::from_millis(*delay_ms)).await;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(CCSwarmError::Other {
                    message: "Max retry attempts reached".to_string(),
                    source: None,
                })
            }
            RecoveryStrategy::Fallback => {
                match operation().await {
                    Ok(result) => Ok(result),
                    Err(_) if fallback.is_some() => Ok(fallback.unwrap()),
                    Err(e) => Err(e),
                }
            }
            RecoveryStrategy::CircuitBreaker { threshold, reset_time: _ } => {
                if self.failure_count >= *threshold {
                    return Err(CCSwarmError::Other {
                        message: "Circuit breaker open".to_string(),
                        source: None,
                    });
                }

                match operation().await {
                    Ok(result) => {
                        self.failure_count = 0;
                        Ok(result)
                    }
                    Err(e) => {
                        self.failure_count += 1;
                        Err(e)
                    }
                }
            }
            RecoveryStrategy::Ignore => {
                operation().await
            }
        }
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new(RecoveryStrategy::Retry {
            max_attempts: 3,
            delay_ms: 1000,
        })
    }
}

/// Recovery action to take
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry,
    Restart,
    Rollback,
    Skip,
    Abort,
}

/// Error resolver
pub struct ErrorResolver {
    strategies: Vec<RecoveryStrategy>,
}

impl Default for ErrorResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorResolver {
    pub fn new() -> Self {
        Self {
            strategies: vec![RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 1000 }],
        }
    }

    pub fn add_strategy(&mut self, strategy: RecoveryStrategy) {
        self.strategies.push(strategy);
    }
}

/// Error recovery database
pub struct ErrorRecoveryDB {
    known_errors: std::collections::HashMap<String, RecoveryAction>,
}

impl Default for ErrorRecoveryDB {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryDB {
    pub fn new() -> Self {
        Self {
            known_errors: std::collections::HashMap::new(),
        }
    }

    pub fn add_error(&mut self, error_type: String, action: RecoveryAction) {
        self.known_errors.insert(error_type, action);
    }

    pub fn get_action(&self, error_type: &str) -> Option<&RecoveryAction> {
        self.known_errors.get(error_type)
    }

    pub fn get_recovery(&self, error_type: &str) -> Option<RecoveryStep> {
        self.get_action(error_type).map(|action| {
            RecoveryStep::new(action.clone(), format!("Recovery for {}", error_type))
        })
    }

    pub async fn auto_fix(&self, error_type: &str) -> Result<(), CCSwarmError> {
        if let Some(recovery) = self.get_recovery(error_type) {
            match recovery {
                RecoveryStep::Command { .. } => {
                    // Can potentially auto-fix commands
                    Ok(())
                }
                _ => {
                    Err(CCSwarmError::Other {
                        message: "Cannot auto-fix this error".to_string(),
                        source: None,
                    })
                }
            }
        } else {
            Err(CCSwarmError::Other {
                message: "No recovery found for error".to_string(),
                source: None,
            })
        }
    }
}

/// Recovery step
#[derive(Debug, Clone)]
pub enum RecoveryStep {
    Command {
        cmd: String,
        description: String,
    },
    FileCreate {
        path: String,
        content: String,
    },
    EnvVar {
        name: String,
        example: String,
    },
    UserAction {
        description: String,
    },
}

impl RecoveryStep {
    pub fn new(_action: RecoveryAction, description: String) -> Self {
        Self::UserAction { description }
    }

    pub fn with_time(self, _duration: Duration) -> Self {
        self
    }
}