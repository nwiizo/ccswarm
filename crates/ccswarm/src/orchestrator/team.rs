//! Team Management for Agent Teams
//!
//! Implements lead + teammates model compatible with Claude Code Agent Teams.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Team configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub lead_id: String,
    pub teammate_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub task_list: SharedTaskList,
    pub config: TeamConfig,
}

/// Team configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub max_teammates: usize,
    pub delegate_mode: bool,
    pub auto_assign: bool,
    pub file_lock_enabled: bool,
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            max_teammates: 10,
            delegate_mode: false,
            auto_assign: true,
            file_lock_enabled: true,
        }
    }
}

/// Shared task list for team coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedTaskList {
    pub tasks: Vec<TeamTask>,
}

/// A task in the shared list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTask {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TeamTaskStatus,
    pub assigned_to: Option<String>,
    pub blocked_by: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Status of a team task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamTaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl Team {
    pub fn new(name: impl Into<String>, lead_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            lead_id: lead_id.into(),
            teammate_ids: Vec::new(),
            created_at: Utc::now(),
            task_list: SharedTaskList { tasks: Vec::new() },
            config: TeamConfig::default(),
        }
    }

    pub fn add_teammate(&mut self, teammate_id: impl Into<String>) -> Result<(), String> {
        let id = teammate_id.into();
        if self.teammate_ids.len() >= self.config.max_teammates {
            return Err("Maximum teammates reached".to_string());
        }
        if self.teammate_ids.contains(&id) {
            return Err(format!("Teammate {} already in team", id));
        }
        self.teammate_ids.push(id);
        Ok(())
    }

    pub fn remove_teammate(&mut self, teammate_id: &str) -> bool {
        let before = self.teammate_ids.len();
        self.teammate_ids.retain(|id| id != teammate_id);
        self.teammate_ids.len() < before
    }

    pub fn add_task(
        &mut self,
        subject: impl Into<String>,
        description: impl Into<String>,
    ) -> String {
        let task = TeamTask {
            id: Uuid::new_v4().to_string(),
            subject: subject.into(),
            description: description.into(),
            status: TeamTaskStatus::Pending,
            assigned_to: None,
            blocked_by: Vec::new(),
            created_at: Utc::now(),
            completed_at: None,
        };
        let id = task.id.clone();
        self.task_list.tasks.push(task);
        id
    }

    pub fn claim_task(&mut self, task_id: &str, agent_id: &str) -> Result<(), String> {
        let task = self
            .task_list
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        if task.status != TeamTaskStatus::Pending {
            return Err(format!("Task {} is not pending", task_id));
        }
        if !task.blocked_by.is_empty() {
            return Err(format!("Task {} is blocked", task_id));
        }

        task.status = TeamTaskStatus::InProgress;
        task.assigned_to = Some(agent_id.to_string());
        Ok(())
    }

    pub fn complete_task(&mut self, task_id: &str) -> Result<(), String> {
        let task = self
            .task_list
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        task.status = TeamTaskStatus::Completed;
        task.completed_at = Some(Utc::now());

        // Unblock dependent tasks
        let completed_id = task_id.to_string();
        for t in &mut self.task_list.tasks {
            t.blocked_by.retain(|b| b != &completed_id);
            if t.blocked_by.is_empty() && t.status == TeamTaskStatus::Blocked {
                t.status = TeamTaskStatus::Pending;
            }
        }

        Ok(())
    }

    pub fn pending_tasks(&self) -> Vec<&TeamTask> {
        self.task_list
            .tasks
            .iter()
            .filter(|t| t.status == TeamTaskStatus::Pending)
            .collect()
    }
}

/// Team manager for persistence
pub struct TeamManager {
    teams: HashMap<String, Team>,
    config_dir: PathBuf,
}

impl TeamManager {
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ccswarm")
            .join("teams");
        Self {
            teams: HashMap::new(),
            config_dir,
        }
    }

    pub fn create_team(&mut self, name: impl Into<String>, lead_id: impl Into<String>) -> String {
        let team = Team::new(name, lead_id);
        let id = team.id.clone();
        self.teams.insert(id.clone(), team);
        id
    }

    pub fn get(&self, team_id: &str) -> Option<&Team> {
        self.teams.get(team_id)
    }

    pub fn get_mut(&mut self, team_id: &str) -> Option<&mut Team> {
        self.teams.get_mut(team_id)
    }

    pub fn list(&self) -> Vec<&Team> {
        self.teams.values().collect()
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.config_dir).await?;
        for (id, team) in &self.teams {
            let path = self.config_dir.join(format!("{}.json", id));
            let content = serde_json::to_string_pretty(team)?;
            tokio::fs::write(&path, content).await?;
        }
        Ok(())
    }

    pub async fn load(&mut self) -> anyhow::Result<()> {
        if !self.config_dir.exists() {
            return Ok(());
        }
        let mut entries = tokio::fs::read_dir(&self.config_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                if let Ok(team) = serde_json::from_str::<Team>(&content) {
                    self.teams.insert(team.id.clone(), team);
                }
            }
        }
        Ok(())
    }
}

impl Default for TeamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new("test-team", "lead-1");
        assert_eq!(team.lead_id, "lead-1");
        assert!(team.teammate_ids.is_empty());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut team = Team::new("test", "lead");
        let task_id = team.add_task("Fix bug", "Fix the login bug");

        assert!(team.claim_task(&task_id, "agent-1").is_ok());
        assert!(team.complete_task(&task_id).is_ok());

        let task = team
            .task_list
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .unwrap();
        assert_eq!(task.status, TeamTaskStatus::Completed);
    }
}
