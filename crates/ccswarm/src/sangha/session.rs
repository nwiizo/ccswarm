//! Session management for Sangha meetings

use super::*;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Manages Sangha sessions (meetings)
#[derive(Debug)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    active_session: Arc<Mutex<Option<Uuid>>>,
}

/// Represents a Sangha session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub session_type: SessionType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub participants: Vec<String>,
    pub agenda: Vec<AgendaItem>,
    pub decisions: Vec<Decision>,
    pub status: SessionStatus,
    pub notes: String,
}

/// Types of Sangha sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// Regular scheduled meeting
    Regular,
    /// Emergency session for urgent matters
    Emergency,
    /// Special session for specific topics
    Special,
    /// Session for system extensions
    SystemReview,
    /// Session for agent extensions
    ExtensionReview,
}

/// Status of a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

/// An agenda item for discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub item_type: AgendaItemType,
    pub proposal_id: Option<Uuid>,
    pub presenter: String,
    pub time_allocated: Duration,
    pub status: AgendaItemStatus,
}

/// Types of agenda items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgendaItemType {
    ProposalDiscussion,
    StatusUpdate,
    ProblemSolving,
    Planning,
    Review,
}

/// Status of an agenda item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgendaItemStatus {
    Pending,
    InDiscussion,
    Completed,
    Deferred,
}

/// A decision made during a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub agenda_item_id: Uuid,
    pub decision_type: DecisionType,
    pub description: String,
    pub rationale: String,
    pub made_at: DateTime<Utc>,
    pub implementation_deadline: Option<DateTime<Utc>>,
    pub responsible_agents: Vec<String>,
}

/// Types of decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    Approval,
    Rejection,
    Deferral,
    Amendment,
    TaskAssignment,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        session_type: SessionType,
        agenda: Vec<AgendaItem>,
    ) -> Result<Uuid> {
        let session = Session {
            id: Uuid::new_v4(),
            session_type,
            started_at: Utc::now(),
            ended_at: None,
            participants: Vec::new(),
            agenda,
            decisions: Vec::new(),
            status: SessionStatus::Scheduled,
            notes: String::new(),
        };

        let session_id = session.id;
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);

        Ok(session_id)
    }

    /// Start a session
    pub async fn start_session(&self, session_id: Uuid) -> Result<()> {
        let mut active = self.active_session.lock().await;
        if active.is_some() {
            anyhow::bail!("Another session is already in progress");
        }

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id).context("Session not found")?;

        if session.status != SessionStatus::Scheduled {
            anyhow::bail!("Session is not in scheduled state");
        }

        session.status = SessionStatus::InProgress;
        session.started_at = Utc::now();
        *active = Some(session_id);

        Ok(())
    }

    /// End the current session
    pub async fn end_session(&self) -> Result<()> {
        let mut active = self.active_session.lock().await;
        let session_id = active.take().context("No active session")?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id).context("Session not found")?;

        session.status = SessionStatus::Completed;
        session.ended_at = Some(Utc::now());

        Ok(())
    }

    /// Add a participant to the current session
    pub async fn add_participant(&self, agent_id: String) -> Result<()> {
        let active = self.active_session.lock().await;
        let session_id = active.as_ref().context("No active session")?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).context("Session not found")?;

        if !session.participants.contains(&agent_id) {
            session.participants.push(agent_id);
        }

        Ok(())
    }

    /// Remove a participant from the current session
    pub async fn remove_participant(&self, agent_id: &str) -> Result<()> {
        let active = self.active_session.lock().await;
        let session_id = active.as_ref().context("No active session")?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).context("Session not found")?;

        session.participants.retain(|id| id != agent_id);

        Ok(())
    }

    /// Update the status of an agenda item
    pub async fn update_agenda_item_status(
        &self,
        item_id: Uuid,
        status: AgendaItemStatus,
    ) -> Result<()> {
        let active = self.active_session.lock().await;
        let session_id = active.as_ref().context("No active session")?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).context("Session not found")?;

        for item in &mut session.agenda {
            if item.id == item_id {
                item.status = status;
                return Ok(());
            }
        }

        anyhow::bail!("Agenda item not found")
    }

    /// Record a decision made during the session
    pub async fn record_decision(&self, decision: Decision) -> Result<()> {
        let active = self.active_session.lock().await;
        let session_id = active.as_ref().context("No active session")?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).context("Session not found")?;

        session.decisions.push(decision);

        Ok(())
    }

    /// Get the current active session
    pub async fn get_active_session(&self) -> Option<Session> {
        let active = self.active_session.lock().await;
        if let Some(session_id) = active.as_ref() {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }

    /// Get all sessions
    pub async fn get_all_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get sessions by type
    pub async fn get_sessions_by_type(&self, session_type: SessionType) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.session_type == session_type)
            .cloned()
            .collect()
    }

    /// Create an emergency session
    pub async fn create_emergency_session(
        &self,
        reason: String,
        proposals: Vec<Uuid>,
    ) -> Result<Uuid> {
        let agenda: Vec<AgendaItem> = proposals
            .into_iter()
            .map(|proposal_id| AgendaItem {
                id: Uuid::new_v4(),
                title: format!("Emergency Review: Proposal {}", proposal_id),
                description: reason.clone(),
                item_type: AgendaItemType::ProposalDiscussion,
                proposal_id: Some(proposal_id),
                presenter: "system".to_string(),
                time_allocated: Duration::minutes(15),
                status: AgendaItemStatus::Pending,
            })
            .collect();

        let session_id = self.create_session(SessionType::Emergency, agenda).await?;

        // Automatically start emergency sessions
        self.start_session(session_id).await?;

        Ok(session_id)
    }

    /// Get session transcript
    pub async fn get_session_transcript(&self, session_id: Uuid) -> Result<SessionTranscript> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id).context("Session not found")?;

        Ok(SessionTranscript {
            session_id,
            session_type: session.session_type,
            started_at: session.started_at,
            ended_at: session.ended_at,
            duration: session.ended_at.map(|end| end - session.started_at),
            participants: session.participants.clone(),
            agenda_items: session.agenda.len(),
            decisions_made: session.decisions.len(),
            decisions: session.decisions.clone(),
            notes: session.notes.clone(),
        })
    }
}

/// Transcript of a completed session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscript {
    pub session_id: Uuid,
    pub session_type: SessionType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub participants: Vec<String>,
    pub agenda_items: usize,
    pub decisions_made: usize,
    pub decisions: Vec<Decision>,
    pub notes: String,
}

/// Session scheduler for regular meetings
pub struct SessionScheduler {
    schedule: Arc<RwLock<Vec<ScheduledSession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledSession {
    pub id: Uuid,
    pub session_type: SessionType,
    pub scheduled_for: DateTime<Utc>,
    pub recurrence: Option<Recurrence>,
    pub agenda_template: Vec<AgendaItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recurrence {
    Daily,
    Weekly,
    Monthly,
}

impl Default for SessionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionScheduler {
    pub fn new() -> Self {
        Self {
            schedule: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Schedule a recurring session
    pub async fn schedule_recurring(
        &self,
        session_type: SessionType,
        start_time: DateTime<Utc>,
        recurrence: Recurrence,
        agenda_template: Vec<AgendaItem>,
    ) -> Result<()> {
        let scheduled = ScheduledSession {
            id: Uuid::new_v4(),
            session_type,
            scheduled_for: start_time,
            recurrence: Some(recurrence),
            agenda_template,
        };

        let mut schedule = self.schedule.write().await;
        schedule.push(scheduled);

        Ok(())
    }

    /// Get upcoming sessions
    pub async fn get_upcoming(&self, limit: usize) -> Vec<ScheduledSession> {
        let schedule = self.schedule.read().await;
        let now = Utc::now();

        let mut upcoming: Vec<_> = schedule
            .iter()
            .filter(|s| s.scheduled_for > now)
            .cloned()
            .collect();

        upcoming.sort_by_key(|s| s.scheduled_for);
        upcoming.truncate(limit);

        upcoming
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = SessionManager::new();

        let agenda = vec![AgendaItem {
            id: Uuid::new_v4(),
            title: "Test Item".to_string(),
            description: "Test description".to_string(),
            item_type: AgendaItemType::ProposalDiscussion,
            proposal_id: None,
            presenter: "test-agent".to_string(),
            time_allocated: Duration::minutes(10),
            status: AgendaItemStatus::Pending,
        }];

        let session_id = manager
            .create_session(SessionType::Regular, agenda)
            .await
            .unwrap();

        // Start session
        manager.start_session(session_id).await.unwrap();

        // Add participant
        manager
            .add_participant("test-agent".to_string())
            .await
            .unwrap();

        // Get active session
        let active = manager.get_active_session().await;
        assert!(active.is_some());

        // End session
        manager.end_session().await.unwrap();

        // Verify no active session
        let active = manager.get_active_session().await;
        assert!(active.is_none());
    }
}
