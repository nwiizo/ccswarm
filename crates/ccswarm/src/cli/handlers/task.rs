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

        let task_id = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            executor.add_task(task.clone()).await
        } else {
            warn!("Execution engine not available, task will not be executed");
            task.id.clone()
        };

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
        if let Some(ref _engine) = self.execution_engine {
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
                println!("⚡ Immediate Task Execution");
                println!("==========================");
                println!("Task: {}", task);
                if let Some(a) = agent {
                    println!("Agent: {}", a);
                }
                println!("Orchestration: {}", if orchestrate { "Yes" } else { "No" });
                println!();
                println!("❌ Immediate execution not yet implemented");
                println!("💡 Use 'ccswarm task add' to add task to queue for execution");
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task": task,
                        "agent": agent,
                        "orchestrate": orchestrate,
                        "status": "execution_engine_unavailable"
                    }))?
                );
            } else {
                println!("❌ Execution engine not available");
                println!("   Cannot execute tasks without execution engine");
            }
        }
        Ok(())
    }

    pub(crate) async fn list_tasks(
        &self,
        all: bool,
        status_filter: Option<&str>,
        agent_filter: Option<&str>,
        detailed: bool,
        branches: bool,
    ) -> Result<()> {
        let tasks = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();
            task_queue.list_tasks(status_filter, agent_filter).await
        } else {
            Vec::new()
        };

        let displayed_tasks = if all {
            tasks
        } else {
            tasks.into_iter().take(50).collect()
        };

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "tasks": displayed_tasks,
                    "total": displayed_tasks.len(),
                    "filters": {
                        "all": all,
                        "status": status_filter,
                        "agent": agent_filter,
                        "detailed": detailed
                    }
                }))?
            );
        } else {
            println!("📋 Task List");
            println!("============");

            if displayed_tasks.is_empty() {
                println!("No tasks in queue currently.");
                println!();
                println!("💡 Add a task with: ccswarm task add \"Your task description\"");
            } else {
                println!("Found {} tasks:", displayed_tasks.len());
                println!();

                for task in &displayed_tasks {
                    let status_emoji = match &task.status {
                        TaskStatus::Pending => "⏳",
                        TaskStatus::Assigned { .. } => "📋",
                        TaskStatus::InProgress { .. } => "🏃",
                        TaskStatus::Completed { .. } => "✅",
                        TaskStatus::Failed { .. } => "❌",
                        TaskStatus::Cancelled { .. } => "🚫",
                    };

                    let priority_emoji = match task.task.priority {
                        Priority::Critical => "🚨",
                        Priority::High => "🔥",
                        Priority::Medium => "📅",
                        Priority::Low => "💤",
                    };

                    println!(
                        "{} {} {} [{}] {}",
                        status_emoji,
                        priority_emoji,
                        &task.task.id[..8], // Short ID
                        task.task.task_type,
                        task.task.description
                    );

                    if let Some(agent) = &task.assigned_agent {
                        println!("   👤 Assigned to: {}", agent);
                    }

                    if detailed {
                        println!(
                            "   ⏰ Created: {}",
                            task.created_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!(
                            "   🔄 Updated: {}",
                            task.updated_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        if let Some(details) = &task.task.details {
                            println!("   📝 Details: {}", details);
                        }
                    }

                    if branches {
                        // Sanitize task ID for branch name lookup
                        let safe_id: String = task
                            .task
                            .id
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
                        println!("   🌿 Branch: {}", branch_name.bright_green());
                    }

                    println!();
                }
            }
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
    pub(crate) async fn retry_task(&self, task_id: &str, force: bool) -> Result<()> {
        if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();

            // Check task status
            let tasks = task_queue.list_tasks(None, None).await;
            let task = tasks
                .iter()
                .find(|t| t.task.id == task_id)
                .ok_or_else(|| anyhow!("Task '{}' not found", task_id))?;

            let is_failed = matches!(task.status, TaskStatus::Failed { .. });
            if !is_failed && !force {
                return Err(anyhow!(
                    "Task '{}' is not in failed state. Use --force to retry anyway.",
                    task_id
                ));
            }

            // Re-add the task to the queue
            let new_task = task.task.clone();
            let new_id = task_queue.add_task(new_task).await;

            println!(
                "{} Task retried. New task ID: {}",
                "OK".bright_green().bold(),
                new_id.bright_cyan()
            );
        } else {
            return Err(anyhow!(
                "Execution engine not running. Start with: ccswarm start"
            ));
        }

        Ok(())
    }

    /// Delete a task and its associated worktree
    pub(crate) async fn delete_task(&self, task_id: &str, force: bool) -> Result<()> {
        if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();

            // Check task status
            let tasks = task_queue.list_tasks(None, None).await;
            let task = tasks.iter().find(|t| t.task.id == task_id);

            if let Some(task) = task {
                let is_active = matches!(task.status, TaskStatus::InProgress { .. });
                if is_active && !force {
                    return Err(anyhow!(
                        "Task '{}' is in progress. Use --force to delete anyway.",
                        task_id
                    ));
                }

                // Cancel the task
                executor
                    .cancel_task(task_id, Some("Deleted by user".to_string()))
                    .await?;
            }
        }

        // Also clean up any associated worktree
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
        history: bool,
        orchestration: bool,
    ) -> Result<()> {
        let task = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();
            task_queue.get_task(task_id).await
        } else {
            None
        };

        if let Some(task) = task {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task_id": task_id,
                        "task": task,
                        "history": history,
                        "orchestration": orchestration
                    }))?
                );
            } else {
                println!("🔍 Task Status: {}", task_id);
                println!("===============");

                let status_emoji = match &task.status {
                    TaskStatus::Pending => "⏳",
                    TaskStatus::Assigned { .. } => "📋",
                    TaskStatus::InProgress { .. } => "🏃",
                    TaskStatus::Completed { .. } => "✅",
                    TaskStatus::Failed { .. } => "❌",
                    TaskStatus::Cancelled { .. } => "🚫",
                };

                println!("{} Status: {:?}", status_emoji, task.status);
                println!("📝 Description: {}", task.task.description);
                println!("🎯 Priority: {:?}", task.task.priority);
                println!("📋 Type: {:?}", task.task.task_type);
                println!(
                    "⏰ Created: {}",
                    task.created_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!(
                    "🔄 Updated: {}",
                    task.updated_at.format("%Y-%m-%d %H:%M:%S")
                );

                if let Some(agent) = &task.assigned_agent {
                    println!("👤 Assigned to: {}", agent);
                }

                if let Some(details) = &task.task.details {
                    println!("📝 Details: {}", details);
                }

                if history && !task.execution_history.is_empty() {
                    println!();
                    println!("📚 Execution History:");
                    for (i, attempt) in task.execution_history.iter().enumerate() {
                        println!(
                            "  {}. Agent: {} | Started: {}",
                            i + 1,
                            attempt.agent_id,
                            attempt.started_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        if let Some(completed) = attempt.completed_at {
                            println!("     Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                        }
                        if let Some(error) = &attempt.error {
                            println!("     Error: {}", error);
                        }
                    }
                }
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task_id": task_id,
                        "status": "not_found",
                        "history": history,
                        "orchestration": orchestration
                    }))?
                );
            } else {
                println!("🔍 Task Status: {}", task_id);
                println!("===============");
                println!("❌ Task not found");
                println!();
                println!("💡 Use 'ccswarm task list' to see available tasks");
            }
        }
        Ok(())
    }

    /// Cancel a task
    pub(crate) async fn cancel_task(
        &self,
        task_id: &str,
        force: bool,
        reason: Option<&str>,
    ) -> Result<()> {
        let result = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            match executor
                .cancel_task(task_id, reason.map(|s| s.to_string()))
                .await
            {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(anyhow::anyhow!("Execution engine not available"))
        };

        match result {
            Ok(()) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "task_id": task_id,
                            "cancelled": true,
                            "reason": reason,
                            "force": force
                        }))?
                    );
                } else {
                    println!("✅ Task cancelled successfully: {}", task_id);
                    if let Some(r) = reason {
                        println!("   Reason: {}", r);
                    }
                }
            }
            Err(e) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "task_id": task_id,
                            "cancelled": false,
                            "reason": e.to_string(),
                            "force": force
                        }))?
                    );
                } else {
                    println!("❌ Failed to cancel task: {}", task_id);
                    println!("   Reason: {}", e);
                    if let Some(r) = reason {
                        println!("   User reason: {}", r);
                    }
                }
            }
        }
        Ok(())
    }

    /// Show task execution history
    pub(crate) async fn show_task_history(
        &self,
        limit: usize,
        agent_filter: Option<&str>,
        failed_only: bool,
    ) -> Result<()> {
        let history = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            executor.get_execution_history(Some(limit)).await
        } else {
            Vec::new()
        };

        let filtered_history: Vec<_> = history
            .into_iter()
            .filter(|result| {
                if failed_only && result.success {
                    return false;
                }
                if let Some(agent) = agent_filter {
                    if let Some(ref result_agent) = result.agent_used {
                        return result_agent == agent;
                    }
                    return false;
                }
                true
            })
            .collect();

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "history": filtered_history,
                    "limit": limit,
                    "agent_filter": agent_filter,
                    "failed_only": failed_only,
                    "total_count": filtered_history.len()
                }))?
            );
        } else {
            println!("📚 Task History (Last {} tasks)", limit);
            println!("================================");

            if filtered_history.is_empty() {
                println!("No task history available.");
            } else {
                println!("Found {} execution records:", filtered_history.len());
                println!();

                for result in &filtered_history {
                    let status_emoji = if result.success { "✅" } else { "❌" };
                    let orchestration_indicator = if result.orchestration_used {
                        "🎯"
                    } else {
                        "🔄"
                    };

                    println!(
                        "{} {} Task: {} | Duration: {:.2}s",
                        status_emoji,
                        orchestration_indicator,
                        &result.task_id[..8],
                        result.duration.as_secs_f64()
                    );

                    if let Some(agent) = &result.agent_used {
                        println!("   👤 Agent: {}", agent);
                    }

                    if let Some(error) = &result.error {
                        println!("   ❌ Error: {}", error);
                    }
                    println!();
                }
            }

            if let Some(agent) = agent_filter {
                println!("Filter: Agent = {}", agent);
            }
            if failed_only {
                println!("Filter: Failed tasks only");
            }
        }
        Ok(())
    }

    /// Show task queue statistics
    pub(crate) async fn show_task_stats(&self, detailed: bool, performance: bool) -> Result<()> {
        if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let queue_stats = executor.get_task_queue().get_stats().await;
            let execution_stats = executor.get_stats().await;

            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "queue_stats": queue_stats,
                        "execution_stats": execution_stats,
                        "detailed": detailed,
                        "performance": performance
                    }))?
                );
            } else {
                println!("📊 Task Queue Statistics");
                println!("========================");
                println!("⏳ Pending: {}", queue_stats.pending_count);
                println!("🏃 In Progress: {}", queue_stats.active_count);
                println!("✅ Completed: {}", queue_stats.completed_count);
                println!("❌ Failed: {}", queue_stats.failed_count);
                println!("📋 Total: {}", queue_stats.total_count);

                if performance || detailed {
                    println!();
                    println!("🎯 Execution Statistics");
                    println!("=======================");
                    println!("Tasks Executed: {}", execution_stats.tasks_executed);
                    println!(
                        "Success Rate: {:.1}%",
                        if execution_stats.tasks_executed > 0 {
                            (execution_stats.tasks_succeeded as f64
                                / execution_stats.tasks_executed as f64)
                                * 100.0
                        } else {
                            0.0
                        }
                    );
                    println!(
                        "Average Duration: {:.2}s",
                        execution_stats.average_duration.as_secs_f64()
                    );
                    println!(
                        "Total Duration: {:.2}s",
                        execution_stats.total_duration.as_secs_f64()
                    );
                    println!(
                        "Orchestration Usage: {:.1}%",
                        execution_stats.orchestration_usage
                    );
                }

                if detailed {
                    println!();
                    println!("📈 Queue Health");
                    println!("===============");
                    let failure_rate = if queue_stats.total_count > 0 {
                        (queue_stats.failed_count as f64 / queue_stats.total_count as f64) * 100.0
                    } else {
                        0.0
                    };

                    let health_emoji = if failure_rate < 5.0 {
                        "🟢"
                    } else if failure_rate < 15.0 {
                        "🟡"
                    } else {
                        "🔴"
                    };

                    println!(
                        "{} Overall Health: {:.1}% failure rate",
                        health_emoji, failure_rate
                    );

                    if queue_stats.active_count > 10 {
                        println!(
                            "⚠️  High concurrent load: {} tasks",
                            queue_stats.active_count
                        );
                    }

                    if queue_stats.pending_count > 50 {
                        println!(
                            "⚠️  Queue backlog: {} pending tasks",
                            queue_stats.pending_count
                        );
                    }
                }
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": "execution_engine_unavailable",
                        "detailed": detailed,
                        "performance": performance
                    }))?
                );
            } else {
                println!("❌ Execution engine not available");
                println!("   Cannot display task statistics");
            }
        }
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
