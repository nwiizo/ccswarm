/// Manager for Claude Code subagents
///
/// This module provides functionality to manage, create, and coordinate
/// subagents within the ccswarm system.
use super::{
    parser::SubagentParser, SubagentConfig, SubagentDefinition, SubagentError, SubagentResult,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages subagent lifecycle and coordination
pub struct SubagentManager {
    config: SubagentConfig,
    agents: Arc<RwLock<HashMap<String, SubagentInstance>>>,
    definitions: Arc<RwLock<HashMap<String, (SubagentDefinition, String)>>>,
}

/// Represents an active subagent instance
#[derive(Debug, Clone)]
pub struct SubagentInstance {
    pub definition: SubagentDefinition,
    pub instructions: String,
    pub status: SubagentStatus,
    pub session_id: Option<String>,
}

/// Status of a subagent
#[derive(Debug, Clone, PartialEq)]
pub enum SubagentStatus {
    /// Subagent is available for tasks
    Available,
    /// Subagent is currently working on a task
    Busy,
    /// Subagent is being initialized
    Initializing,
    /// Subagent has encountered an error
    Error(String),
}

impl SubagentManager {
    /// Create a new subagent manager
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the manager by loading subagent definitions
    pub async fn initialize(&self) -> SubagentResult<()> {
        // Load definitions from the agents directory
        let definitions = SubagentParser::parse_directory(&self.config.agents_dir)?;

        let mut defs = self.definitions.write().await;
        for (definition, instructions) in definitions {
            log::info!("Loaded subagent definition: {}", definition.name);
            defs.insert(definition.name.clone(), (definition, instructions));
        }

        log::info!(
            "Initialized SubagentManager with {} definitions",
            defs.len()
        );
        Ok(())
    }

    /// Create a new subagent instance
    pub async fn create_subagent(&self, name: &str) -> SubagentResult<String> {
        // Check if we have the definition
        let definitions = self.definitions.read().await;
        let (definition, instructions) = definitions
            .get(name)
            .ok_or_else(|| SubagentError::NotFound(name.to_string()))?
            .clone();

        // Check concurrent agent limit
        let agents = self.agents.read().await;
        let active_count = agents
            .values()
            .filter(|a| matches!(a.status, SubagentStatus::Busy | SubagentStatus::Available))
            .count();

        if active_count >= self.config.max_concurrent_agents {
            return Err(SubagentError::Validation(format!(
                "Maximum concurrent agents ({}) reached",
                self.config.max_concurrent_agents
            )));
        }
        drop(agents);

        // Create the instance
        let instance = SubagentInstance {
            definition,
            instructions,
            status: SubagentStatus::Initializing,
            session_id: None,
        };

        // Generate a unique ID for this instance
        let instance_id = format!("{}-{}", name, uuid::Uuid::new_v4());

        let mut agents = self.agents.write().await;
        agents.insert(instance_id.clone(), instance);

        log::info!("Created subagent instance: {}", instance_id);

        // Initialize the subagent asynchronously
        let self_clone = self.clone();
        let id_clone = instance_id.clone();
        tokio::spawn(async move {
            if let Err(e) = self_clone.initialize_subagent(&id_clone).await {
                crate::utils::logging::log_init_failure("subagent", &id_clone, &e);
            }
        });

        Ok(instance_id)
    }

    /// Initialize a subagent instance
    async fn initialize_subagent(&self, instance_id: &str) -> SubagentResult<()> {
        // Here we would integrate with ai-session to create the actual session
        // For now, we'll simulate the initialization

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(instance_id) {
            // In a real implementation, we would create an ai-session here
            agent.session_id = Some(format!("session-{}", instance_id));
            agent.status = SubagentStatus::Available;
            log::info!("Subagent {} initialized successfully", instance_id);
        }

        Ok(())
    }

    /// Get the status of a subagent
    pub async fn get_status(&self, instance_id: &str) -> Option<SubagentStatus> {
        let agents = self.agents.read().await;
        agents.get(instance_id).map(|a| a.status.clone())
    }

    /// List all subagent definitions
    pub async fn list_definitions(&self) -> Vec<String> {
        let definitions = self.definitions.read().await;
        definitions.keys().cloned().collect()
    }

    /// List all active subagent instances
    pub async fn list_instances(&self) -> HashMap<String, SubagentStatus> {
        let agents = self.agents.read().await;
        agents
            .iter()
            .map(|(id, agent)| (id.clone(), agent.status.clone()))
            .collect()
    }

    /// Delegate a task to a subagent
    pub async fn delegate_task(&self, instance_id: &str, task: &str) -> SubagentResult<String> {
        let mut agents = self.agents.write().await;

        let agent = agents
            .get_mut(instance_id)
            .ok_or_else(|| SubagentError::NotFound(instance_id.to_string()))?;

        if agent.status != SubagentStatus::Available {
            return Err(SubagentError::Delegation(format!(
                "Subagent {} is not available",
                instance_id
            )));
        }

        agent.status = SubagentStatus::Busy;

        // Here we would actually delegate to the ai-session
        // For now, return a placeholder task ID
        let task_id = format!("task-{}", uuid::Uuid::new_v4());

        log::info!("Delegated task to subagent {}: {}", instance_id, task);

        Ok(task_id)
    }

    /// Mark a subagent as available after completing a task
    pub async fn complete_task(&self, instance_id: &str) -> SubagentResult<()> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get_mut(instance_id) {
            agent.status = SubagentStatus::Available;
            log::info!("Subagent {} completed task", instance_id);
        }

        Ok(())
    }

    /// Remove a subagent instance
    pub async fn remove_subagent(&self, instance_id: &str) -> SubagentResult<()> {
        let mut agents = self.agents.write().await;

        if agents.remove(instance_id).is_some() {
            log::info!("Removed subagent instance: {}", instance_id);
            Ok(())
        } else {
            Err(SubagentError::NotFound(instance_id.to_string()))
        }
    }

    /// Create a dynamic subagent definition
    pub async fn create_dynamic_definition(
        &self,
        name: String,
        description: String,
        tools: super::SubagentTools,
        capabilities: Vec<String>,
        instructions: String,
    ) -> SubagentResult<()> {
        if !self.config.enable_dynamic_generation {
            return Err(SubagentError::Validation(
                "Dynamic subagent generation is disabled".to_string(),
            ));
        }

        let definition = SubagentDefinition {
            name: name.clone(),
            description,
            tools,
            capabilities,
            metadata: HashMap::new(),
        };

        // Validate the definition
        SubagentParser::validate_definition(&mut definition.clone())?;

        let mut definitions = self.definitions.write().await;
        definitions.insert(name.clone(), (definition, instructions));

        log::info!("Created dynamic subagent definition: {}", name);

        Ok(())
    }
}

impl Clone for SubagentManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            agents: Arc::clone(&self.agents),
            definitions: Arc::clone(&self.definitions),
        }
    }
}

