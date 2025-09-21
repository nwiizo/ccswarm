//! Simple tmux module placeholder
//! This replaces the previous ai_session::tmux_bridge dependency

use anyhow::Result;

/// Placeholder for tmux functionality
pub struct TmuxBridge;

impl TmuxBridge {
    /// Create new tmux bridge
    pub fn new() -> Self {
        Self
    }

    /// Check if tmux is available
    pub fn is_available() -> bool {
        false
    }

    /// Start tmux session
    pub async fn start_session(&self, _name: &str) -> Result<()> {
        anyhow::bail!("Tmux functionality not implemented")
    }

    /// Stop tmux session
    pub async fn stop_session(&self, _name: &str) -> Result<()> {
        anyhow::bail!("Tmux functionality not implemented")
    }
}

impl Default for TmuxBridge {
    fn default() -> Self {
        Self::new()
    }
}
