//! Error handling utilities

use anyhow::{Context, Result};

/// Helper macro for consistent error context
#[macro_export]
macro_rules! context_wrap {
    ($result:expr, $operation:expr) => {
        $result.context(format!("Failed to {}", $operation))
    };
}

/// Extension trait for Result types
pub trait ResultExt<T> {
    /// Add context with a standard "Failed to" prefix
    fn context_op(self, operation: &str) -> Result<T>;
    
    /// Add context with file path information
    fn context_path(self, operation: &str, path: &std::path::Path) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context_op(self, operation: &str) -> Result<T> {
        self.context(format!("Failed to {}", operation))
    }
    
    fn context_path(self, operation: &str, path: &std::path::Path) -> Result<T> {
        self.context(format!("Failed to {} at '{}'", operation, path.display()))
    }
}