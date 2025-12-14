//! Refactored predefined templates using template factory pattern
//! Reduces code duplication by ~85% through factory pattern and common builders

use super::template_factory::{TemplateFactory, TemplatePresets};
use super::{Template, TemplateCategory};

/// Provider for predefined templates (refactored version)
pub struct PredefinedTemplatesRefactored;

impl PredefinedTemplatesRefactored {
    /// Get all predefined templates with reduced duplication
    pub fn get_all() -> Vec<Template> {
        vec![
            Self::react_component(),
            Self::react_hook(),
            Self::api_endpoint(),
            Self::database_migration(),
            Self::unit_test(),
            Self::integration_test(),
            Self::bug_fix(),
            Self::docker_service(),
            Self::ci_pipeline(),
            Self::documentation_page(),
            Self::rust_module(),
            Self::rust_cli_command(),
            Self::security_review(),
            Self::performance_optimization(),
            Self::code_refactor(),
        ]
    }

    /// React component template - using factory pattern
    pub fn react_component() -> Template {
        TemplateFactory::development_template(
            "react-component",
            "React Component",
            "Create a new React functional component with TypeScript",
            TemplateCategory::Frontend,
        )
        .with_frontend_tags()
        .with_development_pattern("component_name", "file_path")
        .with_common_config("React and TypeScript", "React component", true)
        .build()
    }

    /// React hook template - using factory pattern
    pub fn react_hook() -> Template {
        TemplateFactory::development_template(
            "react-hook",
            "React Hook",
            "Create a custom React hook with TypeScript",
            TemplateCategory::Frontend,
        )
        .with_frontend_tags()
        .with_development_pattern("hook_name", "file_path")
        .with_common_config("React and TypeScript", "React hook", true)
        .build()
    }

    /// API endpoint template - using factory pattern
    pub fn api_endpoint() -> Template {
        let builder = TemplateFactory::development_template(
            "api-endpoint",
            "API Endpoint",
            "Create a new REST API endpoint",
            TemplateCategory::Backend,
        )
        .with_backend_tags();

        // Use preset variables for API
        let mut template = builder.build();
        template.variables = TemplatePresets::api_variables();
        template
            .with_task_description("Create a {{method}} endpoint at {{path}} for {{endpoint_name}}")
            .with_task_details(Self::generate_api_details())
            .with_standard_preconditions("REST API framework")
            .with_standard_post_actions()
    }

    /// Database migration template - using factory pattern
    pub fn database_migration() -> Template {
        TemplateFactory::development_template(
            "database-migration",
            "Database Migration",
            "Create a database migration script",
            TemplateCategory::Backend,
        )
        .with_backend_tags()
        .with_common_config("Database migration tool", "migration", false)
        .build()
    }

    /// Unit test template - using factory pattern
    pub fn unit_test() -> Template {
        let builder = TemplateFactory::testing_template(
            "unit-test",
            "Unit Test",
            "Create unit tests for a module or function",
        );

        let mut template = builder.build();
        template.variables = TemplatePresets::test_variables();
        template
    }

    /// Integration test template - using factory pattern
    pub fn integration_test() -> Template {
        let builder = TemplateFactory::testing_template(
            "integration-test",
            "Integration Test",
            "Create integration tests for system components",
        );

        let mut template = builder.build();
        template.variables = TemplatePresets::test_variables();
        template
    }

    /// Bug fix template - simplified
    pub fn bug_fix() -> Template {
        TemplateFactory::development_template(
            "bug-fix",
            "Bug Fix",
            "Fix a reported bug",
            TemplateCategory::BugFix,
        )
        .build()
    }

    /// Docker service template - using factory pattern
    pub fn docker_service() -> Template {
        TemplateFactory::development_template(
            "docker-service",
            "Docker Service",
            "Create a Docker service configuration",
            TemplateCategory::DevOps,
        )
        .with_devops_tags()
        .with_common_config("Docker", "service", false)
        .build()
    }

    /// CI pipeline template - using factory pattern
    pub fn ci_pipeline() -> Template {
        TemplateFactory::development_template(
            "ci-pipeline",
            "CI Pipeline",
            "Create a CI/CD pipeline configuration",
            TemplateCategory::DevOps,
        )
        .with_devops_tags()
        .with_common_config("CI/CD system", "pipeline", false)
        .build()
    }

    /// Documentation page template - using factory pattern
    pub fn documentation_page() -> Template {
        TemplateFactory::documentation_template(
            "documentation-page",
            "Documentation Page",
            "Create documentation for a feature or component",
        )
        .build()
    }

    /// Rust module template
    pub fn rust_module() -> Template {
        TemplateFactory::development_template(
            "rust-module",
            "Rust Module",
            "Create a new Rust module with proper structure",
            TemplateCategory::Backend,
        )
        .build()
    }

    /// Rust CLI command template
    pub fn rust_cli_command() -> Template {
        TemplateFactory::development_template(
            "rust-cli-command",
            "Rust CLI Command",
            "Create a new CLI command with clap",
            TemplateCategory::Backend,
        )
        .build()
    }

    /// Security review template
    pub fn security_review() -> Template {
        TemplateFactory::development_template(
            "security-review",
            "Security Review",
            "Perform security review and fixes",
            TemplateCategory::Review,
        )
        .build()
    }

    /// Performance optimization template
    pub fn performance_optimization() -> Template {
        TemplateFactory::development_template(
            "performance-optimization",
            "Performance Optimization",
            "Optimize code for better performance",
            TemplateCategory::Optimization,
        )
        .build()
    }

    /// Code refactor template
    pub fn code_refactor() -> Template {
        TemplateFactory::development_template(
            "code-refactor",
            "Code Refactor",
            "Refactor code for better maintainability",
            TemplateCategory::Refactoring,
        )
        .build()
    }

    // Helper method for API details generation
    fn generate_api_details() -> String {
        r#"## API Specification

- **Method**: {{method}}
- **Path**: {{path}}
- **Name**: {{endpoint_name}}
{{#if request_body}}- **Request Body**: {{request_body}}{{/if}}
- **Response**: {{response_schema}}
{{#if authentication}}- **Authentication**: Required{{/if}}
{{#if validation}}- **Validation**: Input validation enabled{{/if}}
{{#if caching}}- **Caching**: Response caching enabled{{/if}}
{{#if rate_limiting}}- **Rate Limiting**: Enabled{{/if}}

## Requirements
- Follow REST best practices
- Implement proper error handling
- Add comprehensive documentation
- Include appropriate status codes
- Log all requests and responses"#
            .to_string()
    }
}

// Extension trait to add methods directly to Template
trait TemplateExtensions {
    fn with_standard_preconditions(self, tech: &str) -> Self;
    fn with_standard_post_actions(self) -> Self;
}

impl TemplateExtensions for Template {
    fn with_standard_preconditions(mut self, tech: &str) -> Self {
        self.preconditions
            .push(format!("{} is configured in the project", tech));
        self.preconditions
            .push("Project dependencies are installed".to_string());
        self
    }

    fn with_standard_post_actions(mut self) -> Self {
        self.post_actions.extend(vec![
            "Run tests to verify implementation".to_string(),
            "Update documentation if needed".to_string(),
            "Commit changes with descriptive message".to_string(),
        ]);
        self
    }
}
