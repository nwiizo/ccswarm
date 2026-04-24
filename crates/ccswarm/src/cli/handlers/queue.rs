//! Task queue: append tasks to `.ccswarm/queue.yaml` and drain them through the pipeline.
//!
//! OK/NG-driven usage: user queues tasks throughout the day, then runs `ccswarm queue drain`
//! and says y/n at commit + PR time — no per-task babysitting.

use super::super::*;
use super::queue_state::{QUEUE_FILE, QueueTask, load_queue, save_queue};
use chrono::Utc;

/// Fetch an issue body + title from `gh` CLI. Returns "<title>\n\n<body>".
async fn fetch_issue(number: u64) -> Result<String> {
    let output = tokio::process::Command::new("gh")
        .args(["issue", "view", &number.to_string(), "--json", "title,body"])
        .output()
        .await
        .context("Failed to run `gh issue view`. Is GitHub CLI installed and authenticated?")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh issue view failed: {}", stderr));
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let title = v.get("title").and_then(|s| s.as_str()).unwrap_or("");
    let body = v.get("body").and_then(|s| s.as_str()).unwrap_or("");
    if title.is_empty() {
        return Err(anyhow!("Issue #{} returned no title", number));
    }
    Ok(format!("{}\n\n{}", title, body))
}

impl CliRunner {
    pub(crate) async fn handle_queue(&self, action: &QueueAction) -> Result<()> {
        let path = self.repo_path.join(QUEUE_FILE);
        match action {
            QueueAction::Add {
                task,
                from_issue,
                file,
                flow,
            } => {
                self.queue_add(&path, task, *from_issue, file.as_deref(), flow.as_deref())
                    .await
            }
            QueueAction::List => self.queue_list(&path).await,
            QueueAction::Clear => self.queue_clear(&path).await,
            QueueAction::Drain {
                flow,
                timeout,
                fail_fast,
                interactive,
                create_pr,
            } => {
                self.queue_drain(
                    &path,
                    flow.as_deref(),
                    *timeout,
                    *fail_fast,
                    *interactive,
                    *create_pr,
                )
                .await
            }
        }
    }

    async fn queue_add(
        &self,
        path: &std::path::Path,
        task: &str,
        from_issue: Option<u64>,
        file: Option<&std::path::Path>,
        flow: Option<&str>,
    ) -> Result<()> {
        // Input precedence: --from-issue > --file > `-` (stdin) > positional argument.
        // Allowing all four would be confusing; we error if >1 is given.
        let sources_count = [from_issue.is_some(), file.is_some(), task == "-"]
            .iter()
            .filter(|&&b| b)
            .count();
        if sources_count > 1 {
            return Err(anyhow!(
                "choose only one of: --from-issue <N>, --file <path>, or `-` (stdin)"
            ));
        }

        let body = if let Some(n) = from_issue {
            fetch_issue(n).await?
        } else if let Some(p) = file {
            tokio::fs::read_to_string(p)
                .await
                .with_context(|| format!("Failed to read task file {}", p.display()))?
        } else if task == "-" {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read task from stdin")?;
            buf
        } else if task.trim().is_empty() {
            return Err(anyhow!(
                "Provide a task description, --file <path>, --from-issue <N>, or pass `-` to read stdin."
            ));
        } else {
            task.to_string()
        };

        let mut queue = load_queue(path).await?;
        let id = format!("q-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let entry = QueueTask {
            id: id.clone(),
            task: body.clone(),
            flow: flow.map(String::from),
            state: "pending".to_string(),
            created_at: Utc::now(),
            completed_at: None,
            run_id: None,
        };
        queue.tasks.push(entry);
        save_queue(path, &queue).await?;
        let preview = body.lines().next().unwrap_or("").trim();
        println!(
            "{} queued {} — {}",
            "OK".bright_green().bold(),
            id.bright_cyan(),
            preview
        );
        Ok(())
    }

    async fn queue_list(&self, path: &std::path::Path) -> Result<()> {
        let queue = load_queue(path).await?;
        if queue.tasks.is_empty() {
            println!("Queue is empty.");
            return Ok(());
        }
        // #43 fix: surface the short run_id so users can jump directly to
        // `ccswarm run view <short>` / `ccswarm cost <short>` without cd-ing into
        // `.ccswarm/runs/` to find it.
        println!(
            "{:<12}  {:<9}  {:<10}  {:<10}  {}",
            "ID".bright_cyan().bold(),
            "State".bright_cyan().bold(),
            "Flow".bright_cyan().bold(),
            "Run".bright_cyan().bold(),
            "Task".bright_cyan().bold(),
        );
        for t in &queue.tasks {
            let state = match t.state.as_str() {
                "pending" => t.state.bright_yellow(),
                "running" => t.state.bright_blue(),
                "completed" => t.state.bright_green(),
                "failed" => t.state.bright_red(),
                _ => t.state.bright_white(),
            };
            let preview = t
                .task
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();
            let run_short = t
                .run_id
                .as_deref()
                .map(|r| r.chars().take(8).collect::<String>())
                .unwrap_or_else(|| "-".into());
            println!(
                "{:<12}  {:<9}  {:<10}  {:<10}  {}",
                t.id.bright_cyan(),
                state,
                t.flow.as_deref().unwrap_or("-"),
                run_short.bright_black(),
                preview
            );
        }
        println!();
        println!("Total: {} task(s)", queue.tasks.len());
        Ok(())
    }

    async fn queue_clear(&self, path: &std::path::Path) -> Result<()> {
        let mut queue = load_queue(path).await?;
        let before = queue.tasks.len();
        queue.tasks.retain(|t| t.state != "pending");
        let removed = before - queue.tasks.len();
        save_queue(path, &queue).await?;
        println!(
            "{} cleared {} pending task(s)",
            "OK".bright_green().bold(),
            removed
        );
        Ok(())
    }

    async fn queue_drain(
        &self,
        path: &std::path::Path,
        flow_override: Option<&str>,
        timeout: u64,
        fail_fast: bool,
        interactive: bool,
        create_pr: bool,
    ) -> Result<()> {
        let mut queue = load_queue(path).await?;
        let pending: Vec<usize> = queue
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == "pending")
            .map(|(i, _)| i)
            .collect();

        if pending.is_empty() {
            println!("No pending tasks to drain.");
            return Ok(());
        }

        // #40 fix: drain runs unattended by default. auto_commit flag is ON unless the
        // user explicitly asks for interactive per-task prompts. This closes the gap
        // between the "drain" verb and reality.
        let auto_commit = !interactive;
        println!(
            "{} draining {} pending task(s){}",
            "→".bright_cyan(),
            pending.len(),
            if interactive { " (interactive)" } else { "" }
        );

        let mut ok = 0usize;
        let mut ng = 0usize;

        for idx in pending {
            let task_id = queue.tasks[idx].id.clone();
            let task_body = queue.tasks[idx].task.clone();
            let flow = flow_override
                .map(String::from)
                .or_else(|| queue.tasks[idx].flow.clone())
                .unwrap_or_else(|| "default".to_string());

            println!();
            println!(
                "{} {} flow={} auto_commit={} create_pr={}",
                "▶".bright_cyan().bold(),
                task_id.bright_yellow(),
                flow.bright_white(),
                auto_commit,
                create_pr
            );

            queue.tasks[idx].state = "running".to_string();
            save_queue(path, &queue).await?;

            let result = self
                .handle_pipeline_returning_id(
                    &task_body,
                    &flow,
                    "text",
                    timeout,
                    false,
                    None,
                    false,
                    None,
                    None, // run_budget_tokens
                    None, // model override
                    auto_commit,
                    create_pr,
                )
                .await;

            match result {
                Ok(run_id) => {
                    queue.tasks[idx].state = "completed".to_string();
                    queue.tasks[idx].completed_at = Some(Utc::now());
                    // #43 fix: record run_id so `queue list` can show it.
                    queue.tasks[idx].run_id = Some(run_id.clone());
                    ok += 1;
                    println!(
                        "{} {} completed (run {})",
                        "✓".bright_green().bold(),
                        task_id.bright_yellow(),
                        run_id.chars().take(8).collect::<String>().bright_black()
                    );
                }
                Err(e) => {
                    queue.tasks[idx].state = "failed".to_string();
                    queue.tasks[idx].completed_at = Some(Utc::now());
                    ng += 1;
                    println!(
                        "{} {} failed: {}",
                        "✗".bright_red().bold(),
                        task_id.bright_yellow(),
                        e
                    );
                    save_queue(path, &queue).await?;
                    if fail_fast {
                        return Err(anyhow!(
                            "queue drain stopped: {} tasks ok, {} failed",
                            ok,
                            ng
                        ));
                    }
                }
            }
            save_queue(path, &queue).await?;
        }

        println!();
        println!(
            "{} queue drained — {} ok, {} failed",
            "✓".bright_green().bold(),
            ok,
            ng
        );
        Ok(())
    }
}
