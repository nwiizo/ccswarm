//! JSON file persistence for Sangha state

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use super::Sangha;
use super::proposal::Proposal;

/// Persistent state format
#[derive(Debug, Serialize, Deserialize)]
pub struct SanghaState {
    pub sangha: Sangha,
    pub proposals: Vec<Proposal>,
}

/// Handles JSON persistence for Sangha
pub struct SanghaPersistence {
    state_dir: PathBuf,
}

impl SanghaPersistence {
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
        }
    }

    pub fn default_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ccswarm")
            .join("sangha")
    }

    pub async fn save(&self, state: &SanghaState) -> Result<()> {
        fs::create_dir_all(&self.state_dir).await?;
        let path = self.state_dir.join("state.json");
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<Option<SanghaState>> {
        let path = self.state_dir.join("state.json");
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path).await?;
        let state: SanghaState = serde_json::from_str(&content)?;
        Ok(Some(state))
    }

    pub async fn save_proposal(&self, proposal: &Proposal) -> Result<()> {
        let proposals_dir = self.state_dir.join("proposals");
        fs::create_dir_all(&proposals_dir).await?;
        let path = proposals_dir.join(format!("{}.json", proposal.id));
        let content = serde_json::to_string_pretty(proposal)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn load_proposals(&self) -> Result<Vec<Proposal>> {
        let proposals_dir = self.state_dir.join("proposals");
        if !proposals_dir.exists() {
            return Ok(Vec::new());
        }

        let mut proposals = Vec::new();
        let mut entries = fs::read_dir(&proposals_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(proposal) = serde_json::from_str::<Proposal>(&content) {
                    proposals.push(proposal);
                }
            }
        }
        Ok(proposals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sangha::SanghaMember;

    #[tokio::test]
    async fn test_save_and_load_state() {
        let dir = tempfile::tempdir().unwrap();
        let persistence = SanghaPersistence::new(dir.path());

        let mut sangha = Sangha::new(0.5);
        sangha.add_member(SanghaMember::new("a1", "backend", 1.0));

        let state = SanghaState {
            sangha,
            proposals: Vec::new(),
        };

        persistence.save(&state).await.unwrap();
        let loaded = persistence.load().await.unwrap();
        assert!(loaded.is_some());
        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.sangha.members.len(), 1);
        assert_eq!(loaded_state.sangha.members[0].id, "a1");
    }

    #[tokio::test]
    async fn test_save_and_load_proposal() {
        let dir = tempfile::tempdir().unwrap();
        let persistence = SanghaPersistence::new(dir.path());

        let proposal = Proposal::new("Test Proposal", "Description", "feature", "agent-1", 24);
        let proposal_id = proposal.id.clone();

        persistence.save_proposal(&proposal).await.unwrap();
        let loaded = persistence.load_proposals().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, proposal_id);
        assert_eq!(loaded[0].title, "Test Proposal");
    }

    #[tokio::test]
    async fn test_load_empty_state() {
        let dir = tempfile::tempdir().unwrap();
        let persistence = SanghaPersistence::new(dir.path());

        let loaded = persistence.load().await.unwrap();
        assert!(loaded.is_none());

        let proposals = persistence.load_proposals().await.unwrap();
        assert!(proposals.is_empty());
    }
}
