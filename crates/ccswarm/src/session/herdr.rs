//! Herdr execution for local provider CLIs.
//!
//! ccswarm runs every local provider call inside a background Herdr tab so
//! the work stays visible in the user's persistent terminal workspace. See
//! `docs/HERDR_INTEGRATION.md` for the product contract.
//!
//! - Headless calls run the provider's non-interactive command through a
//!   short `/bin/sh` script; stdout, stderr, and the exit code come back
//!   through a private run directory, never from terminal scraping.
//! - Live calls start an interactive Claude Code session with
//!   `herdr agent start`, submit the prompt with `herdr agent prompt`, and
//!   read the result from the session transcript.
//!
//! Dropping a call (for example on a stage timeout) closes its tab, which
//! stops the provider, and removes its run directory.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::process::Command;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
/// How long a headless tab's shell may take to begin the submitted script.
const HEADLESS_START_TIMEOUT: Duration = Duration::from_secs(30);
/// How often to confirm the pane or agent still exists while a call runs.
const LIVENESS_INTERVAL: Duration = Duration::from_secs(2);
/// How long Claude may take to write the transcript after the prompt.
const LIVE_PROMPT_TIMEOUT: Duration = Duration::from_secs(120);
/// `herdr agent start` readiness timeout in milliseconds.
const LIVE_START_TIMEOUT_MS: &str = "60000";
/// How long a submitted live prompt may take to make Claude start working.
const PROMPT_START_TIMEOUT_MS: &str = "20000";
/// `pane report-agent` / `report-metadata` source identity.
const REPORT_SOURCE: &str = "ccswarm";

/// How a stage's provider runs inside Herdr.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Non-interactive provider command with file-based results.
    #[default]
    Headless,
    /// Interactive Claude Code session the user can watch and steer.
    Live,
}

impl ExecutionMode {
    pub fn is_headless(&self) -> bool {
        *self == Self::Headless
    }
}

/// Identity of one provider call, used for the tab and Herdr reports.
#[derive(Debug, Clone)]
pub struct HerdrCall {
    /// Stage or agent identifier shown as the `$stage` token.
    pub stage: String,
    /// Provider name shown as the `$provider` token.
    pub provider: String,
}

impl HerdrCall {
    fn tab_label(&self) -> String {
        format!("ccswarm {} ({})", self.stage, self.provider)
    }
}

/// Result of a headless provider call.
#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub stdout: String,
    pub stderr: String,
    /// Exit code reported by the shell that ran the provider (128 + signal
    /// number when the provider was killed by a signal).
    pub exit_code: i32,
    pub duration_ms: u64,
}

impl CommandExecution {
    /// Whether the provider exited with status 0.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Inputs for a live Claude Code call.
#[derive(Debug, Clone)]
pub struct LiveClaudeRequest {
    pub call: HerdrCall,
    /// Prompt submitted as the user's message.
    pub prompt: String,
    /// Persona text passed with `--append-system-prompt-file`.
    pub system_prompt: Option<String>,
    pub model: Option<String>,
    /// Tool names for `--allowed-tools` (already provider-formatted).
    pub allowed_tools: Option<String>,
    pub agent_name: Option<String>,
    pub worktree_name: Option<String>,
}

/// Result of a live Claude Code call.
#[derive(Debug, Clone)]
pub struct LiveTurn {
    /// Claude session ID; `claude --resume <id>` reopens it.
    pub session_id: String,
    /// Assistant text after the latest user message.
    pub text: String,
    /// Input tokens including cache reads/writes, and output tokens.
    pub tokens: Option<(u64, u64)>,
    pub duration_ms: u64,
}

/// The Herdr session that hosts provider calls.
#[derive(Debug, Clone)]
pub struct HerdrContext {
    program: PathBuf,
    workspace_id: String,
}

impl HerdrContext {
    /// Use the Herdr session that manages the current pane.
    ///
    /// Fails outside a Herdr-managed pane, which local provider execution
    /// requires.
    pub fn from_env() -> Result<Self> {
        let inside_herdr = std::env::var("HERDR_ENV").is_ok_and(|value| value == "1");
        let workspace_id = non_empty_env("HERDR_WORKSPACE_ID");
        match (inside_herdr, workspace_id) {
            (true, Some(workspace_id)) => Ok(Self::new(herdr_program(), workspace_id)),
            _ => bail!(
                "ccswarm runs local provider CLIs in Herdr, but this process is not inside a \
                 Herdr-managed pane (HERDR_ENV=1 and HERDR_WORKSPACE_ID are required). Start \
                 `herdr`, run ccswarm from one of its panes, and check `ccswarm doctor`. \
                 Remote A2A execution (CCSWARM_A2A_ENDPOINT) does not need Herdr."
            ),
        }
    }

    /// Use an explicit `herdr` executable and workspace.
    pub fn new(program: impl Into<PathBuf>, workspace_id: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            workspace_id: workspace_id.into(),
        }
    }

    /// Run a non-interactive provider command in a new background tab.
    pub async fn run_headless(
        &self,
        command: &Command,
        working_dir: &Path,
        call: &HerdrCall,
    ) -> Result<CommandExecution> {
        let working_dir = &absolute_dir(working_dir)?;
        let run_dir = RunDir::create()?;
        std::fs::write(
            run_dir.file("run.sh"),
            render_script(command, working_dir, &run_dir.0, call),
        )
        .context("Failed to write the Herdr run script")?;

        let tab = self.open_tab(working_dir, call).await?;
        let agent = reported_agent_name(&call.stage);
        self.report(
            &tab.pane_id,
            &[
                "report-agent",
                "--agent",
                &agent,
                "--state",
                "working",
                "--message",
                &call.tab_label(),
            ],
        )
        .await;
        self.report_tokens(&tab.pane_id, call).await;

        let start = Instant::now();
        // Leading space: kept out of history by shells that support it.
        let line = format!(
            " /bin/sh {}",
            shell_quote(run_dir.file("run.sh").as_os_str())
        );
        explain(
            self.herdr(["pane", "run", tab.pane_id.as_str(), line.as_str()])
                .await,
            || format!("Failed to start {} in Herdr", call.tab_label()),
        )?;

        self.wait_for_headless(&run_dir, &tab.pane_id, call).await?;
        let exit_code = read_lossy(&run_dir.file("status"))?
            .trim()
            .parse::<i32>()
            .context("Herdr run wrote an invalid exit status")?;
        Ok(CommandExecution {
            stdout: read_lossy(&run_dir.file("stdout"))?,
            stderr: read_lossy(&run_dir.file("stderr"))?,
            exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Run one turn of an interactive Claude Code session in a new tab.
    pub async fn run_live_claude(
        &self,
        request: &LiveClaudeRequest,
        working_dir: &Path,
        claude_config_dir: &Path,
    ) -> Result<LiveTurn> {
        let working_dir = &absolute_dir(working_dir)?;
        let run_dir = RunDir::create()?;
        let session_id = uuid::Uuid::new_v4().to_string();
        let system_prompt_file = match &request.system_prompt {
            Some(prompt) if !prompt.trim().is_empty() => {
                let path = run_dir.file("system-prompt.md");
                std::fs::write(&path, prompt).context("Failed to write the persona prompt")?;
                Some(path)
            }
            _ => None,
        };
        let args = live_claude_args(request, &session_id, system_prompt_file.as_deref())?;
        let agent = live_agent_name(&request.call.stage);

        let tab = self.open_tab(working_dir, &request.call).await?;
        let mut start_args = vec![
            "agent".to_string(),
            "start".to_string(),
            agent.clone(),
            "--kind".to_string(),
            "claude".to_string(),
            "--pane".to_string(),
            tab.pane_id.clone(),
            "--timeout".to_string(),
            LIVE_START_TIMEOUT_MS.to_string(),
            "--".to_string(),
        ];
        start_args.extend(args);
        match self.herdr(&start_args).await {
            Ok(_) => {}
            // Claude is waiting at a startup dialog such as folder trust.
            // That decision belongs to the user, so wait for it in Herdr.
            Err(error) if error.to_string().contains("agent_not_ready") => {
                self.notify(
                    "ccswarm needs input",
                    &format!(
                        "Claude for {} is waiting in Herdr",
                        request.call.tab_label()
                    ),
                )
                .await;
                self.wait_until_ready(&agent, &request.call).await?;
            }
            Err(error) => bail!(
                "Failed to start Claude for {}: {error:#}",
                request.call.tab_label()
            ),
        }
        self.report_tokens(&tab.pane_id, &request.call).await;

        let start = Instant::now();
        self.submit_prompt(&agent, &request.prompt, &request.call)
            .await?;

        let transcript = self
            .wait_for_live(&agent, &session_id, claude_config_dir, &request.call)
            .await?;
        Ok(LiveTurn {
            session_id,
            text: transcript.text,
            tokens: transcript.tokens,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Show a Herdr notification; failures are logged, never returned.
    pub async fn notify(&self, title: &str, body: &str) {
        if let Err(error) = self
            .herdr(["notification", "show", title, "--body", body])
            .await
        {
            tracing::debug!("Herdr notification failed: {error:#}");
        }
    }

    async fn open_tab(&self, working_dir: &Path, call: &HerdrCall) -> Result<OpenTab> {
        let created = explain(
            self.herdr([
                OsStr::new("tab"),
                OsStr::new("create"),
                OsStr::new("--workspace"),
                OsStr::new(&self.workspace_id),
                OsStr::new("--cwd"),
                working_dir.as_os_str(),
                OsStr::new("--label"),
                OsStr::new(&call.tab_label()),
                OsStr::new("--no-focus"),
            ])
            .await,
            || "Failed to open a Herdr tab for the provider".to_string(),
        )?;
        let (tab_id, pane_id) = parse_tab_create(&created)?;
        Ok(OpenTab {
            program: self.program.clone(),
            tab_id,
            pane_id,
        })
    }

    async fn wait_for_headless(
        &self,
        run_dir: &RunDir,
        pane_id: &str,
        call: &HerdrCall,
    ) -> Result<()> {
        let start = Instant::now();
        let mut last_check = Instant::now();
        loop {
            if run_dir.file("status").exists() {
                return Ok(());
            }
            if !run_dir.file("started").exists() && start.elapsed() > HEADLESS_START_TIMEOUT {
                bail!(
                    "{} did not start in Herdr pane {pane_id} within {}s",
                    call.tab_label(),
                    HEADLESS_START_TIMEOUT.as_secs()
                );
            }
            if last_check.elapsed() > LIVENESS_INTERVAL {
                last_check = Instant::now();
                explain(self.herdr(["pane", "get", pane_id]).await, || {
                    format!(
                        "Herdr pane {pane_id} closed before {} finished",
                        call.tab_label()
                    )
                })?;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Submit the stage prompt and wait until Claude starts working on it.
    ///
    /// Right after `agent start` reports ready, Claude Code can accept the
    /// pasted text but drop the Enter that follows (observed with Claude Code
    /// 2.1.319 in Herdr 0.9.5): the prompt stays in the input box. Herdr then
    /// reports a stall or timeout. The text is already there, so ccswarm
    /// presses Enter once instead of pasting the prompt again.
    async fn submit_prompt(&self, agent: &str, prompt: &str, call: &HerdrCall) -> Result<()> {
        let until_started = [
            "--until",
            "working",
            "--until",
            "blocked",
            "--timeout",
            PROMPT_START_TIMEOUT_MS,
        ];
        // `agent prompt` accepts `--until` only together with `--wait`.
        let mut submit = vec!["agent", "prompt", agent, prompt, "--wait"];
        submit.extend(until_started);
        let error = match self.herdr(&submit).await {
            Ok(_) => return Ok(()),
            Err(error) => error,
        };
        let text = format!("{error:#}");
        if !(text.contains("agent_prompt_stalled") || text.contains("timeout")) {
            bail!("Failed to submit the prompt to {agent}: {text}");
        }

        tracing::info!("Prompt for {agent} did not start; pressing Enter once to submit it");
        explain(
            self.herdr(["agent", "send-keys", agent, "enter"]).await,
            || format!("Failed to submit the prompt to {agent}"),
        )?;
        let mut wait = vec!["agent", "wait", agent];
        wait.extend(until_started);
        explain(self.herdr(&wait).await, || {
            format!(
                "Claude did not start the prompt for {}; inspect its Herdr tab",
                call.tab_label()
            )
        })?;
        Ok(())
    }

    /// Wait until a live agent blocked at startup becomes ready for input.
    async fn wait_until_ready(&self, agent: &str, call: &HerdrCall) -> Result<()> {
        loop {
            let output = self.herdr(["agent", "get", agent]).await.map_err(|error| {
                anyhow::anyhow!(
                    "Claude for {} exited before it was ready (a startup prompt may have been declined): {error:#}",
                    call.tab_label()
                )
            })?;
            if matches!(
                parse_agent_status(&output).as_deref(),
                Some("idle" | "done")
            ) {
                return Ok(());
            }
            tokio::time::sleep(LIVENESS_INTERVAL).await;
        }
    }

    async fn wait_for_live(
        &self,
        agent: &str,
        session_id: &str,
        claude_config_dir: &Path,
        call: &HerdrCall,
    ) -> Result<TranscriptTurn> {
        let start = Instant::now();
        let mut last_check: Option<Instant> = None;
        let mut status = String::new();
        let mut notified_blocked = false;
        loop {
            let transcript = find_transcript(claude_config_dir, session_id)
                .map(|path| read_lossy(&path))
                .transpose()?;
            if transcript.is_none() && start.elapsed() > LIVE_PROMPT_TIMEOUT {
                bail!(
                    "Claude did not record session {session_id} for {} within {}s; inspect the Herdr tab",
                    call.tab_label(),
                    LIVE_PROMPT_TIMEOUT.as_secs()
                );
            }
            if last_check.is_none_or(|checked| checked.elapsed() >= LIVENESS_INTERVAL) {
                last_check = Some(Instant::now());
                let output = explain(self.herdr(["agent", "get", agent]).await, || {
                    format!(
                        "Claude session for {} exited before finishing",
                        call.tab_label()
                    )
                })?;
                status = parse_agent_status(&output).unwrap_or_default();
                if status == "blocked" && !notified_blocked {
                    notified_blocked = true;
                    self.notify(
                        "ccswarm needs input",
                        &format!("{} is waiting in Herdr", call.tab_label()),
                    )
                    .await;
                } else if status != "blocked" {
                    notified_blocked = false;
                }
            }
            if let Some(transcript) = transcript {
                let turn = parse_transcript(&transcript);
                if turn.complete && (status == "idle" || status == "done") {
                    return Ok(turn);
                }
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Best-effort `herdr pane <command> <pane> --source ccswarm ...`.
    async fn report(&self, pane_id: &str, args: &[&str]) {
        let seq = next_seq().to_string();
        let mut full = vec![
            "pane",
            args[0],
            pane_id,
            "--source",
            REPORT_SOURCE,
            "--seq",
            &seq,
        ];
        full.extend_from_slice(&args[1..]);
        if let Err(error) = self.herdr(&full).await {
            tracing::warn!("Herdr {} failed for pane {pane_id}: {error:#}", args[0]);
        }
    }

    async fn report_tokens(&self, pane_id: &str, call: &HerdrCall) {
        let stage = format!("stage={}", call.stage);
        let provider = format!("provider={}", call.provider);
        self.report(
            pane_id,
            &[
                "report-metadata",
                "--title",
                &call.tab_label(),
                "--token",
                &stage,
                "--token",
                &provider,
            ],
        )
        .await;
    }

    async fn herdr<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new(&self.program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .await
            .with_context(|| format!("Failed to execute {}", self.program.display()))?;
        if !output.status.success() {
            bail!(
                "herdr exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// Prefix an error with `message` while keeping its cause in the text, since
/// callers log errors with `{}`, which omits `anyhow` context chains.
fn explain<T>(result: Result<T>, message: impl FnOnce() -> String) -> Result<T> {
    result.map_err(|error| anyhow::anyhow!("{}: {error:#}", message()))
}

/// Absolute form of `path` relative to this process's current directory.
///
/// Herdr resolves relative `--cwd` values against its server, which may
/// run elsewhere, so every directory handed to Herdr must be absolute.
pub fn absolute_dir(path: &Path) -> Result<PathBuf> {
    std::path::absolute(path)
        .with_context(|| format!("Cannot resolve working directory {}", path.display()))
}

/// `HERDR_BIN_PATH` when Herdr provides it, otherwise `herdr` on `PATH`.
pub fn herdr_program() -> PathBuf {
    non_empty_env("HERDR_BIN_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("herdr"))
}

/// Show a Herdr notification when running inside Herdr; otherwise do nothing.
pub async fn notify(title: &str, body: &str) {
    if let Ok(herdr) = HerdrContext::from_env() {
        herdr.notify(title, body).await;
    }
}

/// Claude's configuration directory (`CLAUDE_CONFIG_DIR` or `~/.claude`).
pub fn claude_config_dir() -> Result<PathBuf> {
    if let Some(dir) = non_empty_env("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::home_dir()
        .map(|home| home.join(".claude"))
        .context("Cannot locate the home directory for Claude transcripts")
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Strictly increasing report sequence based on wall-clock milliseconds, as
/// Herdr ignores reports whose `seq` is not newer than the last accepted one.
fn next_seq() -> u64 {
    static LAST: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0);
    let mut last = LAST.load(Ordering::Relaxed);
    loop {
        let next = now.max(last + 1);
        match LAST.compare_exchange(last, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(current) => last = current,
        }
    }
}

/// Display name for a headless stage (`pane report-agent --agent`).
fn reported_agent_name(stage: &str) -> String {
    format!("ccswarm-{}", agent_slug(stage, 24))
}

/// Unique live agent name matching Herdr's `[a-z][a-z0-9_-]{0,31}`.
fn live_agent_name(stage: &str) -> String {
    let suffix = &uuid::Uuid::new_v4().simple().to_string()[..6];
    format!("cs-{}-{suffix}", agent_slug(stage, 22))
}

fn agent_slug(stage: &str, max_len: usize) -> String {
    let slug: String = stage
        .chars()
        .map(|c| match c.to_ascii_lowercase() {
            c @ ('a'..='z' | '0'..='9' | '_' | '-') => c,
            _ => '-',
        })
        .take(max_len)
        .collect();
    if slug.is_empty() {
        "stage".to_string()
    } else {
        slug
    }
}

/// Claude Code arguments for a live session.
///
/// Herdr types the start command into the pane's interactive shell, so each
/// argument must be a plain shell word; the persona travels in a file.
fn live_claude_args(
    request: &LiveClaudeRequest,
    session_id: &str,
    system_prompt_file: Option<&Path>,
) -> Result<Vec<String>> {
    let mut args = vec![
        "--session-id".to_string(),
        session_id.to_string(),
        "--dangerously-skip-permissions".to_string(),
    ];
    let flags = [
        ("--agent", request.agent_name.clone()),
        ("--allowed-tools", request.allowed_tools.clone()),
        ("--model", request.model.clone()),
        (
            "--append-system-prompt-file",
            system_prompt_file.map(|path| path.to_string_lossy().into_owned()),
        ),
        ("--worktree", request.worktree_name.clone()),
    ];
    for (flag, value) in flags {
        let Some(value) = value else { continue };
        if !is_plain_shell_word(&value) {
            bail!(
                "live execution cannot pass {flag} {value:?} through the Herdr pane shell; \
                 use plain names or execution: headless"
            );
        }
        args.push(flag.to_string());
        args.push(value);
    }
    Ok(args)
}

fn is_plain_shell_word(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.,/:=@+".contains(c))
}

/// Extract `(tab_id, root pane_id)` from `herdr tab create` JSON output.
fn parse_tab_create(output: &str) -> Result<(String, String)> {
    let value: serde_json::Value =
        serde_json::from_str(output).context("herdr tab create returned invalid JSON")?;
    let field = |pointer: &str| {
        value
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .with_context(|| format!("herdr tab create response is missing {pointer}"))
    };
    Ok((
        field("/result/tab/tab_id")?,
        field("/result/root_pane/pane_id")?,
    ))
}

/// Extract the lifecycle status from `herdr agent get` JSON output.
fn parse_agent_status(output: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(output).ok()?;
    value
        .pointer("/result/agent/agent_status")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// POSIX shell script that runs the provider and records its results.
///
/// The provider's stdout is shown in the pane and copied to a file; the
/// status file appears only after `tee` has flushed, so a present status
/// means the output files are complete.
fn render_script(
    command: &Command,
    working_dir: &Path,
    run_dir: &Path,
    call: &HerdrCall,
) -> String {
    let std_command = command.as_std();
    let file = |name: &str| shell_quote(run_dir.join(name).as_os_str());

    let mut provider = format!("cd {}", shell_quote(working_dir.as_os_str()));
    for (key, value) in std_command.get_envs() {
        let key = key.to_string_lossy();
        match value {
            Some(value) => provider.push_str(&format!(" && export {key}={}", shell_quote(value))),
            None => provider.push_str(&format!(" && unset {key}")),
        }
    }
    provider.push_str(" && exec ");
    provider.push_str(&shell_quote(std_command.get_program()));
    for arg in std_command.get_args() {
        provider.push(' ');
        provider.push_str(&shell_quote(arg));
    }

    format!(
        ": > {started}\n\
         printf '%s\\n' {banner}\n\
         {{ ( {provider} ) < /dev/null 2> {stderr}; echo $? > {status_tmp}; }} | tee {stdout}\n\
         mv {status_tmp} {status}\n",
        started = file("started"),
        banner = shell_quote(OsStr::new(&format!(
            "{}: {}",
            call.tab_label(),
            working_dir.display()
        ))),
        stderr = file("stderr"),
        stdout = file("stdout"),
        status_tmp = file("status.tmp"),
        status = file("status"),
    )
}

/// Quote a value as one POSIX shell word.
fn shell_quote(value: &OsStr) -> String {
    format!("'{}'", value.to_string_lossy().replace('\'', r"'\''"))
}

fn read_lossy(path: &Path) -> Result<String> {
    let bytes =
        std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Locate `<config>/projects/*/<session_id>.jsonl`.
fn find_transcript(claude_config_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let file_name = format!("{session_id}.jsonl");
    std::fs::read_dir(claude_config_dir.join("projects"))
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().join(&file_name))
        .find(|path| path.is_file())
}

/// The latest turn of a Claude Code session transcript.
#[derive(Debug, Clone, Default, PartialEq)]
struct TranscriptTurn {
    /// Assistant text after the latest user message.
    text: String,
    /// Session usage totals: input (with cache reads/writes) and output.
    tokens: Option<(u64, u64)>,
    /// Whether a `turn_duration` entry follows the latest user message.
    complete: bool,
}

#[derive(Debug, Deserialize)]
struct TranscriptEntry {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default, rename = "isMeta")]
    is_meta: bool,
    #[serde(default)]
    message: Option<TranscriptMessage>,
}

#[derive(Debug, Deserialize)]
struct TranscriptMessage {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    content: serde_json::Value,
    #[serde(default)]
    usage: Option<TranscriptUsage>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct TranscriptUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
}

impl TranscriptEntry {
    /// A message typed by the user or submitted by ccswarm, as opposed to
    /// tool results and meta entries that Claude also records as `user`.
    fn is_user_prompt(&self) -> bool {
        if self.kind != "user" || self.is_meta {
            return false;
        }
        match self.message.as_ref().map(|message| &message.content) {
            Some(serde_json::Value::String(_)) => true,
            Some(serde_json::Value::Array(blocks)) => blocks
                .iter()
                .any(|block| block.get("type").and_then(|t| t.as_str()) == Some("text")),
            _ => false,
        }
    }

    fn assistant_text(&self) -> Option<(String, String)> {
        let message = self.message.as_ref()?;
        let blocks = message.content.as_array()?;
        let text: String = blocks
            .iter()
            .filter(|block| block.get("type").and_then(|t| t.as_str()) == Some("text"))
            .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
            .collect();
        Some((message.id.clone().unwrap_or_default(), text))
    }
}

/// Parse a session transcript. Unparseable lines (such as a partially
/// written final line) are skipped.
fn parse_transcript(content: &str) -> TranscriptTurn {
    let entries: Vec<TranscriptEntry> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let mut usage_by_message: HashMap<String, TranscriptUsage> = HashMap::new();
    for entry in entries.iter().filter(|entry| entry.kind == "assistant") {
        if let Some(message) = &entry.message
            && let (Some(id), Some(usage)) = (&message.id, message.usage)
        {
            usage_by_message.insert(id.clone(), usage);
        }
    }
    let tokens = (!usage_by_message.is_empty()).then(|| {
        usage_by_message
            .values()
            .fold((0, 0), |(input, output), usage| {
                (
                    input
                        + usage.input_tokens
                        + usage.cache_read_input_tokens
                        + usage.cache_creation_input_tokens,
                    output + usage.output_tokens,
                )
            })
    });

    let turn_start = entries
        .iter()
        .rposition(TranscriptEntry::is_user_prompt)
        .map(|index| index + 1)
        .unwrap_or(0);
    let turn = &entries[turn_start..];

    // Claude records one entry per content block, repeating the message ID;
    // join blocks within a message and separate messages by a blank line.
    let mut messages: Vec<(String, String)> = Vec::new();
    for (id, text) in turn
        .iter()
        .filter(|entry| entry.kind == "assistant")
        .filter_map(TranscriptEntry::assistant_text)
    {
        match messages.last_mut() {
            Some((last_id, last_text)) if !id.is_empty() && *last_id == id => {
                last_text.push_str(&text)
            }
            _ => messages.push((id, text)),
        }
    }
    let text = messages
        .into_iter()
        .map(|(_, text)| text)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    TranscriptTurn {
        text,
        tokens,
        complete: turn_start > 0
            && turn.iter().any(|entry| {
                entry.kind == "system" && entry.subtype.as_deref() == Some("turn_duration")
            }),
    }
}

/// Private directory holding one call's files; removed on drop.
struct RunDir(PathBuf);

impl RunDir {
    fn create() -> Result<Self> {
        let path = std::env::temp_dir().join(format!("ccswarm-herdr-{}", uuid::Uuid::new_v4()));
        // Paths reach the pane's interactive shell inside single quotes, and
        // some shells (Fish) treat backslashes there as escapes.
        if !is_plain_shell_word(&path.to_string_lossy()) {
            bail!(
                "temporary directory {} contains characters that cannot be passed to a Herdr pane",
                path.display()
            );
        }
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder
            .create(&path)
            .with_context(|| format!("Failed to create {}", path.display()))?;
        Ok(Self(path))
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Herdr tab owned by one provider call; closed on drop, which also stops a
/// provider that is still running after cancellation or an error.
struct OpenTab {
    program: PathBuf,
    tab_id: String,
    pane_id: String,
}

impl Drop for OpenTab {
    fn drop(&mut self) {
        let closed = std::process::Command::new(&self.program)
            .args(["tab", "close", &self.tab_id])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if !closed.is_ok_and(|status| status.success()) {
            tracing::warn!("Failed to close Herdr tab {}", self.tab_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call() -> HerdrCall {
        HerdrCall {
            stage: "Implement/Coder".to_string(),
            provider: "claude".to_string(),
        }
    }

    #[test]
    fn shell_quote_preserves_single_quotes() {
        assert_eq!(shell_quote(OsStr::new("it's")), r"'it'\''s'");
    }

    #[test]
    fn agent_names_follow_herdr_rules() {
        assert_eq!(
            reported_agent_name("Implement/Coder"),
            "ccswarm-implement-coder"
        );
        let live = live_agent_name("A very long Sangha member identifier!");
        assert!(live.len() <= 32, "{live}");
        assert!(live.starts_with("cs-a-very-long-sangha-me"));
        assert!(
            live.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        );
        assert_ne!(live_agent_name("x"), live_agent_name("x"));
    }

    #[test]
    fn report_sequence_strictly_increases() {
        let first = next_seq();
        assert!(next_seq() > first);
    }

    #[test]
    fn parse_herdr_responses() {
        let output = r#"{"result":{"tab":{"tab_id":"w1:t3"},"root_pane":{"pane_id":"w1:p7"}}}"#;
        assert_eq!(
            parse_tab_create(output).unwrap(),
            ("w1:t3".to_string(), "w1:p7".to_string())
        );
        assert!(parse_tab_create(r#"{"result":{}}"#).is_err());
        assert_eq!(
            parse_agent_status(r#"{"result":{"agent":{"agent_status":"blocked"}}}"#).as_deref(),
            Some("blocked")
        );
    }

    #[test]
    fn script_runs_provider_with_quoted_arguments_and_env_changes() {
        let mut command = Command::new("claude");
        command
            .args(["-p", "fix 'this'"])
            .env("CCSWARM_TEST", "a b")
            .env_remove("CLAUDECODE");
        let script = render_script(&command, Path::new("/work dir"), Path::new("/run"), &call());
        assert!(script.contains("cd '/work dir'"));
        assert!(script.contains("export CCSWARM_TEST='a b'"));
        assert!(script.contains("unset CLAUDECODE"));
        assert!(script.contains(r"exec 'claude' '-p' 'fix '\''this'\'''"));
        assert!(script.contains("< /dev/null 2> '/run/stderr'"));
        assert!(script.ends_with("mv '/run/status.tmp' '/run/status'\n"));
    }

    #[test]
    fn live_args_reject_values_the_pane_shell_would_reinterpret() {
        let mut request = LiveClaudeRequest {
            call: call(),
            prompt: "review".to_string(),
            system_prompt: None,
            model: Some("sonnet".to_string()),
            allowed_tools: Some("Read,Grep".to_string()),
            agent_name: None,
            worktree_name: None,
        };
        let args =
            live_claude_args(&request, "sid", Some(Path::new("/run/system-prompt.md"))).unwrap();
        assert_eq!(
            args,
            [
                "--session-id",
                "sid",
                "--dangerously-skip-permissions",
                "--allowed-tools",
                "Read,Grep",
                "--model",
                "sonnet",
                "--append-system-prompt-file",
                "/run/system-prompt.md",
            ]
        );

        request.allowed_tools = Some("Bash(git status)".to_string());
        assert!(live_claude_args(&request, "sid", None).is_err());
    }

    #[test]
    fn transcript_turn_uses_text_after_latest_user_message() {
        let transcript = [
            r#"{"type":"user","message":{"content":"first task"}}"#,
            r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"text","text":"old"}],"usage":{"input_tokens":2,"output_tokens":5,"cache_read_input_tokens":10}}}"#,
            r#"{"type":"system","subtype":"turn_duration"}"#,
            r#"{"type":"user","isMeta":true,"message":{"content":"caveat"}}"#,
            r#"{"type":"user","message":{"content":[{"type":"text","text":"steer"}]}}"#,
            r#"{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","name":"Read"}],"usage":{"input_tokens":1,"output_tokens":1}}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"file"}]}}"#,
            r#"{"type":"assistant","message":{"id":"m3","content":[{"type":"text","text":"LINE1"}],"usage":{"input_tokens":3,"output_tokens":7,"cache_creation_input_tokens":4}}}"#,
            r#"{"type":"assistant","message":{"id":"m3","content":[{"type":"text","text":"\nVERDICT: APPROVE"}],"usage":{"input_tokens":3,"output_tokens":7,"cache_creation_input_tokens":4}}}"#,
        ]
        .join("\n");

        let running = parse_transcript(&transcript);
        assert_eq!(running.text, "LINE1\nVERDICT: APPROVE");
        assert!(!running.complete);
        assert_eq!(running.tokens, Some((2 + 10 + 1 + 3 + 4, 5 + 1 + 7)));

        let finished = parse_transcript(&format!(
            "{transcript}\n{}\n{{\"type\":\"assis",
            r#"{"type":"system","subtype":"turn_duration"}"#
        ));
        assert!(finished.complete);
        assert_eq!(finished.text, "LINE1\nVERDICT: APPROVE");
    }

    #[test]
    fn transcript_without_user_message_is_incomplete() {
        let turn = parse_transcript(r#"{"type":"system","subtype":"turn_duration"}"#);
        assert!(!turn.complete);
        assert_eq!(turn.tokens, None);
    }

    /// Fake `herdr` that runs submitted commands in the background, like a
    /// pane shell, and logs every invocation.
    #[cfg(unix)]
    fn fake_herdr(dir: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let program = dir.join("herdr");
        let script = format!(
            r#"#!/bin/sh
echo "$*" >> '{log}'
case "$1 $2" in
  "tab create") echo '{{"result":{{"tab":{{"tab_id":"w1:t2"}},"root_pane":{{"pane_id":"w1:p2"}}}}}}' ;;
  "pane run") sh -c "$4" > /dev/null 2>&1 & ;;
  "pane get"|"pane report-agent"|"pane report-metadata"|"tab close") echo '{{}}' ;;
  *) echo "unexpected: $*" >&2; exit 1 ;;
esac
"#,
            log = dir.join("herdr.log").display()
        );
        std::fs::write(&program, script).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        program
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn headless_run_collects_results_reports_state_and_closes_the_tab() {
        let dir = tempfile::tempdir().unwrap();
        let herdr = HerdrContext::new(fake_herdr(dir.path()), "w1");
        let mut command = Command::new("sh");
        command.args(["-c", "echo \"out $(pwd -P)\"; echo oops >&2; exit 3"]);

        let execution = herdr
            .run_headless(&command, dir.path(), &call())
            .await
            .unwrap();

        let cwd = dir.path().canonicalize().unwrap();
        assert_eq!(execution.stdout.trim(), format!("out {}", cwd.display()));
        assert_eq!(execution.stderr.trim(), "oops");
        assert_eq!(execution.exit_code, 3);
        assert!(!execution.success());

        let log = std::fs::read_to_string(dir.path().join("herdr.log")).unwrap();
        let calls: Vec<&str> = log.lines().collect();
        assert!(calls[0].starts_with("tab create --workspace w1 --cwd "));
        assert!(calls[0].ends_with("--label ccswarm Implement/Coder (claude) --no-focus"));
        assert!(calls[1].starts_with("pane report-agent w1:p2 --source ccswarm --seq "));
        assert!(calls[1].contains("--agent ccswarm-implement-coder --state working"));
        assert!(calls[2].contains("--token stage=Implement/Coder --token provider=claude"));
        assert!(calls[3].starts_with("pane run w1:p2  /bin/sh '"));
        assert_eq!(calls.last(), Some(&"tab close w1:t2"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn headless_run_sends_herdr_an_absolute_working_directory() {
        let dir = tempfile::tempdir().unwrap();
        let herdr = HerdrContext::new(fake_herdr(dir.path()), "w1");
        let mut command = Command::new("sh");
        command.args(["-c", "pwd"]);

        let execution = herdr
            .run_headless(&command, Path::new("."), &call())
            .await
            .unwrap();

        let expected = std::env::current_dir().unwrap();
        assert_eq!(execution.stdout.trim(), expected.display().to_string());
        let log = std::fs::read_to_string(dir.path().join("herdr.log")).unwrap();
        assert!(
            log.contains(&format!("--cwd {} --label", expected.display())),
            "{log}"
        );
    }
}
