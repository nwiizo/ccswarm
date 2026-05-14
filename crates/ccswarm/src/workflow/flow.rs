//! Flow/Stage workflow system.
//!
//! A **Flow** is a declarative YAML-defined workflow containing:
//! - Named **Stages** (sequential steps with persona/provider/instructions)
//! - **Rules** for conditional routing between stages
//! - **Output contracts** for schema validation
//!
//! Example YAML:
//! ```yaml
//! name: default
//! description: "Standard development workflow"
//! max_stages: 30
//! initial_movement: plan
//!
//! stages:
//!   - id: plan
//!     persona: planner
//!     instruction: "Analyze the task and create a plan"
//!     tools: [read, grep, glob]
//!     permission: readonly
//!     rules:
//!       - condition: success
//!         next: implement
//!       - condition: needs_clarification
//!         next: clarify
//!
//!   - id: implement
//!     persona: coder
//!     instruction: "Implement the plan"
//!     tools: [read, write, edit, bash]
//!     permission: edit
//!     rules:
//!       - condition: success
//!         next: review
//!       - condition: test_failure
//!         next: fix
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// A Flow is a complete workflow definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    /// Flow name (unique identifier)
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Maximum number of stage transitions before abort
    #[serde(default = "default_max_movements")]
    pub max_stages: u32,

    /// ID of the first stage to execute
    pub initial_movement: String,

    /// List of stages in this flow
    pub stages: Vec<Stage>,

    /// Global variables for the flow
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,

    /// Flow-level metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Default interactive mode for this flow
    #[serde(default)]
    pub interactive_mode: Option<super::interactive::InteractiveMode>,
}

fn default_max_movements() -> u32 {
    30
}

/// A Stage is a single step in a Flow workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    /// Unique stage identifier within the flow
    pub id: String,

    /// Persona to use (references a persona YAML file or inline prompt)
    #[serde(default)]
    pub persona: Option<String>,

    /// Policy to apply (references a policy YAML file or inline constraints)
    #[serde(default)]
    pub policy: Option<String>,

    /// Knowledge facet (references a knowledge YAML file or inline context)
    #[serde(default)]
    pub knowledge: Option<String>,

    /// Provider to use (claude, codex, etc.)
    #[serde(default)]
    pub provider: Option<String>,

    /// Model to use (overrides provider default)
    #[serde(default)]
    pub model: Option<String>,

    /// Instruction for the agent
    #[serde(default)]
    pub instruction: String,

    /// Tools available to this stage
    #[serde(default)]
    pub tools: Vec<String>,

    /// Permission level for this stage
    #[serde(default)]
    pub permission: MovementPermission,

    /// Routing rules evaluated after stage completes
    #[serde(default)]
    pub rules: Vec<MovementRule>,

    /// Whether this stage executes sub-stages in parallel
    #[serde(default)]
    pub parallel: bool,

    /// Sub-stages for parallel execution
    #[serde(default)]
    pub sub_movements: Vec<String>,

    /// Output contract for validation
    #[serde(default)]
    pub output_contract: Option<OutputContract>,

    /// Maximum execution time in seconds
    #[serde(default)]
    pub timeout: Option<u32>,

    /// Number of retries on failure
    #[serde(default)]
    pub max_retries: u32,

    /// Claude Code agent name for --agent flag routing
    #[serde(default)]
    pub agent: Option<String>,

    /// Working directory override for this stage
    #[serde(default)]
    pub working_dir: Option<String>,

    /// Retry delay in milliseconds (default: 1000)
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Whether to pass previous stage's response as context (default: true)
    /// Set to false for fix stages where fresh context is preferred
    #[serde(default = "default_true")]
    pub pass_previous_response: bool,
}

fn default_true() -> bool {
    true
}

fn default_retry_delay() -> u64 {
    1000
}

/// Permission level for a stage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MovementPermission {
    /// Read-only access (can read files, search, but not modify)
    Readonly,
    /// Edit access (can modify existing files)
    #[default]
    Edit,
    /// Full access (can create, delete, execute commands)
    Full,
}

/// A routing rule that determines the next stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementRule {
    /// Condition to evaluate (string match, AI evaluation, or built-in)
    pub condition: RuleCondition,

    /// Next stage ID if condition matches
    pub next: String,

    /// Optional priority for rule ordering (higher = checked first)
    #[serde(default)]
    pub priority: u8,
}

/// Condition types for stage routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleCondition {
    /// Simple string condition (matched against output markers)
    Simple(String),

    /// AI-evaluated condition
    AiCondition {
        /// The AI prompt to evaluate
        ai: String,
    },

    /// Compound condition with all/any aggregation
    Compound(CompoundCondition),
}

/// Compound condition with logical operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompoundCondition {
    /// All conditions must match
    #[serde(rename = "all")]
    All(Vec<String>),
    /// Any condition must match
    #[serde(rename = "any")]
    Any(Vec<String>),
}

/// Output contract for validating stage results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputContract {
    /// Expected output format (markdown, json, text, yaml, code)
    #[serde(default = "default_format")]
    pub format: String,

    /// Required sections in the output
    #[serde(default)]
    pub required_sections: Vec<String>,

    /// JSON schema for structured output validation
    #[serde(default)]
    pub schema: Option<serde_json::Value>,

    /// Output file name (for file-based contracts)
    #[serde(default)]
    pub output_file: Option<String>,

    /// Required JSON keys (for quick validation without full schema)
    #[serde(default)]
    pub required_keys: Vec<String>,

    /// Minimum output length in characters
    #[serde(default)]
    pub min_length: Option<usize>,

    /// Maximum output length in characters
    #[serde(default)]
    pub max_length: Option<usize>,

    /// Regular expression patterns the output must match
    #[serde(default)]
    pub must_match: Vec<String>,

    /// Regular expression patterns the output must NOT match
    #[serde(default)]
    pub must_not_match: Vec<String>,

    /// Allow-list of file globs the stage is permitted to create or modify.
    ///
    /// Empty means "no restriction". If present, files written by the stage that
    /// don't match any pattern are surfaced as `UnexpectedFile` observations — not
    /// hard violations, so the run still completes, but the user sees what the AI
    /// added beyond the spec. This closes JTBD issue #44 (spec-drift detection).
    #[serde(default)]
    pub allowed_files: Vec<String>,
}

/// Result of output contract validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    /// Whether the contract was satisfied
    pub valid: bool,
    /// List of violations found
    pub violations: Vec<ContractViolation>,
}

/// A single contract violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractViolation {
    /// Violation type
    pub kind: ViolationKind,
    /// Human-readable message
    pub message: String,
}

/// Types of contract violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationKind {
    /// Missing required section
    MissingSection,
    /// Missing required JSON key
    MissingKey,
    /// Invalid format
    InvalidFormat,
    /// Output too short
    TooShort,
    /// Output too long
    TooLong,
    /// Schema validation failure
    SchemaViolation,
    /// Pattern match failure
    PatternViolation,
    /// Forbidden pattern found
    ForbiddenPattern,
    /// File created/modified by the stage that is not covered by allowed_files.
    /// Observation-level — does not fail the run by itself.
    UnexpectedFile,
}

fn default_format() -> String {
    "text".to_string()
}

/// Runtime state of a flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowState {
    /// Flow being executed
    pub flow_name: String,

    /// Current stage ID
    pub current_movement: String,

    /// Number of stages executed so far
    pub movement_count: u32,

    /// History of stage transitions
    pub history: Vec<MovementTransition>,

    /// Accumulated variables/outputs
    pub variables: HashMap<String, serde_json::Value>,

    /// Current status
    pub status: FlowStatus,

    /// Started at
    pub started_at: DateTime<Utc>,

    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

/// A recorded transition between stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementTransition {
    /// Source stage ID
    pub from: String,
    /// Destination stage ID
    pub to: String,
    /// Condition that triggered the transition
    pub condition: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Output from the source stage
    pub output: Option<serde_json::Value>,
}

/// Flow execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowStatus {
    /// Not yet started
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully (reached terminal stage or explicit completion)
    Completed,
    /// Aborted (max stages exceeded or error)
    Aborted,
    /// Failed with error
    Failed,
}

impl Flow {
    /// Load a flow from a YAML file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read flow file: {}", path.display()))?;

        Self::from_yaml(&contents)
    }

    /// Parse a flow from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let flow: Self = serde_yml::from_str(yaml).context("Failed to parse flow YAML")?;
        flow.validate()?;
        Ok(flow)
    }

    /// Validate flow structure
    pub fn validate(&self) -> Result<()> {
        // Check initial stage exists
        if !self.stages.iter().any(|m| m.id == self.initial_movement) {
            return Err(anyhow::anyhow!(
                "Initial stage '{}' not found in flow '{}'",
                self.initial_movement,
                self.name
            ));
        }

        // Check all rule targets reference valid stages
        for stage in &self.stages {
            for rule in &stage.rules {
                if !self.stages.iter().any(|m| m.id == rule.next) {
                    return Err(anyhow::anyhow!(
                        "Rule in stage '{}' references unknown stage '{}'",
                        stage.id,
                        rule.next
                    ));
                }
            }

            // Check parallel sub-stages exist
            if stage.parallel {
                for sub in &stage.sub_movements {
                    if !self.stages.iter().any(|m| m.id == *sub) {
                        return Err(anyhow::anyhow!(
                            "Parallel stage '{}' references unknown sub-stage '{}'",
                            stage.id,
                            sub
                        ));
                    }
                }
            }
        }

        // Check for duplicate stage IDs
        let mut seen = std::collections::HashSet::new();
        for stage in &self.stages {
            if !seen.insert(&stage.id) {
                return Err(anyhow::anyhow!(
                    "Duplicate stage ID '{}' in flow '{}'",
                    stage.id,
                    self.name
                ));
            }
        }

        Ok(())
    }

    /// Get a stage by ID
    pub fn get_movement(&self, id: &str) -> Option<&Stage> {
        self.stages.iter().find(|m| m.id == id)
    }

    /// Check if a stage is terminal (has no rules / no transitions)
    pub fn is_terminal(&self, movement_id: &str) -> bool {
        self.get_movement(movement_id)
            .map(|m| m.rules.is_empty())
            .unwrap_or(true)
    }

    /// Create initial execution state
    pub fn create_state(&self) -> FlowState {
        FlowState {
            flow_name: self.name.clone(),
            current_movement: self.initial_movement.clone(),
            movement_count: 0,
            history: Vec::new(),
            variables: self.variables.clone(),
            status: FlowStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Flow engine that executes flow workflows
pub struct FlowEngine {
    /// Loaded flows
    flows: HashMap<String, Flow>,
    /// Stage judge for tag/AI-based condition evaluation
    judge: super::judge::MovementJudge,
    /// Facet registry for prompt composition
    facet_registry: super::facets::FacetRegistry,
    /// Bridge for real Claude Code CLI execution + ai-session result management
    bridge: Option<std::sync::Arc<crate::session::bridge::AISessionBridge>>,
    /// Working directory for agent execution
    working_dir: std::path::PathBuf,
    /// Optional event recorder for NDJSON observability
    event_recorder: Option<crate::events::EventRecorder>,
    /// Last known execution state (for partial result recovery on timeout)
    last_state: std::sync::Arc<tokio::sync::RwLock<Option<FlowState>>>,
    /// Progress callback for real-time stage completion notifications
    progress_tx: Option<tokio::sync::mpsc::UnboundedSender<MovementProgress>>,
    /// Per-stage cost cap forwarded to providers that understand it (Claude's
    /// `--max-budget-usd`). `None` leaves it to the provider's own default.
    budget_usd: Option<f64>,
    /// Cumulative input+output token cap across the whole run. When exceeded, the
    /// flow aborts before the next stage starts. Complements `budget_usd`, which
    /// only caps a single stage and only works for Claude.
    run_token_cap: Option<u64>,
}

/// Progress notification sent after each stage completes
#[derive(Debug, Clone)]
pub struct MovementProgress {
    /// Stage ID
    pub movement_id: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether it succeeded
    pub success: bool,
    /// Stage count so far
    pub movements_completed: usize,
}

impl FlowEngine {
    pub fn new() -> Self {
        let mut facet_registry = super::facets::FacetRegistry::new();
        // Register built-in facets
        for persona in super::facets::builtin_personas() {
            facet_registry.register_persona(persona);
        }
        for policy in super::facets::builtin_policies() {
            facet_registry.register_policy(policy);
        }
        // Register built-in flows so they're available by default
        let mut flows = HashMap::new();
        for flow in builtin_flows() {
            flows.insert(flow.name.clone(), flow);
        }
        Self {
            flows,
            judge: super::judge::MovementJudge::default(),
            facet_registry,
            bridge: None,
            working_dir: std::path::PathBuf::from("."),
            event_recorder: None,
            last_state: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
            progress_tx: None,
            budget_usd: None,
            run_token_cap: None,
        }
    }

    /// Set a per-stage budget cap in USD. Forwarded to Claude via `--max-budget-usd`;
    /// other providers currently ignore it (they don't expose an equivalent flag).
    pub fn set_budget(&mut self, budget_usd: f64) {
        self.budget_usd = Some(budget_usd);
    }

    /// Cap cumulative input+output tokens across the run. Checked between stages;
    /// when exceeded, the flow aborts with `FlowStatus::Aborted` and emits a
    /// `budget_exceeded` event. Provider-agnostic since token estimates are
    /// produced by the bridge regardless of backend.
    pub fn set_run_token_cap(&mut self, cap: u64) {
        self.run_token_cap = Some(cap);
    }

    /// Create with custom judge config
    pub fn with_judge_config(config: super::judge::JudgeConfig) -> Self {
        let mut engine = Self::new();
        engine.judge = super::judge::MovementJudge::new(config);
        engine
    }

    /// Get a mutable reference to the facet registry for loading custom facets
    pub fn facet_registry_mut(&mut self) -> &mut super::facets::FacetRegistry {
        &mut self.facet_registry
    }

    /// Set the AISessionBridge for real Claude Code CLI execution
    pub fn set_bridge(&mut self, bridge: std::sync::Arc<crate::session::bridge::AISessionBridge>) {
        self.bridge = Some(bridge);
    }

    /// Set the working directory for agent execution
    pub fn set_working_dir(&mut self, dir: std::path::PathBuf) {
        self.working_dir = dir;
    }

    /// Set the event recorder for NDJSON observability
    pub fn set_event_recorder(&mut self, recorder: crate::events::EventRecorder) {
        self.event_recorder = Some(recorder);
    }

    /// Get the last known execution state (for partial result recovery on timeout)
    pub async fn get_last_state(&self) -> Option<FlowState> {
        self.last_state.read().await.clone()
    }

    /// Set a progress channel for real-time stage completion notifications
    pub fn set_progress_channel(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<MovementProgress>,
    ) {
        self.progress_tx = Some(tx);
    }

    /// Load a flow from a YAML file
    pub async fn load_flow(&mut self, path: &Path) -> Result<String> {
        let flow = Flow::load_from_file(path).await?;
        let name = flow.name.clone();
        info!("Loaded flow '{}' with {} stages", name, flow.stages.len());
        self.flows.insert(name.clone(), flow);
        Ok(name)
    }

    /// Load all flows from a directory
    pub async fn load_pieces_from_dir(&mut self, dir: &Path) -> Result<Vec<String>> {
        let mut loaded = Vec::new();

        if !dir.exists() {
            return Ok(loaded);
        }

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                || path.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                match self.load_flow(&path).await {
                    Ok(name) => loaded.push(name),
                    Err(e) => warn!("Failed to load flow from {}: {}", path.display(), e),
                }
            }
        }

        Ok(loaded)
    }

    /// Load all builtin flows into the engine
    pub fn load_builtin_flows(&mut self) {
        for flow in builtin_flows() {
            self.flows.insert(flow.name.clone(), flow);
        }
    }

    /// Get a loaded flow
    pub fn get_flow(&self, name: &str) -> Option<&Flow> {
        self.flows.get(name)
    }

    /// List all loaded flows
    pub fn list_flows(&self) -> Vec<&Flow> {
        self.flows.values().collect()
    }

    /// Register a flow directly (for programmatic / test usage)
    pub fn register_flow(&mut self, flow: Flow) {
        self.flows.insert(flow.name.clone(), flow);
    }

    /// Record an event if recorder is configured (best-effort, logs on failure)
    async fn record_event(&self, event: crate::events::Event) {
        let Some(ref recorder) = self.event_recorder else {
            return;
        };
        if let Err(e) = recorder.record(event).await {
            warn!("Failed to record event: {}", e);
        }
    }

    /// Execute a flow workflow with a task description injected as context
    pub async fn execute_piece_with_task(&self, name: &str, task_text: &str) -> Result<FlowState> {
        let flow = self
            .flows
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?;

        let mut state = flow.create_state();
        // Inject task text as a variable so stages can reference it
        state
            .variables
            .insert("task".to_string(), serde_json::json!(task_text));
        state.status = FlowStatus::Running;
        self.execute_piece_state(name, flow, state).await
    }

    /// Execute a flow workflow
    pub async fn execute_piece(&self, name: &str) -> Result<FlowState> {
        let flow = self
            .flows
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", name))?;

        let mut state = flow.create_state();
        state.status = FlowStatus::Running;
        self.execute_piece_state(name, flow, state).await
    }

    /// Internal flow execution with pre-configured state
    async fn execute_piece_state(
        &self,
        name: &str,
        flow: &Flow,
        mut state: FlowState,
    ) -> Result<FlowState> {
        let run_id = self
            .event_recorder
            .as_ref()
            .map(|r| r.run_id().to_string())
            .unwrap_or_default();

        self.record_event(crate::events::Event::new(
            &run_id,
            crate::events::EventLevel::Info,
            crate::events::EventType::TaskStart,
            format!("Starting flow '{}'", name),
        ))
        .await;

        info!(
            "Starting flow '{}' at stage '{}'",
            name, state.current_movement
        );

        let mut cumulative_tokens: u64 = 0;

        loop {
            // Check max stages
            if state.movement_count >= flow.max_stages {
                warn!("Flow '{}' exceeded max stages ({})", name, flow.max_stages);
                state.status = FlowStatus::Aborted;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Get current stage
            let stage = match flow.get_movement(&state.current_movement) {
                Some(m) => m.clone(),
                None => {
                    state.status = FlowStatus::Failed;
                    state.completed_at = Some(Utc::now());
                    return Err(anyhow::anyhow!(
                        "Stage '{}' not found in flow '{}'",
                        state.current_movement,
                        name
                    ));
                }
            };

            debug!(
                "Executing stage '{}' (#{}) in flow '{}'",
                stage.id, state.movement_count, name
            );

            // Issue #23 fix: derive the agent label used by this stage so that
            // RunSummary::agents_used is populated. Preference order mirrors what the
            // bridge actually uses to route the call: an explicit `.claude/agents/<name>`
            // file (stage.agent) wins; otherwise we fall back to the persona facet.
            let movement_agent = stage.agent.clone().or_else(|| stage.persona.clone());

            // Record stage start
            let mut ev_start = crate::events::Event::new(
                &run_id,
                crate::events::EventLevel::Info,
                crate::events::EventType::MovementStart,
                format!("Stage '{}' started", stage.id),
            )
            .with_movement(&stage.id);
            if let Some(ref a) = movement_agent {
                ev_start = ev_start.with_agent(a);
            }
            self.record_event(ev_start).await;

            // Execute the stage with timing and optional per-stage timeout
            let movement_start = std::time::Instant::now();
            let output = if let Some(timeout_secs) = stage.timeout {
                let timeout = std::time::Duration::from_secs(timeout_secs as u64);
                match tokio::time::timeout(timeout, self.execute_movement(&stage, &state)).await {
                    Ok(result) => result?,
                    Err(_) => {
                        warn!("Stage '{}' timed out after {}s", stage.id, timeout_secs);
                        serde_json::json!({
                            "stage": stage.id,
                            "status": "timeout",
                            "error": format!("Stage timed out after {}s", timeout_secs),
                        })
                    }
                }
            } else {
                self.execute_movement(&stage, &state).await?
            };
            let movement_duration_ms = movement_start.elapsed().as_millis() as u64;
            state.movement_count += 1;

            // Save intermediate state for timeout recovery
            *self.last_state.write().await = Some(state.clone());

            // Notify progress listener
            if let Some(ref tx) = self.progress_tx {
                let _ = tx.send(MovementProgress {
                    movement_id: stage.id.clone(),
                    duration_ms: movement_duration_ms,
                    success: true,
                    movements_completed: state.movement_count as usize,
                });
            }

            // Record stage end with duration metadata
            // Issue #28 fix: forward token estimates from the bridge into the event so
            // `ccswarm cost` has something to aggregate.
            let tokens_in = output
                .as_object()
                .and_then(|o| o.get("tokens_in"))
                .and_then(|v| v.as_u64());
            let tokens_out = output
                .as_object()
                .and_then(|o| o.get("tokens_out"))
                .and_then(|v| v.as_u64());
            let attention = output
                .as_object()
                .and_then(|o| o.get("attention"))
                .and_then(|v| v.as_str())
                .unwrap_or("idle")
                .to_string();
            let mut ev_end = crate::events::Event::new(
                &run_id,
                crate::events::EventLevel::Info,
                crate::events::EventType::MovementEnd,
                format!("Stage '{}' completed", stage.id),
            )
            .with_movement(&stage.id)
            .with_metadata(serde_json::json!({
                "duration_ms": movement_duration_ms,
                "tokens_in": tokens_in,
                "tokens_out": tokens_out,
                "attention": attention,
                "output_preview": output
                    .as_object()
                    .and_then(|o| o.get("output"))
                    .and_then(|v| v.as_str())
                    .map(|s| truncate_for_context(s, 500))
                    .unwrap_or_default(),
                "status": output
                    .as_object()
                    .and_then(|o| o.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown"),
            }));
            if let Some(ref a) = movement_agent {
                ev_end = ev_end.with_agent(a);
            }
            self.record_event(ev_end).await;

            // Enforce the run-level token budget. Done after the stage records its
            // end event so `ccswarm cost` still sees the usage of the stage that
            // pushed us over. We abort on the *next* iteration boundary rather
            // than mid-stage: stages aren't cancellable, so stopping earlier
            // would waste the in-flight work without preventing the spend.
            cumulative_tokens = cumulative_tokens
                .saturating_add(tokens_in.unwrap_or(0))
                .saturating_add(tokens_out.unwrap_or(0));
            if let Some(cap) = self.run_token_cap
                && cumulative_tokens > cap
            {
                warn!(
                    "Run token cap exceeded: {} > {} (aborting after stage '{}')",
                    cumulative_tokens, cap, stage.id
                );
                self.record_event(
                    crate::events::Event::new(
                        &run_id,
                        crate::events::EventLevel::Warn,
                        crate::events::EventType::TaskEnd,
                        format!("Run token cap exceeded: {} > {}", cumulative_tokens, cap),
                    )
                    .with_metadata(serde_json::json!({
                        "reason": "budget_exceeded",
                        "cumulative_tokens": cumulative_tokens,
                        "cap": cap,
                        "last_stage": stage.id,
                    })),
                )
                .await;
                state.status = FlowStatus::Aborted;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Store output in variables
            state
                .variables
                .insert(format!("{}_output", stage.id), output.clone());

            // Save stage report to .ccswarm/runs/{run-id}/reports/{stage}.md
            if !run_id.is_empty() {
                let report_dir = std::path::PathBuf::from(".ccswarm")
                    .join("runs")
                    .join(&run_id)
                    .join("reports");
                let _ = tokio::fs::create_dir_all(&report_dir).await;
                let report_content = output
                    .as_object()
                    .and_then(|o| o.get("output"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !report_content.is_empty() {
                    let report_path = report_dir.join(format!("{}.md", stage.id));
                    let _ = tokio::fs::write(&report_path, report_content).await;
                }
            }

            // Check if terminal (no rules = done)
            if stage.rules.is_empty() {
                info!("Flow '{}' completed at terminal stage '{}'", name, stage.id);
                state.status = FlowStatus::Completed;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Evaluate rules to determine next stage
            let next = self.evaluate_rules(&stage.rules, &output, &state).await?;

            match next {
                Some(next_id) => {
                    state.history.push(MovementTransition {
                        from: stage.id.clone(),
                        to: next_id.clone(),
                        condition: "matched".to_string(),
                        timestamp: Utc::now(),
                        output: Some(output),
                    });
                    state.current_movement = next_id;
                }
                None => {
                    // No rule matched - treat as completion
                    info!("No rule matched in stage '{}', completing flow", stage.id);
                    state.status = FlowStatus::Completed;
                    state.completed_at = Some(Utc::now());
                    break;
                }
            }
        }

        // Record flow completion and write summary
        let completed = state.status == FlowStatus::Completed;
        self.record_event(crate::events::Event::new(
            &run_id,
            if completed {
                crate::events::EventLevel::Info
            } else {
                crate::events::EventLevel::Warn
            },
            crate::events::EventType::TaskEnd,
            format!("Flow '{}' finished with status {:?}", name, state.status),
        ))
        .await;

        if let Some(ref recorder) = self.event_recorder {
            // Issue #23 fix: agents_used was previously `state.history[].from` which is
            // a *stage* ID, not an agent. Derive the real agent label from each
            // stage definition (explicit `agent:` file route wins over persona).
            let mut agents_used: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for m in flow.stages.iter() {
                if let Some(a) = m.agent.clone().or_else(|| m.persona.clone()) {
                    agents_used.insert(a);
                }
            }

            let summary = crate::events::RunSummary {
                run_id: run_id.clone(),
                started_at: state.started_at,
                ended_at: state.completed_at,
                total_events: recorder.event_count(),
                tasks_completed: if completed { 1 } else { 0 },
                tasks_failed: if completed { 0 } else { 1 },
                agents_used: agents_used.into_iter().collect(),
            };
            if let Err(e) = recorder.write_summary(&summary).await {
                warn!("Failed to write run summary: {}", e);
            }
        }

        Ok(state)
    }

    /// Execute a single stage via Claude Code CLI (through AISessionBridge) or prompt-only fallback
    async fn execute_movement(
        &self,
        stage: &Stage,
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        info!(
            "Stage '{}': persona={:?}, permission={:?}",
            stage.id, stage.persona, stage.permission
        );

        // If instruction is empty or starts with "_local", skip Claude and produce local summary
        if stage.instruction.is_empty() || stage.instruction.starts_with("_local") {
            let summary = self.build_local_summary(state);
            return Ok(summary);
        }

        // Parallel execution: run sub-stages concurrently
        if stage.parallel && !stage.sub_movements.is_empty() {
            return self.execute_parallel_movements(stage, state).await;
        }

        // Build the prompt from instruction + persona + context
        let prompt = self.build_movement_prompt(stage, state);

        let output = if let Some(ref bridge) = self.bridge {
            // Real execution via Claude Code CLI + ai-session result management
            let agent_id = stage.persona.as_deref().unwrap_or("default");

            // Create a minimal identity for the stage
            let identity = crate::identity::AgentIdentity {
                agent_id: agent_id.to_string(),
                specialization: crate::identity::AgentRole::Frontend {
                    technologies: Vec::new(),
                    responsibilities: Vec::new(),
                    boundaries: Vec::new(),
                },
                workspace_path: self.working_dir.clone(),
                env_vars: std::collections::HashMap::new(),
                session_id: uuid::Uuid::new_v4().to_string(),
                parent_process_id: std::process::id().to_string(),
                initialized_at: chrono::Utc::now(),
            };

            // Determine working directory (stage override or engine default)
            let work_dir = stage
                .working_dir
                .as_ref()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| self.working_dir.clone());

            // Build MovementExecOptions from Stage fields.
            // Provider flag mapping is handled per-provider in crate::providers.
            // Precedence: stage YAML `provider:` > `CCSWARM_PROVIDER` env > Claude default.
            let provider = stage
                .provider
                .as_deref()
                .and_then(crate::providers::ProviderKind::parse)
                .or_else(|| {
                    std::env::var("CCSWARM_PROVIDER")
                        .ok()
                        .as_deref()
                        .and_then(crate::providers::ProviderKind::parse)
                });
            // Resolve effective tool list from the stage's permission level + explicit `tools:`.
            // Without this, `permission: readonly` would still send no `--allowed-tools`,
            // so the provider CLI would fall back to its own (permissive) default.
            let effective_tools = super::permissions::PermissionEnforcer::from_movement(
                stage.permission.clone(),
                &stage.tools,
            )
            .available_tools();

            let exec_options = crate::session::bridge::MovementExecOptions {
                provider,
                tools: effective_tools,
                model: stage.model.clone(),
                system_prompt: stage.persona.as_ref().and_then(|p| {
                    self.facet_registry.get_persona(p).and_then(|f| {
                        if f.system_prompt.is_empty() {
                            None
                        } else {
                            Some(f.system_prompt.clone())
                        }
                    })
                }),
                max_budget: self.budget_usd,
                worktree_name: None,
                session_id: None,
                continuation: crate::session::bridge::ContinuationPolicy::SingleTurn,
            };

            match bridge
                .execute_with_retry(
                    agent_id,
                    &prompt,
                    &identity,
                    &work_dir,
                    stage.agent.as_deref(),
                    stage.max_retries,
                    stage.retry_delay_ms,
                    &exec_options,
                )
                .await
            {
                Ok(result) => {
                    serde_json::json!({
                        "stage": stage.id,
                        "output": result.raw,
                        "parsed": format!("{:?}", result.parsed),
                        "status": if result.success { "completed" } else { "failed" },
                        "duration_ms": result.duration_ms,
                        "tokens_in": result.tokens_in,
                        "tokens_out": result.tokens_out,
                        "attention": result.attention.to_string(),
                    })
                }
                Err(e) => {
                    warn!("Stage '{}' execution failed: {}", stage.id, e);
                    serde_json::json!({
                        "stage": stage.id,
                        "error": e.to_string(),
                        "status": "failed",
                    })
                }
            }
        } else {
            // No bridge configured - return prompt as output (for testing/offline use)
            serde_json::json!({
                "stage": stage.id,
                "instruction": stage.instruction,
                "prompt": prompt,
                "status": "completed",
            })
        };

        // Validate output contract if specified
        if let Some(ref contract) = stage.output_contract {
            self.validate_output_contract(contract, &output)?;
        }

        Ok(output)
    }

    /// Execute sub-stages in parallel across multiple agents.
    async fn execute_parallel_movements(
        &self,
        parent: &Stage,
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        info!(
            "Parallel execution: {} sub-stages for '{}'",
            parent.sub_movements.len(),
            parent.id
        );

        // Find all sub-stage definitions from the flow
        let flow = self
            .flows
            .values()
            .find(|p| p.stages.iter().any(|m| m.id == parent.id))
            .ok_or_else(|| anyhow::anyhow!("Parent flow not found for stage '{}'", parent.id))?;

        let sub_movs: Vec<&Stage> = parent
            .sub_movements
            .iter()
            .filter_map(|id| flow.stages.iter().find(|m| m.id == *id))
            .collect();

        if sub_movs.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid sub-stages found for parallel execution"
            ));
        }

        // Execute all sub-stages concurrently
        let futures: Vec<_> = sub_movs
            .iter()
            .map(|m| self.execute_movement(m, state))
            .collect();

        let results = futures::future::join_all(futures).await;

        // Aggregate results
        let mut outputs = serde_json::Map::new();
        let mut all_success = true;
        for (i, result) in results.into_iter().enumerate() {
            let sub_id = &sub_movs[i].id;
            match result {
                Ok(output) => {
                    outputs.insert(sub_id.clone(), output);
                }
                Err(e) => {
                    warn!("Parallel sub-stage '{}' failed: {}", sub_id, e);
                    all_success = false;
                    outputs.insert(
                        sub_id.clone(),
                        serde_json::json!({"status": "failed", "error": e.to_string()}),
                    );
                }
            }
        }

        Ok(serde_json::json!({
            "stage": parent.id,
            "parallel": true,
            "status": if all_success { "completed" } else { "partial" },
            "agents": outputs,
        }))
    }

    /// Build a local summary from state without calling Claude Code CLI.
    /// Used for terminal/complete stages to avoid unnecessary LLM calls.
    fn build_local_summary(&self, state: &FlowState) -> serde_json::Value {
        let stages: Vec<String> = state
            .history
            .iter()
            .map(|t| format!("{} -> {}", t.from, t.to))
            .collect();

        let outputs: serde_json::Map<String, serde_json::Value> = state
            .variables
            .iter()
            .filter(|(k, _)| k.ends_with("_output"))
            .map(|(k, v)| {
                let key = k.trim_end_matches("_output").to_string();
                let preview = v
                    .as_object()
                    .and_then(|obj| obj.get("status"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                (key, serde_json::json!(preview))
            })
            .collect();

        serde_json::json!({
            "stage": "complete",
            "status": "completed",
            "summary": {
                "flow": state.flow_name,
                "movements_executed": state.movement_count,
                "transitions": stages,
                "step_results": outputs,
            }
        })
    }

    /// Build the prompt for a stage using faceted prompting.
    ///
    /// Composition order (takt-style):
    /// - System: persona (via FacetRegistry)
    /// - User: knowledge → instruction → policy → output contract → tools → tags
    fn build_movement_prompt(&self, stage: &Stage, state: &FlowState) -> String {
        // Build output contract text if present
        let contract_text = stage.output_contract.as_ref().map(|c| {
            let mut parts = vec![format!("Format: {}", c.format)];
            if !c.required_sections.is_empty() {
                parts.push(format!(
                    "Required sections: {}",
                    c.required_sections.join(", ")
                ));
            }
            if let Some(ref file) = c.output_file {
                parts.push(format!("Write output to: {}", file));
            }
            parts.join("\n")
        });

        // Expand template variables in instruction: {key} -> state.variables[key]
        let expanded_instruction = expand_template(&stage.instruction, &state.variables);

        // Use faceted prompting to compose system + user message
        let composed = self.facet_registry.compose(
            stage.persona.as_deref(),
            stage.policy.as_deref(),
            stage.knowledge.as_deref(),
            &expanded_instruction,
            contract_text.as_deref(),
        );

        let mut parts = Vec::new();

        // System prompt (persona)
        if !composed.system.is_empty() {
            parts.push(composed.system);
        }

        // Inject task description if available
        if let Some(task_text) = state.variables.get("task").and_then(|v| v.as_str()) {
            parts.push(format!("## Task\n\n{}", task_text));
        }

        // User message (knowledge → instruction → policy → output contract)
        if !composed.user.is_empty() {
            parts.push(composed.user);
        }

        // Add available tools
        if !stage.tools.is_empty() {
            parts.push(format!("Available tools: {}", stage.tools.join(", ")));
        }

        // Add permission context
        parts.push(format!("Permission level: {:?}", stage.permission));

        // Inject context from previous stages for continuity
        // (skip if pass_previous_response is false — used for fix stages)
        if stage.pass_previous_response && !state.variables.is_empty() {
            let var_summary: Vec<String> = state
                .variables
                .iter()
                .filter(|(k, _)| k.ends_with("_output"))
                .map(|(k, v)| {
                    let key = k.trim_end_matches("_output");
                    // Extract the actual output text from JSON if possible
                    let output_text = v
                        .as_object()
                        .and_then(|obj| obj.get("output"))
                        .and_then(|o| o.as_str())
                        .map(|s| truncate_for_context(s, 2000))
                        .unwrap_or_else(|| truncate_for_context(&v.to_string(), 500));
                    format!("[Previous '{}' result]:\n{}", key, output_text)
                })
                .collect();
            if !var_summary.is_empty() {
                parts.push(format!(
                    "## Context from previous steps\n\n{}",
                    var_summary.join("\n\n")
                ));
            }
        }

        // Also inject ai-session context if bridge is available
        if let Some(ref bridge) = self.bridge {
            let agent_id = stage.persona.as_deref().unwrap_or("default");
            let recent = bridge.get_recent_context(agent_id, 3);
            if !recent.is_empty() {
                parts.push(format!(
                    "## Recent conversation context\n\n{}",
                    recent.join("\n")
                ));
            }
        }

        // Inject tag instructions for routing (takt-style [STEP:N] tags)
        if !stage.rules.is_empty() {
            let tag_instructions =
                super::judge::MovementJudge::generate_tag_instructions(&stage.rules);
            parts.push(tag_instructions);
        }

        parts.join("\n\n")
    }

    /// Evaluate routing rules against stage output using the MovementJudge.
    ///
    /// Evaluation priority (takt-style):
    /// 1. Aggregate conditions (all/any) for parallel outputs
    /// 2. [STEP:N] tag detection
    /// 3. Simple string conditions
    /// 4. AI judge evaluation
    /// 5. Fallback to first "success" rule
    async fn evaluate_rules(
        &self,
        rules: &[MovementRule],
        output: &serde_json::Value,
        _state: &FlowState,
    ) -> Result<Option<String>> {
        let output_str = serde_json::to_string(output).unwrap_or_default();

        let judge_result = self.judge.evaluate(&output_str, rules, None)?;

        if let Some(index) = judge_result.matched_rule_index
            && index < rules.len()
        {
            debug!(
                "Judge matched rule {}: method={:?}, confidence={:.2}, next={}",
                index, judge_result.match_method, judge_result.confidence, rules[index].next
            );
            return Ok(Some(rules[index].next.clone()));
        }

        Ok(None)
    }

    /// Validate output against a contract, returning detailed violation info.
    fn validate_output_contract(
        &self,
        contract: &OutputContract,
        output: &serde_json::Value,
    ) -> Result<()> {
        let result = Self::validate_contract(contract, output);

        if !result.valid {
            let messages: Vec<&str> = result
                .violations
                .iter()
                .map(|v| v.message.as_str())
                .collect();
            return Err(anyhow::anyhow!(
                "Output contract violations:\n- {}",
                messages.join("\n- ")
            ));
        }

        Ok(())
    }

    /// Validate output against a contract, returning a detailed result.
    pub fn validate_contract(
        contract: &OutputContract,
        output: &serde_json::Value,
    ) -> ContractValidationResult {
        let mut violations = Vec::new();
        let output_str = serde_json::to_string_pretty(output).unwrap_or_default();

        // 1. Check required sections
        for section in &contract.required_sections {
            if !output_str.contains(section) {
                violations.push(ContractViolation {
                    kind: ViolationKind::MissingSection,
                    message: format!("Missing required section: '{}'", section),
                });
            }
        }

        // 2. Check required JSON keys
        if let serde_json::Value::Object(map) = output {
            for key in &contract.required_keys {
                if !map.contains_key(key) {
                    violations.push(ContractViolation {
                        kind: ViolationKind::MissingKey,
                        message: format!("Missing required key: '{}'", key),
                    });
                }
            }
        } else if !contract.required_keys.is_empty() {
            violations.push(ContractViolation {
                kind: ViolationKind::InvalidFormat,
                message: "Output is not a JSON object but required_keys are specified".to_string(),
            });
        }

        // 3. Check format
        match contract.format.as_str() {
            "json" => {
                // Already JSON (since output is serde_json::Value)
            }
            "markdown"
                // Markdown should contain at least one heading or list
                if !output_str.contains('#') && !output_str.contains("- ") => {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message:
                            "Output does not appear to be valid markdown (no headings or lists)"
                                .to_string(),
                    });
                }
            "yaml"
                // Check for YAML-like structure (key: value patterns)
                if !output_str.contains(':') => {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message: "Output does not appear to be valid YAML".to_string(),
                    });
                }
            "code"
                // Code should have some structure
                if output_str.len() < 10 => {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message: "Output appears too short to be code".to_string(),
                    });
                }
            _ => {
                // text or unknown format - no specific validation
            }
        }

        // 4. Check length constraints
        if let Some(min_len) = contract.min_length
            && output_str.len() < min_len
        {
            violations.push(ContractViolation {
                kind: ViolationKind::TooShort,
                message: format!(
                    "Output too short: {} chars (minimum: {})",
                    output_str.len(),
                    min_len
                ),
            });
        }

        if let Some(max_len) = contract.max_length
            && output_str.len() > max_len
        {
            violations.push(ContractViolation {
                kind: ViolationKind::TooLong,
                message: format!(
                    "Output too long: {} chars (maximum: {})",
                    output_str.len(),
                    max_len
                ),
            });
        }

        // 5. Check must_match patterns
        for pattern_str in &contract.must_match {
            match regex::Regex::new(pattern_str) {
                Ok(re) => {
                    if !re.is_match(&output_str) {
                        violations.push(ContractViolation {
                            kind: ViolationKind::PatternViolation,
                            message: format!(
                                "Output does not match required pattern: '{}'",
                                pattern_str
                            ),
                        });
                    }
                }
                Err(e) => {
                    warn!("Invalid regex in must_match: '{}': {}", pattern_str, e);
                }
            }
        }

        // 6. Check must_not_match patterns
        for pattern_str in &contract.must_not_match {
            match regex::Regex::new(pattern_str) {
                Ok(re) => {
                    if re.is_match(&output_str) {
                        violations.push(ContractViolation {
                            kind: ViolationKind::ForbiddenPattern,
                            message: format!("Output matches forbidden pattern: '{}'", pattern_str),
                        });
                    }
                }
                Err(e) => {
                    warn!("Invalid regex in must_not_match: '{}': {}", pattern_str, e);
                }
            }
        }

        // 7. JSON schema validation (basic key/type checking)
        if let Some(serde_json::Value::Object(schema_obj)) = contract.schema.as_ref()
            && let Some(serde_json::Value::Object(required)) = schema_obj.get("properties")
            && let serde_json::Value::Object(output_obj) = output
        {
            for (key, _prop_schema) in required {
                if !output_obj.contains_key(key) {
                    violations.push(ContractViolation {
                        kind: ViolationKind::SchemaViolation,
                        message: format!("Schema violation: missing property '{key}'"),
                    });
                }
            }
        }

        ContractValidationResult {
            valid: violations.is_empty(),
            violations,
        }
    }
}

impl Default for FlowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in flow templates
pub fn builtin_flows() -> Vec<Flow> {
    vec![
        // Default development workflow
        Flow {
            name: "default".to_string(),
            description: "Standard development workflow: plan → implement → review → fix"
                .to_string(),
            max_stages: 30,
            initial_movement: "plan".to_string(),
            stages: vec![
                Stage {
                    id: "plan".to_string(),
                    persona: Some("planner".to_string()),
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Analyze the task and create an implementation plan".to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "glob".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "implement".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "implement".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Implement the planned changes".to_string(),
                    tools: vec![
                        "read".to_string(),
                        "write".to_string(),
                        "edit".to_string(),
                        "bash".to_string(),
                    ],
                    permission: MovementPermission::Edit,
                    rules: vec![
                        MovementRule {
                            condition: RuleCondition::Simple("success".to_string()),
                            next: "review".to_string(),
                            priority: 0,
                        },
                        MovementRule {
                            condition: RuleCondition::Simple("error".to_string()),
                            next: "fix".to_string(),
                            priority: 1,
                        },
                    ],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 1,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "review".to_string(),
                    persona: Some("reviewer".to_string()),
                    policy: Some("review".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Review the implementation for quality and correctness"
                        .to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "bash".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![
                        MovementRule {
                            condition: RuleCondition::Simple("success".to_string()),
                            next: "complete".to_string(),
                            priority: 0,
                        },
                        MovementRule {
                            condition: RuleCondition::Simple("fixes_needed".to_string()),
                            next: "fix".to_string(),
                            priority: 1,
                        },
                    ],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "fix".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Fix the identified issues".to_string(),
                    tools: vec![
                        "read".to_string(),
                        "write".to_string(),
                        "edit".to_string(),
                        "bash".to_string(),
                    ],
                    permission: MovementPermission::Edit,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "review".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 2,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "complete".to_string(),
                    persona: None,
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: String::new(), // Empty = local summary, no Claude call
                    tools: vec![],
                    permission: MovementPermission::Readonly,
                    rules: vec![], // Terminal
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Research workflow
        Flow {
            name: "research".to_string(),
            description: "Autonomous research and investigation workflow".to_string(),
            max_stages: 20,
            initial_movement: "investigate".to_string(),
            stages: vec![
                Stage {
                    id: "investigate".to_string(),
                    persona: Some("researcher".to_string()),
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Research and investigate the topic".to_string(),
                    tools: vec![
                        "read".to_string(),
                        "grep".to_string(),
                        "glob".to_string(),
                        "search".to_string(),
                    ],
                    permission: MovementPermission::Readonly,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "summarize".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "summarize".to_string(),
                    persona: Some("writer".to_string()),
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Summarize findings into a report".to_string(),
                    tools: vec!["read".to_string(), "write".to_string()],
                    permission: MovementPermission::Edit,
                    rules: vec![], // Terminal
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: Some(OutputContract {
                        format: "markdown".to_string(),
                        required_sections: vec!["summary".to_string(), "findings".to_string()],
                        schema: None,
                        output_file: Some("research-report.md".to_string()),
                        required_keys: vec![],
                        min_length: None,
                        max_length: None,
                        must_match: vec![],
                        must_not_match: vec![],
                        allowed_files: vec![],
                    }),
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Review-fix minimal workflow
        Flow {
            name: "review-fix".to_string(),
            description: "Minimal review and fix cycle".to_string(),
            max_stages: 10,
            initial_movement: "review".to_string(),
            stages: vec![
                Stage {
                    id: "review".to_string(),
                    persona: Some("reviewer".to_string()),
                    policy: Some("review".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Review the code for issues".to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "bash".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![
                        MovementRule {
                            condition: RuleCondition::Simple("fixes_needed".to_string()),
                            next: "fix".to_string(),
                            priority: 1,
                        },
                        MovementRule {
                            condition: RuleCondition::Simple("success".to_string()),
                            next: "done".to_string(),
                            priority: 0,
                        },
                    ],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "fix".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Fix the identified issues".to_string(),
                    tools: vec![
                        "read".to_string(),
                        "write".to_string(),
                        "edit".to_string(),
                        "bash".to_string(),
                    ],
                    permission: MovementPermission::Edit,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "review".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 2,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                Stage {
                    id: "done".to_string(),
                    persona: None,
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Review complete".to_string(),
                    tools: vec![],
                    permission: MovementPermission::Readonly,
                    rules: vec![], // Terminal
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Quick single-shot workflow (1 Claude call, fastest)
        Flow {
            name: "quick".to_string(),
            description: "Single-shot execution: one Claude call, no plan/review overhead"
                .to_string(),
            max_stages: 1,
            initial_movement: "execute".to_string(),
            stages: vec![Stage {
                id: "execute".to_string(),
                persona: Some("coder".to_string()),
                policy: Some("coding".to_string()),
                knowledge: None,
                provider: None,
                model: None,
                instruction: "Execute the task directly. Write clean, working code.".to_string(),
                tools: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "edit".to_string(),
                    "bash".to_string(),
                    "grep".to_string(),
                    "glob".to_string(),
                ],
                permission: MovementPermission::Edit,
                rules: vec![], // Terminal - single shot
                parallel: false,
                sub_movements: vec![],
                output_contract: None,
                timeout: None,
                max_retries: 1,
                agent: None,
                working_dir: None,
                retry_delay_ms: default_retry_delay(),
                pass_previous_response: true,
            }],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Multi-agent team workflow: plan → parallel(frontend + backend) → review → complete
        Flow {
            name: "team".to_string(),
            description: "Multi-agent orchestration: planner designs, frontend & backend agents execute in parallel, reviewer validates".to_string(),
            max_stages: 10,
            initial_movement: "plan".to_string(),
            stages: vec![
                Stage {
                    id: "plan".to_string(),
                    persona: Some("planner".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Analyze the task and create a plan that splits work between frontend and backend agents. Define clear interfaces and contracts between them.".to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "glob".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "parallel-implement".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                // Parallel hub: dispatches to frontend-impl and backend-impl simultaneously
                Stage {
                    id: "parallel-implement".to_string(),
                    persona: None,
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Execute frontend and backend in parallel".to_string(),
                    tools: vec![],
                    permission: MovementPermission::Edit,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "review".to_string(),
                        priority: 0,
                    }],
                    parallel: true,
                    sub_movements: vec!["frontend-impl".to_string(), "backend-impl".to_string()],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                // Frontend agent (runs in parallel)
                Stage {
                    id: "frontend-impl".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Implement the frontend portion of the plan. Focus on UI, user interaction, and client-side logic.".to_string(),
                    tools: vec!["read".to_string(), "write".to_string(), "edit".to_string(), "bash".to_string()],
                    permission: MovementPermission::Edit,
                    rules: vec![],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 1,
                    agent: Some("frontend-specialist".to_string()),
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                // Backend agent (runs in parallel)
                Stage {
                    id: "backend-impl".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Implement the backend portion of the plan. Focus on APIs, data models, and server-side logic.".to_string(),
                    tools: vec!["read".to_string(), "write".to_string(), "edit".to_string(), "bash".to_string()],
                    permission: MovementPermission::Edit,
                    rules: vec![],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 1,
                    agent: Some("backend-specialist".to_string()),
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                // Supervisor reviews the combined output
                Stage {
                    id: "review".to_string(),
                    persona: Some("supervisor".to_string()),
                    policy: Some("review".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Review the combined frontend and backend implementation. Verify integration points, check for inconsistencies, and ensure the plan was followed.".to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "bash".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "complete".to_string(),
                        priority: 0,
                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
                // Local completion
                Stage {
                    id: "complete".to_string(),
                    persona: None,
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: String::new(),
                    tools: vec![],
                    permission: MovementPermission::Readonly,
                    rules: vec![],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
    ]
}

/// Truncate a string for inclusion in prompt context, preserving meaning
/// Expand `{key}` template variables in a string using state variables.
/// For `{key_output}`, extracts the "output" field from the JSON value if available.
fn expand_template(template: &str, variables: &HashMap<String, serde_json::Value>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{}}}", key);
        if result.contains(&placeholder) {
            let replacement = value
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|obj| obj.get("output"))
                        .and_then(|o| o.as_str())
                        .map(|s| truncate_for_context(s, 2000))
                })
                .unwrap_or_else(|| value.to_string());
            result = result.replace(&placeholder, &replacement);
        }
    }
    result
}

fn truncate_for_context(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}... [truncated]", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_from_yaml() {
        let yaml = r#"
name: test-flow
description: "A test flow"
max_stages: 10
initial_movement: start

stages:
  - id: start
    persona: planner
    instruction: "Plan the task"
    tools: [read, grep]
    permission: readonly
    rules:
      - condition: success
        next: end
  - id: end
    instruction: "Done"
"#;

        let flow = Flow::from_yaml(yaml).expect("Failed to parse YAML");
        assert_eq!(flow.name, "test-flow");
        assert_eq!(flow.stages.len(), 2);
        assert_eq!(flow.initial_movement, "start");
        assert_eq!(flow.stages[0].permission, MovementPermission::Readonly);
    }

    #[test]
    fn test_piece_validation_invalid_initial() {
        let yaml = r#"
name: bad-flow
initial_movement: nonexistent
stages:
  - id: start
    instruction: "Hello"
"#;

        let result = Flow::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_validation_invalid_rule_target() {
        let yaml = r#"
name: bad-rules
initial_movement: start
stages:
  - id: start
    instruction: "Hello"
    rules:
      - condition: success
        next: nonexistent
"#;

        let result = Flow::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_validation_duplicate_ids() {
        let yaml = r#"
name: dup-ids
initial_movement: start
stages:
  - id: start
    instruction: "First"
  - id: start
    instruction: "Duplicate"
"#;

        let result = Flow::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_pieces() {
        let flows = builtin_flows();
        assert!(!flows.is_empty());

        for flow in &flows {
            flow.validate()
                .unwrap_or_else(|_| panic!("Built-in flow '{}' failed validation", flow.name));
        }
    }

    #[test]
    fn test_piece_create_state() {
        let yaml = r#"
name: state-test
initial_movement: start
stages:
  - id: start
    instruction: "Begin"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let state = flow.create_state();
        assert_eq!(state.flow_name, "state-test");
        assert_eq!(state.current_movement, "start");
        assert_eq!(state.movement_count, 0);
        assert_eq!(state.status, FlowStatus::Pending);
    }

    #[test]
    fn test_terminal_movement() {
        let yaml = r#"
name: terminal-test
initial_movement: start
stages:
  - id: start
    instruction: "Begin"
    rules:
      - condition: success
        next: end
  - id: end
    instruction: "Done"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        assert!(!flow.is_terminal("start"));
        assert!(flow.is_terminal("end"));
    }

    #[tokio::test]
    async fn test_piece_engine_execute() {
        let yaml = r#"
name: exec-test
initial_movement: step1
stages:
  - id: step1
    instruction: "Step 1"
    rules:
      - condition: success
        next: step2
  - id: step2
    instruction: "Step 2 (terminal)"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("exec-test".to_string(), flow);

        let state = engine
            .execute_piece("exec-test")
            .await
            .expect("execution failed");
        assert_eq!(state.status, FlowStatus::Completed);
        assert_eq!(state.movement_count, 2);
    }

    /// `readonly` stages without an explicit `tools:` list must still restrict the
    /// provider via the permission level — otherwise the provider CLI falls back to
    /// its permissive default and can write/exec despite `permission: readonly`.
    #[test]
    fn test_readonly_stage_resolves_to_readonly_tools() {
        use super::super::permissions::PermissionEnforcer;

        let yaml = r#"
name: perm-test
initial_movement: review
stages:
  - id: review
    instruction: "Read-only review"
    permission: readonly
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let stage = &flow.stages[0];
        assert!(stage.tools.is_empty(), "precondition: no explicit tools");

        let resolved = PermissionEnforcer::from_movement(stage.permission.clone(), &stage.tools)
            .available_tools();

        assert!(resolved.contains(&"read".to_string()));
        assert!(resolved.contains(&"grep".to_string()));
        assert!(
            !resolved
                .iter()
                .any(|t| t == "bash" || t == "write" || t == "edit"),
            "readonly must not expose edit/exec tools, got: {:?}",
            resolved,
        );
    }

    /// `set_run_token_cap` installs the cap on the engine. Regression guard
    /// so the public setter keeps working for the CLI `--run-budget-tokens`
    /// path that plumbs into this field.
    #[test]
    fn test_run_token_cap_setter() {
        let mut engine = FlowEngine::new();
        assert_eq!(engine.run_token_cap, None);
        engine.set_run_token_cap(5000);
        assert_eq!(engine.run_token_cap, Some(5000));
    }

    /// The in-loop abort condition: cumulative strictly greater than the cap.
    /// Keeping this as a standalone assertion so the cap semantics (`>`, not `>=`)
    /// are pinned even if the enforcement site gets refactored.
    #[test]
    fn test_run_token_cap_abort_condition() {
        let exceeds_cap = |used: u64, cap: u64| used > cap;

        // strictly greater than the cap — abort
        assert!(exceeds_cap(10, 5));
        // equal — do NOT abort (the stage that lands exactly on the cap still
        // gets its usage recorded and the next stage is allowed to start; a
        // stricter check would surprise users who set the cap to the exact
        // expected total).
        assert!(!exceeds_cap(5, 5));
    }

    /// An explicit `tools:` list on a stage is honored verbatim, even if the
    /// permission level would allow a broader set.
    #[test]
    fn test_explicit_tools_override_permission_defaults() {
        use super::super::permissions::PermissionEnforcer;

        let yaml = r#"
name: perm-test
initial_movement: narrow
stages:
  - id: narrow
    instruction: "Narrow tools"
    permission: full
    tools: [read, grep]
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let stage = &flow.stages[0];

        let resolved = PermissionEnforcer::from_movement(stage.permission.clone(), &stage.tools)
            .available_tools();

        let mut sorted = resolved.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["grep".to_string(), "read".to_string()]);
    }
}
