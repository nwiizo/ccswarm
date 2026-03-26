//! Agent definition generator - creates .claude/agents/*.md from facets

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Generate a .claude/agents/*.md file from faceted prompting components.
///
/// Composes persona + policy + knowledge facets into a Claude Code agent definition
/// compatible with the Agent Teams format.
pub async fn generate_agent_definition(
    name: &str,
    persona: Option<&str>,
    policy: Option<&str>,
    description: Option<&str>,
    model: &str,
    output_dir: &Path,
) -> Result<PathBuf> {
    let mut sections = Vec::new();

    // Frontmatter
    sections.push(format!(
        "---\nmodel: {model}\ndescription: {desc}\n---\n",
        model = model,
        desc = description.unwrap_or(name),
    ));

    // Try to load persona facet
    if let Some(persona_name) = persona {
        let registry = crate::workflow::facets::FacetRegistry::new_with_builtins();
        if let Some(p) = registry.get_persona(persona_name) {
            sections.push(format!("# Role\n\n{}\n", p.system_prompt));
        } else {
            sections.push(format!(
                "# Role\n\nYou are a {} specialist.\n",
                persona_name
            ));
        }
    }

    // Try to load policy facet
    if let Some(policy_name) = policy {
        let registry = crate::workflow::facets::FacetRegistry::new_with_builtins();
        if let Some(p) = registry.get_policy(policy_name) {
            let mut policy_text = String::new();
            if !p.rules.is_empty() {
                policy_text.push_str("## Rules\n\n");
                for rule in &p.rules {
                    policy_text.push_str(&format!("- {}\n", rule));
                }
                policy_text.push('\n');
            }
            if !p.prohibitions.is_empty() {
                policy_text.push_str("## Prohibitions\n\n");
                for rule in &p.prohibitions {
                    policy_text.push_str(&format!("- {}\n", rule));
                }
                policy_text.push('\n');
            }
            sections.push(policy_text);
        }
    }

    // Write the file
    let file_name = format!("{}.md", name);
    let output_path = output_dir.join(&file_name);
    tokio::fs::create_dir_all(output_dir)
        .await
        .with_context(|| format!("Failed to create directory {:?}", output_dir))?;

    let content = sections.join("\n");
    tokio::fs::write(&output_path, &content)
        .await
        .with_context(|| format!("Failed to write agent definition to {:?}", output_path))?;

    Ok(output_path)
}

/// Validate an existing .claude/agents/*.md file
pub async fn validate_agent_definition(path: &Path) -> Result<Vec<String>> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {:?}", path))?;

    let mut issues = Vec::new();

    // Check for frontmatter
    if !content.starts_with("---") {
        issues.push("Missing YAML frontmatter (should start with ---)".to_string());
    }

    // Check for model field
    if !content.contains("model:") {
        issues.push("Missing 'model:' field in frontmatter".to_string());
    }

    // Check for description
    if !content.contains("description:") {
        issues.push("Missing 'description:' field in frontmatter".to_string());
    }

    // Check minimum content length
    if content.len() < 50 {
        issues.push("Agent definition seems too short (< 50 chars)".to_string());
    }

    Ok(issues)
}
