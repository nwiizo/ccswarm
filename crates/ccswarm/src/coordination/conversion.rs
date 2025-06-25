/// Type conversion layer between ccswarm and ai-session coordination messages
///
/// This module provides bidirectional conversion between ccswarm's AgentMessage
/// and ai-session's AgentMessage types, enabling seamless integration while
/// maintaining backward compatibility.
///
/// # Example Usage
///
/// ```rust,no_run
/// use ccswarm::coordination::conversion::{
///     AgentMappingRegistry, UnifiedAgentInfo,
///     convert_to_ai_session, convert_from_ai_session,
/// };
/// use ccswarm::coordination::AgentMessage as CCSwarmMessage;
/// use ccswarm::agent::AgentStatus;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a registry to track agent mappings
///     let registry = AgentMappingRegistry::new();
///     
///     // Register a ccswarm agent with ai-session mapping
///     let agent_info = UnifiedAgentInfo {
///         ccswarm_id: "frontend-specialist".to_string(),
///         ai_session_id: ai_session::coordination::AgentId::new(),
///         role: ccswarm::identity::default_frontend_role(),
///         capabilities: vec!["React".to_string(), "UI/UX".to_string()],
///         metadata: serde_json::json!({
///             "version": "1.0.0",
///             "worktree": "/tmp/frontend-agent",
///         }),
///     };
///     registry.register(agent_info).await;
///     
///     // Convert ccswarm message to ai-session format
///     let ccswarm_msg = CCSwarmMessage::StatusUpdate {
///         agent_id: "frontend-specialist".to_string(),
///         status: AgentStatus::Working,
///         metrics: serde_json::json!({
///             "tasks_completed": 5,
///             "uptime_seconds": 3600,
///         }),
///     };
///     
///     let ai_session_msg = convert_to_ai_session(ccswarm_msg, &registry).await?;
///     
///     // Convert ai-session message back to ccswarm format
///     let ccswarm_restored = convert_from_ai_session(ai_session_msg, &registry).await?;
///     
///     Ok(())
/// }
/// ```
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Type aliases to avoid naming conflicts
use crate::agent::{AgentStatus, Priority, TaskResult};
use crate::coordination::{AgentMessage as CCSwarmMessage, CoordinationType};
use crate::identity::AgentRole;
use ai_session::coordination::{
    AgentId as AISessionAgentId, AgentMessage as AISessionMessage,
    MessagePriority as AISessionPriority, TaskId as AISessionTaskId,
};

/// Error types for message conversion
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Incompatible message type: {0}")]
    IncompatibleType(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid field value: {field} = {value}")]
    InvalidField { field: String, value: String },
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Unified agent information that bridges ccswarm and ai-session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAgentInfo {
    /// CCSwarm agent ID (string-based)
    pub ccswarm_id: String,
    /// AI-Session agent ID (UUID-based)
    pub ai_session_id: AISessionAgentId,
    /// Agent role from ccswarm identity
    pub role: AgentRole,
    /// Capabilities derived from role/specialization
    pub capabilities: Vec<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl UnifiedAgentInfo {
    /// Create from a ccswarm agent
    pub fn from_ccswarm_agent(agent: &crate::agent::ClaudeCodeAgent) -> Self {
        let capabilities = vec![agent.identity.specialization.name().to_string()];

        let metadata = serde_json::json!({
            "worktree_path": agent.worktree_path.to_string_lossy(),
            "branch_name": agent.branch_name,
            "isolation_mode": format!("{:?}", agent.isolation_mode),
            "created_at": Utc::now().to_rfc3339(),
        });

        Self {
            ccswarm_id: agent.identity.agent_id.clone(),
            ai_session_id: AISessionAgentId::new(),
            role: agent.identity.specialization.clone(),
            capabilities,
            metadata,
        }
    }

    /// Create from ai-session registration data
    pub fn from_ai_session_registration(
        agent_id: AISessionAgentId,
        capabilities: Vec<String>,
        metadata: serde_json::Value,
    ) -> Result<Self, ConversionError> {
        let ccswarm_id = metadata
            .get("ccswarm_agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField("ccswarm_agent_id".to_string()))?
            .to_string();

        let role_str = metadata
            .get("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField("role".to_string()))?;

        let role = match role_str {
            "Frontend" => crate::identity::default_frontend_role(),
            "Backend" => crate::identity::default_backend_role(),
            "DevOps" => crate::identity::default_devops_role(),
            "QA" => crate::identity::default_qa_role(),
            _ => {
                return Err(ConversionError::InvalidField {
                    field: "role".to_string(),
                    value: role_str.to_string(),
                })
            }
        };

        Ok(Self {
            ccswarm_id,
            ai_session_id: agent_id,
            role,
            capabilities,
            metadata,
        })
    }
}

/// Agent mapping registry to track ccswarm <-> ai-session ID mappings
pub struct AgentMappingRegistry {
    /// Map from ccswarm agent ID to ai-session AgentId
    ccswarm_to_ai: Arc<RwLock<HashMap<String, AISessionAgentId>>>,
    /// Map from ai-session AgentId to ccswarm agent ID
    ai_to_ccswarm: Arc<RwLock<HashMap<AISessionAgentId, String>>>,
    /// Full agent information
    agent_info: Arc<RwLock<HashMap<String, UnifiedAgentInfo>>>,
}

impl Default for AgentMappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentMappingRegistry {
    pub fn new() -> Self {
        Self {
            ccswarm_to_ai: Arc::new(RwLock::new(HashMap::new())),
            ai_to_ccswarm: Arc::new(RwLock::new(HashMap::new())),
            agent_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent mapping
    pub async fn register(&self, info: UnifiedAgentInfo) {
        let mut ccswarm_map = self.ccswarm_to_ai.write().await;
        let mut ai_map = self.ai_to_ccswarm.write().await;
        let mut info_map = self.agent_info.write().await;

        ccswarm_map.insert(info.ccswarm_id.clone(), info.ai_session_id.clone());
        ai_map.insert(info.ai_session_id.clone(), info.ccswarm_id.clone());
        info_map.insert(info.ccswarm_id.clone(), info);
    }

    /// Get AI-session ID from ccswarm ID
    pub async fn get_ai_session_id(&self, ccswarm_id: &str) -> Option<AISessionAgentId> {
        self.ccswarm_to_ai.read().await.get(ccswarm_id).cloned()
    }

    /// Get ccswarm ID from AI-session ID
    pub async fn get_ccswarm_id(&self, ai_id: &AISessionAgentId) -> Option<String> {
        self.ai_to_ccswarm.read().await.get(ai_id).cloned()
    }

    /// Get full agent information
    pub async fn get_agent_info(&self, ccswarm_id: &str) -> Option<UnifiedAgentInfo> {
        self.agent_info.read().await.get(ccswarm_id).cloned()
    }
}

/// Trait for converting ccswarm messages to ai-session messages
pub trait IntoAISessionMessage {
    fn into_ai_session(
        self,
        registry: &AgentMappingRegistry,
    ) -> impl std::future::Future<Output = Result<AISessionMessage, ConversionError>> + Send;
}

/// Trait for converting ai-session messages to ccswarm messages
pub trait FromAISessionMessage: Sized {
    fn from_ai_session(
        msg: AISessionMessage,
        registry: &AgentMappingRegistry,
    ) -> impl std::future::Future<Output = Result<Self, ConversionError>> + Send;
}

/// Convert ccswarm TaskResult to JSON value for ai-session
fn task_result_to_json(result: &TaskResult) -> serde_json::Value {
    serde_json::json!({
        "success": result.success,
        "output": result.output,
        "error": result.error,
        "duration_secs": result.duration.as_secs(),
    })
}

/// Convert JSON value to ccswarm TaskResult
fn json_to_task_result(value: serde_json::Value) -> Result<TaskResult, ConversionError> {
    let duration_secs = value
        .get("duration_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(TaskResult {
        success: value
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        output: value
            .get("output")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        error: value
            .get("error")
            .and_then(|v| v.as_str())
            .map(String::from),
        duration: std::time::Duration::from_secs(duration_secs),
    })
}

/// Convert ccswarm AgentStatus to string representation
fn agent_status_to_string(status: &AgentStatus) -> String {
    match status {
        AgentStatus::Initializing => "initializing".to_string(),
        AgentStatus::Available => "available".to_string(),
        AgentStatus::Working => "working".to_string(),
        AgentStatus::WaitingForReview => "waiting_for_review".to_string(),
        AgentStatus::Error(e) => format!("error: {}", e),
        AgentStatus::ShuttingDown => "shutting_down".to_string(),
    }
}

/// Convert string to ccswarm AgentStatus
fn string_to_agent_status(status: &str) -> AgentStatus {
    match status {
        "initializing" => AgentStatus::Initializing,
        "available" => AgentStatus::Available,
        "working" => AgentStatus::Working,
        "waiting_for_review" => AgentStatus::WaitingForReview,
        "shutting_down" => AgentStatus::ShuttingDown,
        s if s.starts_with("error: ") => {
            AgentStatus::Error(s.trim_start_matches("error: ").to_string())
        }
        _ => AgentStatus::Available,
    }
}

/// Convert ccswarm Priority to ai-session MessagePriority
#[allow(dead_code)]
fn priority_to_ai_session(priority: &Priority) -> AISessionPriority {
    match priority {
        Priority::Low => AISessionPriority::Low,
        Priority::Medium => AISessionPriority::Normal,
        Priority::High => AISessionPriority::High,
        Priority::Critical => AISessionPriority::Critical,
    }
}

/// Implementation of conversion from ai-session to ccswarm messages
impl FromAISessionMessage for CCSwarmMessage {
    async fn from_ai_session(
        msg: AISessionMessage,
        registry: &AgentMappingRegistry,
    ) -> Result<Self, ConversionError> {
        match msg {
            AISessionMessage::TaskCompleted {
                agent_id,
                task_id,
                result,
            } => {
                let ccswarm_id = registry.get_ccswarm_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ccswarm agent mapping".to_string())
                })?;

                let task_result = json_to_task_result(result)?;

                Ok(CCSwarmMessage::TaskCompleted {
                    agent_id: ccswarm_id,
                    task_id: task_id.to_string(),
                    result: task_result,
                })
            }

            AISessionMessage::StatusUpdate {
                agent_id,
                status,
                metrics,
            } => {
                let ccswarm_id = registry.get_ccswarm_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ccswarm agent mapping".to_string())
                })?;

                let agent_status = string_to_agent_status(&status);

                Ok(CCSwarmMessage::StatusUpdate {
                    agent_id: ccswarm_id,
                    status: agent_status,
                    metrics: metrics.clone(),
                })
            }

            AISessionMessage::HelpRequest {
                agent_id,
                context,
                priority: _,
            } => {
                let ccswarm_id = registry.get_ccswarm_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ccswarm agent mapping".to_string())
                })?;

                Ok(CCSwarmMessage::RequestAssistance {
                    agent_id: ccswarm_id,
                    task_id: "unknown".to_string(), // AI-session doesn't include task_id
                    reason: context,
                })
            }

            AISessionMessage::TaskProgress {
                agent_id,
                task_id,
                progress,
                message,
            } => {
                let ccswarm_id = registry.get_ccswarm_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ccswarm agent mapping".to_string())
                })?;

                // Map to InterAgentMessage as ccswarm doesn't have TaskProgress
                Ok(CCSwarmMessage::InterAgentMessage {
                    from_agent: ccswarm_id,
                    to_agent: "master-claude".to_string(),
                    message: format!(
                        "Task {} progress: {}% - {}",
                        task_id,
                        (progress * 100.0) as u32,
                        message
                    ),
                    timestamp: Utc::now(),
                })
            }

            _ => {
                // Map other variants to Coordination message with Custom type
                Ok(CCSwarmMessage::Coordination {
                    from_agent: "ai-session".to_string(),
                    to_agent: "master-claude".to_string(),
                    message_type: CoordinationType::Custom("ai-session-message".to_string()),
                    payload: serde_json::to_value(&msg)?,
                })
            }
        }
    }
}

/// Helper function to convert ccswarm message to ai-session message
pub async fn convert_to_ai_session(
    msg: CCSwarmMessage,
    registry: &AgentMappingRegistry,
) -> Result<AISessionMessage, ConversionError> {
    msg.into_ai_session(registry).await
}

/// Helper function to convert ai-session message to ccswarm message
pub async fn convert_from_ai_session(
    msg: AISessionMessage,
    registry: &AgentMappingRegistry,
) -> Result<CCSwarmMessage, ConversionError> {
    CCSwarmMessage::from_ai_session(msg, registry).await
}

/// Implementation of conversion from ccswarm to ai-session messages
impl IntoAISessionMessage for CCSwarmMessage {
    async fn into_ai_session(
        self,
        registry: &AgentMappingRegistry,
    ) -> Result<AISessionMessage, ConversionError> {
        match self {
            CCSwarmMessage::TaskCompleted {
                agent_id,
                task_id: _,
                result,
            } => {
                let ai_agent_id = registry.get_ai_session_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ai-session agent mapping".to_string())
                })?;

                Ok(AISessionMessage::TaskCompleted {
                    agent_id: ai_agent_id,
                    task_id: AISessionTaskId::new(), // Generate new ID as we can't parse UUID from string
                    result: task_result_to_json(&result),
                })
            }

            CCSwarmMessage::StatusUpdate {
                agent_id,
                status,
                metrics,
            } => {
                let ai_agent_id = registry.get_ai_session_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ai-session agent mapping".to_string())
                })?;

                Ok(AISessionMessage::StatusUpdate {
                    agent_id: ai_agent_id,
                    status: agent_status_to_string(&status),
                    metrics: metrics.clone(),
                })
            }

            CCSwarmMessage::RequestAssistance {
                agent_id,
                task_id: _,
                reason,
            } => {
                let ai_agent_id = registry.get_ai_session_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ai-session agent mapping".to_string())
                })?;

                Ok(AISessionMessage::HelpRequest {
                    agent_id: ai_agent_id,
                    context: reason,
                    priority: AISessionPriority::Normal,
                })
            }

            CCSwarmMessage::Heartbeat {
                agent_id,
                timestamp,
            } => {
                let ai_agent_id = registry.get_ai_session_id(&agent_id).await.ok_or_else(|| {
                    ConversionError::MissingField("ai-session agent mapping".to_string())
                })?;

                // Map to StatusUpdate as ai-session doesn't have Heartbeat
                Ok(AISessionMessage::StatusUpdate {
                    agent_id: ai_agent_id,
                    status: "heartbeat".to_string(),
                    metrics: serde_json::json!({
                        "timestamp": timestamp.to_rfc3339(),
                    }),
                })
            }

            _ => {
                // Map other variants to Custom message
                Ok(AISessionMessage::Custom {
                    message_type: "ccswarm-message".to_string(),
                    data: serde_json::to_value(&self)?,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_agent_info_creation() {
        let agent = crate::agent::ClaudeCodeAgent::new(
            crate::identity::default_frontend_role(),
            &std::path::PathBuf::from("/tmp/test"),
            "test",
            crate::config::ClaudeConfig::default(),
        )
        .await
        .unwrap();

        let info = UnifiedAgentInfo::from_ccswarm_agent(&agent);
        assert_eq!(info.role.name(), "Frontend");
        assert!(info.capabilities.contains(&"Frontend".to_string()));
    }

    #[tokio::test]
    async fn test_agent_mapping_registry() {
        let registry = AgentMappingRegistry::new();

        let info = UnifiedAgentInfo {
            ccswarm_id: "test-agent-123".to_string(),
            ai_session_id: AISessionAgentId::new(),
            role: crate::identity::default_backend_role(),
            capabilities: vec!["Backend".to_string()],
            metadata: serde_json::json!({}),
        };

        registry.register(info.clone()).await;

        let ai_id = registry.get_ai_session_id("test-agent-123").await;
        assert!(ai_id.is_some());
        assert_eq!(ai_id.unwrap(), info.ai_session_id);

        let ccswarm_id = registry.get_ccswarm_id(&info.ai_session_id).await;
        assert_eq!(ccswarm_id, Some("test-agent-123".to_string()));
    }

    #[tokio::test]
    async fn test_message_conversion_task_completed() {
        let registry = AgentMappingRegistry::new();

        // Register test agent
        let ai_agent_id = AISessionAgentId::new();
        let info = UnifiedAgentInfo {
            ccswarm_id: "test-agent".to_string(),
            ai_session_id: ai_agent_id.clone(),
            role: crate::identity::default_frontend_role(),
            capabilities: vec!["Frontend".to_string()],
            metadata: serde_json::json!({}),
        };
        registry.register(info).await;

        // Test ai-session -> ccswarm conversion
        let ai_msg = AISessionMessage::TaskCompleted {
            agent_id: ai_agent_id,
            task_id: AISessionTaskId::new(),
            result: serde_json::json!({
                "success": true,
                "output": serde_json::json!({"message": "Task completed successfully"}),
                "error": None::<String>,
                "duration_secs": 42,
            }),
        };

        let ccswarm_msg = CCSwarmMessage::from_ai_session(ai_msg, &registry)
            .await
            .unwrap();

        match ccswarm_msg {
            CCSwarmMessage::TaskCompleted {
                agent_id, result, ..
            } => {
                assert_eq!(agent_id, "test-agent");
                assert!(result.success);
                assert!(result.output.get("message").is_some());
            }
            _ => panic!("Expected TaskCompleted message"),
        }
    }

    #[tokio::test]
    async fn test_helper_functions() {
        let registry = AgentMappingRegistry::new();

        // Register test agent
        let ai_agent_id = AISessionAgentId::new();
        let info = UnifiedAgentInfo {
            ccswarm_id: "backend-agent".to_string(),
            ai_session_id: ai_agent_id.clone(),
            role: crate::identity::default_backend_role(),
            capabilities: vec!["Backend".to_string()],
            metadata: serde_json::json!({
                "version": "1.0.0",
                "features": ["database", "api"],
            }),
        };
        registry.register(info).await;

        // Test ccswarm -> ai-session conversion using helper
        let ccswarm_msg = CCSwarmMessage::StatusUpdate {
            agent_id: "backend-agent".to_string(),
            status: AgentStatus::Working,
            metrics: serde_json::json!({
                "cpu_usage": 45.2,
                "memory_mb": 256,
            }),
        };

        let ai_msg = convert_to_ai_session(ccswarm_msg, &registry).await.unwrap();

        match ai_msg {
            AISessionMessage::StatusUpdate {
                agent_id,
                status,
                metrics,
            } => {
                assert_eq!(agent_id, ai_agent_id);
                assert_eq!(status, "working");
                assert_eq!(metrics.get("cpu_usage"), Some(&serde_json::json!(45.2)));
            }
            _ => panic!("Expected StatusUpdate message"),
        }

        // Test ai-session -> ccswarm conversion using helper
        let ai_help_msg = AISessionMessage::HelpRequest {
            agent_id: ai_agent_id,
            context: "Need help with database connection".to_string(),
            priority: AISessionPriority::High,
        };

        let ccswarm_help = convert_from_ai_session(ai_help_msg, &registry)
            .await
            .unwrap();

        match ccswarm_help {
            CCSwarmMessage::RequestAssistance {
                agent_id, reason, ..
            } => {
                assert_eq!(agent_id, "backend-agent");
                assert_eq!(reason, "Need help with database connection");
            }
            _ => panic!("Expected RequestAssistance message"),
        }
    }

    #[tokio::test]
    async fn test_bidirectional_conversion() {
        let registry = AgentMappingRegistry::new();

        // Register test agents
        let frontend_ai_id = AISessionAgentId::new();
        let backend_ai_id = AISessionAgentId::new();

        registry
            .register(UnifiedAgentInfo {
                ccswarm_id: "frontend-agent".to_string(),
                ai_session_id: frontend_ai_id.clone(),
                role: crate::identity::default_frontend_role(),
                capabilities: vec!["Frontend".to_string()],
                metadata: serde_json::json!({}),
            })
            .await;

        registry
            .register(UnifiedAgentInfo {
                ccswarm_id: "backend-agent".to_string(),
                ai_session_id: backend_ai_id.clone(),
                role: crate::identity::default_backend_role(),
                capabilities: vec!["Backend".to_string()],
                metadata: serde_json::json!({}),
            })
            .await;

        // Create inter-agent message
        let original = CCSwarmMessage::InterAgentMessage {
            from_agent: "frontend-agent".to_string(),
            to_agent: "backend-agent".to_string(),
            message: "API endpoint ready for integration".to_string(),
            timestamp: Utc::now(),
        };

        // Convert to ai-session format
        let ai_msg = convert_to_ai_session(original.clone(), &registry)
            .await
            .unwrap();

        // Convert back to ccswarm format
        let restored = convert_from_ai_session(ai_msg, &registry).await.unwrap();

        // Should maintain the essential information through round-trip
        match (original, restored) {
            (
                CCSwarmMessage::InterAgentMessage {
                    from_agent: from1,
                    to_agent: to1,
                    message: msg1,
                    ..
                },
                CCSwarmMessage::Coordination { payload, .. },
            ) => {
                // InterAgentMessage converts to Custom AI message and back to Coordination
                // The payload contains the serialized AISessionMessage::Custom
                let custom = payload.get("Custom").unwrap();
                let data = custom.get("data").unwrap();
                let inter_agent_msg = data.get("InterAgentMessage").unwrap();

                assert_eq!(
                    inter_agent_msg.get("from_agent").unwrap().as_str().unwrap(),
                    from1
                );
                assert_eq!(
                    inter_agent_msg.get("to_agent").unwrap().as_str().unwrap(),
                    to1
                );
                assert_eq!(
                    inter_agent_msg.get("message").unwrap().as_str().unwrap(),
                    msg1
                );
            }
            _ => panic!("Unexpected message types in round-trip conversion"),
        }
    }
}
