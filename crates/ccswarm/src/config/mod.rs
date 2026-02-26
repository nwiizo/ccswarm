use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Think modes available in Claude Code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkMode {
    Think,
    ThinkHard,
    ThinkHarder,
    UltraThink,
    MegaThink,
}

impl ThinkMode {
    pub fn to_prompt_suffix(&self) -> &str {
        match self {
            ThinkMode::Think => "think",
            ThinkMode::ThinkHard => "think hard",
            ThinkMode::ThinkHarder => "think harder",
            ThinkMode::UltraThink => "ultrathink",
            ThinkMode::MegaThink => "megathink",
        }
    }
}

impl std::fmt::Display for ThinkMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_prompt_suffix())
    }
}

/// Output format for Claude Code CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text output
    #[default]
    Text,
    /// Structured JSON output with metadata
    Json,
    /// Streaming JSON output (each message as separate JSON object)
    StreamJson,
}

impl OutputFormat {
    /// Convert to CLI argument value
    pub fn as_cli_arg(&self) -> &'static str {
        match self {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
            OutputFormat::StreamJson => "stream-json",
        }
    }
}

/// Claude Code configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// Model to use (e.g., "claude-3.5-sonnet")
    pub model: String,

    /// Whether to skip permission prompts
    #[serde(rename = "dangerously_skip_permissions")]
    pub dangerous_skip: bool,

    /// Think mode to use
    pub think_mode: Option<ThinkMode>,

    /// Output in JSON format
    pub json_output: bool,

    /// Output format: "text", "json", or "stream-json"
    #[serde(default)]
    pub output_format: OutputFormat,

    /// Append to system prompt (recommended over replacing)
    #[serde(default)]
    pub append_system_prompt: Option<String>,

    /// Maximum agentic turns in non-interactive mode
    #[serde(default)]
    pub max_turns: Option<u32>,

    /// Custom commands available to this agent
    pub custom_commands: Vec<String>,

    /// MCP servers configuration
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, serde_json::Value>,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            model: "claude-3.5-sonnet".to_string(),
            dangerous_skip: true,
            think_mode: Some(ThinkMode::Think),
            json_output: true,
            output_format: OutputFormat::Json,
            append_system_prompt: None,
            max_turns: None,
            custom_commands: Vec::new(),
            mcp_servers: HashMap::new(),
        }
    }
}

impl ClaudeConfig {
    /// Create config for Master Claude
    pub fn for_master() -> Self {
        Self {
            model: "claude-3.5-sonnet".to_string(),
            dangerous_skip: true,
            think_mode: Some(ThinkMode::UltraThink),
            json_output: true,
            output_format: OutputFormat::Json,
            append_system_prompt: None,
            max_turns: None,
            custom_commands: vec![
                "ccswarm status".to_string(),
                "ccswarm review".to_string(),
                "ccswarm deploy".to_string(),
                "ccswarm quality-gate".to_string(),
            ],
            mcp_servers: HashMap::new(),
        }
    }

    /// Create config for worker agents
    pub fn for_agent(role: &str) -> Self {
        let think_mode = match role {
            "frontend" | "backend" => Some(ThinkMode::ThinkHard),
            "devops" => Some(ThinkMode::Think),
            "qa" => Some(ThinkMode::ThinkHard),
            _ => Some(ThinkMode::Think),
        };

        let custom_commands = match role {
            "frontend" => vec![
                "npm test".to_string(),
                "npm run lint".to_string(),
                "npm run build".to_string(),
            ],
            "backend" => vec![
                "npm test".to_string(),
                "npm run migrate".to_string(),
                "npm run api-test".to_string(),
            ],
            "devops" => vec![
                "terraform plan".to_string(),
                "kubectl get pods".to_string(),
                "docker build".to_string(),
            ],
            "qa" => vec![
                "npm test".to_string(),
                "npm run e2e".to_string(),
                "npm run coverage".to_string(),
            ],
            _ => Vec::new(),
        };

        Self {
            model: "claude-3.5-sonnet".to_string(),
            dangerous_skip: true,
            think_mode,
            json_output: true,
            output_format: OutputFormat::Json,
            append_system_prompt: None,
            max_turns: None,
            custom_commands,
            mcp_servers: HashMap::new(),
        }
    }
}

/// Agent configuration from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent specialization
    pub specialization: String,

    /// Git worktree path
    pub worktree: String,

    /// Git branch name
    pub branch: String,

    /// Claude configuration
    pub claude_config: ClaudeConfig,

    /// CLAUDE.md template to use
    pub claude_md_template: String,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub repository: RepositoryConfig,
    pub master_claude: MasterClaudeConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "default-project".to_string(),
            repository: RepositoryConfig::default(),
            master_claude: MasterClaudeConfig::default(),
        }
    }
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub url: String,
    pub main_branch: String,
    /// Local path to the repository (defaults to current directory)
    #[serde(default)]
    pub local_path: Option<PathBuf>,
    /// Enable worktree isolation for task execution
    #[serde(default)]
    pub worktree_isolation: bool,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            url: "https://github.com/example/repo.git".to_string(),
            main_branch: "main".to_string(),
            local_path: None,
            worktree_isolation: false,
        }
    }
}

/// Master Claude configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterClaudeConfig {
    pub role: String,
    pub quality_threshold: f64,
    pub think_mode: ThinkMode,
    pub permission_level: String,
    pub claude_config: ClaudeConfig,

    /// Enable proactive mode by default
    #[serde(default = "default_proactive_mode")]
    pub enable_proactive_mode: bool,

    /// Proactive analysis frequency (seconds)
    #[serde(default = "default_proactive_frequency")]
    pub proactive_frequency: u64,

    /// High-frequency proactive analysis frequency (seconds)
    #[serde(default = "default_high_frequency")]
    pub high_frequency: u64,
}

fn default_proactive_mode() -> bool {
    true // Enable proactive mode by default
}

fn default_proactive_frequency() -> u64 {
    30 // Default 30 seconds
}

fn default_high_frequency() -> u64 {
    15 // Default 15 seconds (high frequency mode)
}

impl Default for MasterClaudeConfig {
    fn default() -> Self {
        Self {
            role: "master".to_string(),
            quality_threshold: 0.85,
            think_mode: ThinkMode::Think,
            permission_level: "standard".to_string(),
            claude_config: ClaudeConfig::default(),
            enable_proactive_mode: default_proactive_mode(),
            proactive_frequency: default_proactive_frequency(),
            high_frequency: default_high_frequency(),
        }
    }
}

/// Coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    pub communication_method: String,
    pub sync_interval: u64,
    pub quality_gate_frequency: String,
    pub master_review_trigger: String,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            communication_method: "json".to_string(),
            sync_interval: 30,
            quality_gate_frequency: "on_task_completion".to_string(),
            master_review_trigger: "auto".to_string(),
        }
    }
}

/// Complete ccswarm configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CcswarmConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    pub coordination: CoordinationConfig,
}

impl CcswarmConfig {
    /// Load configuration from file with validation
    pub async fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let contents = tokio::fs::read_to_string(&path).await.map_err(|e| {
            anyhow::anyhow!("Failed to read config file '{}': {}", path.display(), e)
        })?;
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Invalid JSON in '{}': {}", path.display(), e))?;
        config.validate().map_err(|e| {
            anyhow::anyhow!("Config validation failed for '{}': {}", path.display(), e)
        })?;
        Ok(config)
    }

    /// Save configuration to file
    pub async fn to_file(&self, path: PathBuf) -> anyhow::Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, contents).await?;
        Ok(())
    }

    /// Validate configuration after deserialization
    pub fn validate(&self) -> anyhow::Result<()> {
        // Project name must not be empty
        if self.project.name.trim().is_empty() {
            anyhow::bail!("project.name must not be empty");
        }

        // Repository URL must not be empty
        if self.project.repository.url.trim().is_empty() {
            anyhow::bail!("project.repository.url must not be empty");
        }

        // Main branch must not be empty
        if self.project.repository.main_branch.trim().is_empty() {
            anyhow::bail!("project.repository.main_branch must not be empty");
        }

        // Quality threshold must be between 0.0 and 1.0
        let threshold = self.project.master_claude.quality_threshold;
        if !(0.0..=1.0).contains(&threshold) {
            anyhow::bail!(
                "project.master_claude.quality_threshold must be between 0.0 and 1.0, got {}",
                threshold
            );
        }

        // Validate agent configs
        for (name, agent) in &self.agents {
            if agent.specialization.trim().is_empty() {
                anyhow::bail!("agents.{}.specialization must not be empty", name);
            }
            if agent.worktree.trim().is_empty() {
                anyhow::bail!("agents.{}.worktree must not be empty", name);
            }
            if agent.branch.trim().is_empty() {
                anyhow::bail!("agents.{}.branch must not be empty", name);
            }
        }

        // Sync interval must be positive
        if self.coordination.sync_interval == 0 {
            anyhow::bail!("coordination.sync_interval must be greater than 0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config_creation() {
        let config = ClaudeConfig::default();
        assert_eq!(config.model, "claude-3.5-sonnet");
        assert!(config.dangerous_skip);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = CcswarmConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_project_name() {
        let mut config = CcswarmConfig::default();
        config.project.name = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_threshold() {
        let mut config = CcswarmConfig::default();
        config.project.master_claude.quality_threshold = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_sync_interval() {
        let mut config = CcswarmConfig::default();
        config.coordination.sync_interval = 0;
        assert!(config.validate().is_err());
    }
}
