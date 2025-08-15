/// AI-Session integration for Claude Code subagents
/// 
/// This module provides integration between subagents and the ai-session crate,
/// enabling token-efficient context management and session persistence.

use super::{SubagentDefinition, SubagentError, SubagentResult};
use crate::session::ai_session_adapter::{SessionManagerAdapter, EfficiencyStats};
use crate::identity::AgentRole;
use ai_session::SessionConfig as AISessionConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages AI-Session instances for subagents
pub struct SubagentSessionManager {
    /// Active sessions mapped by subagent instance ID
    sessions: Arc<RwLock<HashMap<String, SubagentSession>>>,
    
    /// Configuration for AI-Session
    config: SubagentSessionConfig,
}

/// Configuration for subagent AI-Session integration
#[derive(Debug, Clone)]
pub struct SubagentSessionConfig {
    /// Enable AI features (context compression, semantic analysis)
    pub enable_ai_features: bool,
    
    /// Maximum tokens per session
    pub max_tokens: usize,
    
    /// Compression threshold (0.0 to 1.0)
    pub compression_threshold: f32,
    
    /// Enable coordination bus for inter-agent communication
    pub enable_coordination: bool,
    
    /// Session persistence directory
    pub persistence_dir: Option<String>,
}

impl Default for SubagentSessionConfig {
    fn default() -> Self {
        Self {
            enable_ai_features: true,
            max_tokens: 4096,
            compression_threshold: 0.8,
            enable_coordination: true,
            persistence_dir: Some(".ccswarm/sessions".to_string()),
        }
    }
}

/// Represents an active subagent session
pub struct SubagentSession {
    /// The subagent instance ID
    pub instance_id: String,
    
    /// The AI-Session adapter
    pub adapter: Arc<SessionManagerAdapter>,
    
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// Metadata about a subagent session
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// Subagent definition name
    pub subagent_name: String,
    
    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
    
    /// Total tokens used
    pub tokens_used: usize,
    
    /// Token savings from compression
    pub tokens_saved: usize,
    
    /// Number of tasks executed
    pub tasks_executed: usize,
}

impl SubagentSessionManager {
    /// Create a new session manager
    pub fn new(config: SubagentSessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create an AI-Session for a subagent
    pub async fn create_session(
        &self,
        instance_id: &str,
        definition: &SubagentDefinition,
    ) -> SubagentResult<String> {
        // Convert subagent definition to AI-Session config
        let ai_config = self.create_ai_session_config(definition);
        
        // Create the AI-Session adapter
        let working_dir = self.config.persistence_dir.as_ref()
            .map(|d| std::path::PathBuf::from(d))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let adapter = Arc::new(SessionManagerAdapter::new(working_dir.clone()));
        
        // Create actual agent session with SessionManagerAdapter
        let agent_role = Self::determine_agent_role(&definition.name);
        let agent_session = adapter.create_agent_session(
            instance_id.to_string(),
            agent_role,
            working_dir,
            Some(definition.description.clone()),
            self.config.enable_ai_features,
        ).await.map_err(|e| SubagentError::Delegation(format!("Failed to create agent session: {}", e)))?;
        
        // Initialize session metadata
        let metadata = SessionMetadata {
            subagent_name: definition.name.clone(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            tokens_used: 0,
            tokens_saved: 0,
            tasks_executed: 0,
        };
        
        // Create the session
        let session = SubagentSession {
            instance_id: instance_id.to_string(),
            adapter,
            metadata,
        };
        
        // Store the session
        let session_id = format!("session-{}", instance_id);
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        log::info!("Created AI-Session for subagent {}: {}", instance_id, session_id);
        
        Ok(session_id)
    }
    
    /// Execute a command in a subagent's AI-Session
    pub async fn execute_command(
        &self,
        session_id: &str,
        command: &str,
    ) -> SubagentResult<String> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        // Update last activity
        session.metadata.last_activity = chrono::Utc::now();
        session.metadata.tasks_executed += 1;
        
        // Send command as input and read output
        session.adapter
            .send_input(&session.instance_id, command)
            .await
            .map_err(|e| SubagentError::Delegation(format!("Command execution failed: {}", e)))?;
        
        let result = session.adapter
            .read_output(&session.instance_id)
            .await
            .map_err(|e| SubagentError::Delegation(format!("Failed to read output: {}", e)))?;
        
        // Update token metrics from efficiency stats
        if let Ok(stats) = session.adapter.get_efficiency_stats().await {
            session.metadata.tokens_used = stats.total_tokens_used;
            session.metadata.tokens_saved = stats.tokens_saved;
        }
        
        Ok(result)
    }
    
    /// Send input to a subagent's AI-Session
    pub async fn send_input(
        &self,
        session_id: &str,
        input: &str,
    ) -> SubagentResult<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        session.metadata.last_activity = chrono::Utc::now();
        
        session.adapter
            .send_input(&session.instance_id, input)
            .await
            .map_err(|e| SubagentError::Delegation(format!("Failed to send input: {}", e)))?;
        
        Ok(())
    }
    
    /// Get output from a subagent's AI-Session
    pub async fn get_output(
        &self,
        session_id: &str,
    ) -> SubagentResult<String> {
        let sessions = self.sessions.read().await;
        
        let session = sessions.get(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        session.adapter
            .read_output(&session.instance_id)
            .await
            .map_err(|e| SubagentError::Delegation(format!("Failed to get output: {}", e)))
    }
    
    /// Get AI context for a subagent's session
    pub async fn get_ai_context(
        &self,
        session_id: &str,
    ) -> SubagentResult<String> {
        let sessions = self.sessions.read().await;
        
        let session = sessions.get(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        let context = session.adapter
            .get_session_context(&session.instance_id)
            .await
            .map_err(|e| SubagentError::Delegation(format!("Failed to get AI context: {}", e)))?;
        
        Ok(format!("Session context: {} messages, status: {:?}", 
            context.messages.len(), 
            context.status))
    }
    
    /// Compress the context of a subagent's session
    pub async fn compress_context(
        &self,
        session_id: &str,
    ) -> SubagentResult<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        // Context compression is handled automatically by ai-session
        // Just update the efficiency stats
        if let Ok(stats) = session.adapter.get_efficiency_stats().await {
            session.metadata.tokens_saved = stats.tokens_saved;
            log::info!("Session {} efficiency: saved {} tokens", session_id, stats.tokens_saved);
        }
        
        Ok(())
    }
    
    /// Send a coordination message between subagents
    pub async fn send_coordination_message(
        &self,
        from_session: &str,
        to_session: &str,
        message: &str,
    ) -> SubagentResult<()> {
        // Get both sessions
        let sessions = self.sessions.read().await;
        
        let from = sessions.get(from_session)
            .ok_or_else(|| SubagentError::NotFound(format!("From session not found: {}", from_session)))?;
        
        let to = sessions.get(to_session)
            .ok_or_else(|| SubagentError::NotFound(format!("To session not found: {}", to_session)))?;
        
        // Create coordination message
        let coord_message = serde_json::json!({
            "from": from.metadata.subagent_name,
            "to": to.metadata.subagent_name,
            "message": message,
            "timestamp": chrono::Utc::now(),
        });
        
        // Send through coordination bus (would integrate with actual ai-session coordination)
        log::info!("Coordination message from {} to {}: {}", 
            from.metadata.subagent_name, 
            to.metadata.subagent_name, 
            message
        );
        
        Ok(())
    }
    
    /// Get session statistics
    pub async fn get_session_stats(
        &self,
        session_id: &str,
    ) -> SubagentResult<SessionMetadata> {
        let sessions = self.sessions.read().await;
        
        let session = sessions.get(session_id)
            .ok_or_else(|| SubagentError::NotFound(format!("Session not found: {}", session_id)))?;
        
        Ok(session.metadata.clone())
    }
    
    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<(String, SessionMetadata)> {
        let sessions = self.sessions.read().await;
        
        sessions.iter()
            .map(|(id, session)| (id.clone(), session.metadata.clone()))
            .collect()
    }
    
    /// Clean up a session
    pub async fn cleanup_session(
        &self,
        session_id: &str,
    ) -> SubagentResult<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(mut session) = sessions.remove(session_id) {
            // Terminate the AI-Session
            session.adapter.terminate_session(&session.instance_id).await
                .map_err(|e| SubagentError::Delegation(format!("Failed to cleanup session: {}", e)))?;
            
            log::info!("Cleaned up session: {}", session_id);
            Ok(())
        } else {
            Err(SubagentError::NotFound(format!("Session not found: {}", session_id)))
        }
    }
    
    /// Create AI-Session configuration from subagent definition
    fn create_ai_session_config(&self, definition: &SubagentDefinition) -> AISessionConfig {
        let mut config = AISessionConfig::default();
        
        // Set basic configuration
        config.name = Some(definition.name.clone());
        config.enable_ai_features = self.config.enable_ai_features;
        
        // Set working directory if persistence is enabled
        if let Some(ref dir) = self.config.persistence_dir {
            config.working_directory = Some(std::path::PathBuf::from(dir));
        }
        
        config
    }
    
    /// Map subagent tools to AI-Session capabilities
    fn map_tools_to_capabilities(&self, definition: &SubagentDefinition) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Standard tools
        for tool in &definition.tools.standard {
            capabilities.push(format!("standard:{}", tool));
        }
        
        // Semantic tools
        for tool in &definition.tools.semantic {
            capabilities.push(format!("semantic:{}", tool));
        }
        
        // Memory tools
        for tool in &definition.tools.memory {
            capabilities.push(format!("memory:{}", tool));
        }
        
        // Custom tools
        for tool in &definition.tools.custom {
            capabilities.push(format!("custom:{}", tool));
        }
        
        capabilities
    }
    
    /// Determine agent role from subagent name
    fn determine_agent_role(name: &str) -> AgentRole {
        let lower_name = name.to_lowercase();
        
        if lower_name.contains("frontend") || lower_name.contains("ui") {
            AgentRole::Frontend {
                technologies: vec!["React".to_string(), "Vue".to_string()],
                responsibilities: vec!["UI/UX".to_string()],
                boundaries: vec!["Client-side only".to_string()],
            }
        } else if lower_name.contains("backend") || lower_name.contains("api") {
            AgentRole::Backend {
                technologies: vec!["API".to_string(), "Database".to_string()],
                responsibilities: vec!["Server logic".to_string()],
                boundaries: vec!["Server-side only".to_string()],
            }
        } else if lower_name.contains("devops") || lower_name.contains("infra") {
            AgentRole::DevOps {
                technologies: vec!["Docker".to_string(), "CI/CD".to_string()],
                responsibilities: vec!["Infrastructure".to_string()],
                boundaries: vec!["Deployment only".to_string()],
            }
        } else if lower_name.contains("qa") || lower_name.contains("test") {
            AgentRole::QA {
                technologies: vec!["Testing".to_string()],
                responsibilities: vec!["Quality assurance".to_string()],
                boundaries: vec!["Testing only".to_string()],
            }
        } else if lower_name.contains("search") {
            AgentRole::Search {
                technologies: vec!["Web search".to_string()],
                responsibilities: vec!["Information gathering".to_string()],
                boundaries: vec!["Search only".to_string()],
            }
        } else {
            // Default to Frontend for unknown roles
            AgentRole::Frontend {
                technologies: vec![],
                responsibilities: vec!["General tasks".to_string()],
                boundaries: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_manager_creation() {
        let config = SubagentSessionConfig::default();
        let manager = SubagentSessionManager::new(config.clone());
        
        assert_eq!(manager.config.enable_ai_features, true);
        assert_eq!(manager.config.max_tokens, 4096);
        assert_eq!(manager.config.compression_threshold, 0.8);
    }
    
    #[test]
    fn test_determine_agent_role() {
        use super::SubagentSessionManager;
        
        // Test frontend detection
        let role = SubagentSessionManager::determine_agent_role("frontend-specialist");
        assert!(matches!(role, AgentRole::Frontend { .. }));
        
        // Test backend detection
        let role = SubagentSessionManager::determine_agent_role("backend-api-service");
        assert!(matches!(role, AgentRole::Backend { .. }));
        
        // Test DevOps detection
        let role = SubagentSessionManager::determine_agent_role("devops-automation");
        assert!(matches!(role, AgentRole::DevOps { .. }));
        
        // Test QA detection
        let role = SubagentSessionManager::determine_agent_role("qa-testing-agent");
        assert!(matches!(role, AgentRole::QA { .. }));
        
        // Test Search detection
        let role = SubagentSessionManager::determine_agent_role("search-agent");
        assert!(matches!(role, AgentRole::Search { .. }));
        
        // Test default fallback
        let role = SubagentSessionManager::determine_agent_role("unknown-agent");
        assert!(matches!(role, AgentRole::Frontend { .. }));
    }
    
    #[test]
    fn test_subagent_session_config_default() {
        let config = SubagentSessionConfig::default();
        
        assert_eq!(config.enable_ai_features, true);
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.compression_threshold, 0.8);
        assert_eq!(config.enable_coordination, true);
        assert_eq!(config.persistence_dir, Some(".ccswarm/sessions".to_string()));
    }
}