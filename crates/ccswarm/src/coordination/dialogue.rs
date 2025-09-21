//! Dialogue system integrated with coordination bus
//! Enables sophisticated multi-agent conversations

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use uuid::Uuid;

use super::{AgentMessage, CoordinationBus, CoordinationType};

/// Dialogue-enhanced coordination bus
pub struct DialogueCoordinationBus {
    /// Base coordination bus
    pub coordination_bus: CoordinationBus,
    /// Active conversations
    pub conversations: HashMap<String, Conversation>,
    /// Dialogue patterns learned from interactions
    pub dialogue_patterns: Vec<DialoguePattern>,
    /// Agent dialogue profiles
    pub agent_profiles: HashMap<String, AgentDialogueProfile>,
}

/// Multi-agent conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub participants: Vec<String>,
    pub topic: String,
    pub context: ConversationContext,
    pub dialogue_history: VecDeque<DialogueEntry>,
    pub conversation_state: ConversationState,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub conversation_type: ConversationType,
}

/// Types of conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationType {
    TaskCoordination, // Coordinating task execution
    ProblemSolving,   // Collaborative problem solving
    KnowledgeSharing, // Sharing information/expertise
    ReviewDiscussion, // Code/work review
    Planning,         // Planning future work
    Casual,           // General communication
}

/// Context for the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub project_context: Option<String>,
    pub task_context: Option<String>,
    pub shared_resources: Vec<String>,
    pub constraints: Vec<String>,
    pub goals: Vec<String>,
    pub related_conversations: Vec<String>,
}

/// State of the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub phase: ConversationPhase,
    pub turn_order: VecDeque<String>,
    pub current_speaker: Option<String>,
    pub waiting_for_response: Vec<String>,
    pub consensus_level: f32, // 0.0-1.0
    pub engagement_levels: HashMap<String, f32>,
    pub progress_indicators: HashMap<String, f32>,
}

/// Phases of conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationPhase {
    Opening,              // Initial greetings/setup
    InformationGathering, // Collecting information
    Discussion,           // Active discussion
    ProblemSolving,       // Working through issues
    DecisionMaking,       // Making decisions
    Planning,             // Planning next steps
    Summarizing,          // Wrapping up
    Closing,              // Final statements
}

/// Individual dialogue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub speaker_id: String,
    pub recipients: Vec<String>,
    pub message_type: DialogueMessageType,
    pub content: String,
    pub context_references: Vec<String>,
    pub emotional_tone: EmotionalTone,
    pub response_expectation: ResponseExpectation,
    pub thread_reference: Option<String>,
    pub urgency_level: UrgencyLevel,
}

/// Types of dialogue messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueMessageType {
    Question,
    Answer,
    Suggestion,
    Clarification,
    Agreement,
    Disagreement,
    Information,
    Request,
    Acknowledgment,
    StatusUpdate,
    Concern,
    Proposal,
}

/// Emotional tone of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmotionalTone {
    Neutral,
    Positive,
    Enthusiastic,
    Concerned,
    Frustrated,
    Curious,
    Confident,
    Uncertain,
    Supportive,
    Urgent,
}

/// What kind of response is expected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseExpectation {
    ImmediateResponse,
    ThoughtfulResponse,
    Acknowledgment,
    NoResponseNeeded,
    ConsensusRequired,
    DecisionRequired,
}

/// Urgency level of the message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Normal,
    High,
    Critical,
}

/// Agent's dialogue profile and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDialogueProfile {
    pub agent_id: String,
    pub communication_style: CommunicationStyle,
    pub response_patterns: ResponsePatterns,
    pub topic_expertise: HashMap<String, f32>,
    pub collaboration_preferences: CollaborationPreferences,
    pub conversation_history: ConversationHistory,
}

/// Communication style characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub formality_level: f32, // 0.0 = very casual, 1.0 = very formal
    pub verbosity: f32,       // 0.0 = terse, 1.0 = very detailed
    pub directness: f32,      // 0.0 = indirect, 1.0 = very direct
    pub supportiveness: f32,  // 0.0 = neutral, 1.0 = very supportive
    pub inquisitiveness: f32, // 0.0 = accepting, 1.0 = questioning
    pub patience_level: f32,  // 0.0 = impatient, 1.0 = very patient
}

/// Patterns in how an agent responds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePatterns {
    pub average_response_time: Duration,
    pub preferred_message_length: MessageLength,
    pub question_asking_frequency: f32,
    pub agreement_tendency: f32,
    pub detail_providing_level: f32,
    pub follow_up_likelihood: f32,
}

/// Preferred message length
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLength {
    Brief,         // < 50 words
    Medium,        // 50-150 words
    Detailed,      // 150-300 words
    Comprehensive, // > 300 words
}

/// Collaboration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationPreferences {
    pub prefers_async: bool,
    pub comfortable_with_interruptions: bool,
    pub likes_consensus_building: bool,
    pub prefers_structured_discussions: bool,
    pub open_to_peer_review: bool,
}

/// Conversation history and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub total_conversations: u32,
    pub successful_collaborations: u32,
    pub preferred_conversation_types: Vec<ConversationType>,
    pub effective_partnerships: HashMap<String, f32>,
    pub conversation_outcomes: HashMap<String, ConversationOutcome>,
}

/// Outcome of conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationOutcome {
    Successful { goals_achieved: Vec<String> },
    Partial { progress_made: f32 },
    Unsuccessful { issues: Vec<String> },
    Interrupted { reason: String },
}

/// Learned dialogue patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialoguePattern {
    pub pattern_id: String,
    pub pattern_name: String,
    pub trigger_conditions: Vec<String>,
    pub conversation_flow: Vec<ConversationFlowStep>,
    pub success_rate: f32,
    pub participants_count: usize,
    pub typical_duration: Duration,
    pub effectiveness_metrics: HashMap<String, f32>,
}

/// Step in conversation flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationFlowStep {
    pub step_type: FlowStepType,
    pub typical_messages: Vec<String>,
    pub expected_responses: Vec<String>,
    pub transition_conditions: Vec<String>,
}

/// Types of flow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowStepType {
    Opening,
    InformationExchange,
    ProblemIdentification,
    SolutionBrainstorming,
    DecisionPoint,
    TaskAssignment,
    ProgressCheck,
    Closure,
}

impl DialogueCoordinationBus {
    /// Create new dialogue-enhanced coordination bus
    pub async fn new() -> Result<Self> {
        Ok(Self {
            coordination_bus: CoordinationBus::new().await?,
            conversations: HashMap::new(),
            dialogue_patterns: Vec::new(),
            agent_profiles: HashMap::new(),
        })
    }

    /// Start a new conversation
    pub async fn start_conversation(
        &mut self,
        participants: Vec<String>,
        topic: String,
        conversation_type: ConversationType,
        context: ConversationContext,
    ) -> Result<String> {
        let conversation_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let conversation = Conversation {
            id: conversation_id.clone(),
            participants: participants.clone(),
            topic,
            context,
            dialogue_history: VecDeque::new(),
            conversation_state: ConversationState {
                phase: ConversationPhase::Opening,
                turn_order: participants.into_iter().collect(),
                current_speaker: None,
                waiting_for_response: Vec::new(),
                consensus_level: 0.0,
                engagement_levels: HashMap::new(),
                progress_indicators: HashMap::new(),
            },
            started_at: now,
            last_activity: now,
            conversation_type,
        };

        // Store conversation data for notifications
        let participants_clone = conversation.participants.clone();
        let topic_clone = conversation.topic.clone();

        self.conversations
            .insert(conversation_id.clone(), conversation);

        // Notify all participants
        for participant in &participants_clone {
            let message = AgentMessage::Coordination {
                from_agent: "system".to_string(),
                to_agent: participant.clone(),
                message_type: CoordinationType::Custom("conversation_started".to_string()),
                payload: serde_json::json!({
                    "conversation_id": conversation_id,
                    "topic": topic_clone,
                    "participants": participants_clone,
                }),
            };
            self.coordination_bus.send_message(message).await?;
        }

        Ok(conversation_id)
    }

    /// Add message to conversation
    pub async fn add_dialogue_message(
        &mut self,
        conversation_id: &str,
        speaker_id: String,
        content: String,
        message_type: DialogueMessageType,
        emotional_tone: EmotionalTone,
        response_expectation: ResponseExpectation,
    ) -> Result<()> {
        let conversation = self
            .conversations
            .get_mut(conversation_id)
            .ok_or_else(|| anyhow::anyhow!("Conversation not found: {}", conversation_id))?;

        let entry = DialogueEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            speaker_id: speaker_id.clone(),
            recipients: conversation
                .participants
                .iter()
                .filter(|&p| p != &speaker_id)
                .cloned()
                .collect(),
            message_type,
            content: content.clone(),
            context_references: Vec::new(),
            emotional_tone,
            response_expectation,
            thread_reference: None,
            urgency_level: UrgencyLevel::Normal,
        };

        conversation.dialogue_history.push_back(entry.clone());
        conversation.last_activity = Utc::now();

        // Update conversation state - store needed data first
        let speaker_id = entry.speaker_id.clone();
        let message_type = entry.message_type.clone();

        // Release the mutable borrow on conversation before calling the method
        conversation.conversation_state.current_speaker = Some(speaker_id.clone());

        // Update turn order
        if let Some(pos) = conversation
            .conversation_state
            .turn_order
            .iter()
            .position(|id| id == &speaker_id)
        {
            let speaker = conversation
                .conversation_state
                .turn_order
                .remove(pos)
                .unwrap();
            conversation
                .conversation_state
                .turn_order
                .push_front(speaker);
        }

        // Update conversation phase based on message type
        conversation.conversation_state.phase = match message_type {
            DialogueMessageType::Question => ConversationPhase::InformationGathering,
            DialogueMessageType::Suggestion | DialogueMessageType::Proposal => {
                ConversationPhase::ProblemSolving
            }
            DialogueMessageType::Agreement | DialogueMessageType::Disagreement => {
                ConversationPhase::DecisionMaking
            }
            DialogueMessageType::StatusUpdate => ConversationPhase::Planning,
            DialogueMessageType::Acknowledgment => ConversationPhase::Closing,
            _ => conversation.conversation_state.phase.clone(),
        };

        // Update engagement levels
        let engagement_boost = match message_type {
            DialogueMessageType::Question | DialogueMessageType::Suggestion => 0.2,
            DialogueMessageType::Information | DialogueMessageType::StatusUpdate => 0.1,
            _ => 0.05,
        };

        conversation
            .conversation_state
            .engagement_levels
            .entry(speaker_id.to_string())
            .and_modify(|e| *e = (*e + engagement_boost).min(1.0))
            .or_insert(0.5 + engagement_boost);

        // Send coordination messages to recipients
        for recipient in &entry.recipients {
            let message = AgentMessage::Coordination {
                from_agent: speaker_id.clone(),
                to_agent: recipient.clone(),
                message_type: CoordinationType::InformationRequest,
                payload: serde_json::json!({
                    "conversation_id": conversation_id,
                    "dialogue_entry": entry,
                }),
            };
            self.coordination_bus.send_message(message).await?;
        }

        // Learn from dialogue patterns
        self.learn_dialogue_patterns(conversation_id).await?;

        Ok(())
    }

    /// Update conversation state based on new message
    #[allow(dead_code)]
    fn update_conversation_state_static(
        &mut self,
        conversation: &mut Conversation,
        speaker_id: &str,
        message_type: &DialogueMessageType,
    ) {
        // Update current speaker
        conversation.conversation_state.current_speaker = Some(speaker_id.to_string());

        // Update turn order
        if let Some(pos) = conversation
            .conversation_state
            .turn_order
            .iter()
            .position(|id| id == speaker_id)
        {
            let speaker = conversation
                .conversation_state
                .turn_order
                .remove(pos)
                .unwrap();
            conversation
                .conversation_state
                .turn_order
                .push_front(speaker);
        }

        // Update conversation phase based on message type
        conversation.conversation_state.phase = match message_type {
            DialogueMessageType::Question => ConversationPhase::InformationGathering,
            DialogueMessageType::Suggestion | DialogueMessageType::Proposal => {
                ConversationPhase::ProblemSolving
            }
            DialogueMessageType::Agreement | DialogueMessageType::Disagreement => {
                ConversationPhase::DecisionMaking
            }
            DialogueMessageType::StatusUpdate => ConversationPhase::Planning,
            DialogueMessageType::Acknowledgment => ConversationPhase::Closing,
            _ => conversation.conversation_state.phase.clone(),
        };

        // Update engagement levels
        let engagement_boost = match message_type {
            DialogueMessageType::Question | DialogueMessageType::Suggestion => 0.2,
            DialogueMessageType::Information | DialogueMessageType::StatusUpdate => 0.1,
            _ => 0.05,
        };

        conversation
            .conversation_state
            .engagement_levels
            .entry(speaker_id.to_string())
            .and_modify(|e| *e = (*e + engagement_boost).min(1.0))
            .or_insert(0.5 + engagement_boost);
    }

    /// Learn dialogue patterns from conversations
    async fn learn_dialogue_patterns(&mut self, conversation_id: &str) -> Result<()> {
        let conversation = self
            .conversations
            .get(conversation_id)
            .ok_or_else(|| anyhow::anyhow!("Conversation not found"))?;

        // Analyze conversation flow
        if conversation.dialogue_history.len() >= 5 {
            let pattern = self.extract_dialogue_pattern(conversation);
            self.dialogue_patterns.push(pattern);
        }

        // Update agent profiles - collect data first to avoid borrow conflicts
        let mut profile_updates = Vec::new();
        for entry in &conversation.dialogue_history {
            profile_updates.push((entry.speaker_id.clone(), entry.clone()));
        }
        let conversation_topic = conversation.topic.clone();

        for (speaker_id, entry) in profile_updates {
            self.update_agent_profile_with_topic(&speaker_id, &entry, &conversation_topic)
                .await?;
        }

        Ok(())
    }

    /// Extract dialogue pattern from conversation
    fn extract_dialogue_pattern(&self, conversation: &Conversation) -> DialoguePattern {
        let recent_messages: Vec<_> = conversation.dialogue_history.iter().rev().take(5).collect();

        let flow_steps = recent_messages
            .iter()
            .map(|entry| ConversationFlowStep {
                step_type: match entry.message_type {
                    DialogueMessageType::Question => FlowStepType::InformationExchange,
                    DialogueMessageType::Suggestion => FlowStepType::SolutionBrainstorming,
                    DialogueMessageType::Agreement => FlowStepType::DecisionPoint,
                    DialogueMessageType::StatusUpdate => FlowStepType::ProgressCheck,
                    _ => FlowStepType::InformationExchange,
                },
                typical_messages: vec![entry.content.clone()],
                expected_responses: Vec::new(),
                transition_conditions: Vec::new(),
            })
            .collect();

        DialoguePattern {
            pattern_id: Uuid::new_v4().to_string(),
            pattern_name: format!("{:?}_pattern", conversation.conversation_type),
            trigger_conditions: vec![conversation.topic.clone()],
            conversation_flow: flow_steps,
            success_rate: 0.8, // Default success rate
            participants_count: conversation.participants.len(),
            typical_duration: Duration::from_secs(
                (Utc::now() - conversation.started_at).num_seconds() as u64,
            ),
            effectiveness_metrics: HashMap::new(),
        }
    }

    /// Update agent dialogue profile with topic
    async fn update_agent_profile_with_topic(
        &mut self,
        agent_id: &str,
        entry: &DialogueEntry,
        conversation_topic: &str,
    ) -> Result<()> {
        let profile = self
            .agent_profiles
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentDialogueProfile {
                agent_id: agent_id.to_string(),
                communication_style: CommunicationStyle {
                    formality_level: 0.5,
                    verbosity: 0.5,
                    directness: 0.5,
                    supportiveness: 0.5,
                    inquisitiveness: 0.5,
                    patience_level: 0.5,
                },
                response_patterns: ResponsePatterns {
                    average_response_time: Duration::from_secs(120),
                    preferred_message_length: MessageLength::Medium,
                    question_asking_frequency: 0.3,
                    agreement_tendency: 0.6,
                    detail_providing_level: 0.5,
                    follow_up_likelihood: 0.4,
                },
                topic_expertise: HashMap::new(),
                collaboration_preferences: CollaborationPreferences {
                    prefers_async: false,
                    comfortable_with_interruptions: true,
                    likes_consensus_building: true,
                    prefers_structured_discussions: false,
                    open_to_peer_review: true,
                },
                conversation_history: ConversationHistory {
                    total_conversations: 0,
                    successful_collaborations: 0,
                    preferred_conversation_types: Vec::new(),
                    effective_partnerships: HashMap::new(),
                    conversation_outcomes: HashMap::new(),
                },
            });

        // Update communication style based on message characteristics
        let message_length = entry.content.len();
        if message_length < 50 {
            profile.communication_style.verbosity =
                (profile.communication_style.verbosity * 0.9 + 0.2 * 0.1).max(0.0);
        } else if message_length > 200 {
            profile.communication_style.verbosity =
                (profile.communication_style.verbosity * 0.9 + 0.8 * 0.1).min(1.0);
        }

        // Update directness based on message type
        match entry.message_type {
            DialogueMessageType::Request | DialogueMessageType::Suggestion => {
                profile.communication_style.directness =
                    (profile.communication_style.directness * 0.9 + 0.7 * 0.1).min(1.0);
            }
            DialogueMessageType::Question => {
                profile.communication_style.inquisitiveness =
                    (profile.communication_style.inquisitiveness * 0.9 + 0.8 * 0.1).min(1.0);
            }
            _ => {}
        }

        // Update topic expertise
        profile
            .topic_expertise
            .entry(conversation_topic.to_string())
            .and_modify(|e| *e = (*e + 0.1).min(1.0))
            .or_insert(0.6);

        profile.conversation_history.total_conversations += 1;

        Ok(())
    }

    /// Get conversation summary
    pub fn get_conversation_summary(&self, conversation_id: &str) -> Option<ConversationSummary> {
        let conversation = self.conversations.get(conversation_id)?;

        Some(ConversationSummary {
            id: conversation.id.clone(),
            topic: conversation.topic.clone(),
            participants: conversation.participants.clone(),
            message_count: conversation.dialogue_history.len(),
            duration: (Utc::now() - conversation.started_at)
                .to_std()
                .unwrap_or(Duration::ZERO),
            current_phase: conversation.conversation_state.phase.clone(),
            consensus_level: conversation.conversation_state.consensus_level,
            last_activity: conversation.last_activity,
            conversation_type: conversation.conversation_type.clone(),
        })
    }

    /// Get active conversations
    pub fn get_active_conversations(&self) -> Vec<ConversationSummary> {
        self.conversations
            .values()
            .filter(|conv| {
                let time_since_activity = Utc::now() - conv.last_activity;
                time_since_activity.num_hours() < 24 // Active if activity within 24 hours
            })
            .filter_map(|conv| self.get_conversation_summary(&conv.id))
            .collect()
    }

    /// Get agent dialogue insights
    pub fn get_agent_dialogue_insights(&self, agent_id: &str) -> Option<AgentDialogueInsights> {
        let profile = self.agent_profiles.get(agent_id)?;

        Some(AgentDialogueInsights {
            agent_id: agent_id.to_string(),
            communication_style: profile.communication_style.clone(),
            total_conversations: profile.conversation_history.total_conversations,
            success_rate: if profile.conversation_history.total_conversations > 0 {
                profile.conversation_history.successful_collaborations as f32
                    / profile.conversation_history.total_conversations as f32
            } else {
                0.0
            },
            top_expertise_areas: profile
                .topic_expertise
                .iter()
                .map(|(topic, expertise)| (topic.clone(), *expertise))
                .collect::<Vec<_>>()
                .into_iter()
                .fold(Vec::new(), |mut acc, (topic, expertise)| {
                    acc.push((topic, expertise));
                    acc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    acc.truncate(5);
                    acc
                }),
            preferred_partners: profile
                .conversation_history
                .effective_partnerships
                .iter()
                .map(|(partner, effectiveness)| (partner.clone(), *effectiveness))
                .collect(),
        })
    }
}

/// Summary of a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub topic: String,
    pub participants: Vec<String>,
    pub message_count: usize,
    pub duration: Duration,
    pub current_phase: ConversationPhase,
    pub consensus_level: f32,
    pub last_activity: DateTime<Utc>,
    pub conversation_type: ConversationType,
}

/// Insights about an agent's dialogue behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDialogueInsights {
    pub agent_id: String,
    pub communication_style: CommunicationStyle,
    pub total_conversations: u32,
    pub success_rate: f32,
    pub top_expertise_areas: Vec<(String, f32)>,
    pub preferred_partners: HashMap<String, f32>,
}

