use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Security module for agent isolation and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub permissions: HashSet<Permission>,
    pub isolation_level: IsolationLevel,
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    ReadFile,
    WriteFile,
    ExecuteCommand,
    NetworkAccess,
    SystemCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    Process,
    Container,
    VirtualMachine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub result: bool,
    pub details: String,
}

impl SecurityContext {
    pub fn new(isolation_level: IsolationLevel) -> Self {
        Self {
            permissions: HashSet::new(),
            isolation_level,
            audit_log: Vec::new(),
        }
    }

    pub fn grant_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
        self.log_audit(format!("Granted permission: {:?}", permission), true);
    }

    pub fn revoke_permission(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
        self.log_audit(format!("Revoked permission: {:?}", permission), true);
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    fn log_audit(&mut self, action: String, result: bool) {
        self.audit_log.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            action: action.clone(),
            result,
            details: String::new(),
        });
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new(IsolationLevel::Process)
    }
}