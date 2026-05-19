//! AISessionBridge: multi-provider CLI execution + ai-session result management
//!
//! This module bridges the gap between ccswarm's orchestration layer and the ai-session
//! crate. Provider-specific command construction is delegated to [`crate::providers`];
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
    /// Rough input-token estimate (prompt + system_prompt bytes / 4). Non-authoritative —
    /// provider CLIs don't surface real counts, but this is useful for `ccswarm cost` to
    /// show relative stage weight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_in: Option<u64>,
    /// Rough output-token estimate (raw output bytes / 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_out: Option<u64>,
    /// Attention triage state derived from the parsed output. Falls back to
    /// `Done` on success / `Error` on failure when the parser has no decisive
    /// signal.
    #[serde(default)]
    pub attention: AttentionState,
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
/// [`crate::providers`]; unsupported fields are silently ignored by providers that don't
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

    /// Execute with retry logic and exponential backoff
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
        let mut last_err = None;
        let attempts = max_retries + 1;

        for attempt in 0..attempts {
            if attempt > 0 {
                let delay = retry_delay_ms * 2u64.pow(attempt - 1);
                tracing::info!(
                    "Retrying Claude Code CLI (attempt {}/{}, delay {}ms)",
                    attempt + 1,
                    attempts,
                    delay
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            let attempt_result = match &options.continuation {
                ContinuationPolicy::SingleTurn => {
                    self.execute_once(
                        agent_id,
                        prompt,
                        _identity,
                        working_dir,
                        agent_name,
                        options,
                    )
                    .await
                }
                ContinuationPolicy::MultiTurn { .. } => {
                    // TODO: per-turn retry.
                    self.execute_multi_turn(
                        agent_id,
                        prompt,
                        _identity,
                        working_dir,
                        agent_name,
                        max_retries,
                        retry_delay_ms,
                        options,
                        &options.continuation,
                    )
                    .await
                }
            };

            match attempt_result {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!("Claude Code CLI attempt {} failed: {}", attempt + 1, e);
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
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
        if !provider
            .same_thread_continuation()
            .supports_explicit_session_id()
        {
            return Err(anyhow::anyhow!(
                "{} provider does not support same-thread multi-turn continuation",
                provider_kind.as_str()
            ));
        }

        let session_id = options
            .session_id
            .clone()
            .or_else(|| {
                self.context_histories
                    .get(agent_id)
                    .map(|context| context.session_id.to_string())
            })
            .unwrap_or_else(|| SessionId::new().to_string());

        let mut turn_options = options.clone();
        turn_options.session_id = Some(session_id.clone());

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
        ensure_same_thread_continuation(&first_execution.metadata, &session_id)?;

        let mut result = first_execution.result;

        if !result.success {
            return Ok(result);
        }

        let mut merged_raw = vec![format!("--- TURN 1 ---\n\n{}", result.raw)];
        let mut duration_ms = result.duration_ms;
        let mut tokens_in = result.tokens_in.unwrap_or(0);
        let mut tokens_out = result.tokens_out.unwrap_or(0);
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

        // When stream-json is on for Claude, project the NDJSON stream down to
        // the result text so the rest of the bridge (parsed output, context
        // history, downstream prompt builders) keeps working unchanged. Tool
        // uses and usage records are not yet wired into EventRecorder — that's
        // a follow-up, since wiring needs an EventRecorder handle the bridge
        // doesn't currently own.
        let (raw_output, _stream_summary) = if claude_stream_json {
            let summary = crate::providers::claude_stream::parse_stream(&raw_stdout);
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
            (summary.result_text.clone(), Some(summary))
        } else {
            (raw_stdout, None)
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

        // Issue #28 fix: emit rough byte→token estimates so `ccswarm cost` has data to
        // aggregate. We don't claim accuracy — provider CLIs don't expose real token
        // counts. Divide by 4 as a GPT-family rule of thumb (actual ratio varies).
        let input_bytes =
            prompt.len() + options.system_prompt.as_ref().map(|s| s.len()).unwrap_or(0);
        let tokens_in = Some((input_bytes / 4) as u64);
        let tokens_out = Some((raw_output.len() / 4) as u64);

        let same_thread_continuation = provider
            .same_thread_continuation()
            .supports_explicit_session_id()
            && options.session_id.is_some();

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
            },
            metadata: BridgeExecutionMetadata {
                provider: kind,
                session_id: options.session_id.clone(),
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

fn merge_turn_result(
    merged_raw: &mut Vec<String>,
    duration_ms: &mut u64,
    tokens_in: &mut u64,
    tokens_out: &mut u64,
    turn: u32,
    next_result: BridgeResult,
) -> BridgeResult {
    merged_raw.push(format!("--- TURN {} ---\n\n{}", turn, next_result.raw));
    *duration_ms = duration_ms.saturating_add(next_result.duration_ms);
    *tokens_in = tokens_in.saturating_add(next_result.tokens_in.unwrap_or(0));
    *tokens_out = tokens_out.saturating_add(next_result.tokens_out.unwrap_or(0));

    BridgeResult {
        raw: merged_raw.join("\n\n"),
        parsed: next_result.parsed,
        success: next_result.success,
        duration_ms: *duration_ms,
        compression_ratio: next_result.compression_ratio,
        tokens_in: Some(*tokens_in),
        tokens_out: Some(*tokens_out),
        attention: next_result.attention,
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
        }
    }

    #[test]
    fn test_merge_turn_result_aggregates_successful_turn() {
        let mut merged_raw = vec!["--- TURN 1 ---\n\nfirst".to_string()];
        let mut duration_ms = 10;
        let mut tokens_in = 3;
        let mut tokens_out = 5;

        let result = merge_turn_result(
            &mut merged_raw,
            &mut duration_ms,
            &mut tokens_in,
            &mut tokens_out,
            2,
            bridge_result(
                "second",
                ParsedOutput::PlainText("continue".to_string()),
                true,
                20,
                Some(7),
                Some(11),
            ),
        );

        assert_eq!(
            result.raw,
            "--- TURN 1 ---\n\nfirst\n\n--- TURN 2 ---\n\nsecond"
        );
        assert_eq!(result.duration_ms, 30);
        assert_eq!(result.tokens_in, Some(10));
        assert_eq!(result.tokens_out, Some(16));
        assert!(result.success);
    }

    #[test]
    fn test_merge_turn_result_includes_failed_later_turn() {
        let mut merged_raw = vec!["--- TURN 1 ---\n\nfirst".to_string()];
        let mut duration_ms = 10;
        let mut tokens_in = 3;
        let mut tokens_out = 5;

        let result = merge_turn_result(
            &mut merged_raw,
            &mut duration_ms,
            &mut tokens_in,
            &mut tokens_out,
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
        let options = MovementExecOptions {
            provider: Some(ProviderKind::Codex),
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
            .expect_err("codex multi-turn should fail before spawning a subprocess");

        assert!(
            err.to_string()
                .contains("does not support same-thread multi-turn continuation")
        );
        Ok(())
    }
}
