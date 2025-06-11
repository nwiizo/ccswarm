#[cfg(test)]
mod provider_tests {
    use crate::providers::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_ai_provider_enum() {
        assert_eq!(AIProvider::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(AIProvider::Aider.display_name(), "Aider");
        assert_eq!(AIProvider::Codex.display_name(), "OpenAI Codex");
        assert_eq!(AIProvider::Custom.display_name(), "Custom");

        assert_eq!(AIProvider::ClaudeCode.icon(), "🤖");
        assert_eq!(AIProvider::Aider.icon(), "🔧");
        assert_eq!(AIProvider::Codex.icon(), "🧠");
        assert_eq!(AIProvider::Custom.icon(), "⚙️");

        assert_eq!(AIProvider::ClaudeCode.color(), "blue");
        assert_eq!(AIProvider::Aider.color(), "green");
        assert_eq!(AIProvider::Codex.color(), "purple");
        assert_eq!(AIProvider::Custom.color(), "gray");
    }

    #[test]
    fn test_claude_code_config_default() {
        let config = ClaudeCodeConfig::default();
        assert_eq!(config.model, "claude-3.5-sonnet");
        assert!(!config.dangerous_skip);
        assert_eq!(config.think_mode, Some("think".to_string()));
        assert!(config.json_output);
        assert!(config.custom_commands.is_empty());
        assert!(config.mcp_servers.is_empty());
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_aider_config_default() {
        let config = AiderConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert!(config.openai_api_key.is_none());
        assert!(config.anthropic_api_key.is_none());
        assert!(config.auto_commit);
        assert!(config.git);
        assert!(config.additional_args.is_empty());
        assert!(config.executable_path.is_none());
    }

    #[test]
    fn test_codex_config_default() {
        let config = CodexConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, Some(2048));
        assert_eq!(config.temperature, Some(0.1));
        assert!(config.api_base.is_none());
        assert!(config.organization.is_none());
    }

    #[test]
    fn test_custom_config_default() {
        let config = CustomConfig::default();
        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["{prompt}"]);
        assert!(config.env_vars.is_empty());
        assert!(config.working_directory.is_none());
        assert_eq!(config.timeout_seconds, Some(300));
        assert!(!config.supports_json);
    }

    #[test]
    fn test_provider_configuration_creation() {
        // Test Claude Code configuration
        let claude_config = ClaudeCodeConfig::default();
        let provider_config = ProviderConfiguration::claude_code(claude_config);
        assert_eq!(provider_config.provider_type, AIProvider::ClaudeCode);
        assert!(provider_config.claude_code.is_some());
        assert!(provider_config.aider.is_none());
        assert!(provider_config.codex.is_none());
        assert!(provider_config.custom.is_none());

        // Test Aider configuration
        let aider_config = AiderConfig::default();
        let provider_config = ProviderConfiguration::aider(aider_config);
        assert_eq!(provider_config.provider_type, AIProvider::Aider);
        assert!(provider_config.claude_code.is_none());
        assert!(provider_config.aider.is_some());
        assert!(provider_config.codex.is_none());
        assert!(provider_config.custom.is_none());

        // Test Codex configuration
        let codex_config = CodexConfig::default();
        let provider_config = ProviderConfiguration::codex(codex_config);
        assert_eq!(provider_config.provider_type, AIProvider::Codex);
        assert!(provider_config.claude_code.is_none());
        assert!(provider_config.aider.is_none());
        assert!(provider_config.codex.is_some());
        assert!(provider_config.custom.is_none());

        // Test Custom configuration
        let custom_config = CustomConfig::default();
        let provider_config = ProviderConfiguration::custom(custom_config);
        assert_eq!(provider_config.provider_type, AIProvider::Custom);
        assert!(provider_config.claude_code.is_none());
        assert!(provider_config.aider.is_none());
        assert!(provider_config.codex.is_none());
        assert!(provider_config.custom.is_some());
    }

    #[tokio::test]
    async fn test_claude_code_config_validation() {
        let config = ClaudeCodeConfig::default();

        // This should fail in test environment as claude CLI isn't available
        let result = config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Claude Code CLI not found"));

        // Test empty model validation
        let invalid_config = ClaudeCodeConfig {
            model: String::new(),
            ..Default::default()
        };
        let result = invalid_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[tokio::test]
    async fn test_aider_config_validation() {
        let config = AiderConfig::default();

        // This should fail in test environment as aider isn't available
        let result = config.validate().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Aider not found"));

        // Test API key validation for GPT models
        let gpt_config = AiderConfig {
            model: "gpt-4".to_string(),
            openai_api_key: None,
            ..Default::default()
        };
        let result = gpt_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("OpenAI API key required"));

        // Test API key validation for Claude models
        let claude_config = AiderConfig {
            model: "claude-3.5-sonnet".to_string(),
            anthropic_api_key: None,
            ..Default::default()
        };
        let result = claude_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Anthropic API key required"));
    }

    #[tokio::test]
    async fn test_codex_config_validation() {
        // Test empty API key
        let empty_key_config = CodexConfig {
            api_key: String::new(),
            ..Default::default()
        };
        let result = empty_key_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("OpenAI API key is required"));

        // Test empty model
        let empty_model_config = CodexConfig {
            api_key: "test-key".to_string(),
            model: String::new(),
            ..Default::default()
        };
        let result = empty_model_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));

        // Test invalid temperature
        let invalid_temp_config = CodexConfig {
            api_key: "test-key".to_string(),
            temperature: Some(2.0), // Invalid: > 1.0
            ..Default::default()
        };
        let result = invalid_temp_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between 0.0 and 1.0"));
    }

    #[tokio::test]
    async fn test_custom_config_validation() {
        // Test empty command
        let empty_command_config = CustomConfig {
            command: String::new(),
            ..Default::default()
        };
        let result = empty_command_config.validate().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command cannot be empty"));

        // Test valid echo command (should exist on most systems)
        let echo_config = CustomConfig {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            ..Default::default()
        };
        let result = echo_config.validate().await;
        // Echo should be available on most systems
        if result.is_err() {
            println!("Echo validation failed: {:?}", result);
        }
    }

    #[test]
    fn test_provider_env_vars() {
        // Test Claude Code env vars
        let claude_config = ClaudeCodeConfig {
            api_key: Some("test-claude-key".to_string()),
            ..Default::default()
        };
        let env_vars = claude_config.get_env_vars();
        assert_eq!(
            env_vars.get("ANTHROPIC_API_KEY"),
            Some(&"test-claude-key".to_string())
        );

        // Test Aider env vars
        let aider_config = AiderConfig {
            openai_api_key: Some("test-openai-key".to_string()),
            anthropic_api_key: Some("test-anthropic-key".to_string()),
            ..Default::default()
        };
        let env_vars = aider_config.get_env_vars();
        assert_eq!(
            env_vars.get("OPENAI_API_KEY"),
            Some(&"test-openai-key".to_string())
        );
        assert_eq!(
            env_vars.get("ANTHROPIC_API_KEY"),
            Some(&"test-anthropic-key".to_string())
        );

        // Test Codex env vars
        let codex_config = CodexConfig {
            api_key: "test-codex-key".to_string(),
            organization: Some("test-org".to_string()),
            api_base: Some("https://custom.api.com".to_string()),
            ..Default::default()
        };
        let env_vars = codex_config.get_env_vars();
        assert_eq!(
            env_vars.get("OPENAI_API_KEY"),
            Some(&"test-codex-key".to_string())
        );
        assert_eq!(
            env_vars.get("OPENAI_ORGANIZATION"),
            Some(&"test-org".to_string())
        );
        assert_eq!(
            env_vars.get("OPENAI_API_BASE"),
            Some(&"https://custom.api.com".to_string())
        );

        // Test Custom env vars
        let mut custom_env = HashMap::new();
        custom_env.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());
        let custom_config = CustomConfig {
            env_vars: custom_env.clone(),
            ..Default::default()
        };
        let env_vars = custom_config.get_env_vars();
        assert_eq!(env_vars, custom_env);
    }

    #[test]
    fn test_provider_factory() {
        // Test Claude Code executor creation
        let claude_config = ProviderConfiguration::claude_code(ClaudeCodeConfig::default());
        let executor_result = ProviderFactory::create_executor(&claude_config);
        assert!(executor_result.is_ok());

        // Test Aider executor creation
        let aider_config = ProviderConfiguration::aider(AiderConfig::default());
        let executor_result = ProviderFactory::create_executor(&aider_config);
        assert!(executor_result.is_ok());

        // Test Codex executor creation
        let codex_config = ProviderConfiguration::codex(CodexConfig::default());
        let executor_result = ProviderFactory::create_executor(&codex_config);
        assert!(executor_result.is_ok());

        // Test Custom executor creation
        let custom_config = ProviderConfiguration::custom(CustomConfig::default());
        let executor_result = ProviderFactory::create_executor(&custom_config);
        assert!(executor_result.is_ok());

        // Test missing configuration error
        let incomplete_config = ProviderConfiguration {
            provider_type: AIProvider::ClaudeCode,
            claude_code: None, // Missing configuration
            aider: None,
            codex: None,
            custom: None,
        };
        let executor_result = ProviderFactory::create_executor(&incomplete_config);
        assert!(executor_result.is_err());
        if let Err(e) = executor_result {
            assert!(e.to_string().contains("Claude Code configuration missing"));
        }
    }

    // Configuration integration tests are temporarily disabled
    // until the config module is fully updated with provider support

    #[tokio::test]
    async fn test_provider_configuration_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        // Create configuration with multiple providers
        let claude_config = ProviderConfiguration::claude_code(ClaudeCodeConfig {
            model: "claude-3.5-sonnet".to_string(),
            dangerous_skip: true,
            think_mode: Some("think_hard".to_string()),
            json_output: true,
            custom_commands: vec!["test".to_string()],
            mcp_servers: HashMap::new(),
            api_key: Some("test-key".to_string()),
        });

        // Test serialization
        let serialized = serde_json::to_string_pretty(&claude_config).unwrap();
        assert!(serialized.contains("claude_code"));
        assert!(serialized.contains("claude-3.5-sonnet"));
        assert!(serialized.contains("test-key"));

        // Test deserialization
        let deserialized: ProviderConfiguration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.provider_type, AIProvider::ClaudeCode);
        assert!(deserialized.claude_code.is_some());

        let claude_config = deserialized.claude_code.unwrap();
        assert_eq!(claude_config.model, "claude-3.5-sonnet");
        assert!(claude_config.dangerous_skip);
        assert_eq!(claude_config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_provider_capabilities() {
        // Test Claude Code capabilities
        let claude_config = ClaudeCodeConfig::default();
        let executor = crate::providers::claude_code::ClaudeCodeExecutor::new(claude_config);
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

        // Test Aider capabilities
        let aider_config = AiderConfig::default();
        let executor = crate::providers::aider::AiderExecutor::new(aider_config);
        let capabilities = executor.get_capabilities();

        assert!(!capabilities.supports_json_output);
        assert!(capabilities.supports_file_operations);
        assert!(capabilities.supports_git_operations);
        assert!(!capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);
        assert_eq!(capabilities.max_context_length, Some(128_000));

        // Test Custom capabilities
        let custom_config = CustomConfig {
            supports_json: true,
            ..Default::default()
        };
        let executor = crate::providers::custom::CustomExecutor::new(custom_config);
        let capabilities = executor.get_capabilities();

        assert!(capabilities.supports_json_output);
        assert!(capabilities.supports_file_operations);
        assert!(capabilities.supports_git_operations);
        assert!(capabilities.supports_code_execution);
        assert!(!capabilities.supports_streaming);
        assert!(capabilities.max_context_length.is_none());
    }
}
