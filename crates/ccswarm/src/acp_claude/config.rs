//! Configuration for Claude Code ACP integration

use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for Claude Code ACP connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeACPConfig {
    /// WebSocket URL for Claude Code (default: ws://localhost:9100)
    pub url: String,

    /// Auto-connect on startup
    pub auto_connect: bool,

    /// Connection timeout in seconds
    pub timeout: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Retry delay in seconds
    pub retry_delay: u64,

    /// Prefer Claude Code for task execution
    pub prefer_claude: bool,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for ClaudeACPConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:9100".to_string(),
            auto_connect: true,
            timeout: 30,
            max_retries: 3,
            retry_delay: 2,
            prefer_claude: true,
            debug: false,
        }
    }
}

impl ClaudeACPConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = env::var("CCSWARM_CLAUDE_ACP_URL") {
            config.url = url;
        }

        if let Ok(auto) = env::var("CCSWARM_CLAUDE_ACP_AUTO_CONNECT") {
            config.auto_connect = auto.parse().unwrap_or(true);
        }

        if let Ok(timeout) = env::var("CCSWARM_CLAUDE_ACP_TIMEOUT") {
            if let Ok(t) = timeout.parse() {
                config.timeout = t;
            }
        }

        if let Ok(retries) = env::var("CCSWARM_CLAUDE_ACP_MAX_RETRIES") {
            if let Ok(r) = retries.parse() {
                config.max_retries = r;
            }
        }

        if let Ok(prefer) = env::var("CCSWARM_CLAUDE_ACP_PREFER_CLAUDE") {
            config.prefer_claude = prefer.parse().unwrap_or(true);
        }

        if let Ok(debug) = env::var("CCSWARM_CLAUDE_ACP_DEBUG") {
            config.debug = debug.parse().unwrap_or(false);
        }

        config
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Create a builder for the configuration
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// Builder for ClaudeACPConfig
pub struct ConfigBuilder {
    config: ClaudeACPConfig,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ClaudeACPConfig::default(),
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }

    pub fn auto_connect(mut self, auto: bool) -> Self {
        self.config.auto_connect = auto;
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn prefer_claude(mut self, prefer: bool) -> Self {
        self.config.prefer_claude = prefer;
        self
    }

    pub fn build(self) -> ClaudeACPConfig {
        self.config
    }
}
