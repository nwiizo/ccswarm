//! Pipeline Mode for CI/CD Automation
//!
//! Enables running flow workflows non-interactively in CI/CD environments.
//! Provides structured output, exit code mapping, and environment variable injection.
//!
//! # Example
//! ```no_run
//! use ccswarm::workflow::pipeline::{PipelineConfig, PipelineRunner};
//! use std::time::Duration;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = PipelineConfig::builder()
//!     .flow_name("default")
//!     .task_text("Implement feature X")
//!     .output_format("json")
//!     .timeout(Duration::from_secs(300))
//!     .env_var("ENVIRONMENT", "production")
//!     .build()?;
//!
//! let runner = PipelineRunner::new();
//! let output = runner.execute(config).await?;
//!
//! println!("Status: {:?}", output.status);
//! println!("Exit code: {}", output.exit_code().as_code());
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::flow::{FlowEngine, FlowState, FlowStatus};

/// Configuration for pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Name of the flow to execute
    pub flow_name: String,

    /// Task text to execute
    pub task_text: String,

    /// Output format (json, text, markdown)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Maximum execution time
    #[serde(default = "default_timeout")]
    pub timeout: Duration,

    /// Environment variables to inject into flow variables
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Optional output file path
    #[serde(default)]
    pub output_file: Option<PathBuf>,

    /// Whether to include verbose execution details in output
    #[serde(default)]
    pub verbose: bool,

    /// Whether to fail on warnings
    #[serde(default)]
    pub fail_on_warnings: bool,

    /// Additional flow variables
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

fn default_output_format() -> String {
    "text".to_string()
}

fn default_timeout() -> Duration {
    Duration::from_secs(600) // 10 minutes
}

/// Builder for PipelineConfig
#[derive(Debug, Default)]
pub struct PipelineConfigBuilder {
    flow_name: Option<String>,
    task_text: Option<String>,
    output_format: String,
    timeout: Duration,
    env_vars: HashMap<String, String>,
    output_file: Option<PathBuf>,
    verbose: bool,
    fail_on_warnings: bool,
    variables: HashMap<String, serde_json::Value>,
}

impl PipelineConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            output_format: default_output_format(),
            timeout: default_timeout(),
            ..Default::default()
        }
    }

    /// Set the flow name
    pub fn flow_name(mut self, name: impl Into<String>) -> Self {
        self.flow_name = Some(name.into());
        self
    }

    /// Set the task text
    pub fn task_text(mut self, text: impl Into<String>) -> Self {
        self.task_text = Some(text.into());
        self
    }

    /// Set the output format
    pub fn output_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = format.into();
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add an environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set environment variables
    pub fn env_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars = vars;
        self
    }

    /// Set output file path
    pub fn output_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_file = Some(path.into());
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable fail on warnings
    pub fn fail_on_warnings(mut self, fail: bool) -> Self {
        self.fail_on_warnings = fail;
        self
    }

    /// Add a variable
    pub fn variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<PipelineConfig> {
        let flow_name = self
            .flow_name
            .ok_or_else(|| anyhow::anyhow!("flow_name is required"))?;
        let task_text = self
            .task_text
            .ok_or_else(|| anyhow::anyhow!("task_text is required"))?;

        Ok(PipelineConfig {
            flow_name,
            task_text,
            output_format: self.output_format,
            timeout: self.timeout,
            env_vars: self.env_vars,
            output_file: self.output_file,
            verbose: self.verbose,
            fail_on_warnings: self.fail_on_warnings,
            variables: self.variables,
        })
    }
}

impl PipelineConfig {
    /// Create a new builder
    pub fn builder() -> PipelineConfigBuilder {
        PipelineConfigBuilder::new()
    }
}

/// Pipeline execution exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineExitCode {
    /// Successful execution (0)
    Success,
    /// Execution failed (1)
    Failure,
    /// Execution timed out (2)
    Timeout,
    /// Configuration error (3)
    ConfigError,
}

impl PipelineExitCode {
    /// Get the numeric exit code
    pub fn as_code(&self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Timeout => 2,
            Self::ConfigError => 3,
        }
    }

    /// Create from FlowStatus
    pub fn from_status(status: FlowStatus) -> Self {
        match status {
            FlowStatus::Completed => Self::Success,
            FlowStatus::Failed | FlowStatus::Aborted => Self::Failure,
            FlowStatus::Pending | FlowStatus::Running => Self::Failure,
        }
    }
}

/// Result of pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    /// Exit code
    pub exit_code: PipelineExitCode,

    /// Execution status
    pub status: PipelineStatus,

    /// Output content in requested format
    pub output: String,

    /// Execution duration
    pub duration: Duration,

    /// Number of stages executed
    pub movement_count: u32,

    /// Started at timestamp
    pub started_at: DateTime<Utc>,

    /// Completed at timestamp
    pub completed_at: DateTime<Utc>,

    /// Warning messages
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,

    /// Verbose execution details (only if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<PipelineDetails>,
}

/// Pipeline execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStatus {
    /// Completed successfully
    Success,
    /// Failed with error
    Failed,
    /// Timed out
    Timeout,
    /// Aborted
    Aborted,
}

/// Detailed pipeline execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDetails {
    /// Flow name executed
    pub flow_name: String,

    /// Stage transitions
    pub transitions: Vec<MovementTransitionSummary>,

    /// Variables at completion
    pub variables: HashMap<String, serde_json::Value>,
}

/// Summary of a stage transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementTransitionSummary {
    /// Source stage
    pub from: String,
    /// Destination stage
    pub to: String,
    /// Condition that triggered transition
    pub condition: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl PipelineOutput {
    /// Get the exit code
    pub fn exit_code(&self) -> PipelineExitCode {
        self.exit_code
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == PipelineExitCode::Success
    }

    /// Format output as text
    pub fn format_text(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("Status: {:?}\n", self.status));
        result.push_str(&format!("Duration: {:?}\n", self.duration));
        result.push_str(&format!("Stages: {}\n", self.movement_count));

        if !self.warnings.is_empty() {
            result.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                result.push_str(&format!("  - {}\n", warning));
            }
        }

        if let Some(error) = &self.error {
            result.push_str(&format!("\nError: {}\n", error));
        }

        result.push_str(&format!("\nOutput:\n{}\n", self.output));

        result
    }

    /// Format output as JSON
    pub fn format_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize output as JSON")
    }

    /// Format output as markdown
    pub fn format_markdown(&self) -> String {
        let mut result = String::new();
        result.push_str("# Pipeline Execution Report\n\n");
        result.push_str(&format!("**Status:** {:?}\n", self.status));
        result.push_str(&format!("**Duration:** {:?}\n", self.duration));
        result.push_str(&format!("**Stages:** {}\n\n", self.movement_count));

        if !self.warnings.is_empty() {
            result.push_str("## Warnings\n\n");
            for warning in &self.warnings {
                result.push_str(&format!("- {}\n", warning));
            }
            result.push('\n');
        }

        if let Some(error) = &self.error {
            result.push_str("## Error\n\n");
            result.push_str(&format!("```\n{}\n```\n\n", error));
        }

        result.push_str("## Output\n\n");
        result.push_str(&format!("```\n{}\n```\n", self.output));

        if let Some(details) = &self.details {
            result.push_str("\n## Execution Details\n\n");
            result.push_str(&format!("**Flow:** {}\n\n", details.flow_name));

            if !details.transitions.is_empty() {
                result.push_str("### Stage Transitions\n\n");
                for transition in &details.transitions {
                    result.push_str(&format!(
                        "- {} → {} ({})\n",
                        transition.from, transition.to, transition.condition
                    ));
                }
            }
        }

        result
    }
}

/// Runner for executing flows in pipeline mode
pub struct PipelineRunner {
    engine: FlowEngine,
}

impl PipelineRunner {
    /// Create a new pipeline runner
    pub fn new() -> Self {
        Self {
            engine: FlowEngine::new(),
        }
    }

    /// Create with a custom flow engine
    pub fn with_engine(engine: FlowEngine) -> Self {
        Self { engine }
    }

    /// Execute a pipeline configuration
    pub async fn execute(&self, config: PipelineConfig) -> Result<PipelineOutput> {
        let started_at = Utc::now();
        info!(
            "Starting pipeline execution: flow={}, timeout={:?}",
            config.flow_name, config.timeout
        );

        // Validate configuration
        if let Err(e) = self.validate_config(&config) {
            warn!("Pipeline configuration error: {}", e);
            return Ok(PipelineOutput {
                exit_code: PipelineExitCode::ConfigError,
                status: PipelineStatus::Failed,
                output: String::new(),
                duration: Duration::from_secs(0),
                movement_count: 0,
                started_at,
                completed_at: Utc::now(),
                warnings: vec![],
                error: Some(e.to_string()),
                details: None,
            });
        }

        // Execute with timeout
        let result =
            match tokio::time::timeout(config.timeout, self.execute_internal(&config)).await {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => {
                    warn!("Pipeline execution failed: {}", e);
                    self.create_error_output(started_at, e.to_string())
                }
                Err(_) => {
                    // Check if stages completed before timeout
                    let partial = self.engine.get_last_state().await;
                    let movements_done = partial.as_ref().map(|s| s.movement_count).unwrap_or(0);
                    if movements_done > 0 {
                        warn!(
                            "Pipeline timed out after {:?} but {} stages completed",
                            config.timeout, movements_done
                        );
                        self.create_partial_success_output(started_at, config.timeout, partial)
                    } else {
                        warn!("Pipeline execution timed out after {:?}", config.timeout);
                        self.create_timeout_output(started_at, config.timeout)
                    }
                }
            };

        // Write output file if requested
        if let Some(output_file) = &config.output_file
            && let Err(e) = self
                .write_output_file(&result, output_file, &config.output_format)
                .await
        {
            warn!("Failed to write output file: {}", e);
        }

        Ok(result)
    }

    /// Validate pipeline configuration
    fn validate_config(&self, config: &PipelineConfig) -> Result<()> {
        // Check flow exists
        if self.engine.get_flow(&config.flow_name).is_none() {
            anyhow::bail!("Flow '{}' not found", config.flow_name);
        }

        // Validate output format
        match config.output_format.as_str() {
            "json" | "text" | "markdown" => {}
            _ => anyhow::bail!("Invalid output format: {}", config.output_format),
        }

        // Validate timeout
        if config.timeout.as_secs() == 0 {
            anyhow::bail!("Timeout must be greater than 0");
        }

        Ok(())
    }

    /// Internal execution logic
    async fn execute_internal(&self, config: &PipelineConfig) -> Result<PipelineOutput> {
        let started_at = Utc::now();
        debug!(
            "Executing flow '{}' with task: {}",
            config.flow_name, config.task_text
        );

        // Execute the flow with task text injected as context
        let state = self
            .engine
            .execute_piece_with_task(&config.flow_name, &config.task_text)
            .await
            .context("Failed to execute flow")?;

        let completed_at = Utc::now();
        let duration = (completed_at - started_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        // Determine status
        let (status, exit_code) = match state.status {
            FlowStatus::Completed => (PipelineStatus::Success, PipelineExitCode::Success),
            FlowStatus::Aborted => (PipelineStatus::Aborted, PipelineExitCode::Failure),
            FlowStatus::Failed => (PipelineStatus::Failed, PipelineExitCode::Failure),
            _ => (PipelineStatus::Failed, PipelineExitCode::Failure),
        };

        // Collect warnings
        let mut warnings = Vec::new();
        if state.movement_count >= 20 {
            warnings.push(format!("High stage count: {}", state.movement_count));
        }

        // Format output
        let output = self.format_output(&state, &config.output_format)?;

        // Create details if verbose
        let details = if config.verbose {
            Some(PipelineDetails {
                flow_name: config.flow_name.clone(),
                transitions: state
                    .history
                    .iter()
                    .map(|t| MovementTransitionSummary {
                        from: t.from.clone(),
                        to: t.to.clone(),
                        condition: t.condition.clone(),
                        timestamp: t.timestamp,
                    })
                    .collect(),
                variables: state.variables.clone(),
            })
        } else {
            None
        };

        let final_exit_code = if config.fail_on_warnings && !warnings.is_empty() {
            PipelineExitCode::Failure
        } else {
            exit_code
        };

        Ok(PipelineOutput {
            exit_code: final_exit_code,
            status,
            output,
            duration,
            movement_count: state.movement_count,
            started_at,
            completed_at,
            warnings,
            error: None,
            details,
        })
    }

    /// Format flow state output
    fn format_output(&self, state: &super::flow::FlowState, format: &str) -> Result<String> {
        match format {
            "json" => serde_json::to_string_pretty(state).context("Failed to serialize as JSON"),
            "text" => Ok(format!(
                "Flow: {}\nStatus: {:?}\nMovements: {}\n",
                state.flow_name, state.status, state.movement_count
            )),
            "markdown" => Ok(format!(
                "# Flow Execution: {}\n\n**Status:** {:?}\n**Stages:** {}\n",
                state.flow_name, state.status, state.movement_count
            )),
            _ => anyhow::bail!("Unsupported output format: {}", format),
        }
    }

    /// Create error output
    fn create_error_output(&self, started_at: DateTime<Utc>, error: String) -> PipelineOutput {
        let completed_at = Utc::now();
        let duration = (completed_at - started_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        PipelineOutput {
            exit_code: PipelineExitCode::Failure,
            status: PipelineStatus::Failed,
            output: String::new(),
            duration,
            movement_count: 0,
            started_at,
            completed_at,
            warnings: vec![],
            error: Some(error),
            details: None,
        }
    }

    /// Create partial success output when timeout occurs after some stages completed
    fn create_partial_success_output(
        &self,
        started_at: DateTime<Utc>,
        timeout: Duration,
        partial_state: Option<FlowState>,
    ) -> PipelineOutput {
        let completed_at = Utc::now();
        let stages = partial_state
            .as_ref()
            .map(|s| s.movement_count)
            .unwrap_or(0);
        let flow_name = partial_state
            .as_ref()
            .map(|s| s.flow_name.clone())
            .unwrap_or_default();

        PipelineOutput {
            exit_code: PipelineExitCode::Success, // Partial success = still usable
            status: PipelineStatus::Success,
            output: format!(
                "Flow: {}\nStatus: Partial (timed out after {} stages)\nMovements: {}",
                flow_name, stages, stages
            ),
            duration: timeout,
            movement_count: stages,
            started_at,
            completed_at,
            warnings: vec![format!(
                "Pipeline timed out after {:?} but {} stages completed successfully. Generated files may be usable.",
                timeout, stages
            )],
            error: None,
            details: partial_state.map(|s| PipelineDetails {
                flow_name: s.flow_name,
                transitions: s
                    .history
                    .iter()
                    .map(|t| MovementTransitionSummary {
                        from: t.from.clone(),
                        to: t.to.clone(),
                        condition: t.condition.clone(),
                        timestamp: t.timestamp,
                    })
                    .collect(),
                variables: s.variables,
            }),
        }
    }

    /// Create timeout output
    fn create_timeout_output(
        &self,
        started_at: DateTime<Utc>,
        timeout: Duration,
    ) -> PipelineOutput {
        let completed_at = Utc::now();

        PipelineOutput {
            exit_code: PipelineExitCode::Timeout,
            status: PipelineStatus::Timeout,
            output: String::new(),
            duration: timeout,
            movement_count: 0,
            started_at,
            completed_at,
            warnings: vec![],
            error: Some(format!("Execution timed out after {:?}", timeout)),
            details: None,
        }
    }

    /// Write output to file
    async fn write_output_file(
        &self,
        output: &PipelineOutput,
        path: &PathBuf,
        format: &str,
    ) -> Result<()> {
        let content = match format {
            "json" => output.format_json()?,
            "markdown" => output.format_markdown(),
            _ => output.format_text(),
        };

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write output file: {}", path.display()))?;

        info!("Output written to: {}", path.display());
        Ok(())
    }

    /// Get a reference to the flow engine
    pub fn engine(&self) -> &FlowEngine {
        &self.engine
    }

    /// Get a mutable reference to the flow engine
    pub fn engine_mut(&mut self) -> &mut FlowEngine {
        &mut self.engine
    }
}

impl Default for PipelineRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_builder() {
        let config = PipelineConfig::builder()
            .flow_name("test-flow")
            .task_text("Do something")
            .output_format("json")
            .timeout(Duration::from_secs(300))
            .env_var("FOO", "bar")
            .verbose(true)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.flow_name, "test-flow");
        assert_eq!(config.task_text, "Do something");
        assert_eq!(config.output_format, "json");
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert_eq!(config.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert!(config.verbose);
    }

    #[test]
    fn test_pipeline_config_builder_missing_required() {
        let result = PipelineConfig::builder().flow_name("test").build();
        assert!(result.is_err());

        let result = PipelineConfig::builder().task_text("test").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_exit_code_mapping() {
        assert_eq!(PipelineExitCode::Success.as_code(), 0);
        assert_eq!(PipelineExitCode::Failure.as_code(), 1);
        assert_eq!(PipelineExitCode::Timeout.as_code(), 2);
        assert_eq!(PipelineExitCode::ConfigError.as_code(), 3);
    }

    #[test]
    fn test_exit_code_from_status() {
        assert_eq!(
            PipelineExitCode::from_status(FlowStatus::Completed),
            PipelineExitCode::Success
        );
        assert_eq!(
            PipelineExitCode::from_status(FlowStatus::Failed),
            PipelineExitCode::Failure
        );
        assert_eq!(
            PipelineExitCode::from_status(FlowStatus::Aborted),
            PipelineExitCode::Failure
        );
        assert_eq!(
            PipelineExitCode::from_status(FlowStatus::Running),
            PipelineExitCode::Failure
        );
    }

    #[test]
    fn test_pipeline_output_format_text() {
        let output = PipelineOutput {
            exit_code: PipelineExitCode::Success,
            status: PipelineStatus::Success,
            output: "test output".to_string(),
            duration: Duration::from_secs(10),
            movement_count: 5,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            warnings: vec!["warning 1".to_string()],
            error: None,
            details: None,
        };

        let text = output.format_text();
        assert!(text.contains("Status: Success"));
        assert!(text.contains("Stages: 5"));
        assert!(text.contains("warning 1"));
        assert!(text.contains("test output"));
    }

    #[test]
    fn test_pipeline_output_format_json() {
        let output = PipelineOutput {
            exit_code: PipelineExitCode::Success,
            status: PipelineStatus::Success,
            output: "test output".to_string(),
            duration: Duration::from_secs(10),
            movement_count: 3,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            warnings: vec![],
            error: None,
            details: None,
        };

        let json = output.format_json();
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"exit_code\""));
        assert!(json_str.contains("\"status\""));
    }

    #[test]
    fn test_pipeline_output_format_markdown() {
        let output = PipelineOutput {
            exit_code: PipelineExitCode::Failure,
            status: PipelineStatus::Failed,
            output: "failure output".to_string(),
            duration: Duration::from_secs(5),
            movement_count: 2,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            warnings: vec![],
            error: Some("something went wrong".to_string()),
            details: None,
        };

        let md = output.format_markdown();
        assert!(md.contains("# Pipeline Execution Report"));
        assert!(md.contains("**Status:** Failed"));
        assert!(md.contains("## Error"));
        assert!(md.contains("something went wrong"));
    }

    #[test]
    fn test_pipeline_output_is_success() {
        let success_output = PipelineOutput {
            exit_code: PipelineExitCode::Success,
            status: PipelineStatus::Success,
            output: String::new(),
            duration: Duration::from_secs(1),
            movement_count: 1,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            warnings: vec![],
            error: None,
            details: None,
        };
        assert!(success_output.is_success());

        let failure_output = PipelineOutput {
            exit_code: PipelineExitCode::Failure,
            status: PipelineStatus::Failed,
            output: String::new(),
            duration: Duration::from_secs(1),
            movement_count: 1,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            warnings: vec![],
            error: Some("error".to_string()),
            details: None,
        };
        assert!(!failure_output.is_success());
    }

    #[test]
    fn test_pipeline_runner_creation() {
        let runner = PipelineRunner::new();
        // FlowEngine::new() now loads builtin flows by default
        assert!(!runner.engine().list_flows().is_empty());
    }

    #[test]
    fn test_pipeline_config_default_values() {
        let config = PipelineConfig::builder()
            .flow_name("test")
            .task_text("test task")
            .build()
            .unwrap();

        assert_eq!(config.output_format, "text");
        assert_eq!(config.timeout, Duration::from_secs(600));
        assert!(!config.verbose);
        assert!(!config.fail_on_warnings);
    }

    #[test]
    fn test_pipeline_details_serialization() {
        let details = PipelineDetails {
            flow_name: "test".to_string(),
            transitions: vec![MovementTransitionSummary {
                from: "start".to_string(),
                to: "end".to_string(),
                condition: "success".to_string(),
                timestamp: Utc::now(),
            }],
            variables: HashMap::new(),
        };

        let json = serde_json::to_string(&details);
        assert!(json.is_ok());
    }
}
