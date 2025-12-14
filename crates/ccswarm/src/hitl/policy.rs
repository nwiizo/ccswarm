//! Approval policies for HITL decisions

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::approval::ApprovalRequest;

/// Risk level of an action
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default,
)]
pub enum RiskLevel {
    /// No risk, can be auto-approved
    None,
    /// Low risk, minor changes
    Low,
    /// Medium risk, needs attention
    #[default]
    Medium,
    /// High risk, requires approval
    High,
    /// Critical risk, requires explicit approval
    Critical,
}

impl RiskLevel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            RiskLevel::None => "None",
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }

    /// Get color for display
    pub fn color(&self) -> &'static str {
        match self {
            RiskLevel::None => "green",
            RiskLevel::Low => "blue",
            RiskLevel::Medium => "yellow",
            RiskLevel::High => "orange",
            RiskLevel::Critical => "red",
        }
    }
}

/// Type of action being performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ActionType {
    /// Reading a file
    FileRead,
    /// Writing to a file
    FileWrite,
    /// Deleting a file
    FileDelete,
    /// Executing a system command
    SystemCommand,
    /// Deploying to an environment
    Deploy,
    /// Modifying database
    DatabaseModify,
    /// Network request
    NetworkRequest,
    /// Git operation
    GitOperation,
    /// Environment variable change
    EnvChange,
    /// Configuration change
    ConfigChange,
    /// API call
    ApiCall,
    /// Custom action type
    Custom,
}

impl ActionType {
    /// Get default risk level for this action type
    pub fn default_risk(&self) -> RiskLevel {
        match self {
            ActionType::FileRead => RiskLevel::None,
            ActionType::FileWrite => RiskLevel::Low,
            ActionType::FileDelete => RiskLevel::High,
            ActionType::SystemCommand => RiskLevel::High,
            ActionType::Deploy => RiskLevel::Critical,
            ActionType::DatabaseModify => RiskLevel::Critical,
            ActionType::NetworkRequest => RiskLevel::Medium,
            ActionType::GitOperation => RiskLevel::Medium,
            ActionType::EnvChange => RiskLevel::High,
            ActionType::ConfigChange => RiskLevel::High,
            ActionType::ApiCall => RiskLevel::Medium,
            ActionType::Custom => RiskLevel::Medium,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ActionType::FileRead => "File Read",
            ActionType::FileWrite => "File Write",
            ActionType::FileDelete => "File Delete",
            ActionType::SystemCommand => "System Command",
            ActionType::Deploy => "Deploy",
            ActionType::DatabaseModify => "Database Modify",
            ActionType::NetworkRequest => "Network Request",
            ActionType::GitOperation => "Git Operation",
            ActionType::EnvChange => "Environment Change",
            ActionType::ConfigChange => "Config Change",
            ActionType::ApiCall => "API Call",
            ActionType::Custom => "Custom",
        }
    }
}

/// An approval policy that defines when approval is required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
    /// Whether this policy is enabled
    pub enabled: bool,
    /// Priority (higher = evaluated first)
    pub priority: i32,
    /// Rules that trigger this policy
    pub rules: Vec<PolicyRule>,
    /// Required approvers
    pub required_approvers: Vec<String>,
    /// Minimum approvals needed
    pub min_approvals: usize,
    /// Allowed approvers (empty = anyone)
    pub allowed_approvers: HashSet<String>,
}

impl ApprovalPolicy {
    /// Create a new policy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            enabled: true,
            priority: 0,
            rules: Vec::new(),
            required_approvers: Vec::new(),
            min_approvals: 1,
            allowed_approvers: HashSet::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a rule
    pub fn with_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set minimum approvals
    pub fn with_min_approvals(mut self, min: usize) -> Self {
        self.min_approvals = min;
        self
    }

    /// Add required approver
    pub fn with_required_approver(mut self, approver: impl Into<String>) -> Self {
        self.required_approvers.push(approver.into());
        self
    }

    /// Add allowed approver
    pub fn with_allowed_approver(mut self, approver: impl Into<String>) -> Self {
        self.allowed_approvers.insert(approver.into());
        self
    }

    /// Check if this policy matches a request
    pub fn matches(&self, request: &ApprovalRequest) -> bool {
        if !self.enabled {
            return false;
        }

        // Must match at least one rule
        self.rules.iter().any(|rule| rule.matches(request))
    }

    /// Check if an approver is allowed
    pub fn can_approve(&self, approver: &str) -> bool {
        // If no restrictions, anyone can approve
        if self.allowed_approvers.is_empty() {
            return true;
        }

        self.allowed_approvers.contains(approver)
    }

    /// Check if required approvers have all approved
    pub fn all_required_approved(&self, approvers: &[String]) -> bool {
        self.required_approvers
            .iter()
            .all(|req| approvers.contains(req))
    }
}

/// A rule within a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyRule {
    /// Require approval for specific action types
    RequireApproval {
        action_types: Vec<ActionType>,
        environments: Option<Vec<String>>,
    },
    /// Require approval above a risk level
    RiskThreshold { min_level: RiskLevel },
    /// Require approval for specific file patterns
    FilePattern { patterns: Vec<String> },
    /// Require approval for specific commands
    CommandPattern { patterns: Vec<String> },
    /// Require approval for specific agents
    AgentRestriction { agent_ids: Vec<String> },
    /// Time-based restriction
    TimeRestriction {
        /// Hours during which approval is required (0-23)
        require_during_hours: Vec<u32>,
        /// Days during which approval is required (0=Sun, 6=Sat)
        require_during_days: Vec<u32>,
    },
    /// Always require approval
    AlwaysRequire,
    /// Never require approval
    NeverRequire,
    /// Custom condition
    Custom { condition: String },
}

impl PolicyRule {
    /// Check if this rule matches a request
    pub fn matches(&self, request: &ApprovalRequest) -> bool {
        match self {
            PolicyRule::RequireApproval {
                action_types,
                environments,
            } => {
                let type_matches = action_types.contains(&request.action_type);
                let env_matches = environments.as_ref().is_none_or(|envs| {
                    request
                        .environment
                        .as_ref()
                        .is_some_and(|e| envs.contains(e))
                });
                type_matches && env_matches
            }
            PolicyRule::RiskThreshold { min_level } => request.risk_level >= *min_level,
            PolicyRule::FilePattern { patterns } => request.affected_files.iter().any(|file| {
                patterns
                    .iter()
                    .any(|pattern| file_matches_pattern(file, pattern))
            }),
            PolicyRule::CommandPattern { patterns } => request.commands.iter().any(|cmd| {
                patterns
                    .iter()
                    .any(|pattern| command_matches_pattern(cmd, pattern))
            }),
            PolicyRule::AgentRestriction { agent_ids } => request
                .agent_id
                .as_ref()
                .is_some_and(|id| agent_ids.contains(id)),
            PolicyRule::TimeRestriction {
                require_during_hours,
                require_during_days,
            } => {
                let now = chrono::Utc::now();
                let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
                let day = now.format("%w").to_string().parse::<u32>().unwrap_or(0);

                let hour_match =
                    require_during_hours.is_empty() || require_during_hours.contains(&hour);
                let day_match =
                    require_during_days.is_empty() || require_during_days.contains(&day);

                hour_match && day_match
            }
            PolicyRule::AlwaysRequire => true,
            PolicyRule::NeverRequire => false,
            PolicyRule::Custom { condition: _ } => {
                // Custom conditions would need a scripting engine
                false
            }
        }
    }
}

/// Check if a file path matches a glob-like pattern
fn file_matches_pattern(file: &str, pattern: &str) -> bool {
    // Simple pattern matching (supports * wildcard)
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return file.starts_with(prefix) && file.ends_with(suffix);
        }
    }
    file == pattern || file.ends_with(pattern)
}

/// Check if a command matches a pattern
fn command_matches_pattern(cmd: &str, pattern: &str) -> bool {
    // Simple pattern matching
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return cmd.starts_with(prefix) && cmd.ends_with(suffix);
        }
    }
    cmd.contains(pattern)
}

/// Predefined policies for common scenarios
#[allow(dead_code)]
pub struct PredefinedPolicies;

#[allow(dead_code)]
impl PredefinedPolicies {
    /// Block production deployments without approval
    pub fn production_deploy() -> ApprovalPolicy {
        ApprovalPolicy::new("production_deploy")
            .with_description("Require approval for production deployments")
            .with_priority(100)
            .with_rule(PolicyRule::RequireApproval {
                action_types: vec![ActionType::Deploy],
                environments: Some(vec!["production".to_string(), "prod".to_string()]),
            })
    }

    /// Require approval for all critical operations
    pub fn critical_operations() -> ApprovalPolicy {
        ApprovalPolicy::new("critical_operations")
            .with_description("Require approval for critical risk operations")
            .with_priority(90)
            .with_rule(PolicyRule::RiskThreshold {
                min_level: RiskLevel::Critical,
            })
    }

    /// Protect sensitive files
    pub fn sensitive_files() -> ApprovalPolicy {
        ApprovalPolicy::new("sensitive_files")
            .with_description("Require approval for sensitive file modifications")
            .with_priority(80)
            .with_rule(PolicyRule::FilePattern {
                patterns: vec![
                    ".env".to_string(),
                    "*.key".to_string(),
                    "*.pem".to_string(),
                    "*secret*".to_string(),
                    "*password*".to_string(),
                    "credentials*".to_string(),
                ],
            })
    }

    /// Block dangerous commands
    pub fn dangerous_commands() -> ApprovalPolicy {
        ApprovalPolicy::new("dangerous_commands")
            .with_description("Require approval for dangerous commands")
            .with_priority(70)
            .with_rule(PolicyRule::CommandPattern {
                patterns: vec![
                    "rm -rf".to_string(),
                    "drop database".to_string(),
                    "DELETE FROM".to_string(),
                    "TRUNCATE".to_string(),
                    "git push --force".to_string(),
                    "git reset --hard".to_string(),
                ],
            })
    }

    /// After-hours restrictions
    pub fn after_hours() -> ApprovalPolicy {
        ApprovalPolicy::new("after_hours")
            .with_description("Require approval outside business hours")
            .with_priority(50)
            .with_rule(PolicyRule::TimeRestriction {
                // Require approval between 6 PM and 8 AM
                require_during_hours: (0..8).chain(18..24).collect(),
                // And on weekends
                require_during_days: vec![0, 6], // Sunday, Saturday
            })
    }

    /// Get all predefined policies
    pub fn all() -> Vec<ApprovalPolicy> {
        vec![
            Self::production_deploy(),
            Self::critical_operations(),
            Self::sensitive_files(),
            Self::dangerous_commands(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
        assert!(RiskLevel::Low > RiskLevel::None);
    }

    #[test]
    fn test_action_type_default_risk() {
        assert_eq!(ActionType::FileRead.default_risk(), RiskLevel::None);
        assert_eq!(ActionType::Deploy.default_risk(), RiskLevel::Critical);
        assert_eq!(ActionType::FileDelete.default_risk(), RiskLevel::High);
    }

    #[test]
    fn test_policy_creation() {
        let policy = ApprovalPolicy::new("test_policy")
            .with_description("Test description")
            .with_priority(10)
            .with_rule(PolicyRule::RiskThreshold {
                min_level: RiskLevel::High,
            });

        assert_eq!(policy.name, "test_policy");
        assert_eq!(policy.priority, 10);
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_policy_matching() {
        let policy = ApprovalPolicy::new("production").with_rule(PolicyRule::RequireApproval {
            action_types: vec![ActionType::Deploy],
            environments: Some(vec!["production".to_string()]),
        });

        let request_prod = ApprovalRequest::new("Deploy", ActionType::Deploy, RiskLevel::High)
            .with_environment("production");

        let request_dev = ApprovalRequest::new("Deploy", ActionType::Deploy, RiskLevel::Medium)
            .with_environment("development");

        assert!(policy.matches(&request_prod));
        assert!(!policy.matches(&request_dev));
    }

    #[test]
    fn test_risk_threshold_rule() {
        let rule = PolicyRule::RiskThreshold {
            min_level: RiskLevel::High,
        };

        let high_risk = ApprovalRequest::new("Delete", ActionType::FileDelete, RiskLevel::High);
        let low_risk = ApprovalRequest::new("Read", ActionType::FileRead, RiskLevel::Low);

        assert!(rule.matches(&high_risk));
        assert!(!rule.matches(&low_risk));
    }

    #[test]
    fn test_file_pattern_rule() {
        let rule = PolicyRule::FilePattern {
            patterns: vec!["*.env".to_string(), ".secret*".to_string()],
        };

        let matches = ApprovalRequest::new("Edit", ActionType::FileWrite, RiskLevel::Medium)
            .with_files(vec!["production.env".to_string()]);

        let no_match = ApprovalRequest::new("Edit", ActionType::FileWrite, RiskLevel::Medium)
            .with_files(vec!["README.md".to_string()]);

        assert!(rule.matches(&matches));
        assert!(!rule.matches(&no_match));
    }

    #[test]
    fn test_command_pattern_rule() {
        let rule = PolicyRule::CommandPattern {
            patterns: vec!["rm -rf".to_string(), "drop database".to_string()],
        };

        let dangerous = ApprovalRequest::new("Delete", ActionType::SystemCommand, RiskLevel::High)
            .with_commands(vec!["rm -rf /tmp/test".to_string()]);

        let safe = ApprovalRequest::new("List", ActionType::SystemCommand, RiskLevel::Low)
            .with_commands(vec!["ls -la".to_string()]);

        assert!(rule.matches(&dangerous));
        assert!(!rule.matches(&safe));
    }

    #[test]
    fn test_file_matches_pattern() {
        assert!(file_matches_pattern("config.env", "*.env"));
        assert!(file_matches_pattern(".env", ".env"));
        assert!(file_matches_pattern("src/secret.key", "*.key"));
        assert!(!file_matches_pattern("readme.md", "*.env"));
    }

    #[test]
    fn test_predefined_policies() {
        let policies = PredefinedPolicies::all();
        assert!(!policies.is_empty());

        let production_policy = PredefinedPolicies::production_deploy();
        assert_eq!(production_policy.name, "production_deploy");
    }

    #[test]
    fn test_approver_restrictions() {
        let policy = ApprovalPolicy::new("restricted")
            .with_allowed_approver("admin")
            .with_allowed_approver("lead");

        assert!(policy.can_approve("admin"));
        assert!(policy.can_approve("lead"));
        assert!(!policy.can_approve("dev"));
    }

    #[test]
    fn test_required_approvers() {
        let policy = ApprovalPolicy::new("multi_approval")
            .with_required_approver("security")
            .with_required_approver("lead");

        let partial = vec!["security".to_string()];
        let complete = vec!["security".to_string(), "lead".to_string()];

        assert!(!policy.all_required_approved(&partial));
        assert!(policy.all_required_approved(&complete));
    }
}
