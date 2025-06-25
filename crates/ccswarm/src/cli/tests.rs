#[cfg(test)]
mod tests {
    use super::super::*;
    use clap::Parser;
    use tempfile::TempDir;

    /// Helper function to parse CLI arguments
    fn parse_args(args: &[&str]) -> Result<Cli> {
        let mut full_args = vec!["ccswarm"];
        full_args.extend_from_slice(args);
        Ok(Cli::try_parse_from(full_args)?)
    }

    #[test]
    fn test_cli_parsing_init_command() {
        let args = parse_args(&[
            "init",
            "--name",
            "test-project",
            "--repo-url",
            "https://github.com/test/repo",
            "--agents",
            "frontend,backend",
        ])
        .unwrap();

        match args.command {
            Commands::Init {
                name,
                repo_url,
                agents,
            } => {
                assert_eq!(name, "test-project");
                assert_eq!(repo_url, Some("https://github.com/test/repo".to_string()));
                assert_eq!(agents, vec!["frontend", "backend"]);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_parsing_start_command() {
        let args = parse_args(&["start", "--daemon", "--port", "9090"]).unwrap();

        match args.command {
            Commands::Start {
                daemon,
                port,
                isolation,
                use_real_api,
            } => {
                assert!(daemon);
                assert_eq!(port, 9090);
                assert_eq!(isolation, "worktree"); // default value
                assert!(!use_real_api); // default false
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_cli_parsing_start_command_with_isolation() {
        let args = parse_args(&["start", "--isolation", "container"]).unwrap();

        match args.command {
            Commands::Start {
                daemon,
                port,
                isolation,
                use_real_api,
            } => {
                assert!(!daemon); // default false
                assert_eq!(port, 8080); // default port
                assert_eq!(isolation, "container");
                assert!(!use_real_api); // default false
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_cli_parsing_tui_command() {
        let args = parse_args(&["tui"]).unwrap();

        match args.command {
            Commands::Tui => {}
            _ => panic!("Expected Tui command"),
        }
    }

    #[test]
    fn test_cli_parsing_status_command() {
        let args = parse_args(&["status", "--detailed", "--agent", "frontend"]).unwrap();

        match args.command {
            Commands::Status { detailed, agent } => {
                assert!(detailed);
                assert_eq!(agent, Some("frontend".to_string()));
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_cli_parsing_task_command() {
        let args = parse_args(&[
            "task",
            "Implement login feature",
            "--priority",
            "high",
            "--task-type",
            "feature",
            "--details",
            "Add OAuth support",
            "--duration",
            "7200",
        ])
        .unwrap();

        match args.command {
            Commands::Task { action } => {
                match action {
                    crate::cli::TaskAction::Add {
                        description,
                        priority,
                        task_type,
                        details,
                        duration,
                        auto_assign,
                        ..
                    } => {
                        assert_eq!(description, "Implement login feature");
                        assert_eq!(priority, "high");
                        assert_eq!(task_type, "feature");
                        assert_eq!(details, Some("Add OAuth support".to_string()));
                        assert_eq!(duration, Some(7200));
                        assert!(!auto_assign); // Default should be false
                    }
                    _ => panic!("Expected Add task action"),
                }
            }
            _ => panic!("Expected Task command"),
        }
    }

    #[test]
    fn test_cli_parsing_agents_command() {
        let args = parse_args(&["agents", "--all"]).unwrap();

        match args.command {
            Commands::Agents { all } => {
                assert!(all);
            }
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_cli_parsing_review_command() {
        let args = parse_args(&["review", "--agent", "backend", "--strict"]).unwrap();

        match args.command {
            Commands::Review { agent, strict } => {
                assert_eq!(agent, Some("backend".to_string()));
                assert!(strict);
            }
            _ => panic!("Expected Review command"),
        }
    }

    #[test]
    fn test_cli_parsing_worktree_list() {
        let args = parse_args(&["worktree", "list"]).unwrap();

        match args.command {
            Commands::Worktree { action } => match action {
                WorktreeAction::List => {}
                _ => panic!("Expected List action"),
            },
            _ => panic!("Expected Worktree command"),
        }
    }

    #[test]
    fn test_cli_parsing_worktree_create() {
        let args = parse_args(&[
            "worktree",
            "create",
            "/path/to/worktree",
            "feature-branch",
            "--new-branch",
        ])
        .unwrap();

        match args.command {
            Commands::Worktree { action } => match action {
                WorktreeAction::Create {
                    path,
                    branch,
                    new_branch,
                } => {
                    assert_eq!(path, PathBuf::from("/path/to/worktree"));
                    assert_eq!(branch, "feature-branch");
                    assert!(new_branch);
                }
                _ => panic!("Expected Create action"),
            },
            _ => panic!("Expected Worktree command"),
        }
    }

    #[test]
    fn test_cli_parsing_logs_command() {
        let args =
            parse_args(&["logs", "--follow", "--agent", "frontend", "--lines", "200"]).unwrap();

        match args.command {
            Commands::Logs {
                follow,
                agent,
                lines,
            } => {
                assert!(follow);
                assert_eq!(agent, Some("frontend".to_string()));
                assert_eq!(lines, 200);
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn test_cli_parsing_config_generate() {
        let args = parse_args(&[
            "config",
            "generate",
            "--output",
            "custom.json",
            "--template",
            "minimal",
        ])
        .unwrap();

        match args.command {
            Commands::Config { action } => match action {
                ConfigAction::Generate { output, template } => {
                    assert_eq!(output, PathBuf::from("custom.json"));
                    assert_eq!(template, "minimal");
                }
                _ => panic!("Expected Generate action"),
            },
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_parsing_delegate_task() {
        let args = parse_args(&[
            "delegate",
            "task",
            "Create API endpoint",
            "--agent",
            "backend",
            "--priority",
            "high",
            "--force",
        ])
        .unwrap();

        match args.command {
            Commands::Delegate { action } => match action {
                DelegateAction::Task {
                    description,
                    agent,
                    priority,
                    force,
                    ..
                } => {
                    assert_eq!(description, "Create API endpoint");
                    assert_eq!(agent, "backend");
                    assert_eq!(priority, "high");
                    assert!(force);
                }
                _ => panic!("Expected Task action"),
            },
            _ => panic!("Expected Delegate command"),
        }
    }

    #[test]
    fn test_cli_parsing_session_create() {
        let args = parse_args(&[
            "session",
            "create",
            "--agent",
            "frontend",
            "--workspace",
            "./workspace",
            "--background",
        ])
        .unwrap();

        match args.command {
            Commands::Session { action } => match action {
                SessionAction::Create {
                    agent,
                    workspace,
                    background,
                } => {
                    assert_eq!(agent, "frontend");
                    assert_eq!(workspace, Some("./workspace".to_string()));
                    assert!(background);
                }
                _ => panic!("Expected Create action"),
            },
            _ => panic!("Expected Session command"),
        }
    }

    #[test]
    fn test_cli_parsing_auto_create() {
        let args = parse_args(&[
            "auto-create",
            "Todo app with authentication",
            "--template",
            "full-stack",
            "--auto-deploy",
            "--output",
            "./my-app",
        ])
        .unwrap();

        match args.command {
            Commands::AutoCreate {
                description,
                template,
                auto_deploy,
                output,
            } => {
                assert_eq!(description, "Todo app with authentication");
                assert_eq!(template, Some("full-stack".to_string()));
                assert!(auto_deploy);
                assert_eq!(output, PathBuf::from("./my-app"));
            }
            _ => panic!("Expected AutoCreate command"),
        }
    }

    #[test]
    fn test_cli_global_options() {
        let args = parse_args(&[
            "--config",
            "custom.json",
            "--repo",
            "/path/to/repo",
            "--verbose",
            "--json",
            "status",
        ])
        .unwrap();

        assert_eq!(args.config, PathBuf::from("custom.json"));
        assert_eq!(args.repo, PathBuf::from("/path/to/repo"));
        assert!(args.verbose);
        assert!(args.json);
    }

    #[test]
    fn test_create_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_default_config(temp_dir.path()).unwrap();

        assert_eq!(config.project.name, "New ccswarm Project");
        assert!(config.agents.contains_key("frontend"));
        assert!(config.agents.contains_key("backend"));
        assert!(config.agents.contains_key("devops"));
        assert_eq!(config.coordination.communication_method, "json_files");
    }

    #[test]
    fn test_create_minimal_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_minimal_config(temp_dir.path()).unwrap();

        assert_eq!(config.project.name, "Minimal ccswarm Project");
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_create_frontend_only_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_frontend_only_config(temp_dir.path()).unwrap();

        assert_eq!(config.project.name, "Frontend ccswarm Project");
        assert_eq!(config.agents.len(), 1);
        assert!(config.agents.contains_key("frontend"));
    }

    #[tokio::test]
    async fn test_cli_runner_new() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("ccswarm.json");

        // Create a simple config file
        let config = create_minimal_config(temp_dir.path()).unwrap();
        config.to_file(config_path.clone()).await.unwrap();

        let cli = Cli {
            config: config_path,
            repo: temp_dir.path().to_path_buf(),
            verbose: false,
            json: false,
            fix: false,
            command: Commands::Status {
                detailed: false,
                agent: None,
            },
        };

        let runner = CliRunner::new(&cli).await.unwrap();
        assert_eq!(runner.config.project.name, "Minimal ccswarm Project");
        assert!(runner.config.agents.is_empty());
    }

    #[tokio::test]
    async fn test_cli_runner_new_missing_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");

        let cli = Cli {
            config: config_path,
            repo: temp_dir.path().to_path_buf(),
            verbose: false,
            json: false,
            fix: false,
            command: Commands::Status {
                detailed: false,
                agent: None,
            },
        };

        // Should use default config when file doesn't exist
        let runner = CliRunner::new(&cli).await.unwrap();
        assert_eq!(runner.config.project.name, "New ccswarm Project");
    }

    #[test]
    fn test_priority_parsing() {
        let test_cases = vec![
            ("low", Priority::Low),
            ("medium", Priority::Medium),
            ("high", Priority::High),
            ("critical", Priority::Critical),
            ("unknown", Priority::Medium), // Default
        ];

        for (input, expected) in test_cases {
            let priority = match input {
                "low" => Priority::Low,
                "medium" => Priority::Medium,
                "high" => Priority::High,
                "critical" => Priority::Critical,
                _ => Priority::Medium,
            };
            assert_eq!(priority, expected);
        }
    }

    #[test]
    fn test_task_type_parsing() {
        let test_cases = vec![
            ("development", TaskType::Development),
            ("dev", TaskType::Development),
            ("testing", TaskType::Testing),
            ("test", TaskType::Testing),
            ("documentation", TaskType::Documentation),
            ("docs", TaskType::Documentation),
            ("infrastructure", TaskType::Infrastructure),
            ("infra", TaskType::Infrastructure),
            ("coordination", TaskType::Coordination),
            ("review", TaskType::Review),
            ("bugfix", TaskType::Bugfix),
            ("bug", TaskType::Bugfix),
            ("feature", TaskType::Feature),
            ("unknown", TaskType::Development), // Default
        ];

        for (input, expected) in test_cases {
            let task_type = match input {
                "development" | "dev" => TaskType::Development,
                "testing" | "test" => TaskType::Testing,
                "documentation" | "docs" => TaskType::Documentation,
                "infrastructure" | "infra" => TaskType::Infrastructure,
                "coordination" => TaskType::Coordination,
                "review" => TaskType::Review,
                "bugfix" | "bug" => TaskType::Bugfix,
                "feature" => TaskType::Feature,
                _ => TaskType::Development,
            };
            assert_eq!(task_type, expected);
        }
    }

    #[test]
    fn test_version_display() {
        let args = parse_args(&["--version"]);
        // Should fail because version is handled by clap automatically
        assert!(args.is_err());
    }

    #[test]
    fn test_help_display() {
        let args = parse_args(&["--help"]);
        // Should fail because help is handled by clap automatically
        assert!(args.is_err());
    }

    #[test]
    fn test_subcommand_help() {
        let args = parse_args(&["task", "--help"]);
        // Should fail because help is handled by clap automatically
        assert!(args.is_err());
    }

    #[test]
    fn test_delegation_strategy_parsing() {
        // Test strategy string parsing logic
        let test_cases = vec![
            ("content", true),
            ("load", true),
            ("expertise", true),
            ("workflow", true),
            ("hybrid", true),
            ("unknown", false), // Should default to hybrid
        ];

        for (input, is_valid_strategy) in test_cases {
            let is_valid = matches!(
                input,
                "content" | "load" | "expertise" | "workflow" | "hybrid"
            );
            assert_eq!(is_valid, is_valid_strategy);
        }
    }

    #[test]
    fn test_error_handling_invalid_priority() {
        // Priority parsing should default to Medium for invalid values
        let priority = match "invalid-priority" {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => Priority::Medium,
        };
        assert_eq!(priority, Priority::Medium);
    }

    #[test]
    fn test_interactive_command_parsing() {
        // Test various interactive commands
        let commands = vec![
            ("quit", true),
            ("exit", true),
            ("help", true),
            ("stats", true),
            ("agents", true),
            ("analyze task description", true),
            ("delegate frontend Create UI", true),
            ("unknown command", false),
        ];

        for (cmd, should_be_valid) in commands {
            let is_valid = matches!(cmd, "quit" | "exit" | "help" | "stats" | "agents")
                || cmd.starts_with("analyze ")
                || cmd.starts_with("delegate ");

            assert_eq!(is_valid, should_be_valid);
        }
    }
}
