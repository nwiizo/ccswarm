//! Utility functions and helpers

pub mod command;
pub mod error;
pub mod error_diagrams;
pub mod error_recovery;
pub mod user_error;

#[cfg(test)]
mod error_tests;

pub use command::CommandExecutor;
pub use error::ResultExt;
pub use error_diagrams::{show_diagram, ErrorDiagrams};
pub use error_recovery::{ErrorRecoveryDB, ErrorResolver, RecoveryAction};
pub use user_error::{show_progress, CommonErrors, UserError, UserErrorExt};
