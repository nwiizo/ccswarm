/// Converter for migrating existing agents to Claude Code subagent format
///
/// This module provides tools to convert existing ccswarm agent configurations
/// to the new Claude Code subagent definition format.
use super::{SubagentDefinition, SubagentError, SubagentResult, SubagentTools};
use crate::config::AgentConfig;
use crate::identity::AgentRole;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Converter for agent configurations
pub struct AgentConverter;

impl AgentConverter {
    /// Convert an existing AgentConfig to SubagentDefinition
    pub fn convert_agent_config(
        config: &AgentConfig,
    ) -> SubagentResult<(SubagentDefinition, String)> {
        // Determine role from specialization
        let role = Self::role_from_specialization(&config.specialization);
        let (tools, capabilities) = Self::extract_tools_and_capabilities(&role);

        let definition = SubagentDefinition {
            name: config.specialization.clone(),
            description: Self::generate_description(&role),
            tools,
            capabilities,
            metadata: Self::extract_metadata(config),
        };

        let instructions = Self::generate_instructions(&role, &config.specialization);

        Ok((definition, instructions))
    }

    /// Extract tools and capabilities based on agent role
    fn extract_tools_and_capabilities(role: &AgentRole) -> (SubagentTools, Vec<String>) {
        match role {
            AgentRole::Frontend { .. } => (
                SubagentTools {
                    standard: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "execute_command".to_string(),
                    ],
                    semantic: vec!["find_symbol".to_string(), "search_for_pattern".to_string()],
                    memory: vec!["read_memory".to_string(), "write_memory".to_string()],
                    custom: vec![
                        "component_generator".to_string(),
                        "style_analyzer".to_string(),
                    ],
                },
                vec![
                    "React/Vue development".to_string(),
                    "UI/UX implementation".to_string(),
                    "Component architecture".to_string(),
                    "Responsive design".to_string(),
                    "State management".to_string(),
                ],
            ),
            AgentRole::Backend { .. } => (
                SubagentTools {
                    standard: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "execute_command".to_string(),
                        "database_query".to_string(),
                    ],
                    semantic: vec![
                        "find_symbol".to_string(),
                        "replace_symbol_body".to_string(),
                        "find_referencing_symbols".to_string(),
                        "search_for_pattern".to_string(),
                    ],
                    memory: vec![
                        "read_memory".to_string(),
                        "write_memory".to_string(),
                        "list_memories".to_string(),
                    ],
                    custom: vec!["api_generator".to_string(), "schema_validator".to_string()],
                },
                vec![
                    "API design and implementation".to_string(),
                    "Database optimization".to_string(),
                    "Business logic implementation".to_string(),
                    "Security best practices".to_string(),
                    "Performance optimization".to_string(),
                ],
            ),
            AgentRole::DevOps { .. } => (
                SubagentTools {
                    standard: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "execute_command".to_string(),
                    ],
                    semantic: vec!["search_for_pattern".to_string()],
                    memory: vec!["read_memory".to_string(), "write_memory".to_string()],
                    custom: vec![
                        "docker_builder".to_string(),
                        "ci_cd_generator".to_string(),
                        "infrastructure_analyzer".to_string(),
                    ],
                },
                vec![
                    "Docker containerization".to_string(),
                    "CI/CD pipeline setup".to_string(),
                    "Infrastructure as Code".to_string(),
                    "Deployment automation".to_string(),
                    "Monitoring and logging".to_string(),
                ],
            ),
            AgentRole::QA { .. } => (
                SubagentTools {
                    standard: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "execute_command".to_string(),
                    ],
                    semantic: vec![
                        "find_symbol".to_string(),
                        "search_for_pattern".to_string(),
                        "find_referencing_symbols".to_string(),
                    ],
                    memory: vec!["read_memory".to_string(), "write_memory".to_string()],
                    custom: vec![
                        "test_generator".to_string(),
                        "coverage_analyzer".to_string(),
                        "bug_detector".to_string(),
                    ],
                },
                vec![
                    "Test strategy development".to_string(),
                    "Unit and integration testing".to_string(),
                    "Test coverage analysis".to_string(),
                    "Bug detection and reporting".to_string(),
                    "Quality metrics tracking".to_string(),
                ],
            ),
            _ => (
                SubagentTools {
                    standard: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "execute_command".to_string(),
                    ],
                    semantic: vec![],
                    memory: vec![],
                    custom: vec![],
                },
                vec!["General task execution".to_string()],
            ),
        }
    }

    /// Generate description based on role
    fn generate_description(role: &AgentRole) -> String {
        match role {
            AgentRole::Frontend { .. } => {
                "Frontend development specialist with expertise in modern web frameworks and UI/UX implementation.".to_string()
            }
            AgentRole::Backend { .. } => {
                "Backend development specialist focused on API design, database optimization, and business logic.".to_string()
            }
            AgentRole::DevOps { .. } => {
                "DevOps specialist handling containerization, CI/CD, and infrastructure automation.".to_string()
            }
            AgentRole::QA { .. } => {
                "Quality assurance specialist ensuring code quality through comprehensive testing and analysis.".to_string()
            }
            _ => "General purpose agent for various development tasks.".to_string(),
        }
    }

    /// Generate instructions based on role
    fn generate_instructions(role: &AgentRole, name: &str) -> String {
        let role_specific = match role {
            AgentRole::Frontend { .. } => include_str!("../../templates/frontend_instructions.md"),
            AgentRole::Backend { .. } => include_str!("../../templates/backend_instructions.md"),
            AgentRole::DevOps { .. } => include_str!("../../templates/devops_instructions.md"),
            AgentRole::QA { .. } => include_str!("../../templates/qa_instructions.md"),
            _ => include_str!("../../templates/general_instructions.md"),
        };

        format!("# {} Agent Instructions\n\n{}", name, role_specific)
    }

    /// Extract metadata from agent config
    fn extract_metadata(config: &AgentConfig) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();

        metadata.insert(
            "specialization".to_string(),
            serde_json::json!(config.specialization),
        );

        metadata.insert("worktree".to_string(), serde_json::json!(config.worktree));

        metadata.insert("branch".to_string(), serde_json::json!(config.branch));

        metadata
    }

    /// Determine role from specialization string
    fn role_from_specialization(specialization: &str) -> AgentRole {
        match specialization.to_lowercase().as_str() {
            s if s.contains("frontend") => AgentRole::Frontend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            s if s.contains("backend") => AgentRole::Backend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            s if s.contains("devops") => AgentRole::DevOps {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            s if s.contains("qa") || s.contains("test") => AgentRole::QA {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            s if s.contains("search") => AgentRole::Search {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
            // Default to Frontend for unknown specializations
            _ => AgentRole::Frontend {
                technologies: vec![],
                responsibilities: vec![],
                boundaries: vec![],
            },
        }
    }

    /// Convert and save a subagent definition to file
    pub fn save_subagent_definition(
        definition: &SubagentDefinition,
        instructions: &str,
        output_dir: &Path,
    ) -> SubagentResult<()> {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(output_dir)?;

        // Generate the filename
        let filename = format!("{}.md", definition.name);
        let filepath = output_dir.join(filename);

        // Serialize the definition to YAML frontmatter
        let frontmatter = serde_yaml::to_string(definition).map_err(|e| {
            SubagentError::Serialization(serde_json::Error::io(std::io::Error::other(
                e.to_string(),
            )))
        })?;

        // Combine frontmatter and instructions
        let content = format!("---\n{}---\n\n{}", frontmatter, instructions);

        // Write to file
        fs::write(&filepath, content)?;

        log::info!("Saved subagent definition to {:?}", filepath);

        Ok(())
    }

    /// Batch convert all agents in a project
    pub fn batch_convert_project(
        project_config_path: &Path,
        output_dir: &Path,
    ) -> SubagentResult<Vec<String>> {
        // Read the project configuration
        let config_content = fs::read_to_string(project_config_path)?;
        let project_config: serde_json::Value = serde_json::from_str(&config_content)?;

        let mut converted = Vec::new();

        // Extract agents array
        if let Some(agents) = project_config.get("agents").and_then(|a| a.as_array()) {
            for agent_value in agents {
                if let Ok(agent_config) = serde_json::from_value::<AgentConfig>(agent_value.clone())
                {
                    let (definition, instructions) = Self::convert_agent_config(&agent_config)?;
                    Self::save_subagent_definition(&definition, &instructions, output_dir)?;
                    converted.push(definition.name);
                }
            }
        }

        log::info!("Converted {} agents to subagent format", converted.len());

        Ok(converted)
    }
}

mod tests {

    #[test]
    fn test_convert_frontend_agent() {
        let config = AgentConfig {
            specialization: "frontend-specialist".to_string(),
            worktree: "agents/frontend".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: ClaudeConfig::default(),
            claude_md_template: "frontend.md".to_string(),
        };

        let result = AgentConverter::convert_agent_config(&config);
        assert!(result.is_ok());

        let (definition, _instructions) = result.unwrap();
        assert_eq!(definition.name, "frontend-specialist");
        assert!(!definition.tools.standard.is_empty());
        assert!(!definition.capabilities.is_empty());
    }

    #[test]
    fn test_convert_backend_agent() {
        let config = AgentConfig {
            specialization: "backend-specialist".to_string(),
            worktree: "agents/backend".to_string(),
            branch: "feature/backend".to_string(),
            claude_config: ClaudeConfig::default(),
            claude_md_template: "backend.md".to_string(),
        };

        let result = AgentConverter::convert_agent_config(&config);
        assert!(result.is_ok());

        let (definition, _instructions) = result.unwrap();
        assert!(definition
            .tools
            .standard
            .contains(&"database_query".to_string()));
        assert!(!definition.tools.semantic.is_empty());
    }
}
