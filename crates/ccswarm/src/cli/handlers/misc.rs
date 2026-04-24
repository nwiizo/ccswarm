use super::super::*;

impl CliRunner {
    pub(crate) async fn show_logs(
        &self,
        follow: bool,
        agent: Option<&str>,
        lines: usize,
    ) -> Result<()> {
        use std::fs;
        use std::io::{BufRead, BufReader};

        let logs_dir = self.repo_path.join(".ccswarm/logs");

        // Check if logs directory exists
        if !logs_dir.exists() {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "success",
                        "message": "No logs directory found",
                        "lines": 0,
                    }))?
                );
            } else {
                println!("📝 Logs");
                println!("======");
                println!("No logs directory found at {}", logs_dir.display());
                println!("Run 'ccswarm pipeline --task \"...\"' to create logs");
            }
            return Ok(());
        }

        // Read log files
        let mut log_entries = Vec::new();

        for entry in fs::read_dir(&logs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Filter by agent if specified
            if let Some(agent_filter) = agent
                && let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && !filename.contains(agent_filter)
            {
                continue;
            }

            // Read log file
            let file = fs::File::open(&path)?;
            let reader = BufReader::new(file);

            for line_content in reader.lines().map_while(Result::ok) {
                log_entries.push((
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    line_content,
                ));
            }
        }

        // Get last N lines
        let start_idx = if log_entries.len() > lines {
            log_entries.len() - lines
        } else {
            0
        };
        let displayed_logs: Vec<_> = log_entries[start_idx..].to_vec();

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Logs displayed",
                    "lines": displayed_logs.len(),
                    "total_lines": log_entries.len(),
                    "logs": displayed_logs.iter().map(|(file, content)| {
                        serde_json::json!({
                            "file": file,
                            "content": content,
                        })
                    }).collect::<Vec<_>>(),
                }))?
            );
        } else {
            println!("📝 Logs");
            println!("======");
            if let Some(agent_filter) = agent {
                println!("Agent filter: {}", agent_filter);
            }
            println!("Showing last {} lines", lines);
            println!();

            for (file, content) in displayed_logs {
                println!("[{}] {}", file.bright_blue(), content);
            }

            if log_entries.is_empty() {
                println!("No logs available yet");
            }

            if follow {
                println!();
                println!("⚠️  Follow mode requires the TUI");
                println!("   Run: ccswarm tui --follow-logs");
            }
        }

        Ok(())
    }

    pub(crate) async fn list_agents(&self, _all: bool) -> Result<()> {
        if self.json_output {
            println!("{}", serde_json::to_string_pretty(&self.config.agents)?);
        } else {
            println!("🤖 Configured Agents");
            println!("==================");

            for (name, config) in &self.config.agents {
                println!("Agent: {}", name);
                println!("  Specialization: {}", config.specialization);
                println!("  Worktree: {}", config.worktree);
                println!("  Branch: {}", config.branch);
                println!("  Model: {}", config.claude_config.model);
                println!("  Output Format: {:?}", config.claude_config.output_format);
                println!();
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_worktree(&self, action: &WorktreeAction) -> Result<()> {
        let manager = crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone())?;

        match action {
            WorktreeAction::List => {
                let worktrees = manager.list_worktrees().await?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&worktrees)?);
                } else {
                    println!("🌳 Git Worktrees");
                    println!("===============");

                    for wt in &worktrees {
                        println!("Path: {}", wt.path.display());
                        println!("  Branch: {}", wt.branch);
                        println!("  Head: {}", wt.head_commit);
                        println!("  Locked: {}", wt.is_locked);
                        println!();
                    }
                }
            }
            WorktreeAction::Create {
                path,
                branch,
                new_branch,
            } => {
                let info = if *new_branch {
                    manager.create_worktree_full(path, branch, true).await?
                } else {
                    manager.create_worktree(path, branch).await?
                };

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&info)?);
                } else {
                    println!("✅ Worktree created");
                    println!("   Path: {}", info.path.display());
                    println!("   Branch: {}", info.branch);
                }
            }
            WorktreeAction::Remove { path, force } => {
                if *force {
                    manager.remove_worktree_full(path, true).await?
                } else {
                    manager.remove_worktree(path).await?
                };

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Worktree removed",
                            "path": path,
                        }))?
                    );
                } else {
                    println!("✅ Worktree removed: {}", path.display());
                }
            }
            WorktreeAction::Prune => {
                manager.prune_worktrees().await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Worktrees pruned",
                        }))?
                    );
                } else {
                    println!("✅ Stale worktrees pruned");
                }
            }
            WorktreeAction::Clean { force } => {
                use std::io::{self, Write};

                // Find all ccswarm-related worktrees
                let worktrees = manager.list_worktrees().await?;
                let ccswarm_worktrees: Vec<_> = worktrees
                    .iter()
                    .filter(|w| w.branch.contains("agent") || w.branch.contains("feature/"))
                    .collect();

                if ccswarm_worktrees.is_empty() {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "No ccswarm worktrees found",
                            }))?
                        );
                    } else {
                        println!("✅ No ccswarm worktrees to clean");
                    }
                    return Ok(());
                }

                // Ask for confirmation unless forced
                if !force {
                    println!("⚠️  Found {} ccswarm worktrees:", ccswarm_worktrees.len());
                    for w in &ccswarm_worktrees {
                        println!("   - {} ({})", w.path.display(), w.branch);
                    }
                    print!("\nAre you sure you want to remove all these worktrees? [y/N] ");
                    io::stdout().flush()?;

                    let mut response = String::new();
                    io::stdin().read_line(&mut response)?;

                    if !response.trim().eq_ignore_ascii_case("y") {
                        println!("❌ Cleanup cancelled");
                        return Ok(());
                    }
                }

                // Remove all ccswarm worktrees
                let mut removed_count = 0;
                for worktree in ccswarm_worktrees {
                    match manager.remove_worktree(&worktree.path).await {
                        Ok(_) => {
                            removed_count += 1;
                            if !self.json_output {
                                println!("   ✓ Removed {}", worktree.path.display());
                            }
                        }
                        Err(e) => {
                            if !self.json_output {
                                println!(
                                    "   ✗ Failed to remove {}: {}",
                                    worktree.path.display(),
                                    e
                                );
                            }
                        }
                    }
                }

                // Also clean up branches
                let output = tokio::process::Command::new("git")
                    .args(["branch", "--list", "*agent*", "feature/*"])
                    .output()
                    .await?;

                if output.status.success() {
                    let branches = String::from_utf8_lossy(&output.stdout);
                    let branch_count = branches.lines().count();

                    if branch_count > 0 {
                        tokio::process::Command::new("git")
                            .args(&["branch", "-D"])
                            .args(branches.lines().map(|b| b.trim().trim_start_matches("* ")))
                            .output()
                            .await?;
                    }
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Cleanup completed",
                            "worktrees_removed": removed_count,
                        }))?
                    );
                } else {
                    println!(
                        "\n✅ Cleanup completed: {} worktrees removed",
                        removed_count
                    );
                }
            }
        }

        Ok(())
    }
}
