//! AISessionBridge: multi-provider CLI execution + ai-session result management
//!
//! This module bridges the gap between ccswarm's orchestration layer and the ai-session
//! crate. Provider-specific command construction is delegated to the providers module;
//! everything else (context compression, output parsing, persistence, retry) is neutral
//! to which CLI is spoken.

use anyhow::{Context as _, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use ai_session::context::{MessageRole, SessionContext};
use ai_session::core::{AttentionState, SessionId};
use ai_session::output::{OutputParser, ParsedOutput};
use ai_session::persistence::PersistenceManager;

use crate::identity::AgentIdentity;
use crate::providers::{ProviderKind, ProviderOptions};

const DEFAULT_CONTINUATION_PROMPT: &str = "The previous turn completed but the task is still active. Continue with the next sub-step. Stop when the task is fully done or you cannot make progress.";

/// Result of an AISessionBridge execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResult {
    /// Raw output from Claude Code CLI
    pub raw: String,
    /// Parsed output from ai-session's semantic parser
    pub parsed: ParsedOutput,
    /// Whether the execution was successful (based on parsed output)
    pub success: bool,
    /// Execution duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
    /// Context compression ratio (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
    /// Input tokens for the call. Real counts (incl. cache reads/writes) from
    /// the stream-json result envelope when `CCSWARM_CLAUDE_STREAM_JSON=1`;
    /// otherwise a rough bytes/4 estimate so `ccswarm cost` still has relative
    /// stage weights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_in: Option<u64>,
    /// Output tokens for the call (real when stream-json is on, else bytes/4).
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

/// Switch `options` to a fallback provider after a rate limit. Clears any
/// session continuation — a new provider cannot resume another provider's
/// thread (including codex's provider-assigned thread IDs). Returns the name
/// of the provider being abandoned.
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
    options.session_id = None;
    options.continuation = ContinuationPolicy::SingleTurn;
    from
}

/// Prompt prefix telling the fallback provider it's picking up mid-task.
fn fallback_notice_prompt(original_prompt: &str) -> String {
    format!(
        "# Provider fallback notice\nA previous provider hit a rate limit mid-task. \
         You are taking over fresh — no prior session context is available beyond this prompt.\n\n{}",
        original_prompt
    )
}

/// Projection of a Claude stream-json stdout buffer down to the pieces the
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
    if !summary.tool_uses.is_empty() {
        tracing::debug!(
            "stream-json: {} tool_use blocks ({:?})",
            summary.tool_uses.len(),
            summary
                .tool_uses
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
        );
    }
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

/// Controls whether a stage is allowed to continue in the same provider thread.
#[derive(Debug, Clone)]
pub enum ContinuationPolicy {
    SingleTurn,
    MultiTurn {
        max_turns: u32,
        continuation_prompt: String,
    },
}

impl ContinuationPolicy {
    pub fn multi_turn(max_turns: u32) -> Self {
        Self::MultiTurn {
            max_turns,
            continuation_prompt: DEFAULT_CONTINUATION_PROMPT.to_string(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ContinuationPolicy {
    fn default() -> Self {
        Self::SingleTurn
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
    /// Session ID for continuation.
    pub session_id: Option<String>,
    /// Same-thread continuation policy.
    pub continuation: ContinuationPolicy,
    /// Providers to fall back to (in order) when this stage's provider hits a
    /// rate limit. Each entry is `(provider, optional model override)`. Comes
    /// from the flow-level `on_rate_limit` YAML field.
    pub(crate) rate_limit_fallbacks: Vec<(ProviderKind, Option<String>)>,
}

/// Claude Code CLI execution + ai-session result management layer.
///
/// This bridge provides:
/// - Direct Claude Code CLI execution via subprocess
/// - zstd context compression via ai-session's `TokenEfficientHistory` (93% token reduction)
/// - Semantic output parsing via ai-session's `OutputParser`
/// - Session persistence via ai-session's `PersistenceManager`
pub struct AISessionBridge {
    /// Per-agent context histories (zstd compressed)
    context_histories: DashMap<String, SessionContext>,
    /// Semantic output parser
    output_parser: OutputParser,
    /// Session persistence manager
    persistence: PersistenceManager,
}

#[derive(Debug, Clone)]
struct BridgeExecutionMetadata {
    provider: ProviderKind,
    session_id: Option<String>,
    same_thread_continuation: bool,
}

#[derive(Debug, Clone)]
struct BridgeExecution {
    result: BridgeResult,
    metadata: BridgeExecutionMetadata,
}

impl AISessionBridge {
    /// Create a new AISessionBridge
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            context_histories: DashMap::new(),
            output_parser: OutputParser::new(),
            persistence: PersistenceManager::new(storage_path),
        }
    }

    /// Register an agent context for tracking conversation history.
    ///
    /// This creates a `SessionContext` with zstd-compressed `TokenEfficientHistory`.
    pub fn register_agent(&self, agent_id: &str) -> Result<()> {
        let session_id = SessionId::new();
        let context = SessionContext::new(session_id);
        self.context_histories.insert(agent_id.to_string(), context);

        tracing::info!("Registered agent '{}' with AISessionBridge", agent_id);
        Ok(())
    }

    /// Execute a task via Claude Code CLI and manage results with ai-session.
    ///
    /// Flow:
    /// 1. Execute `claude -p <prompt>` via subprocess (with optional --agent/--team)
    /// 2. Parse output with ai-session's `OutputParser`
    /// 3. Store in context history (zstd compressed when threshold reached)
    /// 4. Persist session state for crash recovery
    pub async fn execute_task(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<BridgeResult> {
        self.execute_with_retry(
            agent_id,
            prompt,
            _identity,
            working_dir,
            None,
            0,
            1000,
            &MovementExecOptions::default(),
        )
        .await
    }

    /// Execute with retry logic and exponential backoff.
    ///
    /// When the provider fails with a rate-limit error and
    /// `options.rate_limit_fallbacks` is non-empty, the call switches to the
    /// next provider in the chain instead of burning retries on the limited
    /// one (takt's `rate_limit_fallback.switch_chain`). The switch resets the
    /// retry budget, clears any session continuation (a new provider can't
    /// resume another provider's thread), and prepends a fallback notice so
    /// the model knows it's picking up mid-task.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_with_retry(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
        max_retries: u32,
        retry_delay_ms: u64,
        options: &MovementExecOptions,
    ) -> Result<BridgeResult> {
        let mut current_options = options.clone();
        let mut current_prompt = prompt.to_string();
        let mut fallback_index = 0usize;
        let mut fallbacks_used: Vec<String> = Vec::new();

        loop {
            let mut last_err = None;
            let attempts = max_retries + 1;

            'attempts: for attempt in 0..attempts {
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

                let attempt_result = match &current_options.continuation {
                    ContinuationPolicy::SingleTurn => {
                        self.execute_once(
                            agent_id,
                            &current_prompt,
                            _identity,
                            working_dir,
                            agent_name,
                            &current_options,
                        )
                        .await
                    }
                    ContinuationPolicy::MultiTurn { .. } => {
                        // TODO: per-turn retry.
                        self.execute_multi_turn(
                            agent_id,
                            &current_prompt,
                            _identity,
                            working_dir,
                            agent_name,
                            max_retries,
                            retry_delay_ms,
                            &current_options,
                            &current_options.continuation.clone(),
                        )
                        .await
                    }
                };

                match attempt_result {
                    Ok(mut result) => {
                        result.fallbacks_used = fallbacks_used;
                        return Ok(result);
                    }
                    Err(e) => {
                        // A rate-limited provider won't recover within our
                        // backoff window; jump to the fallback chain instead
                        // of spending the remaining attempts.
                        if is_rate_limit_error(&e)
                            && fallback_index < current_options.rate_limit_fallbacks.len()
                        {
                            last_err = Some(e);
                            break 'attempts;
                        }
                        tracing::warn!("Provider CLI attempt {} failed: {}", attempt + 1, e);
                        last_err = Some(e);
                    }
                }
            }

            // Out of attempts (or rate-limited): advance the fallback chain if
            // the failure was a rate limit and targets remain.
            let rate_limited = last_err.as_ref().is_some_and(is_rate_limit_error);
            if rate_limited && fallback_index < current_options.rate_limit_fallbacks.len() {
                let target = current_options.rate_limit_fallbacks[fallback_index].clone();
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

            return Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")));
        }
    }

    /// Execute a stage across multiple turns in the same provider thread.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_multi_turn(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
        _max_retries: u32,
        _retry_delay_ms: u64,
        options: &MovementExecOptions,
        continuation: &ContinuationPolicy,
    ) -> Result<BridgeResult> {
        let (max_turns, continuation_prompt) = match continuation {
            ContinuationPolicy::SingleTurn => {
                return self
                    .execute_once(
                        agent_id,
                        prompt,
                        _identity,
                        working_dir,
                        agent_name,
                        options,
                    )
                    .await;
            }
            ContinuationPolicy::MultiTurn {
                max_turns,
                continuation_prompt,
            } => ((*max_turns).max(1), continuation_prompt),
        };

        let provider_kind = options.provider.unwrap_or(ProviderKind::Claude);
        let provider = crate::providers::resolve(provider_kind);
        let continuation_mode = provider.same_thread_continuation();
        if !continuation_mode.supports_multi_turn() {
            return Err(anyhow::anyhow!(
                "{} provider does not support same-thread multi-turn continuation",
                provider_kind.as_str()
            ));
        }

        let mut turn_options = options.clone();

        // ExplicitSessionId (claude): pick the ID up front and pass it every
        // turn. ProviderAssignedId (codex): the first turn runs without an ID;
        // the provider assigns one and the bridge learns it from the turn's
        // metadata (codex --json thread.started event).
        let mut session_id = match continuation_mode {
            crate::providers::SameThreadContinuation::ExplicitSessionId => {
                let sid = options
                    .session_id
                    .clone()
                    .or_else(|| {
                        self.context_histories
                            .get(agent_id)
                            .map(|context| context.session_id.to_string())
                    })
                    .unwrap_or_else(|| SessionId::new().to_string());
                turn_options.session_id = Some(sid.clone());
                Some(sid)
            }
            _ => options.session_id.clone(),
        };

        let first_execution = self
            .execute_once_with_metadata(
                agent_id,
                prompt,
                _identity,
                working_dir,
                agent_name,
                &turn_options,
            )
            .await?;
        match (&session_id, &first_execution.metadata.session_id) {
            // Caller-chosen ID (claude): the turn must confirm it ran under it.
            (Some(expected), _) => {
                ensure_same_thread_continuation(&first_execution.metadata, expected)?;
            }
            // Provider-assigned ID (codex): adopt it for the remaining turns.
            (None, Some(assigned)) => {
                session_id = Some(assigned.clone());
                turn_options.session_id = Some(assigned.clone());
            }
            (None, None) => {
                return Err(anyhow::anyhow!(
                    "{} did not report a session/thread ID on the first turn; cannot continue multi-turn",
                    provider_kind.as_str()
                ));
            }
        }
        let session_id = session_id.unwrap_or_default();

        let mut result = first_execution.result;

        if !result.success {
            return Ok(result);
        }

        let mut merged_raw = vec![format!("--- TURN 1 ---\n\n{}", result.raw)];
        let mut duration_ms = result.duration_ms;
        let mut tokens_in = result.tokens_in.unwrap_or(0);
        let mut tokens_out = result.tokens_out.unwrap_or(0);
        let mut tool_names = result.tool_names.clone();
        let mut total_cost_usd = result.total_cost_usd;
        let mut previous_turn_raw = result.raw.clone();

        result.raw = merged_raw.join("\n\n");
        result.duration_ms = duration_ms;
        result.tokens_in = Some(tokens_in);
        result.tokens_out = Some(tokens_out);

        if result.success && is_task_terminal(&result.parsed) {
            return Ok(result);
        }

        for turn in 2..=max_turns {
            let continuation_prompt = format!(
                "{}\n\n# Previous turn output (truncated)\n{}",
                continuation_prompt,
                truncate(&previous_turn_raw, 2048)
            );

            let next_execution = self
                .execute_once_with_metadata(
                    agent_id,
                    &continuation_prompt,
                    _identity,
                    working_dir,
                    agent_name,
                    &turn_options,
                )
                .await?;
            ensure_same_thread_continuation(&next_execution.metadata, &session_id)?;
            let next_result = next_execution.result;

            previous_turn_raw = next_result.raw.clone();
            result = merge_turn_result(
                &mut merged_raw,
                &mut duration_ms,
                &mut tokens_in,
                &mut tokens_out,
                &mut tool_names,
                &mut total_cost_usd,
                turn,
                next_result,
            );

            if !result.success {
                return Ok(result);
            }

            if result.success && is_task_terminal(&result.parsed) {
                return Ok(result);
            }
        }

        Ok(result)
    }

    /// Single execution attempt — delegates command construction to the selected provider.
    async fn execute_once(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
        options: &MovementExecOptions,
    ) -> Result<BridgeResult> {
        self.execute_once_with_metadata(
            agent_id,
            prompt,
            _identity,
            working_dir,
            agent_name,
            options,
        )
        .await
        .map(|execution| execution.result)
    }

    /// Single execution attempt with provider metadata for continuation safety checks.
    async fn execute_once_with_metadata(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
        options: &MovementExecOptions,
    ) -> Result<BridgeExecution> {
        let start = std::time::Instant::now();

        let kind = options.provider.unwrap_or(ProviderKind::Claude);
        let provider = crate::providers::resolve(kind);

        // Opt-in stream-json for Claude. Defaults to off so v0.7.0 doesn't change
        // the production output path until users explicitly trial it. v0.8.0 is
        // the candidate to flip the default once we've validated parsing against
        // real Claude Code releases.
        let claude_stream_json = kind == ProviderKind::Claude
            && std::env::var("CCSWARM_CLAUDE_STREAM_JSON")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

        // JSONL output for Codex: opt-in via env for telemetry, but forced on
        // whenever session continuation is in play — the thread ID needed for
        // `codex exec resume` only arrives via the `thread.started` event.
        let codex_json = kind == ProviderKind::Codex
            && (std::env::var("CCSWARM_CODEX_JSON")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
                || options.session_id.is_some()
                || matches!(options.continuation, ContinuationPolicy::MultiTurn { .. }));

        let provider_options = ProviderOptions {
            allowed_tools: options.tools.clone(),
            model: options.model.clone(),
            system_prompt: options.system_prompt.clone(),
            agent_name: agent_name.map(String::from),
            session_id: options.session_id.clone(),
            continue_session: options.session_id.is_none()
                && self.context_histories.contains_key(agent_id),
            max_budget: options.max_budget,
            worktree_name: options.worktree_name.clone(),
            claude_stream_json,
            codex_json,
        };

        // code #4 fix: cap prompt size before we hand it to a subprocess. Linux `execve`
        // truncates near ~2 MB of total argv with a cryptic E2BIG; surfacing a clear
        // error is friendlier than chasing "Failed to execute provider CLI".
        const MAX_PROMPT_BYTES: usize = 200_000;
        if prompt.len() > MAX_PROMPT_BYTES {
            return Err(anyhow::anyhow!(
                "prompt is {} bytes (cap is {}) — shorten the task description or split it",
                prompt.len(),
                MAX_PROMPT_BYTES
            ));
        }

        // Issue #22 fix: prepend the working directory as explicit context so the
        // provider LLM uses relative paths anchored here, not $HOME or random absolute
        // paths. Separate marker line keeps the original prompt intact and scannable.
        let prompt_with_cwd = format!(
            "# Working directory\n{}\nUse this directory as the current working directory. Prefer relative paths anchored here.\n\n# Task\n{}",
            working_dir.display(),
            prompt
        );

        let mut cmd = provider.build_command(&prompt_with_cwd, working_dir, &provider_options);
        // code #7 fix: centrally enforce working_dir regardless of what each provider did.
        // A future provider implementation that forgets `.current_dir(...)` would otherwise
        // silently run against ccswarm's own cwd and scatter generated files into $HOME.
        cmd.current_dir(working_dir);

        let output = cmd.output().await.with_context(|| {
            format!(
                "Failed to execute provider CLI: {}",
                provider.kind().as_str()
            )
        })?;

        let duration = start.elapsed();

        let raw_stdout = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "{} provider CLI failed: {}",
                provider.kind().as_str(),
                stderr
            ));
        };

        // When structured output is on (Claude stream-json / Codex JSONL),
        // project the event stream down to the result text so the rest of the
        // bridge (parsed output, context history, downstream prompt builders)
        // keeps working unchanged. Tool names, real token totals, and cost ride
        // along on BridgeResult so the engine (which owns the EventRecorder)
        // can emit ProviderCall events.
        let mut learned_session_id: Option<String> = None;
        let stream_meta = if claude_stream_json {
            Some(project_stream(&raw_stdout))
        } else if codex_json {
            let summary = crate::providers::codex_stream::parse_stream(&raw_stdout);
            if let Some(message) = summary.failed {
                return Err(anyhow::anyhow!("codex turn failed: {}", message));
            }
            // Codex assigns the thread ID; capture it so multi-turn can resume.
            learned_session_id = summary.thread_id;
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

        // 2. Parse output semantically with ai-session
        let parsed = self
            .output_parser
            .parse(&raw_output)
            .unwrap_or(ParsedOutput::PlainText(raw_output.clone()));

        let success = is_parsed_success(&parsed);
        // Decisive parsed signals (BuildOutput / TestResults / Error log) drive
        // attention directly; for ambiguous output (PlainText / CodeExecution)
        // fall back to the success bit so the column still says something
        // useful per stage.
        let attention = AttentionState::from_parsed(&parsed).unwrap_or(if success {
            AttentionState::Done
        } else {
            AttentionState::Error
        });

        // 3. Add to agent's context history (auto-compresses via zstd when threshold reached)
        if let Some(mut context) = self.context_histories.get_mut(agent_id) {
            context.add_message_raw(MessageRole::User, prompt.to_string());
            context.add_message_raw(MessageRole::Assistant, raw_output.clone());

            // Trigger compression check
            context.compress_context().await;
        }

        // 4. Persist session state (best-effort)
        if let Some(context) = self.context_histories.get(agent_id) {
            let session_id = context.session_id.clone();
            let state = ai_session::persistence::SessionState {
                session_id: session_id.clone(),
                config: ai_session::SessionConfig::default(),
                status: ai_session::SessionStatus::Running,
                context: context.clone(),
                command_history: Vec::new(),
                metadata: ai_session::persistence::SessionMetadata::default(),
            };
            if let Err(e) = self.persistence.save_session(&session_id, &state).await {
                tracing::warn!("Failed to persist session state for {}: {}", agent_id, e);
            }
        }

        // Get compression ratio if context exists
        let compression_ratio = self
            .get_compression_stats(agent_id)
            .map(|stats| stats.compression_ratio);

        // Token accounting: prefer the real counts from the stream-json result
        // envelope when available; otherwise fall back to the Issue #28 rough
        // byte→token estimates (divide by 4, GPT-family rule of thumb) so
        // `ccswarm cost` always has data to aggregate.
        let (tokens_in, tokens_out) = match stream_meta.as_ref().and_then(|p| p.tokens) {
            Some((real_in, real_out)) => (Some(real_in), Some(real_out)),
            None => {
                let input_bytes =
                    prompt.len() + options.system_prompt.as_ref().map(|s| s.len()).unwrap_or(0);
                (
                    Some((input_bytes / 4) as u64),
                    Some((raw_output.len() / 4) as u64),
                )
            }
        };

        // The session ID this turn actually ran under: provider-assigned IDs
        // (codex thread.started) win over the caller-supplied one, so the
        // multi-turn loop can learn the ID after the first turn.
        let session_id_for_metadata = learned_session_id.or_else(|| options.session_id.clone());
        let same_thread_continuation = match provider.same_thread_continuation() {
            crate::providers::SameThreadContinuation::ExplicitSessionId => {
                options.session_id.is_some()
            }
            crate::providers::SameThreadContinuation::ProviderAssignedId => {
                session_id_for_metadata.is_some()
            }
            crate::providers::SameThreadContinuation::Unsupported => false,
        };

        Ok(BridgeExecution {
            result: BridgeResult {
                raw: raw_output,
                parsed,
                success,
                duration_ms: duration.as_millis() as u64,
                compression_ratio,
                tokens_in,
                tokens_out,
                attention,
                tool_names: stream_meta
                    .as_ref()
                    .map(|p| p.tool_names.clone())
                    .unwrap_or_default(),
                total_cost_usd: stream_meta.as_ref().and_then(|p| p.total_cost_usd),
                fallbacks_used: Vec::new(),
            },
            metadata: BridgeExecutionMetadata {
                provider: kind,
                session_id: session_id_for_metadata,
                same_thread_continuation,
            },
        })
    }

    /// Get compression statistics for an agent's context
    pub fn get_compression_stats(
        &self,
        agent_id: &str,
    ) -> Option<ai_session::context::CompressionStats> {
        self.context_histories
            .get(agent_id)
            .map(|ctx| ctx.get_compression_stats())
    }

    /// Get the number of registered agents
    pub fn agent_count(&self) -> usize {
        self.context_histories.len()
    }

    /// Get recent context messages for an agent (for prompt augmentation)
    pub fn get_recent_context(&self, agent_id: &str, n: usize) -> Vec<String> {
        self.context_histories
            .get(agent_id)
            .map(|ctx| {
                ctx.get_recent_messages(n)
                    .into_iter()
                    .map(|m| format!("[{:?}] {}", m.role, m.content))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Check if parsed output indicates success
fn is_parsed_success(parsed: &ParsedOutput) -> bool {
    match parsed {
        ParsedOutput::PlainText(_) => true, // Assume success for plain text
        ParsedOutput::CodeExecution { .. } => true,
        ParsedOutput::BuildOutput { status, .. } => {
            matches!(status, ai_session::output::BuildStatus::Success)
        }
        ParsedOutput::TestResults { failed, .. } => *failed == 0,
        ParsedOutput::StructuredLog { level, .. } => {
            !matches!(level, ai_session::output::LogLevel::Error)
        }
    }
}

fn is_task_terminal(parsed: &ParsedOutput) -> bool {
    match parsed {
        ParsedOutput::BuildOutput { status, .. } => {
            matches!(status, ai_session::output::BuildStatus::Success)
        }
        ParsedOutput::TestResults { failed, .. } => *failed == 0,
        _ => false,
    }
}

fn truncate(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let boundary = s
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= max_bytes)
        .last()
        .unwrap_or(0);

    s[..boundary].to_string()
}

#[allow(clippy::too_many_arguments)]
fn merge_turn_result(
    merged_raw: &mut Vec<String>,
    duration_ms: &mut u64,
    tokens_in: &mut u64,
    tokens_out: &mut u64,
    tool_names: &mut Vec<String>,
    total_cost_usd: &mut Option<f64>,
    turn: u32,
    next_result: BridgeResult,
) -> BridgeResult {
    merged_raw.push(format!("--- TURN {} ---\n\n{}", turn, next_result.raw));
    *duration_ms = duration_ms.saturating_add(next_result.duration_ms);
    *tokens_in = tokens_in.saturating_add(next_result.tokens_in.unwrap_or(0));
    *tokens_out = tokens_out.saturating_add(next_result.tokens_out.unwrap_or(0));
    tool_names.extend(next_result.tool_names.iter().cloned());
    *total_cost_usd = match (*total_cost_usd, next_result.total_cost_usd) {
        (Some(a), Some(b)) => Some(a + b),
        (a, b) => a.or(b),
    };

    BridgeResult {
        raw: merged_raw.join("\n\n"),
        parsed: next_result.parsed,
        success: next_result.success,
        duration_ms: *duration_ms,
        compression_ratio: next_result.compression_ratio,
        tokens_in: Some(*tokens_in),
        tokens_out: Some(*tokens_out),
        attention: next_result.attention,
        tool_names: tool_names.clone(),
        total_cost_usd: *total_cost_usd,
        fallbacks_used: next_result.fallbacks_used,
    }
}

fn ensure_same_thread_continuation(
    metadata: &BridgeExecutionMetadata,
    expected_session_id: &str,
) -> Result<()> {
    if metadata.same_thread_continuation
        && metadata.session_id.as_deref() == Some(expected_session_id)
    {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "{} provider did not confirm same-thread continuation for session {}",
        metadata.provider.as_str(),
        expected_session_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentRole;
    use ai_session::output::{BuildStatus, ExecutionMetrics, TestDetails};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_bridge_creation() {
        let bridge = AISessionBridge::new(PathBuf::from("/tmp/ccswarm-test"));
        assert_eq!(bridge.agent_count(), 0);
    }

    #[test]
    fn test_agent_registration() {
        let bridge = AISessionBridge::new(PathBuf::from("/tmp/ccswarm-test"));
        bridge.register_agent("frontend-agent").unwrap();
        assert_eq!(bridge.agent_count(), 1);
    }

    #[test]
    fn test_is_parsed_success() {
        assert!(is_parsed_success(&ParsedOutput::PlainText(
            "ok".to_string()
        )));
        assert!(!is_parsed_success(&ParsedOutput::TestResults {
            passed: 5,
            failed: 1,
            details: ai_session::output::TestDetails {
                suite: Some("cargo".to_string()),
                duration: Some(std::time::Duration::from_secs(0)),
                failed_tests: vec!["test_one".to_string()],
            },
        }));
    }

    #[test]
    fn test_continuation_policy_default_is_single_turn() {
        assert!(matches!(
            ContinuationPolicy::default(),
            ContinuationPolicy::SingleTurn
        ));
    }

    #[test]
    fn test_is_task_terminal_strict() {
        assert!(is_task_terminal(&ParsedOutput::BuildOutput {
            status: BuildStatus::Success,
            artifacts: Vec::new(),
        }));
        assert!(!is_task_terminal(&ParsedOutput::BuildOutput {
            status: BuildStatus::Failed("failed".to_string()),
            artifacts: Vec::new(),
        }));
        assert!(is_task_terminal(&ParsedOutput::TestResults {
            passed: 3,
            failed: 0,
            details: TestDetails::default(),
        }));
        assert!(!is_task_terminal(&ParsedOutput::TestResults {
            passed: 3,
            failed: 1,
            details: TestDetails::default(),
        }));
        assert!(!is_task_terminal(&ParsedOutput::PlainText(
            "done".to_string()
        )));
        assert!(!is_task_terminal(&ParsedOutput::CodeExecution {
            result: "done".to_string(),
            metrics: ExecutionMetrics {
                execution_time: std::time::Duration::from_millis(1),
                memory_usage: None,
                cpu_usage: None,
            },
        }));
    }

    #[test]
    fn test_truncate_is_utf8_safe() {
        let input = "abあcd";

        assert_eq!(truncate(input, 4), "ab");
        assert_eq!(truncate(input, 5), "abあ");
        assert_eq!(truncate(input, 99), input);
    }

    #[test]
    fn test_multi_turn_constructor_sets_default_prompt() {
        match ContinuationPolicy::multi_turn(3) {
            ContinuationPolicy::MultiTurn {
                max_turns,
                continuation_prompt,
            } => {
                assert_eq!(max_turns, 3);
                assert_eq!(continuation_prompt, DEFAULT_CONTINUATION_PROMPT);
            }
            ContinuationPolicy::SingleTurn => {
                panic!("multi_turn constructor returned SingleTurn");
            }
        }
    }

    fn bridge_result(
        raw: &str,
        parsed: ParsedOutput,
        success: bool,
        duration_ms: u64,
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
    ) -> BridgeResult {
        BridgeResult {
            raw: raw.to_string(),
            parsed,
            success,
            duration_ms,
            compression_ratio: Some(1.0),
            tokens_in,
            tokens_out,
            attention: AttentionState::Idle,
            tool_names: Vec::new(),
            total_cost_usd: None,
            fallbacks_used: Vec::new(),
        }
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
    fn provider_switch_drops_provider_assigned_id() {
        // Shared guard between the fallback chain and codex's
        // ProviderAssignedId continuation: switching providers must clear
        // session state, since the new provider can't resume the old thread.
        let mut options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
            session_id: Some("thread-from-codex".to_string()),
            continuation: ContinuationPolicy::multi_turn(3),
            model: Some("gpt-5".to_string()),
            ..Default::default()
        };

        let from =
            apply_rate_limit_fallback(&mut options, (ProviderKind::Claude, Some("opus".into())));

        assert_eq!(from, "codex");
        assert_eq!(options.provider, Some(ProviderKind::Claude));
        assert_eq!(options.model.as_deref(), Some("opus"));
        assert!(options.session_id.is_none());
        assert!(matches!(
            options.continuation,
            ContinuationPolicy::SingleTurn
        ));
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
        // No usage record → bridge falls back to byte/4 estimates.
        assert_eq!(p.tokens, None);
        assert!(p.total_cost_usd.is_none());
    }

    #[test]
    fn test_merge_turn_result_aggregates_successful_turn() {
        let mut merged_raw = vec!["--- TURN 1 ---\n\nfirst".to_string()];
        let mut duration_ms = 10;
        let mut tokens_in = 3;
        let mut tokens_out = 5;
        let mut tool_names = vec!["Read".to_string()];
        let mut total_cost_usd = Some(0.01);

        let mut next = bridge_result(
            "second",
            ParsedOutput::PlainText("continue".to_string()),
            true,
            20,
            Some(7),
            Some(11),
        );
        next.tool_names = vec!["Bash".to_string()];
        next.total_cost_usd = Some(0.02);

        let result = merge_turn_result(
            &mut merged_raw,
            &mut duration_ms,
            &mut tokens_in,
            &mut tokens_out,
            &mut tool_names,
            &mut total_cost_usd,
            2,
            next,
        );

        assert_eq!(
            result.raw,
            "--- TURN 1 ---\n\nfirst\n\n--- TURN 2 ---\n\nsecond"
        );
        assert_eq!(result.duration_ms, 30);
        assert_eq!(result.tokens_in, Some(10));
        assert_eq!(result.tokens_out, Some(16));
        assert!(result.success);
        assert_eq!(result.tool_names, vec!["Read", "Bash"]);
        assert!((result.total_cost_usd.unwrap_or(0.0) - 0.03).abs() < 1e-9);
    }

    #[test]
    fn test_merge_turn_result_includes_failed_later_turn() {
        let mut merged_raw = vec!["--- TURN 1 ---\n\nfirst".to_string()];
        let mut duration_ms = 10;
        let mut tokens_in = 3;
        let mut tokens_out = 5;
        let mut tool_names = Vec::new();
        let mut total_cost_usd = None;

        let result = merge_turn_result(
            &mut merged_raw,
            &mut duration_ms,
            &mut tokens_in,
            &mut tokens_out,
            &mut tool_names,
            &mut total_cost_usd,
            2,
            bridge_result(
                "failed",
                ParsedOutput::TestResults {
                    passed: 1,
                    failed: 1,
                    details: TestDetails::default(),
                },
                false,
                20,
                None,
                Some(11),
            ),
        );

        assert_eq!(
            result.raw,
            "--- TURN 1 ---\n\nfirst\n\n--- TURN 2 ---\n\nfailed"
        );
        assert_eq!(result.duration_ms, 30);
        assert_eq!(result.tokens_in, Some(3));
        assert_eq!(result.tokens_out, Some(16));
        assert!(!result.success);
        assert!(matches!(
            result.parsed,
            ParsedOutput::TestResults { failed: 1, .. }
        ));
    }

    #[tokio::test]
    async fn test_multi_turn_rejects_provider_without_same_thread_continuation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let bridge = AISessionBridge::new(dir.path().join("sessions"));
        let identity = AgentIdentity {
            agent_id: "agent-1".to_string(),
            specialization: AgentRole::Search {
                technologies: Vec::new(),
                responsibilities: Vec::new(),
                boundaries: Vec::new(),
            },
            workspace_path: dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            session_id: "session-1".to_string(),
            parent_process_id: "parent-1".to_string(),
            initialized_at: chrono::Utc::now(),
        };
        // Copilot is the remaining provider with no continuation support
        // (codex gained ProviderAssignedId continuation in v0.8.0).
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Copilot),
            continuation: ContinuationPolicy::multi_turn(2),
            ..MovementExecOptions::default()
        };

        let err = bridge
            .execute_multi_turn(
                "agent-1",
                "do work",
                &identity,
                dir.path(),
                None,
                0,
                0,
                &options,
                &options.continuation,
            )
            .await
            .expect_err("copilot multi-turn should fail before spawning a subprocess");

        assert!(
            err.to_string()
                .contains("does not support same-thread multi-turn continuation")
        );
        Ok(())
    }
}
