# ai-session Migration Plan for ccswarm

## Overview

This document outlines the migration plan to leverage ai-session's advanced features in ccswarm, replacing the current SessionManager implementation with ai-session's more powerful capabilities.

## Current State Analysis

### ccswarm's SessionManager (src/session/mod.rs)
- Custom tmux-based session management
- Basic session lifecycle (Active, Paused, Detached, Background, Terminated)
- Memory system integration
- Auto-accept functionality
- Agent role-based session creation

### ai-session's Advanced Features
- **AISession**: Token-efficient context handling with 93% reduction
- **SessionContext**: Intelligent conversation history management
- **MessageBus**: Multi-agent coordination
- **Observability**: Decision tracking and performance profiling
- **Security**: Capability-based access control
- **Output Parsing**: Semantic understanding of command outputs

## Migration Strategy

### Phase 1: Session Management Core
1. Replace ccswarm's SessionManager with ai-session's SessionManager
2. Migrate AgentSession to use AISession
3. Update session lifecycle states to match ai-session's model

### Phase 2: Context Integration
1. Replace ccswarm's memory system with ai-session's SessionContext
2. Enable token-efficient context management
3. Implement context compression strategies

### Phase 3: Multi-Agent Coordination
1. Replace JSON file coordination with ai-session's MessageBus
2. Implement agent discovery and registration
3. Enable real-time inter-agent communication

### Phase 4: Advanced Features
1. Add observability for decision tracking
2. Enable security features for safe command execution
3. Implement intelligent output parsing

## Implementation Details

### 1. Session Manager Adapter

```rust
// src/session/ai_session_adapter.rs
use ai_session::{SessionManager as AISessionManager, SessionConfig, AISession};
use crate::identity::AgentRole;
use crate::session::{AgentSession, SessionStatus};

pub struct SessionManagerAdapter {
    ai_manager: AISessionManager,
}

impl SessionManagerAdapter {
    pub async fn create_agent_session(
        &self,
        agent_id: String,
        agent_role: AgentRole,
        working_directory: PathBuf,
    ) -> Result<AgentSession> {
        let mut config = SessionConfig::default();
        config.name = Some(format!("ccswarm-{}-{}", agent_role.name(), &agent_id[..8]));
        config.working_directory = working_directory;
        config.enable_ai_features = true;
        config.agent_role = Some(agent_role.name().to_string());
        
        // Enable advanced features
        config.context_config.max_tokens = 8192; // Double the default
        config.compress_output = true;
        config.parse_output = true;
        
        let ai_session = self.ai_manager.create_session_with_config(config).await?;
        
        // Convert to ccswarm's AgentSession format
        Ok(self.adapt_session(ai_session, agent_id, agent_role))
    }
}
```

### 2. Context Management Integration

```rust
// src/session/context_bridge.rs
use ai_session::context::{Message, MessageRole, SessionContext};
use crate::session::memory::{WorkingMemoryType, EpisodeType};

pub struct ContextBridge {
    ai_context: Arc<RwLock<SessionContext>>,
}

impl ContextBridge {
    pub async fn add_agent_memory(
        &self,
        content: String,
        memory_type: WorkingMemoryType,
        priority: f32,
    ) -> Result<()> {
        let message = Message {
            role: MessageRole::System,
            content: format!("[{}] {}", memory_type, content),
            timestamp: Utc::now(),
            token_count: estimate_tokens(&content),
        };
        
        let mut context = self.ai_context.write().await;
        context.add_message(message).await?;
        
        // Trigger compression if needed
        if context.get_total_tokens() > context.config.max_tokens as usize * 0.8 {
            context.compress_context().await?;
        }
        
        Ok(())
    }
}
```

### 3. MessageBus Integration

```rust
// src/coordination/ai_message_bus.rs
use ai_session::coordination::{MessageBus, AgentMessage, AgentId};
use crate::orchestrator::MasterClaude;

pub struct AICoordinationBridge {
    message_bus: Arc<MessageBus>,
    master_handler: Arc<MasterClaude>,
}

impl AICoordinationBridge {
    pub async fn initialize(&self) -> Result<()> {
        // Subscribe to agent messages
        let mut receiver = self.message_bus.subscribe_all();
        
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    AgentMessage::TaskCompleted { agent_id, task_id, result } => {
                        self.master_handler.handle_task_completion(agent_id, task_id, result).await;
                    }
                    AgentMessage::HelpRequest { agent_id, context } => {
                        self.master_handler.handle_help_request(agent_id, context).await;
                    }
                    // ... other message types
                }
            }
        });
        
        Ok(())
    }
}
```

### 4. Observability Integration

```rust
// src/monitoring/ai_observability.rs
use ai_session::observability::{DecisionTracker, AIProfiler};

pub struct ObservabilityBridge {
    decision_tracker: Arc<DecisionTracker>,
    profiler: Arc<AIProfiler>,
}

impl ObservabilityBridge {
    pub async fn track_delegation_decision(
        &self,
        task: &Task,
        agent: &AgentRole,
        confidence: f64,
        reasoning: String,
    ) -> Result<()> {
        self.decision_tracker.track_decision(
            "task_delegation",
            serde_json::json!({
                "task_id": task.id,
                "agent": agent.name(),
                "confidence": confidence,
            }),
            reasoning,
        ).await?;
        
        Ok(())
    }
}
```

## Migration Steps

### Step 1: Create Adapter Layer (Week 1)
- [ ] Create `ai_session_adapter.rs` with SessionManagerAdapter
- [ ] Implement session lifecycle mapping
- [ ] Add conversion utilities between session types
- [ ] Update existing code to use adapter

### Step 2: Context Management (Week 2)
- [ ] Create `context_bridge.rs` for memory system migration
- [ ] Replace WorkingMemory with SessionContext
- [ ] Implement token counting and compression
- [ ] Update agents to use new context system

### Step 3: Coordination (Week 3)
- [ ] Create `ai_message_bus.rs` for coordination
- [ ] Replace JSON file coordination
- [ ] Implement agent discovery
- [ ] Update Master Claude to use MessageBus

### Step 4: Advanced Features (Week 4)
- [ ] Add observability tracking
- [ ] Enable security features
- [ ] Implement output parsing
- [ ] Add performance monitoring

## Testing Plan

1. **Unit Tests**: Test each adapter component
2. **Integration Tests**: Test full session lifecycle
3. **Performance Tests**: Verify 93% token reduction
4. **Migration Tests**: Ensure backward compatibility

## Rollback Plan

1. Keep original SessionManager as fallback
2. Feature flag for gradual rollout
3. Data migration utilities for session state

## Success Metrics

- 93% reduction in API tokens
- Improved agent coordination latency
- Enhanced debugging through observability
- Zero downtime during migration

## Timeline

- Week 1: Adapter layer and basic integration
- Week 2: Context management migration
- Week 3: Coordination system upgrade
- Week 4: Advanced features and testing
- Week 5: Rollout and monitoring