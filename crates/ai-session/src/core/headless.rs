//! Headless (non-PTY) terminal management used when PTYs are unavailable.
//!
//! This module provides a lightweight fallback that relies on piped stdin/stdout
//! instead of allocating a real PTY. It allows the ai-session crate to run inside
//! restricted environments (CI sandboxes, containers without `openpty`, etc.)
//! where creating a pseudo-terminal would otherwise fail with `EPERM`.

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

/// Shared buffer for aggregated stdout/stderr output.
type OutputBuffer = Arc<Mutex<Vec<u8>>>;

/// Headless terminal handle that mimics the PTY interface with buffered IO.
pub struct HeadlessHandle {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    output: OutputBuffer,
    child: Arc<Mutex<Option<Child>>>,
}

impl HeadlessHandle {
    /// Spawn an interactive shell that communicates via pipes.
    pub async fn spawn_shell<'a>(
        shell: &str,
        working_dir: &Path,
        env: impl IntoIterator<Item = (&'a String, &'a String)>,
    ) -> Result<Self> {
        let mut command = Command::new(shell);
        command
            .current_dir(working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (key, value) in env {
            command.env(key, value);
        }

        let mut child = command.spawn().context("Failed to spawn headless shell")?;

        let stdin = child
            .stdin
            .take()
            .context("Missing stdin for headless shell")?;
        let stdout = child
            .stdout
            .take()
            .context("Missing stdout for headless shell")?;
        let stderr = child
            .stderr
            .take()
            .context("Missing stderr for headless shell")?;

        let output = Arc::new(Mutex::new(Vec::new()));
        let handle = Self {
            stdin: Arc::new(Mutex::new(Some(stdin))),
            output: output.clone(),
            child: Arc::new(Mutex::new(Some(child))),
        };

        spawn_output_task(stdout, output.clone());
        spawn_output_task(stderr, output);

        Ok(handle)
    }

    /// Write data to the shell stdin.
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            stdin.write_all(data).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Headless shell stdin unavailable"))
        }
    }

    /// Drain buffered output from stdout/stderr.
    pub async fn read(&self) -> Result<Vec<u8>> {
        let mut buffer = self.output.lock().await;
        if buffer.is_empty() {
            return Ok(Vec::new());
        }
        let data = buffer.clone();
        buffer.clear();
        Ok(data)
    }

    /// Read buffered output with a timeout.
    pub async fn read_with_timeout(&self, timeout_ms: u64) -> Result<Vec<u8>> {
        match timeout(Duration::from_millis(timeout_ms), self.read()).await {
            Ok(result) => result,
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Check whether the underlying process is still running.
    pub async fn is_running(&self) -> bool {
        let mut guard = self.child.lock().await;
        if let Some(child) = guard.as_mut() {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Terminate the headless shell if it is still running.
    pub async fn shutdown(self) -> Result<()> {
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

fn spawn_output_task<R>(mut reader: R, output: OutputBuffer)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = vec![0u8; 4096];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut out = output.lock().await;
                    out.extend_from_slice(&buffer[..n]);
                    // cap at ~1MB to avoid unbounded growth
                    if out.len() > 1_048_576 {
                        let drain = out.len() - 1_048_576;
                        out.drain(..drain);
                    }
                }
                Err(err) => {
                    tracing::debug!("Headless shell read error: {}", err);
                    break;
                }
            }
        }
    });
}
