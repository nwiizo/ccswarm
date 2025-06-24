//! Native session management - replaces tmux dependency

use anyhow::{Context, Result};
use nix::pty::{openpty, OpenptyResult, Winsize};
use nix::unistd::{close, dup2};
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Native session that replaces tmux functionality
pub struct NativeSession {
    /// Session ID
    id: String,
    /// Session name
    name: String,
    /// PTY master file descriptor
    pty_master: RawFd,
    /// Child process
    child: Option<Child>,
    /// Output buffer
    output_buffer: Arc<Mutex<Vec<u8>>>,
    /// Input channel
    input_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    /// Window size
    window_size: Winsize,
    /// Working directory
    working_dir: PathBuf,
    /// Environment variables
    env_vars: HashMap<String, String>,
    /// Session status
    status: Arc<RwLock<SessionStatus>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    /// Session is created but not started
    Created,
    /// Session is running
    Running,
    /// Session is paused
    Paused,
    /// Session has stopped
    Stopped,
    /// Session encountered an error
    Error(String),
}

impl NativeSession {
    /// Create a new native session
    pub fn new(name: &str) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let window_size = Winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // Open PTY
        let OpenptyResult { master, slave } = openpty(Some(&window_size), None)?;

        // Close slave FD as we'll dup it in the child
        close(slave)?;

        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

        let session = Self {
            id: id.clone(),
            name: name.to_string(),
            pty_master: master,
            child: None,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            input_tx,
            window_size,
            working_dir: std::env::current_dir()?,
            env_vars: std::env::vars().collect(),
            status: Arc::new(RwLock::new(SessionStatus::Created)),
        };

        Ok(session)
    }

    /// Start the session with a shell
    pub async fn start(&mut self) -> Result<()> {
        self.start_with_command("/bin/bash").await
    }

    /// Start the session with a specific command
    pub async fn start_with_command(&mut self, command: &str) -> Result<()> {
        let OpenptyResult { master, slave } = openpty(Some(&self.window_size), None)?;

        self.pty_master = master;

        let mut cmd = Command::new(command);
        cmd.current_dir(&self.working_dir);

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Configure PTY for child process
        unsafe {
            cmd.pre_exec(move || {
                // Create new session
                nix::unistd::setsid()?;

                // Make PTY slave the controlling terminal
                nix::pty::unlockpt(master)?;
                let slave_path = nix::pty::ptsname_r(master)?;
                let slave_fd = nix::fcntl::open(
                    &slave_path,
                    nix::fcntl::OFlag::O_RDWR,
                    nix::sys::stat::Mode::empty(),
                )?;

                // Redirect stdin/stdout/stderr to PTY
                dup2(slave_fd, 0)?;
                dup2(slave_fd, 1)?;
                dup2(slave_fd, 2)?;

                // Close the slave FD
                close(slave_fd)?;
                close(slave)?;

                Ok(())
            });
        }

        let child = cmd.spawn().context("Failed to spawn child process")?;

        self.child = Some(child);

        // Update status
        *self.status.write().await = SessionStatus::Running;

        // Start output reader task
        let master_fd = self.pty_master;
        let output_buffer = self.output_buffer.clone();
        let status = self.status.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            let mut file = unsafe {
                use std::os::unix::io::FromRawFd;
                tokio::fs::File::from_raw_fd(master_fd)
            };

            loop {
                match file.read(&mut buffer).await {
                    Ok(0) => {
                        // EOF
                        *status.write().await = SessionStatus::Stopped;
                        break;
                    }
                    Ok(n) => {
                        let mut output = output_buffer.lock().await;
                        output.extend_from_slice(&buffer[..n]);

                        // Keep buffer size reasonable (1MB max)
                        if output.len() > 1_048_576 {
                            output.drain(..output.len() - 1_048_576);
                        }
                    }
                    Err(e) => {
                        log::error!("PTY read error: {}", e);
                        *status.write().await = SessionStatus::Error(e.to_string());
                        break;
                    }
                }
            }
        });

        // Start input writer task
        let master_fd = self.pty_master;
        let mut input_rx = tokio::sync::mpsc::channel::<Vec<u8>>(100).1;
        let input_tx = self.input_tx.clone();

        tokio::spawn(async move {
            let mut file = unsafe {
                use std::os::unix::io::FromRawFd;
                tokio::fs::File::from_raw_fd(master_fd)
            };

            while let Some(data) = input_rx.recv().await {
                if let Err(e) = file.write_all(&data).await {
                    log::error!("PTY write error: {}", e);
                    break;
                }
            }
        });

        close(slave)?;

        Ok(())
    }

    /// Send input to the session
    pub async fn send_input(&self, data: &str) -> Result<()> {
        self.input_tx
            .send(data.as_bytes().to_vec())
            .await
            .context("Failed to send input")?;
        Ok(())
    }

    /// Get recent output
    pub async fn get_output(&self, last_n_lines: usize) -> Result<Vec<String>> {
        let output = self.output_buffer.lock().await;
        let text = String::from_utf8_lossy(&output);

        let lines: Vec<String> = text
            .lines()
            .rev()
            .take(last_n_lines)
            .rev()
            .map(|s| s.to_string())
            .collect();

        Ok(lines)
    }

    /// Get all output
    pub async fn get_all_output(&self) -> Result<Vec<u8>> {
        let output = self.output_buffer.lock().await;
        Ok(output.clone())
    }

    /// Clear output buffer
    pub async fn clear_output(&self) -> Result<()> {
        let mut output = self.output_buffer.lock().await;
        output.clear();
        Ok(())
    }

    /// Resize the terminal
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let window_size = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // Window resize is not directly supported in nix 0.29, would need ioctl
        // For now, we'll skip this functionality
        let _ = window_size;
        Ok(())
    }

    /// Stop the session
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
        }

        *self.status.write().await = SessionStatus::Stopped;
        Ok(())
    }

    /// Get session status
    pub async fn get_status(&self) -> SessionStatus {
        self.status.read().await.clone()
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get session name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Native session manager that replaces tmux
pub struct NativeSessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<NativeSession>>>>>,
}

impl NativeSessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, name: &str) -> Result<Arc<Mutex<NativeSession>>> {
        let mut session = NativeSession::new(name)?;
        session.start().await?;

        let session = Arc::new(Mutex::new(session));
        let mut sessions = self.sessions.write().await;
        sessions.insert(name.to_string(), session.clone());

        Ok(session)
    }

    /// Get session by name
    pub async fn get_session(&self, name: &str) -> Option<Arc<Mutex<NativeSession>>> {
        let sessions = self.sessions.read().await;
        sessions.get(name).cloned()
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Delete session
    pub async fn delete_session(&self, name: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(name) {
            let mut session = session.lock().await;
            session.stop().await?;
        }
        Ok(())
    }

    /// Check if session exists
    pub async fn has_session(&self, name: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_native_session() -> Result<()> {
        let mut session = NativeSession::new("test")?;
        session.start().await?;

        // Send a command
        session.send_input("echo 'Hello Native Session'\n").await?;

        // Wait a bit for output
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get output
        let output = session.get_output(10).await?;
        assert!(!output.is_empty());

        // Check that our echo appears in output
        let full_output = String::from_utf8_lossy(&session.get_all_output().await?);
        assert!(full_output.contains("Hello Native Session"));

        session.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_session_manager() -> Result<()> {
        let manager = NativeSessionManager::new();

        // Create session
        let session = manager.create_session("test-session").await?;

        // Check it exists
        assert!(manager.has_session("test-session").await);

        // List sessions
        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert!(sessions.contains(&"test-session".to_string()));

        // Delete session
        manager.delete_session("test-session").await?;
        assert!(!manager.has_session("test-session").await);

        Ok(())
    }
}
