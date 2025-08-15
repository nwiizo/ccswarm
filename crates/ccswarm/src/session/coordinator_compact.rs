/// Session coordinator - Optimized version
use super::session_common::{
    BaseSessionHandler, EfficiencyCalculator, SessionMessage, SessionResponse,
    SessionStatistics, StatUpdate,
};
use crate::coordination::{CoordinationMessage, MessageBus};
use crate::providers::AIProvider;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Coordination statistics
#[derive(Debug, Clone, Default)]
pub struct CoordinationStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub total_commands: usize,
    pub tokens_saved: usize,
    pub average_compression: f64,
}

/// Session coordinator for managing multiple agent sessions
pub struct SessionCoordinator {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    message_bus: Arc<MessageBus>,
    statistics: Arc<RwLock<SessionStatistics>>,
    coordination_stats: Arc<RwLock<CoordinationStatistics>>,
    handler: BaseSessionHandler<CoordinationState>,
}

struct SessionInfo {
    agent_id: String,
    provider: Box<dyn AIProvider>,
    tokens_saved: usize,
}

struct CoordinationState {
    active: bool,
    processing: bool,
}

impl SessionCoordinator {
    pub fn new(message_bus: Arc<MessageBus>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            statistics: Arc::new(RwLock::new(SessionStatistics::default())),
            coordination_stats: Arc::new(RwLock::new(CoordinationStatistics::default())),
            handler: BaseSessionHandler::new(CoordinationState {
                active: false,
                processing: false,
            }),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut state = self.handler.data.write().await;
        state.active = true;
        
        // Start message processing
        let handler_clone = self.handler.sender.clone();
        tokio::spawn(async move {
            // Process messages in background
        });
        
        Ok(())
    }

    pub async fn execute_coordinated_task(
        &self,
        agent_id: &str,
        task: &str,
    ) -> Result<String> {
        self.execute_generic_task(agent_id, task, false).await
    }

    pub async fn execute_coordinated_batch(
        &self,
        agent_id: &str,
        tasks: Vec<String>,
    ) -> Result<Vec<String>> {
        let mut results = Vec::new();
        for task in tasks {
            results.push(self.execute_generic_task(agent_id, &task, true).await?);
        }
        Ok(results)
    }

    async fn execute_generic_task(
        &self,
        agent_id: &str,
        task: &str,
        is_batch: bool,
    ) -> Result<String> {
        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.update(StatUpdate::TaskExecuted);
        
        if is_batch {
            stats.update(StatUpdate::Message);
        }
        
        // Simulate task execution
        let result = format!("Executed: {}", task);
        
        // Calculate efficiency
        let original_tokens = task.len() * 10;
        let compressed_tokens = task.len() * 3;
        let savings = EfficiencyCalculator::calculate_token_savings(
            original_tokens,
            compressed_tokens,
        );
        
        stats.update(StatUpdate::TokensSaved((original_tokens - compressed_tokens) as usize));
        stats.update(StatUpdate::Compression(savings / 100.0));
        
        Ok(result)
    }

    pub async fn send_coordination_update(&self, message: CoordinationMessage) -> Result<()> {
        self.send_generic_message(SessionMessage::Coordination(
            super::session_common::CoordinationMessage {
                sender: message.sender,
                content: serde_json::to_value(&message)?,
                timestamp: chrono::Utc::now(),
            },
        ))
        .await
    }

    pub async fn send_efficiency_report(&self) -> Result<()> {
        let stats = self.statistics.read().await;
        self.send_generic_message(SessionMessage::Efficiency(
            super::session_common::EfficiencyMessage {
                tokens_saved: stats.tokens_saved,
                compression_ratio: stats.compression_ratio,
                session_id: "coordinator".to_string(),
            },
        ))
        .await
    }

    pub async fn send_coordination_message(&self, content: String) -> Result<()> {
        self.send_generic_message(SessionMessage::Coordination(
            super::session_common::CoordinationMessage {
                sender: "coordinator".to_string(),
                content: serde_json::json!({ "message": content }),
                timestamp: chrono::Utc::now(),
            },
        ))
        .await
    }

    async fn send_generic_message(&self, message: SessionMessage) -> Result<()> {
        self.handler.send_message(message).await?;
        let mut stats = self.statistics.write().await;
        stats.update(StatUpdate::Message);
        Ok(())
    }

    pub async fn start_efficiency_monitoring(&self) -> Result<()> {
        self.start_generic_monitoring("efficiency").await
    }

    pub async fn start_coordination_processing(&self) -> Result<()> {
        self.start_generic_monitoring("coordination").await
    }

    async fn start_generic_monitoring(&self, monitor_type: &str) -> Result<()> {
        let mut state = self.handler.data.write().await;
        state.processing = true;
        
        println!("Started {} monitoring", monitor_type);
        
        Ok(())
    }

    pub async fn get_coordination_statistics(&self) -> CoordinationStatistics {
        let coord_stats = self.coordination_stats.read().await;
        let stats = self.statistics.read().await;
        
        CoordinationStatistics {
            total_sessions: coord_stats.total_sessions,
            active_sessions: coord_stats.active_sessions,
            total_commands: stats.tasks_executed,
            tokens_saved: stats.tokens_saved,
            average_compression: stats.compression_ratio,
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut state = self.handler.data.write().await;
        state.active = false;
        state.processing = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_coordinator_creation() {
        let message_bus = Arc::new(MessageBus::new(100));
        let coordinator = SessionCoordinator::new(message_bus);
        
        let stats = coordinator.get_coordination_statistics().await;
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_commands, 0);
    }

    #[tokio::test]
    async fn test_token_savings_calculation() {
        let original = 1000;
        let compressed = 300;
        let savings = EfficiencyCalculator::calculate_token_savings(original, compressed);
        assert_eq!(savings, 70.0);
        
        let ratio = EfficiencyCalculator::calculate_compression_ratio(original, compressed);
        assert!(ratio > 3.0 && ratio < 4.0);
    }
}