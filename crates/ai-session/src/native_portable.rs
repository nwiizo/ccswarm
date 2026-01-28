//! Native session management using portable-pty

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use uuid::Uuid;

/// Native session that replaces tmux functionality
pub struct NativeSession {
    /// Session ID
    id: String,
    /// Session name
    name: String,
    /// PTY master
    #[allow(dead_code)]
    pty_master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    /// Child process
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    /// Output buffer
    output_buffer: Arc<Mutex<Vec<u8>>>,
    /// Input channel
    input_tx: mpsc::Sender<Vec<u8>>,
    input_rx: Option<mpsc::Receiver<Vec<u8>>>,
    /// Window size
    window_size: PtySize,
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
        let window_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let (input_tx, input_rx) = mpsc::channel::<Vec<u8>>(100);

        let session = Self {
            id: id.clone(),
            name: name.to_string(),
            pty_master: None,
            child: None,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            input_tx,
            input_rx: Some(input_rx),
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
        // Create PTY system
        let pty_system = portable_pty::native_pty_system();

        // Create PTY pair
        let pty_pair = pty_system
            .openpty(self.window_size)
            .context("Failed to open PTY")?;

        // Create command
        let mut cmd = CommandBuilder::new(command);
        cmd.cwd(&self.working_dir);

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Spawn child process
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn child process")?;

        // Get reader and writer from the master
        let reader = pty_pair
            .master
            .try_clone_reader()
            .context("Failed to clone reader")?;
        let writer = pty_pair
            .master
            .take_writer()
            .context("Failed to take writer")?;

        // Store the child process
        self.child = Some(child);

        // Note: We can't store the master after taking the writer from it
        // The master is consumed by take_writer(), so we'll handle resize differently

        // Update status
        *self.status.write().await = SessionStatus::Running;

        // Start output reader task
        let output_buffer = self.output_buffer.clone();
        let status = self.status.clone();

        tokio::spawn(async move {
            use std::io::Read;
            let mut reader = reader;
            let mut buffer = vec![0u8; 4096];

            loop {
                match reader.read(&mut buffer) {
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
                            let drain_amount = output.len() - 1_048_576;
                            output.drain(..drain_amount);
                        }
                    }
                    Err(e) => {
                        tracing::error!("PTY read error: {}", e);
                        *status.write().await = SessionStatus::Error(e.to_string());
                        break;
                    }
                }
            }
        });

        // Start input writer task
        if let Some(mut input_rx) = self.input_rx.take() {
            tokio::spawn(async move {
                use std::io::Write;
                let mut writer = writer;

                while let Some(data) = input_rx.recv().await {
                    if let Err(e) = writer.write_all(&data) {
                        tracing::error!("PTY write error: {}", e);
                        break;
                    }
                    let _ = writer.flush();
                }
            });
        }

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

        let all_lines: Vec<&str> = text.lines().collect();
        let lines: Vec<String> = all_lines
            .iter()
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
    pub async fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        self.window_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Note: Since we don't store the pty_master after taking the writer,
        // we can't resize the PTY. This is a limitation of the current design.
        // In a production implementation, we would need to handle this differently.

        Ok(())
    }

    /// Stop the session
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
            let _ = child.wait();
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

impl Default for NativeSessionManager {
    fn default() -> Self {
        Self::new()
    }
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore] // Ignore this test as it requires a proper terminal environment
    async fn test_native_session() -> Result<()> {
        // This test is ignored by default because:
        // 1. It requires a proper terminal/PTY environment
        // 2. It can hang in CI environments without proper PTY support
        // 3. The portable-pty library has known issues with certain environments

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
        let output_bytes = session.get_all_output().await?;
        let full_output = String::from_utf8_lossy(&output_bytes);
        assert!(full_output.contains("Hello Native Session"));

        session.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_session_manager() -> Result<()> {
        let manager = NativeSessionManager::new();

        // Create session
        let _session = manager.create_session("test-session").await?;

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
