use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

use super::{
    ClaudeCodeConfig, ProviderCapabilities, ProviderConfig, ProviderExecutor, ProviderHealthStatus,
};
use crate::agent::{Task, TaskResult, TaskType};
use crate::identity::AgentIdentity;

/// Claude Code provider executor implementation
pub struct ClaudeCodeExecutor {
    config: ClaudeCodeConfig,
}

impl ClaudeCodeExecutor {
    /// Create a new Claude Code executor
    pub fn new(config: ClaudeCodeConfig) -> Self {
        Self { config }
    }

    /// Generate task-specific prompt for Claude Code
    fn generate_task_prompt(&self, identity: &AgentIdentity, task: &Task) -> String {
        let agent_header = format!(
            "🤖 AGENT: {}\n📁 WORKSPACE: {}\n🎯 SCOPE: {}\n",
            identity.specialization.name(),
            identity.workspace_path.display(),
            format!("{:?}", task.task_type)
        );

        let task_context = format!(
            "TASK ID: {}\nTASK TYPE: {:?}\nPRIORITY: {:?}\n",
            task.id, task.task_type, task.priority
        );

        let boundaries = self.generate_boundary_reminder(&identity.specialization);

        format!(
            "{}\n{}\n{}\nTASK DESCRIPTION:\n{}\n\n{}",
            agent_header,
            task_context,
            boundaries,
            task.description,
            task.details.as_deref().unwrap_or("")
        )
    }

    /// Generate boundary reminder based on agent specialization
    fn generate_boundary_reminder(&self, specialization: &crate::identity::AgentRole) -> String {
        match specialization {
            crate::identity::AgentRole::Frontend { .. } => {
                "REMEMBER: You are a FRONTEND specialist. Focus ONLY on:\n\
                 - UI components and interfaces\n\
                 - Client-side logic and state management\n\
                 - Styling and responsive design\n\
                 - Frontend testing and optimization\n\
                 DELEGATE backend/API work to backend agents."
            }
            crate::identity::AgentRole::Backend { .. } => {
                "REMEMBER: You are a BACKEND specialist. Focus ONLY on:\n\
                 - API endpoints and server logic\n\
                 - Database operations and migrations\n\
                 - Authentication and authorization\n\
                 - Backend testing and performance\n\
                 DELEGATE frontend/UI work to frontend agents."
            }
            crate::identity::AgentRole::DevOps { .. } => {
                "REMEMBER: You are a DEVOPS specialist. Focus ONLY on:\n\
                 - Infrastructure and deployment\n\
                 - CI/CD pipelines and automation\n\
                 - Monitoring and observability\n\
                 - Security and compliance\n\
                 DELEGATE application code to dev agents."
            }
            crate::identity::AgentRole::QA { .. } => {
                "REMEMBER: You are a QA specialist. Focus ONLY on:\n\
                 - Test strategy and implementation\n\
                 - Quality assurance and validation\n\
                 - Bug reporting and regression testing\n\
                 - Performance and security testing\n\
                 DELEGATE implementation to dev agents."
            }
            crate::identity::AgentRole::Master { .. } => {
                "REMEMBER: You are the MASTER orchestrator. Focus ONLY on:\n\
                 - Coordinating between agents\n\
                 - Quality reviews and approvals\n\
                 - Strategic decisions and planning\n\
                 - NEVER write code directly - delegate to specialists."
            }
        }
        .to_string()
    }

    /// Execute Claude Code command with enhanced error handling
    async fn execute_claude_command(
        &self,
        args: Vec<String>,
        working_dir: &PathBuf,
        identity: &AgentIdentity,
    ) -> Result<String> {
        let mut cmd = Command::new("claude");

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
            std::time::Duration::from_secs(300), // 5 minute timeout
            cmd.output(),
        )
        .await
        .context("Claude Code command timed out")?
        .context("Failed to execute Claude Code")?;

        let duration = start.elapsed();

        tracing::debug!(
            "Claude Code execution completed in {:?} for agent {}",
            duration,
            identity.agent_id
        );

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            if stdout.trim().is_empty() {
                // Sometimes Claude outputs to stderr even on success
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !stderr.trim().is_empty() {
                    return Ok(stderr);
                }
            }
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            Err(anyhow::anyhow!(
                "Claude Code execution failed (exit code: {:?})\nStderr: {}\nStdout: {}",
                output.status.code(),
                stderr,
                stdout
            ))
        }
    }

    /// Build Claude Code command arguments
    fn build_command_args(&self, prompt: &str) -> Vec<String> {
        let mut args = Vec::new();

        // Add prompt
        args.push("-p".to_string());
        args.push(prompt.to_string());

        // Add JSON output if enabled
        if self.config.json_output {
            args.push("--json".to_string());
        }

        // Add dangerous skip if enabled
        if self.config.dangerous_skip {
            args.push("--dangerously-skip-permissions".to_string());
        }

        // Add think mode if specified
        if let Some(think_mode) = &self.config.think_mode {
            args.push("--think".to_string());
            args.push(think_mode.clone());
        }

        // Add model if not default
        if self.config.model != "claude-3.5-sonnet" {
            args.push("--model".to_string());
            args.push(self.config.model.clone());
        }

        args
    }

    /// Parse task result from Claude Code output
    fn parse_task_result(
        &self,
        output: String,
        task: &Task,
        duration: std::time::Duration,
    ) -> TaskResult {
        // Try to parse as JSON if JSON output is enabled
        if self.config.json_output {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&output) {
                return TaskResult {
                    success: true,
                    output: json_value,
                    error: None,
                    duration,
                };
            }
        }

        // Fallback to text output
        TaskResult {
            success: true,
            output: serde_json::json!({
                "response": output,
                "task_id": task.id,
                "format": "text"
            }),
            error: None,
            duration,
        }
    }
}

#[async_trait]
impl ProviderExecutor for ClaudeCodeExecutor {
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        working_dir: &PathBuf,
    ) -> Result<String> {
        let args = self.build_command_args(prompt);
        self.execute_claude_command(args, working_dir, identity)
            .await
    }

    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &PathBuf,
    ) -> Result<TaskResult> {
        let start = Instant::now();

        // Generate enhanced prompt with identity and boundaries
        let prompt = self.generate_task_prompt(identity, task);

        tracing::info!(
            "Executing task '{}' with Claude Code for agent {}",
            task.description,
            identity.agent_id
        );

        // Execute the prompt
        match self.execute_prompt(&prompt, identity, working_dir).await {
            Ok(output) => {
                let duration = start.elapsed();
                let result = self.parse_task_result(output, task, duration);

                tracing::info!(
                    "Task completed successfully in {:?} for agent {}",
                    duration,
                    identity.agent_id
                );

                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();

                tracing::error!(
                    "Task failed after {:?} for agent {}: {}",
                    duration,
                    identity.agent_id,
                    e
                );

                Ok(TaskResult {
                    success: false,
                    output: serde_json::json!({}),
                    error: Some(e.to_string()),
                    duration,
                })
            }
        }
    }

    async fn health_check(&self, working_dir: &PathBuf) -> Result<ProviderHealthStatus> {
        let start = Instant::now();

        // Try to execute a simple version check
        let result = Command::new("claude")
            .arg("--version")
            .current_dir(working_dir)
            .output()
            .await;

        let duration = start.elapsed();
        let response_time_ms = duration.as_millis() as u64;

        match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
                error_message: Some(format!("Failed to execute: {}", e)),
                response_time_ms: Some(response_time_ms),
            }),
        }
    }

    fn get_capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_json_output: true,
            supports_streaming: false, // Claude Code doesn't support streaming yet
            supports_file_operations: true,
            supports_git_operations: true,
            supports_code_execution: true,
            max_context_length: Some(200_000), // Claude 3.5 Sonnet context length
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c++".to_string(),
                "c".to_string(),
                "c#".to_string(),
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
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_identity() -> AgentIdentity {
        let temp_dir = TempDir::new().unwrap();
        AgentIdentity {
            agent_id: "test-agent".to_string(),
            specialization: AgentRole::Frontend {
                technologies: vec!["React".to_string(), "TypeScript".to_string()],
                responsibilities: vec!["UI components".to_string()],
                boundaries: vec!["No backend work".to_string()],
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
            description: "Create a React component".to_string(),
            details: Some("Build a button component with TypeScript".to_string()),
            priority: crate::agent::Priority::Medium,
            task_type: TaskType::Development,
            estimated_duration: None,
        }
    }

    #[test]
    fn test_claude_code_executor_creation() {
        let config = ClaudeCodeConfig::default();
        let executor = ClaudeCodeExecutor::new(config);
        assert_eq!(executor.config.model, "claude-3.5-sonnet");
    }

    #[test]
    fn test_generate_task_prompt() {
        let config = ClaudeCodeConfig::default();
        let executor = ClaudeCodeExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();

        let prompt = executor.generate_task_prompt(&identity, &task);

        assert!(prompt.contains("🤖 AGENT: Frontend"));
        assert!(prompt.contains("FRONTEND specialist"));
        assert!(prompt.contains("Create a React component"));
        assert!(prompt.contains("DELEGATE backend"));
    }

    #[test]
    fn test_build_command_args() {
        let mut config = ClaudeCodeConfig::default();
        config.json_output = true;
        config.dangerous_skip = true;
        config.think_mode = Some("think_hard".to_string());

        let executor = ClaudeCodeExecutor::new(config);
        let args = executor.build_command_args("test prompt");

        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"test prompt".to_string()));
        assert!(args.contains(&"--json".to_string()));
        assert!(args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(args.contains(&"--think".to_string()));
        assert!(args.contains(&"think_hard".to_string()));
    }

    #[test]
    fn test_parse_task_result_json() {
        let config = ClaudeCodeConfig {
            json_output: true,
            ..Default::default()
        };
        let executor = ClaudeCodeExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_millis(100);

        let json_output = r#"{"status": "completed", "files_changed": 2}"#;
        let result = executor.parse_task_result(json_output.to_string(), &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.duration, duration);

        // Should parse as JSON
        let output = result.output.as_object().unwrap();
        assert_eq!(output["status"], "completed");
        assert_eq!(output["files_changed"], 2);
    }

    #[test]
    fn test_parse_task_result_text() {
        let config = ClaudeCodeConfig {
            json_output: false,
            ..Default::default()
        };
        let executor = ClaudeCodeExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_millis(100);

        let text_output = "Task completed successfully!";
        let result = executor.parse_task_result(text_output.to_string(), &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());

        // Should wrap in JSON with text format
        let output = result.output.as_object().unwrap();
        assert_eq!(output["response"], "Task completed successfully!");
        assert_eq!(output["format"], "text");
    }

    #[test]
    fn test_get_capabilities() {
        let config = ClaudeCodeConfig::default();
        let executor = ClaudeCodeExecutor::new(config);
        let capabilities = executor.get_capabilities();

        assert!(capabilities.supports_json_output);
        assert!(capabilities.supports_file_operations);
        assert!(capabilities.supports_git_operations);
        assert!(capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);
        assert_eq!(capabilities.max_context_length, Some(200_000));
        assert!(capabilities
            .supported_languages
            .contains(&"rust".to_string()));
        assert!(capabilities
            .supported_languages
            .contains(&"typescript".to_string()));
    }

    #[test]
    fn test_boundary_reminder_generation() {
        let config = ClaudeCodeConfig::default();
        let executor = ClaudeCodeExecutor::new(config);

        let frontend_role = AgentRole::Frontend {
            technologies: vec!["React".to_string()],
            responsibilities: vec!["UI".to_string()],
            boundaries: vec!["No backend".to_string()],
        };

        let reminder = executor.generate_boundary_reminder(&frontend_role);
        assert!(reminder.contains("FRONTEND specialist"));
        assert!(reminder.contains("DELEGATE backend"));
        assert!(reminder.contains("UI components"));
    }
}
