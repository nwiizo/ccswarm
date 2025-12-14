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
        register_command!(self, "tui", runner, runner.start_tui());
        register_command!(self, "stop", runner, runner.stop_orchestrator());
        register_command!(self, "setup", runner, runner.handle_setup());

        // Commands with parameters - use macro for cleaner registration
        register_command!(self, "init", runner, cmd,
            Commands::Init { name, repo_url, agents } =>
            runner.init_project(name, repo_url.as_deref(), agents)
        );

        register_command!(self, "start", runner, cmd,
            Commands::Start { daemon, port, isolation, use_real_api } =>
            runner.start_orchestrator(*daemon, *port, isolation, *use_real_api)
        );

        register_command!(self, "status", runner, cmd,
            Commands::Status { detailed, agent } =>
            runner.show_status(*detailed, agent.as_deref())
        );

        register_command!(self, "task", runner, cmd,
            Commands::Task { action } =>
            runner.handle_task(action)
        );

        register_command!(self, "agents", runner, cmd,
            Commands::Agents { all } =>
            runner.list_agents(*all)
        );

        register_command!(self, "review", runner, cmd,
            Commands::Review { agent, strict } =>
            runner.run_review(agent.as_deref(), *strict)
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

        register_command!(self, "delegate", runner, cmd,
            Commands::Delegate { action } =>
            runner.handle_delegate(action)
        );

        register_command!(self, "session", runner, cmd,
            Commands::Session { action } =>
            runner.handle_session(action)
        );

        register_command!(self, "resource", runner, cmd,
            Commands::Resource { action } =>
            runner.handle_resource(action)
        );

        register_command!(self, "auto-create", runner, cmd,
            Commands::AutoCreate { description, template: _, auto_deploy, output } =>
            runner.handle_auto_create(description, None, *auto_deploy, output)
        );

        self.register("sangha", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Sangha { action } = cmd {
                    runner.handle_sangha(action).await
                } else {
                    anyhow::bail!("Command type mismatch in registry handler")
                }
            })
        });

        self.register("extend", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Extend { action } = cmd {
                    runner.handle_extend(action).await
                } else {
                    anyhow::bail!("Command type mismatch in registry handler")
                }
            })
        });

        self.register("search", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Search { action } = cmd {
                    runner.handle_search(action).await
                } else {
                    anyhow::bail!("Command type mismatch in registry handler")
                }
            })
        });

        #[cfg(feature = "claude-acp")]
        register_command!(self, "claude-acp", runner, cmd,
            Commands::ClaudeACP { command } =>
            runner.handle_claude_acp(command)
        );

        register_command!(self, "evolution", runner, cmd,
            Commands::Evolution { action } =>
            runner.handle_evolution(action)
        );

        register_command!(self, "quality", runner, cmd,
            Commands::Quality { action } =>
            runner.handle_quality(action)
        );

        register_command!(self, "template", runner, cmd,
            Commands::Template { action } =>
            runner.handle_template(action)
        );

        register_command!(self, "subagent", _runner, cmd,
            Commands::Subagent { command } =>
            crate::cli::subagent_commands::execute_subagent_command(command.clone())
        );

        register_command!(self, "tutorial", runner, cmd,
            Commands::Tutorial { chapter } =>
            runner.handle_tutorial(*chapter)
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
            Commands::Start { .. } => "start",
            Commands::Tui => "tui",
            Commands::Stop => "stop",
            Commands::Status { .. } => "status",
            Commands::Task { .. } => "task",
            Commands::Agents { .. } => "agents",
            Commands::Review { .. } => "review",
            Commands::Worktree { .. } => "worktree",
            Commands::Logs { .. } => "logs",
            Commands::Config { .. } => "config",
            Commands::Delegate { .. } => "delegate",
            Commands::Session { .. } => "session",
            Commands::Resource { .. } => "resource",
            Commands::AutoCreate { .. } => "auto-create",
            Commands::Sangha { .. } => "sangha",
            Commands::Extend { .. } => "extend",
            Commands::Search { .. } => "search",
            #[cfg(feature = "claude-acp")]
            Commands::ClaudeACP { .. } => "claude-acp",
            Commands::Evolution { .. } => "evolution",
            Commands::Quality { .. } => "quality",
            Commands::Template { .. } => "template",
            Commands::Subagent { .. } => "subagent",
            Commands::Setup => "setup",
            Commands::Tutorial { .. } => "tutorial",
            Commands::HelpTopic { .. } => "help",
            Commands::Health { .. } => "health",
            Commands::Doctor { .. } => "doctor",
            Commands::Quickstart { .. } => "quickstart",
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
