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
            dangerous_skip: false,
            think_mode: Some(ThinkMode::Think),
            json_output: true,
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
            dangerous_skip: false,
            think_mode: Some(ThinkMode::UltraThink),
            json_output: true,
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

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub url: String,
    pub main_branch: String,
}

/// Master Claude configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterClaudeConfig {
    pub role: String,
    pub quality_threshold: f64,
    pub think_mode: ThinkMode,
    pub permission_level: String,
    pub claude_config: ClaudeConfig,
}

/// Coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    pub communication_method: String,
    pub sync_interval: u64,
    pub quality_gate_frequency: String,
    pub master_review_trigger: String,
}

/// Complete ccswarm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcswarmConfig {
    pub project: ProjectConfig,
    pub agents: HashMap<String, AgentConfig>,
    pub coordination: CoordinationConfig,
}

impl CcswarmConfig {
    /// Load configuration from file
    pub async fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let contents = tokio::fs::read_to_string(path).await?;
        let config: Self = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub async fn to_file(&self, path: PathBuf) -> anyhow::Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, contents).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_think_mode_display() {
        assert_eq!(ThinkMode::Think.to_string(), "think");
        assert_eq!(ThinkMode::UltraThink.to_string(), "ultrathink");
    }

    #[test]
    fn test_claude_config_defaults() {
        let config = ClaudeConfig::default();
        assert_eq!(config.model, "claude-3.5-sonnet");
        assert!(!config.dangerous_skip);
        assert!(config.json_output);
    }

    #[test]
    fn test_master_config() {
        let config = ClaudeConfig::for_master();
        assert_eq!(config.think_mode, Some(ThinkMode::UltraThink));
        assert!(!config.dangerous_skip);
        assert!(config
            .custom_commands
            .contains(&"ccswarm status".to_string()));
    }

    #[test]
    fn test_agent_config() {
        let frontend_config = ClaudeConfig::for_agent("frontend");
        assert_eq!(frontend_config.think_mode, Some(ThinkMode::ThinkHard));
        assert!(frontend_config.dangerous_skip);
        assert!(frontend_config
            .custom_commands
            .contains(&"npm test".to_string()));

        let devops_config = ClaudeConfig::for_agent("devops");
        assert_eq!(devops_config.think_mode, Some(ThinkMode::Think));
        assert!(devops_config
            .custom_commands
            .contains(&"terraform plan".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = CcswarmConfig {
            project: ProjectConfig {
                name: "Test Project".to_string(),
                repository: RepositoryConfig {
                    url: "https://github.com/test/repo".to_string(),
                    main_branch: "main".to_string(),
                },
                master_claude: MasterClaudeConfig {
                    role: "technical_lead".to_string(),
                    quality_threshold: 0.9,
                    think_mode: ThinkMode::UltraThink,
                    permission_level: "supervised".to_string(),
                    claude_config: ClaudeConfig::for_master(),
                },
            },
            agents: HashMap::new(),
            coordination: CoordinationConfig {
                communication_method: "json_files".to_string(),
                sync_interval: 30,
                quality_gate_frequency: "on_commit".to_string(),
                master_review_trigger: "all_tasks_complete".to_string(),
            },
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: CcswarmConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.project.name, deserialized.project.name);
    }
}
