use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

use super::{CodexConfig, ProviderCapabilities, ProviderExecutor, ProviderHealthStatus};
use crate::agent::{Task, TaskResult, TaskType};
use crate::identity::AgentIdentity;

/// OpenAI API request structure for chat completions
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

/// OpenAI message structure
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response structure
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    #[allow(dead_code)] // Part of API response structure
    id: String,
    #[allow(dead_code)] // Part of API response structure
    object: String,
    #[allow(dead_code)] // Part of API response structure
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    #[allow(dead_code)] // Part of API response structure
    usage: Option<OpenAIUsage>,
}

/// OpenAI choice structure
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    #[allow(dead_code)] // Part of API response structure
    index: u32,
    message: OpenAIMessage,
    #[allow(dead_code)] // Part of API response structure
    finish_reason: Option<String>,
}

/// OpenAI usage information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI error response
#[derive(Debug, Deserialize)]
struct OpenAIError {
    error: OpenAIErrorDetails,
}

/// OpenAI error details
#[derive(Debug, Deserialize)]
struct OpenAIErrorDetails {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    #[allow(dead_code)] // Part of API response structure
    code: Option<String>,
}

/// OpenAI Codex provider executor implementation
pub struct CodexExecutor {
    config: CodexConfig,
    client: Client,
}

impl CodexExecutor {
    /// Create a new Codex executor
    pub fn new(config: CodexConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.api_key)).unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(org) = &config.organization {
            headers.insert(
                header::HeaderName::from_static("openai-organization"),
                header::HeaderValue::from_str(org).unwrap(),
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();

        Self { config, client }
    }

    /// Generate system prompt for Codex based on agent identity
    fn generate_system_prompt(&self, identity: &AgentIdentity) -> String {
        let agent_context = format!(
            "You are a specialized AI coding assistant working as part of the ccswarm multi-agent system.\n\
             \n\
             Agent ID: {}\n\
             Specialization: {}\n\
             Workspace: {}\n\
             \n",
            identity.agent_id,
            identity.specialization.name(),
            identity.workspace_path.display()
        );

        let specialization_prompt = match &identity.specialization {
            crate::identity::AgentRole::Frontend {
                technologies,
                responsibilities,
                boundaries,
            } => {
                format!(
                    "FRONTEND SPECIALIST\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     Boundaries: {}\n\
                     \n\
                     Focus exclusively on frontend development. Do not write backend code, \
                     database schemas, or infrastructure configurations. If asked about \
                     backend concerns, explain that you need to delegate to a backend specialist.\n",
                    technologies.join(", "),
                    responsibilities.join(", "),
                    boundaries.join(", ")
                )
            }
            crate::identity::AgentRole::Backend {
                technologies,
                responsibilities,
                boundaries,
            } => {
                format!(
                    "BACKEND SPECIALIST\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     Boundaries: {}\n\
                     \n\
                     Focus exclusively on backend development. Do not write frontend UI code, \
                     styling, or client-side logic. If asked about frontend concerns, \
                     explain that you need to delegate to a frontend specialist.\n",
                    technologies.join(", "),
                    responsibilities.join(", "),
                    boundaries.join(", ")
                )
            }
            crate::identity::AgentRole::DevOps {
                technologies,
                responsibilities,
                boundaries,
            } => {
                format!(
                    "DEVOPS SPECIALIST\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     Boundaries: {}\n\
                     \n\
                     Focus exclusively on infrastructure, deployment, and operational concerns. \
                     Do not write application business logic or feature code. If asked about \
                     application development, explain that you need to delegate to development specialists.\n",
                    technologies.join(", "),
                    responsibilities.join(", "),
                    boundaries.join(", ")
                )
            }
            crate::identity::AgentRole::QA {
                responsibilities,
                boundaries,
                ..
            } => {
                format!(
                    "QA SPECIALIST\n\
                     Responsibilities: {}\n\
                     Boundaries: {}\n\
                     \n\
                     Focus exclusively on testing, quality assurance, and validation. \
                     Do not write production application code. Focus on test strategies, \
                     test implementation, and quality verification.\n",
                    responsibilities.join(", "),
                    boundaries.join(", ")
                )
            }
            crate::identity::AgentRole::Master {
                oversight_roles,
                quality_standards,
                ..
            } => {
                format!(
                    "MASTER ORCHESTRATOR\n\
                     Oversight: {}\n\
                     Quality Standards: Test coverage ≥{:.0}%, Max complexity ≤{}, Security scan: {}\n\
                     \n\
                     You are the master orchestrator. Do not write code directly. \
                     Instead, focus on coordination, planning, and quality review. \
                     Delegate specific implementation tasks to specialized agents.\n",
                    oversight_roles.join(", "),
                    quality_standards.min_test_coverage * 100.0,
                    quality_standards.max_complexity,
                    if quality_standards.security_scan_required { "Required" } else { "Optional" }
                )
            }
        };

        let guidelines = "\nGUIDELINES:\n\
                         1. Always identify yourself and your role at the start of responses\n\
                         2. Stay strictly within your specialization boundaries\n\
                         3. Provide clear, actionable code and explanations\n\
                         4. If a request is outside your scope, clearly state what needs to be delegated\n\
                         5. Focus on clean, maintainable, and well-documented code\n\
                         6. Consider security, performance, and best practices\n\
                         7. Respond in a structured format that can be easily parsed\n";

        format!("{}{}\n{}", agent_context, specialization_prompt, guidelines)
    }

    /// Generate task-specific prompt for Codex
    fn generate_task_prompt(&self, _identity: &AgentIdentity, task: &Task) -> String {
        let task_header = format!(
            "TASK REQUEST\n\
             Task ID: {}\n\
             Type: {:?}\n\
             Priority: {:?}\n\
             \n",
            task.id, task.task_type, task.priority
        );

        let task_instructions = match task.task_type {
            TaskType::Development => {
                "DEVELOPMENT TASK\n\
                 Please implement the requested feature with:\n\
                 - Clean, readable code\n\
                 - Proper error handling\n\
                 - Relevant comments\n\
                 - Consider testing implications\n"
            }
            TaskType::Testing => {
                "TESTING TASK\n\
                 Please create comprehensive tests with:\n\
                 - Good test coverage\n\
                 - Edge case handling\n\
                 - Clear test descriptions\n\
                 - Mock/stub strategies where appropriate\n"
            }
            TaskType::Documentation => {
                "DOCUMENTATION TASK\n\
                 Please create clear documentation with:\n\
                 - Comprehensive explanations\n\
                 - Code examples\n\
                 - Usage instructions\n\
                 - API references where applicable\n"
            }
            TaskType::Bugfix => {
                "BUG FIX TASK\n\
                 Please analyze and fix the issue with:\n\
                 - Root cause identification\n\
                 - Minimal, targeted changes\n\
                 - Prevention of regression\n\
                 - Explanation of the fix\n"
            }
            TaskType::Infrastructure => {
                "INFRASTRUCTURE TASK\n\
                 Please handle infrastructure concerns with:\n\
                 - Security best practices\n\
                 - Scalability considerations\n\
                 - Monitoring and observability\n\
                 - Documentation of configurations\n"
            }
            TaskType::Coordination => {
                "COORDINATION TASK\n\
                 Please provide coordination guidance with:\n\
                 - Clear delegation instructions\n\
                 - Task breakdown\n\
                 - Quality requirements\n\
                 - No direct implementation\n"
            }
            TaskType::Review => {
                "REVIEW TASK\n\
                 Please review the code/documentation with:\n\
                 - Quality assessment\n\
                 - Security considerations\n\
                 - Best practice validation\n\
                 - Improvement suggestions\n"
            }
            TaskType::Feature => {
                "FEATURE TASK\n\
                 Please implement the new feature with:\n\
                 - Complete functionality\n\
                 - Proper integration\n\
                 - Testing considerations\n\
                 - Documentation updates\n"
            }
        };

        let task_description = format!("DESCRIPTION:\n{}\n\n", task.description);

        let task_details = if let Some(details) = &task.details {
            format!("ADDITIONAL DETAILS:\n{}\n\n", details)
        } else {
            String::new()
        };

        let response_format = "RESPONSE FORMAT:\n\
                              Please structure your response as:\n\
                              1. Agent identification and scope confirmation\n\
                              2. Task analysis and approach\n\
                              3. Implementation (code/configuration/guidance)\n\
                              4. Verification steps or testing approach\n\
                              5. Any delegation needs or dependencies\n";

        format!(
            "{}{}\n{}{}\n{}",
            task_header, task_instructions, task_description, task_details, response_format
        )
    }

    /// Get the API base URL
    fn get_api_base(&self) -> String {
        self.config
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }

    /// Parse OpenAI response into TaskResult for testing
    #[cfg(test)]
    fn parse_response(
        &self,
        response: OpenAIResponse,
        task: &Task,
        duration: std::time::Duration,
    ) -> TaskResult {
        if let Some(choice) = response.choices.first() {
            TaskResult {
                success: true,
                output: serde_json::json!({
                    "response": choice.message.content,
                    "response_id": response.id,
                    "provider": "codex",
                    "model": response.model,
                    "task_id": task.id,
                }),
                error: None,
                duration,
            }
        } else {
            TaskResult {
                success: false,
                output: serde_json::json!({
                    "provider": "codex",
                    "model": response.model,
                }),
                error: Some("No response choices returned".to_string()),
                duration,
            }
        }
    }

    /// Make API request to OpenAI
    async fn make_api_request(&self, request: OpenAIRequest) -> Result<OpenAIResponse> {
        let url = format!("{}/chat/completions", self.get_api_base());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            serde_json::from_str::<OpenAIResponse>(&response_text)
                .context("Failed to parse OpenAI API response")
        } else {
            let status = response.status();
            let error_text = response.text().await?;

            // Try to parse as OpenAI error format
            if let Ok(error_response) = serde_json::from_str::<OpenAIError>(&error_text) {
                Err(anyhow::anyhow!(
                    "OpenAI API error ({}): {} ({})",
                    status,
                    error_response.error.message,
                    error_response.error.error_type
                ))
            } else {
                Err(anyhow::anyhow!(
                    "OpenAI API error ({}): {}",
                    status,
                    error_text
                ))
            }
        }
    }
}

#[async_trait]
impl ProviderExecutor for CodexExecutor {
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        _working_dir: &PathBuf,
    ) -> Result<String> {
        let system_prompt = self.generate_system_prompt(identity);

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: false,
        };

        let response = self.make_api_request(request).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response choices returned from API"))
        }
    }

    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &PathBuf,
    ) -> Result<TaskResult> {
        let start = Instant::now();

        // Generate comprehensive prompt
        let task_prompt = self.generate_task_prompt(identity, task);

        tracing::info!(
            "Executing task '{}' with OpenAI (model: {}) for agent {}",
            task.description,
            self.config.model,
            identity.agent_id
        );

        // Execute the prompt
        match self
            .execute_prompt(&task_prompt, identity, working_dir)
            .await
        {
            Ok(response_content) => {
                let duration = start.elapsed();

                tracing::info!(
                    "OpenAI task completed in {:?} for agent {}",
                    duration,
                    identity.agent_id
                );

                Ok(TaskResult {
                    success: true,
                    output: serde_json::json!({
                        "response": response_content,
                        "task_id": task.id,
                        "provider": "codex",
                        "model": self.config.model,
                    }),
                    error: None,
                    duration,
                })
            }
            Err(e) => {
                let duration = start.elapsed();

                tracing::error!(
                    "OpenAI task failed after {:?} for agent {}: {}",
                    duration,
                    identity.agent_id,
                    e
                );

                Ok(TaskResult {
                    success: false,
                    output: serde_json::json!({
                        "provider": "codex",
                        "model": self.config.model,
                    }),
                    error: Some(e.to_string()),
                    duration,
                })
            }
        }
    }

    async fn health_check(&self, _working_dir: &PathBuf) -> Result<ProviderHealthStatus> {
        let start = Instant::now();

        // Make a simple API call to check health
        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: "Hello, please respond with 'OK' to confirm you're working.".to_string(),
            }],
            max_tokens: Some(10),
            temperature: Some(0.1),
            stream: false,
        };

        match self.make_api_request(request).await {
            Ok(response) => {
                let duration = start.elapsed();
                let response_time_ms = duration.as_millis() as u64;

                Ok(ProviderHealthStatus {
                    is_healthy: true,
                    version: Some(format!("OpenAI API - Model: {}", response.model)),
                    last_check: chrono::Utc::now(),
                    error_message: None,
                    response_time_ms: Some(response_time_ms),
                })
            }
            Err(e) => {
                let duration = start.elapsed();
                let response_time_ms = duration.as_millis() as u64;

                Ok(ProviderHealthStatus {
                    is_healthy: false,
                    version: None,
                    last_check: chrono::Utc::now(),
                    error_message: Some(e.to_string()),
                    response_time_ms: Some(response_time_ms),
                })
            }
        }
    }

    fn get_capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_json_output: true,      // Can be instructed to return JSON
            supports_streaming: false,       // Not implemented in this executor
            supports_file_operations: false, // API-only, no direct file access
            supports_git_operations: false,  // API-only, no direct git access
            supports_code_execution: false,  // API-only, no code execution
            max_context_length: Some(match self.config.model.as_str() {
                "gpt-4" => 8192,
                "gpt-4-32k" => 32768,
                "gpt-3.5-turbo" => 4096,
                "gpt-3.5-turbo-16k" => 16384,
                _ => 4096, // Default assumption
            }),
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
                "swift".to_string(),
                "kotlin".to_string(),
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
            agent_id: "test-codex-agent".to_string(),
            specialization: AgentRole::Frontend {
                technologies: vec!["React".to_string(), "TypeScript".to_string()],
                responsibilities: vec!["UI development".to_string()],
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
            description: "Create a modal component".to_string(),
            details: Some("Implement a reusable modal with TypeScript".to_string()),
            priority: crate::agent::Priority::Medium,
            task_type: TaskType::Development,
            estimated_duration: None,
        }
    }

    #[test]
    fn test_codex_executor_creation() {
        let config = CodexConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            api_base: None,
            organization: None,
        };

        let executor = CodexExecutor::new(config);
        assert_eq!(executor.config.model, "gpt-4");
        assert_eq!(executor.config.max_tokens, Some(1000));
    }

    #[test]
    fn test_generate_system_prompt() {
        let config = CodexConfig::default();
        let executor = CodexExecutor::new(config);
        let identity = create_test_identity();

        let prompt = executor.generate_system_prompt(&identity);

        assert!(prompt.contains("ccswarm multi-agent system"));
        assert!(prompt.contains("FRONTEND SPECIALIST"));
        assert!(prompt.contains("test-codex-agent"));
        assert!(prompt.contains("React, TypeScript"));
        assert!(prompt.contains("Do not write backend code"));
    }

    #[test]
    fn test_generate_task_prompt() {
        let config = CodexConfig::default();
        let executor = CodexExecutor::new(config);
        let identity = create_test_identity();
        let task = create_test_task();

        let prompt = executor.generate_task_prompt(&identity, &task);

        assert!(prompt.contains("TASK REQUEST"));
        assert!(prompt.contains("Development"));
        assert!(prompt.contains("Create a modal component"));
        assert!(prompt.contains("DEVELOPMENT TASK"));
        assert!(prompt.contains("RESPONSE FORMAT"));
    }

    #[test]
    fn test_get_api_base() {
        let mut config = CodexConfig::default();
        let executor1 = CodexExecutor::new(config.clone());
        assert_eq!(executor1.get_api_base(), "https://api.openai.com/v1");

        config.api_base = Some("https://custom.api.com/v1".to_string());
        let executor2 = CodexExecutor::new(config);
        assert_eq!(executor2.get_api_base(), "https://custom.api.com/v1");
    }

    #[test]
    fn test_get_capabilities() {
        let config = CodexConfig::default();
        let executor = CodexExecutor::new(config);
        let capabilities = executor.get_capabilities();

        assert!(capabilities.supports_json_output);
        assert!(!capabilities.supports_file_operations);
        assert!(!capabilities.supports_git_operations);
        assert!(!capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);

        // Default model should be gpt-4 with 8192 context length
        assert_eq!(capabilities.max_context_length, Some(8192));
        assert!(capabilities
            .supported_languages
            .contains(&"typescript".to_string()));
        assert!(capabilities
            .supported_languages
            .contains(&"rust".to_string()));
    }

    #[test]
    fn test_parse_response() {
        let config = CodexConfig::default();
        let executor = CodexExecutor::new(config);
        let task = create_test_task();
        let duration = std::time::Duration::from_millis(500);

        let response = OpenAIResponse {
            id: "test-response-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: "Here's your modal component implementation...".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(OpenAIUsage {
                prompt_tokens: 100,
                completion_tokens: 200,
                total_tokens: 300,
            }),
        };

        let result = executor.parse_response(response, &task, duration);

        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.duration, duration);

        let output = result.output.as_object().unwrap();
        assert_eq!(output["provider"], "codex");
        assert_eq!(output["model"], "gpt-4");
        assert_eq!(output["response_id"], "test-response-id");
        assert!(output["response"]
            .as_str()
            .unwrap()
            .contains("modal component"));
    }
}
