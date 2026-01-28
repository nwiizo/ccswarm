//! Session lifecycle management

use super::headless::HeadlessHandle;
use super::pty::PtyHandle;
use super::terminal::TerminalHandle;
use super::{AISession, SessionConfig, SessionStatus};
use anyhow::Result;
use portable_pty::CommandBuilder;
use std::io::ErrorKind;

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

    let shell_env = std::env::var("SHELL").ok();
    let shell = session
        .config
        .shell
        .as_deref()
        .or(shell_env.as_deref())
        .unwrap_or("/bin/bash");

    let terminal = if session.config.force_headless {
        TerminalHandle::Headless(
            HeadlessHandle::spawn_shell(
                shell,
                &session.config.working_directory,
                session.config.environment.iter(),
            )
            .await?,
        )
    } else {
        match spawn_pty(&session.config, shell).await {
            Ok(pty) => TerminalHandle::Pty(pty),
            Err(err) => {
                if session.config.allow_headless_fallback && is_permission_denied(&err) {
                    tracing::warn!(
                        "PTY unavailable ({}). Falling back to headless shell for session {}",
                        err,
                        session.id
                    );
                    TerminalHandle::Headless(
                        HeadlessHandle::spawn_shell(
                            shell,
                            &session.config.working_directory,
                            session.config.environment.iter(),
                        )
                        .await?,
                    )
                } else {
                    return Err(err);
                }
            }
        }
    };

    // Store terminal handle
    {
        let mut terminal_lock = session.terminal.write().await;
        *terminal_lock = Some(terminal);
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

    // Clear terminal handle (this will close the underlying IO)
    {
        let mut terminal_lock = session.terminal.write().await;
        if let Some(terminal) = terminal_lock.take() {
            terminal.shutdown().await?;
        }
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

async fn spawn_pty(config: &SessionConfig, shell: &str) -> Result<PtyHandle> {
    let pty = PtyHandle::new(config.pty_size.0, config.pty_size.1)?;
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(&config.working_directory);

    for (key, value) in &config.environment {
        cmd.env(key, value);
    }

    pty.spawn_command(cmd).await?;
    Ok(pty)
}

fn is_permission_denied(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
            io_err.kind() == ErrorKind::PermissionDenied
        } else {
            let msg = cause.to_string();
            msg.contains("PermissionDenied") || msg.contains("Operation not permitted")
        }
    })
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
