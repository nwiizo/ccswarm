//! Piece/Movement workflow system inspired by takt.
//!
//! A **Piece** is a declarative YAML-defined workflow containing:
//! - Named **Movements** (sequential steps with persona/provider/instructions)
//! - **Rules** for conditional routing between movements
//! - **Output contracts** for schema validation
//!
//! Example YAML:
//! ```yaml
//! name: default
//! description: "Standard development workflow"
//! max_movements: 30
//! initial_movement: plan
//!
//! movements:
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

/// A Piece is a complete workflow definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    /// Piece name (unique identifier)
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Maximum number of movement transitions before abort
    #[serde(default = "default_max_movements")]
    pub max_movements: u32,

    /// ID of the first movement to execute
    pub initial_movement: String,

    /// List of movements in this piece
    pub movements: Vec<Movement>,

    /// Global variables for the piece
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,

    /// Piece-level metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Default interactive mode for this piece
    #[serde(default)]
    pub interactive_mode: Option<super::interactive::InteractiveMode>,
}

fn default_max_movements() -> u32 {
    30
}

/// A Movement is a single step in a Piece workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    /// Unique movement identifier within the piece
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

    /// Tools available to this movement
    #[serde(default)]
    pub tools: Vec<String>,

    /// Permission level for this movement
    #[serde(default)]
    pub permission: MovementPermission,

    /// Routing rules evaluated after movement completes
    #[serde(default)]
    pub rules: Vec<MovementRule>,

    /// Whether this movement executes sub-movements in parallel
    #[serde(default)]
    pub parallel: bool,

    /// Sub-movements for parallel execution
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
}

/// Permission level for a movement
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

/// A routing rule that determines the next movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementRule {
    /// Condition to evaluate (string match, AI evaluation, or built-in)
    pub condition: RuleCondition,

    /// Next movement ID if condition matches
    pub next: String,

    /// Optional priority for rule ordering (higher = checked first)
    #[serde(default)]
    pub priority: u8,
}

/// Condition types for movement routing
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

/// Output contract for validating movement results
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
}

fn default_format() -> String {
    "text".to_string()
}

/// Runtime state of a piece execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceState {
    /// Piece being executed
    pub piece_name: String,

    /// Current movement ID
    pub current_movement: String,

    /// Number of movements executed so far
    pub movement_count: u32,

    /// History of movement transitions
    pub history: Vec<MovementTransition>,

    /// Accumulated variables/outputs
    pub variables: HashMap<String, serde_json::Value>,

    /// Current status
    pub status: PieceStatus,

    /// Started at
    pub started_at: DateTime<Utc>,

    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

/// A recorded transition between movements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementTransition {
    /// Source movement ID
    pub from: String,
    /// Destination movement ID
    pub to: String,
    /// Condition that triggered the transition
    pub condition: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Output from the source movement
    pub output: Option<serde_json::Value>,
}

/// Piece execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceStatus {
    /// Not yet started
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully (reached terminal movement or explicit completion)
    Completed,
    /// Aborted (max movements exceeded or error)
    Aborted,
    /// Failed with error
    Failed,
}

impl Piece {
    /// Load a piece from a YAML file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read piece file: {}", path.display()))?;

        Self::from_yaml(&contents)
    }

    /// Parse a piece from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let piece: Self = serde_yaml::from_str(yaml).context("Failed to parse piece YAML")?;
        piece.validate()?;
        Ok(piece)
    }

    /// Validate piece structure
    pub fn validate(&self) -> Result<()> {
        // Check initial movement exists
        if !self.movements.iter().any(|m| m.id == self.initial_movement) {
            return Err(anyhow::anyhow!(
                "Initial movement '{}' not found in piece '{}'",
                self.initial_movement,
                self.name
            ));
        }

        // Check all rule targets reference valid movements
        for movement in &self.movements {
            for rule in &movement.rules {
                if !self.movements.iter().any(|m| m.id == rule.next) {
                    return Err(anyhow::anyhow!(
                        "Rule in movement '{}' references unknown movement '{}'",
                        movement.id,
                        rule.next
                    ));
                }
            }

            // Check parallel sub-movements exist
            if movement.parallel {
                for sub in &movement.sub_movements {
                    if !self.movements.iter().any(|m| m.id == *sub) {
                        return Err(anyhow::anyhow!(
                            "Parallel movement '{}' references unknown sub-movement '{}'",
                            movement.id,
                            sub
                        ));
                    }
                }
            }
        }

        // Check for duplicate movement IDs
        let mut seen = std::collections::HashSet::new();
        for movement in &self.movements {
            if !seen.insert(&movement.id) {
                return Err(anyhow::anyhow!(
                    "Duplicate movement ID '{}' in piece '{}'",
                    movement.id,
                    self.name
                ));
            }
        }

        Ok(())
    }

    /// Get a movement by ID
    pub fn get_movement(&self, id: &str) -> Option<&Movement> {
        self.movements.iter().find(|m| m.id == id)
    }

    /// Check if a movement is terminal (has no rules / no transitions)
    pub fn is_terminal(&self, movement_id: &str) -> bool {
        self.get_movement(movement_id)
            .map(|m| m.rules.is_empty())
            .unwrap_or(true)
    }

    /// Create initial execution state
    pub fn create_state(&self) -> PieceState {
        PieceState {
            piece_name: self.name.clone(),
            current_movement: self.initial_movement.clone(),
            movement_count: 0,
            history: Vec::new(),
            variables: self.variables.clone(),
            status: PieceStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Piece engine that executes piece workflows
pub struct PieceEngine {
    /// Loaded pieces
    pieces: HashMap<String, Piece>,
    /// Movement judge for tag/AI-based condition evaluation
    judge: super::judge::MovementJudge,
    /// Facet registry for prompt composition
    facet_registry: super::facets::FacetRegistry,
}

impl PieceEngine {
    pub fn new() -> Self {
        let mut facet_registry = super::facets::FacetRegistry::new();
        // Register built-in facets
        for persona in super::facets::builtin_personas() {
            facet_registry.register_persona(persona);
        }
        for policy in super::facets::builtin_policies() {
            facet_registry.register_policy(policy);
        }
        Self {
            pieces: HashMap::new(),
            judge: super::judge::MovementJudge::default(),
            facet_registry,
        }
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

    /// Load a piece from a YAML file
    pub async fn load_piece(&mut self, path: &Path) -> Result<String> {
        let piece = Piece::load_from_file(path).await?;
        let name = piece.name.clone();
        info!(
            "Loaded piece '{}' with {} movements",
            name,
            piece.movements.len()
        );
        self.pieces.insert(name.clone(), piece);
        Ok(name)
    }

    /// Load all pieces from a directory
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
                match self.load_piece(&path).await {
                    Ok(name) => loaded.push(name),
                    Err(e) => warn!("Failed to load piece from {}: {}", path.display(), e),
                }
            }
        }

        Ok(loaded)
    }

    /// Get a loaded piece
    pub fn get_piece(&self, name: &str) -> Option<&Piece> {
        self.pieces.get(name)
    }

    /// List all loaded pieces
    pub fn list_pieces(&self) -> Vec<&Piece> {
        self.pieces.values().collect()
    }

    /// Register a piece directly (for programmatic / test usage)
    pub fn register_piece(&mut self, piece: Piece) {
        self.pieces.insert(piece.name.clone(), piece);
    }

    /// Execute a piece workflow
    pub async fn execute_piece(&self, name: &str) -> Result<PieceState> {
        let piece = self
            .pieces
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Piece '{}' not found", name))?;

        let mut state = piece.create_state();
        state.status = PieceStatus::Running;

        info!(
            "Starting piece '{}' at movement '{}'",
            name, state.current_movement
        );

        loop {
            // Check max movements
            if state.movement_count >= piece.max_movements {
                warn!(
                    "Piece '{}' exceeded max movements ({})",
                    name, piece.max_movements
                );
                state.status = PieceStatus::Aborted;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Get current movement
            let movement = match piece.get_movement(&state.current_movement) {
                Some(m) => m.clone(),
                None => {
                    state.status = PieceStatus::Failed;
                    state.completed_at = Some(Utc::now());
                    return Err(anyhow::anyhow!(
                        "Movement '{}' not found in piece '{}'",
                        state.current_movement,
                        name
                    ));
                }
            };

            debug!(
                "Executing movement '{}' (#{}) in piece '{}'",
                movement.id, state.movement_count, name
            );

            // Execute the movement
            let output = self.execute_movement(&movement, &state).await?;
            state.movement_count += 1;

            // Store output in variables
            state
                .variables
                .insert(format!("{}_output", movement.id), output.clone());

            // Check if terminal (no rules = done)
            if movement.rules.is_empty() {
                info!(
                    "Piece '{}' completed at terminal movement '{}'",
                    name, movement.id
                );
                state.status = PieceStatus::Completed;
                state.completed_at = Some(Utc::now());
                break;
            }

            // Evaluate rules to determine next movement
            let next = self
                .evaluate_rules(&movement.rules, &output, &state)
                .await?;

            match next {
                Some(next_id) => {
                    state.history.push(MovementTransition {
                        from: movement.id.clone(),
                        to: next_id.clone(),
                        condition: "matched".to_string(),
                        timestamp: Utc::now(),
                        output: Some(output),
                    });
                    state.current_movement = next_id;
                }
                None => {
                    // No rule matched - treat as completion
                    info!(
                        "No rule matched in movement '{}', completing piece",
                        movement.id
                    );
                    state.status = PieceStatus::Completed;
                    state.completed_at = Some(Utc::now());
                    break;
                }
            }
        }

        Ok(state)
    }

    /// Execute a single movement
    async fn execute_movement(
        &self,
        movement: &Movement,
        state: &PieceState,
    ) -> Result<serde_json::Value> {
        info!(
            "Movement '{}': persona={:?}, permission={:?}",
            movement.id, movement.persona, movement.permission
        );

        // Build the prompt from instruction + persona + context
        let prompt = self.build_movement_prompt(movement, state);

        // For now, return a simulated output
        // In production, this would call the agent execution system
        let output = serde_json::json!({
            "movement": movement.id,
            "instruction": movement.instruction,
            "prompt": prompt,
            "status": "completed",
        });

        // Validate output contract if specified
        if let Some(ref contract) = movement.output_contract {
            self.validate_output_contract(contract, &output)?;
        }

        Ok(output)
    }

    /// Build the prompt for a movement using faceted prompting.
    ///
    /// Composition order (takt-style):
    /// - System: persona (via FacetRegistry)
    /// - User: knowledge → instruction → policy → output contract → tools → tags
    fn build_movement_prompt(&self, movement: &Movement, state: &PieceState) -> String {
        // Build output contract text if present
        let contract_text = movement.output_contract.as_ref().map(|c| {
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

        // Use faceted prompting to compose system + user message
        let composed = self.facet_registry.compose(
            movement.persona.as_deref(),
            movement.policy.as_deref(),
            movement.knowledge.as_deref(),
            &movement.instruction,
            contract_text.as_deref(),
        );

        let mut parts = Vec::new();

        // System prompt (persona)
        if !composed.system.is_empty() {
            parts.push(composed.system);
        }

        // User message (knowledge → instruction → policy → output contract)
        if !composed.user.is_empty() {
            parts.push(composed.user);
        }

        // Add available tools
        if !movement.tools.is_empty() {
            parts.push(format!("Available tools: {}", movement.tools.join(", ")));
        }

        // Add permission context
        parts.push(format!("Permission level: {:?}", movement.permission));

        // Add relevant variable context from previous movements
        if !state.variables.is_empty() {
            let var_summary: Vec<String> = state
                .variables
                .iter()
                .filter(|(k, _)| k.ends_with("_output"))
                .map(|(k, v)| {
                    let key = k.trim_end_matches("_output");
                    format!("Previous '{}' output: {}", key, v)
                })
                .collect();
            if !var_summary.is_empty() {
                parts.push(format!("Context:\n{}", var_summary.join("\n")));
            }
        }

        // Inject tag instructions for routing (takt-style [STEP:N] tags)
        if !movement.rules.is_empty() {
            let tag_instructions =
                super::judge::MovementJudge::generate_tag_instructions(&movement.rules);
            parts.push(tag_instructions);
        }

        parts.join("\n\n")
    }

    /// Evaluate routing rules against movement output using the MovementJudge.
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
        _state: &PieceState,
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
            "markdown" => {
                // Markdown should contain at least one heading or list
                if !output_str.contains('#') && !output_str.contains("- ") {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message:
                            "Output does not appear to be valid markdown (no headings or lists)"
                                .to_string(),
                    });
                }
            }
            "yaml" => {
                // Check for YAML-like structure (key: value patterns)
                if !output_str.contains(':') {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message: "Output does not appear to be valid YAML".to_string(),
                    });
                }
            }
            "code" => {
                // Code should have some structure
                if output_str.len() < 10 {
                    violations.push(ContractViolation {
                        kind: ViolationKind::InvalidFormat,
                        message: "Output appears too short to be code".to_string(),
                    });
                }
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

impl Default for PieceEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in piece templates
pub fn builtin_pieces() -> Vec<Piece> {
    vec![
        // Default development workflow
        Piece {
            name: "default".to_string(),
            description: "Standard development workflow: plan → implement → review → fix"
                .to_string(),
            max_movements: 30,
            initial_movement: "plan".to_string(),
            movements: vec![
                Movement {
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
                },
                Movement {
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
                },
                Movement {
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
                },
                Movement {
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
                },
                Movement {
                    id: "complete".to_string(),
                    persona: None,
                    policy: None,
                    knowledge: None,
                    provider: None,
                    model: None,
                    instruction: "Workflow completed successfully".to_string(),
                    tools: vec![],
                    permission: MovementPermission::Readonly,
                    rules: vec![], // Terminal
                    parallel: false,
                    sub_movements: vec![],
                    output_contract: None,
                    timeout: None,
                    max_retries: 0,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Research workflow
        Piece {
            name: "research".to_string(),
            description: "Autonomous research and investigation workflow".to_string(),
            max_movements: 20,
            initial_movement: "investigate".to_string(),
            movements: vec![
                Movement {
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
                },
                Movement {
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
                    }),
                    timeout: None,
                    max_retries: 0,
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
        // Review-fix minimal workflow
        Piece {
            name: "review-fix".to_string(),
            description: "Minimal review and fix cycle".to_string(),
            max_movements: 10,
            initial_movement: "review".to_string(),
            movements: vec![
                Movement {
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
                },
                Movement {
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
                },
                Movement {
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
                },
            ],
            variables: HashMap::new(),
            metadata: HashMap::new(),
            interactive_mode: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_from_yaml() {
        let yaml = r#"
name: test-piece
description: "A test piece"
max_movements: 10
initial_movement: start

movements:
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

        let piece = Piece::from_yaml(yaml).expect("Failed to parse YAML");
        assert_eq!(piece.name, "test-piece");
        assert_eq!(piece.movements.len(), 2);
        assert_eq!(piece.initial_movement, "start");
        assert_eq!(piece.movements[0].permission, MovementPermission::Readonly);
    }

    #[test]
    fn test_piece_validation_invalid_initial() {
        let yaml = r#"
name: bad-piece
initial_movement: nonexistent
movements:
  - id: start
    instruction: "Hello"
"#;

        let result = Piece::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_validation_invalid_rule_target() {
        let yaml = r#"
name: bad-rules
initial_movement: start
movements:
  - id: start
    instruction: "Hello"
    rules:
      - condition: success
        next: nonexistent
"#;

        let result = Piece::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_validation_duplicate_ids() {
        let yaml = r#"
name: dup-ids
initial_movement: start
movements:
  - id: start
    instruction: "First"
  - id: start
    instruction: "Duplicate"
"#;

        let result = Piece::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_pieces() {
        let pieces = builtin_pieces();
        assert!(!pieces.is_empty());

        for piece in &pieces {
            piece.validate().expect(&format!(
                "Built-in piece '{}' failed validation",
                piece.name
            ));
        }
    }

    #[test]
    fn test_piece_create_state() {
        let yaml = r#"
name: state-test
initial_movement: start
movements:
  - id: start
    instruction: "Begin"
"#;
        let piece = Piece::from_yaml(yaml).expect("parse failed");
        let state = piece.create_state();
        assert_eq!(state.piece_name, "state-test");
        assert_eq!(state.current_movement, "start");
        assert_eq!(state.movement_count, 0);
        assert_eq!(state.status, PieceStatus::Pending);
    }

    #[test]
    fn test_terminal_movement() {
        let yaml = r#"
name: terminal-test
initial_movement: start
movements:
  - id: start
    instruction: "Begin"
    rules:
      - condition: success
        next: end
  - id: end
    instruction: "Done"
"#;
        let piece = Piece::from_yaml(yaml).expect("parse failed");
        assert!(!piece.is_terminal("start"));
        assert!(piece.is_terminal("end"));
    }

    #[tokio::test]
    async fn test_piece_engine_execute() {
        let yaml = r#"
name: exec-test
initial_movement: step1
movements:
  - id: step1
    instruction: "Step 1"
    rules:
      - condition: success
        next: step2
  - id: step2
    instruction: "Step 2 (terminal)"
"#;
        let piece = Piece::from_yaml(yaml).expect("parse failed");
        let mut engine = PieceEngine::new();
        engine.pieces.insert("exec-test".to_string(), piece);

        let state = engine
            .execute_piece("exec-test")
            .await
            .expect("execution failed");
        assert_eq!(state.status, PieceStatus::Completed);
        assert_eq!(state.movement_count, 2);
    }
}
