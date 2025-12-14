//! Predefined templates for common development tasks

use super::{Template, TemplateCategory, TemplateVariable};
use crate::agent::{Priority, TaskType};

/// Provider for predefined templates
pub struct PredefinedTemplates;

impl PredefinedTemplates {
    /// Get all predefined templates
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

    /// React component template
    pub fn react_component() -> Template {
        Template::new(
            "react-component",
            "React Component",
            "Create a new React functional component with TypeScript",
            TemplateCategory::Frontend,
        )
        .with_author("ccswarm")
        .with_tags(vec!["react".to_string(), "frontend".to_string(), "typescript".to_string(), "component".to_string()])
        .with_task_description("Create a {{component_type}} React component named {{component_name}}{{#if props}} with props: {{props}}{{/if}}{{#if styling}} using {{styling}}{{/if}}")
        .with_task_details(
            "## Component Specification\n\n\
            - **Name**: {{component_name}}\n\
            - **Type**: {{component_type}}\n\
            - **File**: `{{file_path}}`\n\
            {{#if props}}- **Props**: {{props}}\n{{/if}}\
            {{#if state}}- **State**: {{state}}\n{{/if}}\
            {{#if styling}}- **Styling**: {{styling}}\n{{/if}}\
            {{#if accessibility}}- **Accessibility**: Include ARIA attributes and keyboard navigation\n{{/if}}\
            {{#if testing}}- **Testing**: Create unit tests with React Testing Library\n{{/if}}\n\n\
            ## Requirements\n\n\
            - Use TypeScript with proper type definitions\n\
            - Follow React best practices and hooks conventions\n\
            - Implement proper error boundaries if needed\n\
            - Add PropTypes or TypeScript interfaces for props\n\
            - Include JSDoc comments for component documentation\n\
            {{#if responsive}}- Make component responsive for mobile and desktop\n{{/if}}\
            {{#if performance}}- Optimize for performance with React.memo if appropriate\n{{/if}}"
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Development)
        .with_duration(45)
        .with_variables(vec![
            TemplateVariable::text("component_name", "Name of the React component"),
            TemplateVariable::choice(
                "component_type",
                "Type of component",
                vec!["functional".to_string(), "class".to_string(), "custom hook".to_string()],
            ),
            TemplateVariable::text_with_default(
                "file_path",
                "File path for the component",
                "src/components/{{component_name}}.tsx"
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
            ).optional(),
            TemplateVariable::boolean("accessibility", "Include accessibility features"),
            TemplateVariable::boolean("testing", "Create unit tests"),
            TemplateVariable::boolean("responsive", "Make responsive design"),
            TemplateVariable::boolean("performance", "Optimize for performance"),
        ])
        .with_target_files(vec![
            "{{file_path}}".to_string(),
            "{{file_path}}.test.tsx".to_string(),
            "{{file_path}}.module.css".to_string(),
        ])
        .with_preconditions(vec![
            "React and TypeScript are configured in the project".to_string(),
            "Component directory structure exists".to_string(),
        ])
        .with_post_actions(vec![
            "Export component from index file".to_string(),
            "Add to Storybook if available".to_string(),
            "Update documentation".to_string(),
        ])
    }

    /// React custom hook template
    pub fn react_hook() -> Template {
        Template::new(
            "react-hook",
            "React Custom Hook",
            "Create a custom React hook with TypeScript",
            TemplateCategory::Frontend,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "react".to_string(),
            "hook".to_string(),
            "typescript".to_string(),
        ])
        .with_task_description(
            "Create a custom React hook named {{hook_name}} for {{hook_purpose}}",
        )
        .with_task_details(
            "## Hook Specification\n\n\
            - **Name**: {{hook_name}}\n\
            - **Purpose**: {{hook_purpose}}\n\
            - **File**: `{{file_path}}`\n\
            {{#if parameters}}- **Parameters**: {{parameters}}\n{{/if}}\
            {{#if return_type}}- **Returns**: {{return_type}}\n{{/if}}\n\n\
            ## Implementation Requirements\n\n\
            - Use TypeScript with proper type definitions\n\
            - Follow React hooks rules and conventions\n\
            - Include proper dependency arrays for useEffect\n\
            - Add comprehensive JSDoc comments\n\
            - Handle cleanup and memory leaks\n\
            {{#if testing}}- Create unit tests with React Hooks Testing Library\n{{/if}}\
            {{#if memoization}}- Use useMemo/useCallback for performance optimization\n{{/if}}",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Development)
        .with_duration(30)
        .with_variables(vec![
            TemplateVariable::text(
                "hook_name",
                "Name of the custom hook (e.g., useLocalStorage)",
            ),
            TemplateVariable::text("hook_purpose", "What the hook does"),
            TemplateVariable::text_with_default(
                "file_path",
                "File path for the hook",
                "src/hooks/{{hook_name}}.ts",
            ),
            TemplateVariable::text("parameters", "Hook parameters (optional)").optional(),
            TemplateVariable::text("return_type", "Hook return type (optional)").optional(),
            TemplateVariable::boolean("testing", "Create unit tests"),
            TemplateVariable::boolean("memoization", "Include performance optimizations"),
        ])
        .with_target_files(vec![
            "{{file_path}}".to_string(),
            "{{file_path}}.test.ts".to_string(),
        ])
    }

    /// API endpoint template
    pub fn api_endpoint() -> Template {
        Template::new(
            "api-endpoint",
            "API Endpoint",
            "Create a new REST API endpoint with proper error handling",
            TemplateCategory::Backend,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "api".to_string(),
            "backend".to_string(),
            "rest".to_string(),
            "endpoint".to_string(),
        ])
        .with_task_description(
            "Create a {{http_method}} API endpoint for {{endpoint_path}} to {{endpoint_purpose}}",
        )
        .with_task_details(
            "## API Endpoint Specification\n\n\
            - **Method**: {{http_method}}\n\
            - **Path**: {{endpoint_path}}\n\
            - **Purpose**: {{endpoint_purpose}}\n\
            {{#if request_body}}- **Request Body**: {{request_body}}\n{{/if}}\
            {{#if response_format}}- **Response**: {{response_format}}\n{{/if}}\
            {{#if authentication}}- **Authentication**: {{authentication}}\n{{/if}}\n\n\
            ## Implementation Requirements\n\n\
            - Implement proper HTTP status codes\n\
            - Add comprehensive error handling\n\
            - Include request validation\n\
            - Add API documentation (OpenAPI/Swagger)\n\
            - Implement proper logging\n\
            {{#if database}}- Database operations with proper transactions\n{{/if}}\
            {{#if caching}}- Implement caching strategy\n{{/if}}\
            {{#if rate_limiting}}- Add rate limiting\n{{/if}}\
            {{#if testing}}- Create unit and integration tests\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Development)
        .with_duration(60)
        .with_variables(vec![
            TemplateVariable::choice(
                "http_method",
                "HTTP method",
                vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                ],
            ),
            TemplateVariable::text("endpoint_path", "API endpoint path (e.g., /api/users)"),
            TemplateVariable::text("endpoint_purpose", "What this endpoint does"),
            TemplateVariable::text("request_body", "Request body format (optional)").optional(),
            TemplateVariable::text("response_format", "Response format (optional)").optional(),
            TemplateVariable::choice(
                "authentication",
                "Authentication method",
                vec![
                    "None".to_string(),
                    "JWT Bearer".to_string(),
                    "API Key".to_string(),
                    "OAuth2".to_string(),
                    "Basic Auth".to_string(),
                ],
            )
            .optional(),
            TemplateVariable::boolean("database", "Includes database operations"),
            TemplateVariable::boolean("caching", "Implement caching"),
            TemplateVariable::boolean("rate_limiting", "Add rate limiting"),
            TemplateVariable::boolean("testing", "Create tests"),
        ])
        .with_target_files(vec![
            "src/routes/{{endpoint_path}}.rs".to_string(),
            "src/handlers/{{endpoint_path}}_handler.rs".to_string(),
            "tests/{{endpoint_path}}_test.rs".to_string(),
        ])
        .with_preconditions(vec![
            "API framework is set up (Express, Axum, etc.)".to_string(),
            "Database connection is configured if needed".to_string(),
        ])
    }

    /// Database migration template
    pub fn database_migration() -> Template {
        Template::new(
            "database-migration",
            "Database Migration",
            "Create a database migration script",
            TemplateCategory::Backend,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "database".to_string(),
            "migration".to_string(),
            "schema".to_string(),
        ])
        .with_task_description("Create database migration to {{migration_purpose}}")
        .with_task_details(
            "## Migration Details\n\n\
            - **Purpose**: {{migration_purpose}}\n\
            - **Type**: {{migration_type}}\n\
            {{#if table_name}}- **Table**: {{table_name}}\n{{/if}}\
            {{#if columns}}- **Columns**: {{columns}}\n{{/if}}\
            {{#if indexes}}- **Indexes**: {{indexes}}\n{{/if}}\n\n\
            ## Requirements\n\n\
            - Write both up and down migrations\n\
            - Include proper constraints and indexes\n\
            - Handle existing data appropriately\n\
            - Add comments for complex operations\n\
            - Test migration on sample data\n\
            {{#if backup}}- Create backup before running\n{{/if}}\
            {{#if rollback_plan}}- Document rollback procedure\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Infrastructure)
        .with_duration(40)
        .with_variables(vec![
            TemplateVariable::text("migration_purpose", "What this migration accomplishes"),
            TemplateVariable::choice(
                "migration_type",
                "Type of migration",
                vec![
                    "Create Table".to_string(),
                    "Alter Table".to_string(),
                    "Drop Table".to_string(),
                    "Add Column".to_string(),
                    "Drop Column".to_string(),
                    "Create Index".to_string(),
                    "Data Migration".to_string(),
                ],
            ),
            TemplateVariable::text("table_name", "Table name (if applicable)").optional(),
            TemplateVariable::text("columns", "Column definitions (optional)").optional(),
            TemplateVariable::text("indexes", "Index definitions (optional)").optional(),
            TemplateVariable::boolean("backup", "Create backup before migration"),
            TemplateVariable::boolean("rollback_plan", "Document rollback procedure"),
        ])
        .with_target_files(vec![
            "migrations/{{current_datetime}}_{{migration_purpose}}.sql".to_string(),
        ])
    }

    /// Unit test template
    pub fn unit_test() -> Template {
        Template::new(
            "unit-test",
            "Unit Test",
            "Create comprehensive unit tests for a function or module",
            TemplateCategory::Testing,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "testing".to_string(),
            "unit-test".to_string(),
            "quality".to_string(),
        ])
        .with_task_description("Create unit tests for {{test_target}} covering {{test_scenarios}}")
        .with_task_details(
            "## Test Specification\n\n\
            - **Target**: {{test_target}}\n\
            - **Test Framework**: {{test_framework}}\n\
            - **Scenarios**: {{test_scenarios}}\n\
            {{#if mocking}}- **Mocking**: {{mocking_strategy}}\n{{/if}}\n\n\
            ## Test Coverage Requirements\n\n\
            - Happy path scenarios\n\
            - Error conditions and edge cases\n\
            - Boundary value testing\n\
            - Input validation testing\n\
            {{#if async_code}}- Async operation testing\n{{/if}}\
            {{#if performance}}- Performance benchmarks\n{{/if}}\
            - Achieve >90% code coverage for the target",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Testing)
        .with_duration(35)
        .with_variables(vec![
            TemplateVariable::text("test_target", "Function, class, or module to test"),
            TemplateVariable::text("test_scenarios", "Specific scenarios to test"),
            TemplateVariable::choice(
                "test_framework",
                "Testing framework",
                vec![
                    "Jest".to_string(),
                    "Vitest".to_string(),
                    "Mocha".to_string(),
                    "pytest".to_string(),
                    "Rust test".to_string(),
                    "Go test".to_string(),
                ],
            ),
            TemplateVariable::choice(
                "mocking_strategy",
                "Mocking approach",
                vec![
                    "Manual mocks".to_string(),
                    "Jest mocks".to_string(),
                    "Sinon".to_string(),
                    "unittest.mock".to_string(),
                    "mockall".to_string(),
                ],
            )
            .optional(),
            TemplateVariable::boolean("async_code", "Testing async operations"),
            TemplateVariable::boolean("performance", "Include performance tests"),
        ])
        .with_target_files(vec![
            "tests/{{test_target}}.test.{{file_extension}}".to_string(),
        ])
    }

    /// Integration test template
    pub fn integration_test() -> Template {
        Template::new(
            "integration-test",
            "Integration Test",
            "Create integration tests for system components",
            TemplateCategory::Testing,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "testing".to_string(),
            "integration".to_string(),
            "e2e".to_string(),
        ])
        .with_task_description(
            "Create integration tests for {{system_component}} testing {{integration_points}}",
        )
        .with_task_details(
            "## Integration Test Specification\n\n\
            - **Component**: {{system_component}}\n\
            - **Integration Points**: {{integration_points}}\n\
            - **Test Environment**: {{test_environment}}\n\
            {{#if database}}- **Database**: Test database setup required\n{{/if}}\
            {{#if external_services}}- **External Services**: {{external_services}}\n{{/if}}\n\n\
            ## Test Requirements\n\n\
            - Set up test environment and data\n\
            - Test end-to-end workflows\n\
            - Verify data consistency across components\n\
            - Test error propagation and recovery\n\
            - Clean up test data after execution\n\
            {{#if performance}}- Measure integration performance\n{{/if}}\
            {{#if security}}- Verify security boundaries\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Testing)
        .with_duration(75)
        .with_variables(vec![
            TemplateVariable::text("system_component", "System component being tested"),
            TemplateVariable::text("integration_points", "What integrations to test"),
            TemplateVariable::choice(
                "test_environment",
                "Test environment",
                vec![
                    "Docker Compose".to_string(),
                    "Testcontainers".to_string(),
                    "In-memory".to_string(),
                    "Staging".to_string(),
                ],
            ),
            TemplateVariable::boolean("database", "Includes database testing"),
            TemplateVariable::text("external_services", "External services involved").optional(),
            TemplateVariable::boolean("performance", "Include performance testing"),
            TemplateVariable::boolean("security", "Include security testing"),
        ])
        .with_target_files(vec![
            "tests/integration/{{system_component}}_integration.test.{{file_extension}}"
                .to_string(),
            "tests/fixtures/{{system_component}}_data.json".to_string(),
        ])
    }

    /// Bug fix template
    pub fn bug_fix() -> Template {
        Template::new(
            "bug-fix",
            "Bug Fix",
            "Fix a reported bug with proper testing and documentation",
            TemplateCategory::BugFix,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "bug".to_string(),
            "fix".to_string(),
            "maintenance".to_string(),
        ])
        .with_task_description("Fix bug: {{bug_description}} ({{bug_severity}})")
        .with_task_details(
            "## Bug Report\n\n\
            - **Description**: {{bug_description}}\n\
            - **Severity**: {{bug_severity}}\n\
            {{#if reproduction_steps}}- **Reproduction**: {{reproduction_steps}}\n{{/if}}\
            {{#if affected_components}}- **Affected Components**: {{affected_components}}\n{{/if}}\
            {{#if user_impact}}- **User Impact**: {{user_impact}}\n{{/if}}\n\n\
            ## Fix Requirements\n\n\
            - Reproduce the bug in a test environment\n\
            - Identify root cause through debugging\n\
            - Implement minimal fix that addresses root cause\n\
            - Add regression tests to prevent recurrence\n\
            - Verify fix doesn't introduce new issues\n\
            {{#if documentation}}- Update relevant documentation\n{{/if}}\
            {{#if release_notes}}- Add to release notes\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Bugfix)
        .with_duration(90)
        .with_variables(vec![
            TemplateVariable::text("bug_description", "Clear description of the bug"),
            TemplateVariable::choice(
                "bug_severity",
                "Bug severity level",
                vec![
                    "Critical".to_string(),
                    "High".to_string(),
                    "Medium".to_string(),
                    "Low".to_string(),
                ],
            ),
            TemplateVariable::text("reproduction_steps", "Steps to reproduce the bug").optional(),
            TemplateVariable::text("affected_components", "Components affected by the bug")
                .optional(),
            TemplateVariable::text("user_impact", "How this affects users").optional(),
            TemplateVariable::boolean("documentation", "Update documentation"),
            TemplateVariable::boolean("release_notes", "Add to release notes"),
        ])
        .with_target_files(vec![
            "src/{{affected_components}}/fix.{{file_extension}}".to_string(),
            "tests/regression/{{bug_description}}.test.{{file_extension}}".to_string(),
        ])
    }

    /// Docker service template
    pub fn docker_service() -> Template {
        Template::new(
            "docker-service",
            "Docker Service",
            "Create a Docker container configuration for a service",
            TemplateCategory::DevOps,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "docker".to_string(),
            "container".to_string(),
            "devops".to_string(),
        ])
        .with_task_description("Create Docker configuration for {{service_name}} service")
        .with_task_details(
            "## Docker Service Configuration\n\n\
            - **Service**: {{service_name}}\n\
            - **Base Image**: {{base_image}}\n\
            {{#if ports}}- **Ports**: {{ports}}\n{{/if}}\
            {{#if volumes}}- **Volumes**: {{volumes}}\n{{/if}}\
            {{#if environment}}- **Environment**: {{environment}}\n{{/if}}\n\n\
            ## Container Requirements\n\n\
            - Multi-stage build for optimization\n\
            - Non-root user for security\n\
            - Health check configuration\n\
            - Proper signal handling\n\
            - Minimal attack surface\n\
            {{#if secrets}}- Secure secret management\n{{/if}}\
            {{#if monitoring}}- Monitoring and logging setup\n{{/if}}",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Infrastructure)
        .with_duration(45)
        .with_variables(vec![
            TemplateVariable::text("service_name", "Name of the service"),
            TemplateVariable::choice(
                "base_image",
                "Base Docker image",
                vec![
                    "alpine".to_string(),
                    "ubuntu".to_string(),
                    "node:alpine".to_string(),
                    "python:alpine".to_string(),
                    "rust:alpine".to_string(),
                    "nginx:alpine".to_string(),
                ],
            ),
            TemplateVariable::text("ports", "Port mappings (e.g., 8080:8080)").optional(),
            TemplateVariable::text("volumes", "Volume mounts").optional(),
            TemplateVariable::text("environment", "Environment variables").optional(),
            TemplateVariable::boolean("secrets", "Include secret management"),
            TemplateVariable::boolean("monitoring", "Add monitoring setup"),
        ])
        .with_target_files(vec![
            "Dockerfile".to_string(),
            "docker-compose.yml".to_string(),
            ".dockerignore".to_string(),
        ])
    }

    /// CI pipeline template
    pub fn ci_pipeline() -> Template {
        Template::new(
            "ci-pipeline",
            "CI/CD Pipeline",
            "Create a continuous integration pipeline",
            TemplateCategory::DevOps,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "ci".to_string(),
            "cd".to_string(),
            "pipeline".to_string(),
            "automation".to_string(),
        ])
        .with_task_description("Create {{ci_platform}} pipeline for {{project_type}} project")
        .with_task_details(
            "## CI/CD Pipeline Configuration\n\n\
            - **Platform**: {{ci_platform}}\n\
            - **Project Type**: {{project_type}}\n\
            {{#if triggers}}- **Triggers**: {{triggers}}\n{{/if}}\
            {{#if environments}}- **Environments**: {{environments}}\n{{/if}}\n\n\
            ## Pipeline Stages\n\n\
            - Code checkout and setup\n\
            - Dependency installation\n\
            - Linting and code quality checks\n\
            - Unit and integration tests\n\
            - Build and package\n\
            {{#if security_scan}}- Security vulnerability scanning\n{{/if}}\
            {{#if deployment}}- Automated deployment to {{environments}}\n{{/if}}\
            {{#if notifications}}- Notification on success/failure\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Infrastructure)
        .with_duration(60)
        .with_variables(vec![
            TemplateVariable::choice(
                "ci_platform",
                "CI/CD platform",
                vec![
                    "GitHub Actions".to_string(),
                    "GitLab CI".to_string(),
                    "Jenkins".to_string(),
                    "CircleCI".to_string(),
                    "Azure DevOps".to_string(),
                ],
            ),
            TemplateVariable::choice(
                "project_type",
                "Project type",
                vec![
                    "Node.js".to_string(),
                    "Python".to_string(),
                    "Rust".to_string(),
                    "Go".to_string(),
                    "Java".to_string(),
                    "Docker".to_string(),
                ],
            ),
            TemplateVariable::text("triggers", "Pipeline triggers (e.g., push, PR)").optional(),
            TemplateVariable::text("environments", "Deployment environments").optional(),
            TemplateVariable::boolean("security_scan", "Include security scanning"),
            TemplateVariable::boolean("deployment", "Include deployment stages"),
            TemplateVariable::boolean("notifications", "Set up notifications"),
        ])
        .with_target_files(vec![
            ".github/workflows/ci.yml".to_string(),
            ".github/workflows/cd.yml".to_string(),
        ])
    }

    /// Documentation page template
    pub fn documentation_page() -> Template {
        Template::new(
            "documentation-page",
            "Documentation Page",
            "Create comprehensive documentation for a feature or API",
            TemplateCategory::Documentation,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "documentation".to_string(),
            "docs".to_string(),
            "guide".to_string(),
        ])
        .with_task_description(
            "Create documentation for {{doc_subject}} explaining {{doc_purpose}}",
        )
        .with_task_details(
            "## Documentation Specification\n\n\
            - **Subject**: {{doc_subject}}\n\
            - **Purpose**: {{doc_purpose}}\n\
            - **Audience**: {{target_audience}}\n\
            {{#if doc_type}}- **Type**: {{doc_type}}\n{{/if}}\n\n\
            ## Content Requirements\n\n\
            - Clear overview and introduction\n\
            - Step-by-step instructions or usage guide\n\
            - Code examples with explanations\n\
            - Common pitfalls and troubleshooting\n\
            - Links to related documentation\n\
            {{#if api_reference}}- Complete API reference with parameters\n{{/if}}\
            {{#if diagrams}}- Architecture or flow diagrams\n{{/if}}\
            {{#if examples}}- Real-world usage examples\n{{/if}}",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Documentation)
        .with_duration(90)
        .with_variables(vec![
            TemplateVariable::text("doc_subject", "What is being documented"),
            TemplateVariable::text("doc_purpose", "Purpose of the documentation"),
            TemplateVariable::choice(
                "target_audience",
                "Target audience",
                vec![
                    "Developers".to_string(),
                    "Users".to_string(),
                    "System Administrators".to_string(),
                    "Product Managers".to_string(),
                ],
            ),
            TemplateVariable::choice(
                "doc_type",
                "Documentation type",
                vec![
                    "API Reference".to_string(),
                    "User Guide".to_string(),
                    "Tutorial".to_string(),
                    "Architecture Guide".to_string(),
                    "Troubleshooting".to_string(),
                ],
            )
            .optional(),
            TemplateVariable::boolean("api_reference", "Include API reference"),
            TemplateVariable::boolean("diagrams", "Include diagrams"),
            TemplateVariable::boolean("examples", "Include code examples"),
        ])
        .with_target_files(vec![
            "docs/{{doc_subject}}.md".to_string(),
            "examples/{{doc_subject}}_example.{{file_extension}}".to_string(),
        ])
    }

    /// Rust module template
    pub fn rust_module() -> Template {
        Template::new(
            "rust-module",
            "Rust Module",
            "Create a new Rust module with proper structure and documentation",
            TemplateCategory::Backend,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "rust".to_string(),
            "module".to_string(),
            "backend".to_string(),
        ])
        .with_task_description("Create Rust module {{module_name}} for {{module_purpose}}")
        .with_task_details(
            "## Rust Module Specification\n\n\
            - **Module**: {{module_name}}\n\
            - **Purpose**: {{module_purpose}}\n\
            - **Visibility**: {{visibility}}\n\
            {{#if traits}}- **Traits**: {{traits}}\n{{/if}}\
            {{#if structs}}- **Structs**: {{structs}}\n{{/if}}\n\n\
            ## Implementation Requirements\n\n\
            - Follow Rust naming conventions\n\
            - Add comprehensive rustdoc comments\n\
            - Implement proper error handling with Result<T, E>\n\
            - Use appropriate visibility modifiers",
        )
    }

    /// Rust CLI command template
    pub fn rust_cli_command() -> Template {
        Template::new(
            "rust-cli-command",
            "Rust CLI Command",
            "Create a new CLI command with clap for the Rust application",
            TemplateCategory::Backend,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "rust".to_string(),
            "cli".to_string(),
            "command".to_string(),
            "clap".to_string(),
        ])
        .with_task_description("Create CLI command {{command_name}} to {{command_purpose}}")
        .with_task_details(
            "## CLI Command Specification\n\n\
            - **Command**: {{command_name}}\n\
            - **Purpose**: {{command_purpose}}\n\
            {{#if subcommands}}- **Subcommands**: {{subcommands}}\n{{/if}}\
            {{#if arguments}}- **Arguments**: {{arguments}}\n{{/if}}\
            {{#if options}}- **Options**: {{options}}\n{{/if}}\n\n\
            ## Implementation Requirements\n\n\
            - Use clap for argument parsing\n\
            - Implement proper error handling\n\
            - Add comprehensive help text\n\
            - Follow CLI best practices\n\
            - Include command examples in documentation",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Development)
        .with_duration(45)
    }

    /// Security review template
    pub fn security_review() -> Template {
        Template::new(
            "security-review",
            "Security Review",
            "Conduct a security review of the codebase or component",
            TemplateCategory::Review,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "security".to_string(),
            "review".to_string(),
            "audit".to_string(),
            "vulnerability".to_string(),
        ])
        .with_task_description(
            "Perform security review of {{review_target}} focusing on {{security_aspects}}",
        )
        .with_task_details(
            "## Security Review Scope\n\n\
            - **Target**: {{review_target}}\n\
            - **Security Aspects**: {{security_aspects}}\n\
            {{#if threat_model}}- **Threat Model**: {{threat_model}}\n{{/if}}\n\n\
            ## Review Checklist\n\n\
            - Authentication and authorization checks\n\
            - Input validation and sanitization\n\
            - SQL injection and XSS prevention\n\
            - Secure data storage and transmission\n\
            - Error handling and information disclosure\n\
            - Dependencies vulnerability scan\n\
            - Access control and privilege escalation\n\
            {{#if penetration_test}}- Basic penetration testing\n{{/if}}\
            {{#if compliance}}- Compliance requirements check\n{{/if}}",
        )
        .with_priority(Priority::High)
        .with_task_type(TaskType::Review)
        .with_duration(120)
    }

    /// Performance optimization template
    pub fn performance_optimization() -> Template {
        Template::new(
            "performance-optimization",
            "Performance Optimization",
            "Optimize code or system performance based on profiling results",
            TemplateCategory::Optimization,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "performance".to_string(),
            "optimization".to_string(),
            "profiling".to_string(),
            "speed".to_string(),
        ])
        .with_task_description("Optimize {{optimization_target}} to improve {{performance_metric}}")
        .with_task_details(
            "## Optimization Specification\n\n\
            - **Target**: {{optimization_target}}\n\
            - **Performance Metric**: {{performance_metric}}\n\
            - **Current Performance**: {{current_performance}}\n\
            - **Target Performance**: {{target_performance}}\n\
            {{#if bottlenecks}}- **Known Bottlenecks**: {{bottlenecks}}\n{{/if}}\n\n\
            ## Optimization Strategy\n\n\
            - Profile and measure current performance\n\
            - Identify performance bottlenecks\n\
            - Implement targeted optimizations\n\
            - Measure improvement after each change\n\
            - Document performance gains\n\
            {{#if caching}}- Implement caching strategies\n{{/if}}\
            {{#if database}}- Optimize database queries and indexes\n{{/if}}\
            {{#if algorithm}}- Improve algorithmic complexity\n{{/if}}",
        )
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Development)
        .with_duration(90)
    }

    /// Code refactor template
    pub fn code_refactor() -> Template {
        Template::new(
            "code-refactor",
            "Code Refactoring",
            "Refactor code to improve maintainability and readability",
            TemplateCategory::Refactoring,
        )
        .with_author("ccswarm")
        .with_tags(vec![
            "refactor".to_string(),
            "clean-code".to_string(),
            "maintenance".to_string(),
            "technical-debt".to_string(),
        ])
        .with_task_description("Refactor {{refactor_target}} to {{refactor_goal}}")
        .with_task_details(
            "## Refactoring Specification\n\n\
            - **Target**: {{refactor_target}}\n\
            - **Goal**: {{refactor_goal}}\n\
            {{#if code_smells}}- **Code Smells**: {{code_smells}}\n{{/if}}\
            {{#if design_patterns}}- **Design Patterns**: {{design_patterns}}\n{{/if}}\n\n\
            ## Refactoring Requirements\n\n\
            - Maintain existing functionality (no behavior changes)\n\
            - Improve code readability and structure\n\
            - Reduce complexity and duplication\n\
            - Add or improve documentation\n\
            - Ensure all tests pass after refactoring\n\
            {{#if extract_methods}}- Extract methods for complex logic\n{{/if}}\
            {{#if rename_variables}}- Rename variables for clarity\n{{/if}}\
            {{#if split_modules}}- Split large modules into smaller ones\n{{/if}}",
        )
        .with_priority(Priority::Low)
        .with_task_type(TaskType::Development)
        .with_duration(60)
    }
}
