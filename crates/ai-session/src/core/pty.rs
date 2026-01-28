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
}
