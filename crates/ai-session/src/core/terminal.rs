//! Terminal handle abstraction that can wrap either a PTY or the headless fallback.

use anyhow::Result;

use super::headless::HeadlessHandle;
use super::pty::PtyHandle;

/// Unified terminal handle used by `AISession`.
pub enum TerminalHandle {
    /// Native PTY backed terminal.
    Pty(PtyHandle),
    /// Headless (pipe) fallback terminal.
    Headless(HeadlessHandle),
}

impl TerminalHandle {
    /// Write data to the underlying terminal.
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        match self {
            TerminalHandle::Pty(handle) => handle.write(data).await,
            TerminalHandle::Headless(handle) => handle.write(data).await,
        }
    }

    /// Read data from the underlying terminal.
    pub async fn read(&self) -> Result<Vec<u8>> {
        match self {
            TerminalHandle::Pty(handle) => handle.read().await,
            TerminalHandle::Headless(handle) => handle.read().await,
        }
    }

    /// Read data with a timeout from the underlying terminal.
    pub async fn read_with_timeout(&self, timeout_ms: u64) -> Result<Vec<u8>> {
        match self {
            TerminalHandle::Pty(handle) => handle.read_with_timeout(timeout_ms).await,
            TerminalHandle::Headless(handle) => handle.read_with_timeout(timeout_ms).await,
        }
    }

    /// Check whether the terminal is still running.
    pub async fn is_running(&self) -> bool {
        match self {
            TerminalHandle::Pty(handle) => handle.is_running(),
            TerminalHandle::Headless(handle) => handle.is_running().await,
        }
    }

    /// Shutdown the terminal, freeing all resources.
    pub async fn shutdown(self) -> Result<()> {
        match self {
            TerminalHandle::Pty(_handle) => Ok(()),
            TerminalHandle::Headless(handle) => handle.shutdown().await,
        }
    }
}
