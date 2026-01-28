//! Security and isolation features for AI sessions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Secure session with isolation and access control
pub struct SecureSession {
    /// Namespace isolation
    pub namespace: Namespace,
    /// Resource limits
    pub cgroups: CGroupLimits,
    /// Security policy
    pub security_policy: SecurityPolicy,
    /// Audit log
    pub audit_log: AuditLog,
}

impl SecureSession {
    /// Create a new secure session
    pub fn new(session_id: &str) -> Self {
        Self {
            namespace: Namespace::new(session_id),
            cgroups: CGroupLimits::default(),
            security_policy: SecurityPolicy::default(),
            audit_log: AuditLog::new(),
        }
    }

    /// Apply security policy
    pub fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        self.security_policy = policy;
        self.audit_log.log(AuditEvent::PolicyApplied {
            policy_name: "custom".to_string(),
        })?;
        Ok(())
    }

    /// Check if an action is allowed
    pub fn is_allowed(&self, action: &Action) -> bool {
        match action {
            Action::FileAccess { path, mode } => {
                self.security_policy.fs_permissions.is_allowed(path, mode)
            }
            Action::NetworkAccess { host, port } => {
                self.security_policy.network_policy.is_allowed(host, *port)
            }
            Action::SystemCall { syscall } => {
                self.security_policy.syscall_access.is_allowed(syscall)
            }
            Action::APICall { endpoint, method } => {
                self.security_policy.api_limits.is_allowed(endpoint, method)
            }
        }
    }

    /// Audit an action
    pub fn audit_action(&mut self, action: Action, allowed: bool) -> Result<()> {
        self.audit_log.log(AuditEvent::ActionAttempted {
            action,
            allowed,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Namespace isolation
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Namespace ID
    pub id: String,
    /// PID namespace
    pub pid_ns: bool,
    /// Network namespace
    pub net_ns: bool,
    /// Mount namespace
    pub mnt_ns: bool,
    /// User namespace
    pub user_ns: bool,
}

impl Namespace {
    /// Create a new namespace
    pub fn new(id: &str) -> Self {
        Self {
            id: format!("ai-session-{}", id),
            pid_ns: true,
            net_ns: true,
            mnt_ns: true,
            user_ns: true,
        }
    }
}

/// Resource limits using cgroups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGroupLimits {
    /// CPU limit (percentage)
    pub cpu_limit: f64,
    /// Memory limit (bytes)
    pub memory_limit: usize,
    /// Disk I/O limit (bytes/sec)
    pub io_limit: usize,
    /// Maximum processes
    pub pids_limit: usize,
}

impl Default for CGroupLimits {
    fn default() -> Self {
        Self {
            cpu_limit: 50.0,       // 50% CPU
            memory_limit: 1 << 30, // 1GB
            io_limit: 100 << 20,   // 100MB/s
            pids_limit: 100,       // 100 processes
        }
    }
}

/// Security policy
#[derive(Debug, Clone, Default)]
pub struct SecurityPolicy {
    /// File system access control
    pub fs_permissions: FileSystemPermissions,
    /// Network access control
    pub network_policy: NetworkPolicy,
    /// System call access
    pub syscall_access: SyscallAccess,
    /// API call limits
    pub api_limits: APILimits,
    /// Secret manager
    pub secret_manager: SecretManager,
}

/// File system permissions
#[derive(Debug, Clone, Default)]
pub struct FileSystemPermissions {
    /// Allowed paths
    allowed_paths: Vec<PathBuf>,
    /// Denied paths
    denied_paths: Vec<PathBuf>,
    /// Read-only paths
    readonly_paths: Vec<PathBuf>,
}

impl FileSystemPermissions {
    /// Check if file access is allowed
    pub fn is_allowed(&self, path: &Path, mode: &FileAccessMode) -> bool {
        // Check denied paths first
        for denied in &self.denied_paths {
            if path.starts_with(denied) {
                return false;
            }
        }

        // Check if in allowed paths
        let in_allowed = self
            .allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed));
        if !in_allowed && !self.allowed_paths.is_empty() {
            return false;
        }

        // Check read-only restrictions
        if matches!(mode, FileAccessMode::Write | FileAccessMode::Execute) {
            for readonly in &self.readonly_paths {
                if path.starts_with(readonly) {
                    return false;
                }
            }
        }

        true
    }

    /// Add an allowed path
    pub fn allow_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    /// Add a denied path
    pub fn deny_path(&mut self, path: PathBuf) {
        self.denied_paths.push(path);
    }

    /// Add a read-only path
    pub fn readonly_path(&mut self, path: PathBuf) {
        self.readonly_paths.push(path);
    }
}

/// File access mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileAccessMode {
    Read,
    Write,
    Execute,
}

/// Network access policy
#[derive(Debug, Clone, Default)]
pub struct NetworkPolicy {
    /// Allowed hosts
    allowed_hosts: Vec<String>,
    /// Blocked hosts
    blocked_hosts: Vec<String>,
    /// Allowed ports
    allowed_ports: Vec<u16>,
    /// Rate limits per host
    rate_limits: HashMap<String, RateLimit>,
}

impl NetworkPolicy {
    /// Check if network access is allowed
    pub fn is_allowed(&self, host: &str, port: u16) -> bool {
        // Check blocked hosts
        if self.blocked_hosts.iter().any(|h| h == host) {
            return false;
        }

        // Check allowed hosts
        if !self.allowed_hosts.is_empty() && !self.allowed_hosts.iter().any(|h| h == host) {
            return false;
        }

        // Check allowed ports
        if !self.allowed_ports.is_empty() && !self.allowed_ports.contains(&port) {
            return false;
        }

        // Check rate limits
        if let Some(limit) = self.rate_limits.get(host) {
            return limit.check();
        }

        true
    }
}

/// System call access control
#[derive(Debug, Clone, Default)]
pub struct SyscallAccess {
    /// Allowed syscalls
    allowed_syscalls: Vec<String>,
    /// Blocked syscalls
    blocked_syscalls: Vec<String>,
}

impl SyscallAccess {
    /// Check if syscall is allowed
    pub fn is_allowed(&self, syscall: &str) -> bool {
        if self.blocked_syscalls.contains(&syscall.to_string()) {
            return false;
        }

        if !self.allowed_syscalls.is_empty() {
            return self.allowed_syscalls.contains(&syscall.to_string());
        }

        true
    }
}

/// API call limits
#[derive(Debug, Clone, Default)]
pub struct APILimits {
    /// Rate limits per endpoint
    endpoint_limits: HashMap<String, RateLimit>,
    /// Global rate limit
    global_limit: Option<RateLimit>,
    /// Token limits
    _token_limits: TokenLimits,
}

impl APILimits {
    /// Check if API call is allowed
    pub fn is_allowed(&self, endpoint: &str, _method: &str) -> bool {
        // Check endpoint-specific limit
        if let Some(limit) = self.endpoint_limits.get(endpoint)
            && !limit.check()
        {
            return false;
        }

        // Check global limit
        if let Some(ref limit) = self.global_limit
            && !limit.check()
        {
            return false;
        }

        true
    }
}

/// Token usage limits
#[derive(Debug, Clone, Default)]
pub struct TokenLimits {
    /// Maximum tokens per request
    pub max_tokens_per_request: usize,
    /// Maximum tokens per hour
    pub max_tokens_per_hour: usize,
    /// Maximum tokens per day
    pub max_tokens_per_day: usize,
}

/// Rate limiting
#[derive(Debug)]
pub struct RateLimit {
    /// Requests per minute
    pub requests_per_minute: usize,
    /// Current minute
    _current_minute: std::time::Instant,
    /// Request count
    request_count: std::sync::Mutex<usize>,
}

impl RateLimit {
    /// Create a new rate limit
    pub fn new(requests_per_minute: usize) -> Self {
        Self {
            requests_per_minute,
            _current_minute: std::time::Instant::now(),
            request_count: std::sync::Mutex::new(0),
        }
    }

    /// Check if request is allowed and increment counter
    pub fn check(&self) -> bool {
        let mut count = self.request_count.lock().unwrap();
        if *count < self.requests_per_minute {
            *count += 1;
            true
        } else {
            false
        }
    }
}

impl Clone for RateLimit {
    fn clone(&self) -> Self {
        let count = *self.request_count.lock().unwrap();
        Self {
            requests_per_minute: self.requests_per_minute,
            _current_minute: self._current_minute,
            request_count: std::sync::Mutex::new(count),
        }
    }
}

/// Secret manager
#[derive(Debug, Clone, Default)]
pub struct SecretManager {
    /// Encrypted secrets
    secrets: HashMap<String, Vec<u8>>,
}

impl SecretManager {
    /// Store a secret
    pub fn store_secret(&mut self, key: &str, value: &[u8]) -> Result<()> {
        // In a real implementation, this would encrypt the value
        self.secrets.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    /// Retrieve a secret
    pub fn get_secret(&self, key: &str) -> Option<Vec<u8>> {
        self.secrets.get(key).cloned()
    }
}

/// Audit log
pub struct AuditLog {
    /// Log entries
    entries: Vec<AuditEvent>,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Log an event
    pub fn log(&mut self, event: AuditEvent) -> Result<()> {
        self.entries.push(event);
        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditEvent] {
        &self.entries
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Session created
    SessionCreated {
        session_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Policy applied
    PolicyApplied { policy_name: String },
    /// Action attempted
    ActionAttempted {
        action: Action,
        allowed: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Security violation
    SecurityViolation {
        description: String,
        severity: Severity,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Security action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// File system access
    FileAccess { path: PathBuf, mode: FileAccessMode },
    /// Network access
    NetworkAccess { host: String, port: u16 },
    /// System call
    SystemCall { syscall: String },
    /// API call
    APICall { endpoint: String, method: String },
}

/// Security severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Capability-based security
pub trait Capabilities {
    /// Request a capability
    fn request_capability(&self, capability: Capability) -> Result<CapabilityToken>;

    /// Check if capability is granted
    fn has_capability(&self, capability: &Capability) -> bool;

    /// Revoke a capability
    fn revoke_capability(&mut self, token: CapabilityToken) -> Result<()>;
}

/// Security capability
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Capability {
    FileRead(PathBuf),
    FileWrite(PathBuf),
    NetworkAccess(String, u16),
    ProcessSpawn(String),
    SystemCall(String),
}

/// Capability token
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CapabilityToken {
    /// Token ID
    pub id: uuid::Uuid,
    /// Capability
    pub capability: Capability,
    /// Expiration
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_permissions() {
        let mut perms = FileSystemPermissions::default();
        perms.allow_path(PathBuf::from("/tmp"));
        perms.deny_path(PathBuf::from("/tmp/secret"));
        perms.readonly_path(PathBuf::from("/tmp/readonly"));

        assert!(perms.is_allowed(&PathBuf::from("/tmp/test.txt"), &FileAccessMode::Read));
        assert!(!perms.is_allowed(&PathBuf::from("/tmp/secret/key.txt"), &FileAccessMode::Read));
        assert!(!perms.is_allowed(
            &PathBuf::from("/tmp/readonly/file.txt"),
            &FileAccessMode::Write
        ));
    }

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new();

        log.log(AuditEvent::SessionCreated {
            session_id: "test-123".to_string(),
            timestamp: chrono::Utc::now(),
        })
        .unwrap();

        assert_eq!(log.entries().len(), 1);
    }
}
