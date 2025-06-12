use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

use super::{CustomConfig, ProviderCapabilities, ProviderExecutor, ProviderHealthStatus};
use crate::agent::{Task, TaskResult, TaskType};
use crate::identity::AgentIdentity;

/// Custom provider executor implementation
/// Allows integration with any command-line tool or script
pub struct CustomExecutor {
    config: CustomConfig,
}

impl CustomExecutor {
    /// Create a new custom executor
    pub fn new(config: CustomConfig) -> Self {
        Self { config }
    }

    /// Generate context information for custom commands
    fn generate_context_info(&self, identity: &AgentIdentity, task: &Task) -> String {
        let context_json = serde_json::json!({
            "agent": {
                "id": identity.agent_id,
                "specialization": identity.specialization.name(),
                "workspace": identity.workspace_path.to_string_lossy(),
                "session_id": identity.session_id,
            },
            "task": {
                "id": task.id,
                "description": task.description,
                "details": task.details,
                "priority": format!("{:?}", task.priority),
                "type": format!("{:?}", task.task_type),
            },
            "boundaries": self.get_agent_boundaries(&identity.specialization),
        });

        serde_json::to_string_pretty(&context_json).unwrap_or_else(|_| {
            format!(
                "Agent: {} ({})\nTask: {}\nDescription: {}",
                identity.agent_id,
                identity.specialization.name(),
                task.id,
                task.description
            )
        })
    }

    /// Get agent boundaries for context
    fn get_agent_boundaries(&self, specialization: &crate::identity::AgentRole) -> Vec<String> {
        match specialization {
            crate::identity::AgentRole::Frontend { boundaries, .. } => boundaries.clone(),
            crate::identity::AgentRole::Backend { boundaries, .. } => boundaries.clone(),
            crate::identity::AgentRole::DevOps { boundaries, .. } => boundaries.clone(),
            crate::identity::AgentRole::QA { boundaries, .. } => boundaries.clone(),
            crate::identity::AgentRole::Master { .. } => {
                vec![
                    "No direct code implementation".to_string(),
                    "Coordination only".to_string(),
                ]
            }
        }
    }

    /// Prepare command arguments with placeholder substitution
    fn prepare_command_args(
        &self,
        prompt: &str,
        context: &str,
        identity: &AgentIdentity,
        task: &Task,
    ) -> Vec<String> {
        self.config
            .args
            .iter()
            .map(|arg| {
                arg.replace("{prompt}", prompt)
                    .replace("{context}", context)
                    .replace("{agent_id}", &identity.agent_id)
                    .replace("{task_id}", &task.id)
                    .replace("{workspace}", &identity.workspace_path.to_string_lossy())
                    .replace("{specialization}", identity.specialization.name())
                    .replace("{task_type}", &format!("{:?}", task.task_type))
                    .replace("{priority}", &format!("{:?}", task.priority))
            })
            .collect()
    }

    /// Execute the custom command
    async fn execute_custom_command(
        &self,
        args: Vec<String>,
        working_dir: &PathBuf,
        identity: &AgentIdentity,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.config.command);

        // Set working directory
        let work_dir = self
            .config
            .working_directory
            .as_ref()
            .unwrap_or(working_dir);
        cmd.current_dir(work_dir);

        // Add identity environment variables
        for (key, value) in &identity.env_vars {
            cmd.env(key, value);
        }

        // Add provider environment variables
        for (key, value) in &self.config.env_vars {
            cmd.env(key, value);
        }

        // Add command arguments
        cmd.args(&args);

        // Execute command with timeout
        let start = Instant::now();
        let timeout_duration =
            std::time::Duration::from_secs(self.config.timeout_seconds.unwrap_or(300));

        let output = tokio::time::timeout(timeout_duration, cmd.output())
            .await
            .context("Custom command timed out")?
            .context("Failed to execute custom command")?;

        let duration = start.elapsed();

        tracing::debug!(
            "Custom command '{}' execution completed in {:?} for agent {}",
            self.config.command,
            duration,
            identity.agent_id
        );

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // Combine stdout and stderr if both have content
            if !stdout.trim().is_empty() && !stderr.trim().is_empty() {
                Ok(format!("{}\n\nSTDERR:\n{}", stdout, stderr))
            } else if !stdout.trim().is_empty() {
                Ok(stdout)
            } else if !stderr.trim().is_empty() {
                Ok(stderr)
            } else {
                Ok("Command executed successfully with no output".to_string())
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            Err(anyhow::anyhow!(
                "Custom command '{}' failed (exit code: {:?})\nStderr: {}\nStdout: {}",
                self.config.command,
                output.status.code(),
                stderr,
                stdout
            ))
        }
    }

    /// Parse command output based on configuration
    fn parse_command_output(
        &self,
        output: String,
        task: &Task,
        duration: std::time::Duration,
    ) -> TaskResult {
        if self.config.supports_json {
            // Try to parse as JSON
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&output) {
                return TaskResult {
                    success: true,
                    output: serde_json::json!({
                        "result": json_value,
                        "task_id": task.id,
                        "provider": "custom",
                        "command": self.config.command,
                        "format": "json"
                    }),
                    error: None,
                    duration,
                };
            }
        }

        // Fallback to text parsing
        let success = self.determine_success_from_output(&output);
        let error = if success {
            None
        } else {
            Some("Command output indicates failure".to_string())
        };

        TaskResult {
            success,
            output: serde_json::json!({
                "response": output,
                "task_id": task.id,
                "provider": "custom",
                "command": self.config.command,
                "format": "text",
                "working_directory": self.config.working_directory,
            }),
            error,
            duration,
        }
    }

    /// Determine success from command output using heuristics
    fn determine_success_from_output(&self, output: &str) -> bool {
        let output_lower = output.to_lowercase();

        // Check for explicit failure indicators
        let failure_indicators = [
            "error:",
            "failed:",
            "exception",
            "panic:",
            "fatal:",
            "command not found",
            "permission denied",
            "file not found",
            "syntax error",
            "compilation failed",
            "test failed",
        ];

        for indicator in &failure_indicators {
            if output_lower.contains(indicator) {
                return false;
            }
        }

        // Check for success indicators
        let success_indicators = [
            "success",
            "completed",
            "finished",
            "done",
            "ok",
            "passed",
            "test passed",
            "build successful",
        ];

        for indicator in &success_indicators {
            if output_lower.contains(indicator) {
                return true;
            }
        }

        // If no clear indicators, assume success if there's meaningful output
        !output.trim().is_empty()
    }

    /// Generate help text for the custom configuration
    pub fn generate_help_text(&self, identity: &AgentIdentity, task: &Task) -> String {
        let mut help = String::new();
        
        help.push_str("Custom Command Configuration:\n");
        help.push_str(&format!("Command: {}\n", self.config.command));
        help.push_str(&format!("Arguments: {:?}\n", self.config.args));
        help.push_str(&format!("Working Directory: {:?}\n", self.config.working_directory));
        help.push_str(&format!("Timeout: {:?} seconds\n", self.config.timeout_seconds));
        help.push_str(&format!("Supports JSON: {}\n", self.config.supports_json));
        
        help.push_str("\nAvailable Placeholders:\n");
        help.push_str(&format!("{{prompt}} -> Task prompt will be substituted here\n"));
        help.push_str(&format!("{{agent_id}} -> {} \n", identity.agent_id));
        help.push_str(&format!("{{task_id}} -> {}\n", task.id));
        help.push_str(&format!("{{task_description}} -> {}\n", task.description));
        help.push_str(&format!("{{workspace}} -> {}\n", identity.workspace_path.display()));
        
        if !self.config.env_vars.is_empty() {
            help.push_str("\nEnvironment Variables:\n");
            for (key, value) in &self.config.env_vars {
                help.push_str(&format!("{} = {}\n", key, value));
            }
        }
        
        help
    }

}

#[async_trait]
impl ProviderExecutor for CustomExecutor {
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        working_dir: &PathBuf,
    ) -> Result<String> {
        // Create a mock task for prompt execution
        let mock_task = Task {
            id: format!("prompt-{}", uuid::Uuid::new_v4()),
            description: "Direct prompt execution".to_string(),
            details: None,
            priority: crate::agent::Priority::Medium,
            task_type: TaskType::Development,
            estimated_duration: None,
        };

        let context = self.generate_context_info(identity, &mock_task);
        let args = self.prepare_command_args(prompt, &context, identity, &mock_task);

        self.execute_custom_command(args, working_dir, identity)
            .await
    }

    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &PathBuf,
    ) -> Result<TaskResult> {
        let start = Instant::now();

        // Generate context and prompt
        let context = self.generate_context_info(identity, task);
        let prompt = format!(
            "{}\n\nAdditional Details: {}",
            task.description,
            task.details.as_deref().unwrap_or("None")
        );

        tracing::info!(
            "Executing task '{}' with custom command '{}' for agent {}",
            task.description,
            self.config.command,
            identity.agent_id
        );

        // Prepare and execute command
        let args = self.prepare_command_args(&prompt, &context, identity, task);

        match self
            .execute_custom_command(args, working_dir, identity)
            .await
        {
            Ok(output) => {
                let duration = start.elapsed();
                let result = self.parse_command_output(output, task, duration);

                tracing::info!(
                    "Custom command task completed in {:?} for agent {} (success: {})",
                    duration,
                    identity.agent_id,
                    result.success
                );

                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();

                tracing::error!(
                    "Custom command task failed after {:?} for agent {}: {}",
                    duration,
                    identity.agent_id,
                    e
                );

                Ok(TaskResult {
                    success: false,
                    output: serde_json::json!({
                        "provider": "custom",
                        "command": self.config.command,
                    }),
                    error: Some(e.to_string()),
                    duration,
                })
            }
        }
    }

    async fn health_check(&self, working_dir: &PathBuf) -> Result<ProviderHealthStatus> {
        let start = Instant::now();

        // Try to execute a simple version or help command
        let test_commands = vec!["--version", "--help", "-h", "-V"];
        let mut last_error = None;

        for test_arg in test_commands {
            let result = Command::new(&self.config.command)
                .arg(test_arg)
                .current_dir(working_dir)
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    let duration = start.elapsed();
                    let response_time_ms = duration.as_millis() as u64;

                    let version_info = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("Custom command available")
                        .to_string();

                    return Ok(ProviderHealthStatus {
                        is_healthy: true,
                        version: Some(format!("{}: {}", self.config.command, version_info)),
                        last_check: chrono::Utc::now(),
                        error_message: None,
                        response_time_ms: Some(response_time_ms),
                    });
                }
                Ok(output) => {
                    last_error = Some(format!(
                        "Command failed with {}: {}",
                        test_arg,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
                Err(e) => {
                    last_error = Some(format!("Failed to execute {}: {}", test_arg, e));
                }
            }
        }

        let duration = start.elapsed();
        let response_time_ms = duration.as_millis() as u64;

        Ok(ProviderHealthStatus {
            is_healthy: false,
            version: None,
            last_check: chrono::Utc::now(),
            error_message: last_error.or_else(|| Some("Command not available".to_string())),
            response_time_ms: Some(response_time_ms),
        })
    }

    fn get_capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_json_output: self.config.supports_json,
            supports_streaming: false, // Custom commands typically don't support streaming
            supports_file_operations: true, // Depends on the command, but assume yes
            supports_git_operations: true, // Depends on the command, but assume yes
            supports_code_execution: true, // Depends on the command, but assume yes
            max_context_length: None,  // Unknown for custom commands
            supported_languages: vec![
                // Generic list - actual support depends on the custom command
                "text".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "markdown".to_string(),
                "bash".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{AgentIdentity, AgentRole};
    use std::collections::HashMap;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_identity() -> AgentIdentity {
        let temp_dir = TempDir::new().unwrap();
        AgentIdentity {
            agent_id: "test-custom-agent".to_string(),
            specialization: AgentRole::DevOps {
                technologies: vec!["Docker".to_string(), "Kubernetes".to_string()],
                responsibilities: vec!["Infrastructure".to_string()],
                boundaries: vec!["No app development".to_string()],
            },
            workspace_path: temp_dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            session_id: Uuid::new_v4().to_string(),
            parent_process_id: "12345".to_string(),
            initialized_at: chrono::Utc::now(),
        }
    }

    fn create_test_task() -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: "Deploy application".to_string(),
            details: Some("Deploy to staging environment".to_string()),
            priority: crate::agent::Priority::High,
            task_type: TaskType::Infrastructure,
            estimated_duration: None,
        }
    }

    #[test]
    fn test_custom_executor_creation() {
        let config = CustomConfig {
            command: "echo".to_string(),
            args: vec!["{prompt}".to_string()],
            env_vars: HashMap::new(),
            working_directory: None,
            timeout_seconds: Some(60),
            supports_json: false,
        };

        let executor = CustomExecutor::new(config);
        assert_eq!(executor.config.command, "echo");
        assert_eq!(executor.config.timeout_seconds, Some(60));
    }

    #[test]
    fn test_generate_context_info() {
        let config = CustomConfig::default();
        let executor = CustomExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();

        let context = executor.generate_context_info(&identity, &task);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&context).unwrap();
        assert_eq!(parsed["agent"]["id"], "test-custom-agent");
        assert_eq!(parsed["agent"]["specialization"], "DevOps");
        assert_eq!(parsed["task"]["description"], "Deploy application");
        assert!(parsed["boundaries"].is_array());
    }

    #[test]
    fn test_prepare_command_args() {
        let config = CustomConfig {
            command: "my-tool".to_string(),
            args: vec![
                "--prompt".to_string(),
                "{prompt}".to_string(),
                "--agent".to_string(),
                "{agent_id}".to_string(),
                "--workspace".to_string(),
                "{workspace}".to_string(),
            ],
            env_vars: HashMap::new(),
            working_directory: None,
            timeout_seconds: None,
            supports_json: false,
        };

        let executor = CustomExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();
        let prompt = "Test prompt";
        let context = "{}";

        let args = executor.prepare_command_args(prompt, context, &identity, &task);

        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "--prompt");
        assert_eq!(args[1], "Test prompt");
        assert_eq!(args[2], "--agent");
        assert_eq!(args[3], "test-custom-agent");
        assert_eq!(args[4], "--workspace");
        assert!(args[5].contains("/tmp") || args[5].contains("/var")); // Workspace path is a temp directory
    }

    #[test]
    fn test_determine_success_from_output() {
        let config = CustomConfig::default();
        let executor = CustomExecutor::new(config);

        // Test success cases
        assert!(executor.determine_success_from_output("Task completed successfully"));
        assert!(executor.determine_success_from_output("Build finished with success"));
        assert!(executor.determine_success_from_output("All tests passed"));
        assert!(executor.determine_success_from_output("Done processing files"));

        // Test failure cases
        assert!(!executor.determine_success_from_output("Error: File not found"));
        assert!(!executor.determine_success_from_output("failed: to compile"));
        assert!(!executor.determine_success_from_output("Exception occurred"));
        assert!(!executor.determine_success_from_output("Fatal: cannot proceed"));

        // Test neutral case with content
        assert!(executor.determine_success_from_output("Processing files..."));

        // Test empty case
        assert!(!executor.determine_success_from_output(""));
    }

    #[test]
    fn test_parse_command_output_json() {
        let config = CustomConfig {
            supports_json: true,
            ..Default::default()
        };
        let executor = CustomExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_millis(100);

        let json_output = r#"{"status": "success", "files_processed": 5}"#;
        let result = executor.parse_command_output(json_output.to_string(), &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());

        let output = result.output.as_object().unwrap();
        assert_eq!(output["format"], "json");
        assert_eq!(output["provider"], "custom");

        let json_result = &output["result"];
        assert_eq!(json_result["status"], "success");
        assert_eq!(json_result["files_processed"], 5);
    }

    #[test]
    fn test_parse_command_output_text() {
        let config = CustomConfig {
            supports_json: false,
            ..Default::default()
        };
        let executor = CustomExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_millis(200);

        let text_output = "Deployment completed successfully!";
        let result = executor.parse_command_output(text_output.to_string(), &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());

        let output = result.output.as_object().unwrap();
        assert_eq!(output["format"], "text");
        assert_eq!(output["provider"], "custom");
        assert_eq!(output["response"], "Deployment completed successfully!");
    }

    #[test]
    fn test_get_capabilities() {
        let config = CustomConfig {
            supports_json: true,
            ..Default::default()
        };
        let executor = CustomExecutor::new(config);
        let capabilities = executor.get_capabilities();

        assert!(capabilities.supports_json_output);
        assert!(capabilities.supports_file_operations);
        assert!(capabilities.supports_git_operations);
        assert!(capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);
        assert!(capabilities.max_context_length.is_none());
        assert!(capabilities
            .supported_languages
            .contains(&"text".to_string()));
        assert!(capabilities
            .supported_languages
            .contains(&"json".to_string()));
    }

    #[test]
    fn test_generate_help_text() {
        let config = CustomConfig {
            command: "my-tool".to_string(),
            args: vec!["--input".to_string(), "{prompt}".to_string()],
            env_vars: {
                let mut env = HashMap::new();
                env.insert("TOOL_MODE".to_string(), "production".to_string());
                env
            },
            working_directory: Some(PathBuf::from("/tmp")),
            timeout_seconds: Some(120),
            supports_json: true,
        };

        let executor = CustomExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();

        let help = executor.generate_help_text(&identity, &task);

        assert!(help.contains("Custom Command Configuration:"));
        assert!(help.contains("Command: my-tool"));
        assert!(help.contains("Timeout: Some(120) seconds"));
        assert!(help.contains("Supports JSON: true"));
        assert!(help.contains("Available Placeholders:"));
        assert!(help.contains("{prompt} ->"));
        assert!(help.contains("{agent_id} ->"));
        assert!(help.contains("Environment Variables:"));
        assert!(help.contains("TOOL_MODE = production"));
    }
}
