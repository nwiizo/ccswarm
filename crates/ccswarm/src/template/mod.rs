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
pub mod template_factory;
pub mod types;
pub mod validation;

// Re-export commonly used types
pub use manager::TemplateManager;
pub use predefined::PredefinedTemplates;
pub use storage::FileSystemTemplateStorage;
pub use types::{
    Template, TemplateCategory, TemplateContext, TemplateError, TemplateQuery, TemplateVariable,
    VariableType,
};
pub use validation::AppliedTemplate;
