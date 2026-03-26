//! Command registry to replace massive match statements

use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{CliRunner, Commands};

/// Macro to register commands with automatic type checking and error handling
macro_rules! register_command {
    ($self:expr, $name:expr, $runner:ident, $cmd:ident, $pattern:pat => $handler:expr) => {
        $self.register($name, |$runner, $cmd| {
            Box::pin(async move {
                if let $pattern = $cmd {
                    $handler.await
                } else {
                    anyhow::bail!("Command type mismatch for {}", $name)
                }
            })
        });
    };

    // For simple commands without parameters
    ($self:expr, $name:expr, $runner:ident, $handler:expr) => {
        $self.register($name, |$runner, _| Box::pin($handler));
    };
}

type CommandHandler = Box<
    dyn for<'a> Fn(
            &'a CliRunner,
            &'a Commands,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync,
>;

/// Registry that maps commands to their handlers
pub struct CommandRegistry {
    handlers: HashMap<&'static str, CommandHandler>,
}

impl CommandRegistry {
    /// Create and populate the command registry
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };

        // Register all commands using a macro to eliminate duplication
        registry.register_commands();
        registry
    }

    /// Register all command handlers
    fn register_commands(&mut self) {
        // Simple commands without parameters
        register_command!(self, "setup", runner, runner.handle_setup());

        // Commands with parameters
        register_command!(self, "init", runner, cmd,
            Commands::Init { name, repo_url, agents } =>
            runner.init_project(name, repo_url.as_deref(), agents)
        );

        register_command!(self, "task", runner, cmd,
            Commands::Task { action } =>
            runner.handle_task(action)
        );

        register_command!(self, "agents", runner, cmd,
            Commands::Agents { all } =>
            runner.list_agents(*all)
        );

        register_command!(self, "agent-gen", runner, cmd,
            Commands::AgentGen { action } =>
            runner.handle_agent_gen(action)
        );

        register_command!(self, "worktree", runner, cmd,
            Commands::Worktree { action } =>
            runner.handle_worktree(action)
        );

        register_command!(self, "logs", runner, cmd,
            Commands::Logs { follow, agent, lines } =>
            runner.show_logs(*follow, agent.as_deref(), *lines)
        );

        register_command!(self, "config", runner, cmd,
            Commands::Config { action } =>
            runner.handle_config(action)
        );

        register_command!(self, "tutorial", runner, cmd,
            Commands::Tutorial { chapter } =>
            runner.handle_tutorial(*chapter)
        );

        register_command!(self, "interactive", runner, cmd,
            Commands::Interactive { mode, piece } =>
            runner.handle_interactive(mode, piece.as_deref())
        );

        register_command!(self, "pipeline", runner, cmd,
            Commands::Pipeline { task, piece, output_format, timeout, verbose, output_file, .. } =>
            runner.handle_pipeline(task, piece, output_format, *timeout, *verbose, output_file.as_deref())
        );

        register_command!(self, "help", runner, cmd,
            Commands::HelpTopic { topic, search } =>
            runner.handle_help(topic.as_deref(), search.as_deref())
        );

        register_command!(self, "health", runner, cmd,
            Commands::Health { check_agents, check_sessions, resources, diagnose, detailed, format } =>
            runner.handle_health(*check_agents, *check_sessions, *resources, *diagnose, *detailed, format)
        );

        register_command!(self, "doctor", runner, cmd,
            Commands::Doctor { fix, error, check_api } =>
            runner.handle_doctor(*fix, error.as_deref(), *check_api)
        );

        register_command!(self, "quickstart", runner, cmd,
            Commands::Quickstart { name, no_prompt, all_agents, with_tests } =>
            runner.handle_quickstart(name.as_deref(), *no_prompt, *all_agents, *with_tests)
        );

        register_command!(self, "piece", runner, cmd,
            Commands::Piece { action } =>
            runner.handle_piece(action)
        );

        register_command!(self, "repertoire", runner, cmd,
            Commands::Repertoire { action } =>
            runner.handle_repertoire(action)
        );

        register_command!(self, "sangha", runner, cmd,
            Commands::Sangha { action } =>
            runner.handle_sangha(action)
        );

        register_command!(self, "extend", runner, cmd,
            Commands::Extend { action } =>
            runner.handle_extend(action)
        );

        register_command!(self, "search", runner, cmd,
            Commands::Search { action } =>
            runner.handle_search_cmd(action)
        );

        register_command!(self, "evolution", runner, cmd,
            Commands::Evolution { action } =>
            runner.handle_evolution(action)
        );

        register_command!(self, "harness", runner, cmd,
            Commands::Harness { action } =>
            runner.handle_harness(action)
        );

        register_command!(self, "approve", runner, cmd,
            Commands::Approve { action } =>
            runner.handle_approve(action)
        );

        register_command!(self, "run", runner, cmd,
            Commands::Run { action } =>
            runner.handle_run(action)
        );

        register_command!(self, "scaffold", runner, cmd,
            Commands::Scaffold { dir, task, piece, timeout } =>
            runner.handle_scaffold(dir, task, piece, *timeout)
        );
    }

    /// Register a command handler
    fn register(
        &mut self,
        name: &'static str,
        handler: impl for<'a> Fn(
            &'a CliRunner,
            &'a Commands,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
    ) {
        self.handlers.insert(name, Box::new(handler));
    }

    /// Execute a command
    pub async fn execute(&self, runner: &CliRunner, command: &Commands) -> Result<()> {
        let command_name = Self::get_command_name(command);

        if let Some(handler) = self.handlers.get(command_name) {
            handler(runner, command).await
        } else {
            anyhow::bail!("Unknown command: {}", command_name)
        }
    }

    /// Get command name from enum variant
    fn get_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Init { .. } => "init",
            Commands::Task { .. } => "task",
            Commands::Agents { .. } => "agents",
            Commands::AgentGen { .. } => "agent-gen",
            Commands::Worktree { .. } => "worktree",
            Commands::Logs { .. } => "logs",
            Commands::Config { .. } => "config",
            Commands::Setup => "setup",
            Commands::Tutorial { .. } => "tutorial",
            Commands::Interactive { .. } => "interactive",
            Commands::HelpTopic { .. } => "help",
            Commands::Health { .. } => "health",
            Commands::Doctor { .. } => "doctor",
            Commands::Quickstart { .. } => "quickstart",
            Commands::Pipeline { .. } => "pipeline",
            Commands::Piece { .. } => "piece",
            Commands::Repertoire { .. } => "repertoire",
            Commands::Sangha { .. } => "sangha",
            Commands::Extend { .. } => "extend",
            Commands::Search { .. } => "search",
            Commands::Evolution { .. } => "evolution",
            Commands::Harness { .. } => "harness",
            Commands::Approve { .. } => "approve",
            Commands::Run { .. } => "run",
            Commands::Scaffold { .. } => "scaffold",
        }
    }
}

/// Macro to generate command registration with minimal boilerplate
#[macro_export]
macro_rules! register_command {
    ($registry:expr, $name:literal, $pattern:pat => $handler:expr) => {
        $registry.register($name, |runner, cmd| {
            Box::pin(async move {
                if let $pattern = cmd {
                    $handler(runner).await
                } else {
                    anyhow::bail!("Command type mismatch in registry handler")
                }
            })
        });
    };
}

/// Get the global command registry instance
pub fn get_command_registry() -> CommandRegistry {
    CommandRegistry::new()
}
