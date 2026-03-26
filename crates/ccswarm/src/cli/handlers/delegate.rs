use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_delegate(&self, action: &DelegateAction) -> Result<()> {
        match action {
            DelegateAction::Task {
                description,
                agent,
                priority,
                task_type,
                details,
                force,
            } => {
                let task = self.create_task_from_args(
                    description,
                    priority,
                    task_type,
                    details.as_deref(),
                    None,
                )?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Task delegated",
                            "task": task,
                            "target_agent": agent,
                            "forced": force,
                        }))?
                    );
                } else {
                    println!("Delegating task to {} agent", agent);
                    println!("   Task: {}", task.description);
                    println!("   Priority: {:?}", task.priority);
                    println!("   Type: {:?}", task.task_type);
                    if *force {
                        println!("   Forced delegation");
                    }
                }
            }

            DelegateAction::Analyze {
                description,
                verbose: _,
                strategy: _,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "removed",
                            "message": "Delegation engine has been removed. Use 'ccswarm pipeline' for workflow execution.",
                            "description": description,
                        }))?
                    );
                } else {
                    println!("Delegation engine has been removed.");
                    println!("Use 'ccswarm pipeline' for workflow execution.");
                }
            }

            DelegateAction::Stats { period, .. } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "removed",
                            "message": "Delegation statistics have been removed.",
                            "period_hours": period,
                        }))?
                    );
                } else {
                    println!("Delegation statistics have been removed.");
                    println!("Use 'ccswarm status' to view orchestrator status.");
                }
            }

            DelegateAction::Interactive => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "error": "Interactive mode not available in JSON output",
                        }))?
                    );
                } else {
                    println!("Interactive Delegation Mode");
                    self.run_interactive_delegation().await?;
                }
            }

            DelegateAction::Show { file } => {
                let config = CcswarmConfig::from_file(file.clone()).await?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                } else {
                    println!("Delegation Configuration: {}", file.display());
                    println!("========================");
                    println!("Project: {}", config.project.name);
                    println!("Repository: {}", config.project.repository.url);
                    println!("Agents: {}", config.agents.len());
                    for (name, agent_config) in &config.agents {
                        println!("  - {}: {}", name, agent_config.specialization);
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn run_interactive_delegation(&self) -> Result<()> {
        use std::io::{self, Write};

        println!("Welcome to Interactive Delegation Mode");
        println!("   Type 'help' for commands, 'quit' to exit");
        println!();

        loop {
            print!("ccswarm> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match input {
                "quit" | "exit" => {
                    println!("Goodbye!");
                    break;
                }
                "help" => {
                    println!("Interactive Delegation Commands:");
                    println!("   delegate <agent> <task>     - Delegate task to specific agent");
                    println!("   agents                      - List available agents");
                    println!("   quit                        - Exit interactive mode");
                    println!();
                }
                "agents" => {
                    println!("Available Agents:");
                    println!("   Frontend - React/TypeScript UI development");
                    println!("   Backend - Node.js/Express API development");
                    println!("   DevOps - Infrastructure and deployment");
                    println!("   QA - Testing and quality assurance");
                    println!();
                }
                _ if input.starts_with("delegate ") => {
                    let parts: Vec<&str> = input[9..].splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let agent = parts[0];
                        let task_desc = parts[1];

                        if ["frontend", "backend", "devops", "qa"].contains(&agent) {
                            println!("Delegating '{}' to {} agent", task_desc, agent);
                            println!("   Task queued for delegation");
                            println!();
                        } else {
                            println!("Unknown agent: {}", agent);
                            println!("   Available: frontend, backend, devops, qa");
                            println!();
                        }
                    } else {
                        println!("Usage: delegate <agent> <task_description>");
                        println!("   Example: delegate frontend Create responsive navigation bar");
                        println!();
                    }
                }
                "" => {
                    // Empty input, continue
                }
                _ => {
                    println!("Unknown command: {}", input);
                    println!("   Type 'help' for available commands");
                    println!();
                }
            }
        }

        Ok(())
    }
}
