//! Per-movement permission model for Piece/Movement workflows.
//!
//! Enforces tool and file access restrictions based on `MovementPermission`:
//! - **Readonly** — Can read files, search, but not modify anything
//! - **Edit** — Can modify existing files but not create/delete or run commands
//! - **Full** — Can create, delete, execute commands, full access
//!
//! Inspired by takt's per-step permission boundaries.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, warn};

use super::piece::MovementPermission;

/// Tools available at each permission level
const READONLY_TOOLS: &[&str] = &["read", "grep", "glob", "search", "list", "cat", "find"];
const EDIT_TOOLS: &[&str] = &[
    "read", "grep", "glob", "search", "list", "cat", "find", "edit", "write", "replace",
];
const FULL_TOOLS: &[&str] = &[
    "read", "grep", "glob", "search", "list", "cat", "find", "edit", "write", "replace", "bash",
    "shell", "exec", "delete", "create", "mkdir", "rm",
];

/// Permission enforcer that validates tool usage and file access
#[derive(Debug, Clone)]
pub struct PermissionEnforcer {
    /// Current permission level
    permission: MovementPermission,
    /// Explicitly allowed tools (overrides permission level)
    allowed_tools: HashSet<String>,
    /// Explicitly denied tools
    denied_tools: HashSet<String>,
    /// Protected file patterns that cannot be modified at any level
    protected_patterns: Vec<String>,
    /// Whether to enforce strictly (error) or permissively (warn)
    strict: bool,
}

/// Result of a permission check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    /// Whether the action is allowed
    pub allowed: bool,
    /// Reason for denial (if not allowed)
    pub reason: Option<String>,
    /// Permission level that was checked
    pub permission: String,
    /// The tool or action that was checked
    pub action: String,
}

/// A record of a permission violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionViolation {
    /// The movement that violated permissions
    pub movement_id: String,
    /// The tool or action attempted
    pub action: String,
    /// The permission level of the movement
    pub permission: String,
    /// Human-readable explanation
    pub message: String,
}

impl PermissionEnforcer {
    /// Create a new enforcer for the given permission level
    pub fn new(permission: MovementPermission) -> Self {
        Self {
            permission,
            allowed_tools: HashSet::new(),
            denied_tools: HashSet::new(),
            protected_patterns: vec![
                ".env".to_string(),
                "*.key".to_string(),
                "*.pem".to_string(),
                ".git/**".to_string(),
                "credentials*".to_string(),
            ],
            strict: true,
        }
    }

    /// Create from a movement's permission and explicit tool list
    pub fn from_movement(permission: MovementPermission, tools: &[String]) -> Self {
        let mut enforcer = Self::new(permission);
        if !tools.is_empty() {
            enforcer.allowed_tools = tools.iter().cloned().collect();
        }
        enforcer
    }

    /// Set strict mode (errors on violation) or permissive (warnings only)
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Add a protected file pattern
    pub fn add_protected_pattern(&mut self, pattern: &str) {
        self.protected_patterns.push(pattern.to_string());
    }

    /// Add an explicitly denied tool
    pub fn deny_tool(&mut self, tool: &str) {
        self.denied_tools.insert(tool.to_string());
    }

    /// Check if a tool is allowed at the current permission level
    pub fn check_tool(&self, tool: &str) -> PermissionCheckResult {
        let tool_lower = tool.to_lowercase();

        // Check explicit denials first
        if self.denied_tools.contains(&tool_lower) {
            return PermissionCheckResult {
                allowed: false,
                reason: Some(format!("Tool '{}' is explicitly denied", tool)),
                permission: format!("{:?}", self.permission),
                action: tool.to_string(),
            };
        }

        // Check explicit allowances (override permission level)
        if !self.allowed_tools.is_empty() {
            let allowed = self.allowed_tools.contains(&tool_lower);
            return PermissionCheckResult {
                allowed,
                reason: if allowed {
                    None
                } else {
                    Some(format!(
                        "Tool '{}' not in movement's allowed tools list",
                        tool
                    ))
                },
                permission: format!("{:?}", self.permission),
                action: tool.to_string(),
            };
        }

        // Check against permission level defaults
        let level_tools = match self.permission {
            MovementPermission::Readonly => READONLY_TOOLS,
            MovementPermission::Edit => EDIT_TOOLS,
            MovementPermission::Full => FULL_TOOLS,
        };

        let allowed = level_tools.iter().any(|t| *t == tool_lower);

        if !allowed {
            debug!(
                "Tool '{}' denied at {:?} permission level",
                tool, self.permission
            );
        }

        PermissionCheckResult {
            allowed,
            reason: if allowed {
                None
            } else {
                Some(format!(
                    "Tool '{}' not allowed at {:?} permission level",
                    tool, self.permission
                ))
            },
            permission: format!("{:?}", self.permission),
            action: tool.to_string(),
        }
    }

    /// Check if a file operation is allowed
    pub fn check_file_access(&self, path: &str, write: bool) -> PermissionCheckResult {
        // Check protected patterns
        for pattern in &self.protected_patterns {
            if matches_glob_pattern(pattern, path) {
                return PermissionCheckResult {
                    allowed: false,
                    reason: Some(format!(
                        "File '{}' matches protected pattern '{}'",
                        path, pattern
                    )),
                    permission: format!("{:?}", self.permission),
                    action: if write {
                        format!("write:{}", path)
                    } else {
                        format!("read:{}", path)
                    },
                };
            }
        }

        // Read access is allowed at all levels
        if !write {
            return PermissionCheckResult {
                allowed: true,
                reason: None,
                permission: format!("{:?}", self.permission),
                action: format!("read:{}", path),
            };
        }

        // Write access depends on permission level
        let allowed = matches!(
            self.permission,
            MovementPermission::Edit | MovementPermission::Full
        );

        PermissionCheckResult {
            allowed,
            reason: if allowed {
                None
            } else {
                Some(format!(
                    "Write access to '{}' denied at Readonly permission level",
                    path
                ))
            },
            permission: format!("{:?}", self.permission),
            action: format!("write:{}", path),
        }
    }

    /// Check if command execution is allowed
    pub fn check_command_execution(&self, command: &str) -> PermissionCheckResult {
        let allowed = self.permission == MovementPermission::Full;

        if !allowed {
            warn!(
                "Command execution denied at {:?} level: {}",
                self.permission, command
            );
        }

        PermissionCheckResult {
            allowed,
            reason: if allowed {
                None
            } else {
                Some(format!(
                    "Command execution denied at {:?} permission level (requires Full)",
                    self.permission
                ))
            },
            permission: format!("{:?}", self.permission),
            action: format!("exec:{}", command),
        }
    }

    /// Get all tools available at the current permission level
    pub fn available_tools(&self) -> Vec<String> {
        if !self.allowed_tools.is_empty() {
            // Explicit tool list takes precedence
            return self
                .allowed_tools
                .iter()
                .filter(|t| !self.denied_tools.contains(*t))
                .cloned()
                .collect();
        }

        let level_tools = match self.permission {
            MovementPermission::Readonly => READONLY_TOOLS,
            MovementPermission::Edit => EDIT_TOOLS,
            MovementPermission::Full => FULL_TOOLS,
        };

        level_tools
            .iter()
            .map(|s| s.to_string())
            .filter(|t| !self.denied_tools.contains(t))
            .collect()
    }

    /// Whether this enforcer is in strict mode
    pub fn is_strict(&self) -> bool {
        self.strict
    }
}

/// Simple glob-style pattern matching for file paths
fn matches_glob_pattern(pattern: &str, path: &str) -> bool {
    let path_obj = Path::new(path);
    let filename = path_obj.file_name().and_then(|f| f.to_str()).unwrap_or("");

    // Handle ** (any subdirectory)
    if pattern.contains("**") {
        let prefix = pattern.split("**").next().unwrap_or("");
        return path.starts_with(prefix.trim_end_matches('/'));
    }

    // Handle * wildcard in filename
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let (prefix, suffix) = (parts[0], parts[1]);
            return filename.starts_with(prefix) && filename.ends_with(suffix);
        }
    }

    // Exact match (filename or full path)
    filename == pattern || path == pattern || path.ends_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_permission() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Readonly);

        assert!(enforcer.check_tool("read").allowed);
        assert!(enforcer.check_tool("grep").allowed);
        assert!(enforcer.check_tool("glob").allowed);
        assert!(!enforcer.check_tool("edit").allowed);
        assert!(!enforcer.check_tool("write").allowed);
        assert!(!enforcer.check_tool("bash").allowed);
    }

    #[test]
    fn test_edit_permission() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Edit);

        assert!(enforcer.check_tool("read").allowed);
        assert!(enforcer.check_tool("edit").allowed);
        assert!(enforcer.check_tool("write").allowed);
        assert!(!enforcer.check_tool("bash").allowed);
        assert!(!enforcer.check_tool("delete").allowed);
    }

    #[test]
    fn test_full_permission() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Full);

        assert!(enforcer.check_tool("read").allowed);
        assert!(enforcer.check_tool("edit").allowed);
        assert!(enforcer.check_tool("bash").allowed);
        assert!(enforcer.check_tool("delete").allowed);
    }

    #[test]
    fn test_explicit_tool_list() {
        let enforcer = PermissionEnforcer::from_movement(
            MovementPermission::Full,
            &["read".to_string(), "grep".to_string()],
        );

        // Only explicitly listed tools allowed despite Full permission
        assert!(enforcer.check_tool("read").allowed);
        assert!(enforcer.check_tool("grep").allowed);
        assert!(!enforcer.check_tool("bash").allowed);
    }

    #[test]
    fn test_denied_tools() {
        let mut enforcer = PermissionEnforcer::new(MovementPermission::Full);
        enforcer.deny_tool("bash");

        assert!(!enforcer.check_tool("bash").allowed);
        assert!(enforcer.check_tool("read").allowed);
    }

    #[test]
    fn test_file_access_readonly() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Readonly);

        assert!(enforcer.check_file_access("src/main.rs", false).allowed);
        assert!(!enforcer.check_file_access("src/main.rs", true).allowed);
    }

    #[test]
    fn test_file_access_edit() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Edit);

        assert!(enforcer.check_file_access("src/main.rs", false).allowed);
        assert!(enforcer.check_file_access("src/main.rs", true).allowed);
    }

    #[test]
    fn test_protected_files() {
        let enforcer = PermissionEnforcer::new(MovementPermission::Full);

        assert!(!enforcer.check_file_access(".env", false).allowed);
        assert!(!enforcer.check_file_access("secret.key", false).allowed);
        assert!(!enforcer.check_file_access("server.pem", false).allowed);
        assert!(!enforcer.check_file_access(".git/config", false).allowed);
    }

    #[test]
    fn test_command_execution() {
        let readonly = PermissionEnforcer::new(MovementPermission::Readonly);
        let edit = PermissionEnforcer::new(MovementPermission::Edit);
        let full = PermissionEnforcer::new(MovementPermission::Full);

        assert!(!readonly.check_command_execution("cargo test").allowed);
        assert!(!edit.check_command_execution("cargo test").allowed);
        assert!(full.check_command_execution("cargo test").allowed);
    }

    #[test]
    fn test_available_tools() {
        let readonly = PermissionEnforcer::new(MovementPermission::Readonly);
        let tools = readonly.available_tools();
        assert!(tools.contains(&"read".to_string()));
        assert!(!tools.contains(&"bash".to_string()));
    }

    #[test]
    fn test_glob_patterns() {
        assert!(matches_glob_pattern("*.key", "secret.key"));
        assert!(matches_glob_pattern("*.pem", "cert.pem"));
        assert!(matches_glob_pattern(".env", ".env"));
        assert!(matches_glob_pattern(".git/**", ".git/config"));
        assert!(!matches_glob_pattern("*.key", "main.rs"));
    }

    #[test]
    fn test_custom_protected_pattern() {
        let mut enforcer = PermissionEnforcer::new(MovementPermission::Full);
        enforcer.add_protected_pattern("*.secret");

        assert!(!enforcer.check_file_access("db.secret", false).allowed);
        assert!(enforcer.check_file_access("db.config", false).allowed);
    }
}
