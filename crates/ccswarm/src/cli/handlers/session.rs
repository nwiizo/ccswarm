use super::super::*;
use crate::events::SessionInfo;

impl CliRunner {
    pub(crate) async fn handle_session(&self, action: &SessionAction) -> Result<()> {
        match action {
            SessionAction::List { all } => self.session_list(*all).await,
            SessionAction::View { id } => self.session_view(id).await,
            SessionAction::Create {
                agent,
                workspace,
                background,
            } => {
                self.session_create(agent, workspace.as_deref(), *background)
                    .await
            }
            SessionAction::Pause { session_id } => self.session_pause(session_id).await,
            SessionAction::Resume { session_id } => self.session_resume(session_id).await,
            SessionAction::Attach { session_id } => self.session_attach(session_id).await,
            SessionAction::Detach { session_id } => self.session_detach(session_id).await,
            SessionAction::Kill { session_id, force } => {
                self.session_kill(session_id, *force).await
            }
        }
    }

    /// Collect [`SessionInfo`] for every run directory under `.ccswarm/runs/`.
    ///
    /// For each directory:
    /// 1. If `summary.json` exists → build SessionInfo from summary, then
    ///    supplement with events.ndjson if available (task, movement fields).
    /// 2. Else if `events.ndjson` exists → build SessionInfo purely from events.
    /// 3. Else → skip unless `show_all` is true (empty directory → "empty" status).
    async fn collect_sessions(&self, show_all: bool) -> Result<Vec<SessionInfo>> {
        let runs_dir = self.repo_path.join(".ccswarm/runs");

        if !runs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions: Vec<SessionInfo> = Vec::new();
        let mut dir = tokio::fs::read_dir(&runs_dir).await?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let summary_path = path.join("summary.json");
            let events_path = path.join("events.ndjson");

            if summary_path.exists() {
                // Path 1: summary.json exists
                let content = tokio::fs::read_to_string(&summary_path).await?;
                if let Ok(summary_val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut info = SessionInfo::from_summary(&summary_val);

                    // Supplement with events data for task/movement fields
                    if events_path.exists()
                        && let Ok(events_content) = tokio::fs::read_to_string(&events_path).await
                    {
                        let events_info = SessionInfo::from_events(&dir_name, &events_content);
                        if info.task.is_none() {
                            info.task = events_info.task;
                        }
                        if info.last_movement.is_none() {
                            info.last_movement = events_info.last_movement;
                        }
                        if info.movements_completed == 0 {
                            info.movements_completed = events_info.movements_completed;
                        }
                    }
                    sessions.push(info);
                }
            } else if events_path.exists() {
                // Path 2: only events.ndjson exists (no summary)
                let events_content = tokio::fs::read_to_string(&events_path).await?;
                let info = SessionInfo::from_events(&dir_name, &events_content);

                // Skip empty event logs unless --all
                if info.total_events == 0 && !show_all {
                    continue;
                }
                sessions.push(info);
            } else if show_all {
                // Path 3: empty directory, only shown with --all
                sessions.push(SessionInfo {
                    run_id: dir_name,
                    started_at: None,
                    ended_at: None,
                    duration: None,
                    status: "empty".to_owned(),
                    total_events: 0,
                    task: None,
                    last_movement: None,
                    movements_completed: 0,
                    agents_used: Vec::new(),
                    has_errors: false,
                });
            }
        }

        // Sort by started_at descending (newest first). Sessions without
        // timestamps sort to the end.
        sessions.sort_by(|a, b| {
            let ts_a = a.started_at.as_ref();
            let ts_b = b.started_at.as_ref();
            match (ts_b, ts_a) {
                (Some(tb), Some(ta)) => tb.cmp(ta),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.run_id.cmp(&b.run_id),
            }
        });

        Ok(sessions)
    }

    async fn session_list(&self, show_all: bool) -> Result<()> {
        let sessions = self.collect_sessions(show_all).await?;

        if sessions.is_empty() {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "success",
                        "message": "No sessions found.",
                        "data": []
                    }))?
                );
            } else {
                println!("No sessions found.");
                println!(
                    "{}",
                    "Tip: Run a pipeline to create a session.".bright_black()
                );
            }
            return Ok(());
        }

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "data": sessions,
                }))?
            );
            return Ok(());
        }

        // ── Table header ────────────────────────────────────────────────
        println!(
            "{:<36}  {:<19}  {:<10}  {:>8}  {:>6}  {:<12}  {}",
            "Session ID".bright_cyan().bold(),
            "Date".bright_cyan().bold(),
            "Status".bright_cyan().bold(),
            "Duration".bright_cyan().bold(),
            "Events".bright_cyan().bold(),
            "Piece".bright_cyan().bold(),
            "Movement".bright_cyan().bold(),
        );
        println!("{}", "─".repeat(110).bright_black());

        for info in &sessions {
            let started_str = info
                .started_at
                .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "—".to_owned());

            let duration_str = info.duration.as_deref().unwrap_or("—");

            let status_colored = match info.status.as_str() {
                "completed" => info.status.as_str().bright_green(),
                "failed" => info.status.as_str().bright_red(),
                "running" => info.status.as_str().bright_yellow(),
                "empty" => info.status.as_str().bright_black(),
                _ => info.status.as_str().bright_white(),
            };

            let task_str = info.task.as_deref().unwrap_or("—");
            let movement_str = info.last_movement.as_deref().unwrap_or("—");

            println!(
                "{:<36}  {:<19}  {:<10}  {:>8}  {:>6}  {:<12}  {}",
                info.run_id.bright_yellow(),
                started_str,
                status_colored,
                duration_str,
                info.total_events,
                task_str,
                movement_str,
            );
        }

        println!();

        // ── Summary line ────────────────────────────────────────────────
        let completed = sessions.iter().filter(|s| s.status == "completed").count();
        let failed = sessions.iter().filter(|s| s.status == "failed").count();
        let running = sessions.iter().filter(|s| s.status == "running").count();

        print!(
            "Total: {} session(s)",
            sessions.len().to_string().bright_white().bold()
        );
        if completed > 0 {
            print!("  {} completed", completed.to_string().bright_green());
        }
        if failed > 0 {
            print!("  {} failed", failed.to_string().bright_red());
        }
        if running > 0 {
            print!("  {} running", running.to_string().bright_yellow());
        }
        println!();

        Ok(())
    }

    async fn session_view(&self, id: &str) -> Result<()> {
        let run_dir = self.repo_path.join(".ccswarm/runs").join(id);

        if !run_dir.exists() {
            anyhow::bail!("Session '{}' not found in .ccswarm/runs/", id);
        }

        let summary_path = run_dir.join("summary.json");
        let summary: Option<serde_json::Value> = if summary_path.exists() {
            let content = tokio::fs::read_to_string(&summary_path).await?;
            serde_json::from_str(&content).ok()
        } else {
            None
        };

        let events_path = run_dir.join("events.ndjson");
        let events: Vec<serde_json::Value> = if events_path.exists() {
            let content = tokio::fs::read_to_string(&events_path).await?;
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .collect()
        } else {
            Vec::new()
        };

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "data": {
                        "summary": summary,
                        "events": events,
                    }
                }))?
            );
            return Ok(());
        }

        println!(
            "{}  {}",
            "Session".bright_cyan().bold(),
            id.bright_yellow().bold()
        );
        println!("{}", "=".repeat(70).bright_black());

        if let Some(ref s) = summary {
            let started_at = s.get("started_at").and_then(|v| v.as_str()).unwrap_or("?");
            let ended_at = s
                .get("ended_at")
                .and_then(|v| v.as_str())
                .unwrap_or("in progress");
            let total_events = s.get("total_events").and_then(|v| v.as_u64()).unwrap_or(0);
            let tasks_completed = s
                .get("tasks_completed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let agents_used: Vec<&str> = s
                .get("agents_used")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|a| a.as_str()).collect())
                .unwrap_or_default();

            println!("  Started:          {}", started_at);
            println!("  Ended:            {}", ended_at);
            println!("  Total events:     {}", total_events);
            println!(
                "  Tasks completed:  {}",
                tasks_completed.to_string().bright_green()
            );
            if !agents_used.is_empty() {
                println!("  Agents:           {}", agents_used.join(", "));
            }
        } else {
            println!("  (no summary.json found)");
        }

        println!();
        println!("{}", "Events".bright_cyan().bold());
        println!("{}", "-".repeat(70).bright_black());

        if events.is_empty() {
            println!("  (no events found)");
        } else {
            for event in &events {
                let ts = event.get("ts").and_then(|v| v.as_str()).unwrap_or("?");
                let level = event
                    .get("level")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");
                let event_type = event
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let message = event.get("message").and_then(|v| v.as_str()).unwrap_or("");

                let level_colored = match level {
                    "error" => format!("{:<5}", level).bright_red(),
                    "warn" => format!("{:<5}", level).bright_yellow(),
                    "debug" => format!("{:<5}", level).bright_black(),
                    _ => format!("{:<5}", level).bright_white(),
                };

                let ts_short = if ts.len() >= 19 { &ts[..19] } else { ts };

                let duration_str = event
                    .get("metadata")
                    .and_then(|m| m.get("duration_ms"))
                    .and_then(|d| d.as_u64())
                    .map(|ms| format!(" ({}ms)", ms))
                    .unwrap_or_default();

                println!(
                    "  [{}] [{}] [{:<20}] {}{}",
                    ts_short.bright_black(),
                    level_colored,
                    event_type,
                    message,
                    duration_str.bright_black(),
                );
            }
        }

        println!();

        Ok(())
    }

    async fn session_create(
        &self,
        agent: &str,
        workspace: Option<&str>,
        background: bool,
    ) -> Result<()> {
        let valid_agents = ["frontend", "backend", "devops", "qa"];
        if !valid_agents.contains(&agent) {
            anyhow::bail!(
                "Invalid agent type '{}'. Must be one of: {}",
                agent,
                valid_agents.join(", ")
            );
        }

        let workspace_path = workspace
            .map(PathBuf::from)
            .unwrap_or_else(|| self.repo_path.clone());

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "message": "Session creation is handled by the pipeline engine.",
                    "agent": agent,
                    "workspace": workspace_path.display().to_string(),
                    "background": background,
                    "suggestion": "Use 'ccswarm pipeline' to execute workflows with sessions."
                }))?
            );
        } else {
            println!(
                "{} Session creation is handled by the pipeline engine.",
                "ℹ".bright_cyan()
            );
            println!("  Agent:     {}", agent.bright_yellow());
            println!(
                "  Workspace: {}",
                workspace_path.display().to_string().bright_white()
            );
            if background {
                println!("  Mode:      background");
            }
            println!();
            println!(
                "Use {} to execute workflows with automatic session management.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }

    async fn session_pause(&self, session_id: &str) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "session_id": session_id,
                    "message": "Session pause/resume is managed by the pipeline engine."
                }))?
            );
        } else {
            println!(
                "{} Session '{}' pause/resume is managed by the pipeline engine.",
                "ℹ".bright_cyan(),
                session_id.bright_yellow()
            );
            println!(
                "Use {} to manage running pipelines.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }

    async fn session_resume(&self, session_id: &str) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "session_id": session_id,
                    "message": "Session resume is managed by the pipeline engine."
                }))?
            );
        } else {
            println!(
                "{} Session '{}' resume is managed by the pipeline engine.",
                "ℹ".bright_cyan(),
                session_id.bright_yellow()
            );
            println!(
                "Use {} to manage running pipelines.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }

    async fn session_attach(&self, session_id: &str) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "session_id": session_id,
                    "message": "Session attach/detach is managed by the pipeline engine."
                }))?
            );
        } else {
            println!(
                "{} Session '{}' attach/detach is managed by the pipeline engine.",
                "ℹ".bright_cyan(),
                session_id.bright_yellow()
            );
            println!(
                "Use {} to manage running pipelines.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }

    async fn session_detach(&self, session_id: &str) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "session_id": session_id,
                    "message": "Session detach is managed by the pipeline engine."
                }))?
            );
        } else {
            println!(
                "{} Session '{}' detach is managed by the pipeline engine.",
                "ℹ".bright_cyan(),
                session_id.bright_yellow()
            );
            println!(
                "Use {} to manage running pipelines.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }

    async fn session_kill(&self, session_id: &str, force: bool) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "session_id": session_id,
                    "force": force,
                    "message": "Session termination is managed by the pipeline engine."
                }))?
            );
        } else {
            println!(
                "{} Session '{}' termination is managed by the pipeline engine.",
                "ℹ".bright_cyan(),
                session_id.bright_yellow()
            );
            if force {
                println!("  (force flag noted)");
            }
            println!(
                "Use {} to manage running pipelines.",
                "ccswarm pipeline".bright_cyan()
            );
        }

        Ok(())
    }
}
