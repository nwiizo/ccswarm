//! Mailbox messaging system for agent communication
//!
//! Provides direct and broadcast messaging compatible with Agent Teams.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// A message in the mailbox system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage {
    pub id: String,
    pub from: String,
    pub to: MessageTarget,
    pub subject: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub priority: MailboxPriority,
}

/// Message target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Direct message to a specific agent
    Direct(String),
    /// Broadcast to all agents
    Broadcast,
    /// Message to a specific team
    Team(String),
}

/// Message priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MailboxPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl MailboxMessage {
    pub fn direct(
        from: impl Into<String>,
        to: impl Into<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: from.into(),
            to: MessageTarget::Direct(to.into()),
            subject: subject.into(),
            body: body.into(),
            timestamp: Utc::now(),
            read: false,
            priority: MailboxPriority::Normal,
        }
    }

    pub fn broadcast(
        from: impl Into<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: from.into(),
            to: MessageTarget::Broadcast,
            subject: subject.into(),
            body: body.into(),
            timestamp: Utc::now(),
            read: false,
            priority: MailboxPriority::Normal,
        }
    }

    pub fn with_priority(mut self, priority: MailboxPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// Mailbox for a single agent
#[derive(Debug, Default)]
pub struct AgentMailbox {
    inbox: VecDeque<MailboxMessage>,
    max_size: usize,
}

impl AgentMailbox {
    pub fn new(max_size: usize) -> Self {
        Self {
            inbox: VecDeque::new(),
            max_size,
        }
    }

    pub fn deliver(&mut self, message: MailboxMessage) {
        if self.inbox.len() >= self.max_size {
            self.inbox.pop_front(); // Drop oldest
        }
        self.inbox.push_back(message);
    }

    pub fn read_unread(&mut self) -> Vec<&MailboxMessage> {
        self.inbox.iter().filter(|m| !m.read).collect()
    }

    pub fn mark_read(&mut self, message_id: &str) {
        if let Some(msg) = self.inbox.iter_mut().find(|m| m.id == message_id) {
            msg.read = true;
        }
    }

    pub fn unread_count(&self) -> usize {
        self.inbox.iter().filter(|m| !m.read).count()
    }

    pub fn all_messages(&self) -> Vec<&MailboxMessage> {
        self.inbox.iter().collect()
    }
}

/// Central mailbox system managing all agent mailboxes
pub struct MailboxSystem {
    mailboxes: HashMap<String, AgentMailbox>,
    /// Team membership: team_id -> set of agent_ids
    team_members: HashMap<String, Vec<String>>,
    max_mailbox_size: usize,
}

impl MailboxSystem {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            team_members: HashMap::new(),
            max_mailbox_size: 1000,
        }
    }

    pub fn register_agent(&mut self, agent_id: impl Into<String>) {
        let id = agent_id.into();
        self.mailboxes
            .entry(id)
            .or_insert_with(|| AgentMailbox::new(self.max_mailbox_size));
    }

    /// Add an agent to a team for team-scoped messaging
    pub fn add_to_team(&mut self, agent_id: impl Into<String>, team_id: impl Into<String>) {
        let agent = agent_id.into();
        let team = team_id.into();
        self.team_members.entry(team).or_default().push(agent);
    }

    /// Remove an agent from a team
    pub fn remove_from_team(&mut self, agent_id: &str, team_id: &str) {
        if let Some(members) = self.team_members.get_mut(team_id) {
            members.retain(|id| id != agent_id);
        }
    }

    pub fn send(&mut self, message: MailboxMessage) {
        match &message.to {
            MessageTarget::Direct(to) => {
                if let Some(mailbox) = self.mailboxes.get_mut(to) {
                    mailbox.deliver(message);
                }
            }
            MessageTarget::Broadcast => {
                let sender = message.from.clone();
                for (agent_id, mailbox) in &mut self.mailboxes {
                    if *agent_id != sender {
                        mailbox.deliver(message.clone());
                    }
                }
            }
            MessageTarget::Team(team_id) => {
                let sender = message.from.clone();

                // Filter delivery to team members only
                if let Some(members) = self.team_members.get(team_id) {
                    for member_id in members {
                        if *member_id != sender
                            && let Some(mailbox) = self.mailboxes.get_mut(member_id)
                        {
                            mailbox.deliver(message.clone());
                        }
                    }
                }
                // If no team membership info, don't broadcast (fail silently)
            }
        }
    }

    pub fn get_mailbox(&self, agent_id: &str) -> Option<&AgentMailbox> {
        self.mailboxes.get(agent_id)
    }

    pub fn get_mailbox_mut(&mut self, agent_id: &str) -> Option<&mut AgentMailbox> {
        self.mailboxes.get_mut(agent_id)
    }

    pub fn idle_agents(&self) -> Vec<String> {
        self.mailboxes
            .iter()
            .filter(|(_, mb)| mb.unread_count() == 0)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for MailboxSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_message() {
        let mut system = MailboxSystem::new();
        system.register_agent("agent-1");
        system.register_agent("agent-2");

        let msg = MailboxMessage::direct("agent-1", "agent-2", "Hello", "Test message");
        system.send(msg);

        let mailbox = system.get_mailbox("agent-2").unwrap();
        assert_eq!(mailbox.unread_count(), 1);
    }

    #[test]
    fn test_broadcast() {
        let mut system = MailboxSystem::new();
        system.register_agent("lead");
        system.register_agent("agent-1");
        system.register_agent("agent-2");

        let msg = MailboxMessage::broadcast("lead", "Update", "Sprint complete");
        system.send(msg);

        // Lead shouldn't get their own broadcast
        assert_eq!(system.get_mailbox("lead").unwrap().unread_count(), 0);
        assert_eq!(system.get_mailbox("agent-1").unwrap().unread_count(), 1);
        assert_eq!(system.get_mailbox("agent-2").unwrap().unread_count(), 1);
    }
}
