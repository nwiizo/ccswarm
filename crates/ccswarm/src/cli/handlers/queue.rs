//! Task queue: append tasks to `.ccswarm/queue.yaml` and drain them through the pipeline.
//!
//! OK/NG-driven usage: user queues tasks throughout the day, then runs `ccswarm queue drain`
//! and says y/n at commit + PR time — no per-task babysitting.

use super::super::*;
use super::queue_state::{ClaimState, QUEUE_FILE, QueueState, QueueTask, load_queue};
use crate::run_id::validate_run_id;
use crate::tracker::{default_tracker_name, resolve_tracker};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;
use tracing::info;

type TrackerResolver = fn(&str) -> Result<Box<dyn crate::tracker::TrackerAdapter>>;

struct QueueAddTracker<'a> {
    name: &'a str,
    resolver: TrackerResolver,
}

fn now_ms() -> u64 {
    chrono::Utc::now().timestamp_millis().max(0) as u64
}

fn should_clear_task(task: &QueueTask) -> bool {
    task.state == "pending" && matches!(task.claim, ClaimState::Unclaimed)
}

#[derive(Debug, Serialize)]
pub struct ReconcileReport {
    pub released: usize,
    pub requeued: usize,
    pub kept_running: usize,
}

#[derive(Debug, Serialize)]
pub struct DispatchPlan {
    pub ready: usize,
}

#[derive(Debug, Serialize)]
struct ReconcileOnlyReport {
    active_runs: ReconcileReport,
    retry_queue: DispatchPlan,
}

struct QueueDrainOptions<'a> {
    flow_override: Option<&'a str>,
    timeout: u64,
    fail_fast: bool,
    interactive: bool,
    create_pr: bool,
    reconcile_only: bool,
    approval_gate: Option<std::time::Duration>,
}

pub async fn reconcile_active_runs(state: &QueueState, runs_dir: &Path) -> Result<ReconcileReport> {
    let queue = state.load().await?;
    let mut report = ReconcileReport {
        released: 0,
        requeued: 0,
        kept_running: 0,
    };

    for task in queue.tasks {
        let ClaimState::Running {
            run_id,
            started_at_ms,
        } = task.claim
        else {
            continue;
        };

        if let Some(terminal_status) = run_summary_terminal_status(runs_dir, &run_id).await? {
            if state
                .release_terminal_if_running(
                    &task.id,
                    &run_id,
                    started_at_ms,
                    terminal_status.queue_state(),
                    "terminal_state_reconciled",
                )
                .await?
            {
                report.released += 1;
            }
        } else {
            report.kept_running += 1;
        }
    }

    Ok(report)
}

pub async fn reconcile_retry_queue(state: &QueueState) -> Result<DispatchPlan> {
    let queue = state.load().await?;
    let now = now_ms();
    let mut plan = DispatchPlan { ready: 0 };

    for task in queue.tasks {
        let ClaimState::RetryQueued { attempt, due_at_ms } = task.claim else {
            continue;
        };

        if state
            .make_retry_ready_if_due(&task.id, attempt, due_at_ms, now)
            .await?
        {
            plan.ready += 1;
        }
    }

    Ok(plan)
}

async fn run_reconciliation_pass(
    state: &QueueState,
    runs_dir: &Path,
) -> Result<ReconcileOnlyReport> {
    let active_runs = reconcile_active_runs(state, runs_dir).await?;
    tracing::info!(report = ?active_runs, "reconciliation pass complete");
    let retry_queue = reconcile_retry_queue(state).await?;
    tracing::info!(report = ?retry_queue, "reconciliation pass complete");
    Ok(ReconcileOnlyReport {
        active_runs,
        retry_queue,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalRunStatus {
    Succeeded,
    Failed,
}

impl TerminalRunStatus {
    fn queue_state(self) -> &'static str {
        match self {
            Self::Succeeded => "completed",
            Self::Failed => "failed",
        }
    }
}

async fn run_summary_terminal_status(
    runs_dir: &Path,
    run_id: &str,
) -> Result<Option<TerminalRunStatus>> {
    validate_run_id(run_id).context("invalid run ID in queue claim")?;
    let summary_path = runs_dir.join(run_id).join("summary.json");
    if !summary_path.exists() {
        return Ok(None);
    }

    let raw = tokio::fs::read_to_string(&summary_path)
        .await
        .with_context(|| format!("Failed to read run summary {}", summary_path.display()))?;
    let summary: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse run summary {}", summary_path.display()))?;

    let explicit_status = summary.get("status").and_then(|status| status.as_str());
    match explicit_status {
        Some("succeeded") => return Ok(Some(TerminalRunStatus::Succeeded)),
        Some("failed") => return Ok(Some(TerminalRunStatus::Failed)),
        _ => {}
    }

    if summary
        .get("ended_at")
        .and_then(|ended_at| ended_at.as_str())
        .is_none()
    {
        return Ok(None);
    }

    if summary
        .get("tasks_failed")
        .and_then(|tasks_failed| tasks_failed.as_u64())
        .is_some_and(|tasks_failed| tasks_failed > 0)
    {
        Ok(Some(TerminalRunStatus::Failed))
    } else {
        Ok(Some(TerminalRunStatus::Succeeded))
    }
}

fn issue_to_queue_body(issue: crate::tracker::NormalizedIssue) -> String {
    format!(
        "{}\n\n{}",
        issue.title,
        issue.description.unwrap_or_default()
    )
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
                self.queue_add(
                    &path,
                    task,
                    from_issue.as_deref(),
                    file.as_deref(),
                    flow.as_deref(),
                )
                .await
            }
            QueueAction::List => self.queue_list(&path).await,
            QueueAction::Clear => self.queue_clear(&path).await,
            QueueAction::Release { id, reason } => self.queue_release(&path, id, reason).await,
            QueueAction::Drain {
                flow,
                timeout,
                fail_fast,
                interactive,
                create_pr,
                reconcile_only,
                require_approval,
                approval_timeout,
            } => {
                self.queue_drain(
                    &path,
                    QueueDrainOptions {
                        flow_override: flow.as_deref(),
                        timeout: *timeout,
                        fail_fast: *fail_fast,
                        interactive: *interactive,
                        create_pr: *create_pr,
                        reconcile_only: *reconcile_only,
                        approval_gate: require_approval
                            .then(|| std::time::Duration::from_secs(*approval_timeout)),
                    },
                )
                .await
            }
        }
    }

    async fn queue_add(
        &self,
        path: &std::path::Path,
        task: &str,
        from_issue: Option<&str>,
        file: Option<&std::path::Path>,
        flow: Option<&str>,
    ) -> Result<()> {
        let tracker_name = default_tracker_name();
        self.queue_add_with_tracker_resolver(
            path,
            task,
            from_issue,
            file,
            flow,
            QueueAddTracker {
                name: &tracker_name,
                resolver: resolve_tracker,
            },
        )
        .await
    }

    async fn queue_add_with_tracker_resolver(
        &self,
        path: &std::path::Path,
        task: &str,
        from_issue: Option<&str>,
        file: Option<&std::path::Path>,
        flow: Option<&str>,
        tracker: QueueAddTracker<'_>,
    ) -> Result<()> {
        // Input precedence: --from-issue > --file > `-` (stdin) > positional argument.
        // Allowing all four would be confusing; we error if >1 is given.
        let sources_count = [from_issue.is_some(), file.is_some(), task == "-"]
            .iter()
            .filter(|&&b| b)
            .count();
        if sources_count > 1 {
            return Err(anyhow!(
                "choose only one of: --from-issue <issue>, --file <path>, or `-` (stdin)"
            ));
        }

        let body = if let Some(identifier) = from_issue {
            let adapter = (tracker.resolver)(tracker.name)?;
            let issue = adapter.fetch_issue(identifier).await?;
            issue_to_queue_body(issue)
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
                "Provide a task description, --file <path>, --from-issue <issue>, or pass `-` to read stdin."
            ));
        } else {
            task.to_string()
        };

        let id = format!("q-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let entry = QueueTask {
            id: id.clone(),
            task: body.clone(),
            flow: flow.map(String::from),
            state: "pending".to_string(),
            created_at: Utc::now(),
            completed_at: None,
            run_id: None,
            claim: ClaimState::default(),
        };
        QueueState::new(path.to_path_buf())
            .update_queue(|queue| {
                queue.tasks.push(entry);
                Ok(())
            })
            .await?;
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
        let mut removed = 0usize;
        QueueState::new(path.to_path_buf())
            .update_queue(|queue| {
                let before = queue.tasks.len();
                queue.tasks.retain(|task| !should_clear_task(task));
                removed = before - queue.tasks.len();
                Ok(())
            })
            .await?;
        println!(
            "{} cleared {} pending task(s)",
            "OK".bright_green().bold(),
            removed
        );
        Ok(())
    }

    async fn queue_release(&self, path: &std::path::Path, id: &str, reason: &str) -> Result<()> {
        let state = QueueState::new(path.to_path_buf());
        state
            .update_task(id, |task| {
                task.claim = ClaimState::Released {
                    reason: reason.to_string(),
                };
                if task.state == "running" {
                    task.state = "pending".to_string();
                    task.completed_at = None;
                    task.run_id = None;
                }
                Ok(())
            })
            .await?;
        println!(
            "{} released {} ({})",
            "OK".bright_green().bold(),
            id.bright_cyan(),
            reason
        );
        Ok(())
    }

    async fn queue_drain(
        &self,
        path: &std::path::Path,
        options: QueueDrainOptions<'_>,
    ) -> Result<()> {
        let queue_state = QueueState::new(path.to_path_buf());
        let runs_dir = self.repo_path.join(".ccswarm").join("runs");
        let reconcile_report = run_reconciliation_pass(&queue_state, &runs_dir).await?;

        if options.reconcile_only {
            println!("{}", serde_json::to_string_pretty(&reconcile_report)?);
            return Ok(());
        }

        let queue = load_queue(path).await?;
        let pending: Vec<QueueTask> = queue
            .tasks
            .iter()
            .filter(|task| task.state == "pending")
            .cloned()
            .collect();

        if pending.is_empty() {
            println!("No pending tasks to drain.");
            return Ok(());
        }

        // #40 fix: drain runs unattended by default. auto_commit flag is ON unless the
        // user explicitly asks for interactive per-task prompts. This closes the gap
        // between the "drain" verb and reality.
        let auto_commit = !options.interactive;
        println!(
            "{} draining {} pending task(s){}",
            "→".bright_cyan(),
            pending.len(),
            if options.interactive {
                " (interactive)"
            } else {
                ""
            }
        );

        let mut ok = 0usize;
        let mut ng = 0usize;

        for queued_task in pending {
            let task_id = queued_task.id.clone();
            let task_body = queued_task.task.clone();
            let flow = options
                .flow_override
                .map(String::from)
                .or_else(|| queued_task.flow.clone())
                .unwrap_or_else(|| "default".to_string());
            let run_id = uuid::Uuid::new_v4().to_string();

            println!();
            println!(
                "{} {} flow={} auto_commit={} create_pr={}",
                "▶".bright_cyan().bold(),
                task_id.bright_yellow(),
                flow.bright_white(),
                auto_commit,
                options.create_pr
            );

            if !queue_state.try_claim(&task_id, &run_id).await? {
                info!(task_id = %task_id, "skipping: already claimed by another drain");
                continue;
            }

            if let Err(e) = queue_state
                .update_task(&task_id, |task| {
                    task.state = "running".to_string();
                    Ok(())
                })
                .await
            {
                queue_state
                    .release(&task_id, &format!("failed: {e}"))
                    .await?;
                return Err(e);
            }

            let result = self
                .handle_pipeline_returning_reserved_id(
                    &run_id,
                    &task_body,
                    &flow,
                    "text",
                    options.timeout,
                    false,
                    None,
                    false,
                    None,
                    None, // run_budget_tokens
                    None, // model override
                    auto_commit,
                    options.create_pr,
                    options.approval_gate,
                )
                .await;

            match result {
                Ok(run_id) => {
                    if let Err(e) = queue_state
                        .update_task(&task_id, |task| {
                            task.state = "completed".to_string();
                            task.completed_at = Some(Utc::now());
                            // #43 fix: record run_id so `queue list` can show it.
                            task.run_id = Some(run_id.clone());
                            Ok(())
                        })
                        .await
                    {
                        queue_state
                            .release(&task_id, &format!("failed: {e}"))
                            .await?;
                        return Err(e);
                    }
                    queue_state.release(&task_id, "completed").await?;
                    ok += 1;
                    println!(
                        "{} {} completed (run {})",
                        "✓".bright_green().bold(),
                        task_id.bright_yellow(),
                        run_id.chars().take(8).collect::<String>().bright_black()
                    );
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Err(update_error) = queue_state
                        .update_task(&task_id, |task| {
                            task.state = "failed".to_string();
                            task.completed_at = Some(Utc::now());
                            task.run_id = Some(run_id.clone());
                            Ok(())
                        })
                        .await
                    {
                        queue_state
                            .release(&task_id, &format!("failed: {update_error}"))
                            .await?;
                        return Err(update_error);
                    }
                    queue_state
                        .release(&task_id, &format!("failed: {error_message}"))
                        .await?;
                    ng += 1;
                    println!(
                        "{} {} failed: {}",
                        "✗".bright_red().bold(),
                        task_id.bright_yellow(),
                        error_message
                    );
                    if options.fail_fast {
                        return Err(anyhow!(
                            "queue drain stopped: {} tasks ok, {} failed",
                            ok,
                            ng
                        ));
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{NormalizedIssue, TrackerAdapter};
    use clap::Parser;

    fn queue_task(id: &str, claim: ClaimState) -> QueueTask {
        let state = match &claim {
            ClaimState::Running { .. } | ClaimState::RetryQueued { .. } => "running",
            ClaimState::Unclaimed | ClaimState::Released { .. } => "pending",
        };
        QueueTask {
            id: id.to_string(),
            task: "test task".to_string(),
            flow: None,
            state: state.to_string(),
            created_at: Utc::now(),
            completed_at: None,
            run_id: None,
            claim,
        }
    }

    struct FakeTracker;

    #[async_trait::async_trait]
    impl TrackerAdapter for FakeTracker {
        fn name(&self) -> &'static str {
            "fake"
        }

        async fn fetch_issue(&self, identifier: &str) -> Result<NormalizedIssue> {
            if identifier != "TEAM-42" {
                return Err(anyhow!("unexpected fake tracker identifier: {identifier}"));
            }

            Ok(NormalizedIssue {
                id: "fake-id".to_string(),
                identifier: identifier.to_string(),
                title: "Tracker title".to_string(),
                description: Some("Tracker body".to_string()),
                state: "OPEN".to_string(),
                labels: vec!["backend".to_string()],
                blocked_by: Vec::new(),
                url: "https://tracker.example/TEAM-42".to_string(),
                priority: None,
                updated_at_ms: Some(1_700_000_000_000),
            })
        }

        async fn list_active(&self, _limit: usize) -> Result<Vec<NormalizedIssue>> {
            Ok(Vec::new())
        }
    }

    fn fake_tracker_resolver(name: &str) -> Result<Box<dyn TrackerAdapter>> {
        if name == "fake" {
            Ok(Box::new(FakeTracker))
        } else {
            Err(anyhow!("unexpected fake tracker name: {name}"))
        }
    }

    async fn runner_for_repo(repo: &Path) -> Result<CliRunner> {
        let repo = repo
            .to_str()
            .context("temporary repository path is not UTF-8")?;
        let cli = Cli::try_parse_from(["ccswarm", "--repo", repo, "queue", "list"])?;
        CliRunner::new(&cli).await
    }

    async fn write_queue(path: &Path, tasks: Vec<QueueTask>) -> Result<()> {
        let state = QueueState::new(path.to_path_buf());
        state
            .update_queue(|queue| {
                queue.tasks = tasks;
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn test_queue_add_from_issue_uses_tracker_and_writes_task_body() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join(QUEUE_FILE);
        let runner = runner_for_repo(dir.path()).await?;

        runner
            .queue_add_with_tracker_resolver(
                &queue_path,
                "",
                Some("TEAM-42"),
                None,
                Some("review-fix"),
                QueueAddTracker {
                    name: "fake",
                    resolver: fake_tracker_resolver,
                },
            )
            .await?;

        let queue = load_queue(&queue_path).await?;
        assert_eq!(queue.tasks.len(), 1);
        assert_eq!(queue.tasks[0].task, "Tracker title\n\nTracker body");
        assert_eq!(queue.tasks[0].flow.as_deref(), Some("review-fix"));
        assert_eq!(queue.tasks[0].state, "pending");
        assert_eq!(queue.tasks[0].claim, ClaimState::Unclaimed);
        Ok(())
    }

    #[tokio::test]
    async fn test_queue_add_from_issue_linear_returns_not_implemented() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join(QUEUE_FILE);
        let runner = runner_for_repo(dir.path()).await?;

        let err = runner
            .queue_add_with_tracker_resolver(
                &queue_path,
                "",
                Some("ENG-123"),
                None,
                None,
                QueueAddTracker {
                    name: "linear",
                    resolver: resolve_tracker,
                },
            )
            .await
            .expect_err("linear tracker should fail until it is implemented");

        assert_eq!(
            err.to_string(),
            "LinearAdapter not yet implemented; set CCSWARM_TRACKER=github"
        );
        assert!(load_queue(&queue_path).await?.tasks.is_empty());
        Ok(())
    }

    #[test]
    fn test_queue_clear_only_removes_unclaimed_pending_tasks() {
        let mut unclaimed_pending = queue_task("q-1", ClaimState::Unclaimed);
        unclaimed_pending.state = "pending".to_string();

        let mut running_pending = queue_task(
            "q-2",
            ClaimState::Running {
                run_id: "run-1".to_string(),
                started_at_ms: now_ms(),
            },
        );
        running_pending.state = "pending".to_string();

        let mut released_pending = queue_task(
            "q-3",
            ClaimState::Released {
                reason: "manual".to_string(),
            },
        );
        released_pending.state = "pending".to_string();

        assert!(should_clear_task(&unclaimed_pending));
        assert!(!should_clear_task(&running_pending));
        assert!(!should_clear_task(&released_pending));
    }

    #[tokio::test]
    async fn test_reconcile_releases_terminal_runs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join("queue.yaml");
        let runs_dir = dir.path().join("runs");
        let run_id = "run-terminal";
        let run_dir = runs_dir.join(run_id);
        tokio::fs::create_dir_all(&run_dir).await?;
        tokio::fs::write(run_dir.join("summary.json"), r#"{"status":"succeeded"}"#).await?;

        write_queue(
            &queue_path,
            vec![queue_task(
                "q-1",
                ClaimState::Running {
                    run_id: run_id.to_string(),
                    started_at_ms: now_ms(),
                },
            )],
        )
        .await?;

        let state = QueueState::new(queue_path.clone());
        let report = reconcile_active_runs(&state, &runs_dir).await?;

        assert_eq!(report.released, 1);
        assert_eq!(report.requeued, 0);
        assert_eq!(report.kept_running, 0);

        let queue = load_queue(&queue_path).await?;
        assert_eq!(
            queue.tasks[0].claim,
            ClaimState::Released {
                reason: "terminal_state_reconciled".to_string()
            }
        );
        assert_eq!(queue.tasks[0].state, "completed");
        assert_eq!(queue.tasks[0].run_id.as_deref(), Some(run_id));
        assert!(queue.tasks[0].completed_at.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_marks_failed_terminal_runs_failed() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join("queue.yaml");
        let runs_dir = dir.path().join("runs");
        let run_id = "run-terminal-failed";
        let run_dir = runs_dir.join(run_id);
        tokio::fs::create_dir_all(&run_dir).await?;
        tokio::fs::write(run_dir.join("summary.json"), r#"{"status":"failed"}"#).await?;

        write_queue(
            &queue_path,
            vec![queue_task(
                "q-1",
                ClaimState::Running {
                    run_id: run_id.to_string(),
                    started_at_ms: now_ms(),
                },
            )],
        )
        .await?;

        let state = QueueState::new(queue_path.clone());
        let report = reconcile_active_runs(&state, &runs_dir).await?;

        assert_eq!(report.released, 1);
        let queue = load_queue(&queue_path).await?;
        assert_eq!(queue.tasks[0].state, "failed");
        assert_eq!(queue.tasks[0].run_id.as_deref(), Some(run_id));
        assert!(queue.tasks[0].completed_at.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_keeps_non_terminal_running_claims() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join("queue.yaml");
        let started_at_ms = now_ms().saturating_sub(7_200_000);

        write_queue(
            &queue_path,
            vec![queue_task(
                "q-1",
                ClaimState::Running {
                    run_id: "run-stuck".to_string(),
                    started_at_ms,
                },
            )],
        )
        .await?;

        let state = QueueState::new(queue_path.clone());
        let report = reconcile_active_runs(&state, &dir.path().join("runs")).await?;

        assert_eq!(report.released, 0);
        assert_eq!(report.requeued, 0);
        assert_eq!(report.kept_running, 1);

        let queue = load_queue(&queue_path).await?;
        assert!(matches!(
            queue.tasks[0].claim,
            ClaimState::Running { ref run_id, .. } if run_id == "run-stuck"
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_retry_queue_marks_due_task_pending() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join("queue.yaml");
        write_queue(
            &queue_path,
            vec![queue_task(
                "q-1",
                ClaimState::RetryQueued {
                    attempt: 1,
                    due_at_ms: now_ms().saturating_sub(1),
                },
            )],
        )
        .await?;

        let state = QueueState::new(queue_path.clone());
        let plan = reconcile_retry_queue(&state).await?;

        assert_eq!(plan.ready, 1);
        let queue = load_queue(&queue_path).await?;
        assert_eq!(queue.tasks[0].claim, ClaimState::Unclaimed);
        assert_eq!(queue.tasks[0].state, "pending");
        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_only_flag_does_not_dispatch() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let queue_path = dir.path().join(QUEUE_FILE);
        write_queue(&queue_path, vec![queue_task("q-1", ClaimState::Unclaimed)]).await?;

        let repo = dir
            .path()
            .to_str()
            .context("temporary repository path is not UTF-8")?;
        let cli = Cli::try_parse_from([
            "ccswarm",
            "--repo",
            repo,
            "queue",
            "drain",
            "--reconcile-only",
        ])?;
        let runner = CliRunner::new(&cli).await?;

        runner.run(&cli.command).await?;

        let queue = load_queue(&queue_path).await?;
        assert_eq!(queue.tasks[0].state, "pending");
        assert_eq!(queue.tasks[0].claim, ClaimState::Unclaimed);
        assert_eq!(queue.tasks[0].run_id, None);
        Ok(())
    }
}
