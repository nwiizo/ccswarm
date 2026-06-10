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

        register_command!(self, "interactive", runner, cmd,
            Commands::Interactive { mode, flow } =>
            runner.handle_interactive(mode, flow.as_deref())
        );

        register_command!(self, "pipeline", runner, cmd,
            Commands::Pipeline { task, flow, output_format, timeout, verbose, output_file, isolate, budget, run_budget_tokens, model_override, auto_commit, create_pr, dry_run, .. } =>
            runner.handle_pipeline_with_dry_run(task, flow, output_format, *timeout, *verbose, output_file.as_deref(), *isolate, *budget, *run_budget_tokens, model_override.as_deref(), *auto_commit, *create_pr, None, *dry_run)
        );

        register_command!(self, "doctor", runner, cmd,
            Commands::Doctor { fix, error, check_api } =>
            runner.handle_doctor(*fix, error.as_deref(), *check_api)
        );

        register_command!(self, "quickstart", runner, cmd,
            Commands::Quickstart { name, no_prompt, all_agents, with_tests } =>
            runner.handle_quickstart(name.as_deref(), *no_prompt, *all_agents, *with_tests)
        );

        register_command!(self, "flow", runner, cmd,
            Commands::Flow { action } =>
            runner.handle_piece(action)
        );

        register_command!(self, "repertoire", runner, cmd,
            Commands::Repertoire { action } =>
            runner.handle_repertoire(action)
        );

        register_command!(self, "lab", runner, cmd,
            Commands::Lab { action } =>
            runner.handle_lab(action)
        );

        register_command!(self, "harness", runner, cmd,
            Commands::Harness { action } =>
            runner.handle_harness(action)
        );

        register_command!(self, "approve", runner, cmd,
            Commands::Approve { action } =>
            runner.handle_approve(action)
        );

        register_command!(self, "session", runner, cmd,
            Commands::Session { action } =>
            runner.handle_session(action)
        );

        register_command!(self, "run", runner, cmd,
            Commands::Run { action } =>
            runner.handle_run(action)
        );

        register_command!(self, "scaffold", runner, cmd,
            Commands::Scaffold { dir, task, flow, timeout } =>
            runner.handle_scaffold(dir, task, flow, *timeout)
        );

        register_command!(self, "facets", runner, cmd,
            Commands::Facets { kind, detailed } =>
            runner.handle_facets(kind, *detailed)
        );

        register_command!(self, "tail", runner, cmd,
            Commands::Tail { run_id, no_follow } =>
            runner.handle_tail(run_id.as_deref(), *no_follow)
        );

        register_command!(self, "cost", runner, cmd,
            Commands::Cost { run_id } =>
            runner.handle_cost(run_id.as_deref())
        );

        register_command!(self, "queue", runner, cmd,
            Commands::Queue { action } =>
            runner.handle_queue(action)
        );

        register_command!(self, "undo", runner, cmd,
            Commands::Undo { run_id } =>
            runner.handle_undo(run_id.as_deref())
        );

        register_command!(self, "replay", runner, cmd,
            Commands::Replay { run_id, flow, timeout } =>
            runner.handle_replay(run_id.as_deref(), flow.as_deref(), *timeout)
        );

        register_command!(self, "auto", runner, cmd,
            Commands::Auto { task, flow, watch, poll_secs, max_iterations, wall_budget_secs, stop_on_error, timeout, create_pr, require_approval, approval_timeout } =>
            runner.handle_auto(task.as_deref(), flow, *watch, *poll_secs, *max_iterations, *wall_budget_secs, *stop_on_error, *timeout, *create_pr, require_approval.then(|| std::time::Duration::from_secs(*approval_timeout)))
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
            Commands::Interactive { .. } => "interactive",
            Commands::Doctor { .. } => "doctor",
            Commands::Quickstart { .. } => "quickstart",
            Commands::Facets { .. } => "facets",
            Commands::Tail { .. } => "tail",
            Commands::Cost { .. } => "cost",
            Commands::Queue { .. } => "queue",
            Commands::Undo { .. } => "undo",
            Commands::Replay { .. } => "replay",
            Commands::Auto { .. } => "auto",
            Commands::Pipeline { .. } => "pipeline",
            Commands::Flow { .. } => "flow",
            Commands::Repertoire { .. } => "repertoire",
            Commands::Lab { .. } => "lab",
            Commands::Harness { .. } => "harness",
            Commands::Approve { .. } => "approve",
            Commands::Session { .. } => "session",
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
