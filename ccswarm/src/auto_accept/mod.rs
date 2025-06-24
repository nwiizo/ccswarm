use crate::agent::task::Task;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for auto-accept mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAcceptConfig {
    /// Whether auto-accept is enabled
    pub enabled: bool,

    /// Operations that are trusted and can be auto-accepted
    pub trusted_operations: Vec<OperationType>,

    /// Maximum number of file changes allowed in a single operation
    pub max_file_changes: usize,

    /// Whether tests must pass for auto-accept to trigger
    pub require_tests_pass: bool,

    /// Maximum execution time in seconds before requiring manual approval
    pub max_execution_time: u32,

    /// File patterns that require manual approval (e.g., "*.sql", "Cargo.toml")
    pub restricted_files: Vec<String>,

    /// Whether to require git status to be clean before auto-accepting
    pub require_clean_git: bool,

    /// Emergency stop - if true, all auto-accept is disabled
    pub emergency_stop: bool,
}

impl Default for AutoAcceptConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Conservative default - must be explicitly enabled
            trusted_operations: vec![
                OperationType::ReadFile,
                OperationType::FormatCode,
                OperationType::RunTests,
                OperationType::LintCode,
            ],
            max_file_changes: 5,
            require_tests_pass: true,
            max_execution_time: 300, // 5 minutes
            restricted_files: vec![
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "*.sql".to_string(),
                "*.env".to_string(),
                "**/migrations/*".to_string(),
            ],
            require_clean_git: true,
            emergency_stop: false,
        }
    }
}

/// Types of operations that can be evaluated for auto-acceptance
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Reading files or directories
    ReadFile,

    /// Writing new files
    WriteFile,

    /// Editing existing files
    EditFile,

    /// Deleting files
    DeleteFile,

    /// Running tests
    RunTests,

    /// Code formatting operations
    FormatCode,

    /// Linting code
    LintCode,

    /// Git operations (commit, push, etc.)
    GitOperation,

    /// Installing dependencies
    InstallDependencies,

    /// Running build commands
    Build,

    /// Database operations
    DatabaseOperation,

    /// Network requests
    NetworkRequest,

    /// System commands
    SystemCommand,

    /// Creating directories
    CreateDirectory,

    /// Other/unknown operations
    Other,
}

/// Represents an operation that needs to be evaluated for auto-acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Type of operation
    pub operation_type: OperationType,

    /// Description of what the operation will do
    pub description: String,

    /// Files that will be affected
    pub affected_files: Vec<PathBuf>,

    /// Commands that will be executed
    pub commands: Vec<String>,

    /// Estimated risk level (0-10, where 10 is highest risk)
    pub risk_level: u8,

    /// Whether this operation is reversible
    pub reversible: bool,

    /// Associated task information
    pub task: Option<Task>,
}

/// Engine for evaluating whether operations should be auto-accepted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAcceptEngine {
    pub config: AutoAcceptConfig,
    operation_history: HashMap<String, Vec<Operation>>,
}

impl AutoAcceptEngine {
    /// Create a new auto-accept engine with the given configuration
    pub fn new(config: AutoAcceptConfig) -> Self {
        Self {
            config,
            operation_history: HashMap::new(),
        }
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: AutoAcceptConfig) {
        self.config = config;
    }

    /// Emergency stop - disable all auto-accept immediately
    pub fn emergency_stop(&mut self) {
        self.config.emergency_stop = true;
        self.config.enabled = false;
    }

    /// Re-enable auto-accept after emergency stop (requires manual intervention)
    pub fn reset_emergency_stop(&mut self) {
        self.config.emergency_stop = false;
    }

    /// Analyze an operation and determine its characteristics
    pub fn analyze_operation(&self, commands: &[String], task: Option<&Task>) -> Result<Operation> {
        let mut operation = Operation {
            operation_type: OperationType::Other,
            description: format!("Commands: {}", commands.join("; ")),
            affected_files: Vec::new(),
            commands: commands.to_vec(),
            risk_level: 5, // Default medium risk
            reversible: false,
            task: task.cloned(),
        };

        // Analyze commands to determine operation type and risk
        for command in commands {
            let cmd_lower = command.to_lowercase();

            // Determine operation type
            if cmd_lower.contains("cat ")
                || cmd_lower.contains("ls ")
                || cmd_lower.contains("find ")
            {
                operation.operation_type = OperationType::ReadFile;
                operation.risk_level = operation.risk_level.min(1);
                operation.reversible = true;
            } else if cmd_lower.contains("echo ") && cmd_lower.contains(" > ") {
                operation.operation_type = OperationType::WriteFile;
                operation.risk_level = operation.risk_level.max(4);
            } else if cmd_lower.contains("sed ")
                || cmd_lower.contains("awk ")
                || cmd_lower.contains(" edit ")
            {
                operation.operation_type = OperationType::EditFile;
                operation.risk_level = operation.risk_level.max(3);
            } else if cmd_lower.contains("rm ") || cmd_lower.contains("delete ") {
                operation.operation_type = OperationType::DeleteFile;
                operation.risk_level = operation.risk_level.max(8);
            } else if cmd_lower.contains("test")
                || cmd_lower.contains("cargo test")
                || cmd_lower.contains("npm test")
            {
                operation.operation_type = OperationType::RunTests;
                operation.risk_level = operation.risk_level.min(2);
                operation.reversible = true;
            } else if cmd_lower.contains("fmt")
                || cmd_lower.contains("format")
                || cmd_lower.contains("prettier")
            {
                operation.operation_type = OperationType::FormatCode;
                operation.risk_level = operation.risk_level.min(1);
                operation.reversible = true;
            } else if cmd_lower.contains("lint")
                || cmd_lower.contains("clippy")
                || cmd_lower.contains("eslint")
            {
                operation.operation_type = OperationType::LintCode;
                operation.risk_level = operation.risk_level.min(1);
                operation.reversible = true;
            } else if cmd_lower.contains("git ") {
                operation.operation_type = OperationType::GitOperation;
                if cmd_lower.contains("git push") || cmd_lower.contains("git reset --hard") {
                    operation.risk_level = operation.risk_level.max(7);
                } else {
                    operation.risk_level = operation.risk_level.max(3);
                }
            } else if cmd_lower.contains("cargo install")
                || cmd_lower.contains("npm install")
                || cmd_lower.contains("pip install")
            {
                operation.operation_type = OperationType::InstallDependencies;
                operation.risk_level = operation.risk_level.max(5);
            } else if cmd_lower.contains("build")
                || cmd_lower.contains("cargo build")
                || cmd_lower.contains("npm run build")
            {
                operation.operation_type = OperationType::Build;
                operation.risk_level = operation.risk_level.max(2);
                operation.reversible = true;
            } else if cmd_lower.contains("psql")
                || cmd_lower.contains("mysql")
                || cmd_lower.contains("sqlite")
            {
                operation.operation_type = OperationType::DatabaseOperation;
                operation.risk_level = operation.risk_level.max(9);
            } else if cmd_lower.contains("curl")
                || cmd_lower.contains("wget")
                || cmd_lower.contains("http")
            {
                operation.operation_type = OperationType::NetworkRequest;
                operation.risk_level = operation.risk_level.max(4);
            } else if cmd_lower.contains("mkdir") || cmd_lower.contains("mkdirs") {
                operation.operation_type = OperationType::CreateDirectory;
                operation.risk_level = operation.risk_level.min(2);
                operation.reversible = true;
            } else {
                operation.operation_type = OperationType::SystemCommand;
                operation.risk_level = operation.risk_level.max(6);
            }

            // Extract file paths (simplified - could be more sophisticated)
            self.extract_file_paths(command, &mut operation.affected_files);
        }

        Ok(operation)
    }

    /// Determine whether an operation should be auto-accepted
    pub fn should_auto_accept(&self, operation: &Operation) -> Result<AutoAcceptDecision> {
        // Quick rejections
        if self.config.emergency_stop {
            return Ok(AutoAcceptDecision::Reject(
                "Emergency stop is active".to_string(),
            ));
        }

        if !self.config.enabled {
            return Ok(AutoAcceptDecision::Reject(
                "Auto-accept is disabled".to_string(),
            ));
        }

        // Check if operation type is trusted
        if !self
            .config
            .trusted_operations
            .contains(&operation.operation_type)
        {
            return Ok(AutoAcceptDecision::Reject(format!(
                "Operation type {:?} is not in trusted operations list",
                operation.operation_type
            )));
        }

        // Check risk level (reject anything above 5 for auto-accept)
        if operation.risk_level > 5 {
            return Ok(AutoAcceptDecision::Reject(format!(
                "Risk level too high: {} > 5",
                operation.risk_level
            )));
        }

        // Check file change limits
        if operation.affected_files.len() > self.config.max_file_changes {
            return Ok(AutoAcceptDecision::Reject(format!(
                "Too many file changes: {} > {}",
                operation.affected_files.len(),
                self.config.max_file_changes
            )));
        }

        // Check for restricted files
        for file in &operation.affected_files {
            let file_str = file.to_string_lossy();
            for pattern in &self.config.restricted_files {
                if self.matches_pattern(&file_str, pattern) {
                    return Ok(AutoAcceptDecision::Reject(format!(
                        "File {} matches restricted pattern {}",
                        file_str, pattern
                    )));
                }
            }
        }

        // All checks passed - auto-accept with conditions
        let mut conditions = Vec::new();

        if self.config.require_tests_pass {
            conditions.push("Tests must pass".to_string());
        }

        if self.config.require_clean_git {
            conditions.push("Git working directory must be clean".to_string());
        }

        Ok(AutoAcceptDecision::Accept(conditions))
    }

    /// Validate changes after an operation has been executed
    pub fn validate_changes(
        &self,
        _operation: &Operation,
        execution_time: u32,
    ) -> Result<ValidationResult> {
        let mut issues = Vec::new();

        // Check execution time
        if execution_time > self.config.max_execution_time {
            issues.push(format!(
                "Execution time exceeded limit: {}s > {}s",
                execution_time, self.config.max_execution_time
            ));
        }

        // TODO: Add more validation logic here
        // - Check if tests still pass
        // - Verify no unexpected files were changed
        // - Check git status if required

        if issues.is_empty() {
            Ok(ValidationResult::Valid)
        } else {
            Ok(ValidationResult::Invalid(issues))
        }
    }

    /// Record an operation in the history for learning and analysis
    pub fn record_operation(&mut self, session_id: String, operation: Operation) {
        self.operation_history
            .entry(session_id)
            .or_default()
            .push(operation);
    }

    /// Get operation history for a session
    pub fn get_operation_history(&self, session_id: &str) -> Option<&Vec<Operation>> {
        self.operation_history.get(session_id)
    }

    /// Clear operation history for a session
    pub fn clear_history(&mut self, session_id: &str) {
        self.operation_history.remove(session_id);
    }

    /// Simple pattern matching for file restrictions
    pub fn matches_pattern(&self, file: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple glob matching - could be improved with a proper glob library
            if pattern.starts_with("**/") && pattern.ends_with("/*") {
                // Pattern like "**/migrations/*" - match any path containing the directory
                let dir_name = &pattern[3..pattern.len() - 2]; // Remove "**/" and "/*"
                file.contains(&format!("/{}/", dir_name))
                    || file.starts_with(&format!("{}/", dir_name))
                    || file.contains(&format!("/{}", dir_name)) // Also match if at end
            } else if let Some(suffix) = pattern.strip_prefix("**/") {
                file.contains(suffix)
            } else if pattern.starts_with('*') && pattern.ends_with('*') {
                let middle = &pattern[1..pattern.len() - 1];
                file.contains(middle)
            } else if let Some(suffix) = pattern.strip_prefix('*') {
                file.ends_with(suffix)
            } else if let Some(prefix) = pattern.strip_suffix('*') {
                file.starts_with(prefix)
            } else {
                file == pattern
            }
        } else {
            file == pattern
        }
    }

    /// Extract file paths from command strings (simplified implementation)
    fn extract_file_paths(&self, command: &str, paths: &mut Vec<PathBuf>) {
        // This is a simplified implementation - a more robust version would
        // properly parse shell commands and extract file arguments
        let parts: Vec<&str> = command.split_whitespace().collect();

        for part in parts {
            if part.contains('/') || part.contains('.') {
                // Likely a file path
                if let Ok(path) = std::path::Path::new(part).canonicalize() {
                    if !paths.contains(&path) {
                        paths.push(path);
                    }
                } else {
                    // Add as-is if we can't canonicalize (file might not exist yet)
                    let path = PathBuf::from(part);
                    if !paths.contains(&path) {
                        paths.push(path);
                    }
                }
            }
        }
    }
}

/// Decision result for auto-acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoAcceptDecision {
    /// Accept the operation with optional conditions
    Accept(Vec<String>),

    /// Reject the operation with reason
    Reject(String),
}

/// Result of validating changes after execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Changes are valid
    Valid,

    /// Changes are invalid with list of issues
    Invalid(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutoAcceptConfig::default();
        assert!(!config.enabled); // Should be disabled by default
        assert!(config.trusted_operations.contains(&OperationType::ReadFile));
        assert_eq!(config.max_file_changes, 5);
    }

    #[test]
    fn test_analyze_read_operation() {
        let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
        let commands = vec!["cat src/main.rs".to_string()];

        let operation = engine.analyze_operation(&commands, None).unwrap();
        assert_eq!(operation.operation_type, OperationType::ReadFile);
        assert_eq!(operation.risk_level, 1);
        assert!(operation.reversible);
    }

    #[test]
    fn test_analyze_dangerous_operation() {
        let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
        let commands = vec!["rm -rf /".to_string()];

        let operation = engine.analyze_operation(&commands, None).unwrap();
        assert_eq!(operation.operation_type, OperationType::DeleteFile);
        assert!(operation.risk_level >= 8);
    }

    #[test]
    fn test_should_auto_accept_disabled() {
        let config = AutoAcceptConfig {
            enabled: false,
            ..AutoAcceptConfig::default()
        };
        let engine = AutoAcceptEngine::new(config);

        let operation = Operation {
            operation_type: OperationType::ReadFile,
            description: "Read file".to_string(),
            affected_files: vec![],
            commands: vec!["cat test.txt".to_string()],
            risk_level: 1,
            reversible: true,
            task: None,
        };

        let decision = engine.should_auto_accept(&operation).unwrap();
        matches!(decision, AutoAcceptDecision::Reject(_));
    }

    #[test]
    fn test_should_auto_accept_trusted_operation() {
        let config = AutoAcceptConfig {
            enabled: true,
            trusted_operations: vec![OperationType::ReadFile],
            ..AutoAcceptConfig::default()
        };
        let engine = AutoAcceptEngine::new(config);

        let operation = Operation {
            operation_type: OperationType::ReadFile,
            description: "Read file".to_string(),
            affected_files: vec![],
            commands: vec!["cat test.txt".to_string()],
            risk_level: 1,
            reversible: true,
            task: None,
        };

        let decision = engine.should_auto_accept(&operation).unwrap();
        matches!(decision, AutoAcceptDecision::Accept(_));
    }

    #[test]
    fn test_pattern_matching() {
        let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());

        assert!(engine.matches_pattern("test.sql", "*.sql"));
        assert!(engine.matches_pattern("src/migrations/001.sql", "**/migrations/*"));
        assert!(engine.matches_pattern("Cargo.toml", "Cargo.toml"));
        assert!(!engine.matches_pattern("test.rs", "*.sql"));
    }

    #[test]
    fn test_emergency_stop() {
        let mut engine = AutoAcceptEngine::new(AutoAcceptConfig {
            enabled: true,
            ..AutoAcceptConfig::default()
        });

        engine.emergency_stop();
        assert!(engine.config.emergency_stop);
        assert!(!engine.config.enabled);

        let operation = Operation {
            operation_type: OperationType::ReadFile,
            description: "Read file".to_string(),
            affected_files: vec![],
            commands: vec!["cat test.txt".to_string()],
            risk_level: 1,
            reversible: true,
            task: None,
        };

        let decision = engine.should_auto_accept(&operation).unwrap();
        matches!(decision, AutoAcceptDecision::Reject(_));
    }
}
