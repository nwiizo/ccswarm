use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_task(&self, action: &TaskAction) -> Result<()> {
        match action {
            TaskAction::Add {
                description,
                priority,
                task_type,
                details,
                duration,
                auto_assign: _,
                template: _,
                template_vars: _,
                interactive: _,
            } => {
                self.add_task(
                    description,
                    priority,
                    task_type,
                    details.as_deref(),
                    *duration,
                )
                .await
            }
            TaskAction::List {
                all,
                status,
                agent,
                detailed,
                branches,
            } => {
                self.list_tasks(
                    *all,
                    status.as_deref(),
                    agent.as_deref(),
                    *detailed,
                    *branches,
                )
                .await
            }
            TaskAction::Status {
                task_id,
                history,
                orchestration,
            } => {
                self.show_task_status(task_id, *history, *orchestration)
                    .await
            }
            TaskAction::Cancel {
                task_id,
                force,
                reason,
            } => self.cancel_task(task_id, *force, reason.as_deref()).await,
            TaskAction::History {
                limit,
                agent,
                failed_only,
            } => {
                self.show_task_history(*limit, agent.as_deref(), *failed_only)
                    .await
            }
            TaskAction::Execute {
                task,
                agent,
                orchestrate,
            } => {
                self.execute_task_immediate(task, agent.as_deref(), *orchestrate)
                    .await
            }
            TaskAction::Stats {
                detailed,
                performance,
            } => self.show_task_stats(*detailed, *performance).await,
            TaskAction::Merge {
                task_id,
                cleanup,
                yes,
            } => self.merge_task_branch(task_id, *cleanup, *yes).await,
            TaskAction::Retry { task_id, force } => self.retry_task(task_id, *force).await,
            TaskAction::Delete { task_id, force } => self.delete_task(task_id, *force).await,
        }
    }

    pub(crate) async fn add_task(
        &self,
        description: &str,
        priority: &str,
        task_type: &str,
        details: Option<&str>,
        duration: Option<u32>,
    ) -> Result<()> {
        use crate::utils::user_error::CommonErrors;

        if description.trim().is_empty() {
            CommonErrors::invalid_task_format()
                .with_details("Task description cannot be empty")
                .with_suggestion("Provide a clear, actionable task description")
                .display();
            return Err(anyhow!("Invalid task description"));
        }

        println!(
            "Creating task: {}...",
            description.chars().take(50).collect::<String>()
        );

        let priority = match priority.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => Priority::Medium,
        };

        let task_type = match task_type.to_lowercase().as_str() {
            "development" | "dev" => TaskType::Development,
            "testing" | "test" => TaskType::Testing,
            "documentation" | "docs" => TaskType::Documentation,
            "infrastructure" | "infra" => TaskType::Infrastructure,
            "coordination" => TaskType::Coordination,
            "review" => TaskType::Review,
            "bugfix" | "bug" => TaskType::Bugfix,
            "feature" => TaskType::Feature,
            _ => TaskType::Development,
        };

        let task_type_clone = task_type;
        let mut task = Task::new(
            uuid::Uuid::new_v4().to_string(),
            description.to_string(),
            priority,
            task_type,
        );

        if let Some(details) = details {
            task = task.with_details(details.to_string());
        }

        if let Some(duration) = duration {
            task = task.with_duration(duration);
        }

        let task_id = task.id.clone();

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Task added",
                    "task_id": task_id,
                    "description": description,
                    "priority": priority,
                }))?
            );
        } else {
            println!();
            println!("{}", "✅ Task created successfully!".bright_green().bold());
            println!();
            println!(
                "   {} {}",
                "Task ID:".bright_cyan(),
                task_id[..8].bright_white()
            );
            println!("   {} {}", "Description:".bright_cyan(), description);
            println!(
                "   {} {}",
                "Priority:".bright_cyan(),
                match priority {
                    Priority::Critical => "🔴 Critical".bright_red(),
                    Priority::High => "🟡 High".bright_yellow(),
                    Priority::Medium => "🟢 Medium".bright_green(),
                    Priority::Low => "🔵 Low".bright_blue(),
                }
            );
            println!(
                "   {} {}",
                "Type:".bright_cyan(),
                format!("{:?}", task_type_clone).bright_white()
            );

            if let Some(duration) = task.estimated_duration {
                println!("   {} {} minutes", "Est. Duration:".bright_cyan(), duration);
            }

            show_quick_help("task-created");
        }

        Ok(())
    }

    /// Execute a task immediately
    pub(crate) async fn execute_task_immediate(
        &self,
        task: &str,
        agent: Option<&str>,
        orchestrate: bool,
    ) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "task": task,
                    "agent": agent,
                    "orchestrate": orchestrate,
                    "status": "not_implemented"
                }))?
            );
        } else {
            println!("Task execution engine has been removed.");
            println!("Use 'ccswarm pipeline' for workflow execution.");
        }
        Ok(())
    }

    pub(crate) async fn list_tasks(
        &self,
        _all: bool,
        _status_filter: Option<&str>,
        _agent_filter: Option<&str>,
        _detailed: bool,
        branches: bool,
    ) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "tasks": [],
                    "total": 0,
                    "message": "Task queue management has been removed. Use 'ccswarm pipeline' for workflow execution."
                }))?
            );
        } else {
            println!("Task queue management has been removed.");
            println!("Use 'ccswarm pipeline' for workflow execution.");
        }

        // Show worktree summary if --branches flag is set
        if branches {
            println!("{}", "Worktree Summary".bright_cyan().bold());
            println!("{}", "================".bright_cyan());
            if let Ok(manager) =
                crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone())
            {
                match manager.list_worktrees().await {
                    Ok(worktrees) => {
                        let task_worktrees: Vec<_> = worktrees
                            .iter()
                            .filter(|wt| wt.branch.starts_with("task/"))
                            .collect();
                        if task_worktrees.is_empty() {
                            println!("No task worktrees found.");
                        } else {
                            for wt in &task_worktrees {
                                println!(
                                    "  🌿 {} ({})",
                                    wt.branch.bright_green(),
                                    wt.path.display()
                                );
                            }
                            println!();
                            println!("Total: {} task worktrees", task_worktrees.len());
                        }
                    }
                    Err(e) => {
                        println!("Failed to list worktrees: {}", e.to_string().bright_red());
                    }
                }
            }
            println!();
        }

        Ok(())
    }

    /// Merge a completed task's worktree branch into the main branch
    pub(crate) async fn merge_task_branch(
        &self,
        task_id: &str,
        cleanup: bool,
        _yes: bool,
    ) -> Result<()> {
        let manager = crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone())?;

        // Find the task's worktree branch
        let safe_id: String = task_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let branch_name = format!("task/{}", safe_id);

        println!(
            "{} Merging branch {} into main...",
            ">>>".bright_green().bold(),
            branch_name.bright_cyan()
        );

        // Check the branch exists via worktree list
        let worktrees = manager.list_worktrees().await?;
        let task_wt = worktrees.iter().find(|wt| wt.branch == branch_name);

        if task_wt.is_none() {
            return Err(anyhow!("No worktree found for branch '{}'", branch_name));
        }

        // Merge using git merge
        let output = tokio::process::Command::new("git")
            .args([
                "merge",
                &branch_name,
                "--no-ff",
                "-m",
                &format!("Merge task {} branch", task_id),
            ])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to execute git merge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Merge failed: {}", stderr));
        }

        println!("{}", "OK Merged successfully".bright_green().bold());

        // Cleanup worktree if requested
        if cleanup && let Some(wt) = task_wt {
            if let Err(e) = manager.remove_worktree(&wt.path).await {
                warn!("Failed to cleanup worktree: {}", e);
            } else {
                println!("Cleaned up worktree at {}", wt.path.display());
            }
        }

        Ok(())
    }

    /// Retry a failed task
    pub(crate) async fn retry_task(&self, _task_id: &str, _force: bool) -> Result<()> {
        println!(
            "Task queue management has been removed. Use 'ccswarm pipeline' for workflow execution."
        );
        Ok(())
    }

    /// Delete a task and its associated worktree
    pub(crate) async fn delete_task(&self, task_id: &str, _force: bool) -> Result<()> {
        // Clean up any associated worktree
        let safe_id: String = task_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();

        if let Ok(manager) = crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone()) {
            let worktrees = manager.list_worktrees().await.unwrap_or_default();
            let branch_name = format!("task/{}", safe_id);
            if let Some(wt) = worktrees.iter().find(|wt| wt.branch == branch_name) {
                if let Err(e) = manager.remove_worktree(&wt.path).await {
                    warn!("Failed to cleanup worktree: {}", e);
                } else {
                    println!("Cleaned up worktree at {}", wt.path.display());
                }
            }
        }

        println!("{} Task '{}' deleted", "OK".bright_green().bold(), task_id);

        Ok(())
    }

    /// Show detailed task status
    pub(crate) async fn show_task_status(
        &self,
        task_id: &str,
        _history: bool,
        _orchestration: bool,
    ) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "task_id": task_id,
                    "status": "not_available",
                    "message": "Task queue management has been removed."
                }))?
            );
        } else {
            println!("Task queue management has been removed.");
            println!("Use 'ccswarm pipeline' for workflow execution.");
        }
        Ok(())
    }

    /// Cancel a task
    pub(crate) async fn cancel_task(
        &self,
        _task_id: &str,
        _force: bool,
        _reason: Option<&str>,
    ) -> Result<()> {
        println!(
            "Task queue management has been removed. Use 'ccswarm pipeline' for workflow execution."
        );
        Ok(())
    }

    /// Show task execution history
    pub(crate) async fn show_task_history(
        &self,
        _limit: usize,
        _agent_filter: Option<&str>,
        _failed_only: bool,
    ) -> Result<()> {
        println!(
            "Task queue management has been removed. Use 'ccswarm pipeline' for workflow execution."
        );
        Ok(())
    }

    /// Show task queue statistics
    pub(crate) async fn show_task_stats(&self, _detailed: bool, _performance: bool) -> Result<()> {
        println!(
            "Task queue management has been removed. Use 'ccswarm pipeline' for workflow execution."
        );
        Ok(())
    }

    pub(crate) fn create_task_from_args(
        &self,
        description: &str,
        priority: &str,
        task_type: &str,
        details: Option<&str>,
        duration: Option<u32>,
    ) -> Result<Task> {
        let priority = match priority.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            _ => Priority::Medium,
        };

        let task_type = match task_type.to_lowercase().as_str() {
            "development" => TaskType::Development,
            "testing" => TaskType::Testing,
            "infrastructure" => TaskType::Infrastructure,
            "documentation" => TaskType::Documentation,
            "bugfix" => TaskType::Bugfix,
            "feature" => TaskType::Feature,
            "review" => TaskType::Review,
            "coordination" => TaskType::Coordination,
            _ => TaskType::Development,
        };

        let estimated_duration = duration.map(|d| std::time::Duration::from_secs(d as u64));

        let mut task = Task::new(
            uuid::Uuid::new_v4().to_string(),
            description.to_string(),
            priority,
            task_type,
        );

        if let Some(details) = details {
            task = task.with_details(details.to_string());
        }

        task.estimated_duration = estimated_duration.map(|d| d.as_secs() as u32);

        Ok(task)
    }
}
