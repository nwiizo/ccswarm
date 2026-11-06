//! A2ABridge: A2A or local provider execution with retry and rate-limit
//! fallback.
//!
//! Provider-specific command construction lives in the providers module and
//! local execution in Herdr (`session::herdr`). The bridge selects the route,
//! retries, parses output, and reports token usage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::providers::{ProviderKind, ProviderOptions, WorkspaceAccess};
use crate::session::execution::{
    DEFAULT_MAX_PROMPT_BYTES, prepare_provider_prompt, run_provider_command,
};
use crate::session::herdr::{ExecutionMode, HerdrCall, HerdrContext, LiveClaudeRequest};
use crate::session::output::{BuildStatus, LogLevel, OutputParser, ParsedOutput};
use crate::session::types::AttentionState;

/// Result of an A2ABridge execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResult {
    /// Provider output text.
    pub raw: String,
    /// Parsed output from ccswarm's semantic parser.
    pub parsed: ParsedOutput,
    /// Whether the execution was successful (based on parsed output)
    pub success: bool,
    /// Execution duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
    /// Input tokens for the call: real counts (including cache reads/writes)
    /// from Claude stream-json, Codex JSONL, or a live transcript; otherwise a
    /// bytes/4 estimate so `ccswarm cost` still has relative stage weights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_in: Option<u64>,
    /// Output tokens for the call (real or bytes/4, as for `tokens_in`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_out: Option<u64>,
    /// Attention triage state derived from the parsed output. Falls back to
    /// `Done` on success / `Error` on failure when the parser has no decisive
    /// signal.
    #[serde(default)]
    pub attention: AttentionState,
    /// Tool names invoked during the call, parsed from Claude stream-json
    /// output (empty unless `CCSWARM_CLAUDE_STREAM_JSON=1`). Tool inputs are
    /// deliberately not propagated — they can embed entire file contents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_names: Vec<String>,
    /// Total cost in USD as reported by the provider's result envelope
    /// (stream-json only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// Providers that were rate-limited and skipped before this result was
    /// produced (empty when the first provider answered). The engine records
    /// these as ProviderError events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks_used: Vec<String>,
}

/// Heuristic rate-limit detection on provider CLI failure text. Provider CLIs
/// don't expose structured error kinds, so match the common phrasings.
fn is_rate_limit_error(err: &anyhow::Error) -> bool {
    let text = err.to_string().to_lowercase();
    [
        "rate limit",
        "rate_limit",
        "429",
        "too many requests",
        "quota",
        "overloaded",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

/// Switch `options` to a fallback provider after a rate limit. Returns the
/// name of the provider being abandoned.
fn apply_rate_limit_fallback(
    options: &mut MovementExecOptions,
    (next_provider, next_model): (ProviderKind, Option<String>),
) -> String {
    let from = options
        .provider
        .unwrap_or(ProviderKind::Claude)
        .as_str()
        .to_string();
    options.provider = Some(next_provider);
    if next_model.is_some() {
        options.model = next_model;
    }
    from
}

/// Refuse a rate-limit fallback that would drop an enforced read-only
/// sandbox: a read-only stage on Codex must not silently continue on a
/// provider that can write to the workspace.
fn check_fallback_access(options: &MovementExecOptions, next: ProviderKind) -> Result<(), String> {
    let current = options.provider.unwrap_or(ProviderKind::Claude);
    let read_only = options.workspace_access == WorkspaceAccess::ReadOnly;
    if read_only && current.enforces_read_only() && !next.enforces_read_only() {
        return Err(format!(
            "refusing rate-limit fallback from '{}' to '{}': the stage requires a \
             read-only workspace and '{}' cannot enforce it",
            current.as_str(),
            next.as_str(),
            next.as_str()
        ));
    }
    Ok(())
}

/// What the retry loop does after a provider's attempts are exhausted.
#[derive(Debug, PartialEq)]
enum FallbackStep {
    /// Return the last error.
    Stop,
    /// Return the rate-limit error with this refusal as context.
    Refuse(String),
    /// Continue on the next `on_rate_limit` target.
    Switch((ProviderKind, Option<String>)),
}

/// Decide the next step after a failed provider: switch only on a rate
/// limit with a remaining fallback target, and refuse a target that would
/// drop an enforced read-only sandbox.
fn next_fallback(
    options: &MovementExecOptions,
    fallback_index: usize,
    last_err: Option<&anyhow::Error>,
) -> FallbackStep {
    let Some(target) = options.rate_limit_fallbacks.get(fallback_index) else {
        return FallbackStep::Stop;
    };
    if !last_err.is_some_and(is_rate_limit_error) {
        return FallbackStep::Stop;
    }
    match check_fallback_access(options, target.0) {
        Ok(()) => FallbackStep::Switch(target.clone()),
        Err(refusal) => FallbackStep::Refuse(refusal),
    }
}

/// Prompt prefix telling the fallback provider it's picking up mid-task.
fn fallback_notice_prompt(original_prompt: &str) -> String {
    format!(
        "# Provider fallback notice\nA previous provider hit a rate limit mid-task. \
         You are taking over fresh — no prior session context is available beyond this prompt.\n\n{}",
        original_prompt
    )
}

/// The stage's A2A endpoint, else `CCSWARM_A2A_ENDPOINT`; blank values count
/// as unset.
fn resolve_a2a_endpoint(options: &MovementExecOptions) -> Option<String> {
    let non_blank = |value: &str| Some(value.trim().to_string()).filter(|v| !v.is_empty());
    options
        .a2a_endpoint
        .as_deref()
        .and_then(non_blank)
        .or_else(|| {
            std::env::var("CCSWARM_A2A_ENDPOINT")
                .ok()
                .as_deref()
                .and_then(non_blank)
        })
}

fn prepare_a2a_prompt(prompt: &str, system_prompt: Option<&str>) -> String {
    match system_prompt.filter(|value| !value.trim().is_empty()) {
        Some(system_prompt) => format!(
            "# System instructions\n{}\n\n{}",
            system_prompt.trim(),
            prompt
        ),
        None => prompt.to_string(),
    }
}

fn env_flag(key: &str) -> bool {
    std::env::var(key).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Projection of a structured provider stdout buffer down to the pieces the
/// bridge propagates.
struct StreamProjection {
    /// Final answer text (`result.result`, or concatenated assistant text).
    text: String,
    /// Tool names in invocation order.
    tool_names: Vec<String>,
    /// Real token totals `(input incl. cache, output)` from the last usage
    /// record — the result envelope's usage is cumulative for the run, so
    /// summing all records would double-count the per-turn entries.
    tokens: Option<(u64, u64)>,
    total_cost_usd: Option<f64>,
}

fn project_stream(raw_stdout: &str) -> StreamProjection {
    let summary = crate::providers::claude_stream::parse_stream(raw_stdout);
    let tokens = summary.usage.last().map(|u| {
        (
            u.input_tokens + u.cache_creation_input_tokens + u.cache_read_input_tokens,
            u.output_tokens,
        )
    });
    StreamProjection {
        text: summary.result_text,
        tool_names: summary.tool_uses.into_iter().map(|t| t.name).collect(),
        tokens,
        total_cost_usd: summary.total_cost_usd,
    }
}

/// Options for a single stage execution. Provider-specific flag mapping happens in
/// the providers module; unsupported fields are silently ignored by providers that don't
/// understand them, keeping flow YAML portable.
#[derive(Debug, Clone, Default)]
pub struct MovementExecOptions {
    /// Provider to use for this stage (default: Claude).
    pub(crate) provider: Option<ProviderKind>,
    /// Tools to allow (mapped to --allowed-tools on Claude).
    pub tools: Vec<String>,
    /// Model override.
    pub model: Option<String>,
    /// System prompt from persona.
    pub system_prompt: Option<String>,
    /// Budget limit in USD (Claude only).
    pub max_budget: Option<f64>,
    /// Execute in isolated worktree (Claude only).
    pub worktree_name: Option<String>,
    /// Optional A2A server endpoint. If absent, `CCSWARM_A2A_ENDPOINT` is used
    /// when set; otherwise the selected local provider CLI is executed.
    pub a2a_endpoint: Option<String>,
    /// Providers to fall back to (in order) when this stage's provider hits a
    /// rate limit. Each entry is `(provider, optional model override)`. Comes
    /// from the flow-level `on_rate_limit` YAML field.
    pub(crate) rate_limit_fallbacks: Vec<(ProviderKind, Option<String>)>,
    /// How the local provider runs in its Herdr tab. `Live` applies to Claude;
    /// other providers run headless.
    pub execution: ExecutionMode,
    /// Workspace access the stage requires, from its `permission` level.
    /// Codex enforces `ReadOnly` with its sandbox, and a rate-limit fallback
    /// to a provider that cannot enforce it is refused.
    pub(crate) workspace_access: WorkspaceAccess,
}

/// A2A/local provider execution layer.
#[derive(Default)]
pub struct A2ABridge {
    output_parser: OutputParser,
}

impl A2ABridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute with retry logic and exponential backoff.
    ///
    /// When the provider fails with a rate-limit error and
    /// `options.rate_limit_fallbacks` is non-empty, the call switches to the
    /// next provider in the chain instead of burning retries on the limited
    /// one. The switch resets the retry budget and prepends a fallback notice
    /// so the model knows it's picking up mid-task.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_with_retry(
        &self,
        agent_id: &str,
        prompt: &str,
        working_dir: &Path,
        agent_name: Option<&str>,
        max_retries: u32,
        retry_delay_ms: u64,
        options: &MovementExecOptions,
    ) -> Result<BridgeResult> {
        // Running outside Herdr cannot be fixed by retrying or switching
        // providers, so report it before the retry loop.
        if resolve_a2a_endpoint(options).is_none() {
            HerdrContext::from_env()?;
        }
        let mut current_options = options.clone();
        let mut current_prompt = prompt.to_string();
        let mut fallback_index = 0usize;
        let mut fallbacks_used: Vec<String> = Vec::new();

        loop {
            let mut last_err = None;
            let attempts = max_retries + 1;

            for attempt in 0..attempts {
                if attempt > 0 {
                    let delay = retry_delay_ms * 2u64.pow(attempt - 1);
                    tracing::info!(
                        "Retrying provider CLI (attempt {}/{}, delay {}ms)",
                        attempt + 1,
                        attempts,
                        delay
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                match self
                    .execute_once(
                        agent_id,
                        &current_prompt,
                        working_dir,
                        agent_name,
                        &current_options,
                    )
                    .await
                {
                    Ok(mut result) => {
                        result.fallbacks_used = fallbacks_used;
                        return Ok(result);
                    }
                    Err(e) => {
                        // A rate-limited provider won't recover within our
                        // backoff window; jump to the fallback chain instead
                        // of spending the remaining attempts.
                        let rate_limited = is_rate_limit_error(&e)
                            && fallback_index < current_options.rate_limit_fallbacks.len();
                        if !rate_limited {
                            tracing::warn!("Provider CLI attempt {} failed: {}", attempt + 1, e);
                        }
                        last_err = Some(e);
                        if rate_limited {
                            break;
                        }
                    }
                }
            }

            // Out of attempts (or rate-limited): advance the fallback chain if
            // the failure was a rate limit and targets remain.
            match next_fallback(&current_options, fallback_index, last_err.as_ref()) {
                FallbackStep::Stop => {}
                FallbackStep::Refuse(refusal) => {
                    let rate_limit = last_err.unwrap_or_else(|| anyhow::anyhow!("rate limited"));
                    return Err(rate_limit.context(refusal));
                }
                FallbackStep::Switch(target) => {
                    fallback_index += 1;
                    let from = apply_rate_limit_fallback(&mut current_options, target);
                    tracing::warn!(
                        "Rate limit on '{}' — falling back to '{}'",
                        from,
                        current_options
                            .provider
                            .unwrap_or(ProviderKind::Claude)
                            .as_str()
                    );
                    fallbacks_used.push(from);
                    current_prompt = fallback_notice_prompt(prompt);
                    continue;
                }
            }

            return Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")));
        }
    }

    /// Single execution attempt through A2A, a live Claude session, or a
    /// headless provider command.
    async fn execute_once(
        &self,
        agent_id: &str,
        prompt: &str,
        working_dir: &Path,
        agent_name: Option<&str>,
        options: &MovementExecOptions,
    ) -> Result<BridgeResult> {
        // Herdr resolves relative tab directories against its server, not
        // this process, so the provider and its prompt get an absolute path.
        let working_dir = &crate::session::herdr::absolute_dir(working_dir)?;
        let kind = options.provider.unwrap_or(ProviderKind::Claude);
        let provider = crate::providers::resolve(kind);
        let a2a_endpoint = resolve_a2a_endpoint(options);
        let live = a2a_endpoint.is_none()
            && options.execution == ExecutionMode::Live
            && kind == ProviderKind::Claude;
        if a2a_endpoint.is_none() && options.execution == ExecutionMode::Live && !live {
            tracing::warn!(
                "live execution supports Claude only; running {} headless for '{}'",
                kind.as_str(),
                agent_id
            );
        }

        // Structured output is opt-in; live sessions report through the
        // transcript instead.
        let claude_stream_json = a2a_endpoint.is_none()
            && !live
            && kind == ProviderKind::Claude
            && env_flag("CCSWARM_CLAUDE_STREAM_JSON");
        let codex_json =
            a2a_endpoint.is_none() && kind == ProviderKind::Codex && env_flag("CCSWARM_CODEX_JSON");

        let provider_options = ProviderOptions {
            allowed_tools: options.tools.clone(),
            model: options.model.clone(),
            system_prompt: options.system_prompt.clone(),
            agent_name: agent_name.map(String::from),
            max_budget: options.max_budget,
            worktree_name: options.worktree_name.clone(),
            claude_stream_json,
            codex_json,
            workspace_access: options.workspace_access,
        };

        let prompt_with_cwd =
            prepare_provider_prompt(prompt, working_dir, DEFAULT_MAX_PROMPT_BYTES)?;
        let herdr_call = HerdrCall {
            stage: agent_id.to_string(),
            provider: kind.as_str().to_string(),
        };
        let mut live_tokens = None;
        let (raw_stdout, duration_ms) = if let Some(endpoint) = a2a_endpoint {
            let a2a_prompt = prepare_a2a_prompt(&prompt_with_cwd, options.system_prompt.as_deref());
            let execution = crate::session::a2a::A2AClient::new(endpoint)
                .send_text(&a2a_prompt)
                .await?;
            (execution.output, execution.duration_ms)
        } else if live {
            let request = LiveClaudeRequest {
                call: herdr_call,
                prompt: prompt_with_cwd,
                system_prompt: options.system_prompt.clone(),
                model: options.model.clone(),
                allowed_tools: crate::providers::claude::allowed_tools_arg(&options.tools),
                agent_name: agent_name.map(String::from),
                worktree_name: options.worktree_name.clone(),
            };
            let turn = HerdrContext::from_env()?
                .run_live_claude(
                    &request,
                    working_dir,
                    &crate::session::herdr::claude_config_dir()?,
                )
                .await?;
            tracing::info!(
                "Live Claude session {} finished for '{}' (claude --resume {})",
                turn.session_id,
                agent_id,
                turn.session_id
            );
            live_tokens = turn.tokens;
            (turn.text, turn.duration_ms)
        } else {
            let cmd = provider.build_command(&prompt_with_cwd, working_dir, &provider_options);
            let output = run_provider_command(cmd, working_dir, &herdr_call).await?;
            if !output.success() {
                return Err(anyhow::anyhow!(
                    "{} provider CLI failed: {}",
                    provider.kind().as_str(),
                    output.stderr
                ));
            }
            (output.stdout, output.duration_ms)
        };

        // Project structured output (Claude stream-json / Codex JSONL) down to
        // the result text so parsing and downstream prompts see plain text.
        // Tool names, real token totals, and cost ride along on BridgeResult
        // for the engine's ProviderCall events.
        let stream_meta = if claude_stream_json {
            Some(project_stream(&raw_stdout))
        } else if codex_json {
            let summary = crate::providers::codex_stream::parse_stream(&raw_stdout);
            if let Some(message) = summary.failed {
                return Err(anyhow::anyhow!("codex turn failed: {}", message));
            }
            Some(StreamProjection {
                text: summary.result_text,
                tool_names: summary.tool_names,
                tokens: summary.tokens,
                total_cost_usd: None, // codex does not report cost
            })
        } else {
            None
        };
        let raw_output = match &stream_meta {
            Some(projection) => projection.text.clone(),
            None => raw_stdout,
        };

        let parsed = self
            .output_parser
            .parse(&raw_output)
            .unwrap_or(ParsedOutput::PlainText(raw_output.clone()));
        let success = is_parsed_success(&parsed);
        // Decisive parsed signals (BuildOutput / TestResults / Error log) drive
        // attention directly; ambiguous output falls back to the success bit.
        let attention = AttentionState::from_parsed(&parsed).unwrap_or(if success {
            AttentionState::Done
        } else {
            AttentionState::Error
        });

        let real_tokens = stream_meta.as_ref().and_then(|p| p.tokens).or(live_tokens);
        let (tokens_in, tokens_out) = match real_tokens {
            Some((real_in, real_out)) => (real_in, real_out),
            None => {
                let input_bytes =
                    prompt.len() + options.system_prompt.as_ref().map_or(0, |s| s.len());
                ((input_bytes / 4) as u64, (raw_output.len() / 4) as u64)
            }
        };

        Ok(BridgeResult {
            raw: raw_output,
            parsed,
            success,
            duration_ms,
            tokens_in: Some(tokens_in),
            tokens_out: Some(tokens_out),
            attention,
            tool_names: stream_meta
                .as_ref()
                .map(|p| p.tool_names.clone())
                .unwrap_or_default(),
            total_cost_usd: stream_meta.as_ref().and_then(|p| p.total_cost_usd),
            fallbacks_used: Vec::new(),
        })
    }
}

/// Check if parsed output indicates success
fn is_parsed_success(parsed: &ParsedOutput) -> bool {
    match parsed {
        ParsedOutput::PlainText(_) | ParsedOutput::CodeExecution { .. } => true,
        ParsedOutput::BuildOutput { status, .. } => matches!(status, BuildStatus::Success),
        ParsedOutput::TestResults { failed, .. } => *failed == 0,
        ParsedOutput::StructuredLog { level, .. } => !matches!(level, LogLevel::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::output::TestDetails;

    #[test]
    fn test_is_parsed_success() {
        assert!(is_parsed_success(&ParsedOutput::PlainText(
            "ok".to_string()
        )));
        assert!(!is_parsed_success(&ParsedOutput::TestResults {
            passed: 5,
            failed: 1,
            details: TestDetails {
                suite: Some("cargo".to_string()),
                duration: Some(std::time::Duration::from_secs(0)),
                failed_tests: vec!["test_one".to_string()],
            },
        }));
    }

    #[test]
    fn test_is_rate_limit_error_matches_common_phrasings() {
        for msg in [
            "claude provider CLI failed: API rate limit exceeded",
            "codex provider CLI failed: HTTP 429",
            "Too Many Requests",
            "quota exhausted for this billing period",
            "Error: model overloaded, retry later",
        ] {
            assert!(
                is_rate_limit_error(&anyhow::anyhow!("{msg}")),
                "should match: {msg}"
            );
        }
        for msg in [
            "compile error in main.rs",
            "provider CLI failed: connection refused",
        ] {
            assert!(
                !is_rate_limit_error(&anyhow::anyhow!("{msg}")),
                "should NOT match: {msg}"
            );
        }
    }

    #[test]
    fn fallback_switches_provider_and_model() {
        let mut options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            model: Some("gpt-5".to_string()),
            ..Default::default()
        };

        let from =
            apply_rate_limit_fallback(&mut options, (ProviderKind::Claude, Some("opus".into())));

        assert_eq!(from, "codex");
        assert_eq!(options.provider, Some(ProviderKind::Claude));
        assert_eq!(options.model.as_deref(), Some("opus"));
    }

    #[test]
    fn fallback_without_model_keeps_current_model() {
        let mut options = MovementExecOptions {
            provider: Some(ProviderKind::Claude),
            model: Some("sonnet".to_string()),
            ..Default::default()
        };
        apply_rate_limit_fallback(&mut options, (ProviderKind::Codex, None));
        assert_eq!(options.provider, Some(ProviderKind::Codex));
        assert_eq!(options.model.as_deref(), Some("sonnet"));
    }

    #[test]
    fn read_only_codex_stage_refuses_fallback_to_claude() {
        // Explicit write tools do not lift the read-only requirement.
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            tools: ["read", "bash"].map(String::from).to_vec(),
            workspace_access: WorkspaceAccess::ReadOnly,
            ..Default::default()
        };

        let refusal = check_fallback_access(&options, ProviderKind::Claude)
            .expect_err("weaker fallback must be refused");
        assert!(refusal.contains("read-only"), "{refusal}");
        // Another sandboxed provider keeps the boundary.
        assert!(check_fallback_access(&options, ProviderKind::Codex).is_ok());
    }

    #[test]
    fn fallback_is_allowed_when_no_read_only_sandbox_is_lost() {
        let writable = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            workspace_access: WorkspaceAccess::Write,
            ..Default::default()
        };
        assert!(check_fallback_access(&writable, ProviderKind::Claude).is_ok());

        // Claude does not enforce read-only, so switching away loses nothing.
        let claude_read_only = MovementExecOptions {
            provider: Some(ProviderKind::Claude),
            workspace_access: WorkspaceAccess::ReadOnly,
            ..Default::default()
        };
        assert!(check_fallback_access(&claude_read_only, ProviderKind::Codex).is_ok());
    }

    #[test]
    fn rate_limited_read_only_codex_stage_refuses_claude_fallback() {
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            workspace_access: WorkspaceAccess::ReadOnly,
            rate_limit_fallbacks: vec![(ProviderKind::Claude, None)],
            ..Default::default()
        };
        let rate_limit = anyhow::anyhow!("codex provider CLI failed: HTTP 429");

        match next_fallback(&options, 0, Some(&rate_limit)) {
            FallbackStep::Refuse(refusal) => assert!(refusal.contains("read-only"), "{refusal}"),
            step => panic!("expected a refusal, got {step:?}"),
        }
    }

    #[test]
    fn rate_limited_writable_stage_switches_to_the_next_target() {
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            workspace_access: WorkspaceAccess::Write,
            rate_limit_fallbacks: vec![(ProviderKind::Claude, Some("opus".into()))],
            ..Default::default()
        };
        let rate_limit = anyhow::anyhow!("codex provider CLI failed: HTTP 429");

        assert_eq!(
            next_fallback(&options, 0, Some(&rate_limit)),
            FallbackStep::Switch((ProviderKind::Claude, Some("opus".into())))
        );
    }

    #[test]
    fn fallback_stops_without_a_rate_limit_or_remaining_target() {
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            workspace_access: WorkspaceAccess::ReadOnly,
            rate_limit_fallbacks: vec![(ProviderKind::Claude, None)],
            ..Default::default()
        };
        let other = anyhow::anyhow!("codex provider CLI failed: connection refused");
        let rate_limit = anyhow::anyhow!("Too Many Requests");

        assert_eq!(next_fallback(&options, 0, Some(&other)), FallbackStep::Stop);
        assert_eq!(next_fallback(&options, 0, None), FallbackStep::Stop);
        assert_eq!(
            next_fallback(&options, 1, Some(&rate_limit)),
            FallbackStep::Stop
        );
    }

    #[test]
    fn fallback_notice_prepends_original_prompt() {
        let notice = fallback_notice_prompt("original task");
        assert!(notice.starts_with("# Provider fallback notice"));
        assert!(notice.ends_with("original task"));
    }

    #[test]
    fn test_project_stream_extracts_tool_names_cost_and_real_tokens() {
        let stdout = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"path":"/a.rs"}},{"type":"text","text":"hi"}],"usage":{"input_tokens":10,"output_tokens":2}}}
{"type":"result","subtype":"success","result":"Done.","total_cost_usd":0.01,"usage":{"input_tokens":100,"output_tokens":20,"cache_read_input_tokens":50}}
"#;
        let p = project_stream(stdout);
        assert_eq!(p.text, "Done.");
        assert_eq!(p.tool_names, vec!["Read".to_string()]);
        // Last usage record is the cumulative result envelope: input 100 +
        // cache reads 50; per-turn records must not be summed on top.
        assert_eq!(p.tokens, Some((150, 20)));
        assert!((p.total_cost_usd.unwrap_or(0.0) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn test_project_stream_without_usage_leaves_tokens_none() {
        let p = project_stream(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"plain"}]}}"#,
        );
        assert_eq!(p.text, "plain");
        assert!(p.tool_names.is_empty());
        assert_eq!(p.tokens, None);
        assert!(p.total_cost_usd.is_none());
    }

    #[test]
    fn test_prepare_a2a_prompt_preserves_system_instructions() {
        let prompt = prepare_a2a_prompt("# Task\nReview it", Some("Act as a reviewer."));
        assert_eq!(
            prompt,
            "# System instructions\nAct as a reviewer.\n\n# Task\nReview it"
        );
        assert_eq!(
            prepare_a2a_prompt("# Task\nReview it", Some("  ")),
            "# Task\nReview it"
        );
    }

    #[test]
    fn a2a_endpoint_option_wins_and_blank_values_are_ignored() {
        let options = MovementExecOptions {
            a2a_endpoint: Some(" https://example.test/a2a ".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_a2a_endpoint(&options).as_deref(),
            Some("https://example.test/a2a")
        );
    }
}
