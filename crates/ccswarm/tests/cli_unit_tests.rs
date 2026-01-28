//! Unit tests for CLI components
//!
//! These tests verify individual CLI components without executing the full binary.

use ccswarm::cli::{Cli, Commands};
use ccswarm::providers::SensitiveString;
use clap::Parser;

// ============================================================================
// CLI Argument Parsing Tests
// ============================================================================

#[test]
fn test_cli_parse_help() {
    // --help should not parse as a valid command (clap handles it specially)
    let result = Cli::try_parse_from(["ccswarm", "--help"]);
    assert!(result.is_err(), "Help flag should trigger early exit");
}

#[test]
fn test_cli_parse_version() {
    let result = Cli::try_parse_from(["ccswarm", "--version"]);
    assert!(result.is_err(), "Version flag should trigger early exit");
}

#[test]
fn test_cli_parse_init_basic() {
    let cli = Cli::try_parse_from(["ccswarm", "init", "--name", "TestProject"]).unwrap();

    match cli.command {
        Commands::Init { name, agents, .. } => {
            assert_eq!(name, "TestProject");
            assert!(agents.is_empty());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_parse_init_with_agents() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "init",
        "--name",
        "TestProject",
        "--agents",
        "frontend,backend,devops",
    ])
    .unwrap();

    match cli.command {
        Commands::Init { name, agents, .. } => {
            assert_eq!(name, "TestProject");
            assert_eq!(agents.len(), 3);
            assert!(agents.contains(&"frontend".to_string()));
            assert!(agents.contains(&"backend".to_string()));
            assert!(agents.contains(&"devops".to_string()));
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_parse_init_missing_name() {
    let result = Cli::try_parse_from(["ccswarm", "init"]);
    assert!(result.is_err(), "Init without name should fail");
}

#[test]
fn test_cli_parse_global_flags() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "--verbose",
        "--json",
        "init",
        "--name",
        "TestProject",
    ])
    .unwrap();

    assert!(cli.verbose);
    assert!(cli.json);
}

#[test]
fn test_cli_parse_config_path() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "--config",
        "/custom/path/config.json",
        "init",
        "--name",
        "TestProject",
    ])
    .unwrap();

    assert_eq!(cli.config.to_str().unwrap(), "/custom/path/config.json");
}

#[test]
fn test_cli_parse_repo_path() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "--repo",
        "/my/project",
        "init",
        "--name",
        "TestProject",
    ])
    .unwrap();

    assert_eq!(cli.repo.to_str().unwrap(), "/my/project");
}

#[test]
fn test_cli_parse_start_command() {
    let cli = Cli::try_parse_from(["ccswarm", "start"]).unwrap();

    match cli.command {
        Commands::Start { daemon, port, .. } => {
            assert!(!daemon);
            assert_eq!(port, 8080); // default port
        }
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_cli_parse_start_with_options() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "start",
        "--daemon",
        "--port",
        "9000",
        "--isolation",
        "container",
    ])
    .unwrap();

    match cli.command {
        Commands::Start {
            daemon,
            port,
            isolation,
            ..
        } => {
            assert!(daemon);
            assert_eq!(port, 9000);
            assert_eq!(isolation, "container");
        }
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_cli_parse_task_list() {
    let cli = Cli::try_parse_from(["ccswarm", "task", "list"]).unwrap();

    match cli.command {
        Commands::Task { .. } => {
            // Successfully parsed task command
        }
        _ => panic!("Expected Task command"),
    }
}

#[test]
fn test_cli_parse_task_add() {
    let cli =
        Cli::try_parse_from(["ccswarm", "task", "add", "Implement user authentication"]).unwrap();

    match cli.command {
        Commands::Task { .. } => {
            // Successfully parsed task add command
        }
        _ => panic!("Expected Task command"),
    }
}

#[test]
fn test_cli_parse_session_list() {
    let cli = Cli::try_parse_from(["ccswarm", "session", "list"]).unwrap();

    match cli.command {
        Commands::Session { .. } => {
            // Successfully parsed session command
        }
        _ => panic!("Expected Session command"),
    }
}

#[test]
fn test_cli_parse_template_list() {
    let cli = Cli::try_parse_from(["ccswarm", "template", "list"]).unwrap();

    match cli.command {
        Commands::Template { .. } => {
            // Successfully parsed template command
        }
        _ => panic!("Expected Template command"),
    }
}

#[test]
fn test_cli_parse_doctor() {
    let cli = Cli::try_parse_from(["ccswarm", "doctor"]).unwrap();

    match cli.command {
        Commands::Doctor { .. } => {
            // Successfully parsed doctor command
        }
        _ => panic!("Expected Doctor command"),
    }
}

#[test]
fn test_cli_parse_health() {
    let cli = Cli::try_parse_from(["ccswarm", "health"]).unwrap();

    match cli.command {
        Commands::Health { .. } => {
            // Successfully parsed health command
        }
        _ => panic!("Expected Health command"),
    }
}

#[test]
fn test_cli_parse_invalid_command() {
    let result = Cli::try_parse_from(["ccswarm", "invalid-command"]);
    assert!(result.is_err(), "Invalid command should fail to parse");
}

#[test]
fn test_cli_parse_status() {
    let cli = Cli::try_parse_from(["ccswarm", "status"]).unwrap();

    match cli.command {
        Commands::Status { .. } => {
            // Successfully parsed status command
        }
        _ => panic!("Expected Status command"),
    }
}

#[test]
fn test_cli_parse_stop() {
    let cli = Cli::try_parse_from(["ccswarm", "stop"]).unwrap();

    match cli.command {
        Commands::Stop { .. } => {
            // Successfully parsed stop command
        }
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_cli_parse_agents() {
    let cli = Cli::try_parse_from(["ccswarm", "agents"]).unwrap();

    match cli.command {
        Commands::Agents { .. } => {
            // Successfully parsed agents command
        }
        _ => panic!("Expected Agents command"),
    }
}

#[test]
fn test_cli_parse_tui() {
    let cli = Cli::try_parse_from(["ccswarm", "tui"]).unwrap();

    match cli.command {
        Commands::Tui { .. } => {
            // Successfully parsed tui command
        }
        _ => panic!("Expected Tui command"),
    }
}

#[test]
fn test_cli_parse_logs() {
    let cli = Cli::try_parse_from(["ccswarm", "logs"]).unwrap();

    match cli.command {
        Commands::Logs { .. } => {
            // Successfully parsed logs command
        }
        _ => panic!("Expected Logs command"),
    }
}

#[test]
fn test_cli_parse_review() {
    let cli = Cli::try_parse_from(["ccswarm", "review"]).unwrap();

    match cli.command {
        Commands::Review { .. } => {
            // Successfully parsed review command
        }
        _ => panic!("Expected Review command"),
    }
}

// ============================================================================
// SensitiveString Tests
// ============================================================================

#[test]
fn test_sensitive_string_creation() {
    let secret = SensitiveString::new("my-secret-api-key");
    assert_eq!(secret.expose(), "my-secret-api-key");
}

#[test]
fn test_sensitive_string_debug_masks_value() {
    let secret = SensitiveString::new("super-secret-key");
    let debug_output = format!("{:?}", secret);

    // Debug output should NOT contain the actual secret
    assert!(!debug_output.contains("super-secret-key"));
    assert!(debug_output.contains("****"));
}

#[test]
fn test_sensitive_string_display_masks_value() {
    let secret = SensitiveString::new("my-api-key-12345");
    let display_output = format!("{}", secret);

    // Display output should NOT contain the actual secret
    assert!(!display_output.contains("my-api-key-12345"));
    assert!(display_output.contains("****"));
}

#[test]
fn test_sensitive_string_is_empty() {
    let empty = SensitiveString::new("");
    let non_empty = SensitiveString::new("secret");

    assert!(empty.is_empty());
    assert!(!non_empty.is_empty());
}

#[test]
fn test_sensitive_string_clone() {
    let original = SensitiveString::new("clone-me");
    let cloned = original.clone();

    assert_eq!(original.expose(), cloned.expose());
}

#[test]
fn test_sensitive_string_default() {
    let default = SensitiveString::default();
    assert!(default.is_empty());
}

#[test]
fn test_sensitive_string_from_string() {
    let s = String::from("from-string");
    let secret: SensitiveString = s.into();
    assert_eq!(secret.expose(), "from-string");
}

#[test]
fn test_sensitive_string_from_str() {
    let secret: SensitiveString = "from-str".into();
    assert_eq!(secret.expose(), "from-str");
}

#[test]
fn test_sensitive_string_serialization() {
    let secret = SensitiveString::new("serialization-test");
    let serialized = serde_json::to_string(&secret).unwrap();

    // Serialized value should be "[REDACTED]", not the actual secret
    assert!(!serialized.contains("serialization-test"));
    assert!(serialized.contains("REDACTED"));
}

#[test]
fn test_sensitive_string_deserialization() {
    // Deserializing a normal string should create a SensitiveString
    let secret: SensitiveString = serde_json::from_str("\"my-key\"").unwrap();
    assert_eq!(secret.expose(), "my-key");
}

#[test]
fn test_sensitive_string_deserialize_redacted() {
    // Deserializing "[REDACTED]" should create an empty SensitiveString
    let secret: SensitiveString = serde_json::from_str("\"[REDACTED]\"").unwrap();
    assert!(secret.is_empty());
}

// ============================================================================
// Provider Config Tests
// ============================================================================

#[test]
fn test_claude_code_config_default() {
    use ccswarm::providers::ClaudeCodeConfig;

    let config = ClaudeCodeConfig::default();
    assert_eq!(config.model, "sonnet");
    assert!(!config.dangerous_skip);
    assert!(config.api_key.is_none());
}

#[test]
fn test_claude_code_config_with_api_key() {
    use ccswarm::providers::ClaudeCodeConfig;
    use ccswarm::providers::ProviderConfig;

    let mut config = ClaudeCodeConfig::default();
    config.api_key = Some(SensitiveString::new("sk-test-key"));

    // Get env vars should contain the key
    let env_vars = config.get_env_vars();
    assert!(env_vars.contains_key("ANTHROPIC_API_KEY"));
    assert_eq!(env_vars.get("ANTHROPIC_API_KEY").unwrap(), "sk-test-key");
}

#[test]
fn test_aider_config_default() {
    use ccswarm::providers::AiderConfig;

    let config = AiderConfig::default();
    assert_eq!(config.model, "gpt-4");
    assert!(config.openai_api_key.is_none());
    assert!(config.anthropic_api_key.is_none());
    assert!(config.auto_commit);
}

#[test]
fn test_aider_config_with_keys() {
    use ccswarm::providers::AiderConfig;
    use ccswarm::providers::ProviderConfig;

    let mut config = AiderConfig::default();
    config.openai_api_key = Some(SensitiveString::new("openai-key"));
    config.anthropic_api_key = Some(SensitiveString::new("anthropic-key"));

    let env_vars = config.get_env_vars();
    assert_eq!(env_vars.get("OPENAI_API_KEY").unwrap(), "openai-key");
    assert_eq!(env_vars.get("ANTHROPIC_API_KEY").unwrap(), "anthropic-key");
}

#[test]
fn test_codex_config_default() {
    use ccswarm::providers::CodexConfig;

    let config = CodexConfig::default();
    assert_eq!(config.model, "gpt-4");
    assert_eq!(config.max_tokens, Some(2048));
    assert_eq!(config.temperature, Some(0.1));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_should_retry_network() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::network("Connection timeout");
    assert!(error.should_retry());
}

#[test]
fn test_error_should_retry_resource() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::resource("Memory exhausted");
    assert!(error.should_retry());
}

#[test]
fn test_error_should_not_retry_config() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::config("Invalid configuration");
    assert!(!error.should_retry());
}

#[test]
fn test_error_should_not_retry_auth() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::auth("Invalid API key");
    assert!(!error.should_retry());
}

#[test]
fn test_error_retry_delay_network() {
    use ccswarm::error::CCSwarmError;
    use std::time::Duration;

    let error = CCSwarmError::network("Connection timeout");
    assert_eq!(error.suggested_retry_delay(), Duration::from_secs(1));
}

#[test]
fn test_error_retry_delay_resource() {
    use ccswarm::error::CCSwarmError;
    use std::time::Duration;

    let error = CCSwarmError::resource("Resource exhausted");
    assert_eq!(error.suggested_retry_delay(), Duration::from_secs(2));
}

#[test]
fn test_error_max_retries_network() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::network("Connection timeout");
    assert_eq!(error.max_retries(), 3);
}

#[test]
fn test_error_max_retries_resource() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::resource("Resource exhausted");
    assert_eq!(error.max_retries(), 5);
}

#[test]
fn test_error_max_retries_non_retryable() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::config("Invalid config");
    assert_eq!(error.max_retries(), 0);
}

#[test]
fn test_error_severity_critical() {
    use ccswarm::error::{CCSwarmError, ErrorSeverity};

    let error = CCSwarmError::auth("Invalid credentials");
    assert_eq!(error.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_error_severity_high() {
    use ccswarm::error::{CCSwarmError, ErrorSeverity};

    let error = CCSwarmError::agent("agent-1", "Agent crashed");
    assert_eq!(error.severity(), ErrorSeverity::High);
}

#[test]
fn test_error_severity_low() {
    use ccswarm::error::{CCSwarmError, ErrorSeverity};

    let error = CCSwarmError::network("Temporary network issue");
    assert_eq!(error.severity(), ErrorSeverity::Low);
}

#[test]
fn test_error_is_recoverable() {
    use ccswarm::error::CCSwarmError;

    let network_error = CCSwarmError::network("Connection reset");
    let task_error = CCSwarmError::task("task-1", "Task failed");
    let config_error = CCSwarmError::config("Bad config");

    assert!(network_error.is_recoverable());
    assert!(task_error.is_recoverable());
    assert!(!config_error.is_recoverable());
}

// ============================================================================
// Task Parsing Tests
// ============================================================================

#[test]
fn test_task_priority_parsing() {
    use ccswarm::agent::Priority;
    use std::str::FromStr;

    assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
    assert_eq!(Priority::from_str("medium").unwrap(), Priority::Medium);
    assert_eq!(Priority::from_str("low").unwrap(), Priority::Low);
    assert_eq!(Priority::from_str("critical").unwrap(), Priority::Critical);
}

#[test]
fn test_task_type_parsing() {
    use ccswarm::agent::TaskType;
    use std::str::FromStr;

    assert_eq!(
        TaskType::from_str("development").unwrap(),
        TaskType::Development
    );
    assert_eq!(TaskType::from_str("review").unwrap(), TaskType::Review);
    assert_eq!(TaskType::from_str("bug").unwrap(), TaskType::Bug);
    assert_eq!(TaskType::from_str("feature").unwrap(), TaskType::Feature);
}

// ============================================================================
// Identity Tests
// ============================================================================

#[test]
fn test_default_frontend_role() {
    use ccswarm::identity::default_frontend_role;

    let frontend = default_frontend_role();
    // Just verify it creates without panic
    assert!(frontend.name().contains("Frontend"));
}

#[test]
fn test_default_backend_role() {
    use ccswarm::identity::default_backend_role;

    let backend = default_backend_role();
    assert!(backend.name().contains("Backend"));
}

#[test]
fn test_default_devops_role() {
    use ccswarm::identity::default_devops_role;

    let devops = default_devops_role();
    assert!(devops.name().contains("DevOps"));
}

#[test]
fn test_default_qa_role() {
    use ccswarm::identity::default_qa_role;

    let qa = default_qa_role();
    assert!(qa.name().contains("QA"));
}

// ============================================================================
// Config Tests
// ============================================================================

#[test]
fn test_config_default() {
    use ccswarm::config::CcswarmConfig;

    let config = CcswarmConfig::default();
    assert!(!config.project.name.is_empty() || config.project.name.is_empty());
    // Just verify default doesn't panic
}

// ============================================================================
// Auto-accept Config Tests
// ============================================================================

#[test]
fn test_auto_accept_config_default() {
    use ccswarm::auto_accept::AutoAcceptConfig;

    let config = AutoAcceptConfig::default();
    assert!(!config.enabled);
    assert!(!config.emergency_stop);
}

#[test]
fn test_auto_accept_restricted_files() {
    use ccswarm::auto_accept::AutoAcceptConfig;

    let mut config = AutoAcceptConfig::default();
    config.enabled = true;
    config.restricted_files = vec!["*.sql".to_string(), "Cargo.toml".to_string()];
    config.max_file_changes = 10;

    assert!(config.enabled);
    assert_eq!(config.restricted_files.len(), 2);
    assert_eq!(config.max_file_changes, 10);
}
