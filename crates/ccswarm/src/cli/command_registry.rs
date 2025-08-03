//! Command registry to replace massive match statements

use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{CliRunner, Commands};

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
        self.register("tui", |runner, _| Box::pin(runner.start_tui()));
        self.register("stop", |runner, _| Box::pin(runner.stop_orchestrator()));
        self.register("setup", |runner, _| Box::pin(runner.handle_setup()));

        // Commands with parameters - use closures to extract parameters
        self.register("init", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Init {
                    name,
                    repo_url,
                    agents,
                } = cmd
                {
                    runner.init_project(name, repo_url.as_deref(), agents).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("start", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Start {
                    daemon,
                    port,
                    isolation,
                    use_real_api,
                } = cmd
                {
                    runner
                        .start_orchestrator(*daemon, *port, isolation, *use_real_api)
                        .await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("status", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Status { detailed, agent } = cmd {
                    runner.show_status(*detailed, agent.as_deref()).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("task", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Task { action } = cmd {
                    runner.handle_task(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("agents", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Agents { all } = cmd {
                    runner.list_agents(*all).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("review", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Review { agent, strict } = cmd {
                    runner.run_review(agent.as_deref(), *strict).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("worktree", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Worktree { action } = cmd {
                    runner.handle_worktree(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("logs", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Logs {
                    follow,
                    agent,
                    lines,
                } = cmd
                {
                    runner.show_logs(*follow, agent.as_deref(), *lines).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("config", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Config { action } = cmd {
                    runner.handle_config(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("delegate", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Delegate { action } = cmd {
                    runner.handle_delegate(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("session", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Session { action } = cmd {
                    runner.handle_session(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("resource", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Resource { action } = cmd {
                    runner.handle_resource(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("auto-create", |runner, cmd| {
            Box::pin(async move {
                if let Commands::AutoCreate {
                    description,
                    template: _,
                    auto_deploy,
                    output,
                } = cmd
                {
                    runner
                        .handle_auto_create(description, None, *auto_deploy, output)
                        .await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("sangha", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Sangha { action } = cmd {
                    runner.handle_sangha(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("extend", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Extend { action } = cmd {
                    runner.handle_extend(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("search", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Search { action } = cmd {
                    runner.handle_search(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("evolution", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Evolution { action } = cmd {
                    runner.handle_evolution(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("quality", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Quality { action } = cmd {
                    runner.handle_quality(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("template", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Template { action } = cmd {
                    runner.handle_template(action).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("tutorial", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Tutorial { chapter } = cmd {
                    runner.handle_tutorial(*chapter).await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("help", |runner, cmd| {
            Box::pin(async move {
                if let Commands::HelpTopic { topic, search } = cmd {
                    runner
                        .handle_help(topic.as_deref(), search.as_deref())
                        .await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("health", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Health {
                    check_agents,
                    check_sessions,
                    resources,
                    diagnose,
                    detailed,
                    format,
                } = cmd
                {
                    runner
                        .handle_health(
                            *check_agents,
                            *check_sessions,
                            *resources,
                            *diagnose,
                            *detailed,
                            format,
                        )
                        .await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("doctor", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Doctor {
                    fix,
                    error,
                    check_api,
                } = cmd
                {
                    runner
                        .handle_doctor(*fix, error.as_deref(), *check_api)
                        .await
                } else {
                    unreachable!()
                }
            })
        });

        self.register("quickstart", |runner, cmd| {
            Box::pin(async move {
                if let Commands::Quickstart {
                    name,
                    no_prompt,
                    all_agents,
                    with_tests,
                } = cmd
                {
                    runner
                        .handle_quickstart(name.as_deref(), *no_prompt, *all_agents, *with_tests)
                        .await
                } else {
                    unreachable!()
                }
            })
        });
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
            Commands::Evolution { .. } => "evolution",
            Commands::Quality { .. } => "quality",
            Commands::Template { .. } => "template",
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
                    unreachable!()
                }
            })
        });
    };
}

/// Get the global command registry instance
pub fn get_command_registry() -> CommandRegistry {
    CommandRegistry::new()
}
