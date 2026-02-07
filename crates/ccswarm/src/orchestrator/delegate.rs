//! Delegate Mode for Lead-only orchestration
//!
//! In delegate mode, the lead agent only orchestrates - no direct code execution.
//! Tasks are split, assigned, and results integrated by the lead.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::team::{Team, TeamTaskStatus};

/// Delegate mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateConfig {
    pub enabled: bool,
    pub max_task_split: usize,
    pub auto_merge_results: bool,
    pub require_all_complete: bool,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_task_split: 10,
            auto_merge_results: true,
            require_all_complete: true,
        }
    }
}

/// A task split decision by the lead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSplit {
    pub original_task: String,
    pub subtasks: Vec<SubtaskAssignment>,
}

/// Assignment of a subtask to a teammate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskAssignment {
    pub task_id: String,
    pub description: String,
    pub assigned_to: String,
    pub depends_on: Vec<String>,
}

/// Delegate mode orchestrator
pub struct DelegateOrchestrator {
    config: DelegateConfig,
    splits: HashMap<String, TaskSplit>,
}

impl DelegateOrchestrator {
    pub fn new(config: DelegateConfig) -> Self {
        Self {
            config,
            splits: HashMap::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Split a task into subtasks
    pub fn split_task(
        &mut self,
        original_task: impl Into<String>,
        subtasks: Vec<SubtaskAssignment>,
    ) -> Result<String, String> {
        if subtasks.len() > self.config.max_task_split {
            return Err(format!(
                "Too many subtasks: {} (max: {})",
                subtasks.len(),
                self.config.max_task_split
            ));
        }

        let task = original_task.into();
        let split = TaskSplit {
            original_task: task.clone(),
            subtasks,
        };
        self.splits.insert(task.clone(), split);
        Ok(task)
    }

    /// Check if all subtasks for an original task are complete
    pub fn is_task_complete(&self, original_task: &str, team: &Team) -> bool {
        if let Some(split) = self.splits.get(original_task) {
            split.subtasks.iter().all(|sub| {
                team.task_list
                    .tasks
                    .iter()
                    .find(|t| t.id == sub.task_id)
                    .map(|t| t.status == TeamTaskStatus::Completed)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    pub fn get_split(&self, original_task: &str) -> Option<&TaskSplit> {
        self.splits.get(original_task)
    }
}

impl Default for DelegateOrchestrator {
    fn default() -> Self {
        Self::new(DelegateConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_config() {
        let config = DelegateConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_task_split, 10);
    }

    #[test]
    fn test_split_task() {
        let mut orchestrator = DelegateOrchestrator::new(DelegateConfig {
            enabled: true,
            ..Default::default()
        });

        let result = orchestrator.split_task(
            "build-auth",
            vec![SubtaskAssignment {
                task_id: "t1".to_string(),
                description: "Create API".to_string(),
                assigned_to: "backend-agent".to_string(),
                depends_on: vec![],
            }],
        );

        assert!(result.is_ok());
        assert!(orchestrator.get_split("build-auth").is_some());
    }
}
