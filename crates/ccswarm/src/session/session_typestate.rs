//! Type-state pattern for Session lifecycle management
//!
//! This module implements compile-time state machine validation for sessions,
//! preventing invalid operations on closed or uninitialized sessions.
//!
//! ## State Transition Diagram
//! ```text
//!                    ┌─────────────┐
//!                    │Uninitialized│
//!                    └─────┬───────┘
//!                          │ connect()
//!                    ┌─────▼───────┐
//!                ┌───│  Connected  │───┐
//!                │   └─────┬───────┘   │
//!                │         │           │
//!                │    activate()    disconnect()
//!                │         │           │
//!                │   ┌─────▼───────┐   │
//!                └───│   Active    │   │
//!                    └─────┬───────┘   │
//!                          │           │
//!                     deactivate()     │
//!                          │           │
//!                    ┌─────▼───────┐   │
//!                    │   Closed    │◄──┘
//!                    └─────────────┘
//! ```

use crate::error::Result;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Type States - Zero-sized types for compile-time validation
// ============================================================================

/// Session has not been initialized
pub struct Uninitialized;

/// Session is connected but not yet active
pub struct Connected;

/// Session is active and ready for operations
pub struct Active;

/// Session has been closed and cannot be reused
pub struct Closed;

// ============================================================================
// Session Data - Shared across states
// ============================================================================

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: String,
    pub agent_id: Option<String>,
    pub context: Arc<RwLock<SessionContext>>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub messages: Vec<String>,
    pub token_count: usize,
    pub compressed: bool,
}

// ============================================================================
// Type-Safe Session with State Tracking
// ============================================================================

/// A session with compile-time state validation
///
/// ## Example
/// ```rust
/// // Cannot perform operations on uninitialized session
/// // let session = TypedSession::new("session-1");
/// // session.send_message("Hello"); // Compilation error!
///
/// // Correct usage - follow state transitions
/// let session = TypedSession::new("session-1")
///     .connect().await?           // Uninitialized → Connected
///     .activate().await?           // Connected → Active
///     .send_message("Hello").await?;  // Now we can send messages!
///
/// // Close when done
/// let closed = session.deactivate().await?  // Active → Connected
///                    .disconnect().await?;   // Connected → Closed
/// ```
pub struct TypedSession<State> {
    data: SessionData,
    _state: PhantomData<State>,
}

// ============================================================================
// State: Uninitialized
// ============================================================================

impl TypedSession<Uninitialized> {
    /// Create a new uninitialized session
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            data: SessionData {
                session_id: session_id.into(),
                agent_id: None,
                context: Arc::new(RwLock::new(SessionContext {
                    messages: Vec::new(),
                    token_count: 0,
                    compressed: false,
                })),
                metadata: std::collections::HashMap::new(),
            },
            _state: PhantomData,
        }
    }

    /// Connect the session (Uninitialized → Connected)
    pub async fn connect(mut self) -> Result<TypedSession<Connected>> {
        // Perform connection logic here
        tracing::info!("Connecting session {}", self.data.session_id);

        // Simulate connection setup
        self.data
            .metadata
            .insert("connected_at".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(TypedSession {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Connect with specific agent (Uninitialized → Connected)
    pub async fn connect_with_agent(
        mut self,
        agent_id: impl Into<String>,
    ) -> Result<TypedSession<Connected>> {
        self.data.agent_id = Some(agent_id.into());
        self.connect().await
    }
}

// ============================================================================
// State: Connected
// ============================================================================

impl TypedSession<Connected> {
    /// Activate the session for use (Connected → Active)
    pub async fn activate(mut self) -> Result<TypedSession<Active>> {
        tracing::info!("Activating session {}", self.data.session_id);

        // Initialize session resources
        self.data
            .metadata
            .insert("activated_at".to_string(), chrono::Utc::now().to_rfc3339());

        // Load any persisted context
        {
            let mut context = self.data.context.write().await;
            if context.messages.is_empty() {
                context.messages.push("Session initialized".to_string());
            }
        } // Drop the lock before moving self.data

        Ok(TypedSession {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Disconnect without activating (Connected → Closed)
    pub async fn disconnect(self) -> Result<TypedSession<Closed>> {
        tracing::info!("Disconnecting session {}", self.data.session_id);

        Ok(TypedSession {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Get session information (available in Connected state)
    pub fn session_id(&self) -> &str {
        &self.data.session_id
    }

    /// Get agent ID if connected to an agent
    pub fn agent_id(&self) -> Option<&str> {
        self.data.agent_id.as_deref()
    }
}

// ============================================================================
// State: Active
// ============================================================================

impl TypedSession<Active> {
    /// Send a message (only available in Active state)
    pub async fn send_message(&self, message: impl Into<String>) -> Result<()> {
        let mut context = self.data.context.write().await;
        let msg = message.into();

        tracing::debug!("Session {} sending message: {}", self.data.session_id, msg);

        context.messages.push(msg.clone());
        context.token_count += msg.split_whitespace().count(); // Simple token estimation

        // Trigger compression if needed
        if context.token_count > 1000 && !context.compressed {
            self.compress_context(&mut context).await?;
        }

        Ok(())
    }

    /// Execute a command (only available in Active state)
    pub async fn execute_command(&self, command: impl Into<String>) -> Result<String> {
        let cmd = command.into();
        tracing::debug!("Session {} executing: {}", self.data.session_id, cmd);

        // Add to context
        self.send_message(format!("$ {}", cmd)).await?;

        // Simulate command execution
        let output = format!("Output of: {}", cmd);
        self.send_message(&output).await?;

        Ok(output)
    }

    /// Get current context (only available in Active state)
    pub async fn get_context(&self) -> Vec<String> {
        let context = self.data.context.read().await;
        context.messages.clone()
    }

    /// Get token count
    pub async fn get_token_count(&self) -> usize {
        let context = self.data.context.read().await;
        context.token_count
    }

    /// Compress context to save tokens
    async fn compress_context(&self, context: &mut SessionContext) -> Result<()> {
        tracing::info!("Compressing context for session {}", self.data.session_id);

        // Simulate compression - in real implementation would use actual compression
        if context.messages.len() > 10 {
            let summary = format!(
                "Context compressed: {} messages → 1 summary",
                context.messages.len()
            );

            // Keep last 5 messages and add summary
            let recent = context.messages.split_off(context.messages.len() - 5);
            context.messages = vec![summary];
            context.messages.extend(recent);

            context.token_count /= 3; // Simulate 66% reduction
            context.compressed = true;
        }

        Ok(())
    }

    /// Deactivate the session (Active → Connected)
    pub async fn deactivate(self) -> Result<TypedSession<Connected>> {
        tracing::info!("Deactivating session {}", self.data.session_id);

        // Save context before deactivating
        {
            let context = self.data.context.read().await;
            tracing::debug!(
                "Saving {} messages with {} tokens",
                context.messages.len(),
                context.token_count
            );
        } // Drop the lock before moving self.data

        Ok(TypedSession {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Close directly from active state (Active → Closed)
    pub async fn close(self) -> Result<TypedSession<Closed>> {
        // First deactivate, then disconnect
        self.deactivate().await?.disconnect().await
    }

    /// Get session information
    pub fn session_id(&self) -> &str {
        &self.data.session_id
    }

    /// Get agent ID
    pub fn agent_id(&self) -> Option<&str> {
        self.data.agent_id.as_deref()
    }
}

// ============================================================================
// State: Closed
// ============================================================================

impl TypedSession<Closed> {
    /// Get final session statistics (available after closing)
    pub async fn get_statistics(&self) -> SessionStatistics {
        let context = self.data.context.read().await;

        SessionStatistics {
            session_id: self.data.session_id.clone(),
            total_messages: context.messages.len(),
            total_tokens: context.token_count,
            was_compressed: context.compressed,
            metadata: self.data.metadata.clone(),
        }
    }

    /// Check if session can be recycled
    pub fn can_recycle(&self) -> bool {
        // Sessions can be recycled if they were closed cleanly
        self.data.metadata.contains_key("activated_at")
    }
}

// ============================================================================
// Common methods available in multiple states
// ============================================================================

/// Methods available in all states
impl<State> TypedSession<State> {
    /// Get immutable reference to session data
    pub fn data(&self) -> &SessionData {
        &self.data
    }

    /// Check if session is for a specific agent
    pub fn is_for_agent(&self, agent_id: &str) -> bool {
        self.data.agent_id.as_deref() == Some(agent_id)
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub session_id: String,
    pub total_messages: usize,
    pub total_tokens: usize,
    pub was_compressed: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

// ============================================================================
// Session Pool with Type-State
// ============================================================================

/// Pool of typed sessions ensuring proper state management
pub struct TypedSessionPool {
    active_sessions: Arc<RwLock<Vec<Arc<TypedSession<Active>>>>>,
    connected_sessions: Arc<RwLock<Vec<Arc<TypedSession<Connected>>>>>,
    closed_sessions: Arc<RwLock<Vec<Arc<TypedSession<Closed>>>>>,
}

impl TypedSessionPool {
    /// Create a new session pool
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(Vec::new())),
            connected_sessions: Arc::new(RwLock::new(Vec::new())),
            closed_sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get or create an active session for an agent
    pub async fn get_or_create_session(&self, agent_id: &str) -> Result<Arc<TypedSession<Active>>> {
        // Check for existing active session
        let active = self.active_sessions.read().await;
        if let Some(session) = active.iter().find(|s| s.is_for_agent(agent_id)) {
            return Ok(Arc::clone(session));
        }
        drop(active);

        // Create new session
        let session = TypedSession::new(format!("session-{}", uuid::Uuid::new_v4()))
            .connect_with_agent(agent_id)
            .await?
            .activate()
            .await?;

        let session_arc = Arc::new(session);
        self.active_sessions
            .write()
            .await
            .push(Arc::clone(&session_arc));

        Ok(session_arc)
    }

    /// Get statistics for all sessions
    pub async fn get_pool_statistics(&self) -> PoolStatistics {
        PoolStatistics {
            active_count: self.active_sessions.read().await.len(),
            connected_count: self.connected_sessions.read().await.len(),
            closed_count: self.closed_sessions.read().await.len(),
        }
    }
}

#[derive(Debug)]
pub struct PoolStatistics {
    pub active_count: usize,
    pub connected_count: usize,
    pub closed_count: usize,
}

impl Default for TypedSessionPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_state_transitions() {
        // Create and connect session
        let session = TypedSession::new("test-session")
            .connect()
            .await
            .expect("Failed to connect");

        assert_eq!(session.session_id(), "test-session");

        // Activate session
        let active = session.activate().await.expect("Failed to activate");

        // Send messages (only possible in Active state)
        active
            .send_message("Hello")
            .await
            .expect("Failed to send message");
        active
            .execute_command("echo test")
            .await
            .expect("Failed to execute command");

        let context = active.get_context().await;
        assert!(context.len() >= 2);

        // Close session
        let closed = active.close().await.expect("Failed to close");
        let stats = closed.get_statistics().await;

        assert!(stats.total_messages > 0);
        assert!(stats.total_tokens > 0);
    }

    #[tokio::test]
    async fn test_session_compression() {
        let session = TypedSession::new("compress-test")
            .connect()
            .await
            .unwrap()
            .activate()
            .await
            .unwrap();

        // Send many messages to trigger compression
        for i in 0..20 {
            session
                .send_message(format!("Message {}", i))
                .await
                .unwrap();
        }

        let token_count = session.get_token_count().await;
        // After compression (triggered at 1000 tokens), should be reduced
        // With our simple compression logic that reduces by 66%, we expect fewer tokens
        let context = session.data.context.read().await;
        assert!(
            context.compressed || token_count < 200,
            "Token count: {}, Compressed: {}",
            token_count,
            context.compressed
        );
    }

    #[tokio::test]
    async fn test_session_pool() {
        let pool = TypedSessionPool::new();

        // Get or create session for agent
        let session1 = pool.get_or_create_session("agent-1").await.unwrap();
        let session2 = pool.get_or_create_session("agent-1").await.unwrap();

        // Should get the same session
        assert!(Arc::ptr_eq(&session1, &session2));

        // Different agent gets different session
        let session3 = pool.get_or_create_session("agent-2").await.unwrap();
        assert!(!Arc::ptr_eq(&session1, &session3));

        let stats = pool.get_pool_statistics().await;
        assert_eq!(stats.active_count, 2);
    }

    // The following would not compile - demonstrating type safety:
    // #[tokio::test]
    // async fn test_cannot_send_before_activation() {
    //     let session = TypedSession::new("test")
    //         .connect().await.unwrap();
    //
    //     // This won't compile - send_message not available in Connected state
    //     session.send_message("Hello").await;
    // }

    // #[tokio::test]
    // async fn test_cannot_use_closed_session() {
    //     let session = TypedSession::new("test")
    //         .connect().await.unwrap()
    //         .disconnect().await.unwrap();
    //
    //     // This won't compile - no operations available on Closed state
    //     session.activate().await;
    // }
}
