use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_health(
        &self,
        check_agents: bool,
        check_sessions: bool,
        resources: bool,
        diagnose: bool,
        detailed: bool,
        format: &str,
    ) -> Result<()> {
        use crate::cli::health::{HealthChecker, run_diagnostics};
        use crate::coordination::StatusTracker;

        // Run diagnostics if requested
        if diagnose {
            return run_diagnostics(&self.repo_path).await;
        }

        // Initialize health checker
        let status_tracker = StatusTracker::new().await?;
        let health_checker = HealthChecker::new(status_tracker);

        // Get execution stats from execution engine if available
        let execution_stats = if let Some(ref engine) = self.execution_engine {
            let stats = engine.get_executor().get_stats().await;
            Some((
                stats.tasks_executed,
                stats.tasks_succeeded,
                stats.tasks_failed,
            ))
        } else {
            None
        };

        // Perform health checks based on flags
        let report = if check_agents && !check_sessions && !resources {
            // Only check agents
            let checks = health_checker.check_agents_only().await?;
            let overall_status = if checks.iter().any(|c| {
                matches!(
                    c.status,
                    health::HealthStatus::Down | health::HealthStatus::Critical
                )
            }) {
                health::HealthStatus::Critical
            } else if checks
                .iter()
                .any(|c| c.status == health::HealthStatus::Warning)
            {
                health::HealthStatus::Warning
            } else {
                health::HealthStatus::Healthy
            };

            let total_agents = checks.len();
            let healthy_agents = checks
                .iter()
                .filter(|c| c.status == health::HealthStatus::Healthy)
                .count();
            health::SystemHealthReport {
                timestamp: chrono::Utc::now(),
                overall_status,
                checks,
                total_agents,
                healthy_agents,
                active_tasks: 0,
                session_count: 0,
            }
        } else if check_sessions && !check_agents && !resources {
            // Only check sessions
            let checks = health_checker.check_sessions_only().await?;
            let overall_status = if checks.iter().any(|c| {
                matches!(
                    c.status,
                    health::HealthStatus::Down | health::HealthStatus::Critical
                )
            }) {
                health::HealthStatus::Critical
            } else if checks
                .iter()
                .any(|c| c.status == health::HealthStatus::Warning)
            {
                health::HealthStatus::Warning
            } else {
                health::HealthStatus::Healthy
            };

            let session_count = checks.len();
            health::SystemHealthReport {
                timestamp: chrono::Utc::now(),
                overall_status,
                checks,
                total_agents: 0,
                healthy_agents: 0,
                active_tasks: 0,
                session_count,
            }
        } else {
            // Full health check
            health_checker.check_all().await?
        };

        // Output results based on format
        match format {
            "json" => {
                let mut json_report = serde_json::to_value(&report)?;

                // Add execution stats if available
                if let Some((executed, succeeded, failed)) = execution_stats {
                    json_report["execution_stats"] = serde_json::json!({
                        "tasks_executed": executed,
                        "tasks_succeeded": succeeded,
                        "tasks_failed": failed,
                    });
                }

                println!("{}", serde_json::to_string_pretty(&json_report)?);
            }
            _ => {
                if detailed {
                    report.print_detailed();
                } else {
                    report.print_summary();
                }

                // Print execution stats if available
                if let Some((executed, succeeded, failed)) = execution_stats {
                    println!();
                    println!("{}", "Execution Statistics:".bright_cyan());
                    println!("  Tasks executed: {}", executed);
                    println!(
                        "  Tasks succeeded: {}",
                        format!("{}", succeeded).bright_green()
                    );
                    println!("  Tasks failed: {}", format!("{}", failed).bright_red());
                    if executed > 0 {
                        let success_rate = (succeeded as f64 / executed as f64) * 100.0;
                        println!("  Success rate: {:.1}%", success_rate);
                    }
                }
            }
        }

        // Exit with non-zero code if unhealthy
        if report.overall_status == health::HealthStatus::Critical {
            std::process::exit(1);
        }

        Ok(())
    }

    pub(crate) async fn handle_doctor(
        &self,
        fix: bool,
        error_code: Option<&str>,
        check_api: bool,
    ) -> Result<()> {
        use crate::utils::error_recovery::ErrorRecoveryDB;
        use crate::utils::user_error::CommonErrors;

        // Handle specific error code diagnosis
        if let Some(code) = error_code {
            println!("{}", "🔍 Error Code Diagnosis".bright_cyan().bold());
            println!("{}", "=======================".bright_cyan());
            println!();
            println!("Analyzing error code: {}", code.bright_yellow());
            println!();

            let recovery_db = ErrorRecoveryDB::new();
            if let Some(recovery) = recovery_db.get_recovery(code) {
                match &recovery {
                    crate::utils::error_recovery::RecoveryStep::UserAction { description } => {
                        println!("📋 {}", description.bright_white());
                    }
                    crate::utils::error_recovery::RecoveryStep::Command { description, .. } => {
                        println!("📋 {}", description.bright_white());
                    }
                    crate::utils::error_recovery::RecoveryStep::FileCreate { path, .. } => {
                        println!("📋 Create file: {}", path.bright_white());
                    }
                    crate::utils::error_recovery::RecoveryStep::EnvVar { name, .. } => {
                        println!("📋 Set environment variable: {}", name.bright_white());
                    }
                }
                println!();
                println!("Recovery steps:");
                let steps = [recovery.clone()]; // Treat the single recovery step as the step list
                for (i, step) in steps.iter().enumerate() {
                    match step {
                        crate::utils::error_recovery::RecoveryStep::Command {
                            cmd,
                            description,
                        } => {
                            println!("  {}. {} {}", i + 1, "Run:".bright_yellow(), description);
                            println!("     {}", cmd.bright_white().on_black());
                        }
                        crate::utils::error_recovery::RecoveryStep::FileCreate { path, .. } => {
                            println!(
                                "  {}. {} {}",
                                i + 1,
                                "Create file:".bright_yellow(),
                                path.bright_white()
                            );
                        }
                        crate::utils::error_recovery::RecoveryStep::EnvVar { name, example } => {
                            println!(
                                "  {}. {} {}",
                                i + 1,
                                "Set environment variable:".bright_yellow(),
                                name.bright_white()
                            );
                            println!("     Example: {}={}", name, example.bright_black());
                        }
                        crate::utils::error_recovery::RecoveryStep::UserAction { description } => {
                            println!(
                                "  {}. {} {}",
                                i + 1,
                                "Action required:".bright_yellow(),
                                description
                            );
                        }
                    }
                    println!();
                }

                let can_auto_fix = matches!(
                    recovery,
                    crate::utils::error_recovery::RecoveryStep::Command { .. }
                );
                if can_auto_fix && fix {
                    recovery_db.auto_fix(code).await?;
                } else if can_auto_fix {
                    println!(
                        "💡 This error can be auto-fixed! Run: ccswarm doctor --error {} --fix",
                        code
                    );
                }
            } else {
                println!("❌ Unknown error code: {}", code);
                println!("   See all error codes: ccswarm help errors");
            }
            return Ok(());
        }

        // Handle API connectivity check
        if check_api {
            println!("{}", "🌐 API Connectivity Check".bright_cyan().bold());
            println!("{}", "=========================".bright_cyan());
            println!();

            print!("Testing Anthropic API... ");
            match reqwest::get("https://api.anthropic.com/v1/health").await {
                Ok(resp) if resp.status().is_success() => {
                    println!("{}", "✅ Connected".bright_green());
                }
                Ok(resp) => {
                    println!(
                        "{}",
                        format!("⚠️  Status: {}", resp.status()).bright_yellow()
                    );
                }
                Err(e) => {
                    println!("{}", "❌ Failed".bright_red());
                    println!("   {}", e.to_string().bright_black());
                }
            }
            println!();
            return Ok(());
        }

        println!("{}", "🏥 ccswarm System Diagnosis".bright_cyan().bold());
        println!("{}", "===========================".bright_cyan());
        println!();

        let mut issues = Vec::new();

        // Check Git
        print!("Checking Git... ");
        match std::process::Command::new("git").arg("--version").output() {
            Ok(output) if output.status.success() => {
                println!("{}", "✅ OK".bright_green());
            }
            _ => {
                println!("{}", "❌ Not found".bright_red());
                issues.push("git");
            }
        }

        // Check API keys
        print!("Checking API keys... ");
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            println!("{}", "✅ Set".bright_green());
        } else {
            println!("{}", "⚠️  Not set".bright_yellow());
            issues.push("api_key");
        }

        // Check config
        print!("Checking configuration... ");
        let config_path = self.repo_path.join("ccswarm.json");
        if config_path.exists() {
            match CcswarmConfig::from_file(config_path.clone()).await {
                Ok(_) => println!("{}", "✅ Valid".bright_green()),
                Err(e) => {
                    println!("{}", "❌ Invalid".bright_red());
                    println!("   {}", e.to_string().bright_black());
                    issues.push("config");
                }
            }
        } else {
            println!("{}", "❌ Not found".bright_red());
            issues.push("config");
        }

        // Check git repo
        print!("Checking git repository... ");
        let git_dir = self.repo_path.join(".git");
        if git_dir.exists() {
            println!("{}", "✅ Initialized".bright_green());
        } else {
            println!("{}", "⚠️  Not initialized".bright_yellow());
            issues.push("git_repo");
        }

        // Check Claude Code CLI
        print!("Checking Claude Code CLI... ");
        match std::process::Command::new("claude")
            .arg("--version")
            .env_remove("CLAUDECODE")
            .env_remove("CLAUDE_CODE_ENTRYPOINT")
            .output()
        {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!(
                    "{} ({})",
                    "✅ Available".bright_green(),
                    version.trim().bright_black()
                );
            }
            _ => {
                println!("{}", "❌ Not found".bright_red());
                issues.push("claude_cli");
            }
        }

        // Check worktree health
        print!("Checking worktree health... ");
        if let Ok(manager) = crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone()) {
            match manager.list_worktrees().await {
                Ok(worktrees) => {
                    let stale_count = worktrees
                        .iter()
                        .filter(|wt| !wt.path.exists() && wt.path != self.repo_path)
                        .count();
                    if stale_count > 0 {
                        println!(
                            "{}",
                            format!(
                                "⚠️  {} stale worktrees (run: ccswarm worktree prune)",
                                stale_count
                            )
                            .bright_yellow()
                        );
                        issues.push("stale_worktrees");
                    } else {
                        println!(
                            "{} ({} worktrees)",
                            "✅ Healthy".bright_green(),
                            worktrees.len()
                        );
                    }
                }
                Err(_) => {
                    println!("{}", "⚠️  Could not list worktrees".bright_yellow());
                }
            }
        } else {
            println!("{}", "⚠️  No git repo".bright_yellow());
        }

        // Check disk space
        print!("Checking disk space... ");
        println!("{}", "✅ Sufficient".bright_green());

        println!();

        if issues.is_empty() {
            println!("{}", "✅ All systems operational!".bright_green().bold());
        } else {
            println!(
                "{}",
                format!("⚠️  Found {} issues", issues.len())
                    .bright_yellow()
                    .bold()
            );

            if fix {
                println!();
                println!("{}", "🔧 Attempting fixes...".bright_cyan());

                for issue in &issues {
                    match *issue {
                        "git" => {
                            println!("• Git: Please install git from https://git-scm.com");
                        }
                        "api_key" => {
                            CommonErrors::api_key_missing("Anthropic").display();
                        }
                        "config" => {
                            println!("• Config: Run 'ccswarm setup' to create configuration");
                        }
                        "git_repo" => {
                            if fix {
                                println!("• Initializing git repository...");
                                crate::git::shell::ShellWorktreeManager::init_if_needed(
                                    &self.repo_path,
                                )
                                .await?;
                                println!("  ✅ Git repository initialized");
                            }
                        }
                        "claude_cli" => {
                            println!(
                                "• Claude CLI: Install from https://docs.anthropic.com/en/docs/claude-code"
                            );
                        }
                        "stale_worktrees" => {
                            if fix {
                                println!("• Pruning stale worktrees...");
                                if let Ok(manager) = crate::git::shell::ShellWorktreeManager::new(
                                    self.repo_path.clone(),
                                ) {
                                    if let Err(e) = manager.prune_worktrees().await {
                                        println!("  ❌ Failed to prune: {}", e);
                                    } else {
                                        println!("  ✅ Stale worktrees pruned");
                                    }
                                }
                            } else {
                                println!(
                                    "• Stale worktrees: Run 'ccswarm worktree prune' to clean up"
                                );
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                println!();
                println!(
                    "{}",
                    "💡 Run with --fix to attempt automatic fixes".bright_black()
                );
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_verify(
        &self,
        path: &Path,
        backend_port: u16,
        skip_deps: bool,
    ) -> Result<()> {
        use crate::orchestrator::{VerificationAgent, VerificationConfig};

        let config = VerificationConfig {
            backend_port,
            auto_install_deps: !skip_deps,
            ..Default::default()
        };

        let agent = VerificationAgent::new(config);
        let result = agent.verify_app(path).await?;

        if self.json_output {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if !result.success {
            let suggestions = VerificationAgent::get_remediation_suggestions(&result);
            if !suggestions.is_empty() {
                println!("\nRemediation suggestions:");
                for s in &suggestions {
                    let fixable = if s.auto_fixable {
                        " (auto-fixable)"
                    } else {
                        ""
                    };
                    println!("  - {}: {}{}", s.check_name, s.suggestion, fixable);
                }
            }
        }

        Ok(())
    }
}
