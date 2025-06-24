//! Session lifecycle management

use super::pty::PtyHandle;
use super::{AISession, SessionStatus};
use anyhow::Result;
use portable_pty::CommandBuilder;

/// Start a session
pub async fn start_session(session: &AISession) -> Result<()> {
    // Update status
    {
        let mut status = session.status.write().await;
        if *status != SessionStatus::Initializing {
            return Err(anyhow::anyhow!("Session already started"));
        }
        *status = SessionStatus::Running;
    }

    // Create PTY
    let pty = PtyHandle::new(session.config.pty_size.0, session.config.pty_size.1)?;

    // Build command
    let shell_env = std::env::var("SHELL").ok();
    let shell = session
        .config
        .shell
        .as_deref()
        .or_else(|| shell_env.as_deref())
        .unwrap_or("/bin/bash");

    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(&session.config.working_directory);

    // Set environment variables
    for (key, value) in &session.config.environment {
        cmd.env(key, value);
    }

    // Spawn command in PTY
    pty.spawn_command(cmd).await?;

    // Store PTY handle
    {
        let mut pty_lock = session.pty.write().await;
        *pty_lock = Some(pty);
    }

    // Update last activity
    *session.last_activity.write().await = chrono::Utc::now();

    Ok(())
}

/// Stop a session
pub async fn stop_session(session: &AISession) -> Result<()> {
    // Update status
    {
        let mut status = session.status.write().await;
        if *status != SessionStatus::Running && *status != SessionStatus::Paused {
            return Ok(()); // Already stopped
        }
        *status = SessionStatus::Terminating;
    }

    // Clear PTY handle (this will close the PTY)
    {
        let mut pty_lock = session.pty.write().await;
        *pty_lock = None;
    }

    // Clear process handle
    {
        let mut process_lock = session.process.write().await;
        if let Some(mut process) = process_lock.take() {
            let _ = process.kill().await;
        }
    }

    // Update status
    {
        let mut status = session.status.write().await;
        *status = SessionStatus::Terminated;
    }

    Ok(())
}

/// Pause a session
pub async fn pause_session(session: &AISession) -> Result<()> {
    let mut status = session.status.write().await;
    if *status != SessionStatus::Running {
        return Err(anyhow::anyhow!("Session not running"));
    }
    *status = SessionStatus::Paused;
    Ok(())
}

/// Resume a session
pub async fn resume_session(session: &AISession) -> Result<()> {
    let mut status = session.status.write().await;
    if *status != SessionStatus::Paused {
        return Err(anyhow::anyhow!("Session not paused"));
    }
    *status = SessionStatus::Running;
    *session.last_activity.write().await = chrono::Utc::now();
    Ok(())
}
