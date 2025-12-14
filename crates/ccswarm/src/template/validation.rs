//! Template validation utilities

use super::Template;
use crate::template::types::TemplateError;

/// Result of applying a template with substituted values
#[derive(Debug, Clone)]
pub struct AppliedTemplate {
    /// The original template that was applied
    pub template: Template,
    /// The substituted task description
    pub task_description: String,
    /// The substituted task details (if any)
    pub task_details: Option<String>,
    /// Applied variables
    pub applied_variables: std::collections::HashMap<String, String>,

    // Additional fields for compatibility
    /// The description (alias for task_description)
    pub description: String,
    /// The details (alias for task_details)
    pub details: Option<String>,
    /// The priority from the template
    pub priority: crate::agent::Priority,
    /// The task type from the template
    pub task_type: crate::agent::TaskType,
    /// Estimated duration from template
    pub estimated_duration: Option<u32>,
    /// Target files from template
    pub target_files: Vec<String>,
}

/// Template validator for ensuring template quality and correctness
pub struct TemplateValidator;

impl TemplateValidator {
    /// Create a new template validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a template (basic implementation)
    pub fn validate(&self, template: &Template) -> ValidationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Basic validation
        if template.id.is_empty() {
            errors.push(ValidationError {
                message: "Template ID cannot be empty".to_string(),
                field: Some("id".to_string()),
                severity: ErrorSeverity::Critical,
                suggestion: Some("Provide a unique identifier".to_string()),
            });
        }

        if template.name.is_empty() {
            errors.push(ValidationError {
                message: "Template name cannot be empty".to_string(),
                field: Some("name".to_string()),
                severity: ErrorSeverity::Critical,
                suggestion: Some("Provide a descriptive name".to_string()),
            });
        }

        if template.task_description.is_empty() {
            errors.push(ValidationError {
                message: "Task description cannot be empty".to_string(),
                field: Some("task_description".to_string()),
                severity: ErrorSeverity::Critical,
                suggestion: Some("Provide a task description template".to_string()),
            });
        }

        let is_valid = errors.iter().all(|e| e.severity != ErrorSeverity::Critical);
        let quality_score = if is_valid { 0.8 } else { 0.3 };

        ValidationResult {
            is_valid,
            errors,
            warnings,
            quality_score,
        }
    }
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Template validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub quality_score: f64,
}

/// Validation error with severity and context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
    pub severity: ErrorSeverity,
    pub suggestion: Option<String>,
}

/// Validation warning for potential issues
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Critical,
    Major,
    Minor,
}

/// Quick validation function for basic template validation
pub fn validate_template(template: &Template) -> Result<(), TemplateError> {
    let validator = TemplateValidator::new();
    let result = validator.validate(template);

    if !result.is_valid {
        let critical_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Critical)
            .map(|e| e.message.clone())
            .collect();

        if !critical_errors.is_empty() {
            return Err(TemplateError::ValidationFailed {
                reason: critical_errors.join("; "),
            });
        }
    }

    Ok(())
}
