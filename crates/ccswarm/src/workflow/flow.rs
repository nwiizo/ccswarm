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

    /// Maximum *total* stage executions across the whole run before abort
    /// (complementary to `max_stage_visits`, which bounds each stage
    /// individually)
    #[serde(default = "default_max_movements")]
    pub max_stages: u32,

    /// Maximum visits per *individual* stage before abort (bounds review→fix loops)
    #[serde(default = "default_max_stage_visits")]
    pub max_stage_visits: u32,

    /// Fallback providers to try, in order, when a provider call fails with a
    /// rate-limit error (takt's `rate_limit_fallback.switch_chain`). Empty by
    /// default: rate limits surface as ordinary provider errors.
    #[serde(default)]
    pub on_rate_limit: Vec<FallbackTarget>,

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

fn default_max_stage_visits() -> u32 {
    3
}

/// One entry in a flow's `on_rate_limit` fallback chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackTarget {
    /// Provider to switch to (claude | codex).
    pub provider: String,
    /// Optional model override for the fallback provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
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

    /// Invoke another flow as this stage's body. When set, the stage's
    /// `instruction` / `persona` / `policy` are ignored and the named child
    /// flow runs end-to-end; the child's final output becomes this stage's
    /// output, which downstream rules then evaluate as usual. Adopted from
    /// takt's `kind: workflow_call`.
    ///
    /// Variables in `call.args` are injected into the child's initial state
    /// after template-expanding values against the parent's variables. The
    /// child writes reports into the same `.ccswarm/runs/<id>/reports/`
    /// directory as the parent; declared report names must be globally
    /// unique within a run (v0.7.0 first-cut; namespacing is deferred).
    #[serde(default)]
    pub call: Option<WorkflowCallSpec>,

    /// Provider/model escalation rules applied from the Nth visit of this
    /// stage onward (takt's `promotion`). The last matching entry wins.
    /// Ignored on parallel sub-stages, where visit counts track the parent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promotion: Vec<PromotionRule>,

    /// Machine-executed quality gates run after the agent completes this
    /// stage (takt's command quality gates). On failure, bounded command
    /// output is appended to the instruction and the stage re-runs, up to
    /// `max_retries` additional attempts. Gates run in the stage's working
    /// directory. Flow YAML already executes arbitrary edit-permission
    /// prompts, so gates add no new trust surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<CommandGate>,

    /// Orchestrator-worker decomposition (takt's `team_leader`): a leader
    /// call splits this stage's task into parts at runtime, the parts execute
    /// concurrently as synthesized worker stages, and their outputs aggregate
    /// into the parallel shape so `all()`/`any()` rules work unchanged.
    /// Mutually exclusive with `parallel` and `call`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_leader: Option<super::team_leader::TeamLeaderSpec>,

    /// Sangha consensus round: multiple independent members evaluate the
    /// stage and quorum decides whether the stage advances. Mutually
    /// exclusive with `parallel`, `call`, and `team_leader`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sangha: Option<super::sangha::SanghaSpec>,
}

/// One machine-executed gate command on a stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGate {
    /// Display name, used in events and failure feedback.
    pub name: String,
    /// Shell command executed via `sh -c` in the stage's working directory.
    pub command: String,
    /// Per-gate timeout in seconds (default 300).
    #[serde(default = "default_gate_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_gate_timeout_secs() -> u64 {
    300
}

/// One provider/model escalation rule on a stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRule {
    /// Applies from this visit number onward (1 = first execution).
    pub at: u32,
    /// Provider to switch to (claude | codex). `None` keeps the current one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Model to switch to. `None` keeps the current one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Specification for invoking another flow as a single stage's body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCallSpec {
    /// Name of the child flow to invoke. Must be registered with the
    /// `FlowEngine` (built-in or loaded). Unknown names fail the stage.
    pub flow: String,

    /// Variables to seed into the child flow's initial state. Values are
    /// expanded against the parent's variables (e.g. `"task": "{task}"` pipes
    /// the parent's `task` through).
    #[serde(default)]
    pub args: HashMap<String, String>,
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

    /// Skip this rule entirely in non-interactive (pipeline / queue drain / CI)
    /// runs. Use for review-fix loops that only make sense when a human is in
    /// the loop. Adopted from takt's `interactive_only` rule field.
    #[serde(default)]
    pub interactive_only: bool,

    /// Indicates the rule's `next` stage expects human input. Pipeline runs
    /// treat this rule as inert (same as `interactive_only`); interactive runs
    /// flag the upcoming stage as awaiting user reply so the UI can prompt.
    /// Adopted from takt's `requires_user_input` rule field.
    #[serde(default)]
    pub requires_user_input: bool,
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
#[derive(Debug, Clone, Deserialize)]
pub enum CompoundCondition {
    /// All conditions must match
    #[serde(rename = "all")]
    All(Vec<String>),
    /// Any condition must match
    #[serde(rename = "any")]
    Any(Vec<String>),
}

// Hand-written Serialize: the derived (externally tagged) form serializes to
// a YAML `!all`/`!any` tag, which the untagged `RuleCondition` wrapper cannot
// re-parse — breaking the serialize→parse round trip `flow check` does on
// builtin flows. Emitting a single-entry map (`all: [..]`) matches the YAML
// authors write and what the derived Deserialize accepts.
impl Serialize for CompoundCondition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::All(conditions) => map.serialize_entry("all", conditions)?,
            Self::Any(conditions) => map.serialize_entry("any", conditions)?,
        }
        map.end()
    }
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

    /// Named reports the stage produces under `.ccswarm/runs/<run-id>/reports/`.
    ///
    /// Each entry maps to a deterministically named file (e.g. `plan.md`) that
    /// downstream stages can reference via the `{report:<name>}` template
    /// variable. Adopted from takt's `output_contracts.report` to replace the
    /// brittle `{plan_output}` state-variable wiring with a contract that's
    /// readable from disk after the run.
    ///
    /// v0.7.0 semantics: if exactly one report is declared, the stage's full
    /// response is written verbatim to it. Multi-report support (with AI-emitted
    /// `<<<REPORT:name>>>` delimiters) is intentionally deferred.
    #[serde(default)]
    pub reports: Vec<ReportContract>,
}

/// A named report file a stage produces under the run's `reports/` directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportContract {
    /// File name written under `.ccswarm/runs/<run-id>/reports/`. Must not contain
    /// path separators — validated at flow-load time.
    pub name: String,

    /// Format hint surfaced in the prompt to guide the model (e.g. `markdown`,
    /// `json`, `text`).
    #[serde(default = "default_format")]
    pub format: String,

    /// Optional one-line description rendered into the prompt's output-contract
    /// block to clarify intent (e.g. "investigation summary").
    #[serde(default)]
    pub description: Option<String>,
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

        for target in &self.on_rate_limit {
            if crate::providers::ProviderKind::parse(&target.provider).is_none() {
                return Err(anyhow::anyhow!(
                    "Flow '{}' has unknown on_rate_limit provider '{}' (expected: claude | codex | copilot)",
                    self.name,
                    target.provider
                ));
            }
        }

        // Check all rule targets reference valid stages
        for stage in &self.stages {
            if let Some(provider) = &stage.provider
                && crate::providers::ProviderKind::parse(provider).is_none()
            {
                return Err(anyhow::anyhow!(
                    "Stage '{}' has unknown provider '{}' (expected: claude | codex | copilot)",
                    stage.id,
                    provider
                ));
            }

            for rule in &stage.promotion {
                if let Some(provider) = &rule.provider
                    && crate::providers::ProviderKind::parse(provider).is_none()
                {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' has unknown promotion provider '{}' (expected: claude | codex | copilot)",
                        stage.id,
                        provider
                    ));
                }
            }

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

            // team_leader is its own execution mode — combining it with
            // declared-parallel or workflow_call would be ambiguous.
            if let Some(spec) = &stage.team_leader {
                if stage.parallel {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' combines team_leader with parallel — pick one",
                        stage.id
                    ));
                }
                if stage.call.is_some() {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' combines team_leader with call — pick one",
                        stage.id
                    ));
                }
                if spec.max_parts == 0 {
                    return Err(anyhow::anyhow!(
                        "Stage '{}': team_leader.max_parts must be >= 1",
                        stage.id
                    ));
                }
            }

            // sangha is its own execution mode — combining consensus with
            // other body execution modes would make routing ambiguous.
            if let Some(spec) = &stage.sangha {
                if stage.parallel {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' combines sangha with parallel — pick one",
                        stage.id
                    ));
                }
                if stage.call.is_some() {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' combines sangha with call — pick one",
                        stage.id
                    ));
                }
                if stage.team_leader.is_some() {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' combines sangha with team_leader — pick one",
                        stage.id
                    ));
                }
                if spec.quorum == 0 {
                    return Err(anyhow::anyhow!(
                        "Stage '{}': sangha.quorum must be >= 1",
                        stage.id
                    ));
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
    /// Whether this engine instance runs in interactive mode. Pipeline / queue
    /// drain / CI runs default to `false` and skip rules tagged
    /// `interactive_only` or `requires_user_input`. Interactive entry points
    /// (e.g. `ccswarm` with no subcommand) call `set_interactive(true)` so
    /// human-in-the-loop rules are honored. Adopted from takt's
    /// interactive_only / requires_user_input rule fields.
    interactive: bool,
    /// Default provider for stages that don't pin one in flow YAML
    /// (`--provider` flag). Sits between stage YAML and the CCSWARM_PROVIDER
    /// env var in the resolution order.
    default_provider: Option<crate::providers::ProviderKind>,
    /// CLI model override applied to every live stage.
    model_override: Option<String>,
    /// Optional isolated worktree name forwarded through AISessionBridge.
    worktree_name: Option<String>,
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
            interactive: false,
            default_provider: None,
            model_override: None,
            worktree_name: None,
        }
    }

    /// Mark this engine as running in interactive mode. Pipeline / queue drain
    /// runs leave the default (`false`), which filters out rules tagged
    /// `interactive_only` or `requires_user_input`.
    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
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

    /// Set the default provider for stages that don't pin one in flow YAML
    /// (`--provider` flag). Overrides the CCSWARM_PROVIDER env var.
    pub(crate) fn set_default_provider(&mut self, provider: crate::providers::ProviderKind) {
        self.default_provider = Some(provider);
    }

    /// Set a CLI model override for all stages in this engine run.
    pub(crate) fn set_model_override(&mut self, model: impl Into<String>) {
        self.model_override = Some(model.into());
    }

    /// Set the provider worktree isolation name for live stage execution.
    pub(crate) fn set_worktree_name(&mut self, name: impl Into<String>) {
        self.worktree_name = Some(name.into());
    }

    /// Resolve the provider and model a stage execution should use.
    ///
    /// Base precedence: stage YAML `provider:` > `--provider` flag >
    /// `CCSWARM_PROVIDER` env > Claude default (stage YAML wins because it
    /// expresses deliberate per-stage intent). On top of that, `promotion`
    /// rules escalate provider/model from the Nth visit of the stage onward
    /// (takt-style, last matching entry wins). Promotion is skipped when no
    /// visit count is available — notably for parallel sub-stages, whose
    /// count would otherwise reflect the parent stage.
    fn resolve_effective_provider(
        &self,
        stage: &Stage,
        state: &FlowState,
    ) -> (Option<crate::providers::ProviderKind>, Option<String>) {
        let mut provider = stage
            .provider
            .as_deref()
            .and_then(crate::providers::ProviderKind::parse)
            .or(self.default_provider)
            .or_else(|| {
                std::env::var("CCSWARM_PROVIDER")
                    .ok()
                    .as_deref()
                    .and_then(crate::providers::ProviderKind::parse)
            });
        let mut model = stage.model.clone();

        let visit_count = state
            .variables
            .get("__visit_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        if visit_count >= 1 {
            // Reverse scan = last matching entry wins (takt semantics).
            if let Some(rule) = stage
                .promotion
                .iter()
                .rev()
                .find(|rule| visit_count >= rule.at.max(1))
            {
                info!(
                    "Stage '{}' visit {} matched promotion rule (at: {}): provider={:?} model={:?}",
                    stage.id, visit_count, rule.at, rule.provider, rule.model
                );
                if let Some(p) = rule
                    .provider
                    .as_deref()
                    .and_then(crate::providers::ProviderKind::parse)
                {
                    provider = Some(p);
                }
                if rule.model.is_some() {
                    model = rule.model.clone();
                }
            }
        }

        if self.model_override.is_some() {
            model = self.model_override.clone();
        }

        (provider, model)
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
    #[tracing::instrument(name = "flow.run", skip_all, fields(flow = %name))]
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

        // Stash run_id in state so expand_template can resolve `{report:<name>}`
        // by reading from `.ccswarm/runs/<run_id>/reports/`. Uses a double-underscore
        // prefix to flag as internal — user instructions should never name a variable
        // `__run_id`.
        if !run_id.is_empty() {
            state.variables.insert(
                "__run_id".to_string(),
                serde_json::Value::String(run_id.clone()),
            );
        }

        let mut cumulative_tokens: u64 = 0;
        let mut loop_tracker = super::cycle::LoopTracker::new(flow.max_stage_visits);

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

            // Per-stage loop guard: abort before executing a stage whose visit
            // count exceeds max_stage_visits, so a stuck review→fix loop stops
            // before burning another provider call.
            if loop_tracker.record_visit(&stage.id) {
                let pattern = loop_tracker.detect_pattern();
                warn!(
                    "Flow '{}' aborted: stage '{}' visited {} times (max {}){}",
                    name,
                    stage.id,
                    loop_tracker.visit_count(&stage.id),
                    flow.max_stage_visits,
                    pattern
                        .as_ref()
                        .map(|p| format!(", repeating pattern: {}", p.join(" -> ")))
                        .unwrap_or_default()
                );
                self.record_event(
                    crate::events::Event::new(
                        &run_id,
                        crate::events::EventLevel::Warn,
                        crate::events::EventType::TaskEnd,
                        format!(
                            "Loop detected: stage '{}' exceeded max_stage_visits ({})",
                            stage.id, flow.max_stage_visits
                        ),
                    )
                    .with_movement(&stage.id)
                    .with_metadata(serde_json::json!({
                        "reason": "loop_detected",
                        "stage": stage.id,
                        "visits": loop_tracker.visit_count(&stage.id),
                        "max_stage_visits": flow.max_stage_visits,
                        "pattern": pattern,
                    })),
                )
                .await;
                state.status = FlowStatus::Aborted;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Expose the per-stage visit count so promotion rules can match
            // (internal `__` prefix keeps it out of prompt template expansion,
            // same convention as `__run_id`).
            state.variables.insert(
                "__visit_count".to_string(),
                serde_json::json!(loop_tracker.visit_count(&stage.id)),
            );

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
            let movement_succeeded = stage_output_succeeded(&output);

            // Save intermediate state for timeout recovery
            *self.last_state.write().await = Some(state.clone());

            // Notify progress listener
            if let Some(ref tx) = self.progress_tx {
                let _ = tx.send(MovementProgress {
                    movement_id: stage.id.clone(),
                    duration_ms: movement_duration_ms,
                    success: movement_succeeded,
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

            // Save stage report to .ccswarm/runs/{run-id}/reports/.
            // Always writes a `<stage.id>.md` for backward-compat; additionally
            // writes any declared `output_contract.reports[*].name` files so
            // downstream stages can pull them via `{report:<name>}`.
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
                    let default_path = report_dir.join(format!("{}.md", stage.id));
                    let _ = tokio::fs::write(&default_path, report_content).await;

                    if let Some(contract) = stage.output_contract.as_ref() {
                        for report in &contract.reports {
                            if !is_safe_report_name(&report.name) {
                                warn!(
                                    "Skipping declared report '{}' on stage '{}': name must not contain path separators or '..'",
                                    report.name, stage.id
                                );
                                continue;
                            }
                            let report_path = report_dir.join(&report.name);
                            if let Err(e) = tokio::fs::write(&report_path, report_content).await {
                                warn!(
                                    "Failed to write declared report '{}' for stage '{}': {}",
                                    report.name, stage.id, e
                                );
                            }
                        }
                    }
                }
            }

            // Check if terminal (no rules = done)
            if stage.rules.is_empty() {
                if movement_succeeded {
                    info!("Flow '{}' completed at terminal stage '{}'", name, stage.id);
                    state.status = FlowStatus::Completed;
                } else {
                    warn!("Flow '{}' failed at terminal stage '{}'", name, stage.id);
                    state.status = FlowStatus::Failed;
                }
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
                    if movement_succeeded {
                        info!("No rule matched in stage '{}', completing flow", stage.id);
                        state.status = FlowStatus::Completed;
                    } else {
                        warn!("No rule matched failed stage '{}'", stage.id);
                        state.status = FlowStatus::Failed;
                    }
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
    #[tracing::instrument(
        name = "flow.stage",
        skip_all,
        fields(stage = %stage.id, persona = stage.persona.as_deref(), provider = stage.provider.as_deref())
    )]
    async fn execute_movement(
        &self,
        stage: &Stage,
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        info!(
            "Stage '{}': persona={:?}, permission={:?}",
            stage.id, stage.persona, stage.permission
        );

        // Sub-workflow dispatch (takt-style `kind: workflow_call`). Resolved
        // before the local/parallel/CLI branches: a workflow_call stage's
        // instruction is ignored, so the empty-instruction check below would
        // otherwise short-circuit it to a local summary.
        if let Some(call) = stage.call.as_ref() {
            return self.execute_workflow_call(stage, call, state).await;
        }

        // If instruction is empty or starts with "_local", skip Claude and produce local summary
        if stage.instruction.is_empty() || stage.instruction.starts_with("_local") {
            let summary = self.build_local_summary(state);
            return Ok(summary);
        }

        // Orchestrator-worker: the leader decomposes, workers run in parallel.
        // Dispatched before the declared-parallel branch (validate() rejects
        // combining the two).
        if let Some(spec) = stage.team_leader.as_ref() {
            return self.execute_team_leader(stage, spec, state).await;
        }

        // Sangha consensus: multiple independent members vote; no leader.
        if let Some(spec) = stage.sangha.as_ref() {
            return self.execute_sangha(stage, spec, state).await;
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

            let (provider, model) = self.resolve_effective_provider(stage, state);
            // Resolve effective tool list from the stage's permission level + explicit `tools:`.
            // Without this, `permission: readonly` would still send no `--allowed-tools`,
            // so the provider CLI would fall back to its own (permissive) default.
            let effective_tools = super::permissions::PermissionEnforcer::from_movement(
                stage.permission.clone(),
                &stage.tools,
            )
            .available_tools();

            // Flow-level rate-limit fallback chain, parsed into provider kinds.
            let rate_limit_fallbacks: Vec<(crate::providers::ProviderKind, Option<String>)> = self
                .flows
                .get(&state.flow_name)
                .map(|flow| {
                    flow.on_rate_limit
                        .iter()
                        .filter_map(|target| {
                            crate::providers::ProviderKind::parse(&target.provider)
                                .map(|kind| (kind, target.model.clone()))
                        })
                        .collect()
                })
                .unwrap_or_default();

            let exec_options = crate::session::bridge::MovementExecOptions {
                provider,
                tools: effective_tools,
                model,
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
                worktree_name: self.worktree_name.clone(),
                session_id: None,
                continuation: crate::session::bridge::ContinuationPolicy::SingleTurn,
                rate_limit_fallbacks,
            };

            // Command-gate loop: when the agent call succeeds but a declared
            // gate fails, bounded gate output is appended to the prompt and
            // the stage re-runs (up to max_retries additional attempts).
            let mut gate_feedback: Option<String> = None;
            let mut gate_attempts_left = stage.max_retries;
            loop {
                let effective_prompt = match &gate_feedback {
                    Some(feedback) => format!("{}\n\n{}", prompt, feedback),
                    None => prompt.clone(),
                };
                let attempt_output = match bridge
                    .execute_with_retry(
                        agent_id,
                        &effective_prompt,
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
                        // Rate-limit fallbacks that fired on the way to this result
                        // are recorded as ProviderError events so the switch is
                        // visible in the run's audit trail.
                        if !result.fallbacks_used.is_empty() {
                            let run_id = self
                                .event_recorder
                                .as_ref()
                                .map(|r| r.run_id().to_string())
                                .unwrap_or_default();
                            self.record_event(
                                crate::events::Event::new(
                                    &run_id,
                                    crate::events::EventLevel::Warn,
                                    crate::events::EventType::ProviderError,
                                    format!(
                                        "Rate-limit fallback engaged for stage '{}': skipped [{}]",
                                        stage.id,
                                        result.fallbacks_used.join(", ")
                                    ),
                                )
                                .with_movement(&stage.id)
                                .with_metadata(serde_json::json!({
                                    "reason": "rate_limit_fallback",
                                    "rate_limited_providers": result.fallbacks_used,
                                })),
                            )
                            .await;
                        }
                        // Stream-json metadata (tool names, cost) rides on
                        // BridgeResult; surface it as a ProviderCall event so
                        // events.ndjson carries per-call telemetry. Token counts are
                        // deliberately NOT repeated here — they ride on the stage's
                        // MovementEnd event, and `ccswarm cost` sums token fields
                        // across all events, so duplicating them would double-count.
                        if !result.tool_names.is_empty() || result.total_cost_usd.is_some() {
                            let run_id = self
                                .event_recorder
                                .as_ref()
                                .map(|r| r.run_id().to_string())
                                .unwrap_or_default();
                            self.record_event(
                                crate::events::Event::new(
                                    &run_id,
                                    crate::events::EventLevel::Info,
                                    crate::events::EventType::ProviderCall,
                                    format!("Provider call completed for stage '{}'", stage.id),
                                )
                                .with_movement(&stage.id)
                                .with_metadata(serde_json::json!({
                                    "tool_names": result.tool_names,
                                    "cost_usd": result.total_cost_usd,
                                })),
                            )
                            .await;
                        }
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
                };

                // Gates only run after a successful agent call; a failed call
                // already routes through the stage's failure rules.
                let call_succeeded = attempt_output
                    .get("status")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| s == "completed");
                if stage.gates.is_empty() || !call_succeeded {
                    break attempt_output;
                }

                match run_command_gates(&stage.gates, &work_dir).await {
                    None => break attempt_output,
                    Some((gate_name, feedback)) => {
                        warn!(
                            "Stage '{}' gate '{}' failed ({} retry attempts left)",
                            stage.id, gate_name, gate_attempts_left
                        );
                        if gate_attempts_left == 0 {
                            let mut failed = attempt_output;
                            if let Some(obj) = failed.as_object_mut() {
                                obj.insert("status".into(), serde_json::json!("failed"));
                                obj.insert("gate_failed".into(), serde_json::json!(gate_name));
                            }
                            break failed;
                        }
                        gate_attempts_left -= 1;
                        gate_feedback = Some(feedback);
                    }
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

        let sub_movs: Vec<Stage> = parent
            .sub_movements
            .iter()
            .filter_map(|id| flow.stages.iter().find(|m| m.id == *id))
            .cloned()
            .collect();

        if sub_movs.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid sub-stages found for parallel execution"
            ));
        }

        self.run_stages_parallel(&parent.id, &sub_movs, state).await
    }

    /// Run a set of stages concurrently and aggregate their outputs into the
    /// parallel shape `{"parallel": true, "agents": {id: output}, ...}` that
    /// `evaluate_rules` and `all()`/`any()` aggregation understand. Shared by
    /// declared `parallel:` stages and team_leader-synthesized workers.
    async fn run_stages_parallel(
        &self,
        parent_id: &str,
        stages: &[Stage],
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        // Strip the parent's visit count so promotion rules don't fire on
        // sub-stages (takt excludes promotion on parallel sub-steps — the
        // count tracks the parent, not them).
        let mut sub_state = state.clone();
        sub_state.variables.remove("__visit_count");
        let futures: Vec<_> = stages
            .iter()
            .map(|m| self.execute_movement(m, &sub_state))
            .collect();

        let results = futures::future::join_all(futures).await;

        // Aggregate results
        let mut outputs = serde_json::Map::new();
        let mut all_success = true;
        for (i, result) in results.into_iter().enumerate() {
            let sub_id = &stages[i].id;
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
            "stage": parent_id,
            "parallel": true,
            "status": if all_success { "completed" } else { "partial" },
            "agents": outputs,
        }))
    }

    /// Execute a team_leader stage: a leader call decomposes the task into
    /// parts (JSON), the parts run concurrently as synthesized worker stages,
    /// and the outputs aggregate into the parallel shape.
    ///
    /// Robustness ladder: a malformed leader reply gets ONE retry with the
    /// parse error attached; if that also fails, the stage degrades to a
    /// single worker executing the original instruction — a decomposition
    /// failure must not kill work a single agent could do.
    async fn execute_team_leader(
        &self,
        stage: &Stage,
        spec: &super::team_leader::TeamLeaderSpec,
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        use super::team_leader;

        let expanded_instruction = expand_template(&stage.instruction, &state.variables);
        let max_parts = spec.max_parts.max(1);

        // Leader phase: readonly decomposition call (the leader plans; the
        // workers edit).
        let mut leader_stage = stage.clone();
        leader_stage.team_leader = None;
        leader_stage.gates = Vec::new();
        leader_stage.permission = MovementPermission::Readonly;
        leader_stage.tools = Vec::new();

        let mut parts = None;
        let mut last_parse_error = String::new();
        for attempt in 0..2 {
            let mut prompt_stage = leader_stage.clone();
            prompt_stage.instruction =
                team_leader::decomposition_prompt(&expanded_instruction, max_parts);
            if attempt > 0 {
                prompt_stage.instruction.push_str(&format!(
                    "\n\n# Previous attempt failed\n{}\nReply with ONLY the JSON array this time.",
                    last_parse_error
                ));
            }

            // Box::pin: execute_movement → execute_team_leader →
            // execute_movement would otherwise be an infinite-sized future.
            let leader_output = Box::pin(self.execute_movement(&prompt_stage, state)).await?;
            let leader_text = leader_output
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            match team_leader::parse_parts(leader_text, max_parts) {
                Ok(p) => {
                    parts = Some(p);
                    break;
                }
                Err(e) => {
                    warn!(
                        "team_leader '{}' decomposition attempt {} unparsable: {}",
                        stage.id,
                        attempt + 1,
                        e
                    );
                    last_parse_error = e;
                }
            }
        }

        // Graceful degradation: run the whole task as one worker.
        let parts = parts.unwrap_or_else(|| {
            warn!(
                "team_leader '{}' falling back to a single worker (decomposition failed twice)",
                stage.id
            );
            vec![super::team_leader::TaskPart {
                id: format!("{}-worker", stage.id),
                title: String::new(),
                instruction: expanded_instruction.clone(),
            }]
        });

        info!(
            "team_leader '{}' decomposed into {} part(s): [{}]",
            stage.id,
            parts.len(),
            parts
                .iter()
                .map(|p| p.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let workers: Vec<Stage> = parts
            .iter()
            .map(|part| team_leader::worker_stage(stage, spec, part))
            .collect();

        Box::pin(self.run_stages_parallel(&stage.id, &workers, state)).await
    }

    /// Execute a Sangha consensus stage. Members run independently, then their
    /// explicit `SANGHA_DECISION=*` lines are tallied. The stage succeeds only
    /// when approvals meet quorum.
    async fn execute_sangha(
        &self,
        stage: &Stage,
        spec: &super::sangha::SanghaSpec,
        state: &FlowState,
    ) -> Result<serde_json::Value> {
        use super::sangha;

        let expanded_instruction = expand_template(&stage.instruction, &state.variables);
        let mut prompt_parent = stage.clone();
        prompt_parent.instruction = expanded_instruction;

        let members = sangha::members_or_default(spec);
        let quorum = spec.quorum.max(1) as usize;
        let member_stages = members
            .iter()
            .map(|member| sangha::member_stage(&prompt_parent, spec, member))
            .collect::<Vec<_>>();

        info!(
            "sangha '{}' collecting consensus from {} member(s), quorum={}",
            stage.id,
            member_stages.len(),
            quorum
        );

        let parallel_output =
            Box::pin(self.run_stages_parallel(&stage.id, &member_stages, state)).await?;
        let member_outputs = parallel_output
            .get("agents")
            .and_then(|agents| agents.as_object())
            .cloned()
            .unwrap_or_default();

        let mut approvals = 0usize;
        let mut revisions = 0usize;
        let mut abstentions = 0usize;
        let mut decisions = serde_json::Map::new();

        for (member_id, output) in &member_outputs {
            let text = output
                .get("output")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let decision = sangha::extract_decision(text);
            match decision {
                sangha::SanghaDecision::Approve => approvals += 1,
                sangha::SanghaDecision::Revise => revisions += 1,
                sangha::SanghaDecision::Abstain => abstentions += 1,
            }
            decisions.insert(
                member_id.clone(),
                serde_json::json!({
                    "decision": decision.as_str(),
                    "status": output.get("status").cloned().unwrap_or_else(|| serde_json::json!("unknown")),
                }),
            );
        }

        let accepted = approvals >= quorum;
        Ok(serde_json::json!({
            "stage": stage.id,
            "sangha": true,
            "parallel": true,
            "status": if accepted { "completed" } else { "failed" },
            "decision": if accepted { "accepted" } else { "needs_revision" },
            "quorum": quorum,
            "approvals": approvals,
            "revisions": revisions,
            "abstentions": abstentions,
            "decisions": decisions,
            "members": member_outputs,
        }))
    }

    /// Execute a sub-workflow as a single stage. The child flow inherits the
    /// parent's variables, additionally seeded with any `call.args` (with `{key}`
    /// substitution against parent variables). On success the child's final
    /// FlowState is collapsed into a JSON value the parent's rules can match
    /// against; on failure the error bubbles up so the parent can route via a
    /// FAIL rule. Re-entrant: the child runs through the same `execute_piece_state`
    /// loop, so nesting depth is bounded only by `flow.max_stages` per level.
    async fn execute_workflow_call(
        &self,
        stage: &Stage,
        call: &WorkflowCallSpec,
        parent_state: &FlowState,
    ) -> Result<serde_json::Value> {
        // Resolve the child flow up front so misspellings fail fast with a
        // useful message rather than a downstream "stage not found".
        let child_flow = self
            .flows
            .get(&call.flow)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Stage '{}' calls unknown flow '{}'. Registered flows: {}",
                    stage.id,
                    call.flow,
                    self.flows.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            })?
            .clone();

        info!(
            "Stage '{}' invoking sub-workflow '{}' (parent_variables={}, args={})",
            stage.id,
            call.flow,
            parent_state.variables.len(),
            call.args.len()
        );

        // Seed child state with the parent's variables so `{task}`, prior
        // `{<stage>_output}`, and `__run_id` flow through unchanged. Then layer
        // explicit `call.args` on top with template expansion against the parent.
        let mut child_variables = parent_state.variables.clone();
        for (k, v) in &call.args {
            let expanded = expand_template(v, &parent_state.variables);
            child_variables.insert(k.clone(), serde_json::Value::String(expanded));
        }

        let child_state = FlowState {
            flow_name: child_flow.name.clone(),
            current_movement: child_flow.initial_movement.clone(),
            movement_count: 0,
            history: vec![],
            variables: child_variables,
            status: FlowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
        };

        // Box::pin the recursive call: Rust async fns can't have infinite-sized
        // futures, and execute_workflow_call → execute_piece_state → execute_movement
        // → execute_workflow_call would otherwise form a cycle.
        let final_state =
            Box::pin(self.execute_piece_state(&child_flow.name, &child_flow, child_state))
                .await
                .map_err(|e| anyhow::anyhow!("Sub-workflow '{}' failed: {}", call.flow, e))?;

        // Collapse the child's terminal output into a value the parent's rules
        // can match. Surfacing both `status` and the last `_output` keeps
        // tag-style ("COMPLETE") and content-style judging both viable.
        let last_output = final_state
            .history
            .last()
            .and_then(|t| t.output.clone())
            .unwrap_or(serde_json::Value::Null);

        Ok(serde_json::json!({
            "stage": stage.id,
            "workflow_call": call.flow,
            "status": format!("{:?}", final_state.status).to_lowercase(),
            "output": last_output,
            "stages_executed": final_state.movement_count,
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
            for report in &c.reports {
                let mut line = format!(
                    "This stage produces a named report `{}` ({}); downstream stages reference it as `{{report:{}}}`",
                    report.name, report.format, report.name
                );
                if let Some(desc) = &report.description {
                    line.push_str(" — ");
                    line.push_str(desc);
                }
                parts.push(line);
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

        // Inject task description when the stage instruction did not already
        // expand `{task}` into the user prompt.
        if let Some(task_text) = state.variables.get("task").and_then(|v| v.as_str())
            && !expanded_instruction.contains(task_text)
        {
            parts.push(format!("## User Task\n\n{}", task_text));
        }

        // User message (knowledge → instruction → policy → output contract).
        // The persona system prompt is forwarded separately through
        // ProviderOptions.system_prompt during live execution.
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
        // Filter rules tagged interactive_only / requires_user_input when this
        // engine is in pipeline mode. Both flags imply "skip without a human in
        // the loop"; requires_user_input additionally signals to interactive UIs
        // that the next stage expects user input (consumed by the front-end, not
        // here).
        let filtered: Vec<&MovementRule> = rules
            .iter()
            .filter(|r| self.interactive || !(r.interactive_only || r.requires_user_input))
            .collect();

        if filtered.is_empty() {
            return Ok(None);
        }

        let output_str = serde_json::to_string(output).unwrap_or_default();

        // Parallel stages return {"parallel": true, "agents": {sub_id: output}}
        // (built by execute_parallel_movements). Extract per-agent texts so the
        // judge can evaluate all()/any() aggregate conditions across them.
        let parallel_outputs: Option<HashMap<String, String>> = output
            .get("parallel")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            .then(|| output.get("agents").and_then(|v| v.as_object()))
            .flatten()
            .map(|agents| {
                agents
                    .iter()
                    .map(|(sub_id, val)| {
                        let text = val
                            .get("output")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                            .unwrap_or_else(|| serde_json::to_string(val).unwrap_or_default());
                        (sub_id.clone(), text)
                    })
                    .collect()
            });

        // Judge takes a slice of owned rules; clone the filtered references into
        // a temporary Vec. Rules are tiny structs (4 short strings + flags) so
        // this is a negligible allocation in practice.
        let filtered_owned: Vec<MovementRule> = filtered.iter().map(|&r| r.clone()).collect();

        // Real-LLM judgments for ai() rules (opt-in via CCSWARM_LLM_JUDGE=1).
        // Computed here — the async side — and handed to the sync judge as a
        // verdict map so rule-priority ordering stays intact.
        let llm_verdicts = self
            .llm_evaluate_ai_rules(&filtered_owned, &output_str)
            .await;

        let judge_result = self.judge.evaluate(
            &output_str,
            &filtered_owned,
            parallel_outputs.as_ref(),
            llm_verdicts.as_ref(),
        )?;

        if let Some(index) = judge_result.matched_rule_index
            && index < filtered_owned.len()
        {
            debug!(
                "Judge matched rule {}: method={:?}, confidence={:.2}, next={}",
                index,
                judge_result.match_method,
                judge_result.confidence,
                filtered_owned[index].next
            );
            return Ok(Some(filtered_owned[index].next.clone()));
        }

        Ok(None)
    }

    /// Evaluate `ai()` rule conditions with a real LLM call (one short YES/NO
    /// question per rule). Returns a verdict map keyed by rule index, or
    /// `None` when disabled (`CCSWARM_LLM_JUDGE` unset), no bridge is
    /// configured, or no `ai()` rules exist. Per-rule failures are logged at
    /// warn and left out of the map so the lexical heuristic covers them —
    /// never a silent fallback.
    async fn llm_evaluate_ai_rules(
        &self,
        rules: &[MovementRule],
        output: &str,
    ) -> Option<HashMap<usize, bool>> {
        let enabled = std::env::var("CCSWARM_LLM_JUDGE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !enabled {
            return None;
        }
        let bridge = self.bridge.as_ref()?;
        if !rules
            .iter()
            .any(|r| matches!(r.condition, RuleCondition::AiCondition { .. }))
        {
            return None;
        }

        let identity = crate::identity::AgentIdentity {
            agent_id: "llm-judge".to_string(),
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
        let options = crate::session::bridge::MovementExecOptions {
            provider: self.default_provider.or_else(|| {
                std::env::var("CCSWARM_PROVIDER")
                    .ok()
                    .as_deref()
                    .and_then(crate::providers::ProviderKind::parse)
            }),
            ..Default::default()
        };

        let mut verdicts = HashMap::new();
        for (index, rule) in rules.iter().enumerate() {
            let RuleCondition::AiCondition { ai: condition } = &rule.condition else {
                continue;
            };
            let prompt = format!(
                "You are a routing judge for an automated workflow. Decide whether \
                 the condition holds for the agent output below.\n\n\
                 # Condition\n{}\n\n# Agent output (truncated)\n{}\n\n\
                 Reply with exactly YES or NO on the first line. No other text.",
                condition,
                truncate_for_context(output, 1500)
            );
            match bridge
                .execute_with_retry(
                    "llm-judge",
                    &prompt,
                    &identity,
                    &self.working_dir,
                    None,
                    0,
                    0,
                    &options,
                )
                .await
            {
                Ok(result) => match super::judge::parse_judge_reply(&result.raw) {
                    Some(verdict) => {
                        verdicts.insert(index, verdict);
                    }
                    None => warn!(
                        "LLM judge gave an ambiguous reply for ai() rule {} — \
                         falling back to the lexical heuristic for it",
                        index
                    ),
                },
                Err(e) => warn!(
                    "LLM judge call failed for ai() rule {}: {} — \
                     falling back to the lexical heuristic for it",
                    index, e
                ),
            }
        }

        (!verdicts.is_empty()).then_some(verdicts)
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
/// Builtin `team-dynamic` flow: plan → team_leader implement → review.
/// Defined in YAML for legibility; parse+validate happen once at registry
/// construction and the unit tests cover both.
fn team_dynamic_flow() -> Flow {
    const YAML: &str = r#"
name: team-dynamic
description: "Orchestrator-worker: planner designs, a team leader decomposes the implementation into parallel parts at runtime, reviewer validates"
max_stages: 12
initial_movement: plan
stages:
  - id: plan
    persona: planner
    instruction: |
      Analyze the following task and produce a concise implementation plan
      (key components, order of work, risks):

      {task}
    permission: readonly
    rules:
      - condition: success
        next: implement
  - id: implement
    persona: coder
    instruction: |
      Implement the following task according to the plan.

      # Task
      {task}

      # Plan
      {plan_output}
    permission: edit
    team_leader:
      max_parts: 3
      part_persona: coder
      part_permission: edit
    rules:
      - condition:
          all: ["completed"]
        next: review
      - condition:
          any: ["failed"]
        next: review
  - id: review
    persona: reviewer
    instruction: |
      Review the implementation work for the task below. Identify bugs,
      missing pieces, and quality issues. End with APPROVED or NEEDS_FIX.

      # Task
      {task}
    permission: readonly
    rules:
      - condition: success
        next: complete
  - id: complete
    instruction: "_local"
"#;
    Flow::from_yaml(YAML).expect("builtin team-dynamic flow must parse")
}

pub fn builtin_flows() -> Vec<Flow> {
    vec![
        team_dynamic_flow(),
        // Default development workflow
        Flow {
            name: "default".to_string(),
            description: "Sangha consensus workflow: plan → sangha consensus → implement → review → fix"
                .to_string(),
            max_stages: 30,
            max_stage_visits: 3,
            on_rate_limit: Vec::new(),
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
                        next: "sangha".to_string(),
                        priority: 0,
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
                Stage {
                    id: "sangha".to_string(),
                    persona: Some("planner".to_string()),
                    policy: Some("review".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Review the plan and task as a Sangha. Approve only when the implementation direction is clear, scoped, and testable.\n\n# Task\n{task}\n\n# Plan\n{plan_output}".to_string(),
                    tools: vec!["read".to_string(), "grep".to_string(), "glob".to_string()],
                    permission: MovementPermission::Readonly,
                    rules: vec![MovementRule {
                        condition: RuleCondition::Simple("success".to_string()),
                        next: "implement".to_string(),
                        priority: 0,
                        interactive_only: false,
                        requires_user_input: false,
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
                    call: None,
                    promotion: Vec::new(),
                    gates: Vec::new(),
                    team_leader: None,
                    sangha: Some(super::sangha::SanghaSpec {
                        quorum: 2,
                        members: Vec::new(),
                        member_permission: None,
                        member_tools: None,
                        member_timeout_secs: None,
                    }),
                },
                Stage {
                    id: "implement".to_string(),
                    persona: Some("coder".to_string()),
                    policy: Some("coding".to_string()),
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Implement the planned changes according to the Sangha consensus.\n\n# Consensus\n{sangha_output}".to_string(),
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
                            interactive_only: false,
                            requires_user_input: false,                        },
                        MovementRule {
                            condition: RuleCondition::Simple("error".to_string()),
                            next: "fix".to_string(),
                            priority: 1,
                            interactive_only: false,
                            requires_user_input: false,                        },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                            interactive_only: false,
                            requires_user_input: false,                        },
                        MovementRule {
                            condition: RuleCondition::Simple("fixes_needed".to_string()),
                            next: "fix".to_string(),
                            priority: 1,
                            interactive_only: false,
                            requires_user_input: false,                        },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 2,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
            max_stage_visits: 3,
            on_rate_limit: Vec::new(),
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                        reports: vec![],
                    }),
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
            max_stage_visits: 3,
            on_rate_limit: Vec::new(),
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
                            interactive_only: false,
                            requires_user_input: false,                        },
                        MovementRule {
                            condition: RuleCondition::Simple("success".to_string()),
                            next: "done".to_string(),
                            priority: 0,
                            interactive_only: false,
                            requires_user_input: false,                        },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 2,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
            max_stage_visits: 3,
            on_rate_limit: Vec::new(),
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
                call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,            }],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Multi-agent team workflow: plan → parallel(frontend + backend) → review → complete
        Flow {
            name: "team".to_string(),
            description: "Multi-agent orchestration: planner designs, frontend & backend agents execute in parallel, reviewer validates".to_string(),
            max_stages: 10,
            max_stage_visits: 3,
            on_rate_limit: Vec::new(),
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: true,
                    sub_movements: vec!["frontend-impl".to_string(), "backend-impl".to_string()],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                        interactive_only: false,
                        requires_user_input: false,                    }],
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                    agent: None,
                    working_dir: None,
                    retry_delay_ms: default_retry_delay(),
                    pass_previous_response: true,
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
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
                    call: None, promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
                    sangha: None,                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
    ]
}

/// Reject report names that could escape `.ccswarm/runs/<id>/reports/`.
/// Allowed chars: ASCII alphanumerics, '-', '_', '.'. Must not be `.` / `..` /
/// contain consecutive dots or path separators.
fn is_safe_report_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    if name == "." || name == ".." || name.contains("..") {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Expand `{key}` template variables in a string using state variables.
///
/// Recognized forms:
/// - `{key}`: replaced with the string form of `variables[key]`.
/// - `{key_output}`: prefers the `output` field from a JSON object value (the
///   standard shape stages emit).
/// - `{report:<name>}`: reads `.ccswarm/runs/<run_id>/reports/<name>` from disk,
///   using the `__run_id` variable stashed by `execute_piece_state`. Missing /
///   unsafe names expand to an empty string so the prompt doesn't leak the
///   literal token. Adopted from takt's `output_contracts` — replaces brittle
///   `{plan_output}` state-variable chaining with a named on-disk contract.
fn expand_template(template: &str, variables: &HashMap<String, serde_json::Value>) -> String {
    let mut result = template.to_string();

    // First handle `{report:<name>}` so it doesn't collide with the `{key}` loop.
    result = expand_report_references(&result, variables);

    for (key, value) in variables {
        if key.starts_with("__") {
            // Internal book-keeping (e.g. `__run_id`) — never user-substitutable.
            continue;
        }
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

fn expand_report_references(
    template: &str,
    variables: &HashMap<String, serde_json::Value>,
) -> String {
    const PREFIX: &str = "{report:";

    if !template.contains(PREFIX) {
        return template.to_string();
    }

    let run_id = variables
        .get("__run_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find(PREFIX) {
        out.push_str(&rest[..start]);
        let after_prefix = &rest[start + PREFIX.len()..];

        let Some(end) = after_prefix.find('}') else {
            // Unterminated; emit verbatim and stop.
            out.push_str(&rest[start..]);
            return out;
        };

        let name = &after_prefix[..end];
        let remainder = &after_prefix[end + 1..];

        if run_id.is_empty() || !is_safe_report_name(name) {
            // Silent empty-expand: surfacing the raw token would leak internals
            // into the prompt and confuse the model.
            tracing::warn!(
                "Skipping `{{report:{}}}` expansion (run_id_empty={}, unsafe_name={})",
                name,
                run_id.is_empty(),
                !is_safe_report_name(name)
            );
        } else {
            let path = std::path::PathBuf::from(".ccswarm")
                .join("runs")
                .join(run_id)
                .join("reports")
                .join(name);
            match std::fs::read_to_string(&path) {
                Ok(content) => out.push_str(&truncate_for_context(&content, 8000)),
                Err(e) => tracing::warn!(
                    "Failed to read report '{}' for template expansion: {}",
                    path.display(),
                    e
                ),
            }
        }

        rest = remainder;
    }

    out.push_str(rest);
    out
}

fn truncate_for_context(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let boundary = s
            .char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index <= max_len)
            .last()
            .unwrap_or(0);
        format!("{}... [truncated]", &s[..boundary])
    }
}

/// Run a stage's command gates sequentially in `work_dir`. Returns `None`
/// when every gate passes, or `Some((gate_name, feedback))` for the first
/// failure — a bounded, prompt-ready block (stdout/stderr each ≤1000 chars)
/// the engine appends to the instruction before re-running the stage.
async fn run_command_gates(
    gates: &[CommandGate],
    work_dir: &std::path::Path,
) -> Option<(String, String)> {
    for gate in gates {
        info!("Running gate '{}': {}", gate.name, gate.command);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(gate.timeout_secs),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&gate.command)
                .current_dir(work_dir)
                .output(),
        )
        .await;

        let feedback = match result {
            Err(_) => format!(
                "# Gate failure: {}\nCommand `{}` timed out after {}s. \
                 Make the change converge faster or fix what the command checks.",
                gate.name, gate.command, gate.timeout_secs
            ),
            Ok(Err(e)) => format!(
                "# Gate failure: {}\nCommand `{}` could not be spawned: {}",
                gate.name, gate.command, e
            ),
            Ok(Ok(output)) if output.status.success() => continue,
            Ok(Ok(output)) => {
                let stdout = truncate_for_context(&String::from_utf8_lossy(&output.stdout), 1000);
                let stderr = truncate_for_context(&String::from_utf8_lossy(&output.stderr), 1000);
                format!(
                    "# Gate failure: {} (exit code {})\nCommand: `{}`\n\n\
                     ## stdout\n{}\n\n## stderr\n{}\n\n\
                     Fix the issues above and ensure `{}` passes.",
                    gate.name,
                    output.status.code().unwrap_or(-1),
                    gate.command,
                    stdout,
                    stderr,
                    gate.command
                )
            }
        };
        return Some((gate.name.clone(), feedback));
    }
    None
}

fn stage_output_succeeded(output: &serde_json::Value) -> bool {
    output
        .get("status")
        .and_then(|status| status.as_str())
        .is_some_and(|status| status == "completed")
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

    #[tokio::test]
    async fn test_loop_guard_aborts_cyclic_flow() {
        // ping <-> pong cycle, bounded by max_stage_visits rather than max_stages
        let yaml = r#"
name: loop-test
max_stages: 30
max_stage_visits: 2
initial_movement: ping
stages:
  - id: ping
    instruction: "Ping"
    rules:
      - condition: success
        next: pong
  - id: pong
    instruction: "Pong"
    rules:
      - condition: success
        next: ping
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("loop-test".to_string(), flow);

        let state = engine
            .execute_piece("loop-test")
            .await
            .expect("execution failed");
        assert_eq!(state.status, FlowStatus::Aborted);
        // Two visits each executed; the third visit to 'ping' aborts before
        // execution, so only 4 stages ran.
        assert_eq!(state.movement_count, 4);
    }

    #[tokio::test]
    async fn test_parallel_all_aggregate_routes_when_every_output_matches() {
        // Without a bridge, sub-stage output JSON embeds the instruction text,
        // so marker words in instructions drive the aggregate conditions.
        let yaml = r#"
name: par-all
max_stages: 10
initial_movement: fanout
stages:
  - id: fanout
    instruction: "Fan out to reviewers"
    parallel: true
    sub_movements: [left, right]
    rules:
      - condition:
          all: ["approved"]
        next: done
      - condition:
          any: ["needs_fix"]
        next: fixit
  - id: left
    instruction: "left review approved"
  - id: right
    instruction: "right review approved"
  - id: fixit
    instruction: "apply fixes"
  - id: done
    instruction: "terminal"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("par-all".to_string(), flow);

        let state = engine
            .execute_piece("par-all")
            .await
            .expect("execution failed");
        assert_eq!(state.status, FlowStatus::Completed);
        let visited: Vec<&str> = state.history.iter().map(|t| t.to.as_str()).collect();
        assert_eq!(
            visited,
            vec!["done"],
            "all(approved) should route fanout -> done, got history: {:?}",
            visited
        );
    }

    #[tokio::test]
    async fn test_parallel_any_aggregate_routes_on_single_match() {
        let yaml = r#"
name: par-any
max_stages: 10
initial_movement: fanout
stages:
  - id: fanout
    instruction: "Fan out to reviewers"
    parallel: true
    sub_movements: [left, right]
    rules:
      - condition:
          all: ["approved"]
        next: done
      - condition:
          any: ["needs_fix"]
        next: fixit
  - id: left
    instruction: "left review approved"
  - id: right
    instruction: "right review needs_fix"
  - id: fixit
    instruction: "apply fixes"
  - id: done
    instruction: "terminal"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("par-any".to_string(), flow);

        let state = engine
            .execute_piece("par-any")
            .await
            .expect("execution failed");
        assert_eq!(state.status, FlowStatus::Completed);
        let visited: Vec<&str> = state.history.iter().map(|t| t.to.as_str()).collect();
        assert_eq!(
            visited,
            vec!["fixit"],
            "any(needs_fix) should route fanout -> fixit when all(approved) fails, got history: {:?}",
            visited
        );
    }

    #[test]
    fn test_promotion_last_match_wins() {
        let yaml = r#"
name: promo
initial_movement: fix
stages:
  - id: fix
    instruction: "fix it"
    promotion:
      - { at: 2, model: opus }
      - { at: 3, provider: codex, model: gpt-5 }
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("promo".to_string(), flow);
        let stage = engine.flows["promo"].stages[0].clone();
        let mut state = engine.flows["promo"].create_state();

        // Visit 1: no rule matches — base resolution.
        state
            .variables
            .insert("__visit_count".to_string(), serde_json::json!(1));
        let (provider, model) = engine.resolve_effective_provider(&stage, &state);
        assert_eq!(provider, None);
        assert_eq!(model, None);

        // Visit 2: only `at: 2` matches.
        state
            .variables
            .insert("__visit_count".to_string(), serde_json::json!(2));
        let (_, model) = engine.resolve_effective_provider(&stage, &state);
        assert_eq!(model.as_deref(), Some("opus"));

        // Visit 3+: both match — the LAST entry wins.
        state
            .variables
            .insert("__visit_count".to_string(), serde_json::json!(5));
        let (provider, model) = engine.resolve_effective_provider(&stage, &state);
        assert_eq!(provider, Some(crate::providers::ProviderKind::Codex));
        assert_eq!(model.as_deref(), Some("gpt-5"));
    }

    #[test]
    fn test_promotion_skipped_without_visit_count() {
        // Parallel sub-stages run with __visit_count stripped; promotion must
        // not fire there.
        let yaml = r#"
name: promo-skip
initial_movement: a
stages:
  - id: a
    instruction: "x"
    promotion:
      - { at: 1, model: opus }
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("promo-skip".to_string(), flow);
        let stage = engine.flows["promo-skip"].stages[0].clone();
        let state = engine.flows["promo-skip"].create_state(); // no __visit_count

        let (_, model) = engine.resolve_effective_provider(&stage, &state);
        assert_eq!(model, None, "promotion must not fire without a visit count");
    }

    #[test]
    fn test_on_rate_limit_parses_from_yaml() {
        let yaml = r#"
name: fallback
initial_movement: a
on_rate_limit:
  - { provider: codex, model: gpt-5 }
  - { provider: claude }
stages:
  - id: a
    instruction: "x"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        assert_eq!(flow.on_rate_limit.len(), 2);
        assert_eq!(flow.on_rate_limit[0].provider, "codex");
        assert_eq!(flow.on_rate_limit[0].model.as_deref(), Some("gpt-5"));
        assert_eq!(flow.on_rate_limit[1].provider, "claude");
        assert!(flow.on_rate_limit[1].model.is_none());
    }

    #[test]
    fn test_flow_validation_rejects_unknown_providers() {
        let stage_provider = r#"
name: bad-stage-provider
initial_movement: a
stages:
  - id: a
    provider: madeup
    instruction: "x"
"#;
        assert!(
            Flow::from_yaml(stage_provider)
                .expect_err("unknown stage provider must fail validation")
                .to_string()
                .contains("unknown provider")
        );

        let promotion_provider = r#"
name: bad-promotion-provider
initial_movement: a
stages:
  - id: a
    instruction: "x"
    promotion:
      - { at: 2, provider: madeup }
"#;
        assert!(
            Flow::from_yaml(promotion_provider)
                .expect_err("unknown promotion provider must fail validation")
                .to_string()
                .contains("unknown promotion provider")
        );

        let fallback_provider = r#"
name: bad-fallback-provider
initial_movement: a
on_rate_limit:
  - { provider: madeup }
stages:
  - id: a
    instruction: "x"
"#;
        assert!(
            Flow::from_yaml(fallback_provider)
                .expect_err("unknown fallback provider must fail validation")
                .to_string()
                .contains("unknown on_rate_limit provider")
        );
    }

    #[tokio::test]
    async fn test_command_gates_pass_returns_none() {
        let gates = vec![CommandGate {
            name: "noop".to_string(),
            command: "true".to_string(),
            timeout_secs: 30,
        }];
        let result = run_command_gates(&gates, std::path::Path::new("/tmp")).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_command_gates_failure_returns_bounded_feedback() {
        let gates = vec![
            CommandGate {
                name: "ok".to_string(),
                command: "true".to_string(),
                timeout_secs: 30,
            },
            CommandGate {
                name: "boom".to_string(),
                command: "echo broken output; echo to stderr 1>&2; exit 3".to_string(),
                timeout_secs: 30,
            },
        ];
        let (name, feedback) = run_command_gates(&gates, std::path::Path::new("/tmp"))
            .await
            .expect("second gate fails");
        assert_eq!(name, "boom");
        assert!(feedback.contains("# Gate failure: boom (exit code 3)"));
        assert!(feedback.contains("broken output"));
        assert!(feedback.contains("to stderr"));
    }

    #[tokio::test]
    async fn test_command_gates_truncate_long_output() {
        let gates = vec![CommandGate {
            name: "noisy".to_string(),
            command: "yes x | head -c 5000; exit 1".to_string(),
            timeout_secs: 30,
        }];
        let (_, feedback) = run_command_gates(&gates, std::path::Path::new("/tmp"))
            .await
            .expect("gate fails");
        assert!(feedback.contains("[truncated]"));
        // 5000 chars of stdout must have been bounded to ~1000.
        assert!(
            feedback.len() < 2500,
            "feedback too long: {}",
            feedback.len()
        );
    }

    #[tokio::test]
    async fn test_command_gates_timeout_is_reported() {
        let gates = vec![CommandGate {
            name: "slow".to_string(),
            command: "sleep 5".to_string(),
            timeout_secs: 1,
        }];
        let (name, feedback) = run_command_gates(&gates, std::path::Path::new("/tmp"))
            .await
            .expect("gate times out");
        assert_eq!(name, "slow");
        assert!(feedback.contains("timed out after 1s"));
    }

    #[test]
    fn test_gates_parse_from_yaml_with_default_timeout() {
        let yaml = r#"
name: gated
initial_movement: build
stages:
  - id: build
    instruction: "implement"
    gates:
      - { name: build, command: "cargo build" }
      - { name: lint, command: "cargo clippy", timeout_secs: 120 }
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let gates = &flow.stages[0].gates;
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0].timeout_secs, 300, "default timeout");
        assert_eq!(gates[1].timeout_secs, 120);
    }

    #[test]
    fn test_team_leader_parses_with_defaults_and_legacy_yaml_unaffected() {
        let yaml = r#"
name: tl
initial_movement: build
stages:
  - id: build
    instruction: "implement {task}"
    team_leader:
      part_persona: coder
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let spec = flow.stages[0].team_leader.as_ref().expect("spec");
        assert_eq!(spec.max_parts, 3, "default max_parts");
        assert_eq!(spec.part_persona.as_deref(), Some("coder"));

        // Legacy YAML without the field still parses.
        let legacy = r#"
name: legacy
initial_movement: a
stages:
  - id: a
    instruction: "x"
"#;
        let flow = Flow::from_yaml(legacy).expect("legacy parse failed");
        assert!(flow.stages[0].team_leader.is_none());
    }

    #[test]
    fn test_team_leader_validate_rejects_parallel_and_call_combos() {
        let with_parallel = r#"
name: bad1
initial_movement: a
stages:
  - id: a
    instruction: "x"
    parallel: true
    sub_movements: [b]
    team_leader: {}
  - id: b
    instruction: "y"
"#;
        assert!(
            Flow::from_yaml(with_parallel)
                .expect_err("must reject")
                .to_string()
                .contains("team_leader with parallel")
        );

        let with_call = r#"
name: bad2
initial_movement: a
stages:
  - id: a
    instruction: "x"
    team_leader: {}
    call:
      flow: other
"#;
        assert!(
            Flow::from_yaml(with_call)
                .expect_err("must reject")
                .to_string()
                .contains("team_leader with call")
        );

        let zero_parts = r#"
name: bad3
initial_movement: a
stages:
  - id: a
    instruction: "x"
    team_leader:
      max_parts: 0
"#;
        assert!(
            Flow::from_yaml(zero_parts)
                .expect_err("must reject")
                .to_string()
                .contains("max_parts")
        );
    }

    #[test]
    fn test_sangha_parses_and_rejects_ambiguous_modes() {
        let yaml = r#"
name: sangha-test
initial_movement: decide
stages:
  - id: decide
    instruction: "Review the task"
    sangha:
      quorum: 2
      members:
        - { id: planner, persona: planner }
        - { id: reviewer, persona: reviewer }
"#;
        let flow = Flow::from_yaml(yaml).expect("sangha flow should parse");
        let spec = flow.stages[0].sangha.as_ref().expect("sangha spec");
        assert_eq!(spec.quorum, 2);
        assert_eq!(spec.members.len(), 2);

        let bad = r#"
name: bad-sangha
initial_movement: decide
stages:
  - id: decide
    instruction: "Review the task"
    parallel: true
    sangha: {}
"#;
        assert!(
            Flow::from_yaml(bad)
                .expect_err("sangha must reject ambiguous execution modes")
                .to_string()
                .contains("sangha with parallel")
        );
    }

    #[test]
    fn default_flow_uses_sangha_consensus() {
        let default = builtin_flows()
            .into_iter()
            .find(|flow| flow.name == "default")
            .expect("default flow should exist");
        let stage_ids = default
            .stages
            .iter()
            .map(|stage| stage.id.as_str())
            .collect::<Vec<_>>();

        assert!(stage_ids.contains(&"sangha"));
        assert!(
            default
                .stages
                .iter()
                .any(|stage| stage.id == "sangha" && stage.sangha.is_some())
        );
    }

    #[test]
    fn cli_model_override_wins_for_all_stage_execution() {
        let flow = Flow::from_yaml(
            r#"
name: model-test
initial_movement: build
stages:
  - id: build
    provider: codex
    model: stage-model
    instruction: "Build"
    promotion:
      - at: 1
        model: promoted-model
"#,
        )
        .expect("flow should parse");
        let mut state = flow.create_state();
        state
            .variables
            .insert("__visit_count".to_string(), serde_json::json!(1));
        let stage = flow.stages[0].clone();

        let mut engine = FlowEngine::new();
        engine.set_model_override("cli-model");
        engine.set_worktree_name("ccswarm-test-run");

        let (_, model) = engine.resolve_effective_provider(&stage, &state);

        assert_eq!(model.as_deref(), Some("cli-model"));
        assert_eq!(engine.worktree_name.as_deref(), Some("ccswarm-test-run"));
    }

    #[tokio::test]
    async fn test_team_leader_offline_aggregates_parallel_shape() {
        // Without a bridge, the leader's "output" is the prompt echo (no JSON
        // array with parts)… the no-bridge path returns {"instruction", "prompt"}
        // with no "output" key, so decomposition fails twice and degrades to a
        // single worker — exercising the graceful-degradation path end-to-end.
        let yaml = r#"
name: tl-offline
initial_movement: build
stages:
  - id: build
    instruction: "implement the thing"
    team_leader:
      max_parts: 2
    rules:
      - condition:
          all: ["completed"]
        next: done
  - id: done
    instruction: "terminal"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let mut engine = FlowEngine::new();
        engine.flows.insert("tl-offline".to_string(), flow);

        let state = engine
            .execute_piece("tl-offline")
            .await
            .expect("execution failed");
        assert_eq!(state.status, FlowStatus::Completed);
        // The team_leader stage output must carry the parallel shape with the
        // degraded single worker, and all("completed") must route to done.
        let visited: Vec<&str> = state.history.iter().map(|t| t.to.as_str()).collect();
        assert_eq!(visited, vec!["done"]);
        let build_output = state
            .history
            .first()
            .and_then(|t| t.output.as_ref())
            .expect("build output");
        assert_eq!(build_output.get("parallel"), Some(&serde_json::json!(true)));
        let agents = build_output
            .get("agents")
            .and_then(|v| v.as_object())
            .expect("agents map");
        assert_eq!(agents.len(), 1, "degraded to a single worker");
        assert!(agents.contains_key("build-worker"));
    }

    #[test]
    fn test_builtin_flows_roundtrip_through_yaml() {
        // `flow check <builtin>` serializes a builtin flow to YAML and
        // re-parses it; every builtin must survive the round trip.
        for flow in builtin_flows() {
            let yaml = serde_yml::to_string(&flow).expect("serialize");
            if let Err(e) = Flow::from_yaml(&yaml) {
                panic!(
                    "builtin flow '{}' failed YAML round-trip: {:#}\n--- yaml ---\n{}",
                    flow.name, e, yaml
                );
            }
        }
    }

    #[test]
    fn test_max_stage_visits_defaults_to_three() {
        let yaml = r#"
name: defaults
initial_movement: a
stages:
  - id: a
    instruction: "x"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        assert_eq!(flow.max_stage_visits, 3);
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

    #[test]
    fn build_prompt_does_not_duplicate_persona_or_expanded_task() {
        let yaml = r#"
name: prompt-test
initial_movement: plan
stages:
  - id: plan
    persona: coder
    instruction: "Plan this task: {task}"
"#;
        let flow = Flow::from_yaml(yaml).expect("parse failed");
        let stage = &flow.stages[0];
        let mut state = flow.create_state();
        state.variables.insert(
            "task".to_string(),
            serde_json::Value::String("Fix duplicated prompt content".to_string()),
        );
        let engine = FlowEngine::new();

        let prompt = engine.build_movement_prompt(stage, &state);

        assert!(
            !prompt.contains("You are the implementer"),
            "persona system prompt must be passed via ProviderOptions, not duplicated in the user prompt"
        );
        assert_eq!(
            prompt.matches("Fix duplicated prompt content").count(),
            1,
            "task text should appear once when instruction already expands {{task}}"
        );
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

    #[test]
    fn is_safe_report_name_rejects_traversal_and_separators() {
        assert!(super::is_safe_report_name("plan.md"));
        assert!(super::is_safe_report_name("test-report.json"));
        assert!(super::is_safe_report_name("a_b_c.txt"));
        assert!(!super::is_safe_report_name(""));
        assert!(!super::is_safe_report_name("."));
        assert!(!super::is_safe_report_name(".."));
        assert!(!super::is_safe_report_name("a/b"));
        assert!(!super::is_safe_report_name("a\\b"));
        assert!(!super::is_safe_report_name("../etc/passwd"));
        assert!(!super::is_safe_report_name("a..b"));
        assert!(!super::is_safe_report_name(&"a".repeat(200)));
    }

    #[test]
    fn expand_template_resolves_report_reference_from_disk() {
        use std::collections::HashMap;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");

        let reports_dir = std::path::Path::new(".ccswarm")
            .join("runs")
            .join("test-run-1")
            .join("reports");
        std::fs::create_dir_all(&reports_dir).expect("create reports dir");
        std::fs::write(reports_dir.join("plan.md"), "PLAN_BODY").expect("write report");

        let mut vars: HashMap<String, serde_json::Value> = HashMap::new();
        vars.insert(
            "__run_id".to_string(),
            serde_json::Value::String("test-run-1".to_string()),
        );

        let out = super::expand_template("Plan was: {report:plan.md}", &vars);

        // Restore CWD before asserting so a failure doesn't leak directory state.
        std::env::set_current_dir(prev).expect("restore cwd");

        assert_eq!(out, "Plan was: PLAN_BODY");
    }

    #[test]
    fn expand_template_unsafe_report_name_expands_to_empty() {
        use std::collections::HashMap;

        let mut vars: HashMap<String, serde_json::Value> = HashMap::new();
        vars.insert(
            "__run_id".to_string(),
            serde_json::Value::String("test-run-2".to_string()),
        );

        let out = super::expand_template("escape: {report:../passwd}", &vars);
        assert_eq!(out, "escape: ");
    }

    #[tokio::test]
    async fn workflow_call_unknown_flow_fails_fast() {
        let engine = FlowEngine::new();
        let parent_state = FlowState {
            flow_name: "parent".to_string(),
            current_movement: "call-stage".to_string(),
            movement_count: 0,
            history: vec![],
            variables: std::collections::HashMap::new(),
            status: FlowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
        };
        let stage = Stage {
            id: "call-stage".to_string(),
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
            retry_delay_ms: 0,
            pass_previous_response: true,
            call: Some(WorkflowCallSpec {
                flow: "no-such-flow".to_string(),
                args: std::collections::HashMap::new(),
            }),
            promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
            sangha: None,
        };
        let call = stage.call.clone().expect("call");

        let err = engine
            .execute_workflow_call(&stage, &call, &parent_state)
            .await
            .expect_err("unknown flow must error");
        let msg = err.to_string();
        assert!(
            msg.contains("no-such-flow"),
            "error should name the missing flow, got: {msg}"
        );
    }

    #[tokio::test]
    async fn evaluate_rules_drops_interactive_only_in_pipeline_mode() {
        let engine = FlowEngine::new(); // interactive defaults to false
        let rules = vec![
            MovementRule {
                condition: RuleCondition::Simple("review".to_string()),
                next: "ask-human".to_string(),
                priority: 0,
                interactive_only: true,
                requires_user_input: false,
            },
            MovementRule {
                condition: RuleCondition::Simple("review".to_string()),
                next: "complete".to_string(),
                priority: 0,
                interactive_only: false,
                requires_user_input: false,
            },
        ];
        let state = FlowState {
            flow_name: "t".to_string(),
            current_movement: "start".to_string(),
            movement_count: 0,
            status: FlowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            history: vec![],
            variables: std::collections::HashMap::new(),
        };
        let output = serde_json::json!({"output": "review"});

        let next = engine
            .evaluate_rules(&rules, &output, &state)
            .await
            .expect("evaluate");
        // In pipeline mode the interactive_only rule is filtered out, so the
        // fallback "complete" rule wins.
        assert_eq!(next.as_deref(), Some("complete"));
    }

    #[tokio::test]
    async fn evaluate_rules_keeps_interactive_only_in_interactive_mode() {
        let mut engine = FlowEngine::new();
        engine.set_interactive(true);
        let rules = vec![MovementRule {
            condition: RuleCondition::Simple("review".to_string()),
            next: "ask-human".to_string(),
            priority: 0,
            interactive_only: true,
            requires_user_input: false,
        }];
        let state = FlowState {
            flow_name: "t".to_string(),
            current_movement: "start".to_string(),
            movement_count: 0,
            status: FlowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            history: vec![],
            variables: std::collections::HashMap::new(),
        };
        let output = serde_json::json!({"output": "review"});

        let next = engine
            .evaluate_rules(&rules, &output, &state)
            .await
            .expect("evaluate");
        assert_eq!(next.as_deref(), Some("ask-human"));
    }

    #[test]
    fn expand_template_skips_internal_run_id_variable() {
        use std::collections::HashMap;

        let mut vars: HashMap<String, serde_json::Value> = HashMap::new();
        vars.insert(
            "__run_id".to_string(),
            serde_json::Value::String("secret-run".to_string()),
        );

        let out = super::expand_template("run is {__run_id}", &vars);
        assert_eq!(out, "run is {__run_id}");
    }

    #[test]
    fn truncate_for_context_is_utf8_safe() {
        let input = "abあcd";

        assert_eq!(super::truncate_for_context(input, 4), "ab... [truncated]");
        assert_eq!(super::truncate_for_context(input, 5), "abあ... [truncated]");
        assert_eq!(super::truncate_for_context(input, 99), input);
    }

    #[test]
    fn stage_output_succeeded_only_accepts_completed_status() {
        assert!(super::stage_output_succeeded(&serde_json::json!({
            "status": "completed"
        })));
        assert!(!super::stage_output_succeeded(&serde_json::json!({
            "status": "failed"
        })));
        assert!(!super::stage_output_succeeded(&serde_json::json!({
            "status": "timeout"
        })));
        assert!(!super::stage_output_succeeded(&serde_json::json!({})));
    }
}
