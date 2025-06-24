# Implement Internal Session Management to Replace tmux Dependency

## Overview

ccswarm currently relies heavily on tmux for session management, which creates portability issues and adds an external dependency that may not be available in all environments (e.g., Windows, containerized deployments, CI/CD pipelines). This issue proposes implementing an internal session management system that provides the same functionality while being fully integrated into ccswarm.

## Motivation

### Why Remove tmux Dependency?

1. **Portability**: tmux is not available on Windows without WSL, limiting ccswarm's cross-platform compatibility
2. **Better Integration**: Direct control over session lifecycle without shell command overhead
3. **Reduced External Dependencies**: One less system dependency to install and manage
4. **Enhanced Features**: Ability to add ccswarm-specific session features without tmux limitations
5. **Improved Testing**: Easier to mock and test session behavior in unit tests
6. **Container Friendliness**: Simplified deployment in Docker/Kubernetes environments

## Current tmux Usage Analysis

Based on code analysis, ccswarm uses tmux for the following key features:

### 1. Session Management
```rust
// From src/tmux/mod.rs
pub fn create_session(&self, session_name: &str, working_directory: &str)
pub fn kill_session(&self, session_name: &str)
pub fn attach_session(&self, session_name: &str)
pub fn detach_session(&self, session_name: &str)
pub fn list_sessions(&self) -> Vec<TmuxSession>
```

### 2. Command Execution
```rust
pub fn send_keys(&self, session_name: &str, keys: &str)
pub fn send_command(&self, session_name: &str, command: &str)
```

### 3. Output Capture
```rust
pub fn capture_pane(&self, session_name: &str, pane_id: Option<&str>) -> String
```

### 4. Window/Pane Management
```rust
pub fn new_window(&self, session_name: &str, window_name: &str, working_directory: Option<&str>)
pub fn list_windows(&self, session_name: &str) -> Vec<TmuxWindow>
```

### 5. Environment & Configuration
```rust
pub fn set_environment(&self, session_name: &str, name: &str, value: &str)
pub fn set_option(&self, session_name: &str, option: &str, value: &str)
```

## Proposed Architecture

### Core Components

```rust
// src/session/internal/mod.rs
pub struct InternalSessionManager {
    sessions: Arc<RwLock<HashMap<String, InternalSession>>>,
    executor: Arc<dyn SessionExecutor>,
    config: SessionConfig,
}

pub struct InternalSession {
    id: String,
    name: String,
    process: Option<Child>,
    pty: PseudoTerminal,
    output_buffer: RingBuffer<String>,
    environment: HashMap<String, String>,
    working_directory: PathBuf,
    status: SessionStatus,
    created_at: DateTime<Utc>,
}

// Platform-specific implementations
trait SessionExecutor {
    fn create_session(&self, config: &SessionConfig) -> Result<InternalSession>;
    fn execute_command(&self, session: &mut InternalSession, cmd: &str) -> Result<()>;
    fn capture_output(&self, session: &InternalSession) -> Result<String>;
    fn attach_terminal(&self, session: &mut InternalSession) -> Result<()>;
}
```

### Key Features to Implement

1. **Pseudo-Terminal (PTY) Support**
   - Use `portable-pty` crate for cross-platform PTY handling
   - Manage input/output streams for interactive sessions

2. **Process Management**
   - Use `tokio::process` for async process spawning
   - Implement proper signal handling (SIGTERM, SIGINT)
   - Support for background processes

3. **Output Buffering**
   - Ring buffer for storing recent output (configurable size)
   - Streaming output capture for real-time monitoring
   - Search capabilities within buffer

4. **Session Persistence**
   - Serialize session state to disk
   - Restore sessions after ccswarm restart
   - Session migration between hosts

5. **Multiplexing**
   - Multiple virtual "windows" per session
   - Split pane functionality through output routing
   - Focus management for input routing

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Create `InternalSessionManager` structure
- [ ] Implement basic process spawning with PTY
- [ ] Add output capture and buffering
- [ ] Create platform-specific executors (Unix/Windows)

### Phase 2: Feature Parity (Week 3-4)
- [ ] Implement all tmux commands used by ccswarm
- [ ] Add session listing and management
- [ ] Environment variable handling
- [ ] Working directory management

### Phase 3: Migration Layer (Week 5)
- [ ] Create `TmuxCompatibilityLayer` that wraps internal sessions
- [ ] Implement fallback to tmux if available
- [ ] Add migration commands for existing tmux sessions

### Phase 4: Advanced Features (Week 6-7)
- [ ] Session persistence and restoration
- [ ] Enhanced output search and filtering
- [ ] Performance optimizations
- [ ] Comprehensive testing suite

### Phase 5: Integration (Week 8)
- [ ] Update all ccswarm components to use new session manager
- [ ] Update documentation
- [ ] Deprecation notices for tmux-specific features

## Backward Compatibility Strategy

1. **Feature Detection**
   ```rust
   pub enum SessionBackend {
       Internal(InternalSessionManager),
       Tmux(TmuxClient),
   }
   
   impl SessionBackend {
       pub fn auto_detect() -> Self {
           if std::env::var("CCSWARM_USE_TMUX").is_ok() {
               SessionBackend::Tmux(TmuxClient::new().expect("tmux required"))
           } else {
               SessionBackend::Internal(InternalSessionManager::new())
           }
       }
   }
   ```

2. **Migration Path**
   - v0.4.0: Internal sessions as opt-in feature
   - v0.5.0: Internal sessions as default, tmux as fallback
   - v0.6.0: Deprecate tmux support
   - v0.7.0: Remove tmux dependency

3. **Configuration**
   ```json
   {
     "session_backend": "internal", // or "tmux" or "auto"
     "internal_session_config": {
       "pty_size": { "rows": 24, "cols": 80 },
       "output_buffer_size": 10000,
       "persist_sessions": true
     }
   }
   ```

## Testing Strategy

1. **Unit Tests**
   - Test each session operation independently
   - Mock PTY interactions
   - Test error handling and edge cases

2. **Integration Tests**
   - Full session lifecycle tests
   - Multi-session coordination
   - Platform-specific behavior

3. **Compatibility Tests**
   - Ensure feature parity with tmux implementation
   - Test migration scenarios
   - Performance benchmarks

## Dependencies

### Required Crates
- `portable-pty`: Cross-platform PTY support
- `crossterm`: Terminal manipulation
- `nix` (Unix) / `winapi` (Windows): Platform-specific features
- `bytes`: Efficient buffer management
- `futures`: Async stream handling

### Optional Crates
- `ratatui`: For session UI (already in use)
- `notify`: File system watching for session persistence

## Acceptance Criteria

- [ ] All current tmux-dependent features work with internal sessions
- [ ] Cross-platform support (Linux, macOS, Windows)
- [ ] Performance equal or better than tmux implementation
- [ ] Comprehensive test coverage (>80%)
- [ ] Documentation for migration and new features
- [ ] No regression in existing functionality

## Risks and Mitigation

1. **Platform Differences**
   - Risk: PTY behavior varies across platforms
   - Mitigation: Extensive platform-specific testing, use proven libraries

2. **Performance**
   - Risk: Internal implementation slower than tmux
   - Mitigation: Benchmark early, optimize critical paths, use efficient data structures

3. **Feature Completeness**
   - Risk: Missing subtle tmux features that some users depend on
   - Mitigation: Comprehensive feature audit, beta testing period

## Future Enhancements

Once the internal session management is implemented, we can add:
- Web-based session viewer
- Session recording and playback
- Distributed session management
- Enhanced security features (session encryption)
- Integration with container orchestration

## References

- [portable-pty documentation](https://docs.rs/portable-pty/)
- [tmux source code](https://github.com/tmux/tmux) for implementation details
- [Microsoft's ConPTY](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/) for Windows considerations

---

**Labels:** `enhancement`, `breaking-change`, `architecture`

**Milestone:** v0.4.0

**Assignees:** TBD

**Estimated Effort:** 8 weeks (1-2 developers)