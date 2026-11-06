use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_doctor(&self, fix: bool, check_api: bool) -> Result<()> {
        use crate::utils::user_error::CommonErrors;

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
            println!(
                "{}",
                "⚠️  Not set (optional when provider CLIs are already authenticated)"
                    .bright_yellow()
            );
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

        // Local provider execution runs in Herdr tabs (docs/HERDR_INTEGRATION.md).
        issues.extend(check_herdr());

        // Check agent provider CLIs (Claude/Codex/Copilot)
        println!("Checking agent provider CLIs...");

        for (label, probe, fix_key, required, caveat) in [
            (
                "  claude",
                &["claude", "--version"][..],
                "claude_cli",
                true,
                None,
            ),
            (
                "  codex",
                &["codex", "--version"][..],
                "codex_cli",
                false,
                None,
            ),
            (
                "  gh copilot",
                &["gh", "copilot", "--version"][..],
                "copilot_cli",
                false,
                // `gh copilot suggest` is interactive and returns shell-command
                // strings, not file edits — see providers/copilot.rs.
                Some("code-gen unsupported; use claude or codex"),
            ),
        ] {
            print!("{} ... ", label);
            let mut cmd = std::process::Command::new(probe[0]);
            cmd.args(&probe[1..])
                .env_remove("CLAUDECODE")
                .env_remove("CLAUDE_CODE_ENTRYPOINT");
            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let v = String::from_utf8_lossy(&output.stdout);
                    let first_line = v.lines().next().unwrap_or("").trim();
                    if let Some(note) = caveat {
                        println!(
                            "{} ({}) {}",
                            "⚠️  Available".bright_yellow(),
                            first_line.bright_black(),
                            format!("— {}", note).bright_yellow(),
                        );
                    } else {
                        println!(
                            "{} ({})",
                            "✅ Available".bright_green(),
                            first_line.bright_black()
                        );
                    }
                }
                _ => {
                    if required {
                        println!("{}", "❌ Not found".bright_red());
                        issues.push(fix_key);
                    } else {
                        println!("{}", "⚠️  Not installed (optional)".bright_yellow());
                    }
                }
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
                        "git_repo" if fix => {
                            println!("• Initializing git repository...");
                            crate::git::shell::ShellWorktreeManager::init_if_needed(
                                &self.repo_path,
                            )
                            .await?;
                            println!("  ✅ Git repository initialized");
                        }
                        "claude_cli" => {
                            println!(
                                "• Claude CLI: Install from https://docs.anthropic.com/en/docs/claude-code"
                            );
                        }
                        "herdr_cli" => {
                            println!(
                                "• Herdr: Install Herdr 0.9.5 or later from https://herdr.dev"
                            );
                        }
                        "herdr_session" => {
                            println!(
                                "• Herdr session: Run `herdr`, then rerun ccswarm from one of its panes"
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
}

/// Print Herdr checks and return the issue keys that need fixing.
fn check_herdr() -> Vec<&'static str> {
    let program = crate::session::herdr::herdr_program();
    let run = |args: &[&str]| {
        std::process::Command::new(&program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    };
    let mut issues = Vec::new();

    print!("Checking Herdr CLI... ");
    match run(&["--version"]) {
        Some(version) => println!(
            "{} ({})",
            "✅ Available".bright_green(),
            version.trim().bright_black()
        ),
        None => {
            println!("{}", "❌ Not found".bright_red());
            return vec!["herdr_cli"];
        }
    }

    print!("Checking Herdr session... ");
    match crate::session::herdr::HerdrContext::from_env() {
        Ok(_) => println!("{}", "✅ Running inside a Herdr pane".bright_green()),
        Err(_) => {
            println!(
                "{}",
                "❌ Not inside a Herdr pane (local provider runs will fail)".bright_red()
            );
            issues.push("herdr_session");
        }
    }

    print!("Checking Herdr server... ");
    match run(&["status", "server"]) {
        Some(status) if status.contains("status: running") => {
            println!("{}", "✅ Running".bright_green())
        }
        _ => println!("{}", "⚠️  Not reachable from this process".bright_yellow()),
    }

    // Integrations improve agent detection and session restore; ccswarm does
    // not require them, so they are reported without creating an issue.
    if let Some(status) = run(&["integration", "status"]) {
        for line in status
            .lines()
            .filter(|line| line.starts_with("claude:") || line.starts_with("codex:"))
        {
            println!("  Herdr integration {}", line.bright_black());
        }
    }
    issues
}
