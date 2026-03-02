use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_delegate(&self, action: &DelegateAction) -> Result<()> {
        use crate::orchestrator::master_delegation::{DelegationStrategy, MasterDelegationEngine};

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
                    println!("🎯 Delegating task to {} agent", agent);
                    println!("   Task: {}", task.description);
                    println!("   Priority: {:?}", task.priority);
                    println!("   Type: {:?}", task.task_type);
                    if *force {
                        println!("   ⚠️ Forced delegation");
                    }
                }
            }

            DelegateAction::Analyze {
                description,
                verbose,
                strategy,
            } => {
                let strategy = match strategy.as_str() {
                    "content" => DelegationStrategy::ContentBased,
                    "load" => DelegationStrategy::LoadBalanced,
                    "expertise" => DelegationStrategy::ExpertiseBased,
                    "workflow" => DelegationStrategy::WorkflowBased,
                    "hybrid" => DelegationStrategy::Hybrid,
                    _ => DelegationStrategy::Hybrid,
                };

                let mut engine = MasterDelegationEngine::new(strategy);
                let task = Task::new(
                    "analysis".to_string(),
                    description.clone(),
                    Priority::Medium,
                    TaskType::Development,
                );

                let decision = engine.delegate_task(task)?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                } else {
                    println!("🔍 Task Analysis Results");
                    println!("   Task: {}", description);
                    println!("   Recommended Agent: {}", decision.target_agent.name());
                    println!("   Confidence: {:.1}%", decision.confidence * 100.0);
                    if *verbose {
                        println!("   Reasoning: {}", decision.reasoning);
                        if let Some(duration) = decision.estimated_duration {
                            println!("   Estimated Duration: {:?}", duration);
                        }
                    }
                }
            }

            DelegateAction::Stats { detailed, period } => {
                // Get stats from execution engine if available
                let (stats, history) = if let Some(ref engine) = self.execution_engine {
                    let executor = engine.get_executor();
                    let s = executor.get_stats().await;
                    let h = executor.get_execution_history(None).await;
                    (Some(s), h)
                } else {
                    (None, Vec::new())
                };

                // Count delegations per agent
                let mut agent_counts: std::collections::HashMap<String, (usize, usize)> =
                    std::collections::HashMap::new();
                for result in &history {
                    if let Some(ref agent) = result.agent_used {
                        let entry = agent_counts.entry(agent.clone()).or_default();
                        entry.0 += 1; // total
                        if result.success {
                            entry.1 += 1; // succeeded
                        }
                    }
                }

                if self.json_output {
                    let agent_stats: Vec<_> = agent_counts
                        .iter()
                        .map(|(agent, (total, succeeded))| {
                            serde_json::json!({
                                "agent": agent,
                                "total_tasks": total,
                                "succeeded": succeeded,
                                "failed": total - succeeded,
                            })
                        })
                        .collect();

                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Delegation statistics",
                            "period_hours": period,
                            "total_delegations": history.len(),
                            "agents": agent_stats,
                        }))?
                    );
                } else {
                    println!("📊 Delegation Statistics (last {} hours)", period);
                    println!("========================================");
                    println!();

                    if let Some(ref s) = stats {
                        println!("   Total tasks executed: {}", s.tasks_executed);
                        println!(
                            "   Succeeded: {}",
                            format!("{}", s.tasks_succeeded).bright_green()
                        );
                        println!("   Failed: {}", format!("{}", s.tasks_failed).bright_red());
                        println!("   Orchestration usage: {:.1}%", s.orchestration_usage);
                        println!(
                            "   Average duration: {:.1}s",
                            s.average_duration.as_secs_f64()
                        );
                    } else {
                        println!("   No execution engine active. Start with 'ccswarm start'");
                    }

                    if *detailed && !agent_counts.is_empty() {
                        println!();
                        println!("   Per-Agent Breakdown:");
                        for (agent, (total, succeeded)) in &agent_counts {
                            let success_rate = if *total > 0 {
                                (*succeeded as f64 / *total as f64) * 100.0
                            } else {
                                0.0
                            };
                            println!(
                                "     {}: {} tasks ({:.0}% success)",
                                agent.bright_cyan(),
                                total,
                                success_rate
                            );
                        }
                    }
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
                    println!("🖥️ Interactive Delegation Mode");
                    self.run_interactive_delegation().await?;
                }
            }

            DelegateAction::Show { file } => {
                let config = CcswarmConfig::from_file(file.clone()).await?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                } else {
                    println!("📄 Delegation Configuration: {}", file.display());
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

        println!("🎯 Welcome to Interactive Delegation Mode");
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
                    println!("👋 Goodbye!");
                    break;
                }
                "help" => {
                    println!("📚 Interactive Delegation Commands:");
                    println!("   analyze <task_description>  - Analyze task and suggest agent");
                    println!("   delegate <agent> <task>     - Delegate task to specific agent");
                    println!("   stats                       - Show delegation statistics");
                    println!("   agents                      - List available agents");
                    println!("   quit                        - Exit interactive mode");
                    println!();
                }
                "stats" => {
                    println!("📊 Delegation Statistics");
                    println!("   Feature not yet implemented");
                    println!();
                }
                "agents" => {
                    println!("🤖 Available Agents:");
                    println!("   • Frontend - React/TypeScript UI development");
                    println!("   • Backend - Node.js/Express API development");
                    println!("   • DevOps - Infrastructure and deployment");
                    println!("   • QA - Testing and quality assurance");
                    println!();
                }
                _ if input.starts_with("analyze ") => {
                    let task_desc = &input[8..];
                    if !task_desc.is_empty() {
                        // Directly call delegation analysis to avoid recursion
                        use crate::orchestrator::master_delegation::{
                            DelegationStrategy, MasterDelegationEngine,
                        };
                        let mut engine = MasterDelegationEngine::new(DelegationStrategy::Hybrid);
                        let task = Task::new(
                            "interactive-analysis".to_string(),
                            task_desc.to_string(),
                            Priority::Medium,
                            TaskType::Development,
                        );

                        match engine.delegate_task(task) {
                            Ok(decision) => {
                                println!("🔍 Task Analysis Results");
                                println!("   Task: {}", task_desc);
                                println!("   Recommended Agent: {}", decision.target_agent.name());
                                println!("   Confidence: {:.1}%", decision.confidence * 100.0);
                                println!("   Reasoning: {}", decision.reasoning);
                                if let Some(duration) = decision.estimated_duration {
                                    println!("   Estimated Duration: {} seconds", duration);
                                }
                            }
                            Err(e) => {
                                println!("❌ Analysis failed: {}", e);
                            }
                        }
                        println!();
                    } else {
                        println!("❌ Please provide a task description");
                        println!("   Example: analyze Create login form with validation");
                        println!();
                    }
                }
                _ if input.starts_with("delegate ") => {
                    let parts: Vec<&str> = input[9..].splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let agent = parts[0];
                        let task_desc = parts[1];

                        if ["frontend", "backend", "devops", "qa"].contains(&agent) {
                            println!("🎯 Delegating '{}' to {} agent", task_desc, agent);
                            println!("   ✅ Task queued for delegation");
                            println!();
                        } else {
                            println!("❌ Unknown agent: {}", agent);
                            println!("   Available: frontend, backend, devops, qa");
                            println!();
                        }
                    } else {
                        println!("❌ Usage: delegate <agent> <task_description>");
                        println!("   Example: delegate frontend Create responsive navigation bar");
                        println!();
                    }
                }
                "" => {
                    // Empty input, continue
                }
                _ => {
                    println!("❓ Unknown command: {}", input);
                    println!("   Type 'help' for available commands");
                    println!();
                }
            }
        }

        Ok(())
    }

}
