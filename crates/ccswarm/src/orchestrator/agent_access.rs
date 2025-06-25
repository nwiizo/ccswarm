/// Standardized agent attribute access helpers for MasterClaude
///
/// This module provides consistent ways to access agent properties
/// across the ccswarm/ai-session boundary.
use crate::agent::ClaudeCodeAgent;
use crate::identity::AgentRole;

/// Trait for standardized agent attribute access
pub trait AgentAttributeAccess {
    /// Get the agent's ID
    fn agent_id(&self) -> &str;

    /// Get the agent's role
    fn role(&self) -> &AgentRole;

    /// Get the agent's specialization
    fn specialization(&self) -> &str;

    /// Get the agent's capabilities as a list
    fn capabilities(&self) -> Vec<String>;

    /// Check if agent has a specific capability
    fn has_capability(&self, capability: &str) -> bool;
}

impl AgentAttributeAccess for ClaudeCodeAgent {
    fn agent_id(&self) -> &str {
        &self.identity.agent_id
    }

    fn role(&self) -> &AgentRole {
        &self.identity.specialization
    }

    fn specialization(&self) -> &str {
        self.identity.specialization.name()
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            self.identity.specialization.name().to_string(),
            self.identity.specialization.name().to_lowercase(),
        ]
    }

    fn has_capability(&self, capability: &str) -> bool {
        self.identity
            .specialization
            .name()
            .eq_ignore_ascii_case(capability)
    }
}

/// Helper to get agent role from specialization string
pub fn role_from_specialization(specialization: &str) -> AgentRole {
    match specialization {
        "react_typescript" | "frontend" | "Frontend" => crate::identity::default_frontend_role(),
        "node_microservices" | "backend" | "Backend" => crate::identity::default_backend_role(),
        "aws_kubernetes" | "devops" | "DevOps" => crate::identity::default_devops_role(),
        "qa" | "testing" | "QA" => crate::identity::default_qa_role(),
        _ => {
            tracing::warn!(
                "Unknown specialization: {}, defaulting to frontend role",
                specialization
            );
            crate::identity::default_frontend_role()
        }
    }
}

/// Helper to get specialization from role
pub fn specialization_from_role(role: &AgentRole) -> String {
    match role.name() {
        "Frontend" => "frontend".to_string(),
        "Backend" => "backend".to_string(),
        "DevOps" => "devops".to_string(),
        "QA" => "qa".to_string(),
        _ => role.name().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_attribute_access() {
        let agent = ClaudeCodeAgent::new(
            crate::identity::default_frontend_role(),
            &std::path::PathBuf::from("/tmp/test"),
            "test",
            crate::config::ClaudeConfig::default(),
        )
        .await
        .unwrap();

        assert_eq!(agent.role().name(), "Frontend");
        assert_eq!(agent.specialization(), "Frontend");
        assert!(agent.has_capability("Frontend"));
        assert!(agent.has_capability("frontend"));
        assert!(!agent.has_capability("Backend"));

        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 2);
        assert!(capabilities.contains(&"Frontend".to_string()));
    }

    #[test]
    fn test_role_conversions() {
        // Test role from specialization
        assert_eq!(role_from_specialization("frontend").name(), "Frontend");
        assert_eq!(
            role_from_specialization("react_typescript").name(),
            "Frontend"
        );
        assert_eq!(role_from_specialization("Backend").name(), "Backend");
        assert_eq!(role_from_specialization("unknown").name(), "Frontend"); // default

        // Test specialization from role
        let frontend_role = crate::identity::default_frontend_role();
        assert_eq!(specialization_from_role(&frontend_role), "frontend");

        let backend_role = crate::identity::default_backend_role();
        assert_eq!(specialization_from_role(&backend_role), "backend");
    }
}
