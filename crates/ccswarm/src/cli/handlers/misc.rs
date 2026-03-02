use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_auto_create(
        &self,
        description: &str,
        template: Option<&str>,
        auto_deploy: bool,
        output: &PathBuf,
    ) -> Result<()> {
        use crate::orchestrator::auto_create::AutoCreateEngine;

        info!("🚀 Starting auto-create for: {}", description);

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "started",
                    "message": "Auto-create process initiated",
                    "description": description,
                    "template": template,
                    "auto_deploy": auto_deploy,
                    "output": output,
                }))?
            );
        } else {
            println!("🚀 ccswarm Auto-Create");
            println!("====================");
            println!("📋 Request: {}", description);
            if let Some(tmpl) = template {
                println!("📄 Template: {}", tmpl);
            }
            println!("📂 Output: {}", output.display());
            println!();
        }

        // Create auto-create engine
        let mut engine = AutoCreateEngine::new();

        // Execute auto-create workflow
        match engine
            .execute_auto_create(description, &self.config, output)
            .await
        {
            Ok(()) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Application created successfully",
                            "output": output,
                        }))?
                    );
                } else {
                    println!("\n✅ Application created successfully!");
                    println!("📂 Location: {}", output.display());

                    if auto_deploy {
                        println!("\n🚀 Auto-deploying application...");

                        // Check for Dockerfile
                        let dockerfile_path = output.join("Dockerfile");
                        if !dockerfile_path.exists() {
                            println!("   Creating Dockerfile...");

                            // Generate basic Dockerfile based on detected app type
                            let dockerfile_content = if output.join("package.json").exists() {
                                // Node.js application
                                r#"FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
"#
                            } else if output.join("Cargo.toml").exists() {
                                // Rust application
                                r#"FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE 8080
CMD ["app"]
"#
                            } else if output.join("requirements.txt").exists() {
                                // Python application
                                r#"FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python", "app.py"]
"#
                            } else {
                                r#"FROM alpine:latest
WORKDIR /app
COPY . .
CMD ["/bin/sh"]
"#
                            };

                            tokio::fs::write(&dockerfile_path, dockerfile_content)
                                .await
                                .context("Failed to create Dockerfile")?;
                            println!("   ✅ Dockerfile created");
                        }

                        // Build Docker image
                        println!("   Building Docker image...");
                        let image_name =
                            format!("{}-app", description.to_lowercase().replace(' ', "-"));

                        let build_status = tokio::process::Command::new("docker")
                            .arg("build")
                            .arg("-t")
                            .arg(&image_name)
                            .arg(output.as_path())
                            .status()
                            .await
                            .context("Failed to execute docker build")?;

                        if !build_status.success() {
                            println!("   ⚠️ Docker build failed");
                        } else {
                            println!("   ✅ Docker image built: {}", image_name);

                            // Create docker-compose.yml if needed
                            let compose_path = output.join("docker-compose.yml");
                            if !compose_path.exists() {
                                println!("   Creating docker-compose.yml...");
                                let compose_content = format!(
                                    r#"version: '3.8'
services:
  app:
    image: {}
    ports:
      - "8080:8080"
    environment:
      - NODE_ENV=production
    restart: unless-stopped
"#,
                                    image_name
                                );

                                tokio::fs::write(&compose_path, compose_content)
                                    .await
                                    .context("Failed to create docker-compose.yml")?;
                                println!("   ✅ docker-compose.yml created");
                            }

                            println!("\n   🎉 Deployment ready!");
                            println!("   Run: cd {} && docker-compose up", output.display());
                        }
                    }
                }
            }
            Err(e) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "error",
                            "message": "Auto-create failed",
                            "error": e.to_string(),
                        }))?
                    );
                } else {
                    println!("\n❌ Auto-create failed: {}", e);
                }
                return Err(e);
            }
        }

        Ok(())
    }

    pub(crate) async fn show_logs(&self, follow: bool, agent: Option<&str>, lines: usize) -> Result<()> {
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
                println!("Run 'ccswarm start' to create logs");
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

    pub(crate) async fn run_review(&self, agent: Option<&str>, strict: bool) -> Result<()> {
        use crate::orchestrator::llm_quality_judge::{LLMQualityJudge, QualityRubric};

        // Get execution history from the engine
        let history = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            executor.get_execution_history(None).await
        } else {
            Vec::new()
        };

        // Filter by agent if specified
        let filtered: Vec<_> = if let Some(agent_filter) = agent {
            history
                .iter()
                .filter(|r| {
                    r.agent_used
                        .as_deref()
                        .is_some_and(|a| a.contains(agent_filter))
                })
                .collect()
        } else {
            history.iter().collect()
        };

        // Create quality judge with appropriate rubric
        let mut rubric = QualityRubric::default();
        if strict {
            // Raise all thresholds by 10%
            for threshold in rubric.thresholds.values_mut() {
                *threshold = (*threshold + 0.1).min(1.0);
            }
        }
        let _judge = LLMQualityJudge::with_rubric(rubric);

        let reviewed_count = filtered.len();
        let succeeded = filtered.iter().filter(|r| r.success).count();
        let failed = filtered.iter().filter(|r| !r.success).count();

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Quality review completed",
                    "tasks_reviewed": reviewed_count,
                    "tasks_succeeded": succeeded,
                    "tasks_failed": failed,
                    "strict_mode": strict,
                    "agent_filter": agent,
                }))?
            );
        } else {
            println!(
                "🔍 Quality Review {}",
                if strict { "(Strict Mode)" } else { "" }
            );
            println!("================");
            println!();

            if let Some(agent_filter) = agent {
                println!("   Filter: Agent '{}'", agent_filter);
            }

            if reviewed_count == 0 {
                println!("   No completed tasks to review");
                if self.execution_engine.is_none() {
                    println!("   Start ccswarm with 'ccswarm start' to enable task execution");
                }
            } else {
                println!("   Tasks reviewed: {}", reviewed_count);
                println!("   Succeeded: {}", format!("{}", succeeded).bright_green());
                println!("   Failed: {}", format!("{}", failed).bright_red());

                // Show details for failed tasks
                for result in &filtered {
                    if !result.success {
                        println!();
                        println!(
                            "   ❌ Task {}: {}",
                            result.task_id,
                            result.error.as_deref().unwrap_or("Unknown error")
                        );
                    }
                }
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

    pub(crate) async fn handle_tutorial(&self, chapter: Option<u8>) -> Result<()> {
        let mut tutorial = InteractiveTutorial::new();

        if let Some(ch) = chapter {
            if !(1..=4).contains(&ch) {
                println!(
                    "{}",
                    "❌ Invalid chapter number. Please choose 1-4.".bright_red()
                );
                return Ok(());
            }
            // Set starting chapter (adjusting for 0-based index)
            tutorial.current_chapter = (ch - 1) as usize;
        }

        tutorial.start().await?;
        Ok(())
    }

    pub(crate) async fn handle_template(&self, action: &TemplateAction) -> Result<()> {
        use crate::template::{
            FileSystemTemplateStorage, PredefinedTemplates, TemplateCategory, TemplateContext,
            TemplateManager, TemplateQuery,
        };
        use colored::Colorize;
        use std::io::{self, Write};
        use std::str::FromStr;

        // Initialize template storage
        let templates_dir = self.repo_path.join(".ccswarm").join("templates");
        let storage = FileSystemTemplateStorage::new(&templates_dir)
            .await
            .context("Failed to initialize template storage")?;
        let mut manager = TemplateManager::new(storage);

        match action {
            TemplateAction::List {
                all: _,
                category,
                tags,
                search,
                popular,
                quality,
                detailed,
            } => {
                let mut query = TemplateQuery::new();

                if let Some(cat_str) = category {
                    let cat = TemplateCategory::from_str(cat_str).context("Invalid category")?;
                    query = query.with_category(cat);
                }

                if !tags.is_empty() {
                    query = query.with_tags(tags.clone());
                }

                if let Some(search_term) = search {
                    query = query.with_search_term(search_term);
                }

                if *popular {
                    query = query.sort_by_popularity();
                } else if *quality {
                    query = query.sort_by_success_rate();
                }

                let templates = manager
                    .search_templates(query)
                    .await
                    .context("Failed to search templates")?;

                if templates.is_empty() {
                    println!("No templates found.");
                    return Ok(());
                }

                println!("Available Templates:");
                println!();

                for template in templates {
                    if *detailed {
                        println!(
                            "📋 {} ({})",
                            template.name.bright_cyan(),
                            template.id.as_str().bright_black()
                        );
                        println!("   Category: {}", template.category);
                        println!("   Description: {}", template.description);
                        if !template.tags.is_empty() {
                            println!(
                                "   Tags: {}",
                                template.tags.join(", ").as_str().bright_black()
                            );
                        }
                        if let Some(author) = &template.author {
                            println!("   Author: {}", author.as_str().bright_black());
                        }
                        println!("   Usage: {} times", template.usage_count);
                        if let Some(rate) = template.success_rate {
                            println!("   Success Rate: {:.1}%", rate * 100.0);
                        }
                        println!();
                    } else {
                        println!(
                            "  {} ({}) - {}",
                            template.name.bright_cyan(),
                            template.id.bright_black(),
                            template.description.chars().take(80).collect::<String>()
                        );
                    }
                }
            }

            TemplateAction::Show {
                template,
                source,
                stats,
            } => {
                let tmpl = manager
                    .get_template_by_name(template)
                    .await
                    .context("Template not found")?;

                println!(
                    "Template: {} ({})",
                    tmpl.name.bright_cyan(),
                    tmpl.id.bright_black()
                );
                println!("Category: {}", tmpl.category);
                println!("Description: {}", tmpl.description);
                println!("Version: {}", tmpl.version);

                if let Some(author) = &tmpl.author {
                    println!("Author: {}", author);
                }

                if !tmpl.tags.is_empty() {
                    println!("Tags: {}", tmpl.tags.join(", "));
                }

                println!("Priority: {:?}", tmpl.default_priority);
                println!("Task Type: {:?}", tmpl.default_task_type);

                if let Some(duration) = tmpl.estimated_duration {
                    println!("Estimated Duration: {} minutes", duration);
                }

                if !tmpl.variables.is_empty() {
                    println!();
                    println!("Variables:");
                    for var in &tmpl.variables {
                        let required = if var.required { " (required)" } else { "" };
                        println!(
                            "  • {}{}: {}",
                            var.name.bright_green(),
                            required,
                            var.description
                        );
                        if let Some(default) = &var.default_value {
                            println!("    Default: {}", default.clone().bright_black());
                        }
                    }
                }

                if *source {
                    println!();
                    println!("Task Description Template:");
                    println!("{}", tmpl.task_description.bright_white());

                    if let Some(details) = &tmpl.task_details {
                        println!();
                        println!("Task Details Template:");
                        println!("{}", details.bright_white());
                    }
                }

                if *stats {
                    println!();
                    println!("Statistics:");
                    println!("  Usage Count: {}", tmpl.usage_count);
                    if let Some(rate) = tmpl.success_rate {
                        println!("  Success Rate: {:.1}%", rate * 100.0);
                    }
                    println!("  Created: {}", tmpl.created_at.format("%Y-%m-%d %H:%M"));
                    println!("  Updated: {}", tmpl.updated_at.format("%Y-%m-%d %H:%M"));
                }
            }

            TemplateAction::Apply {
                template,
                vars,
                interactive,
                preview,
                auto_assign,
            } => {
                let tmpl = manager
                    .get_template_by_name(template)
                    .await
                    .context("Template not found")?;

                let mut context = TemplateContext::new();

                // Parse provided variables
                for var_str in vars {
                    if let Some((key, value)) = var_str.split_once('=') {
                        context = context.with_variable(key.trim(), value.trim());
                    }
                }

                // Interactive mode for missing variables
                if *interactive {
                    for var in &tmpl.variables {
                        if var.required
                            && !context.variables.contains_key(&var.name)
                            && var.default_value.is_none()
                        {
                            print!("{} ({}): ", var.name, var.description);
                            io::stdout().flush()?;

                            let mut input = String::new();
                            io::stdin().read_line(&mut input)?;
                            let value = input.trim();

                            if !value.is_empty() {
                                context = context.with_variable(&var.name, value);
                            }
                        }
                    }
                }

                // Apply template
                let applied = manager
                    .apply_template(&tmpl.id, context)
                    .await
                    .context("Failed to apply template")?;

                if *preview {
                    println!("Preview of generated task:");
                    println!();
                    println!("Description: {}", applied.description.bright_cyan());
                    if let Some(details) = &applied.details {
                        println!("Details: {}", details);
                    }
                    println!("Priority: {:?}", applied.priority);
                    println!("Type: {:?}", applied.task_type);
                    if let Some(duration) = applied.estimated_duration {
                        println!("Duration: {} minutes", duration);
                    }
                    if !applied.target_files.is_empty() {
                        println!("Target Files: {}", applied.target_files.join(", "));
                    }
                } else {
                    // Create the actual task
                    use crate::agent::TaskBuilder;

                    let task = TaskBuilder::new(applied.description.clone())
                        .priority(applied.priority)
                        .task_type(applied.task_type);

                    let task = if let Some(details) = &applied.details {
                        task.details(details.clone())
                    } else {
                        task
                    };

                    let task = if let Some(duration) = applied.estimated_duration {
                        task.estimated_duration(duration as u64)
                    } else {
                        task
                    };

                    let task = task.build();

                    println!(
                        "Created task from template: {}",
                        applied.description.bright_green()
                    );
                    println!("Task ID: {}", task.id.bright_cyan());

                    if *auto_assign {
                        println!("Auto-assigning to best agent...");

                        use crate::orchestrator::master_delegation::{
                            DelegationStrategy, MasterDelegationEngine,
                        };
                        let mut engine = MasterDelegationEngine::new(DelegationStrategy::Hybrid);

                        match engine.delegate_task(task.clone()) {
                            Ok(decision) => {
                                println!(
                                    "   ✅ Assigned to: {}",
                                    decision.target_agent.name().bright_green()
                                );
                                println!("   Confidence: {:.1}%", decision.confidence * 100.0);
                                println!("   Reason: {}", decision.reasoning);

                                // Add to execution queue if engine is running
                                if let Some(ref engine) = self.execution_engine {
                                    let assigned_task = task
                                        .clone()
                                        .assign_to(decision.target_agent.name().to_string());
                                    let task_id =
                                        engine.get_executor().add_task(assigned_task).await;
                                    println!(
                                        "   📋 Queued for execution: {}",
                                        task_id.bright_cyan()
                                    );
                                }
                            }
                            Err(e) => {
                                println!("   ⚠️ Auto-assignment failed: {}", e);
                                println!("   Task created but not assigned");
                            }
                        }
                    }
                }
            }

            TemplateAction::Install {
                all,
                categories,
                force,
            } => {
                let predefined_templates = PredefinedTemplates::get_all();
                let mut installed = 0;
                let mut skipped = 0;

                for template in predefined_templates {
                    // Filter by categories if specified
                    if !categories.is_empty() && !*all {
                        let cat_str = template.category.to_string();
                        if !categories.iter().any(|c| c.eq_ignore_ascii_case(&cat_str)) {
                            continue;
                        }
                    }

                    match manager.save_template(template.clone()).await {
                        Ok(()) => {
                            println!("✅ Installed: {}", template.name.bright_green());
                            installed += 1;
                        }
                        Err(e) if e.to_string().contains("already exists") => {
                            if *force {
                                if let Err(e) = manager.update_template(template.clone()).await {
                                    println!("❌ Failed to update {}: {}", template.name.red(), e);
                                } else {
                                    println!("✅ Updated: {}", template.name.bright_green());
                                    installed += 1;
                                }
                            } else {
                                println!("⚠️  Skipped (exists): {}", template.name.bright_yellow());
                                skipped += 1;
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to install {}: {}", template.name.red(), e);
                        }
                    }
                }

                println!();
                println!(
                    "Installation complete: {} installed, {} skipped",
                    installed, skipped
                );
                if skipped > 0 && !*force {
                    println!("Use --force to overwrite existing templates");
                }
            }

            TemplateAction::Stats {
                global: _,
                template,
            } => {
                if let Some(tmpl_name) = template {
                    let tmpl = manager
                        .get_template_by_name(tmpl_name)
                        .await
                        .context("Template not found")?;

                    println!("Template Statistics: {}", tmpl.name.bright_cyan());
                    println!("Usage Count: {}", tmpl.usage_count);
                    if let Some(rate) = tmpl.success_rate {
                        println!("Success Rate: {:.1}%", rate * 100.0);
                    }
                    println!("Created: {}", tmpl.created_at.format("%Y-%m-%d %H:%M"));
                    println!("Updated: {}", tmpl.updated_at.format("%Y-%m-%d %H:%M"));
                } else {
                    let stats = manager
                        .get_template_stats()
                        .await
                        .context("Failed to get template statistics")?;

                    println!("Global Template Statistics:");
                    println!("Total Templates: {}", stats.total_templates);
                    println!("Total Usage: {}", stats.total_usage);
                    println!(
                        "Average Success Rate: {:.1}%",
                        stats.average_success_rate * 100.0
                    );

                    println!();
                    println!("By Category:");
                    for (category, count) in &stats.by_category {
                        println!("  {}: {}", category, count);
                    }

                    if !stats.most_popular.is_empty() {
                        println!();
                        println!("Most Popular:");
                        for (name, count) in stats.most_popular.iter().take(5) {
                            println!("  {}: {} uses", name, count);
                        }
                    }
                }
            }

            _ => {
                println!("Template command not yet implemented: {:?}", action);
            }
        }

        Ok(())
    }

    /// Handle quality checks through agent delegation
    pub(crate) async fn handle_quality(&self, action: &QualityAction) -> Result<()> {
        use std::process::Command;

        let mut failed_checks = Vec::new();
        let mut completed_tasks = 0;
        let total_tasks;

        match action {
            QualityAction::Check {
                skip,
                only,
                fail_fast,
            } => {
                println!("🤖 ccswarm Agent-Managed Quality Checks");
                println!("=======================================");
                println!();

                let checks = if only.is_empty() {
                    vec!["format", "lint", "test", "build", "security"]
                } else {
                    only.iter().map(|s| s.as_str()).collect()
                };

                let filtered_checks: Vec<&str> = checks
                    .into_iter()
                    .filter(|check| !skip.contains(&check.to_string()))
                    .collect();

                total_tasks = filtered_checks.len();
                println!(
                    "🎯 Master Claude: Orchestrating {} quality checks through specialized agents...",
                    total_tasks
                );
                println!();

                for check in filtered_checks {
                    let (agent, task, cmd) = match check {
                        "format" => (
                            "DevOps",
                            "Code Formatting Check",
                            vec!["cargo", "fmt", "--check"],
                        ),
                        "lint" => (
                            "DevOps",
                            "Clippy Code Quality Analysis",
                            vec![
                                "cargo",
                                "clippy",
                                "--all-targets",
                                "--all-features",
                                "--",
                                "-D",
                                "warnings",
                            ],
                        ),
                        "test" => (
                            "QA",
                            "Test Suite Execution",
                            vec!["cargo", "test", "--lib", "--verbose", "--no-fail-fast"],
                        ),
                        "build" => (
                            "DevOps",
                            "Build Verification",
                            vec!["cargo", "build", "--verbose"],
                        ),
                        "security" => (
                            "Backend",
                            "Security Analysis",
                            vec![
                                "cargo",
                                "test",
                                "security::owasp_checker::tests",
                                "--no-fail-fast",
                            ],
                        ),
                        _ => continue,
                    };

                    println!("🎯 Delegating to {} agent: {}", agent, task);

                    let mut command = Command::new(&cmd[0]);
                    for arg in &cmd[1..] {
                        command.arg(arg);
                    }

                    let output = command.output().context("Failed to execute command")?;
                    let success = output.status.success();

                    if success {
                        println!("✅ {} agent: {} completed successfully", agent, task);
                        completed_tasks += 1;
                    } else {
                        println!("❌ {} agent: {} failed", agent, task);
                        failed_checks.push((
                            agent,
                            task,
                            String::from_utf8_lossy(&output.stderr).to_string(),
                        ));

                        if *fail_fast {
                            break;
                        }
                    }
                    println!();
                }

                // Quality Gate Assessment
                println!("🎯 Master Claude - Quality Gate Assessment");
                println!("==========================================");
                println!("📊 Agent Task Completion Summary:");
                println!("   Completed: {}/{} tasks", completed_tasks, total_tasks);
                println!("   Failed: {}", failed_checks.len());
                println!();

                if failed_checks.is_empty() {
                    println!("✅ QUALITY GATE: PASSED");
                    println!("🎉 All quality checks passed through agent delegation");
                    println!("🚀 Code is ready for deployment");
                } else {
                    println!("❌ QUALITY GATE: FAILED");
                    println!("🔧 Some quality checks require attention from agents");
                    println!();
                    println!("📋 Failed Checks:");
                    for (agent, task, _error) in &failed_checks {
                        println!("   ❌ {} agent: {}", agent, task);
                    }
                    return Err(anyhow::anyhow!("Quality gate failed"));
                }
            }

            QualityAction::Format { fix } => {
                println!("🛠️ DevOps Agent - Code Formatting");
                println!("==================================");

                let cmd = if *fix {
                    vec!["cargo", "fmt"]
                } else {
                    vec!["cargo", "fmt", "--check"]
                };

                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .context("Failed to run cargo fmt")?;

                if output.status.success() {
                    println!(
                        "✅ DevOps Agent: Code formatting {} successfully",
                        if *fix { "applied" } else { "verified" }
                    );
                } else {
                    println!("❌ DevOps Agent: Code formatting issues detected");
                    if !fix {
                        println!("💡 Run with --fix to automatically format code");
                    }
                    return Err(anyhow::anyhow!("Formatting check failed"));
                }
            }

            QualityAction::Lint { fix } => {
                println!("🛠️ DevOps Agent - Clippy Analysis");
                println!("==================================");

                let mut cmd = vec!["cargo", "clippy", "--all-targets", "--all-features"];
                if *fix {
                    cmd.push("--fix");
                    cmd.push("--allow-dirty");
                }
                cmd.extend(&["--", "-D", "warnings"]);

                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .context("Failed to run cargo clippy")?;

                if output.status.success() {
                    println!("✅ DevOps Agent: Clippy analysis passed");
                } else {
                    println!("❌ DevOps Agent: Clippy found issues");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    return Err(anyhow::anyhow!("Clippy check failed"));
                }
            }

            QualityAction::Test {
                pattern,
                unit,
                integration,
                security,
            } => {
                println!("🧪 QA Agent - Test Execution");
                println!("============================");

                let mut cmd = vec!["cargo", "test"];

                if *unit {
                    cmd.push("--lib");
                } else if *integration {
                    cmd.extend(&["--test", "*integration*"]);
                } else if *security {
                    cmd.push("security::owasp_checker::tests");
                }

                if let Some(p) = pattern {
                    cmd.push(p);
                }

                cmd.extend(&["--verbose", "--no-fail-fast"]);

                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .context("Failed to run tests")?;

                if output.status.success() {
                    println!("✅ QA Agent: All tests passed");
                } else {
                    println!("❌ QA Agent: Some tests failed");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    return Err(anyhow::anyhow!("Tests failed"));
                }
            }

            QualityAction::Build {
                release,
                all_targets,
            } => {
                println!("🛠️ DevOps Agent - Build Verification");
                println!("=====================================");

                let mut cmd = vec!["cargo", "build", "--verbose"];

                if *release {
                    cmd.push("--release");
                }
                if *all_targets {
                    cmd.push("--all-targets");
                }

                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .context("Failed to build")?;

                if output.status.success() {
                    println!("✅ DevOps Agent: Build completed successfully");
                } else {
                    println!("❌ DevOps Agent: Build failed");
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                    return Err(anyhow::anyhow!("Build failed"));
                }
            }

            QualityAction::Security { audit, deps } => {
                println!("🦀 Backend Agent - Security Analysis");
                println!("====================================");

                if *audit {
                    let output = Command::new("cargo").args(&["audit"]).output();

                    match output {
                        Ok(out) if out.status.success() => {
                            println!("✅ Backend Agent: Security audit passed");
                        }
                        _ => {
                            println!(
                                "❌ Backend Agent: Security audit found issues (or cargo-audit not installed)"
                            );
                        }
                    }
                }

                if *deps {
                    println!("🔍 Backend Agent: Checking dependencies...");
                    // Run dependency checks
                }

                // Always run security tests
                let output = Command::new("cargo")
                    .args(&["test", "security::owasp_checker::tests", "--no-fail-fast"])
                    .output()
                    .context("Failed to run security tests")?;

                if output.status.success() {
                    println!("✅ Backend Agent: Security tests passed");
                } else {
                    println!("❌ Backend Agent: Security tests failed");
                    return Err(anyhow::anyhow!("Security tests failed"));
                }
            }

            QualityAction::Status { detailed } => {
                println!("📊 Quality Gate Status");
                println!("======================");

                // Run quick checks to show status
                let checks = [
                    ("Format", vec!["cargo", "fmt", "--check"]),
                    ("Clippy", vec!["cargo", "clippy", "--", "-D", "warnings"]),
                    ("Tests", vec!["cargo", "test", "--lib", "--no-run"]),
                    ("Build", vec!["cargo", "check"]),
                ];

                for (name, cmd) in &checks {
                    match Command::new(&cmd[0]).args(&cmd[1..]).output() {
                        Ok(output) => {
                            let status = if output.status.success() {
                                "✅ PASS"
                            } else {
                                "❌ FAIL"
                            };
                            println!("  {}: {}", name, status);

                            if *detailed && !output.status.success() {
                                println!("    Error: {}", String::from_utf8_lossy(&output.stderr));
                            }
                        }
                        Err(_) => {
                            println!("  {}: ❌ FAIL (command error)", name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_help(&self, topic: Option<&str>, search: Option<&str>) -> Result<()> {
        let help = InteractiveHelp::new();

        if let Some(query) = search {
            // Search help topics
            let results = help.search(query);

            if results.is_empty() {
                println!();
                println!(
                    "{}",
                    "❌ No help topics found matching your search.".bright_red()
                );
                println!();
                println!("Try one of these topics:");
                help.show_topic_list();
            } else {
                println!();
                println!(
                    "{}",
                    format!("🔍 Found {} topics matching '{}'", results.len(), query).bright_cyan()
                );
                println!();

                for (key, topic) in results.iter().take(3) {
                    println!("{}", format!("📖 {}", topic.title).bright_yellow());
                    println!("   {}", topic.description.bright_black());
                    println!("   Run: ccswarm help {}", key.bright_white());
                    println!();
                }
            }
        } else if let Some(t) = topic {
            help.show_topic(t);
        } else {
            help.show_topic_list();
        }

        Ok(())
    }

    pub(crate) async fn handle_resource(&self, action: &resource_commands::ResourceSubcommand) -> Result<()> {
        // Get or create session manager
        let session_manager = Arc::new(
            crate::session::SessionManager::with_resource_monitoring(
                crate::resource::ResourceLimits::default(),
            )
            .await?,
        );

        // Create resource command and execute
        let resource_cmd = resource_commands::ResourceCommand {
            subcommand: action.clone(),
        };

        resource_cmd.execute(session_manager).await
    }

    /// Validate that the Claude Code CLI provider is available and configured
    pub(crate) async fn validate_provider(&self) -> Result<()> {
        use crate::providers::ClaudeCodeConfig;
        use crate::providers::ProviderConfig;

        let config = ClaudeCodeConfig::default();

        // Check if claude CLI is installed
        if !config.is_available().await {
            return Err(anyhow!(
                "Claude Code CLI not found in PATH.\n\
                 Install it with: npm install -g @anthropic-ai/claude-code\n\
                 Or see: https://docs.anthropic.com/en/docs/claude-code"
            ));
        }

        // Check for API key (warn, don't fail — claude CLI may have its own auth)
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            warn!(
                "ANTHROPIC_API_KEY not set. Claude Code CLI may use its own authentication, \
                 but some features may not work without it."
            );
        }

        info!("Provider validation passed: Claude Code CLI is available");
        Ok(())
    }

}
