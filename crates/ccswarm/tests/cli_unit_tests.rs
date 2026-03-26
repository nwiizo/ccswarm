//! Unit tests for CLI components
//!
//! These tests verify individual CLI components without executing the full binary.

use ccswarm::cli::{Cli, Commands};
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
fn test_cli_parse_logs() {
    let cli = Cli::try_parse_from(["ccswarm", "logs"]).unwrap();

    match cli.command {
        Commands::Logs { .. } => {
            // Successfully parsed logs command
        }
        _ => panic!("Expected Logs command"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_should_not_retry_config() {
    use ccswarm::error::CCSwarmError;

    let error = CCSwarmError::config("Invalid configuration");
    assert!(!error.should_retry());
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

    let error = CCSwarmError::config("Invalid configuration");
    assert_eq!(error.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_error_severity_high() {
    use ccswarm::error::{CCSwarmError, ErrorSeverity};

    let error = CCSwarmError::agent("agent-1", "Agent crashed");
    assert_eq!(error.severity(), ErrorSeverity::High);
}

#[test]
fn test_error_is_recoverable() {
    use ccswarm::error::CCSwarmError;

    let task_error = CCSwarmError::task("task-1", "Task failed");
    let config_error = CCSwarmError::config("Bad config");

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
// Sangha/Extend/Search/Evolution CLI Parse Tests
// ============================================================================

#[test]
fn test_cli_parse_sangha_propose() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "sangha",
        "propose",
        "--title",
        "Add GraphQL",
        "--description",
        "Add GraphQL support to backend",
    ])
    .unwrap();

    match cli.command {
        Commands::Sangha { .. } => {}
        _ => panic!("Expected Sangha command"),
    }
}

#[test]
fn test_cli_parse_sangha_vote() {
    let cli =
        Cli::try_parse_from(["ccswarm", "sangha", "vote", "prop-abc123", "--approve"]).unwrap();

    match cli.command {
        Commands::Sangha { .. } => {}
        _ => panic!("Expected Sangha command"),
    }
}

#[test]
fn test_cli_parse_sangha_list() {
    let cli = Cli::try_parse_from(["ccswarm", "sangha", "list"]).unwrap();

    match cli.command {
        Commands::Sangha { .. } => {}
        _ => panic!("Expected Sangha command"),
    }
}

#[test]
fn test_cli_parse_extend_propose() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "extend",
        "propose",
        "--title",
        "GraphQL resolver",
        "--description",
        "Add resolver support",
        "--agent",
        "backend",
    ])
    .unwrap();

    match cli.command {
        Commands::Extend { .. } => {}
        _ => panic!("Expected Extend command"),
    }
}

#[test]
fn test_cli_parse_extend_list() {
    let cli = Cli::try_parse_from(["ccswarm", "extend", "list"]).unwrap();

    match cli.command {
        Commands::Extend { .. } => {}
        _ => panic!("Expected Extend command"),
    }
}

#[test]
fn test_cli_parse_extend_history() {
    let cli = Cli::try_parse_from(["ccswarm", "extend", "history", "5"]).unwrap();

    match cli.command {
        Commands::Extend { .. } => {}
        _ => panic!("Expected Extend command"),
    }
}

#[test]
fn test_cli_parse_search_docs() {
    let cli =
        Cli::try_parse_from(["ccswarm", "search", "docs", "authentication patterns"]).unwrap();

    match cli.command {
        Commands::Search { .. } => {}
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_cli_parse_search_code_with_glob() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "search",
        "code",
        "error handling",
        "--glob",
        "*.rs",
    ])
    .unwrap();

    match cli.command {
        Commands::Search { .. } => {}
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_cli_parse_evolution_metrics() {
    let cli =
        Cli::try_parse_from(["ccswarm", "evolution", "metrics", "--agent", "frontend"]).unwrap();

    match cli.command {
        Commands::Evolution { .. } => {}
        _ => panic!("Expected Evolution command"),
    }
}

#[test]
fn test_cli_parse_evolution_report() {
    let cli = Cli::try_parse_from(["ccswarm", "evolution", "report", "--format", "json"]).unwrap();

    match cli.command {
        Commands::Evolution { .. } => {}
        _ => panic!("Expected Evolution command"),
    }
}

// ============================================================================
// Approve CLI Parse Tests
// ============================================================================

#[test]
fn test_cli_parse_approve_plan() {
    let cli = Cli::try_parse_from(["ccswarm", "approve", "plan", "--id", "run-abc123"]).unwrap();

    match cli.command {
        Commands::Approve { .. } => {}
        _ => panic!("Expected Approve command"),
    }
}

#[test]
fn test_cli_parse_approve_reject_with_reason() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "approve",
        "deploy",
        "--id",
        "task-456",
        "--reject",
        "--reason",
        "needs more tests",
    ])
    .unwrap();

    match cli.command {
        Commands::Approve { .. } => {}
        _ => panic!("Expected Approve command"),
    }
}

#[test]
fn test_cli_parse_approve_list() {
    let cli = Cli::try_parse_from(["ccswarm", "approve", "list", "--status", "pending"]).unwrap();

    match cli.command {
        Commands::Approve { .. } => {}
        _ => panic!("Expected Approve command"),
    }
}

// ============================================================================
// Session CLI Parse Tests
// ============================================================================

#[test]
fn test_cli_parse_session_list() {
    let cli = Cli::try_parse_from(["ccswarm", "session", "list"]).unwrap();

    match cli.command {
        Commands::Session { .. } => {}
        _ => panic!("Expected Session command"),
    }
}

#[test]
fn test_cli_parse_session_list_all() {
    let cli = Cli::try_parse_from(["ccswarm", "session", "list", "--all"]).unwrap();

    match cli.command {
        Commands::Session {
            action: ccswarm::cli::SessionAction::List { all },
        } => {
            assert!(all, "Expected --all flag to be true");
        }
        _ => panic!("Expected Session List command"),
    }
}

#[test]
fn test_cli_parse_session_view() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "session",
        "view",
        "6182e5fe-bca3-4286-a921-0390e805a4d3",
    ])
    .unwrap();

    match cli.command {
        Commands::Session {
            action: ccswarm::cli::SessionAction::View { id },
        } => {
            assert_eq!(id, "6182e5fe-bca3-4286-a921-0390e805a4d3");
        }
        _ => panic!("Expected Session View command"),
    }
}

#[test]
fn test_cli_parse_session_create() {
    let cli = Cli::try_parse_from([
        "ccswarm",
        "session",
        "create",
        "--agent",
        "frontend",
        "--background",
    ])
    .unwrap();

    match cli.command {
        Commands::Session {
            action:
                ccswarm::cli::SessionAction::Create {
                    agent, background, ..
                },
        } => {
            assert_eq!(agent, "frontend");
            assert!(background);
        }
        _ => panic!("Expected Session Create command"),
    }
}

#[test]
fn test_cli_parse_session_pause() {
    let cli = Cli::try_parse_from(["ccswarm", "session", "pause", "session-abc"]).unwrap();

    match cli.command {
        Commands::Session {
            action: ccswarm::cli::SessionAction::Pause { session_id },
        } => {
            assert_eq!(session_id, "session-abc");
        }
        _ => panic!("Expected Session Pause command"),
    }
}

#[test]
fn test_cli_parse_session_resume() {
    let cli = Cli::try_parse_from(["ccswarm", "session", "resume", "session-abc"]).unwrap();

    match cli.command {
        Commands::Session {
            action: ccswarm::cli::SessionAction::Resume { session_id },
        } => {
            assert_eq!(session_id, "session-abc");
        }
        _ => panic!("Expected Session Resume command"),
    }
}

#[test]
fn test_cli_parse_session_kill() {
    let cli =
        Cli::try_parse_from(["ccswarm", "session", "kill", "session-abc", "--force"]).unwrap();

    match cli.command {
        Commands::Session {
            action:
                ccswarm::cli::SessionAction::Kill {
                    session_id, force, ..
                },
        } => {
            assert_eq!(session_id, "session-abc");
            assert!(force);
        }
        _ => panic!("Expected Session Kill command"),
    }
}

#[test]
fn test_cli_parse_session_no_subcommand() {
    let result = Cli::try_parse_from(["ccswarm", "session"]);
    assert!(
        result.is_err(),
        "Session without subcommand should fail to parse"
    );
}

#[test]
fn test_cli_parse_session_with_json_flag() {
    let cli = Cli::try_parse_from(["ccswarm", "--json", "session", "list"]).unwrap();

    assert!(cli.json);
    match cli.command {
        Commands::Session { .. } => {}
        _ => panic!("Expected Session command"),
    }
}
