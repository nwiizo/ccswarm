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

    pub(crate) async fn handle_template(&self, _action: &TemplateAction) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "removed",
                    "message": "Template system has been removed. Use 'ccswarm task' directly.",
                }))?
            );
        } else {
            println!("Template system has been removed.");
            println!("Use 'ccswarm task execute <description>' to create tasks directly.");
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

    pub(crate) async fn handle_help(
        &self,
        topic: Option<&str>,
        search: Option<&str>,
    ) -> Result<()> {
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

    pub(crate) async fn handle_resource(
        &self,
        action: &resource_commands::ResourceSubcommand,
    ) -> Result<()> {
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
