//! Faceted Prompting system for Piece/Movement workflows.
//!
//! Decomposes prompts into five orthogonal concerns (takt-style):
//! 1. **Persona** — Agent role, expertise, behavioral principles (system prompt)
//! 2. **Policy** — Rules, prohibitions, quality standards
//! 3. **Instruction** — Step-specific procedures and goals
//! 4. **Knowledge** — Domain context, reference materials
//! 5. **Output Contract** — Output structure and format (handled by piece.rs OutputContract)
//!
//! Composition order:
//! - System prompt: Persona
//! - User message: Knowledge → Instruction → Policy → Output Contract

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A facet definition that can be inline text or a file reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FacetSource {
    /// Inline text content
    Inline(String),
    /// Reference to a YAML file containing the facet
    FileRef { file: String },
}

/// A persona facet defining agent identity and role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaFacet {
    /// Persona name/identifier
    pub name: String,
    /// Role description
    #[serde(default)]
    pub role: String,
    /// Expertise areas
    #[serde(default)]
    pub expertise: Vec<String>,
    /// Behavioral principles
    #[serde(default)]
    pub principles: Vec<String>,
    /// System prompt content (the actual persona text)
    #[serde(default)]
    pub system_prompt: String,
}

/// A policy facet defining rules and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyFacet {
    /// Policy name
    pub name: String,
    /// Rules to follow
    #[serde(default)]
    pub rules: Vec<String>,
    /// Prohibited actions
    #[serde(default)]
    pub prohibitions: Vec<String>,
    /// Quality standards
    #[serde(default)]
    pub standards: Vec<String>,
    /// Raw policy text
    #[serde(default)]
    pub content: String,
}

/// A knowledge facet providing domain context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFacet {
    /// Knowledge name
    pub name: String,
    /// Domain context items
    #[serde(default)]
    pub context: Vec<String>,
    /// Reference materials (file paths or URLs)
    #[serde(default)]
    pub references: Vec<String>,
    /// Raw knowledge text
    #[serde(default)]
    pub content: String,
}

/// Registry of all loaded facets
#[derive(Debug, Default)]
pub struct FacetRegistry {
    /// Loaded personas by name
    pub personas: HashMap<String, PersonaFacet>,
    /// Loaded policies by name
    pub policies: HashMap<String, PolicyFacet>,
    /// Loaded knowledge by name
    pub knowledge: HashMap<String, KnowledgeFacet>,
    /// Base directory for resolving file references
    base_dir: Option<PathBuf>,
}

/// Composed prompt from facets
#[derive(Debug, Clone)]
pub struct ComposedPrompt {
    /// System prompt (persona-derived)
    pub system: String,
    /// User message (knowledge + instruction + policy + output contract)
    pub user: String,
}

impl FacetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set base directory for file references
    pub fn with_base_dir(mut self, dir: PathBuf) -> Self {
        self.base_dir = Some(dir);
        self
    }

    /// Load all facets from a directory structure.
    ///
    /// Expected layout:
    /// ```text
    /// base_dir/
    ///   personas/
    ///     coder.yaml
    ///     reviewer.yaml
    ///   policies/
    ///     coding.yaml
    ///     security.yaml
    ///   knowledge/
    ///     architecture.yaml
    /// ```
    pub async fn load_from_dir(&mut self, dir: &Path) -> Result<()> {
        self.base_dir = Some(dir.to_path_buf());

        let personas_dir = dir.join("personas");
        if personas_dir.exists() {
            self.load_personas(&personas_dir).await?;
        }

        let policies_dir = dir.join("policies");
        if policies_dir.exists() {
            self.load_policies(&policies_dir).await?;
        }

        let knowledge_dir = dir.join("knowledge");
        if knowledge_dir.exists() {
            self.load_knowledge(&knowledge_dir).await?;
        }

        Ok(())
    }

    async fn load_personas(&mut self, dir: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if is_yaml_file(&path) {
                match load_yaml::<PersonaFacet>(&path).await {
                    Ok(persona) => {
                        debug!("Loaded persona: {}", persona.name);
                        self.personas.insert(persona.name.clone(), persona);
                    }
                    Err(e) => warn!("Failed to load persona from {}: {}", path.display(), e),
                }
            }
        }
        Ok(())
    }

    async fn load_policies(&mut self, dir: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if is_yaml_file(&path) {
                match load_yaml::<PolicyFacet>(&path).await {
                    Ok(policy) => {
                        debug!("Loaded policy: {}", policy.name);
                        self.policies.insert(policy.name.clone(), policy);
                    }
                    Err(e) => warn!("Failed to load policy from {}: {}", path.display(), e),
                }
            }
        }
        Ok(())
    }

    async fn load_knowledge(&mut self, dir: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if is_yaml_file(&path) {
                match load_yaml::<KnowledgeFacet>(&path).await {
                    Ok(knowledge) => {
                        debug!("Loaded knowledge: {}", knowledge.name);
                        self.knowledge.insert(knowledge.name.clone(), knowledge);
                    }
                    Err(e) => warn!("Failed to load knowledge from {}: {}", path.display(), e),
                }
            }
        }
        Ok(())
    }

    /// Register a persona directly
    pub fn register_persona(&mut self, persona: PersonaFacet) {
        self.personas.insert(persona.name.clone(), persona);
    }

    /// Register a policy directly
    pub fn register_policy(&mut self, policy: PolicyFacet) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Register a knowledge facet directly
    pub fn register_knowledge(&mut self, knowledge: KnowledgeFacet) {
        self.knowledge.insert(knowledge.name.clone(), knowledge);
    }

    /// Compose a prompt from facet references and an instruction.
    ///
    /// Follows takt's composition order:
    /// - System: persona
    /// - User: knowledge → instruction → policy → output_contract
    pub fn compose(
        &self,
        persona_name: Option<&str>,
        policy_name: Option<&str>,
        knowledge_name: Option<&str>,
        instruction: &str,
        output_contract_text: Option<&str>,
    ) -> ComposedPrompt {
        // System prompt = persona
        let system = match persona_name {
            Some(name) => self.render_persona(name),
            None => String::new(),
        };

        // User message = knowledge → instruction → policy → output contract
        let mut user_parts = Vec::new();

        // 1. Knowledge (context first)
        if let Some(name) = knowledge_name {
            let knowledge_text = self.render_knowledge(name);
            if !knowledge_text.is_empty() {
                user_parts.push(format!("## Context\n\n{}", knowledge_text));
            }
        }

        // 2. Instruction (the task)
        if !instruction.is_empty() {
            user_parts.push(format!("## Task\n\n{}", instruction));
        }

        // 3. Policy (constraints - at end for recency effect)
        if let Some(name) = policy_name {
            let policy_text = self.render_policy(name);
            if !policy_text.is_empty() {
                user_parts.push(format!("## Constraints\n\n{}", policy_text));
            }
        }

        // 4. Output contract
        if let Some(contract) = output_contract_text {
            user_parts.push(format!("## Output Format\n\n{}", contract));
        }

        ComposedPrompt {
            system,
            user: user_parts.join("\n\n"),
        }
    }

    /// Render persona into system prompt text
    fn render_persona(&self, name: &str) -> String {
        match self.personas.get(name) {
            Some(persona) => {
                let mut parts = Vec::new();

                if !persona.system_prompt.is_empty() {
                    return persona.system_prompt.clone();
                }

                parts.push(format!("You are {}", persona.name));

                if !persona.role.is_empty() {
                    parts.push(format!("Role: {}", persona.role));
                }

                if !persona.expertise.is_empty() {
                    parts.push(format!("Expertise: {}", persona.expertise.join(", ")));
                }

                if !persona.principles.is_empty() {
                    parts.push("Principles:".to_string());
                    for principle in &persona.principles {
                        parts.push(format!("- {}", principle));
                    }
                }

                parts.join("\n")
            }
            None => {
                // Treat name as inline persona description
                format!("You are acting as: {}", name)
            }
        }
    }

    /// Render policy into constraint text
    fn render_policy(&self, name: &str) -> String {
        match self.policies.get(name) {
            Some(policy) => {
                if !policy.content.is_empty() {
                    return policy.content.clone();
                }

                let mut parts = Vec::new();

                if !policy.rules.is_empty() {
                    parts.push("Rules:".to_string());
                    for rule in &policy.rules {
                        parts.push(format!("- {}", rule));
                    }
                }

                if !policy.prohibitions.is_empty() {
                    parts.push("Prohibitions:".to_string());
                    for prohibition in &policy.prohibitions {
                        parts.push(format!("- NEVER: {}", prohibition));
                    }
                }

                if !policy.standards.is_empty() {
                    parts.push("Quality Standards:".to_string());
                    for standard in &policy.standards {
                        parts.push(format!("- {}", standard));
                    }
                }

                parts.join("\n")
            }
            None => {
                // Treat name as inline policy text
                name.to_string()
            }
        }
    }

    /// Render knowledge into context text
    fn render_knowledge(&self, name: &str) -> String {
        match self.knowledge.get(name) {
            Some(knowledge) => {
                if !knowledge.content.is_empty() {
                    return knowledge.content.clone();
                }

                let mut parts = Vec::new();

                if !knowledge.context.is_empty() {
                    for item in &knowledge.context {
                        parts.push(item.clone());
                    }
                }

                if !knowledge.references.is_empty() {
                    parts.push("References:".to_string());
                    for reference in &knowledge.references {
                        parts.push(format!("- {}", reference));
                    }
                }

                parts.join("\n")
            }
            None => {
                // Treat name as inline knowledge
                name.to_string()
            }
        }
    }
}

/// Load and parse a YAML file
async fn load_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read: {}", path.display()))?;
    serde_yaml::from_str(&contents)
        .with_context(|| format!("Failed to parse YAML: {}", path.display()))
}

/// Check if a path is a YAML file
fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "yaml" || e == "yml")
        .unwrap_or(false)
}

/// Provide built-in personas for common agent roles
pub fn builtin_personas() -> Vec<PersonaFacet> {
    vec![
        PersonaFacet {
            name: "planner".to_string(),
            role: "Technical architect and planner".to_string(),
            expertise: vec![
                "system design".to_string(),
                "task decomposition".to_string(),
                "risk assessment".to_string(),
            ],
            principles: vec![
                "Analyze before acting".to_string(),
                "Identify dependencies and risks".to_string(),
                "Break complex tasks into manageable steps".to_string(),
            ],
            system_prompt: String::new(),
        },
        PersonaFacet {
            name: "coder".to_string(),
            role: "Senior software engineer".to_string(),
            expertise: vec![
                "code implementation".to_string(),
                "debugging".to_string(),
                "testing".to_string(),
            ],
            principles: vec![
                "Write clean, maintainable code".to_string(),
                "Follow existing patterns and conventions".to_string(),
                "Test what you implement".to_string(),
                "Keep changes minimal and focused".to_string(),
            ],
            system_prompt: String::new(),
        },
        PersonaFacet {
            name: "reviewer".to_string(),
            role: "Code reviewer and quality gate".to_string(),
            expertise: vec![
                "code review".to_string(),
                "security analysis".to_string(),
                "performance optimization".to_string(),
            ],
            principles: vec![
                "Be thorough but pragmatic".to_string(),
                "Focus on correctness and security".to_string(),
                "Provide actionable feedback".to_string(),
                "Distinguish critical issues from nits".to_string(),
            ],
            system_prompt: String::new(),
        },
        PersonaFacet {
            name: "researcher".to_string(),
            role: "Technical researcher and analyst".to_string(),
            expertise: vec![
                "information gathering".to_string(),
                "pattern analysis".to_string(),
                "documentation".to_string(),
            ],
            principles: vec![
                "Be comprehensive in investigation".to_string(),
                "Cite sources and evidence".to_string(),
                "Distinguish facts from assumptions".to_string(),
            ],
            system_prompt: String::new(),
        },
    ]
}

/// Provide built-in policies
pub fn builtin_policies() -> Vec<PolicyFacet> {
    vec![
        PolicyFacet {
            name: "coding".to_string(),
            rules: vec![
                "Follow existing code style and patterns".to_string(),
                "Write unit tests for new functionality".to_string(),
                "Handle errors explicitly".to_string(),
            ],
            prohibitions: vec![
                "Use .unwrap() in production code".to_string(),
                "Hardcode secrets or API keys".to_string(),
                "Skip error handling".to_string(),
            ],
            standards: vec![
                "All tests must pass".to_string(),
                "No clippy warnings".to_string(),
                "Code must be formatted".to_string(),
            ],
            content: String::new(),
        },
        PolicyFacet {
            name: "security".to_string(),
            rules: vec![
                "Validate all inputs".to_string(),
                "Use parameterized queries".to_string(),
                "Sanitize output".to_string(),
            ],
            prohibitions: vec![
                "Store secrets in code".to_string(),
                "Disable security checks".to_string(),
                "Use eval or dynamic code execution".to_string(),
            ],
            standards: vec![
                "OWASP Top 10 compliance".to_string(),
                "No known CVEs in dependencies".to_string(),
            ],
            content: String::new(),
        },
        PolicyFacet {
            name: "review".to_string(),
            rules: vec![
                "Review all changed files".to_string(),
                "Check for security vulnerabilities".to_string(),
                "Verify test coverage".to_string(),
            ],
            prohibitions: vec![
                "Approve without reading the code".to_string(),
                "Ignore test failures".to_string(),
            ],
            standards: vec![
                "All critical issues must be flagged".to_string(),
                "Provide specific line references".to_string(),
            ],
            content: String::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_full_prompt() {
        let mut registry = FacetRegistry::new();

        registry.register_persona(PersonaFacet {
            name: "coder".to_string(),
            role: "Engineer".to_string(),
            expertise: vec!["Rust".to_string()],
            principles: vec!["Write clean code".to_string()],
            system_prompt: String::new(),
        });

        registry.register_policy(PolicyFacet {
            name: "coding".to_string(),
            rules: vec!["Test everything".to_string()],
            prohibitions: vec!["Use unwrap".to_string()],
            standards: vec![],
            content: String::new(),
        });

        registry.register_knowledge(KnowledgeFacet {
            name: "architecture".to_string(),
            context: vec!["This is a Rust project using tokio".to_string()],
            references: vec![],
            content: String::new(),
        });

        let prompt = registry.compose(
            Some("coder"),
            Some("coding"),
            Some("architecture"),
            "Implement the login endpoint",
            Some("Return JSON response"),
        );

        // System should contain persona
        assert!(prompt.system.contains("coder"));
        assert!(prompt.system.contains("Engineer"));

        // User should contain knowledge, instruction, policy, output contract in order
        assert!(prompt.user.contains("Context"));
        assert!(prompt.user.contains("Rust project"));
        assert!(prompt.user.contains("Task"));
        assert!(prompt.user.contains("login endpoint"));
        assert!(prompt.user.contains("Constraints"));
        assert!(prompt.user.contains("Test everything"));
        assert!(prompt.user.contains("Output Format"));
        assert!(prompt.user.contains("JSON response"));

        // Verify ordering: knowledge before instruction before policy
        let knowledge_pos = prompt.user.find("Context").unwrap();
        let instruction_pos = prompt.user.find("Task").unwrap();
        let policy_pos = prompt.user.find("Constraints").unwrap();
        assert!(knowledge_pos < instruction_pos);
        assert!(instruction_pos < policy_pos);
    }

    #[test]
    fn test_compose_minimal() {
        let registry = FacetRegistry::new();

        // No registered facets - should use inline fallback
        let prompt = registry.compose(Some("reviewer"), None, None, "Review this code", None);

        assert!(prompt.system.contains("reviewer"));
        assert!(prompt.user.contains("Review this code"));
    }

    #[test]
    fn test_compose_with_system_prompt() {
        let mut registry = FacetRegistry::new();

        registry.register_persona(PersonaFacet {
            name: "custom".to_string(),
            role: String::new(),
            expertise: vec![],
            principles: vec![],
            system_prompt: "You are a specialized security auditor.".to_string(),
        });

        let prompt = registry.compose(Some("custom"), None, None, "Audit", None);
        assert_eq!(prompt.system, "You are a specialized security auditor.");
    }

    #[test]
    fn test_builtin_personas() {
        let personas = builtin_personas();
        assert!(personas.len() >= 4);

        let names: Vec<&str> = personas.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"planner"));
        assert!(names.contains(&"coder"));
        assert!(names.contains(&"reviewer"));
        assert!(names.contains(&"researcher"));
    }

    #[test]
    fn test_builtin_policies() {
        let policies = builtin_policies();
        assert!(policies.len() >= 3);

        let names: Vec<&str> = policies.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"coding"));
        assert!(names.contains(&"security"));
        assert!(names.contains(&"review"));
    }

    #[test]
    fn test_inline_fallback() {
        let registry = FacetRegistry::new();

        let prompt = registry.compose(
            Some("expert Python developer"),
            Some("Always use type hints"),
            Some("FastAPI web framework"),
            "Create an endpoint",
            None,
        );

        // Inline persona
        assert!(prompt.system.contains("expert Python developer"));
        // Inline policy
        assert!(prompt.user.contains("type hints"));
        // Inline knowledge
        assert!(prompt.user.contains("FastAPI"));
    }

    #[test]
    fn test_persona_yaml_parse() {
        let yaml = r#"
name: architect
role: System architect
expertise:
  - distributed systems
  - microservices
principles:
  - Think about scalability
  - Consider failure modes
"#;
        let persona: PersonaFacet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(persona.name, "architect");
        assert_eq!(persona.expertise.len(), 2);
        assert_eq!(persona.principles.len(), 2);
    }

    #[test]
    fn test_policy_yaml_parse() {
        let yaml = r#"
name: strict-review
rules:
  - Check all edge cases
prohibitions:
  - Approve without tests
standards:
  - 80% code coverage
"#;
        let policy: PolicyFacet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(policy.name, "strict-review");
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.prohibitions.len(), 1);
    }
}
