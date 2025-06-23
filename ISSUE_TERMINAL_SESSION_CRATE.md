# Create `terminal-session` Crate: A Rust Terminal Session Management Library

## Overview

Extract ccswarm's session management functionality into a standalone, reusable Rust crate called `terminal-session`. This crate will provide cross-platform terminal session management capabilities, initially supporting both native implementation and tmux backend, with a clear migration path away from tmux dependency.

## Motivation

### Why a Separate Crate?

1. **Reusability**: Other Rust projects can benefit from robust terminal session management
2. **Separation of Concerns**: Clean separation between AI orchestration (ccswarm) and terminal session management
3. **Independent Development**: Can evolve independently with its own release cycle
4. **Better Testing**: Easier to test in isolation without ccswarm's complexity
5. **Community Contribution**: Can attract contributors interested in terminal session management specifically
6. **Gradual Migration**: ccswarm can depend on both tmux and native backends during transition

## Crate Architecture

### Public API Design

```rust
// terminal-session/src/lib.rs
use async_trait::async_trait;

/// Main entry point for session management
pub struct SessionManager {
    backend: Box<dyn SessionBackend>,
    config: SessionConfig,
}

/// Configuration for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub backend_type: BackendType,
    pub default_shell: Option<PathBuf>,
    pub default_size: TerminalSize,
    pub env_vars: HashMap<String, String>,
    pub session_timeout: Duration,
    pub output_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendType {
    Native,
    Tmux,
    Auto, // Auto-detect best available
}

/// Core session trait that all backends must implement
#[async_trait]
pub trait SessionBackend: Send + Sync {
    async fn create_session(&self, config: &SessionCreateConfig) -> Result<Session>;
    async fn attach_session(&self, session_id: &str) -> Result<()>;
    async fn detach_session(&self, session_id: &str) -> Result<()>;
    async fn kill_session(&self, session_id: &str) -> Result<()>;
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>>;
    async fn send_command(&self, session_id: &str, command: &str) -> Result<()>;
    async fn send_keys(&self, session_id: &str, keys: &[Key]) -> Result<()>;
    async fn capture_output(&self, session_id: &str, lines: Option<usize>) -> Result<String>;
    async fn resize(&self, session_id: &str, size: TerminalSize) -> Result<()>;
    async fn get_status(&self, session_id: &str) -> Result<SessionStatus>;
}

/// Represents an active session
pub struct Session {
    pub id: String,
    pub name: String,
    pub backend: BackendType,
    pub pty: Option<PtyHandle>,
    pub process: Option<ProcessHandle>,
    pub created_at: DateTime<Utc>,
    pub working_directory: PathBuf,
}

/// Platform-agnostic PTY handling
pub struct PtyHandle {
    inner: Box<dyn Pty>,
}

#[async_trait]
pub trait Pty: Send + Sync {
    async fn read(&mut self) -> Result<Vec<u8>>;
    async fn write(&mut self, data: &[u8]) -> Result<()>;
    async fn resize(&mut self, size: TerminalSize) -> Result<()>;
}
```

### Module Structure

```
terminal-session/
├── Cargo.toml
├── README.md
├── LICENSE
├── examples/
│   ├── basic_session.rs
│   ├── tmux_migration.rs
│   └── interactive_shell.rs
├── src/
│   ├── lib.rs              # Public API
│   ├── error.rs            # Error types
│   ├── config.rs           # Configuration structures
│   ├── session.rs          # Core session management
│   ├── pty/
│   │   ├── mod.rs          # PTY abstraction
│   │   ├── unix.rs         # Unix PTY implementation
│   │   └── windows.rs      # Windows ConPTY implementation
│   ├── process/
│   │   ├── mod.rs          # Process management
│   │   ├── unix.rs         # Unix process handling
│   │   └── windows.rs      # Windows process handling
│   ├── backends/
│   │   ├── mod.rs          # Backend trait and registry
│   │   ├── native/
│   │   │   ├── mod.rs      # Native backend implementation
│   │   │   ├── session.rs  # Session management
│   │   │   └── executor.rs # Command execution
│   │   └── tmux/
│   │       ├── mod.rs      # Tmux backend implementation
│   │       ├── client.rs   # Tmux client (from ccswarm)
│   │       └── compat.rs   # Compatibility layer
│   ├── io/
│   │   ├── mod.rs          # I/O handling
│   │   ├── buffer.rs       # Ring buffer for output
│   │   └── stream.rs       # Async stream utilities
│   └── utils/
│       ├── mod.rs          # Utility functions
│       └── keys.rs         # Key sequence parsing
├── tests/
│   ├── integration/
│   │   ├── native_backend.rs
│   │   ├── tmux_backend.rs
│   │   └── cross_platform.rs
│   └── common/
│       └── mod.rs
└── benches/
    ├── throughput.rs
    └── latency.rs
```

### Key Features

#### 1. Backend Abstraction
```rust
// Easy backend switching
let manager = SessionManager::builder()
    .backend(BackendType::Auto)
    .build()?;

// Or explicit backend selection
let manager = SessionManager::builder()
    .backend(BackendType::Native)
    .build()?;
```

#### 2. Cross-Platform Support
```rust
// Automatic platform detection
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub fn create_pty() -> Result<Box<dyn Pty>> {
    #[cfg(unix)]
    return unix::UnixPty::new();
    
    #[cfg(windows)]
    return windows::ConPty::new();
}
```

#### 3. Async-First Design
```rust
// All operations are async
let session = manager.create_session(config).await?;
let output = session.capture_output(100).await?;

// Stream output in real-time
let mut stream = session.output_stream();
while let Some(data) = stream.next().await {
    println!("Output: {}", String::from_utf8_lossy(&data));
}
```

#### 4. Rich Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("PTY error: {0}")]
    PtyError(#[from] PtyError),
    
    #[error("Platform error: {0}")]
    PlatformError(String),
}
```

### Dependencies

```toml
[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "1.0"
anyhow = "1.0"

# Cross-platform PTY
portable-pty = "0.8"
crossterm = "0.27"

# Platform-specific
[target.'cfg(unix)'.dependencies]
nix = { version = "0.27", features = ["term", "process"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_System_Console"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
```

## Integration with ccswarm

### Phase 1: Create Crate and Basic Implementation
1. Set up `terminal-session` repository
2. Implement core abstractions and native backend
3. Move tmux client code from ccswarm
4. Publish initial version (0.1.0)

### Phase 2: Integrate with ccswarm
```toml
# ccswarm/Cargo.toml
[dependencies]
terminal-session = "0.1"
```

```rust
// ccswarm/src/session/mod.rs
use terminal_session::{SessionManager, BackendType, SessionConfig};

pub struct AgentSessionManager {
    session_manager: SessionManager,
    // ... ccswarm-specific fields
}

impl AgentSessionManager {
    pub async fn create_agent_session(&self, agent: &Agent) -> Result<AgentSession> {
        let session = self.session_manager
            .create_session(/* ... */)
            .await?;
        
        // Wrap in ccswarm-specific AgentSession
        Ok(AgentSession::from_terminal_session(session))
    }
}
```

### Phase 3: Migration Path
1. **v0.4.0**: Both backends available, tmux default
2. **v0.5.0**: Native backend default, tmux opt-in
3. **v0.6.0**: Deprecate tmux backend
4. **v0.7.0**: Remove tmux backend

## Examples

### Basic Usage
```rust
use terminal_session::{SessionManager, SessionConfig, BackendType};

#[tokio::main]
async fn main() -> Result<()> {
    // Create session manager
    let manager = SessionManager::builder()
        .backend(BackendType::Native)
        .build()?;
    
    // Create a session
    let session = manager.create_session(
        SessionConfig::default()
            .name("my-app")
            .working_directory("/tmp")
    ).await?;
    
    // Send commands
    session.send_command("echo 'Hello, World!'").await?;
    
    // Capture output
    let output = session.capture_output(None).await?;
    println!("Output: {}", output);
    
    // Clean up
    session.kill().await?;
    
    Ok(())
}
```

### Tmux Compatibility Mode
```rust
// For existing tmux users
let manager = SessionManager::builder()
    .backend(BackendType::Tmux)
    .tmux_compatible(true)  // Use tmux-style session names
    .build()?;

// Works with existing tmux sessions
let sessions = manager.list_sessions().await?;
for session in sessions {
    println!("Found tmux session: {}", session.name);
}
```

## Testing Strategy

### Unit Tests
- Test each backend independently
- Mock PTY operations
- Test error conditions

### Integration Tests
```rust
#[tokio::test]
async fn test_session_lifecycle() {
    let manager = create_test_manager();
    
    // Create session
    let session = manager.create_session(test_config()).await.unwrap();
    assert!(manager.list_sessions().await.unwrap().len() == 1);
    
    // Send command and verify output
    session.send_command("echo test").await.unwrap();
    let output = session.capture_output(None).await.unwrap();
    assert!(output.contains("test"));
    
    // Clean up
    session.kill().await.unwrap();
    assert!(manager.list_sessions().await.unwrap().is_empty());
}
```

### Platform-Specific Tests
```rust
#[cfg(unix)]
mod unix_tests {
    #[test]
    fn test_signal_handling() {
        // Test Unix signals
    }
}

#[cfg(windows)]
mod windows_tests {
    #[test]
    fn test_conpty() {
        // Test Windows ConPTY
    }
}
```

## Documentation Plan

1. **README.md**: Quick start guide and examples
2. **API Documentation**: Full rustdoc with examples
3. **Migration Guide**: For tmux users
4. **Architecture Guide**: For contributors
5. **Platform Notes**: OS-specific considerations

## Release Plan

### v0.1.0 (Initial Release)
- Core abstractions
- Native backend (Unix/Linux)
- Tmux backend
- Basic documentation

### v0.2.0
- Windows support
- Performance optimizations
- Extended examples

### v0.3.0
- Session persistence
- Advanced PTY features
- Benchmarks

### v1.0.0
- Stable API
- Production ready
- Complete documentation

## Community and Governance

1. **License**: MIT/Apache-2.0 (dual license)
2. **Contributing**: CONTRIBUTING.md with guidelines
3. **Code of Conduct**: Rust community CoC
4. **CI/CD**: GitHub Actions for testing
5. **Release Process**: Semantic versioning

## Risks and Mitigation

### Technical Risks
1. **Cross-platform complexity**: Mitigate with thorough testing
2. **Performance vs tmux**: Benchmark early and often
3. **API stability**: Mark experimental features clearly

### Adoption Risks
1. **User migration**: Provide clear migration guides
2. **Breaking changes**: Follow semantic versioning strictly
3. **Documentation**: Invest heavily in docs and examples

## Success Metrics

1. **Adoption**: Used by ccswarm and 5+ other projects
2. **Performance**: Equal or better than tmux
3. **Stability**: <0.1% crash rate in production
4. **Community**: 10+ contributors within 6 months

---

**Labels:** `enhancement`, `architecture`, `new-crate`

**Milestone:** terminal-session v0.1.0

**Related to:** #[original tmux removal issue]