//! Faceted Prompting system for Flow/Stage workflows.
//!
//! Decomposes prompts into five orthogonal concerns (takt-style):
//! 1. **Persona** — Agent role, expertise, behavioral principles (system prompt)
//! 2. **Policy** — Rules, prohibitions, quality standards
//! 3. **Instruction** — Step-specific procedures and goals
//! 4. **Knowledge** — Domain context, reference materials
//! 5. **Output Contract** — Output structure and format (handled by flow.rs OutputContract)
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
    /// Policy description
    #[serde(default)]
    pub description: String,
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

    /// Create a registry pre-loaded with built-in personas and policies
    pub fn new_with_builtins() -> Self {
        let mut registry = Self::new();
        for persona in builtin_personas() {
            registry.register_persona(persona);
        }
        for policy in builtin_policies() {
            registry.register_policy(policy);
        }
        registry
    }

    /// Get a persona by name
    pub fn get_persona(&self, name: &str) -> Option<&PersonaFacet> {
        self.personas.get(name)
    }

    /// Get a policy by name
    pub fn get_policy(&self, name: &str) -> Option<&PolicyFacet> {
        self.policies.get(name)
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
        let loaded = load_facet_dir::<PersonaFacet>(dir, "persona").await?;
        for persona in loaded {
            self.personas.insert(persona.name.clone(), persona);
        }
        Ok(())
    }

    async fn load_policies(&mut self, dir: &Path) -> Result<()> {
        let loaded = load_facet_dir::<PolicyFacet>(dir, "policy").await?;
        for policy in loaded {
            self.policies.insert(policy.name.clone(), policy);
        }
        Ok(())
    }

    async fn load_knowledge(&mut self, dir: &Path) -> Result<()> {
        let loaded = load_facet_dir::<KnowledgeFacet>(dir, "knowledge").await?;
        for knowledge in loaded {
            self.knowledge.insert(knowledge.name.clone(), knowledge);
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
    /// Composition order:
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
    serde_yml::from_str(&contents)
        .with_context(|| format!("Failed to parse YAML: {}", path.display()))
}

/// Marker trait so the shared `load_facet_dir` helper can log each loaded facet by
/// name. Implemented for the three concrete facet kinds. Intentionally small — we
/// don't want these kinds to share anything else.
trait FacetNamed {
    fn facet_name(&self) -> &str;
}

impl FacetNamed for PersonaFacet {
    fn facet_name(&self) -> &str {
        &self.name
    }
}
impl FacetNamed for PolicyFacet {
    fn facet_name(&self) -> &str {
        &self.name
    }
}
impl FacetNamed for KnowledgeFacet {
    fn facet_name(&self) -> &str {
        &self.name
    }
}

/// Walk `dir`, parse every `*.yaml` / `*.yml` file as `T`, return the list. A
/// single malformed file is logged and skipped rather than aborting the whole
/// load — other facets in the same directory are still picked up. `kind` is the
/// human label ("persona" / "policy" / "knowledge") used in log messages.
async fn load_facet_dir<T>(dir: &Path, kind: &str) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned + FacetNamed,
{
    let mut out: Vec<T> = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !is_yaml_file(&path) {
            continue;
        }
        match load_yaml::<T>(&path).await {
            Ok(facet) => {
                debug!("Loaded {}: {}", kind, facet.facet_name());
                out.push(facet);
            }
            Err(e) => warn!("Failed to load {} from {}: {}", kind, path.display(), e),
        }
    }
    Ok(out)
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
                "Investigate before planning - don't plan without reading existing code".to_string(),
                "Design simply with no excessive abstractions".to_string(),
                "Identify dependencies and risks".to_string(),
                "Verify specifications before specifying implementation approach".to_string(),
            ],
            system_prompt: "You are a task analysis and design planning specialist. Analyze user requirements, investigate code to resolve unknowns, identify impact scope, determine file structure and design patterns, and create implementation guidelines. Investigate before planning - don't plan without reading existing code. Design simply with no excessive abstractions. Verify specifications before specifying implementation approach.".to_string(),
        },
        PersonaFacet {
            name: "coder".to_string(),
            role: "Implementer".to_string(),
            expertise: vec![
                "code implementation".to_string(),
                "debugging".to_string(),
                "testing".to_string(),
            ],
            principles: vec![
                "Thoroughness over speed - code correctness over implementation ease".to_string(),
                "Don't implement by guessing; report unclear points".to_string(),
                "Feedback from review is absolute - fix all flagged issues without arguing".to_string(),
                "Keep changes minimal and focused".to_string(),
            ],
            system_prompt: "You are the implementer. Focus on implementation, not design decisions. Implement according to the plan, write test code, fix issues pointed out in reviews. Thoroughness over speed - code correctness over implementation ease. Don't implement by guessing; report unclear points. Feedback from review is absolute - fix all flagged issues without arguing.".to_string(),
        },
        PersonaFacet {
            name: "reviewer".to_string(),
            role: "Code reviewer specialized in architecture and quality".to_string(),
            expertise: vec![
                "architecture review".to_string(),
                "dependency analysis".to_string(),
                "error handling".to_string(),
                "test coverage".to_string(),
            ],
            principles: vec![
                "Provide specific, actionable feedback with file paths and line references".to_string(),
                "Distinguish critical issues from minor nits".to_string(),
                "Focus on correctness, security, and maintainability".to_string(),
            ],
            system_prompt: "You are a code reviewer specialized in architecture and quality. Check structural design, dependency direction, separation of concerns, error handling, and test coverage. Provide specific, actionable feedback with file paths and line references. Distinguish critical issues from minor nits. Focus on correctness, security, and maintainability.".to_string(),
        },
        PersonaFacet {
            name: "researcher".to_string(),
            role: "Technical researcher and analyst".to_string(),
            expertise: vec![
                "information gathering".to_string(),
                "tradeoff evaluation".to_string(),
                "documentation".to_string(),
            ],
            principles: vec![
                "Investigate solutions by reading actual code and documentation - don't guess".to_string(),
                "Evaluate tradeoffs with evidence".to_string(),
                "Resolve unknowns by verification, not assumption".to_string(),
            ],
            system_prompt: "You are a technical researcher and analyst. Investigate solutions by reading actual code and documentation - don't guess. Evaluate tradeoffs with evidence. Compare approaches systematically. Cite sources, provide examples, and focus on practical applicability. Resolve unknowns by verification, not assumption.".to_string(),
        },
        PersonaFacet {
            name: "supervisor".to_string(),
            role: "Final verifier and human proxy".to_string(),
            expertise: vec![
                "validation".to_string(),
                "requirements verification".to_string(),
                "edge cases".to_string(),
            ],
            principles: vec![
                "Verify the right thing was built, not just built correctly".to_string(),
                "Check requirements are met with evidence".to_string(),
                "Verify no regressions".to_string(),
                "Act as human proxy - would a human approve this?".to_string(),
            ],
            system_prompt: "You are the final verifier and human proxy. While the reviewer confirms 'is it built correctly' (Verification), you verify 'was the right thing built' (Validation). Verify requirements are met, check test/build evidence, identify edge cases and error cases, verify no regressions. Ask yourself: does this really solve the user's problem? Are there unintended side effects? Is it safe to deploy?".to_string(),
        },
        PersonaFacet {
            name: "ai-antipattern-reviewer".to_string(),
            role: "AI code antipattern detector".to_string(),
            expertise: vec![
                "ai-generated code review".to_string(),
                "antipattern detection".to_string(),
            ],
            principles: vec![
                "Detect AI-specific antipatterns".to_string(),
                "Check for hallucinated APIs or imports".to_string(),
                "Verify no placeholder or stub code".to_string(),
            ],
            system_prompt: "You are an AI antipattern reviewer. Detect common AI-generated code issues: hallucinated APIs/imports that don't exist, placeholder implementations, unnecessary abstractions, over-engineering, inconsistent naming, dead code, and missing error handling. Verify all imports resolve to real modules and all function calls match actual signatures.".to_string(),
        },
    ]
}

/// Provide built-in policies
pub fn builtin_policies() -> Vec<PolicyFacet> {
    vec![
        PolicyFacet {
            name: "coding".to_string(),
            description: "Code quality and style standards".to_string(),
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
            description: "Security requirements and prohibitions".to_string(),
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
            description: "Code review process requirements".to_string(),
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
        PolicyFacet {
            name: "testing".to_string(),
            description: "Testing standards".to_string(),
            rules: vec![
                "Write tests before or alongside implementation".to_string(),
                "Test behavior, not implementation details".to_string(),
                "Each test should test one thing".to_string(),
                "Use descriptive test names that explain the scenario".to_string(),
            ],
            prohibitions: vec![
                "Skip tests for new functionality".to_string(),
                "Mock everything - prefer integration tests where practical".to_string(),
                "Write tests that depend on execution order".to_string(),
            ],
            standards: vec![],
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
            description: String::new(),
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
        assert!(personas.len() >= 6);

        let names: Vec<&str> = personas.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"planner"));
        assert!(names.contains(&"coder"));
        assert!(names.contains(&"reviewer"));
        assert!(names.contains(&"researcher"));
        assert!(names.contains(&"supervisor"));
        assert!(names.contains(&"ai-antipattern-reviewer"));
    }

    #[test]
    fn test_builtin_policies() {
        let policies = builtin_policies();
        assert!(policies.len() >= 4);

        let names: Vec<&str> = policies.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"coding"));
        assert!(names.contains(&"security"));
        assert!(names.contains(&"review"));
        assert!(names.contains(&"testing"));
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
        let persona: PersonaFacet = serde_yml::from_str(yaml).unwrap();
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
        let policy: PolicyFacet = serde_yml::from_str(yaml).unwrap();
        assert_eq!(policy.name, "strict-review");
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.prohibitions.len(), 1);
    }
}
