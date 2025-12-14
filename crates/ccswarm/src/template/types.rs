//! Core template types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Template category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemplateCategory {
    /// Frontend development templates
    Frontend,
    /// Backend development templates
    Backend,
    /// DevOps and infrastructure templates
    DevOps,
    /// Testing and QA templates
    Testing,
    /// Documentation templates
    Documentation,
    /// Bug fix templates
    BugFix,
    /// Code review templates
    Review,
    /// Performance optimization templates
    Optimization,
    /// Code refactoring templates
    Refactoring,
    /// General development templates
    General,
    /// Custom user-defined category
    Custom(String),
}

impl FromStr for TemplateCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "frontend" | "front" | "ui" => Ok(TemplateCategory::Frontend),
            "backend" | "back" | "api" => Ok(TemplateCategory::Backend),
            "devops" | "ops" | "infra" | "infrastructure" => Ok(TemplateCategory::DevOps),
            "testing" | "test" | "qa" => Ok(TemplateCategory::Testing),
            "documentation" | "docs" | "doc" => Ok(TemplateCategory::Documentation),
            "bugfix" | "bug" | "fix" => Ok(TemplateCategory::BugFix),
            "review" | "code-review" => Ok(TemplateCategory::Review),
            "optimization" | "optimize" | "performance" => Ok(TemplateCategory::Optimization),
            "refactoring" | "refactor" => Ok(TemplateCategory::Refactoring),
            "general" | "misc" => Ok(TemplateCategory::General),
            _ => Ok(TemplateCategory::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::Frontend => write!(f, "Frontend"),
            TemplateCategory::Backend => write!(f, "Backend"),
            TemplateCategory::DevOps => write!(f, "DevOps"),
            TemplateCategory::Testing => write!(f, "Testing"),
            TemplateCategory::Documentation => write!(f, "Documentation"),
            TemplateCategory::BugFix => write!(f, "BugFix"),
            TemplateCategory::Review => write!(f, "Review"),
            TemplateCategory::Optimization => write!(f, "Optimization"),
            TemplateCategory::Refactoring => write!(f, "Refactoring"),
            TemplateCategory::General => write!(f, "General"),
            TemplateCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Variable type for template substitution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableType {
    /// Simple text variable
    Text,
    /// Boolean variable
    Boolean,
    /// Numeric variable
    Number,
    /// File path variable
    FilePath,
    /// URL variable
    Url,
    /// List of values
    List,
    /// Enum with predefined choices
    Choice(Vec<String>),
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (used in templates as {{name}})
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Variable type
    pub variable_type: VariableType,
    /// Default value if not provided
    pub default_value: Option<String>,
    /// Whether this variable is required
    pub required: bool,
    /// Example value for documentation
    pub example: Option<String>,
    /// Validation pattern (regex)
    pub validation_pattern: Option<String>,
}

impl TemplateVariable {
    /// Create a new text variable
    pub fn text(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            variable_type: VariableType::Text,
            default_value: None,
            required: true,
            example: None,
            validation_pattern: None,
        }
    }

    /// Create a new optional text variable with default
    pub fn text_with_default(
        name: impl Into<String>,
        description: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            variable_type: VariableType::Text,
            default_value: Some(default.into()),
            required: false,
            example: None,
            validation_pattern: None,
        }
    }

    /// Create a choice variable
    pub fn choice(
        name: impl Into<String>,
        description: impl Into<String>,
        choices: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            variable_type: VariableType::Choice(choices),
            default_value: None,
            required: true,
            example: None,
            validation_pattern: None,
        }
    }

    /// Create a boolean variable
    pub fn boolean(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            variable_type: VariableType::Boolean,
            default_value: Some("false".to_string()),
            required: false,
            example: Some("true".to_string()),
            validation_pattern: None,
        }
    }

    /// Make variable optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Add example value
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Add validation pattern
    pub fn with_validation(mut self, pattern: impl Into<String>) -> Self {
        self.validation_pattern = Some(pattern.into());
        self
    }
}

/// A task template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Unique template identifier
    pub id: String,
    /// Human-readable template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template category
    pub category: TemplateCategory,
    /// Author information
    pub author: Option<String>,
    /// Template version
    pub version: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Tags for searchability
    pub tags: Vec<String>,

    // Template content
    /// Task description template with variables
    pub task_description: String,
    /// Task details template (optional)
    pub task_details: Option<String>,
    /// Default priority for tasks created from this template
    pub default_priority: crate::agent::Priority,
    /// Default task type
    pub default_task_type: crate::agent::TaskType,
    /// Estimated duration in minutes
    pub estimated_duration: Option<u32>,

    // Variables and customization
    /// Variables that can be substituted in the template
    pub variables: Vec<TemplateVariable>,
    /// Files that will be created/modified (with variable substitution)
    pub target_files: Vec<String>,
    /// Pre-conditions that should be met before using this template
    pub preconditions: Vec<String>,
    /// Post-template actions or instructions
    pub post_actions: Vec<String>,

    // Metadata
    /// Usage count for popularity tracking
    pub usage_count: u64,
    /// Average success rate (0.0 to 1.0)
    pub success_rate: Option<f64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Template {
    /// Create a new template
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            author: None,
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            task_description: String::new(),
            task_details: None,
            default_priority: crate::agent::Priority::Medium,
            default_task_type: crate::agent::TaskType::Development,
            estimated_duration: None,
            variables: Vec::new(),
            target_files: Vec::new(),
            preconditions: Vec::new(),
            post_actions: Vec::new(),
            usage_count: 0,
            success_rate: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set task description template
    pub fn with_task_description(mut self, description: impl Into<String>) -> Self {
        self.task_description = description.into();
        self
    }

    /// Set task details template
    pub fn with_task_details(mut self, details: impl Into<String>) -> Self {
        self.task_details = Some(details.into());
        self
    }

    /// Set default priority
    pub fn with_priority(mut self, priority: crate::agent::Priority) -> Self {
        self.default_priority = priority;
        self
    }

    /// Set default task type
    pub fn with_task_type(mut self, task_type: crate::agent::TaskType) -> Self {
        self.default_task_type = task_type;
        self
    }

    /// Set estimated duration
    pub fn with_duration(mut self, minutes: u32) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }

    /// Add variables
    pub fn with_variables(mut self, variables: Vec<TemplateVariable>) -> Self {
        self.variables = variables;
        self
    }

    /// Add target files
    pub fn with_target_files(mut self, files: Vec<String>) -> Self {
        self.target_files = files;
        self
    }

    /// Add preconditions
    pub fn with_preconditions(mut self, preconditions: Vec<String>) -> Self {
        self.preconditions = preconditions;
        self
    }

    /// Add post actions
    pub fn with_post_actions(mut self, actions: Vec<String>) -> Self {
        self.post_actions = actions;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Increment usage count
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
        self.updated_at = chrono::Utc::now();
    }

    /// Update success rate
    pub fn update_success_rate(&mut self, success: bool) {
        let current_rate = self.success_rate.unwrap_or(0.0);
        let current_count = self.usage_count as f64;

        let new_rate = if current_count == 0.0 {
            if success { 1.0 } else { 0.0 }
        } else {
            let total_successes = current_rate * current_count;
            let new_successes = if success {
                total_successes + 1.0
            } else {
                total_successes
            };
            new_successes / (current_count + 1.0)
        };

        self.success_rate = Some(new_rate);
        self.updated_at = chrono::Utc::now();
    }

    /// Get all variable names used in this template
    pub fn get_variable_names(&self) -> Vec<String> {
        self.variables.iter().map(|v| v.name.clone()).collect()
    }

    /// Check if template has all required variables defined
    pub fn is_valid(&self) -> bool {
        !self.task_description.is_empty() && !self.name.is_empty() && !self.id.is_empty()
    }
}

/// Template search criteria
#[derive(Debug, Clone, Default)]
pub struct TemplateQuery {
    /// Search by name or description
    pub search_term: Option<String>,
    /// Filter by category
    pub category: Option<TemplateCategory>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by author
    pub author: Option<String>,
    /// Minimum success rate
    pub min_success_rate: Option<f64>,
    /// Sort by popularity (usage count)
    pub sort_by_popularity: bool,
    /// Sort by success rate
    pub sort_by_success_rate: bool,
    /// Sort by creation date
    pub sort_by_date: bool,
    /// Limit number of results
    pub limit: Option<usize>,
}

/// Template context for variable substitution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateContext {
    /// Variables to substitute in the template
    pub variables: HashMap<String, String>,
    /// Additional context data
    pub metadata: HashMap<String, String>,
    /// Project name for templates
    pub project_name: Option<String>,
    /// Agent role for context-specific templates
    pub agent_role: Option<String>,
}

impl TemplateContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context with variables
    pub fn with_variables(variables: HashMap<String, String>) -> Self {
        Self {
            variables,
            metadata: HashMap::new(),
            project_name: None,
            agent_role: None,
        }
    }

    /// Add a variable to the context (builder pattern)
    pub fn with_variable(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Add a variable to the context
    pub fn add_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Add metadata to the context
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Template-specific error types
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// Template not found
    #[error("Template not found: {id}")]
    NotFound { id: String },

    /// Template already exists
    #[error("Template already exists: {id}")]
    AlreadyExists { id: String },

    /// Template validation failed
    #[error("Template validation failed: {reason}")]
    ValidationFailed { reason: String },

    /// Template storage error
    #[error("Template storage error: {0}")]
    StorageError(#[from] std::io::Error),

    /// Template parsing error
    #[error("Template parsing error: {0}")]
    ParseError(String),

    /// Variable substitution error
    #[error("Variable substitution error: missing variable '{name}'")]
    MissingVariable { name: String },

    /// Substitution failed
    #[error("Template substitution failed: {reason}")]
    SubstitutionFailed { reason: String },

    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl TemplateQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Search by term
    pub fn with_search_term(mut self, term: impl Into<String>) -> Self {
        self.search_term = Some(term.into());
        self
    }

    /// Filter by category
    pub fn with_category(mut self, category: TemplateCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Filter by author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set minimum success rate
    pub fn with_min_success_rate(mut self, rate: f64) -> Self {
        self.min_success_rate = Some(rate);
        self
    }

    /// Sort by popularity
    pub fn sort_by_popularity(mut self) -> Self {
        self.sort_by_popularity = true;
        self
    }

    /// Sort by success rate
    pub fn sort_by_success_rate(mut self) -> Self {
        self.sort_by_success_rate = true;
        self
    }

    /// Sort by date
    pub fn sort_by_date(mut self) -> Self {
        self.sort_by_date = true;
        self
    }

    /// Limit results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}
