//! AISessionBridge: Claude Code CLI execution + ai-session result management
//!
//! This module bridges the gap between ccswarm's orchestration layer and the ai-session
//! crate. It uses direct CLI subprocess execution and ai-session's subsystems for context
//! compression, output parsing, inter-agent messaging, and persistence.

use anyhow::{Context as _, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ai_session::context::{MessageRole, SessionContext};
use ai_session::coordination::{AgentId, AgentMessage, MessageBus};
use ai_session::core::SessionId;
use ai_session::output::{OutputParser, ParsedOutput};
use ai_session::persistence::PersistenceManager;

use crate::identity::AgentIdentity;

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
}

/// Claude Code CLI execution + ai-session result management layer.
///
/// This bridge provides:
/// - Direct Claude Code CLI execution via subprocess
/// - zstd context compression via ai-session's `TokenEfficientHistory` (93% token reduction)
/// - Semantic output parsing via ai-session's `OutputParser`
/// - Inter-agent messaging via ai-session's `MessageBus`
/// - Session persistence via ai-session's `PersistenceManager`
pub struct AISessionBridge {
    /// Per-agent context histories (zstd compressed)
    context_histories: DashMap<String, SessionContext>,
    /// Inter-agent message bus
    message_bus: Arc<MessageBus>,
    /// Semantic output parser
    output_parser: OutputParser,
    /// Session persistence manager
    persistence: PersistenceManager,
    /// Agent ID mappings (ccswarm agent_id -> ai-session AgentId)
    agent_mappings: DashMap<String, AgentId>,
}

impl AISessionBridge {
    /// Create a new AISessionBridge
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            context_histories: DashMap::new(),
            message_bus: Arc::new(MessageBus::new()),
            output_parser: OutputParser::new(),
            persistence: PersistenceManager::new(storage_path),
            agent_mappings: DashMap::new(),
        }
    }

    /// Register an agent context for tracking conversation history and messages.
    ///
    /// This creates:
    /// - A `SessionContext` with zstd-compressed `TokenEfficientHistory`
    /// - A message bus registration for inter-agent communication
    pub fn register_agent(&self, agent_id: &str) -> Result<()> {
        // Create ai-session context for this agent
        let session_id = SessionId::new();
        let context = SessionContext::new(session_id);
        self.context_histories.insert(agent_id.to_string(), context);

        // Register on message bus
        let ai_agent_id = AgentId::new();
        self.message_bus
            .register_agent(ai_agent_id.clone())
            .context("Failed to register agent on message bus")?;
        self.agent_mappings
            .insert(agent_id.to_string(), ai_agent_id);

        tracing::info!("Registered agent '{}' with AISessionBridge", agent_id);
        Ok(())
    }

    /// Execute a task via Claude Code CLI and manage results with ai-session.
    ///
    /// Flow:
    /// 1. Execute `claude -p <prompt>` via subprocess (with optional --agent/--team)
    /// 2. Parse output with ai-session's `OutputParser`
    /// 3. Store in context history (zstd compressed when threshold reached)
    /// 4. Broadcast result to other agents via message bus
    /// 5. Persist session state for crash recovery
    pub async fn execute_task(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
    ) -> Result<BridgeResult> {
        self.execute_task_with_options(agent_id, prompt, _identity, working_dir, None)
            .await
    }

    /// Execute with optional Claude Code agent/team routing, resume, and retry
    pub async fn execute_task_with_options(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
    ) -> Result<BridgeResult> {
        self.execute_with_retry(
            agent_id,
            prompt,
            _identity,
            working_dir,
            agent_name,
            0,
            1000,
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

            match self
                .execute_once(agent_id, prompt, _identity, working_dir, agent_name)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!("Claude Code CLI attempt {} failed: {}", attempt + 1, e);
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Single execution attempt
    async fn execute_once(
        &self,
        agent_id: &str,
        prompt: &str,
        _identity: &AgentIdentity,
        working_dir: &Path,
        agent_name: Option<&str>,
    ) -> Result<BridgeResult> {
        let start = std::time::Instant::now();

        // 1. Execute Claude Code CLI via subprocess
        let mut cmd = tokio::process::Command::new("claude");
        cmd.args(["-p", prompt, "--output-format", "text"]);

        // Route to specific agent if specified
        if let Some(name) = agent_name {
            cmd.args(["--agent", name]);
        }

        // Resume session if we have prior context for this agent
        if self.context_histories.contains_key(agent_id) {
            cmd.args(["--resume"]);
        }

        let output = cmd
            .current_dir(working_dir)
            .output()
            .await
            .context("Failed to execute Claude Code CLI")?;

        let duration = start.elapsed();

        let raw_output = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Claude Code CLI failed: {}", stderr));
        };

        // 2. Parse output semantically with ai-session
        let parsed = self
            .output_parser
            .parse(&raw_output)
            .unwrap_or(ParsedOutput::PlainText(raw_output.clone()));

        let success = is_parsed_success(&parsed);

        // 3. Add to agent's context history (auto-compresses via zstd when threshold reached)
        if let Some(mut context) = self.context_histories.get_mut(agent_id) {
            context.add_message_raw(MessageRole::User, prompt.to_string());
            context.add_message_raw(MessageRole::Assistant, raw_output.clone());

            // Trigger compression check
            context.compress_context().await;
        }

        // 4. Notify other agents via message bus
        if let Some(ai_agent_id) = self.agent_mappings.get(agent_id) {
            let task_id = ai_session::coordination::TaskId::new();
            let _msg = AgentMessage::TaskCompleted {
                agent_id: ai_agent_id.clone(),
                task_id,
                result: serde_json::json!({
                    "agent": agent_id,
                    "success": success,
                    "output_preview": truncate_output(&raw_output, 500),
                }),
            };
            // Broadcast is best-effort; log errors for debugging
            if let Err(e) = self.message_bus.broadcast(
                ai_agent_id.clone(),
                ai_session::BroadcastMessage {
                    id: uuid::Uuid::new_v4(),
                    from: ai_agent_id.clone(),
                    content: serde_json::json!({
                        "agent": agent_id,
                        "success": success,
                    })
                    .to_string(),
                    priority: ai_session::MessagePriority::Normal,
                    timestamp: chrono::Utc::now(),
                },
            ) {
                tracing::debug!(
                    "MessageBus broadcast failed for agent '{}': {}",
                    agent_id,
                    e
                );
            }
        }

        // 5. Persist session state (best-effort)
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

        Ok(BridgeResult {
            raw: raw_output,
            parsed,
            success,
            duration_ms: duration.as_millis() as u64,
            compression_ratio,
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

    /// Get the message bus for direct inter-agent communication
    pub fn message_bus(&self) -> &Arc<MessageBus> {
        &self.message_bus
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

/// Truncate output for message bus previews
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...", &output[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_truncate_output() {
        assert_eq!(truncate_output("short", 10), "short");
        assert_eq!(truncate_output("this is longer text", 10), "this is lo...");
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
}
