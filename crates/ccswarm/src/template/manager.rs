//! Template manager for handling template operations

use super::{
    storage::TemplateStorage, types::TemplateQuery, AppliedTemplate, Template, TemplateCategory,
    TemplateContext, TemplateError, TemplateVariable, VariableType,
};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use tracing::{info, warn};

/// Template manager that handles all template operations
pub struct TemplateManager<T: TemplateStorage> {
    storage: T,
    /// Cache for compiled regex patterns
    regex_cache: HashMap<String, Regex>,
}

impl<T: TemplateStorage> TemplateManager<T> {
    /// Create a new template manager
    pub fn new(storage: T) -> Self {
        Self {
            storage,
            regex_cache: HashMap::new(),
        }
    }

    /// Save a new template
    pub async fn save_template(&mut self, template: Template) -> Result<(), TemplateError> {
        self.validate_template(&template)?;
        self.storage.save_template(&template).await?;
        info!("Saved template: {} ({})", template.name, template.id);
        Ok(())
    }

    /// Load a template by ID
    pub async fn load_template(&self, id: &str) -> Result<Template, TemplateError> {
        self.storage.load_template(id).await
    }

    /// Delete a template
    pub async fn delete_template(&mut self, id: &str) -> Result<(), TemplateError> {
        self.storage.delete_template(id).await?;
        info!("Deleted template: {}", id);
        Ok(())
    }

    /// List all templates
    pub async fn list_templates(&self) -> Result<Vec<Template>, TemplateError> {
        self.storage.list_templates().await
    }

    /// Search templates
    pub async fn search_templates(
        &self,
        query: TemplateQuery,
    ) -> Result<Vec<Template>, TemplateError> {
        self.storage.search_templates(&query).await
    }

    /// Apply a template with given context
    pub async fn apply_template(
        &mut self,
        template_id: &str,
        context: TemplateContext,
    ) -> Result<AppliedTemplate, TemplateError> {
        let template = self.load_template(template_id).await?;

        // Validate that all required variables are provided
        self.validate_context(&template, &context)?;

        // Substitute variables in template
        let description =
            self.substitute_variables(&template.task_description, &template, &context)?;
        let details = if let Some(ref details_template) = template.task_details {
            Some(self.substitute_variables(details_template, &template, &context)?)
        } else {
            None
        };

        // Substitute variables in target files
        let target_files = template
            .target_files
            .iter()
            .map(|file| self.substitute_variables(file, &template, &context))
            .collect::<Result<Vec<_>, _>>()?;

        // Update usage statistics
        self.storage.update_usage(&template.id, true).await?;

        Ok(AppliedTemplate {
            description,
            details,
            priority: template.default_priority,
            task_type: template.default_task_type,
            estimated_duration: template.estimated_duration,
            target_files,
            metadata: template.metadata.clone(),
        })
    }

    /// Get template by name (searches if not found by ID)
    pub async fn get_template_by_name(&self, name: &str) -> Result<Template, TemplateError> {
        // First try to load by ID
        if let Ok(template) = self.load_template(name).await {
            return Ok(template);
        }

        // Search by name
        let query = TemplateQuery::new().with_search_term(name).with_limit(1);
        let templates = self.search_templates(query).await?;

        templates
            .into_iter()
            .next()
            .ok_or_else(|| TemplateError::NotFound {
                name: name.to_string(),
            })
    }

    /// Get templates by category
    pub async fn get_templates_by_category(
        &self,
        category: TemplateCategory,
    ) -> Result<Vec<Template>, TemplateError> {
        let query = TemplateQuery::new().with_category(category);
        self.search_templates(query).await
    }

    /// Get popular templates
    pub async fn get_popular_templates(
        &self,
        limit: usize,
    ) -> Result<Vec<Template>, TemplateError> {
        let query = TemplateQuery::new().sort_by_popularity().with_limit(limit);
        self.search_templates(query).await
    }

    /// Get recent templates
    pub async fn get_recent_templates(&self, limit: usize) -> Result<Vec<Template>, TemplateError> {
        let query = TemplateQuery::new().sort_by_date().with_limit(limit);
        self.search_templates(query).await
    }

    /// Validate template
    fn validate_template(&self, template: &Template) -> Result<(), TemplateError> {
        if !template.is_valid() {
            return Err(TemplateError::ValidationFailed {
                reason: "Template is missing required fields".to_string(),
            });
        }

        // Validate variable references in template
        self.validate_variable_references(&template.task_description, &template.variables)?;

        if let Some(ref details) = template.task_details {
            self.validate_variable_references(details, &template.variables)?;
        }

        for file in &template.target_files {
            self.validate_variable_references(file, &template.variables)?;
        }

        // Validate variable definitions
        for variable in &template.variables {
            self.validate_variable_definition(variable)?;
        }

        Ok(())
    }

    /// Validate variable references in text
    fn validate_variable_references(
        &self,
        text: &str,
        variables: &[TemplateVariable],
    ) -> Result<(), TemplateError> {
        let var_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let defined_vars: std::collections::HashSet<_> =
            variables.iter().map(|v| &v.name).collect();

        for capture in var_regex.captures_iter(text) {
            let var_name = capture[1].to_string();
            if !defined_vars.contains(&var_name) {
                return Err(TemplateError::ValidationFailed {
                    reason: format!("Variable '{}' is referenced but not defined", var_name),
                });
            }
        }

        Ok(())
    }

    /// Validate variable definition
    fn validate_variable_definition(
        &self,
        variable: &TemplateVariable,
    ) -> Result<(), TemplateError> {
        // Check name format
        if variable.name.is_empty()
            || !variable
                .name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(TemplateError::ValidationFailed {
                reason: format!("Invalid variable name: '{}'", variable.name),
            });
        }

        // Validate choice options
        if let VariableType::Choice(ref choices) = variable.variable_type {
            if choices.is_empty() {
                return Err(TemplateError::ValidationFailed {
                    reason: format!("Choice variable '{}' has no options", variable.name),
                });
            }
        }

        // Validate default value against type
        if let Some(ref default) = variable.default_value {
            self.validate_variable_value(&variable.name, default, &variable.variable_type)?;
        }

        Ok(())
    }

    /// Validate variable value against type
    fn validate_variable_value(
        &self,
        var_name: &str,
        value: &str,
        var_type: &VariableType,
    ) -> Result<(), TemplateError> {
        match var_type {
            VariableType::Text => {
                // Text can be anything
                Ok(())
            }
            VariableType::Boolean => {
                if !["true", "false", "1", "0", "yes", "no"]
                    .contains(&value.to_lowercase().as_str())
                {
                    Err(TemplateError::ValidationFailed {
                        reason: format!("Invalid boolean value for '{}': '{}'", var_name, value),
                    })
                } else {
                    Ok(())
                }
            }
            VariableType::Number => {
                if value.parse::<f64>().is_err() {
                    Err(TemplateError::ValidationFailed {
                        reason: format!("Invalid number value for '{}': '{}'", var_name, value),
                    })
                } else {
                    Ok(())
                }
            }
            VariableType::FilePath => {
                // Basic path validation
                if value.contains('\0') {
                    Err(TemplateError::ValidationFailed {
                        reason: format!("Invalid file path for '{}': contains null byte", var_name),
                    })
                } else {
                    Ok(())
                }
            }
            VariableType::Url => {
                // Basic URL validation - just check if it contains ://
                if value.contains("://") {
                    Ok(())
                } else {
                    Err(TemplateError::ValidationFailed {
                        reason: format!("Invalid URL for '{}': '{}'", var_name, value),
                    })
                }
            }
            VariableType::List => {
                // Lists are comma-separated values
                Ok(())
            }
            VariableType::Choice(ref choices) => {
                if !choices.contains(&value.to_string()) {
                    Err(TemplateError::ValidationFailed {
                        reason: format!(
                            "Invalid choice for '{}': '{}'. Must be one of: {}",
                            var_name,
                            value,
                            choices.join(", ")
                        ),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Validate context against template requirements
    fn validate_context(
        &self,
        template: &Template,
        context: &TemplateContext,
    ) -> Result<(), TemplateError> {
        for variable in &template.variables {
            if variable.required {
                let value = context.variables.get(&variable.name);
                if value.is_none() && variable.default_value.is_none() {
                    return Err(TemplateError::ValidationFailed {
                        reason: format!("Required variable '{}' not provided", variable.name),
                    });
                }
            }

            // Validate provided values
            if let Some(value) = context.variables.get(&variable.name) {
                self.validate_variable_value(&variable.name, value, &variable.variable_type)?;
            }
        }

        Ok(())
    }

    /// Substitute variables in text
    fn substitute_variables(
        &mut self,
        text: &str,
        template: &Template,
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let var_regex = if let Some(regex) = self.regex_cache.get("variable") {
            regex.clone()
        } else {
            let regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
            self.regex_cache
                .insert("variable".to_string(), regex.clone());
            regex
        };

        let mut result = text.to_string();

        for capture in var_regex.captures_iter(text) {
            let var_name = capture[1].to_string();
            let placeholder = &capture[0];

            // Find variable definition
            let variable = template
                .variables
                .iter()
                .find(|v| v.name == var_name)
                .ok_or_else(|| TemplateError::SubstitutionFailed {
                    variable: var_name.clone(),
                })?;

            // Get value from context or use default
            let value = context
                .variables
                .get(&var_name)
                .or(variable.default_value.as_ref())
                .ok_or_else(|| TemplateError::SubstitutionFailed {
                    variable: var_name.clone(),
                })?;

            // Apply variable-specific transformations
            let transformed_value =
                self.transform_variable_value(value, &variable.variable_type)?;

            result = result.replace(placeholder, &transformed_value);
        }

        // Handle built-in variables
        result = self.substitute_builtin_variables(&result, context)?;

        Ok(result)
    }

    /// Transform variable value based on type
    fn transform_variable_value(
        &self,
        value: &str,
        var_type: &VariableType,
    ) -> Result<String, TemplateError> {
        match var_type {
            VariableType::Boolean => {
                let normalized = match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" => "true",
                    "false" | "0" | "no" => "false",
                    _ => value,
                };
                Ok(normalized.to_string())
            }
            VariableType::List => {
                // Convert comma-separated values to array format
                let items: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
                Ok(format!("[{}]", items.join(", ")))
            }
            _ => Ok(value.to_string()),
        }
    }

    /// Substitute built-in variables
    fn substitute_builtin_variables(
        &self,
        text: &str,
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let mut result = text.to_string();

        // Project name
        if let Some(ref project_name) = context.project_name {
            result = result.replace("{{project_name}}", project_name);
        }

        // Agent role
        if let Some(ref agent_role) = context.agent_role {
            result = result.replace("{{agent_role}}", agent_role);
        }

        // Current date/time
        let now = chrono::Utc::now();
        result = result.replace("{{current_date}}", &now.format("%Y-%m-%d").to_string());
        result = result.replace("{{current_time}}", &now.format("%H:%M:%S").to_string());
        result = result.replace(
            "{{current_datetime}}",
            &now.format("%Y-%m-%d %H:%M:%S").to_string(),
        );

        Ok(result)
    }

    /// Get template usage statistics
    pub async fn get_template_stats(&self) -> Result<super::storage::TemplateStats, TemplateError> {
        self.storage.get_stats().await
    }

    /// Update template (replace existing)
    pub async fn update_template(&mut self, template: Template) -> Result<(), TemplateError> {
        self.validate_template(&template)?;

        // Delete existing template
        if self.storage.exists(&template.id).await? {
            self.storage.delete_template(&template.id).await?;
        }

        // Save updated template
        self.storage.save_template(&template).await?;
        info!("Updated template: {} ({})", template.name, template.id);
        Ok(())
    }

    /// Import templates from another storage
    pub async fn import_templates(
        &mut self,
        templates: Vec<Template>,
    ) -> Result<Vec<String>, TemplateError> {
        let mut imported = Vec::new();
        let mut errors = Vec::new();

        for template in templates {
            match self.save_template(template.clone()).await {
                Ok(()) => {
                    imported.push(template.id.clone());
                    info!("Imported template: {}", template.id);
                }
                Err(e) => {
                    warn!("Failed to import template {}: {}", template.id, e);
                    errors.push(format!("{}: {}", template.id, e));
                }
            }
        }

        if !errors.is_empty() {
            warn!("Failed to import {} templates: {:?}", errors.len(), errors);
        }

        Ok(imported)
    }

    /// Export all templates
    pub async fn export_templates(&self) -> Result<Vec<Template>, TemplateError> {
        self.list_templates().await
    }

    /// Clone template with new ID
    pub async fn clone_template(
        &mut self,
        source_id: &str,
        new_id: &str,
        new_name: Option<String>,
    ) -> Result<Template, TemplateError> {
        let mut template = self.load_template(source_id).await?;

        // Update ID and name
        template.id = new_id.to_string();
        if let Some(name) = new_name {
            template.name = name;
        } else {
            template.name = format!("{} (Copy)", template.name);
        }

        // Reset statistics
        template.usage_count = 0;
        template.success_rate = None;
        template.created_at = chrono::Utc::now();
        template.updated_at = chrono::Utc::now();

        // Save cloned template
        self.save_template(template.clone()).await?;

        Ok(template)
    }
}
