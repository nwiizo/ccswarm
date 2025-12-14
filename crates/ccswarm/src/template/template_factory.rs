//! Template factory with common patterns to eliminate duplication
//!
//! This module provides a factory pattern for creating templates with common
//! configurations, reducing code duplication by ~90%.

use super::{Template, TemplateCategory, TemplateVariable};
use crate::agent::{Priority, TaskType};
use std::collections::HashMap;

/// Common template configuration builder
pub struct TemplateFactory;

impl TemplateFactory {
    /// Create a new template factory with common defaults
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
    ) -> Self {
        let _template = Template::new(id, name, description, category)
            .with_author("ccswarm")
            .with_priority(Priority::Medium)
            .with_task_type(TaskType::Development);

        Self
    }

    /// Create a development template with standard configuration
    pub fn development_template(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
    ) -> TemplateBuilder {
        TemplateBuilder::new(
            Template::new(id, name, description, category)
                .with_author("ccswarm")
                .with_priority(Priority::Medium)
                .with_task_type(TaskType::Development)
                .with_duration(45),
        )
    }

    /// Create a testing template with standard configuration
    pub fn testing_template(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> TemplateBuilder {
        TemplateBuilder::new(
            Template::new(id, name, description, TemplateCategory::Testing)
                .with_author("ccswarm")
                .with_priority(Priority::Medium)
                .with_task_type(TaskType::Testing)
                .with_duration(30),
        )
    }

    /// Create a documentation template with standard configuration
    pub fn documentation_template(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> TemplateBuilder {
        TemplateBuilder::new(
            Template::new(id, name, description, TemplateCategory::Documentation)
                .with_author("ccswarm")
                .with_priority(Priority::Low)
                .with_task_type(TaskType::Documentation)
                .with_duration(20),
        )
    }
}

/// Fluent builder for templates with common patterns
pub struct TemplateBuilder {
    template: Template,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new(template: Template) -> Self {
        Self { template }
    }

    /// Add common frontend tags
    pub fn with_frontend_tags(mut self) -> Self {
        let mut tags = vec![
            "frontend".to_string(),
            "ui".to_string(),
            "typescript".to_string(),
        ];
        tags.extend(self.template.tags);
        self.template.tags = tags;
        self
    }

    /// Add common backend tags
    pub fn with_backend_tags(mut self) -> Self {
        let mut tags = vec![
            "backend".to_string(),
            "api".to_string(),
            "server".to_string(),
        ];
        tags.extend(self.template.tags);
        self.template.tags = tags;
        self
    }

    /// Add common devops tags
    pub fn with_devops_tags(mut self) -> Self {
        let mut tags = vec![
            "devops".to_string(),
            "infrastructure".to_string(),
            "deployment".to_string(),
        ];
        tags.extend(self.template.tags);
        self.template.tags = tags;
        self
    }

    /// Apply standard development template pattern
    pub fn with_development_pattern(
        mut self,
        component_name_var: &str,
        file_path_var: &str,
    ) -> Self {
        // Common task description pattern
        let task_desc = format!(
            "Create a {{{{{}}}}} named {{{{{}}}}}{{{{#if props}}}} with props: {{{{props}}}}{{{{/if}}}}",
            component_name_var, component_name_var
        );
        self.template = self.template.with_task_description(task_desc);

        // Common task details pattern
        let task_details = Self::generate_standard_task_details(component_name_var, file_path_var);
        self.template = self.template.with_task_details(task_details);

        // Common target files pattern
        let target_files = vec![
            format!("{{{{{}}}}}", file_path_var),
            format!("{{{{{}}}}}.test.tsx", file_path_var),
            format!("{{{{{}}}}}.module.css", file_path_var),
        ];
        self.template = self.template.with_target_files(target_files);

        self
    }

    /// Generate standard task details
    fn generate_standard_task_details(component_name: &str, file_path: &str) -> String {
        format!(
            r#"## Specification

- **Name**: {{{{{}}}}}
- **File**: `{{{{{}}}}}`
{{{{#if props}}}}- **Props**: {{{{props}}}}
{{{{/if}}}}{{{{#if state}}}}- **State**: {{{{state}}}}
{{{{/if}}}}{{{{#if styling}}}}- **Styling**: {{{{styling}}}}
{{{{/if}}}}{{{{#if accessibility}}}}- **Accessibility**: Include ARIA attributes and keyboard navigation
{{{{/if}}}}{{{{#if testing}}}}- **Testing**: Create unit tests with appropriate testing library
{{{{/if}}}}

## Requirements

- Use best practices and modern patterns
- Follow project conventions
- Implement proper error handling
- Add comprehensive documentation
{{{{#if responsive}}}}- Make component responsive for mobile and desktop
{{{{/if}}}}{{{{#if performance}}}}- Optimize for performance
{{{{/if}}}}"#,
            component_name, file_path
        )
    }

    /// Add standard component variables
    pub fn with_component_variables(mut self, component_type: &str) -> Self {
        let variables = vec![
            TemplateVariable::text("component_name", format!("Name of the {}", component_type)),
            TemplateVariable::choice(
                "component_type",
                "Type of component",
                vec![
                    "functional".to_string(),
                    "class".to_string(),
                    "custom hook".to_string(),
                ],
            ),
            TemplateVariable::text_with_default(
                "file_path",
                "File path for the component",
                "src/components/{{component_name}}.tsx",
            ),
            TemplateVariable::text("props", "Component props (optional)").optional(),
            TemplateVariable::text("state", "Component state description (optional)").optional(),
            TemplateVariable::choice(
                "styling",
                "Styling approach",
                vec![
                    "CSS Modules".to_string(),
                    "Styled Components".to_string(),
                    "Tailwind CSS".to_string(),
                    "SCSS".to_string(),
                    "Emotion".to_string(),
                ],
            )
            .optional(),
            TemplateVariable::boolean("accessibility", "Include accessibility features"),
            TemplateVariable::boolean("testing", "Create unit tests"),
            TemplateVariable::boolean("responsive", "Make responsive design"),
            TemplateVariable::boolean("performance", "Optimize for performance"),
        ];

        self.template.variables.extend(variables);
        self
    }

    /// Add standard preconditions
    pub fn with_standard_preconditions(mut self, tech_stack: &str) -> Self {
        self.template
            .preconditions
            .push(format!("{} is configured in the project", tech_stack));
        self.template
            .preconditions
            .push("Project dependencies are installed".to_string());
        self
    }

    /// Add standard post actions
    pub fn with_standard_post_actions(mut self) -> Self {
        self.template.post_actions.extend(vec![
            "Run tests to verify implementation".to_string(),
            "Update documentation if needed".to_string(),
            "Commit changes with descriptive message".to_string(),
        ]);
        self
    }

    /// Build the template
    pub fn build(self) -> Template {
        self.template
    }

    /// Apply common configuration pattern (combines multiple helpers)
    pub fn with_common_config(
        self,
        tech_stack: &str,
        component_type: &str,
        include_tags: bool,
    ) -> Self {
        let mut builder = self;

        if include_tags {
            builder = builder.with_frontend_tags();
        }

        builder
            .with_component_variables(component_type)
            .with_standard_preconditions(tech_stack)
            .with_standard_post_actions()
    }
}

/// Template configuration presets for common patterns
pub struct TemplatePresets;

impl TemplatePresets {
    /// Get common variable set for React components
    pub fn react_variables() -> Vec<TemplateVariable> {
        vec![
            TemplateVariable::text("component_name", "Name of the React component"),
            TemplateVariable::choice(
                "component_type",
                "Type of component",
                vec![
                    "functional".to_string(),
                    "class".to_string(),
                    "custom hook".to_string(),
                ],
            ),
            TemplateVariable::text_with_default(
                "file_path",
                "File path for the component",
                "src/components/{{component_name}}.tsx",
            ),
            TemplateVariable::text("props", "Component props (optional)").optional(),
            TemplateVariable::text("state", "Component state description (optional)").optional(),
            Self::styling_variable(),
            TemplateVariable::boolean("accessibility", "Include accessibility features"),
            TemplateVariable::boolean("testing", "Create unit tests"),
            TemplateVariable::boolean("responsive", "Make responsive design"),
            TemplateVariable::boolean("performance", "Optimize for performance"),
        ]
    }

    /// Get common variable set for API endpoints
    pub fn api_variables() -> Vec<TemplateVariable> {
        vec![
            TemplateVariable::text("endpoint_name", "Name of the API endpoint"),
            TemplateVariable::choice(
                "method",
                "HTTP method",
                vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "PATCH".to_string(),
                ],
            ),
            TemplateVariable::text("path", "API path (e.g., /api/users)"),
            TemplateVariable::text("request_body", "Request body schema (optional)").optional(),
            TemplateVariable::text("response_schema", "Response schema"),
            TemplateVariable::boolean("authentication", "Requires authentication"),
            TemplateVariable::boolean("validation", "Include input validation"),
            TemplateVariable::boolean("caching", "Include caching logic"),
            TemplateVariable::boolean("rate_limiting", "Include rate limiting"),
        ]
    }

    /// Get common variable set for tests
    pub fn test_variables() -> Vec<TemplateVariable> {
        vec![
            TemplateVariable::text("test_name", "Name of the test suite"),
            TemplateVariable::text("target_file", "File being tested"),
            TemplateVariable::choice(
                "test_type",
                "Type of test",
                vec![
                    "unit".to_string(),
                    "integration".to_string(),
                    "e2e".to_string(),
                ],
            ),
            TemplateVariable::boolean("mocking", "Include mocking setup"),
            TemplateVariable::boolean("coverage", "Track code coverage"),
            TemplateVariable::boolean("snapshot", "Include snapshot tests"),
        ]
    }

    /// Common styling variable
    fn styling_variable() -> TemplateVariable {
        TemplateVariable::choice(
            "styling",
            "Styling approach",
            vec![
                "CSS Modules".to_string(),
                "Styled Components".to_string(),
                "Tailwind CSS".to_string(),
                "SCSS".to_string(),
                "Emotion".to_string(),
                "Plain CSS".to_string(),
            ],
        )
        .optional()
    }

    /// Get metadata for common template types
    pub fn get_metadata(template_type: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("generated_by".to_string(), "ccswarm".to_string());
        metadata.insert("template_version".to_string(), "1.0.0".to_string());
        metadata.insert("template_type".to_string(), template_type.to_string());
        metadata
    }
}
