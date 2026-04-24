//! `ccswarm auto` — autonomous execution loop.
//!
//! Pulls tasks from the `.ccswarm/queue.yaml` file (or takes a single `--task`), runs them
//! through the pipeline with `--auto-commit` enabled, optionally creates a PR, and repeats.
//! All y/n prompts are suppressed. Decisions are emitted as events so the operator can
//! inspect what the loop did via `ccswarm tail` / `ccswarm runs list`.

use super::super::*;
use super::queue_state::{QUEUE_FILE, load_queue, save_queue};
use chrono::Utc;
use std::path::Path;

const AUTO_LOG: &str = ".ccswarm/auto.ndjson";

/// Outcome signal carried between loop iterations. The `Stop` variant carries a reason
/// string that is currently unused at the call site but is kept to avoid silently losing
/// diagnostic context when the loop exits.
enum LoopSignal {
    Continue,
    Stop(#[allow(dead_code)] String),
}

impl CliRunner {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn handle_auto(
        &self,
        explicit_task: Option<&str>,
        flow: &str,
        watch: bool,
        poll_secs: u64,
        max_iterations: usize,
        wall_budget_secs: u64,
        stop_on_error: bool,
        timeout: u64,
        create_pr: bool,
    ) -> Result<()> {
        let banner = format!(
            "▶ auto mode — flow={} watch={} stop_on_error={} timeout={}s create_pr={}",
            flow, watch, stop_on_error, timeout, create_pr
        );
        println!("{}", banner.bright_cyan().bold());
        self.auto_log("auto.start", serde_json::json!({ "flow": flow }))
            .await;

        let deadline = if wall_budget_secs > 0 {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(wall_budget_secs))
        } else {
            None
        };

        let mut processed: usize = 0;
        let mut ok: usize = 0;
        let mut ng: usize = 0;

        // Path 1: a single explicit task bypasses the queue.
        if let Some(task_body) = explicit_task {
            let (outcome, _run_id) = self
                .auto_run_one("direct", task_body, flow, timeout, create_pr)
                .await;
            processed += 1;
            match outcome {
                Ok(()) => ok += 1,
                Err(e) => {
                    ng += 1;
                    eprintln!("{} {}", "✗".bright_red().bold(), e);
                }
            }
            self.auto_summary_line(ok, ng, processed).await;
            return Ok(());
        }

        // Path 2: drain the queue, optionally watching for more.
        let queue_path = self.repo_path.join(QUEUE_FILE);

        loop {
            if let Some(dl) = deadline
                && std::time::Instant::now() >= dl
            {
                println!(
                    "{} wall-clock budget reached ({}s)",
                    "■".bright_yellow().bold(),
                    wall_budget_secs
                );
                break;
            }

            let signal = self
                .auto_drain_once(
                    &queue_path,
                    flow,
                    timeout,
                    create_pr,
                    stop_on_error,
                    max_iterations,
                    &mut processed,
                    &mut ok,
                    &mut ng,
                )
                .await?;

            if matches!(signal, LoopSignal::Stop(_)) {
                break;
            }

            if !watch {
                break;
            }

            println!(
                "{} queue empty — sleeping {}s before next poll",
                "…".bright_black(),
                poll_secs
            );
            tokio::time::sleep(std::time::Duration::from_secs(poll_secs)).await;
        }

        self.auto_summary_line(ok, ng, processed).await;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn auto_drain_once(
        &self,
        queue_path: &Path,
        piece_default: &str,
        timeout: u64,
        create_pr: bool,
        stop_on_error: bool,
        max_iterations: usize,
        processed: &mut usize,
        ok: &mut usize,
        ng: &mut usize,
    ) -> Result<LoopSignal> {
        let mut queue = load_queue(queue_path).await?;
        let pending_positions: Vec<usize> = queue
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == "pending")
            .map(|(i, _)| i)
            .collect();

        if pending_positions.is_empty() {
            return Ok(LoopSignal::Continue);
        }

        for idx in pending_positions {
            if max_iterations > 0 && *processed >= max_iterations {
                return Ok(LoopSignal::Stop("max iterations reached".to_string()));
            }

            let task_id = queue.tasks[idx].id.clone();
            let task_body = queue.tasks[idx].task.clone();
            let flow_name = queue.tasks[idx]
                .flow
                .clone()
                .unwrap_or_else(|| piece_default.to_string());

            queue.tasks[idx].state = "running".to_string();
            save_queue(queue_path, &queue).await?;

            let (outcome, run_id) = self
                .auto_run_one(&task_id, &task_body, &flow_name, timeout, create_pr)
                .await;
            *processed += 1;

            // Record run_id regardless of outcome so operators can jump from the
            // queue file to the events.ndjson even for failed tasks.
            queue.tasks[idx].run_id = run_id;

            match outcome {
                Ok(()) => {
                    queue.tasks[idx].state = "completed".to_string();
                    queue.tasks[idx].completed_at = Some(Utc::now());
                    *ok += 1;
                }
                Err(e) => {
                    queue.tasks[idx].state = "failed".to_string();
                    queue.tasks[idx].completed_at = Some(Utc::now());
                    *ng += 1;
                    eprintln!(
                        "{} {} failed: {}",
                        "✗".bright_red().bold(),
                        task_id.bright_yellow(),
                        e
                    );
                    save_queue(queue_path, &queue).await?;
                    if stop_on_error {
                        return Ok(LoopSignal::Stop(format!("task {} failed", task_id)));
                    }
                }
            }
            save_queue(queue_path, &queue).await?;
        }

        Ok(LoopSignal::Continue)
    }

    /// Execute a single task through the pipeline. Returns `(result, Option<run_id>)`
    /// so the caller can record the run_id back into `QueueTask.run_id` — this closes
    /// codex #6 (auto.ndjson had no cross-reference to run events.ndjson).
    async fn auto_run_one(
        &self,
        task_id: &str,
        task_body: &str,
        flow_name: &str,
        timeout: u64,
        create_pr: bool,
    ) -> (Result<()>, Option<String>) {
        println!();
        println!(
            "{} {} flow={} auto_commit=true create_pr={}",
            "▶".bright_cyan().bold(),
            task_id.bright_yellow(),
            flow_name.bright_white(),
            create_pr
        );
        self.auto_log(
            "auto.task_start",
            serde_json::json!({
                "task_id": task_id,
                "flow": flow_name,
                "create_pr": create_pr,
            }),
        )
        .await;

        let result = self
            .handle_pipeline_returning_id(
                task_body, flow_name, "text", timeout, false, None, false, None,
                None, // run_budget_tokens
                None, // model override
                /* auto_commit = */ true, create_pr,
            )
            .await;

        match &result {
            Ok(run_id) => {
                println!(
                    "{} {} completed (run {})",
                    "✓".bright_green().bold(),
                    task_id.bright_yellow(),
                    run_id.chars().take(8).collect::<String>().bright_black()
                );
                self.auto_log(
                    "auto.task_end",
                    serde_json::json!({
                        "task_id": task_id,
                        "run_id": run_id,
                        "status": "completed",
                    }),
                )
                .await;
                (Ok(()), Some(run_id.clone()))
            }
            Err(e) => {
                self.auto_log(
                    "auto.task_end",
                    serde_json::json!({
                        "task_id": task_id,
                        "status": "failed",
                        "error": e.to_string(),
                    }),
                )
                .await;
                (Err(anyhow!("{}", e)), None)
            }
        }
    }

    async fn auto_summary_line(&self, ok: usize, ng: usize, processed: usize) {
        println!();
        println!(
            "{} auto session done — processed={} ok={} failed={}",
            "■".bright_green().bold(),
            processed,
            ok,
            ng
        );
        self.auto_log(
            "auto.end",
            serde_json::json!({
                "processed": processed,
                "ok": ok,
                "failed": ng,
            }),
        )
        .await;
    }

    /// Best-effort append to `.ccswarm/auto.ndjson`. Silent on any error — this is for
    /// observability, not correctness.
    async fn auto_log(&self, kind: &str, payload: serde_json::Value) {
        let log_path = self.repo_path.join(AUTO_LOG);
        if let Some(parent) = log_path.parent()
            && tokio::fs::create_dir_all(parent).await.is_err()
        {
            return;
        }
        let line = serde_json::json!({
            "ts": Utc::now().to_rfc3339(),
            "kind": kind,
            "payload": payload,
        });
        let body = format!("{}\n", line);
        // Single synchronous open+append. The loop is not hot — one entry per task
        // boundary — so the blocking cost is negligible and avoids the previous double
        // open (tokio+std) that was dead code.
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = f.write_all(body.as_bytes());
        }
    }
}
