/// Parser for Claude Code subagent definition files
///
/// This module handles parsing of the YAML frontmatter and markdown content
/// from .claude/agents/*.md files
use super::{SubagentDefinition, SubagentError, SubagentResult};
use std::fs;
use std::path::Path;

/// Parser for subagent definition files
pub struct SubagentParser;

impl SubagentParser {
    /// Parse a subagent definition file
    pub fn parse_file(path: &Path) -> SubagentResult<(SubagentDefinition, String)> {
        let content = fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    /// Parse subagent definition from string content
    pub fn parse_content(content: &str) -> SubagentResult<(SubagentDefinition, String)> {
        // Split the content into frontmatter and body
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(SubagentError::Parse(
                "Invalid subagent file format: missing frontmatter".to_string(),
            ));
        }

        // Parse the YAML frontmatter
        let frontmatter = parts[1].trim();
        let body = parts[2].trim();

        let mut definition: SubagentDefinition = serde_yaml::from_str(frontmatter)
            .map_err(|e| SubagentError::Parse(format!("Failed to parse YAML: {}", e)))?;

        // Validate the definition
        Self::validate_definition(&mut definition)?;

        Ok((definition, body.to_string()))
    }

    /// Parse all subagent definitions in a directory
    pub fn parse_directory(dir: &Path) -> SubagentResult<Vec<(SubagentDefinition, String)>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match Self::parse_file(&path) {
                    Ok(agent) => agents.push(agent),
                    Err(e) => {
                        crate::utils::common::logging::log_parse_warning(
                            &path.display().to_string(),
                            &e,
                        );
                    }
                }
            }
        }

        Ok(agents)
    }

    pub fn validate_definition(definition: &mut SubagentDefinition) -> SubagentResult<()> {
        // Validate that the definition has required fields
        if definition.name.is_empty() {
            return Err(SubagentError::Validation(
                "Subagent name cannot be empty".to_string(),
            ));
        }

        if definition.description.is_empty() {
            return Err(SubagentError::Validation(
                "Subagent description cannot be empty".to_string(),
            ));
        }

        // Validate tools
        if definition.tools.standard.is_empty()
            && definition.tools.semantic.is_empty()
            && definition.tools.memory.is_empty()
            && definition.tools.custom.is_empty()
        {
            return Err(SubagentError::Validation(
                "Subagent must have at least one tool".to_string(),
            ));
        }

        // Validate capabilities
        if definition.capabilities.is_empty() {
            return Err(SubagentError::Validation(
                "Subagent must have at least one capability".to_string(),
            ));
        }

        Ok(())
    }
}
