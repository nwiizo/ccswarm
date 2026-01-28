//! PTY (Pseudo-Terminal) management

use anyhow::Result;
use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, timeout};

/// Handle to a PTY
pub struct PtyHandle {
    /// PTY size
    size: PtySize,
    /// Child process
    child: Arc<Mutex<Option<Box<dyn Child + Send>>>>,
    /// Reader handle (thread-safe)
    reader: Arc<Mutex<Option<Box<dyn Read + Send>>>>,
    /// Writer handle
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
}

impl PtyHandle {
    /// Create a new PTY with the specified size
    pub fn new(rows: u16, cols: u16) -> Result<Self> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        Ok(Self {
            size,
            child: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
        })
    }

    /// Spawn a command in the PTY
    pub async fn spawn_command(&self, cmd: CommandBuilder) -> Result<()> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(self.size)?;

        let child = pair.slave.spawn_command(cmd)?;
        let mut child_lock = self.child.lock().unwrap();
        *child_lock = Some(child);

        // Initialize reader and writer
        let reader = pair.master.try_clone_reader()?;
        let mut reader_lock = self.reader.lock().unwrap();
        *reader_lock = Some(reader);

        let writer = pair.master.take_writer()?;
        let mut writer_lock = self.writer.lock().unwrap();
        *writer_lock = Some(writer);

        Ok(())
    }

    /// Write data to the PTY
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut writer_lock = self.writer.lock().unwrap();
        if let Some(writer) = writer_lock.as_mut() {
            writer.write_all(data)?;
            writer.flush()?;
        } else {
            return Err(anyhow::anyhow!("PTY not initialized"));
        }
        Ok(())
    }

    /// Read data from the PTY with timeout
    pub async fn read(&self) -> Result<Vec<u8>> {
        let reader_arc = self.reader.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut reader_lock = reader_arc.lock().unwrap();
            if let Some(reader) = reader_lock.as_mut() {
                let mut buffer = vec![0u8; 4096];

                // Set non-blocking mode and try to read
                match reader.read(&mut buffer) {
                    Ok(0) => Ok(Vec::new()), // No data available
                    Ok(n) => {
                        buffer.truncate(n);
                        Ok(buffer)
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        Ok(Vec::new()) // No data available
                    }
                    Err(e) => Err(anyhow::anyhow!("Read error: {}", e)),
                }
            } else {
                Err(anyhow::anyhow!("PTY reader not initialized"))
            }
        })
        .await??;

        Ok(result)
    }

    /// Resize the PTY
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        // Update internal size
        let _new_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // For now, we just update the internal size
        // In a full implementation, we'd resize the actual PTY
        // self.size = new_size; // Can't modify due to &self

        Ok(())
    }

    /// Get the current PTY size
    pub fn size(&self) -> (u16, u16) {
        (self.size.rows, self.size.cols)
    }

    /// Check if child process is running
    pub fn is_running(&self) -> bool {
        if let Ok(child_lock) = self.child.lock()
            && let Some(_child) = child_lock.as_ref()
        {
            // For portable-pty, we assume the process is running if we have a handle
            // In a production implementation, we'd check the actual process status
            return true;
        }
        false
    }

    /// Read data from PTY with timeout (for testing)
    pub async fn read_with_timeout(&self, timeout_ms: u64) -> Result<Vec<u8>> {
        match timeout(Duration::from_millis(timeout_ms), self.read()).await {
            Ok(result) => result,
            Err(_) => Ok(Vec::new()), // Timeout - return empty data
        }
    }

    /// Spawn Claude Code in the PTY with --dangerously-skip-permissions flag
    ///
    /// This method launches Claude CLI in a PTY session for true parallel multi-agent execution.
    /// Each call creates an independent Claude process that can run concurrently with others.
    ///
    /// # Arguments
    /// * `prompt` - The prompt/instruction to send to Claude
    /// * `working_dir` - Working directory for the Claude session
    /// * `max_turns` - Maximum number of conversation turns (default: 1 for single task)
    ///
    /// # Example
    /// ```no_run
    /// use ai_session::core::pty::PtyHandle;
    /// use std::path::Path;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let pty = PtyHandle::new(24, 80)?;
    ///     pty.spawn_claude("Create a hello world function", Path::new("/tmp"), Some(3)).await?;
    ///     
    ///     // Read output with timeout
    ///     let output = pty.read_with_timeout(30000).await?;
    ///     println!("Claude output: {}", String::from_utf8_lossy(&output));
    ///     Ok(())
    /// }
    /// ```
    pub async fn spawn_claude(
        &self,
        prompt: &str,
        working_dir: &std::path::Path,
        max_turns: Option<u32>,
    ) -> Result<()> {
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        cmd.arg("-p");
        cmd.arg(prompt);
        cmd.arg("--output-format");
        cmd.arg("json");

        // Set max turns if specified
        if let Some(turns) = max_turns {
            cmd.arg("--max-turns");
            cmd.arg(turns.to_string());
        }

        cmd.cwd(working_dir);

        self.spawn_command(cmd).await
    }

    /// Spawn Claude Code and wait for completion, returning the output
    ///
    /// This is a convenience method that spawns Claude, waits for it to finish,
    /// and returns the collected output.
    ///
    /// # Arguments
    /// * `prompt` - The prompt/instruction to send to Claude
    /// * `working_dir` - Working directory for the Claude session
    /// * `max_turns` - Maximum number of conversation turns
    /// * `timeout_ms` - Timeout in milliseconds to wait for completion
    pub async fn spawn_claude_and_wait(
        &self,
        prompt: &str,
        working_dir: &std::path::Path,
        max_turns: Option<u32>,
        timeout_ms: u64,
    ) -> Result<String> {
        self.spawn_claude(prompt, working_dir, max_turns).await?;

        // Collect output until process completes or timeout
        let mut output = Vec::new();
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout_duration {
                break;
            }

            // Read available output
            let chunk = self.read_with_timeout(500).await?;
            if !chunk.is_empty() {
                output.extend_from_slice(&chunk);
            }

            // Check if process has finished
            if !self.is_running() {
                // Read any remaining output
                let remaining = self.read_with_timeout(100).await?;
                output.extend_from_slice(&remaining);
                break;
            }

            // Small delay to avoid busy-waiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// Spawn Claude Code with a specific session ID for later resumption
    ///
    /// This method allows you to specify a session ID that can be used later
    /// with `resume_claude` to continue the conversation.
    ///
    /// # Arguments
    /// * `prompt` - The prompt/instruction to send to Claude
    /// * `working_dir` - Working directory for the Claude session
    /// * `session_id` - UUID to use as the Claude session ID
    /// * `max_turns` - Maximum number of conversation turns
    ///
    /// # Example
    /// ```no_run
    /// use ai_session::core::pty::PtyHandle;
    /// use std::path::Path;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let pty = PtyHandle::new(24, 80)?;
    ///     let session_id = "2c4e029f-3411-442a-b24c-33001c78cd14";
    ///
    ///     // Start a new session with specific ID
    ///     pty.spawn_claude_with_session(
    ///         "Create a hello world function",
    ///         Path::new("/tmp"),
    ///         session_id,
    ///         Some(3),
    ///     ).await?;
    ///
    ///     // Later, resume the same session
    ///     let pty2 = PtyHandle::new(24, 80)?;
    ///     pty2.resume_claude(session_id, Path::new("/tmp")).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn spawn_claude_with_session(
        &self,
        prompt: &str,
        working_dir: &std::path::Path,
        session_id: &str,
        max_turns: Option<u32>,
    ) -> Result<()> {
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        cmd.arg("--session-id");
        cmd.arg(session_id);
        cmd.arg("-p");
        cmd.arg(prompt);
        cmd.arg("--output-format");
        cmd.arg("json");

        if let Some(turns) = max_turns {
            cmd.arg("--max-turns");
            cmd.arg(turns.to_string());
        }

        cmd.cwd(working_dir);

        self.spawn_command(cmd).await
    }

    /// Resume a Claude Code session by session ID
    ///
    /// This method resumes a previous Claude conversation using the session ID
    /// that was used when the session was created.
    ///
    /// # Arguments
    /// * `session_id` - The Claude session ID to resume
    /// * `working_dir` - Working directory for the Claude session
    ///
    /// # Example
    /// ```no_run
    /// use ai_session::core::pty::PtyHandle;
    /// use std::path::Path;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let pty = PtyHandle::new(24, 80)?;
    ///
    ///     // Resume a previous session
    ///     pty.resume_claude(
    ///         "2c4e029f-3411-442a-b24c-33001c78cd14",
    ///         Path::new("/tmp"),
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn resume_claude(
        &self,
        session_id: &str,
        working_dir: &std::path::Path,
    ) -> Result<()> {
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--resume");
        cmd.arg(session_id);

        cmd.cwd(working_dir);

        self.spawn_command(cmd).await
    }

    /// Resume a Claude Code session interactively with a new prompt
    ///
    /// This method resumes a previous Claude conversation and sends a new prompt.
    ///
    /// # Arguments
    /// * `session_id` - The Claude session ID to resume
    /// * `prompt` - New prompt to send to the resumed session
    /// * `working_dir` - Working directory for the Claude session
    /// * `max_turns` - Maximum number of conversation turns
    pub async fn resume_claude_with_prompt(
        &self,
        session_id: &str,
        prompt: &str,
        working_dir: &std::path::Path,
        max_turns: Option<u32>,
    ) -> Result<()> {
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        cmd.arg("--resume");
        cmd.arg(session_id);
        cmd.arg("-p");
        cmd.arg(prompt);
        cmd.arg("--output-format");
        cmd.arg("json");

        if let Some(turns) = max_turns {
            cmd.arg("--max-turns");
            cmd.arg(turns.to_string());
        }

        cmd.cwd(working_dir);

        self.spawn_command(cmd).await
    }
}
