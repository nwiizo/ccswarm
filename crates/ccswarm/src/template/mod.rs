//! Task template system for ccswarm
//!
//! This module provides a comprehensive template system that allows users to:
//! - Create and manage task templates
//! - Use predefined templates for common development tasks
//! - Store and organize custom templates
//! - Apply templates with variable substitution
//! - Discover and list available templates

pub mod manager;
pub mod predefined;
pub mod storage;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use manager::TemplateManager;
pub use predefined::PredefinedTemplates;
pub use storage::{FileSystemTemplateStorage, TemplateStorage};
pub use types::{Template, TemplateCategory, TemplateQuery, TemplateVariable, VariableType};
pub use validation::TemplateValidator;

use std::collections::HashMap;

/// Template system error types
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {name}")]
    NotFound { name: String },

    #[error("Template validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Variable substitution failed: {variable}")]
    SubstitutionFailed { variable: String },

    #[error("Template storage error: {source}")]
    StorageError {
        #[from]
        source: anyhow::Error,
    },

    #[error("Template already exists: {name}")]
    AlreadyExists { name: String },

    #[error("Invalid template format: {reason}")]
    InvalidFormat { reason: String },
}

/// Template application context
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Project name
    pub project_name: Option<String>,
    /// Target agent role
    pub agent_role: Option<String>,
    /// Additional context variables
    pub variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Set agent role
    pub fn with_agent_role(mut self, role: impl Into<String>) -> Self {
        self.agent_role = Some(role.into());
        self
    }

    /// Add a variable
    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables
    pub fn with_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }
}

/// Template application result
#[derive(Debug, Clone)]
pub struct AppliedTemplate {
    /// Generated task description
    pub description: String,
    /// Generated task details
    pub details: Option<String>,
    /// Suggested priority
    pub priority: crate::agent::Priority,
    /// Suggested task type
    pub task_type: crate::agent::TaskType,
    /// Estimated duration in minutes
    pub estimated_duration: Option<u32>,
    /// Files that should be created or modified
    pub target_files: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
