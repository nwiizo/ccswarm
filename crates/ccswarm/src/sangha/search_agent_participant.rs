use crate::sangha::{Proposal, Vote, VoteChoice};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SanghaParticipant: Send + Sync {
    async fn vote_on_proposal(&self, proposal: &Proposal) -> Result<Vote>;
    async fn propose(&self, title: String, description: String) -> Result<Proposal>;
    async fn monitor_proposals(
        &mut self,
        sangha: std::sync::Arc<crate::sangha::Sangha>,
    ) -> Result<()>;
}

pub struct SearchAgentParticipant {
    agent_id: String,
}

impl SearchAgentParticipant {
    pub fn new(agent_id: String) -> Self {
        Self { agent_id }
    }
}

#[async_trait]
impl SanghaParticipant for SearchAgentParticipant {
    async fn vote_on_proposal(&self, proposal: &Proposal) -> Result<Vote> {
        Ok(Vote {
            proposal_id: proposal.id,
            voter_id: self.agent_id.clone(),
            choice: VoteChoice::Abstain,
            reason: Some("Automated vote".to_string()),
            cast_at: chrono::Utc::now(),
            weight: 1.0,
        })
    }

    async fn propose(&self, title: String, description: String) -> Result<Proposal> {
        use crate::sangha::proposal::ProposalBuilder;
        Ok(ProposalBuilder::new(
            title.clone(),
            self.agent_id.clone(),
            crate::sangha::ProposalType::AgentExtension,
        )
        .description(description)
        .build())
    }

    async fn monitor_proposals(
        &mut self,
        _sangha: std::sync::Arc<crate::sangha::Sangha>,
    ) -> Result<()> {
        // Simple monitoring loop
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            // In a real implementation, this would check for new proposals and vote
        }
    }
}

pub fn create_search_agent_participant(agent_id: String) -> Box<dyn SanghaParticipant> {
    Box::new(SearchAgentParticipant::new(agent_id))
}
