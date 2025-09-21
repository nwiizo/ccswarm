//! Agent isolation mode configuration
//!
//! Defines how agents should be isolated from each other and the host system.

use serde::{Deserialize, Serialize};

/// Isolation mode for agent execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationMode {
    /// Use git worktrees for isolation (lightweight, fast)
    /// - File system isolation via separate working directories
    /// - Shared system resources (CPU, memory, network)
    /// - Same OS environment
    GitWorktree,

    /// Use Docker containers for isolation (full isolation)
    /// - Complete process isolation
    /// - Resource limits (CPU, memory)
    /// - Network isolation
    /// - Custom environments per agent
    Container,

    /// Hybrid mode - worktree with container fallback
    /// - Start with worktree for speed
    /// - Fall back to container for specific tasks requiring isolation
    /// - Best of both worlds
    Hybrid,
}

impl Default for IsolationMode {
    fn default() -> Self {
        // Default to git worktree for backward compatibility
        Self::GitWorktree
    }
}

impl IsolationMode {
    /// Check if this mode requires Docker
    pub fn requires_docker(&self) -> bool {
        matches!(self, Self::Container | Self::Hybrid)
    }

    /// Check if this mode uses git worktrees
    pub fn uses_worktree(&self) -> bool {
        matches!(self, Self::GitWorktree | Self::Hybrid)
    }

    /// Get display name for the isolation mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GitWorktree => "Git Worktree",
            Self::Container => "Docker Container",
            Self::Hybrid => "Hybrid (Worktree + Container)",
        }
    }

    /// Parse from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "worktree" | "git" | "gitworktree" => Some(Self::GitWorktree),
            "container" | "docker" => Some(Self::Container),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }
}

/// Configuration for agent isolation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IsolationConfig {
    /// The isolation mode to use
    pub mode: IsolationMode,

    /// Whether to enforce isolation (fail if requested mode unavailable)
    pub enforce: bool,

    /// Container-specific settings (used when mode is Container or Hybrid)
    pub container: ContainerIsolationConfig,
}

/// Container-specific isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerIsolationConfig {
    /// Base image to use for containers
    pub base_image: Option<String>,

    /// Whether to enable GPU support
    pub enable_gpu: bool,

    /// Additional volume mounts
    pub extra_volumes: Vec<(String, String)>,

    /// Additional environment variables
    pub extra_env: std::collections::HashMap<String, String>,

    /// Network mode override
    pub network_mode: Option<String>,

    /// Whether to remove containers on exit
    pub auto_remove: bool,
}

impl Default for ContainerIsolationConfig {
    fn default() -> Self {
        Self {
            base_image: None,
            enable_gpu: false,
            extra_volumes: Vec::new(),
            extra_env: std::collections::HashMap::new(),
            network_mode: None,
            auto_remove: true,
        }
    }
}

