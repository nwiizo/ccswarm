pub mod boundary;

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Core agent identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Unique agent identifier
    pub agent_id: String,

    /// Agent's specialization role
    pub specialization: AgentRole,

    /// Workspace path for this agent
    pub workspace_path: PathBuf,

    /// Environment variables for role identification
    pub env_vars: HashMap<String, String>,

    /// Session identifier (unique per startup)
    pub session_id: String,

    /// Parent orchestrator process ID
    pub parent_process_id: String,

    /// Timestamp of agent initialization
    pub initialized_at: DateTime<Utc>,
}

/// Agent specialization roles with their specific configurations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentRole {
    Frontend {
        technologies: Vec<String>,
        responsibilities: Vec<String>,
        boundaries: Vec<String>,
    },
    Backend {
        technologies: Vec<String>,
        responsibilities: Vec<String>,
        boundaries: Vec<String>,
    },
    DevOps {
        technologies: Vec<String>,
        responsibilities: Vec<String>,
        boundaries: Vec<String>,
    },
    QA {
        technologies: Vec<String>,
        responsibilities: Vec<String>,
        boundaries: Vec<String>,
    },
    Master {
        oversight_roles: Vec<String>,
        quality_standards: QualityStandards,
    },
}

impl AgentRole {
    /// Get the name of the role
    pub fn name(&self) -> &str {
        match self {
            AgentRole::Frontend { .. } => "Frontend",
            AgentRole::Backend { .. } => "Backend",
            AgentRole::DevOps { .. } => "DevOps",
            AgentRole::QA { .. } => "QA",
            AgentRole::Master { .. } => "Master",
        }
    }

    /// Get the technologies associated with this role
    pub fn technologies(&self) -> Vec<String> {
        match self {
            AgentRole::Frontend { technologies, .. }
            | AgentRole::Backend { technologies, .. }
            | AgentRole::DevOps { technologies, .. }
            | AgentRole::QA { technologies, .. } => technologies.clone(),
            AgentRole::Master { .. } => vec!["Orchestration".to_string()],
        }
    }

    /// Get the responsibilities for this role
    pub fn responsibilities(&self) -> Vec<String> {
        match self {
            AgentRole::Frontend {
                responsibilities, ..
            }
            | AgentRole::Backend {
                responsibilities, ..
            }
            | AgentRole::DevOps {
                responsibilities, ..
            }
            | AgentRole::QA {
                responsibilities, ..
            } => responsibilities.clone(),
            AgentRole::Master {
                oversight_roles, ..
            } => oversight_roles.clone(),
        }
    }

    /// Get the boundaries for this role
    pub fn boundaries(&self) -> Vec<String> {
        match self {
            AgentRole::Frontend { boundaries, .. }
            | AgentRole::Backend { boundaries, .. }
            | AgentRole::DevOps { boundaries, .. }
            | AgentRole::QA { boundaries, .. } => boundaries.clone(),
            AgentRole::Master { .. } => vec!["No direct code implementation".to_string()],
        }
    }
}

/// Quality standards for code review and acceptance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityStandards {
    pub min_test_coverage: f64,
    pub max_complexity: u32,
    pub security_scan_required: bool,
    pub performance_threshold: Duration,
}

impl Default for QualityStandards {
    fn default() -> Self {
        Self {
            min_test_coverage: 0.85,
            max_complexity: 10,
            security_scan_required: true,
            performance_threshold: Duration::from_secs(5),
        }
    }
}

/// Identity monitoring status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityStatus {
    Healthy,
    DriftDetected(String),
    BoundaryViolation(String),
    CriticalFailure(String),
}

/// Identity monitor for tracking agent behavior
#[derive(Debug)]
pub struct IdentityMonitor {
    pub agent_id: String,
    pub last_identity_check: Instant,
    pub identity_drift_threshold: Duration,
    pub response_parser: ResponseParser,
}

impl IdentityMonitor {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            last_identity_check: Instant::now(),
            identity_drift_threshold: Duration::from_secs(300), // 5 minutes
            response_parser: ResponseParser::new(),
        }
    }

    /// Monitor a response for identity compliance
    pub async fn monitor_response(&mut self, response: &str) -> Result<IdentityStatus> {
        // Check for identity header
        let has_identity_header = self.check_identity_header(response);

        // Check boundary compliance
        let boundary_compliance = self.check_boundary_compliance(response);

        // Check delegation behavior
        let _delegation_behavior = self.check_delegation_behavior(response);

        if !has_identity_header {
            return Ok(IdentityStatus::DriftDetected(
                "Missing identity header".to_string(),
            ));
        }

        if !boundary_compliance {
            return Ok(IdentityStatus::BoundaryViolation(
                "Response indicates work outside specialization".to_string(),
            ));
        }

        self.last_identity_check = Instant::now();
        Ok(IdentityStatus::Healthy)
    }

    pub fn check_identity_header(&self, response: &str) -> bool {
        let required_pattern = format!("🤖 AGENT: {}", self.agent_id);
        response.contains(&required_pattern)
    }

    fn check_boundary_compliance(&self, response: &str) -> bool {
        // Check for indicators of boundary violations
        let violation_patterns = vec![
            r"working on backend code",
            r"modifying infrastructure",
            r"changing database schema",
        ];

        for pattern in violation_patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(response) {
                return false;
            }
        }
        true
    }

    fn check_delegation_behavior(&self, response: &str) -> bool {
        // Check for proper delegation patterns
        response.contains("DELEGATING TO:")
            || response.contains("outside my specialization")
            || !response.contains("I'll handle this")
    }

    /// Generate correction prompt for identity drift
    pub fn generate_correction_prompt(&self, workspace: &str, specialization: &str) -> String {
        format!(
            r#"
⚠️ IDENTITY DRIFT DETECTED

You seem to have forgotten your role. Let me remind you:

## YOUR IDENTITY
- You are the {} Agent
- Your workspace is {}
- You specialize ONLY in {}
- You must include identity headers in all responses

Please acknowledge your identity and continue with the current task while staying within your boundaries.

Remember to start your response with:
```
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: [Task assessment]
```
"#,
            self.agent_id, workspace, specialization, self.agent_id, workspace
        )
    }
}

/// Response parser for analyzing agent outputs
#[derive(Debug)]
pub struct ResponseParser {
    identity_regex: Regex,
    workspace_regex: Regex,
    scope_regex: Regex,
}

impl Default for ResponseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseParser {
    pub fn new() -> Self {
        Self {
            identity_regex: Regex::new(r"🤖 AGENT: (.+)").unwrap(),
            workspace_regex: Regex::new(r"📁 WORKSPACE: (.+)").unwrap(),
            scope_regex: Regex::new(r"🎯 SCOPE: (.+)").unwrap(),
        }
    }

    /// Parse identity information from response
    pub fn parse_identity(&self, response: &str) -> Option<(String, String, String)> {
        let agent = self
            .identity_regex
            .captures(response)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string());

        let workspace = self
            .workspace_regex
            .captures(response)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string());

        let scope = self
            .scope_regex
            .captures(response)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string());

        match (agent, workspace, scope) {
            (Some(a), Some(w), Some(s)) => Some((a, w, s)),
            _ => None,
        }
    }
}

/// Default role configurations
pub fn default_frontend_role() -> AgentRole {
    AgentRole::Frontend {
        technologies: vec![
            "React".to_string(),
            "TypeScript".to_string(),
            "Tailwind CSS".to_string(),
            "Jest".to_string(),
            "Vite".to_string(),
        ],
        responsibilities: vec![
            "UI Component Development".to_string(),
            "State Management".to_string(),
            "Frontend Testing".to_string(),
            "User Experience".to_string(),
            "Accessibility".to_string(),
        ],
        boundaries: vec![
            "No backend API development".to_string(),
            "No database operations".to_string(),
            "No server-side logic".to_string(),
            "No infrastructure changes".to_string(),
            "No deployment scripts".to_string(),
        ],
    }
}

pub fn default_backend_role() -> AgentRole {
    AgentRole::Backend {
        technologies: vec![
            "Node.js".to_string(),
            "TypeScript".to_string(),
            "Express".to_string(),
            "PostgreSQL".to_string(),
            "Prisma".to_string(),
        ],
        responsibilities: vec![
            "API Development".to_string(),
            "Database Design".to_string(),
            "Authentication".to_string(),
            "Business Logic".to_string(),
            "Data Validation".to_string(),
        ],
        boundaries: vec![
            "No frontend UI code".to_string(),
            "No CSS styling".to_string(),
            "No infrastructure provisioning".to_string(),
            "No deployment automation".to_string(),
        ],
    }
}

pub fn default_devops_role() -> AgentRole {
    AgentRole::DevOps {
        technologies: vec![
            "Docker".to_string(),
            "Kubernetes".to_string(),
            "Terraform".to_string(),
            "AWS".to_string(),
            "GitHub Actions".to_string(),
        ],
        responsibilities: vec![
            "Infrastructure Provisioning".to_string(),
            "CI/CD Pipelines".to_string(),
            "Monitoring Setup".to_string(),
            "Security Configuration".to_string(),
            "Deployment Automation".to_string(),
        ],
        boundaries: vec![
            "No application code changes".to_string(),
            "No business logic implementation".to_string(),
            "No UI development".to_string(),
            "No database schema design".to_string(),
        ],
    }
}

pub fn default_qa_role() -> AgentRole {
    AgentRole::QA {
        technologies: vec![
            "Jest".to_string(),
            "Cypress".to_string(),
            "Playwright".to_string(),
            "Postman".to_string(),
            "K6".to_string(),
        ],
        responsibilities: vec![
            "Test Strategy".to_string(),
            "Test Implementation".to_string(),
            "Quality Assurance".to_string(),
            "Performance Testing".to_string(),
            "Security Testing".to_string(),
        ],
        boundaries: vec![
            "No production code changes".to_string(),
            "No feature implementation".to_string(),
            "No infrastructure changes".to_string(),
            "No deployment execution".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_names() {
        let frontend = default_frontend_role();
        assert_eq!(frontend.name(), "Frontend");

        let backend = default_backend_role();
        assert_eq!(backend.name(), "Backend");
    }

    #[test]
    fn test_identity_monitor_header_check() {
        let monitor = IdentityMonitor::new("Frontend");

        let valid_response = "🤖 AGENT: Frontend\n📁 WORKSPACE: /test\n🎯 SCOPE: UI work";
        assert!(monitor.check_identity_header(valid_response));

        let invalid_response = "Working on the task...";
        assert!(!monitor.check_identity_header(invalid_response));
    }

    #[test]
    fn test_response_parser() {
        let parser = ResponseParser::new();
        let response = r#"
🤖 AGENT: Frontend
📁 WORKSPACE: agents/frontend-agent/
🎯 SCOPE: Component development

Working on React component...
"#;

        let parsed = parser.parse_identity(response);
        assert!(parsed.is_some());

        let (agent, workspace, scope) = parsed.unwrap();
        assert_eq!(agent, "Frontend");
        assert_eq!(workspace, "agents/frontend-agent/");
        assert_eq!(scope, "Component development");
    }
}
