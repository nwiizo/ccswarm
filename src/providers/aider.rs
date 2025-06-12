use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

use super::{
    AiderConfig, ProviderCapabilities, ProviderConfig, ProviderExecutor, ProviderHealthStatus,
};
use crate::agent::{Task, TaskResult, TaskType};
use crate::identity::AgentIdentity;

/// Aider provider executor implementation
pub struct AiderExecutor {
    config: AiderConfig,
}

impl AiderExecutor {
    /// Create a new Aider executor
    pub fn new(config: AiderConfig) -> Self {
        Self { config }
    }

    /// Generate task-specific prompt for Aider
    fn generate_task_prompt(&self, identity: &AgentIdentity, task: &Task) -> String {
        let context_header = format!(
            "# ccswarm Agent Context\n\
             Agent: {} ({})\n\
             Task: {}\n\
             Priority: {:?}\n\
             Type: {:?}\n\n",
            identity.agent_id,
            identity.specialization.name(),
            task.id,
            task.priority,
            task.task_type
        );

        let boundaries = self.generate_agent_boundaries(&identity.specialization);

        let task_description = format!("## Task Description\n{}\n\n", task.description);

        let task_details = if let Some(details) = &task.details {
            format!("## Additional Details\n{}\n\n", details)
        } else {
            String::new()
        };

        let aider_instructions = self.generate_aider_instructions(task);

        format!(
            "{}{}{}{}\n{}",
            context_header, boundaries, task_description, task_details, aider_instructions
        )
    }

    /// Generate agent-specific boundaries for Aider
    fn generate_agent_boundaries(&self, specialization: &crate::identity::AgentRole) -> String {
        match specialization {
            crate::identity::AgentRole::Frontend {
                technologies,
                responsibilities,
                ..
            } => {
                format!(
                    "## Agent Specialization: Frontend\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     \n\
                     **IMPORTANT**: Focus only on frontend concerns. Do not modify:\n\
                     - Backend API endpoints or server code\n\
                     - Database schemas or migrations\n\
                     - Infrastructure or deployment configurations\n\
                     \n",
                    technologies.join(", "),
                    responsibilities.join(", ")
                )
            }
            crate::identity::AgentRole::Backend {
                technologies,
                responsibilities,
                ..
            } => {
                format!(
                    "## Agent Specialization: Backend\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     \n\
                     **IMPORTANT**: Focus only on backend concerns. Do not modify:\n\
                     - Frontend components or UI code\n\
                     - Client-side styling or layouts\n\
                     - Frontend build configurations\n\
                     \n",
                    technologies.join(", "),
                    responsibilities.join(", ")
                )
            }
            crate::identity::AgentRole::DevOps {
                technologies,
                responsibilities,
                ..
            } => {
                format!(
                    "## Agent Specialization: DevOps\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     \n\
                     **IMPORTANT**: Focus only on infrastructure and deployment. Do not modify:\n\
                     - Application business logic\n\
                     - Frontend or backend feature code\n\
                     - Database application schemas\n\
                     \n",
                    technologies.join(", "),
                    responsibilities.join(", ")
                )
            }
            crate::identity::AgentRole::QA {
                responsibilities, ..
            } => {
                format!(
                    "## Agent Specialization: QA\n\
                     Responsibilities: {}\n\
                     \n\
                     **IMPORTANT**: Focus only on testing and quality assurance. Do not modify:\n\
                     - Production application code\n\
                     - Core business logic\n\
                     - Infrastructure configurations\n\
                     \n",
                    responsibilities.join(", ")
                )
            }
            crate::identity::AgentRole::Master { .. } => {
                "## Agent Specialization: Master Orchestrator\n\
                 \n\
                 **IMPORTANT**: You are the orchestrator. Do not modify code directly.\n\
                 Instead, coordinate between other agents and provide guidance.\n\
                 \n"
                .to_string()
            }
        }
    }

    /// Generate Aider-specific instructions based on task type
    fn generate_aider_instructions(&self, task: &Task) -> String {
        let base_instructions = "## Aider Instructions\n\
                                Please analyze the codebase and implement the requested changes.\n";

        let task_specific = match task.task_type {
            TaskType::Development => {
                "- Focus on implementing clean, maintainable code\n\
                 - Follow existing code patterns and conventions\n\
                 - Add appropriate error handling\n\
                 - Include relevant tests if applicable\n"
            }
            TaskType::Testing => {
                "- Write comprehensive tests for the feature/bug\n\
                 - Ensure good test coverage\n\
                 - Follow testing best practices\n\
                 - Use appropriate testing frameworks\n"
            }
            TaskType::Documentation => {
                "- Create clear, comprehensive documentation\n\
                 - Include code examples where appropriate\n\
                 - Follow documentation standards\n\
                 - Update related documentation files\n"
            }
            TaskType::Bugfix => {
                "- Identify and fix the root cause\n\
                 - Add regression tests\n\
                 - Ensure the fix doesn't break existing functionality\n\
                 - Document the fix if necessary\n"
            }
            TaskType::Infrastructure => {
                "- Focus on infrastructure and deployment concerns\n\
                 - Ensure configurations are secure and scalable\n\
                 - Follow infrastructure best practices\n\
                 - Test configuration changes thoroughly\n"
            }
            TaskType::Coordination => {
                "- This is a coordination task\n\
                 - Focus on planning and organizing\n\
                 - Do not implement code directly\n\
                 - Provide clear guidance for other agents\n"
            }
            TaskType::Review => {
                "- Review code and documentation quality\n\
                 - Check for security issues and best practices\n\
                 - Provide constructive feedback\n\
                 - Ensure compliance with coding standards\n"
            }
            TaskType::Feature => {
                "- Implement new feature functionality\n\
                 - Ensure proper integration with existing code\n\
                 - Follow feature specifications carefully\n\
                 - Add appropriate tests and documentation\n"
            }
            TaskType::Remediation => {
                "- Fix the quality issues identified in the review\n\
                 - Follow the specific instructions provided\n\
                 - Ensure all issues are resolved completely\n\
                 - Add tests to prevent regression\n\
                 - Improve code quality as needed\n"
            }
        };

        let git_instructions = if self.config.auto_commit {
            "\n**Git Behavior**: Auto-commit is enabled. Changes will be committed automatically."
        } else {
            "\n**Git Behavior**: Auto-commit is disabled. Review changes before committing."
        };

        format!("{}{}{}", base_instructions, task_specific, git_instructions)
    }

    /// Build Aider command arguments
    fn build_command_args(&self, prompt: &str, _working_dir: &Path) -> Vec<String> {
        let mut args = Vec::new();

        // Add model specification
        args.push("--model".to_string());
        args.push(self.config.model.clone());

        // Add auto-commit behavior
        if self.config.auto_commit {
            args.push("--auto-commits".to_string());
        } else {
            args.push("--no-auto-commits".to_string());
        }

        // Add git behavior
        if !self.config.git {
            args.push("--no-git".to_string());
        }

        // Add message with prompt
        args.push("--message".to_string());
        args.push(prompt.to_string());

        // Add additional custom arguments
        args.extend(self.config.additional_args.clone());

        // Add yes flag for non-interactive execution
        args.push("--yes".to_string());

        args
    }

    /// Execute Aider command
    async fn execute_aider_command(
        &self,
        args: Vec<String>,
        working_dir: &Path,
        identity: &AgentIdentity,
    ) -> Result<String> {
        let executable = if let Some(path) = &self.config.executable_path {
            path.to_string_lossy().to_string()
        } else {
            "aider".to_string()
        };

        let mut cmd = Command::new(&executable);

        // Set working directory
        cmd.current_dir(working_dir);

        // Add identity environment variables
        for (key, value) in &identity.env_vars {
            cmd.env(key, value);
        }

        // Add provider environment variables
        for (key, value) in self.config.get_env_vars() {
            cmd.env(key, value);
        }

        // Add command arguments
        cmd.args(&args);

        // Execute command with timeout
        let start = Instant::now();
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(600), // 10 minute timeout for Aider
            cmd.output(),
        )
        .await
        .context("Aider command timed out")?
        .context("Failed to execute Aider")?;

        let duration = start.elapsed();

        tracing::debug!(
            "Aider execution completed in {:?} for agent {}",
            duration,
            identity.agent_id
        );

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // Aider often outputs useful information to stderr even on success
            if !stderr.trim().is_empty() {
                Ok(format!("{}\n\nAider Output:\n{}", stdout, stderr))
            } else {
                Ok(stdout)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            Err(anyhow::anyhow!(
                "Aider execution failed (exit code: {:?})\nStderr: {}\nStdout: {}",
                output.status.code(),
                stderr,
                stdout
            ))
        }
    }

    /// Parse Aider output and extract useful information
    fn parse_aider_output(
        &self,
        output: String,
        task: &Task,
        duration: std::time::Duration,
    ) -> TaskResult {
        let mut files_changed = Vec::new();
        let mut commits_made = Vec::new();
        let mut errors = Vec::new();

        // Parse output for common Aider patterns
        for line in output.lines() {
            if line.contains("Modified ") || line.contains("Created ") {
                if let Some(file) = self.extract_filename_from_line(line) {
                    files_changed.push(file);
                }
            } else if line.contains("Commit ") && line.contains("hash:") {
                if let Some(commit) = self.extract_commit_from_line(line) {
                    commits_made.push(commit);
                }
            } else if line.contains("Error:") || line.contains("WARNING:") {
                errors.push(line.to_string());
            }
        }

        let success = errors.is_empty() || errors.iter().all(|e| !e.contains("Error:"));

        TaskResult {
            success,
            output: serde_json::json!({
                "aider_output": output,
                "files_changed": files_changed,
                "commits_made": commits_made,
                "warnings": errors.iter().filter(|e| e.contains("WARNING")).collect::<Vec<_>>(),
                "task_id": task.id,
                "provider": "aider",
                "model": self.config.model,
                "auto_commit": self.config.auto_commit,
            }),
            error: if success {
                None
            } else {
                Some(errors.join("; "))
            },
            duration,
        }
    }

    /// Extract filename from Aider output line
    fn extract_filename_from_line(&self, line: &str) -> Option<String> {
        // Look for patterns like "Modified src/main.rs" or "Created test.py"
        if let Some(start) = line.find("Modified ").or_else(|| line.find("Created ")) {
            let start_pos = if line[start..].starts_with("Modified ") {
                start + 9
            } else {
                start + 8
            };

            if let Some(end) = line[start_pos..].find(' ') {
                Some(line[start_pos..start_pos + end].to_string())
            } else {
                Some(line[start_pos..].to_string())
            }
        } else {
            None
        }
    }

    /// Extract commit hash from Aider output line
    fn extract_commit_from_line(&self, line: &str) -> Option<String> {
        // Look for patterns like "Commit abc123 hash: def456"
        if let Some(hash_pos) = line.find("hash: ") {
            let start = hash_pos + 6;
            if let Some(end) = line[start..].find(' ') {
                Some(line[start..start + end].to_string())
            } else {
                Some(line[start..].to_string())
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl ProviderExecutor for AiderExecutor {
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<String> {
        let args = self.build_command_args(prompt, working_dir);
        self.execute_aider_command(args, working_dir, identity)
            .await
    }

    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<TaskResult> {
        let start = Instant::now();

        // Generate enhanced prompt with context and boundaries
        let prompt = self.generate_task_prompt(identity, task);

        tracing::info!(
            "Executing task '{}' with Aider (model: {}) for agent {}",
            task.description,
            self.config.model,
            identity.agent_id
        );

        // Execute the prompt
        match self.execute_prompt(&prompt, identity, working_dir).await {
            Ok(output) => {
                let duration = start.elapsed();
                let result = self.parse_aider_output(output, task, duration);

                tracing::info!(
                    "Aider task completed in {:?} for agent {} (files changed: {})",
                    duration,
                    identity.agent_id,
                    result.output["files_changed"]
                        .as_array()
                        .map(|a| a.len())
                        .unwrap_or(0)
                );

                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();

                tracing::error!(
                    "Aider task failed after {:?} for agent {}: {}",
                    duration,
                    identity.agent_id,
                    e
                );

                Ok(TaskResult {
                    success: false,
                    output: serde_json::json!({
                        "provider": "aider",
                        "model": self.config.model,
                    }),
                    error: Some(e.to_string()),
                    duration,
                })
            }
        }
    }

    async fn health_check(&self, working_dir: &Path) -> Result<ProviderHealthStatus> {
        let start = Instant::now();

        let executable = if let Some(path) = &self.config.executable_path {
            path.to_string_lossy().to_string()
        } else {
            "aider".to_string()
        };

        // Try to execute version check
        let result = Command::new(&executable)
            .arg("--version")
            .current_dir(working_dir)
            .output()
            .await;

        let duration = start.elapsed();
        let response_time_ms = duration.as_millis() as u64;

        match result {
            Ok(output) if output.status.success() => {
                let version_output = String::from_utf8_lossy(&output.stdout);
                let version = version_output
                    .lines()
                    .find(|line| line.contains("aider"))
                    .unwrap_or(version_output.trim())
                    .to_string();

                Ok(ProviderHealthStatus {
                    is_healthy: true,
                    version: Some(version),
                    last_check: chrono::Utc::now(),
                    error_message: None,
                    response_time_ms: Some(response_time_ms),
                })
            }
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                Ok(ProviderHealthStatus {
                    is_healthy: false,
                    version: None,
                    last_check: chrono::Utc::now(),
                    error_message: Some(format!("Command failed: {}", error)),
                    response_time_ms: Some(response_time_ms),
                })
            }
            Err(e) => Ok(ProviderHealthStatus {
                is_healthy: false,
                version: None,
                last_check: chrono::Utc::now(),
                error_message: Some(format!("Failed to execute Aider: {}", e)),
                response_time_ms: Some(response_time_ms),
            }),
        }
    }

    fn get_capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_json_output: false, // Aider doesn't have structured JSON output
            supports_streaming: false,
            supports_file_operations: true,
            supports_git_operations: true,
            supports_code_execution: false, // Aider doesn't execute code directly
            max_context_length: Some(128_000), // Depends on the model used
            supported_languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "java".to_string(),
                "c++".to_string(),
                "c".to_string(),
                "c#".to_string(),
                "go".to_string(),
                "rust".to_string(),
                "php".to_string(),
                "ruby".to_string(),
                "html".to_string(),
                "css".to_string(),
                "sql".to_string(),
                "bash".to_string(),
                "yaml".to_string(),
                "json".to_string(),
                "markdown".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{AgentIdentity, AgentRole};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_identity() -> AgentIdentity {
        let temp_dir = TempDir::new().unwrap();
        AgentIdentity {
            agent_id: "test-agent".to_string(),
            specialization: AgentRole::Backend {
                technologies: vec!["Node.js".to_string(), "Express".to_string()],
                responsibilities: vec!["API development".to_string()],
                boundaries: vec!["No frontend work".to_string()],
            },
            workspace_path: temp_dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            session_id: Uuid::new_v4().to_string(),
            parent_process_id: "12345".to_string(),
            initialized_at: chrono::Utc::now(),
        }
    }

    fn create_test_task() -> Task {
        Task::new(
            Uuid::new_v4().to_string(),
            "Add authentication endpoint".to_string(),
            crate::agent::Priority::High,
            TaskType::Development,
        )
        .with_details("Implement JWT-based authentication".to_string())
    }

    #[test]
    fn test_aider_executor_creation() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);
        assert_eq!(executor.config.model, "gpt-4");
        assert!(executor.config.auto_commit);
    }

    #[test]
    fn test_generate_task_prompt() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();

        let prompt = executor.generate_task_prompt(&identity, &task);

        assert!(prompt.contains("# ccswarm Agent Context"));
        assert!(prompt.contains("Backend"));
        assert!(prompt.contains("Add authentication endpoint"));
        assert!(prompt.contains("**IMPORTANT**: Focus only on backend"));
        assert!(prompt.contains("## Aider Instructions"));
    }

    #[test]
    fn test_build_command_args() {
        let mut config = AiderConfig::default();
        config.model = "claude-3.5-sonnet".to_string();
        config.auto_commit = false;
        config.git = true;
        config.additional_args = vec!["--no-stream".to_string()];

        let executor = AiderExecutor::new(config);
        let working_dir = PathBuf::from("/tmp");
        let args = executor.build_command_args("test prompt", &working_dir);

        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"claude-3.5-sonnet".to_string()));
        assert!(args.contains(&"--no-auto-commits".to_string()));
        assert!(args.contains(&"--message".to_string()));
        assert!(args.contains(&"test prompt".to_string()));
        assert!(args.contains(&"--no-stream".to_string()));
        assert!(args.contains(&"--yes".to_string()));
    }

    #[test]
    fn test_extract_filename_from_line() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);

        let line1 = "Modified src/auth.js with new JWT implementation";
        let filename1 = executor.extract_filename_from_line(line1);
        assert_eq!(filename1, Some("src/auth.js".to_string()));

        let line2 = "Created test/auth.test.js";
        let filename2 = executor.extract_filename_from_line(line2);
        assert_eq!(filename2, Some("test/auth.test.js".to_string()));

        let line3 = "Some other output without file modification";
        let filename3 = executor.extract_filename_from_line(line3);
        assert_eq!(filename3, None);
    }

    #[test]
    fn test_extract_commit_from_line() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);

        let line = "Commit abc123 hash: def456789 completed successfully";
        let commit = executor.extract_commit_from_line(line);
        assert_eq!(commit, Some("def456789".to_string()));

        let line_no_hash = "Some other output without commit hash";
        let no_commit = executor.extract_commit_from_line(line_no_hash);
        assert_eq!(no_commit, None);
    }

    #[test]
    fn test_parse_aider_output() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_secs(10);

        let output = "Modified src/auth.js with authentication\n\
                     Created test/auth.test.js for testing\n\
                     Commit abc123 hash: def456789 completed\n\
                     All changes committed successfully";

        let result = executor.parse_aider_output(output.to_string(), &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());

        let output_obj = result.output.as_object().unwrap();
        let files_changed = output_obj["files_changed"].as_array().unwrap();
        assert_eq!(files_changed.len(), 2);
        assert!(files_changed.contains(&serde_json::Value::String("src/auth.js".to_string())));
        assert!(files_changed.contains(&serde_json::Value::String("test/auth.test.js".to_string())));

        let commits_made = output_obj["commits_made"].as_array().unwrap();
        assert_eq!(commits_made.len(), 1);
        assert!(commits_made.contains(&serde_json::Value::String("def456789".to_string())));
    }

    #[test]
    fn test_get_capabilities() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);
        let capabilities = executor.get_capabilities();

        assert!(!capabilities.supports_json_output);
        assert!(capabilities.supports_file_operations);
        assert!(capabilities.supports_git_operations);
        assert!(!capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);
        assert_eq!(capabilities.max_context_length, Some(128_000));
        assert!(capabilities
            .supported_languages
            .contains(&"python".to_string()));
        assert!(capabilities
            .supported_languages
            .contains(&"rust".to_string()));
    }

    #[test]
    fn test_generate_agent_boundaries() {
        let config = AiderConfig::default();
        let executor = AiderExecutor::new(config);

        let backend_role = AgentRole::Backend {
            technologies: vec!["Node.js".to_string(), "Express".to_string()],
            responsibilities: vec!["API development".to_string()],
            boundaries: vec!["No frontend".to_string()],
        };

        let boundaries = executor.generate_agent_boundaries(&backend_role);
        assert!(boundaries.contains("Backend"));
        assert!(boundaries.contains("Node.js, Express"));
        assert!(boundaries.contains("API development"));
        assert!(boundaries.contains("Do not modify"));
        assert!(boundaries.contains("Frontend components"));
    }
}
