//! `undo` (advisory) and `replay` (re-run task) handlers.
//!
//! Both operate on a past run under `.ccswarm/runs/<run-id>/`. `undo` is intentionally
//! advisory: it prints git commits that landed after the run started, so the user can
//! pick what to revert themselves. `replay` extracts the original task from summary.json
//! and feeds it back into the pipeline.

use super::super::*;
use super::run_utils::{read_summary, resolve_run_path};

impl CliRunner {
    pub(crate) async fn handle_undo(&self, run_id: Option<&str>) -> Result<()> {
        let run_path = resolve_run_path(&self.repo_path, run_id).await?;
        let summary = read_summary(&run_path)?;

        let started_at = summary
            .get("started_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let task = summary.get("task").and_then(|v| v.as_str()).unwrap_or("");

        println!(
            "{} {}",
            "Run:".bright_black(),
            run_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
                .bright_white()
        );
        if !task.is_empty() {
            println!("{} {}", "Task:".bright_black(), task);
        }
        if !started_at.is_empty() {
            println!("{} {}", "Started:".bright_black(), started_at);
        }
        println!();

        if started_at.is_empty() {
            println!(
                "{}",
                "Cannot locate commits (started_at missing). Inspect .ccswarm/runs/<id>/ manually."
                    .bright_yellow()
            );
            return Ok(());
        }

        // code #3 fix: validate that started_at looks like an ISO-8601 timestamp before
        // passing it to `git log --since=` so a crafted summary.json cannot smuggle flags.
        if !looks_like_iso8601(started_at) {
            println!(
                "{}",
                format!(
                    "Refusing to run `git log` — started_at='{}' does not look like ISO-8601.",
                    started_at
                )
                .bright_yellow()
            );
            return Ok(());
        }

        println!("{}", "Commits since run started".bright_cyan().bold());
        println!("{}", "--------------------------".bright_cyan());
        let output = tokio::process::Command::new("git")
            .args([
                "log",
                &format!("--since={}", started_at),
                "--pretty=format:%h  %s  (%an, %ar)",
            ])
            .current_dir(&self.repo_path)
            .output()
            .await;
        match output {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout);
                if s.trim().is_empty() {
                    println!(
                        "  {}",
                        "(no commits since run started — nothing to revert)".bright_black()
                    );
                } else {
                    for line in s.lines() {
                        println!("  {}", line);
                    }
                    println!();
                    println!("{}", "To revert a commit:".bright_cyan().bold());
                    println!("  git revert <hash>");
                    println!();
                    println!(
                        "{}",
                        "(ccswarm never revises history for you — copy the command above.)"
                            .bright_black()
                    );
                }
            }
            _ => println!(
                "{}",
                "(`git log` failed; is this a git repo?)".bright_yellow()
            ),
        }
        Ok(())
    }

    pub(crate) async fn handle_replay(
        &self,
        run_id: Option<&str>,
        piece_override: Option<&str>,
        timeout: u64,
    ) -> Result<()> {
        let run_path = resolve_run_path(&self.repo_path, run_id).await?;
        let summary = read_summary(&run_path)?;

        let original_task = summary
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!(
                    "Run summary has no 'task' field — cannot replay. Use `ccswarm run view` to inspect."
                )
            })?
            .to_string();

        let flow_name = piece_override
            .map(String::from)
            .or_else(|| {
                summary
                    .get("flow")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| "default".to_string());

        println!(
            "{} replaying task from {}",
            "↻".bright_cyan().bold(),
            run_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
                .bright_white()
        );
        println!("  Flow: {}", flow_name.bright_green());
        println!(
            "  Task:  {}",
            original_task.lines().next().unwrap_or("").trim()
        );
        println!();

        self.handle_pipeline(
            &original_task,
            &flow_name,
            "text",
            timeout,
            false,
            None,
            false,
            None,
            None, // run_budget_tokens
            None, // model override
            false,
            false,
        )
        .await
    }
}

/// Loose ISO-8601 shape check: starts with `YYYY-MM-DD` and does not contain a `;`, `|`,
/// `\n`, or starts with `-`. Designed to reject `--exec=foo` style smuggling, not to be a
/// full parser.
fn looks_like_iso8601(s: &str) -> bool {
    if s.is_empty() || s.starts_with('-') || s.contains([';', '|', '\n']) {
        return false;
    }
    let bytes = s.as_bytes();
    bytes.len() >= 10
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3].is_ascii_digit()
        && bytes[4] == b'-'
        && bytes[5].is_ascii_digit()
        && bytes[6].is_ascii_digit()
        && bytes[7] == b'-'
        && bytes[8].is_ascii_digit()
        && bytes[9].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_iso8601() {
        assert!(looks_like_iso8601("2026-04-17T15:18:06.513521Z"));
        assert!(looks_like_iso8601("2026-04-17"));
    }

    #[test]
    fn rejects_smuggling() {
        assert!(!looks_like_iso8601("--exec=rm -rf /"));
        assert!(!looks_like_iso8601("2026-04-17; rm -rf /"));
        assert!(!looks_like_iso8601(""));
        assert!(!looks_like_iso8601("yesterday"));
    }
}
