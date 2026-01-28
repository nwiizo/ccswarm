//! Built-in hooks for common use cases
//!
//! Provides ready-to-use hooks for logging and security validation.

use super::*;
use std::collections::HashSet;

/// Logging hook that records all hook events
pub struct LoggingHook {
    /// Whether to log pre-execution events
    pub log_pre_execution: bool,
    /// Whether to log post-execution events
    pub log_post_execution: bool,
    /// Whether to log tool events
    pub log_tool_events: bool,
    /// Log level for events
    pub log_level: LogLevel,
}

/// Log level for the logging hook
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingHook {
    /// Create a new logging hook with default settings
    pub fn new() -> Self {
        Self {
            log_pre_execution: true,
            log_post_execution: true,
            log_tool_events: true,
            log_level: LogLevel::Info,
        }
    }

    /// Set log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Disable pre-execution logging
    pub fn without_pre_execution(mut self) -> Self {
        self.log_pre_execution = false;
        self
    }

    /// Disable post-execution logging
    pub fn without_post_execution(mut self) -> Self {
        self.log_post_execution = false;
        self
    }

    /// Disable tool event logging
    pub fn without_tool_events(mut self) -> Self {
        self.log_tool_events = false;
        self
    }

    fn log(&self, message: &str) {
        match self.log_level {
            LogLevel::Debug => tracing::debug!("{}", message),
            LogLevel::Info => tracing::info!("{}", message),
            LogLevel::Warn => tracing::warn!("{}", message),
        }
    }
}

#[async_trait]
impl ExecutionHooks for LoggingHook {
    async fn pre_execution(&self, input: PreExecutionInput, ctx: HookContext) -> HookResult {
        if self.log_pre_execution {
            self.log(&format!(
                "[{}] Pre-execution: task='{}' type='{}' priority='{}'",
                ctx.agent_id, input.task_description, input.task_type, input.priority
            ));
        }
        HookResult::Continue
    }

    async fn post_execution(&self, input: PostExecutionInput, ctx: HookContext) -> HookResult {
        if self.log_post_execution {
            self.log(&format!(
                "[{}] Post-execution: task='{}' success={} duration={}ms",
                ctx.agent_id, input.task_description, input.success, input.duration_ms
            ));
        }
        HookResult::Continue
    }

    async fn on_error(&self, input: OnErrorInput, ctx: HookContext) -> HookResult {
        tracing::error!(
            agent_id = %ctx.agent_id,
            error_type = %input.error_type,
            recoverable = input.is_recoverable,
            "Error occurred: {}",
            input.error_message
        );
        HookResult::Continue
    }

    fn name(&self) -> &str {
        "logging"
    }

    fn priority(&self) -> i32 {
        -100 // Run last (lowest priority)
    }
}

#[async_trait]
impl ToolHooks for LoggingHook {
    async fn pre_tool_use(&self, input: PreToolUseInput, ctx: HookContext) -> HookResult {
        if self.log_tool_events {
            self.log(&format!(
                "[{}] Pre-tool-use: tool='{}' args={}",
                ctx.agent_id,
                input.tool_name,
                serde_json::to_string(&input.arguments).unwrap_or_default()
            ));
        }
        HookResult::Continue
    }

    async fn post_tool_use(&self, input: PostToolUseInput, ctx: HookContext) -> HookResult {
        if self.log_tool_events {
            self.log(&format!(
                "[{}] Post-tool-use: tool='{}' success={} duration={}ms",
                ctx.agent_id, input.tool_name, input.success, input.duration_ms
            ));
        }
        HookResult::Continue
    }

    fn name(&self) -> &str {
        "logging"
    }

    fn priority(&self) -> i32 {
        -100
    }
}

/// Security hook that enforces file and command restrictions
pub struct SecurityHook {
    /// Protected file patterns (glob-like)
    protected_patterns: HashSet<String>,
    /// Blocked commands
    blocked_commands: HashSet<String>,
    /// Whether to allow destructive git operations
    allow_destructive_git: bool,
}

impl Default for SecurityHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityHook {
    /// Create a new security hook with default protections
    pub fn new() -> Self {
        let mut protected = HashSet::new();
        // Default protected patterns
        protected.insert(".env".to_string());
        protected.insert("*.key".to_string());
        protected.insert("*.pem".to_string());
        protected.insert("credentials.json".to_string());
        protected.insert("secrets.yaml".to_string());
        protected.insert(".git/config".to_string());

        let mut blocked = HashSet::new();
        // Default blocked commands
        blocked.insert("rm -rf /".to_string());
        blocked.insert("rm -rf /*".to_string());
        blocked.insert(":(){:|:&};:".to_string()); // Fork bomb

        Self {
            protected_patterns: protected,
            blocked_commands: blocked,
            allow_destructive_git: false,
        }
    }

    /// Add a protected file pattern
    pub fn protect_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.protected_patterns.insert(pattern.into());
        self
    }

    /// Block a command
    pub fn block_command(mut self, command: impl Into<String>) -> Self {
        self.blocked_commands.insert(command.into());
        self
    }

    /// Allow destructive git operations
    pub fn allow_destructive_git(mut self) -> Self {
        self.allow_destructive_git = true;
        self
    }

    /// Check if a file path matches protected patterns
    fn is_protected(&self, path: &str) -> bool {
        for pattern in &self.protected_patterns {
            if let Some(suffix) = pattern.strip_prefix('*') {
                // Simple suffix match for *.ext patterns
                if path.ends_with(suffix) {
                    return true;
                }
            } else if path.contains(pattern) || path == pattern {
                return true;
            }
        }
        false
    }

    /// Check if a command is blocked
    fn is_blocked_command(&self, command: &str) -> Option<&str> {
        for blocked in &self.blocked_commands {
            if command.contains(blocked) {
                return Some(blocked);
            }
        }

        // Check for destructive git commands
        if !self.allow_destructive_git {
            if command.contains("git push --force")
                || command.contains("git reset --hard")
                || command.contains("git clean -f")
            {
                return Some("destructive git operation");
            }
        }

        None
    }
}

#[async_trait]
impl ExecutionHooks for SecurityHook {
    async fn pre_execution(&self, input: PreExecutionInput, _ctx: HookContext) -> HookResult {
        // Check task description for suspicious patterns
        let desc_lower = input.task_description.to_lowercase();
        if desc_lower.contains("delete all")
            || desc_lower.contains("remove all")
            || desc_lower.contains("drop database")
        {
            return HookResult::Deny {
                reason: "Task description contains potentially destructive operation".to_string(),
            };
        }
        HookResult::Continue
    }

    async fn post_execution(&self, _input: PostExecutionInput, _ctx: HookContext) -> HookResult {
        HookResult::Continue
    }

    async fn on_error(&self, _input: OnErrorInput, _ctx: HookContext) -> HookResult {
        HookResult::Continue
    }

    fn name(&self) -> &str {
        "security"
    }

    fn priority(&self) -> i32 {
        100 // Run first (highest priority)
    }
}

#[async_trait]
impl ToolHooks for SecurityHook {
    async fn pre_tool_use(&self, input: PreToolUseInput, _ctx: HookContext) -> HookResult {
        // Check for file operations on protected files
        if input.tool_name == "Write" || input.tool_name == "Edit" {
            if let Some(path) = input.arguments.get("file_path").and_then(|v| v.as_str()) {
                if self.is_protected(path) {
                    return HookResult::Deny {
                        reason: format!("Cannot modify protected file: {}", path),
                    };
                }
            }
        }

        // Check for blocked shell commands
        if input.tool_name == "Bash" {
            if let Some(command) = input.arguments.get("command").and_then(|v| v.as_str()) {
                if let Some(blocked) = self.is_blocked_command(command) {
                    return HookResult::Deny {
                        reason: format!("Blocked command detected: {}", blocked),
                    };
                }
            }
        }

        HookResult::Continue
    }

    async fn post_tool_use(&self, _input: PostToolUseInput, _ctx: HookContext) -> HookResult {
        HookResult::Continue
    }

    fn name(&self) -> &str {
        "security"
    }

    fn priority(&self) -> i32 {
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_hook_protected_files() {
        let hook = SecurityHook::new();
        let ctx = HookContext::new("test-agent");

        // Test .env protection
        let input = PreToolUseInput {
            tool_name: "Write".to_string(),
            arguments: serde_json::json!({
                "file_path": "/project/.env"
            }),
            description: None,
        };
        let result = hook.pre_tool_use(input, ctx.clone()).await;
        assert!(result.is_denied());

        // Test *.key protection
        let input = PreToolUseInput {
            tool_name: "Edit".to_string(),
            arguments: serde_json::json!({
                "file_path": "/home/user/private.key"
            }),
            description: None,
        };
        let result = hook.pre_tool_use(input, ctx.clone()).await;
        assert!(result.is_denied());

        // Test allowed file
        let input = PreToolUseInput {
            tool_name: "Write".to_string(),
            arguments: serde_json::json!({
                "file_path": "/project/src/main.rs"
            }),
            description: None,
        };
        let result = hook.pre_tool_use(input, ctx).await;
        assert!(result.should_continue());
    }

    #[tokio::test]
    async fn test_security_hook_destructive_git() {
        let hook = SecurityHook::new();
        let ctx = HookContext::new("test-agent");

        let input = PreToolUseInput {
            tool_name: "Bash".to_string(),
            arguments: serde_json::json!({
                "command": "git push --force origin main"
            }),
            description: None,
        };
        let result = hook.pre_tool_use(input, ctx).await;
        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_security_hook_allowed_with_flag() {
        let hook = SecurityHook::new().allow_destructive_git();
        let ctx = HookContext::new("test-agent");

        let input = PreToolUseInput {
            tool_name: "Bash".to_string(),
            arguments: serde_json::json!({
                "command": "git push --force origin main"
            }),
            description: None,
        };
        let result = hook.pre_tool_use(input, ctx).await;
        assert!(result.should_continue());
    }
}
