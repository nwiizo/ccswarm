use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

use super::{
    ClaudeCodeConfig, ProviderCapabilities, ProviderConfig, ProviderExecutor, ProviderHealthStatus,
};
use crate::agent::{Task, TaskResult};
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
            "🤖 AGENT: {}\n📁 WORKSPACE: {}\n🎯 SCOPE: {:?}\n",
            identity.specialization.name(),
            identity.workspace_path.display(),
            task.task_type
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
            crate::identity::AgentRole::Search { .. } => {
                "REMEMBER: You are a SEARCH specialist. Focus ONLY on:\n\
                 - Web search and information retrieval\n\
                 - Query optimization and filtering\n\
                 - Source evaluation and ranking\n\
                 - Result presentation and summarization\n\
                 - NEVER modify code - only gather information."
            }
        }
        .to_string()
    }

    /// Execute Claude Code command with enhanced error handling
    async fn execute_claude_command(
        &self,
        args: Vec<String>,
        working_dir: &Path,
        identity: &AgentIdentity,
    ) -> Result<String> {
        let mut cmd = Command::new("claude");

        // Set working directory - always use current directory to avoid Node.js module issues
        // This is a workaround for the yoga.wasm module resolution problem
        cmd.current_dir(".");

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

        tracing::info!(
            "Executing Claude Code with args: {:?} in dir: {}",
            args,
            working_dir.display()
        );

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

    /// Build Claude Code command arguments based on current CLI options
    fn build_command_args(&self, prompt: &str) -> Vec<String> {
        let mut args = vec![
            // Add prompt (print mode)
            "-p".to_string(),
            prompt.to_string(),
            // Add output format (replaces deprecated --json flag)
            "--output-format".to_string(),
            self.config.output_format.as_cli_arg().to_string(),
        ];

        // Add dangerous skip if enabled
        if self.config.dangerous_skip {
            args.push("--dangerously-skip-permissions".to_string());
        }

        // Add model (using aliases like "sonnet", "opus", "haiku", "opusplan")
        args.push("--model".to_string());
        args.push(self.config.model.clone());

        // Add append system prompt if specified
        if let Some(system_prompt) = &self.config.append_system_prompt {
            args.push("--append-system-prompt".to_string());
            args.push(system_prompt.clone());
        }

        // Session management options
        if let Some(session_id) = &self.config.session_id {
            args.push("--session-id".to_string());
            args.push(session_id.clone());
        }

        if let Some(resume) = &self.config.resume_session {
            args.push("--resume".to_string());
            args.push(resume.clone());
        }

        if self.config.continue_session {
            args.push("--continue".to_string());
        }

        if self.config.fork_session {
            args.push("--fork-session".to_string());
        }

        // Add max turns if specified
        if let Some(max_turns) = self.config.max_turns {
            args.push("--max-turns".to_string());
            args.push(max_turns.to_string());
        }

        // Add fallback model if specified
        if let Some(fallback) = &self.config.fallback_model {
            args.push("--fallback-model".to_string());
            args.push(fallback.clone());
        }

        // Tool restrictions
        for tool in &self.config.allowed_tools {
            args.push("--allowedTools".to_string());
            args.push(tool.clone());
        }

        for tool in &self.config.disallowed_tools {
            args.push("--disallowedTools".to_string());
            args.push(tool.clone());
        }

        // Logging options
        if self.config.verbose {
            args.push("--verbose".to_string());
        }

        if self.config.mcp_debug {
            args.push("--mcp-debug".to_string());
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
        use crate::providers::OutputFormat;

        // Try to parse as JSON if JSON output format is used
        if (self.config.output_format == OutputFormat::Json
            || self.config.output_format == OutputFormat::StreamJson)
            && let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&output)
        {
            // Extract the result field if present (new JSON format)
            let result_output = if let Some(result) = json_value.get("result") {
                json!({
                    "response": result,
                    "task_id": task.id,
                    "format": "json",
                    "cost_usd": json_value.get("total_cost_usd"),
                    "duration_ms": json_value.get("duration_ms"),
                    "session_id": json_value.get("session_id"),
                    "num_turns": json_value.get("num_turns")
                })
            } else {
                json_value
            };

            return TaskResult {
                success: true,
                output: result_output,
                error: None,
                duration,
            };
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
        working_dir: &Path,
    ) -> Result<String> {
        let args = self.build_command_args(prompt);
        self.execute_claude_command(args, working_dir, identity)
            .await
    }

    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &Path,
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

    async fn health_check(&self, working_dir: &Path) -> Result<ProviderHealthStatus> {
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
