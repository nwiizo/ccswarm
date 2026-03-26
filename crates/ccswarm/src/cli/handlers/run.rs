use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_run(&self, action: &RunAction) -> Result<()> {
        match action {
            RunAction::List => self.run_list().await,
            RunAction::View { id } => self.run_view(id).await,
        }
    }

    async fn run_list(&self) -> Result<()> {
        let runs_dir = self.repo_path.join(".ccswarm/runs");

        if !runs_dir.exists() {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "success",
                        "message": "No pipeline runs found.",
                        "data": []
                    }))?
                );
            } else {
                println!("No pipeline runs found.");
            }
            return Ok(());
        }

        // Collect summary entries from each run sub-directory.
        let mut summaries: Vec<serde_json::Value> = Vec::new();
        let mut dir = tokio::fs::read_dir(&runs_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let summary_path = path.join("summary.json");
            if !summary_path.exists() {
                continue;
            }
            let content = tokio::fs::read_to_string(&summary_path).await?;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                summaries.push(v);
            }
        }

        // Sort by started_at descending (newest first).
        summaries.sort_by(|a, b| {
            let ts_a = a.get("started_at").and_then(|v| v.as_str()).unwrap_or("");
            let ts_b = b.get("started_at").and_then(|v| v.as_str()).unwrap_or("");
            ts_b.cmp(ts_a)
        });

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "data": summaries,
                }))?
            );
            return Ok(());
        }

        if summaries.is_empty() {
            println!("No pipeline runs found.");
            return Ok(());
        }

        // Print table header.
        println!(
            "{:<36}  {:<25}  {:<10}  {:>10}  {:>6}  {}",
            "Run ID".bright_cyan().bold(),
            "Date".bright_cyan().bold(),
            "Status".bright_cyan().bold(),
            "Events".bright_cyan().bold(),
            "Tasks".bright_cyan().bold(),
            "Agents".bright_cyan().bold(),
        );
        println!("{}", "-".repeat(110).bright_black());

        for summary in &summaries {
            let run_id = summary
                .get("run_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let started_at = summary
                .get("started_at")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let ended_at = summary.get("ended_at").and_then(|v| v.as_str());
            let status = if ended_at.is_some() {
                "completed"
            } else {
                "running"
            };
            let total_events = summary
                .get("total_events")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let tasks_completed = summary
                .get("tasks_completed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let agents_used: Vec<&str> = summary
                .get("agents_used")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|a| a.as_str()).collect())
                .unwrap_or_default();

            let status_colored = match status {
                "completed" => status.bright_green(),
                _ => status.bright_yellow(),
            };

            println!(
                "{:<36}  {:<25}  {:<10}  {:>10}  {:>6}  {}",
                run_id.bright_yellow(),
                started_at,
                status_colored,
                total_events,
                tasks_completed,
                agents_used.join(", "),
            );
        }

        println!();
        println!("Total: {} run(s)", summaries.len());

        Ok(())
    }

    async fn run_view(&self, id: &str) -> Result<()> {
        let run_dir = self.repo_path.join(".ccswarm/runs").join(id);

        if !run_dir.exists() {
            anyhow::bail!("Run '{}' not found in .ccswarm/runs/", id);
        }

        // ── Summary ──────────────────────────────────────────────────────────
        let summary_path = run_dir.join("summary.json");
        let summary: Option<serde_json::Value> = if summary_path.exists() {
            let content = tokio::fs::read_to_string(&summary_path).await?;
            serde_json::from_str(&content).ok()
        } else {
            None
        };

        // ── Events ───────────────────────────────────────────────────────────
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

        // ── Human-readable output ────────────────────────────────────────────
        println!(
            "{}  {}",
            "Run".bright_cyan().bold(),
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
            let tasks_failed = s.get("tasks_failed").and_then(|v| v.as_u64()).unwrap_or(0);
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
            if tasks_failed > 0 {
                println!(
                    "  Tasks failed:     {}",
                    tasks_failed.to_string().bright_red()
                );
            }
            println!("  Agents:           {}", agents_used.join(", "));
        } else {
            println!("  (no summary.json found)");
        }

        // ── Events table ─────────────────────────────────────────────────────
        println!();
        println!("{}", "Events".bright_cyan().bold());
        println!("{}", "-".repeat(70).bright_black());

        if events.is_empty() {
            println!("  (no events.ndjson found)");
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

                let type_colored = match event_type {
                    t if t.starts_with("movement") => t.bright_magenta(),
                    t if t.starts_with("task") => t.bright_cyan(),
                    t if t.starts_with("hitl") => t.bright_yellow(),
                    t if t.starts_with("provider") => t.bright_blue(),
                    t => t.bright_white(),
                };

                // Trim timestamp to first 19 chars (YYYY-MM-DDTHH:MM:SS) to keep lines short.
                let ts_short = if ts.len() >= 19 { &ts[..19] } else { ts };

                // Optionally show movement duration from metadata.
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
                    type_colored,
                    message,
                    duration_str.bright_black(),
                );
            }
        }

        println!();

        Ok(())
    }
}
