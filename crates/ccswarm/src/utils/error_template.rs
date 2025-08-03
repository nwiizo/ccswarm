//! Template engine for error diagrams to eliminate duplication

use colored::Colorize;
use std::collections::HashMap;

/// Template engine for generating error diagrams
pub struct ErrorTemplateEngine {
    templates: HashMap<String, String>,
}

impl ErrorTemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };
        engine.register_templates();
        engine
    }
    
    /// Register all error diagram templates
    fn register_templates(&mut self) {
        // Generic box template
        self.templates.insert("box".to_string(), r#"
    ┌─────────────────────────────────────────────────────────┐
    │                    {{title}}                             │
    └─────────────────────────────────────────────────────────┘
    {{content}}
    {{footer}}
"#.to_string());

        // Flow diagram template
        self.templates.insert("flow".to_string(), r#"
    {{#each steps}}
    {{#if @first}}┌─────────┐{{else}}      {{/if}}
    │  {{name}} │{{#unless @last}} ──▶{{/unless}}
    └─────────┘
         │
         ▼
      {{status}}
    {{/each}}
"#.to_string());

        // Network diagram template
        self.templates.insert("network".to_string(), r#"
    ┌─────────────┐       ┌──────────────┐       ┌─────────────┐
    │   {{source}} │  {{s1}}  │  {{middle}}  │  {{s2}}  │   {{dest}}   │
    │  {{s_type}}  │──────▶│   {{m_type}} │──────▶│  {{d_type}} │
    └─────────────┘       └──────────────┘       └─────────────┘
"#.to_string());
    }
    
    /// Render a template with variables
    pub fn render(&self, template_name: &str, vars: HashMap<&str, String>) -> String {
        let template = self.templates.get(template_name)
            .unwrap_or(&String::from("Template not found"));
        
        let mut result = template.clone();
        
        // Simple variable substitution
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), &value);
        }
        
        result
    }
}

/// Builder for error diagrams
pub struct ErrorDiagramBuilder {
    engine: ErrorTemplateEngine,
    vars: HashMap<String, String>,
}

impl ErrorDiagramBuilder {
    pub fn new() -> Self {
        Self {
            engine: ErrorTemplateEngine::new(),
            vars: HashMap::new(),
        }
    }
    
    pub fn title(mut self, title: &str, color: colored::Color) -> Self {
        self.vars.insert("title".to_string(), title.color(color).bold().to_string());
        self
    }
    
    pub fn content(mut self, content: &str) -> Self {
        self.vars.insert("content".to_string(), content.to_string());
        self
    }
    
    pub fn footer(mut self, footer: &str) -> Self {
        self.vars.insert("footer".to_string(), footer.to_string());
        self
    }
    
    pub fn var(mut self, key: &str, value: String) -> Self {
        self.vars.insert(key.to_string(), value);
        self
    }
    
    pub fn build(self, template: &str) -> String {
        let vars_ref: HashMap<&str, String> = self.vars.iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        self.engine.render(template, vars_ref)
    }
}

/// Macro to generate all error diagrams with minimal code
#[macro_export]
macro_rules! error_diagram {
    ($name:ident, $template:expr, $title:expr, $color:expr, $($key:ident => $value:expr),*) => {
        pub fn $name() -> String {
            ErrorDiagramBuilder::new()
                .title($title, $color)
                $(.var(stringify!($key), $value))*
                .build($template)
        }
    };
}

// Generate all error diagrams using the macro
pub struct UnifiedErrorDiagrams;

impl UnifiedErrorDiagrams {
    error_diagram!(network_error, "box", "Network Connection Error", colored::Color::BrightRed,
        content => "Check your internet connection and API keys".to_string(),
        footer => "Use 'ccswarm health' to diagnose issues".to_string()
    );
    
    error_diagram!(session_error, "box", "Session State Error", colored::Color::BrightCyan,
        content => "Session might be in an invalid state".to_string(),
        footer => "Use 'ccswarm session list' to check status".to_string()
    );
    
    error_diagram!(config_error, "box", "Configuration Error", colored::Color::BrightCyan,
        content => "Configuration file is missing or invalid".to_string(),
        footer => "Use 'ccswarm init' to create configuration".to_string()
    );
}