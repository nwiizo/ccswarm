/// Agent ID and message conversion utilities
///
/// This module handles conversion between different agent representations
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::coordination::AgentMessage as CCSwarmMessage;
use crate::identity::AgentRole;

/// Agent ID type alias
pub type AgentId = String;

/// Unified agent information across systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAgentInfo {
    /// CCSwarm agent ID
    pub ccswarm_id: String,
    /// Agent role in ccswarm
    pub role: AgentRole,
    /// Active status
    pub active: bool,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Registry for agent mappings
pub struct AgentMappingRegistry {
    /// Mappings between different agent ID formats
    mappings: RwLock<HashMap<String, UnifiedAgentInfo>>,
}

impl AgentMappingRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            mappings: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent mapping
    pub async fn register(&self, info: UnifiedAgentInfo) -> Result<()> {
        let mut mappings = self
            .mappings
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        mappings.insert(info.ccswarm_id.clone(), info);
        Ok(())
    }

    /// Get unified agent info
    pub fn get_agent_info(&self, agent_id: &str) -> Option<UnifiedAgentInfo> {
        let mappings = self
            .mappings
            .read()
            .map_err(|e| {
                log::error!("Failed to acquire read lock: {}", e);
                e
            })
            .ok()?;
        mappings.get(agent_id).cloned()
    }

    /// Convert CCSwarm agent ID to unified format
    pub fn to_unified(&self, ccswarm_id: &str) -> String {
        // For now, just return the same ID
        ccswarm_id.to_string()
    }

    /// Convert from unified format to CCSwarm ID
    pub fn from_unified(&self, unified_id: &str) -> String {
        // For now, just return the same ID
        unified_id.to_string()
    }
}

impl Default for AgentMappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert message types (placeholder)
pub async fn convert_to_ai_session(
    _msg: CCSwarmMessage,
    _registry: &AgentMappingRegistry,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

/// Convert message types back (placeholder)
pub async fn convert_from_ai_session(
    _msg: serde_json::Value,
    _registry: &AgentMappingRegistry,
) -> Result<CCSwarmMessage> {
    Ok(CCSwarmMessage::StatusUpdate {
        agent_id: "system".to_string(),
        status: crate::agent::AgentStatus::Available,
        metrics: serde_json::json!({}),
    })
}
