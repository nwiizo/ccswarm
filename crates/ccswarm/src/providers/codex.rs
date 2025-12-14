use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::path::Path;
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
    pub fn new(config: CodexConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .context("Invalid API key format for Authorization header")?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(org) = &config.organization {
            headers.insert(
                header::HeaderName::from_static("openai-organization"),
                header::HeaderValue::from_str(org)
                    .context("Invalid organization ID format for OpenAI-Organization header")?,
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { config, client })
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
                    quality_standards.min_test_coverage,
                    quality_standards.max_complexity,
                    if quality_standards.security_scan_required {
                        "Required"
                    } else {
                        "Optional"
                    }
                )
            }
            crate::identity::AgentRole::Search {
                technologies,
                responsibilities,
                boundaries,
            } => {
                format!(
                    "SEARCH SPECIALIST\n\
                     Technologies: {}\n\
                     Responsibilities: {}\n\
                     Boundaries: {}\n\
                     \n\
                     Focus exclusively on information retrieval and search operations. \
                     Do not write or modify any code. Your role is to find and present \
                     relevant information to help other agents complete their tasks.\n",
                    technologies.join(", "),
                    responsibilities.join(", "),
                    boundaries.join(", ")
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
            TaskType::Remediation => {
                "REMEDIATION TASK\n\
                 Please fix the quality issues identified with:\n\
                 - Address all specific issues listed\n\
                 - Follow provided instructions exactly\n\
                 - Add tests to prevent regression\n\
                 - Improve overall code quality\n\
                 - Verify all issues are resolved\n"
            }
            TaskType::Bug => {
                "BUG FIX TASK\n\
                 Please analyze and fix the bug with:\n\
                 - Root cause identification\n\
                 - Minimal, targeted changes\n\
                 - Prevention of regression\n\
                 - Explanation of the fix\n"
            }
            TaskType::Assistance => {
                "ASSISTANCE TASK\n\
                 Please help with the requested task by:\n\
                 - Understanding the blocker\n\
                 - Providing expert guidance\n\
                 - Collaborative problem solving\n\
                 - Knowledge sharing\n"
            }
            TaskType::Research => {
                "RESEARCH TASK\n\
                 Please research and analyze information with:\n\
                 - Review search results thoroughly\n\
                 - Extract key insights\n\
                 - Apply findings to current work\n\
                 - Document recommendations\n"
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

    /// Call OpenAI API with request
    async fn call_api(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let url = format!("{}/chat/completions", self.get_api_base());

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            if let Ok(error) = serde_json::from_str::<OpenAIError>(&error_text) {
                return Err(anyhow::anyhow!(
                    "OpenAI API error: {} ({})",
                    error.error.message,
                    error.error.error_type
                ));
            }
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        response
            .json::<OpenAIResponse>()
            .await
            .context("Failed to parse OpenAI API response")
    }
}

#[async_trait]
impl ProviderExecutor for CodexExecutor {
    /// Execute a prompt with the provider
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        _working_dir: &Path,
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

        let response = self.call_api(&request).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response from OpenAI API"))
        }
    }

    /// Execute a task with full context
    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        _working_dir: &Path,
    ) -> Result<TaskResult> {
        let start_time = Instant::now();

        let system_prompt = self.generate_system_prompt(identity);
        let task_prompt = self.generate_task_prompt(identity, task);

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: task_prompt,
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: false,
        };

        let response = self.call_api(&request).await?;

        let output = if let Some(choice) = response.choices.first() {
            choice.message.content.clone()
        } else {
            return Err(anyhow::anyhow!("No response from OpenAI API"));
        };

        let duration = start_time.elapsed();

        Ok(TaskResult {
            success: true,
            output: serde_json::json!({
                "response": output,
                "model": response.model,
                "provider": "codex"
            }),
            error: None,
            duration,
        })
    }

    /// Test provider connectivity and functionality
    async fn health_check(&self, _working_dir: &Path) -> Result<ProviderHealthStatus> {
        let start_time = Instant::now();

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: "Say 'ok' if you can receive this message.".to_string(),
            }],
            max_tokens: Some(10),
            temperature: Some(0.1),
            stream: false,
        };

        let result = self.call_api(&request).await;
        let response_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(response) => Ok(ProviderHealthStatus {
                is_healthy: true,
                version: Some(response.model),
                last_check: chrono::Utc::now(),
                error_message: None,
                response_time_ms: Some(response_time),
            }),
            Err(e) => Ok(ProviderHealthStatus {
                is_healthy: false,
                version: None,
                last_check: chrono::Utc::now(),
                error_message: Some(e.to_string()),
                response_time_ms: Some(response_time),
            }),
        }
    }

    /// Get provider-specific capabilities
    fn get_capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_json_output: self.config.json_mode.unwrap_or(false),
            supports_streaming: self.config.stream.unwrap_or(false),
            supports_file_operations: false,
            supports_git_operations: false,
            supports_code_execution: false,
            max_context_length: Some(128000), // GPT-4 max context
            supported_languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "rust".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c++".to_string(),
                "c#".to_string(),
                "ruby".to_string(),
                "php".to_string(),
                "swift".to_string(),
                "kotlin".to_string(),
            ],
        }
    }
}

// Tests removed to minimize test suite
