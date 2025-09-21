use ccswarm::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn test_quickstart_command_parsing() {
    // Test basic quickstart command
    let args = vec!["ccswarm", "quickstart"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Quickstart {
            name,
            no_prompt,
            all_agents,
            with_tests,
        } => {
            assert!(name.is_none());
            assert!(!no_prompt);
            assert!(!all_agents);
            assert!(!with_tests);
        }
        _ => panic!("Expected Quickstart command"),
    }
}

#[test]
fn test_quickstart_with_options() {
    // Test quickstart with all options
    let args = vec![
        "ccswarm",
        "quickstart",
        "--name",
        "test-project",
        "--no-prompt",
        "--all-agents",
        "--with-tests",
    ];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Quickstart {
            name,
            no_prompt,
            all_agents,
            with_tests,
        } => {
            assert_eq!(name, Some("test-project".to_string()));
            assert!(no_prompt);
            assert!(all_agents);
            assert!(with_tests);
        }
        _ => panic!("Expected Quickstart command"),
    }
}

#[test]
fn test_quickstart_short_options() {
    // Test quickstart with short option for name
    let args = vec!["ccswarm", "quickstart", "-n", "my-app"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Quickstart { name, .. } => {
            assert_eq!(name, Some("my-app".to_string()));
        }
        _ => panic!("Expected Quickstart command"),
    }
}
