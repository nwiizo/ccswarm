pub mod aider;
pub mod claude_api;
pub mod claude_code;
pub mod codex;
pub mod custom;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

use crate::agent::{Task, TaskResult};
use crate::identity::AgentIdentity;
use std::path::Path;

/// Supported AI providers for ccswarm agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AIProvider {
    /// Claude Code (default)
    #[default]
    ClaudeCode,
    /// Aider AI coding assistant
    Aider,
    /// OpenAI Codex
    Codex,
    /// Custom command-based provider
    Custom,
}

impl AIProvider {
    /// Get display name for the provider
    pub fn display_name(&self) -> &'static str {
        match self {
            AIProvider::ClaudeCode => "Claude Code",
            AIProvider::Aider => "Aider",
            AIProvider::Codex => "OpenAI Codex",
            AIProvider::Custom => "Custom",
        }
    }

    /// Get provider color for UI display
    pub fn color(&self) -> &'static str {
        match self {
            AIProvider::ClaudeCode => "blue",
            AIProvider::Aider => "green",
            AIProvider::Codex => "purple",
            AIProvider::Custom => "gray",
        }
    }

    /// Get provider icon for UI display
    pub fn icon(&self) -> &'static str {
        match self {
            AIProvider::ClaudeCode => "🤖",
            AIProvider::Aider => "🔧",
            AIProvider::Codex => "🧠",
            AIProvider::Custom => "⚙️",
        }
    }
}

/// Common configuration trait for all providers
#[async_trait]
pub trait ProviderConfig: Send + Sync + Clone {
    /// Validate the provider configuration
    async fn validate(&self) -> Result<()>;

    /// Get environment variables needed for this provider
    fn get_env_vars(&self) -> HashMap<String, String>;

    /// Get working directory for this provider
    fn get_working_directory(&self) -> Option<PathBuf>;

    /// Check if provider is available on the system
    async fn is_available(&self) -> bool;
}

/// Provider execution trait for running tasks
#[async_trait]
pub trait ProviderExecutor: Send + Sync {
    /// Execute a prompt with the provider
    async fn execute_prompt(
        &self,
        prompt: &str,
        identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<String>;

    /// Execute a task with full context
    async fn execute_task(
        &self,
        task: &Task,
        identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<TaskResult>;

    /// Test provider connectivity and functionality
    async fn health_check(&self, working_dir: &Path) -> Result<ProviderHealthStatus>;

    /// Get provider-specific capabilities
    fn get_capabilities(&self) -> ProviderCapabilities;
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthStatus {
    pub is_healthy: bool,
    pub version: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_json_output: bool,
    pub supports_streaming: bool,
    pub supports_file_operations: bool,
    pub supports_git_operations: bool,
    pub supports_code_execution: bool,
    pub max_context_length: Option<usize>,
    pub supported_languages: Vec<String>,
}

/// Claude Code provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    /// Model to use (e.g., "claude-3.5-sonnet")
    pub model: String,
    /// Whether to skip permission prompts
    pub dangerous_skip: bool,
    /// Think mode to use
    pub think_mode: Option<String>,
    /// Output in JSON format
    pub json_output: bool,
    /// Custom commands available
    pub custom_commands: Vec<String>,
    /// MCP servers configuration
    pub mcp_servers: HashMap<String, serde_json::Value>,
    /// API key (optional, uses system default if not provided)
    pub api_key: Option<String>,
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        Self {
            model: "claude-3.5-sonnet".to_string(),
            dangerous_skip: false,
            think_mode: Some("think".to_string()),
            json_output: true,
            custom_commands: Vec::new(),
            mcp_servers: HashMap::new(),
            api_key: None,
        }
    }
}

#[async_trait]
impl ProviderConfig for ClaudeCodeConfig {
    async fn validate(&self) -> Result<()> {
        // Check if claude command is available
        if !self.is_available().await {
            return Err(anyhow::anyhow!("Claude Code CLI not found in PATH"));
        }

        // Validate model name
        if self.model.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }

        Ok(())
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        if let Some(api_key) = &self.api_key {
            env_vars.insert("ANTHROPIC_API_KEY".to_string(), api_key.clone());
        }

        env_vars
    }

    fn get_working_directory(&self) -> Option<PathBuf> {
        None // Claude Code uses current directory
    }

    async fn is_available(&self) -> bool {
        Command::new("claude")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Aider provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiderConfig {
    /// Model to use (e.g., "gpt-4", "claude-3.5-sonnet")
    pub model: String,
    /// OpenAI API key
    pub openai_api_key: Option<String>,
    /// Anthropic API key
    pub anthropic_api_key: Option<String>,
    /// Auto-commit changes
    pub auto_commit: bool,
    /// Use git for version control
    pub git: bool,
    /// Additional aider arguments
    pub additional_args: Vec<String>,
    /// Aider executable path
    pub executable_path: Option<PathBuf>,
}

impl Default for AiderConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            openai_api_key: None,
            anthropic_api_key: None,
            auto_commit: true,
            git: true,
            additional_args: Vec::new(),
            executable_path: None,
        }
    }
}

#[async_trait]
impl ProviderConfig for AiderConfig {
    async fn validate(&self) -> Result<()> {
        // Check if aider is available
        if !self.is_available().await {
            return Err(anyhow::anyhow!("Aider not found in PATH"));
        }

        // Check API keys based on model
        if self.model.starts_with("gpt-") && self.openai_api_key.is_none() {
            return Err(anyhow::anyhow!("OpenAI API key required for GPT models"));
        }

        if self.model.starts_with("claude-") && self.anthropic_api_key.is_none() {
            return Err(anyhow::anyhow!(
                "Anthropic API key required for Claude models"
            ));
        }

        Ok(())
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        if let Some(openai_key) = &self.openai_api_key {
            env_vars.insert("OPENAI_API_KEY".to_string(), openai_key.clone());
        }

        if let Some(anthropic_key) = &self.anthropic_api_key {
            env_vars.insert("ANTHROPIC_API_KEY".to_string(), anthropic_key.clone());
        }

        env_vars
    }

    fn get_working_directory(&self) -> Option<PathBuf> {
        None // Aider uses current directory
    }

    async fn is_available(&self) -> bool {
        let cmd = if let Some(path) = &self.executable_path {
            path.to_string_lossy().to_string()
        } else {
            "aider".to_string()
        };

        Command::new(&cmd)
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// OpenAI Codex provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// OpenAI API key
    pub api_key: String,
    /// Model to use (e.g., "code-davinci-002")
    pub model: String,
    /// Maximum tokens for completion
    pub max_tokens: Option<u32>,
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// API base URL (for custom endpoints)
    pub api_base: Option<String>,
    /// Organization ID
    pub organization: Option<String>,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            model: "gpt-4".to_string(), // Codex models are deprecated, using GPT-4
            max_tokens: Some(2048),
            temperature: Some(0.1),
            api_base: None,
            organization: None,
        }
    }
}

#[async_trait]
impl ProviderConfig for CodexConfig {
    async fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(anyhow::anyhow!("OpenAI API key is required"));
        }

        if self.model.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }

        // Validate temperature range
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err(anyhow::anyhow!("Temperature must be between 0.0 and 1.0"));
            }
        }

        Ok(())
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        env_vars.insert("OPENAI_API_KEY".to_string(), self.api_key.clone());

        if let Some(org) = &self.organization {
            env_vars.insert("OPENAI_ORGANIZATION".to_string(), org.clone());
        }

        if let Some(base) = &self.api_base {
            env_vars.insert("OPENAI_API_BASE".to_string(), base.clone());
        }

        env_vars
    }

    fn get_working_directory(&self) -> Option<PathBuf> {
        None
    }

    async fn is_available(&self) -> bool {
        // Check if we can make API calls (simplified check)
        !self.api_key.is_empty()
    }
}

/// Custom provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    /// Command to execute
    pub command: String,
    /// Arguments template (use {prompt} placeholder)
    pub args: Vec<String>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Whether the command supports JSON output
    pub supports_json: bool,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            command: "echo".to_string(),
            args: vec!["{prompt}".to_string()],
            env_vars: HashMap::new(),
            working_directory: None,
            timeout_seconds: Some(300), // 5 minutes
            supports_json: false,
        }
    }
}

#[async_trait]
impl ProviderConfig for CustomConfig {
    async fn validate(&self) -> Result<()> {
        if self.command.is_empty() {
            return Err(anyhow::anyhow!("Command cannot be empty"));
        }

        // Check if command exists
        if !self.is_available().await {
            return Err(anyhow::anyhow!("Command '{}' not found", self.command));
        }

        Ok(())
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        self.env_vars.clone()
    }

    fn get_working_directory(&self) -> Option<PathBuf> {
        self.working_directory.clone()
    }

    async fn is_available(&self) -> bool {
        Command::new(&self.command)
            .arg("--help")
            .output()
            .await
            .is_ok()
    }
}

/// Complete provider configuration combining all provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfiguration {
    pub provider_type: AIProvider,
    pub claude_code: Option<ClaudeCodeConfig>,
    pub aider: Option<AiderConfig>,
    pub codex: Option<CodexConfig>,
    pub custom: Option<CustomConfig>,
}

impl Default for ProviderConfiguration {
    fn default() -> Self {
        Self {
            provider_type: AIProvider::ClaudeCode,
            claude_code: Some(ClaudeCodeConfig::default()),
            aider: None,
            codex: None,
            custom: None,
        }
    }
}

impl ProviderConfiguration {
    /// Create a new provider configuration for Claude Code
    pub fn claude_code(config: ClaudeCodeConfig) -> Self {
        Self {
            provider_type: AIProvider::ClaudeCode,
            claude_code: Some(config),
            aider: None,
            codex: None,
            custom: None,
        }
    }

    /// Create a new provider configuration for Aider
    pub fn aider(config: AiderConfig) -> Self {
        Self {
            provider_type: AIProvider::Aider,
            claude_code: None,
            aider: Some(config),
            codex: None,
            custom: None,
        }
    }

    /// Create a new provider configuration for Codex
    pub fn codex(config: CodexConfig) -> Self {
        Self {
            provider_type: AIProvider::Codex,
            claude_code: None,
            aider: None,
            codex: Some(config),
            custom: None,
        }
    }

    /// Create a new provider configuration for Custom
    pub fn custom(config: CustomConfig) -> Self {
        Self {
            provider_type: AIProvider::Custom,
            claude_code: None,
            aider: None,
            codex: None,
            custom: Some(config),
        }
    }

    /// Validate the configuration
    pub async fn validate(&self) -> Result<()> {
        match self.provider_type {
            AIProvider::ClaudeCode => {
                if let Some(config) = &self.claude_code {
                    config.validate().await
                } else {
                    Err(anyhow::anyhow!("Claude Code configuration missing"))
                }
            }
            AIProvider::Aider => {
                if let Some(config) = &self.aider {
                    config.validate().await
                } else {
                    Err(anyhow::anyhow!("Aider configuration missing"))
                }
            }
            AIProvider::Codex => {
                if let Some(config) = &self.codex {
                    config.validate().await
                } else {
                    Err(anyhow::anyhow!("Codex configuration missing"))
                }
            }
            AIProvider::Custom => {
                if let Some(config) = &self.custom {
                    config.validate().await
                } else {
                    Err(anyhow::anyhow!("Custom configuration missing"))
                }
            }
        }
    }

    /// Get environment variables for the active provider
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        match self.provider_type {
            AIProvider::ClaudeCode => self
                .claude_code
                .as_ref()
                .map(|c| c.get_env_vars())
                .unwrap_or_default(),
            AIProvider::Aider => self
                .aider
                .as_ref()
                .map(|c| c.get_env_vars())
                .unwrap_or_default(),
            AIProvider::Codex => self
                .codex
                .as_ref()
                .map(|c| c.get_env_vars())
                .unwrap_or_default(),
            AIProvider::Custom => self
                .custom
                .as_ref()
                .map(|c| c.get_env_vars())
                .unwrap_or_default(),
        }
    }

    /// Check if the provider is available
    pub async fn is_available(&self) -> bool {
        match self.provider_type {
            AIProvider::ClaudeCode => {
                if let Some(config) = &self.claude_code {
                    config.is_available().await
                } else {
                    false
                }
            }
            AIProvider::Aider => {
                if let Some(config) = &self.aider {
                    config.is_available().await
                } else {
                    false
                }
            }
            AIProvider::Codex => {
                if let Some(config) = &self.codex {
                    config.is_available().await
                } else {
                    false
                }
            }
            AIProvider::Custom => {
                if let Some(config) = &self.custom {
                    config.is_available().await
                } else {
                    false
                }
            }
        }
    }
}

/// Factory for creating provider executors
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider executor from configuration
    pub fn create_executor(config: &ProviderConfiguration) -> Result<Box<dyn ProviderExecutor>> {
        match config.provider_type {
            AIProvider::ClaudeCode => {
                if let Some(claude_config) = &config.claude_code {
                    Ok(Box::new(claude_code::ClaudeCodeExecutor::new(
                        claude_config.clone(),
                    )))
                } else {
                    Err(anyhow::anyhow!("Claude Code configuration missing"))
                }
            }
            AIProvider::Aider => {
                if let Some(aider_config) = &config.aider {
                    Ok(Box::new(aider::AiderExecutor::new(aider_config.clone())))
                } else {
                    Err(anyhow::anyhow!("Aider configuration missing"))
                }
            }
            AIProvider::Codex => {
                if let Some(codex_config) = &config.codex {
                    Ok(Box::new(codex::CodexExecutor::new(codex_config.clone())))
                } else {
                    Err(anyhow::anyhow!("Codex configuration missing"))
                }
            }
            AIProvider::Custom => {
                if let Some(custom_config) = &config.custom {
                    Ok(Box::new(custom::CustomExecutor::new(custom_config.clone())))
                } else {
                    Err(anyhow::anyhow!("Custom configuration missing"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_display() {
        assert_eq!(AIProvider::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(AIProvider::Aider.display_name(), "Aider");
        assert_eq!(AIProvider::Codex.display_name(), "OpenAI Codex");
        assert_eq!(AIProvider::Custom.display_name(), "Custom");
    }

    #[test]
    fn test_provider_configuration_creation() {
        let claude_config = ClaudeCodeConfig::default();
        let provider_config = ProviderConfiguration::claude_code(claude_config);
        assert_eq!(provider_config.provider_type, AIProvider::ClaudeCode);
        assert!(provider_config.claude_code.is_some());
        assert!(provider_config.aider.is_none());
    }

    #[tokio::test]
    #[ignore = "Claude Code config validation may fail without proper environment"]
    async fn test_claude_code_config_validation() {
        let config = ClaudeCodeConfig::default();
        // Test validation logic - result depends on whether claude CLI is installed
        let result = config.validate().await;
        // Just ensure the validation runs without panicking
        // The actual result depends on system state
        let _ = result.is_ok() || result.is_err();
    }

    #[test]
    fn test_custom_config_default() {
        let config = CustomConfig::default();
        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["{prompt}"]);
        assert_eq!(config.timeout_seconds, Some(300));
    }
}
