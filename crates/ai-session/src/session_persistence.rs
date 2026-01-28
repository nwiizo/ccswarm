//! Persistent session manager for the CLI
//!
//! This module provides a SessionManager wrapper that persists sessions
//! across CLI invocations using the ai-session library's PersistenceManager.

use crate::core::{SessionId, SessionStatus};
use crate::persistence::{PersistenceManager, SessionMetadata, SessionState};
use crate::{AISession, SessionConfig, SessionManager as InnerSessionManager};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Global session manager instance
static MANAGER_INSTANCE: tokio::sync::OnceCell<Arc<PersistentSessionManager>> =
    tokio::sync::OnceCell::const_new();

/// Get the global persistent session manager
pub async fn get_session_manager() -> Result<Arc<PersistentSessionManager>> {
    Ok(MANAGER_INSTANCE
        .get_or_init(|| async { Arc::new(PersistentSessionManager::new().await.unwrap()) })
        .await
        .clone())
}

/// Persistent session manager that wraps the inner SessionManager
/// and provides persistence across CLI invocations
pub struct PersistentSessionManager {
    /// Inner session manager
    inner: InnerSessionManager,
    /// Persistence manager for saving/loading sessions
    persistence: PersistenceManager,
    /// Cache directory for session data
    #[allow(dead_code)]
    storage_path: PathBuf,
}

impl PersistentSessionManager {
    /// Create a new persistent session manager
    pub async fn new() -> Result<Self> {
        let storage_path = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("ai-session")
            .join("sessions");

        std::fs::create_dir_all(&storage_path)?;
        eprintln!("Using storage path: {}", storage_path.display());

        let persistence = PersistenceManager::new(storage_path.clone());
        let inner = InnerSessionManager::new();

        let mut manager = Self {
            inner,
            persistence,
            storage_path,
        };

        // Load existing sessions
        manager.restore_sessions().await?;

        Ok(manager)
    }

    /// Restore sessions from persistent storage
    async fn restore_sessions(&mut self) -> Result<()> {
        eprintln!("Restoring sessions from persistence...");
        let session_ids = self.persistence.list_sessions().await?;
        eprintln!("Found {} sessions to restore", session_ids.len());

        for session_id in session_ids {
            match self.persistence.load_session(&session_id).await {
                Ok(state) => {
                    // Only restore sessions that were running or paused
                    match state.status {
                        SessionStatus::Running | SessionStatus::Paused => {
                            // Restore the session with its original ID
                            match self
                                .inner
                                .restore_session(
                                    state.session_id.clone(),
                                    state.config.clone(),
                                    state.metadata.created_at,
                                )
                                .await
                            {
                                Ok(_session) => {
                                    eprintln!("Restored session: {}", session_id);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to restore session {}: {}",
                                        session_id, e
                                    );
                                }
                            }
                        }
                        _ => {
                            // Skip terminated or error sessions
                            eprintln!("Skipping terminated session: {}", session_id);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to restore session {}: {}", session_id, e);
                }
            }
        }

        Ok(())
    }

    /// Create a new session with config and persist it
    pub async fn create_session_with_config(
        &self,
        config: SessionConfig,
    ) -> Result<Arc<AISession>> {
        let session = self.inner.create_session_with_config(config).await?;

        // Start the session first
        session.start().await?;

        // Save the session state after starting
        let state = SessionState {
            session_id: session.id.clone(),
            config: session.config.clone(),
            status: session.status().await,
            context: session.context.read().await.clone(),
            command_history: vec![],
            metadata: SessionMetadata::default(),
        };

        eprintln!(
            "Saving session {} with status {:?}",
            session.id, state.status
        );
        self.persistence.save_session(&session.id, &state).await?;
        eprintln!("Session {} saved successfully", session.id);

        Ok(session)
    }

    /// Get a session by ID, starting it if necessary
    pub async fn get_session(&self, id: &SessionId) -> Option<Arc<AISession>> {
        if let Some(session) = self.inner.get_session(id) {
            // Check if session needs to be started
            if session.status().await == SessionStatus::Initializing
                && let Err(e) = session.start().await
            {
                eprintln!("Error: Failed to start session {id}: {e}");
                return None;
            }
            Some(session)
        } else {
            // Try to load from persistence
            match self.persistence.load_session(id).await {
                Ok(state) => {
                    // Recreate the session
                    match self
                        .inner
                        .create_session_with_config(state.config.clone())
                        .await
                    {
                        Ok(session) => {
                            // Start the session
                            if let Err(e) = session.start().await {
                                eprintln!("Error: Failed to start restored session {}: {}", id, e);
                                return None;
                            }
                            Some(session)
                        }
                        Err(e) => {
                            eprintln!("Error: Failed to recreate session {}: {}", id, e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Debug: Session {} not found in persistence: {}", id, e);
                    None
                }
            }
        }
    }

    /// List all sessions (both active and persisted)
    pub async fn list_all_sessions(&self) -> Result<Vec<SessionId>> {
        let mut all_sessions = self.inner.list_sessions();
        let persisted = self.persistence.list_sessions().await?;

        // Add persisted sessions that aren't already active
        for session_id in persisted {
            if !all_sessions.contains(&session_id) {
                all_sessions.push(session_id);
            }
        }

        Ok(all_sessions)
    }

    /// Remove a session and delete its persistent data
    pub async fn remove_session(&self, id: &SessionId) -> Result<()> {
        // Remove from active sessions
        self.inner.remove_session(id).await?;

        // Delete persistent data
        self.persistence.delete_session(id).await?;

        Ok(())
    }

    /// Update session state in persistence
    pub async fn update_session_state(&self, session: &AISession) -> Result<()> {
        let state = SessionState {
            session_id: session.id.clone(),
            config: session.config.clone(),
            status: session.status().await,
            context: session.context.read().await.clone(),
            command_history: session.get_command_history().await,
            metadata: SessionMetadata {
                created_at: session.created_at,
                last_accessed: *session.last_activity.read().await,
                command_count: session.get_command_count().await,
                total_tokens: session.get_total_tokens().await,
                custom: session.metadata.read().await.clone(),
            },
        };

        self.persistence.save_session(&session.id, &state).await?;
        Ok(())
    }
}
