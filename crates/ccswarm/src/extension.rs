//! Extension system with type-safe plugin support
//!
//! This module provides a comprehensive extension system that allows
//! for safe, versioned plugin management with proper lifecycle handling.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{CCSwarmError, Result};
use crate::traits::{Configurable, Identifiable, Monitorable, Stateful, Validatable};

/// Semantic version for extensions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Create a version with pre-release identifier
    pub fn new_prerelease(major: u32, minor: u32, patch: u32, pre_release: String) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        }
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        // Compatible if major versions match and this version is >= other
        self.major == other.major && self >= other
    }

    /// Check if this is a stable release
    pub fn is_stable(&self) -> bool {
        self.pre_release.is_none()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref pre) = self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

impl FromStr for Version {
    type Err = CCSwarmError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        let version_part = parts[0];
        let pre_release = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };

        let version_nums: Vec<&str> = version_part.split('.').collect();
        if version_nums.len() != 3 {
            return Err(CCSwarmError::config(format!(
                "Invalid version format: {}",
                s
            )));
        }

        let major = version_nums[0].parse::<u32>().map_err(|_| {
            CCSwarmError::config(format!("Invalid major version: {}", version_nums[0]))
        })?;
        let minor = version_nums[1].parse::<u32>().map_err(|_| {
            CCSwarmError::config(format!("Invalid minor version: {}", version_nums[1]))
        })?;
        let patch = version_nums[2].parse::<u32>().map_err(|_| {
            CCSwarmError::config(format!("Invalid patch version: {}", version_nums[2]))
        })?;

        Ok(Version {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

/// Extension state in its lifecycle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtensionState {
    /// Extension is installed but not loaded
    Installed,
    /// Extension is loaded and ready to use
    Loaded,
    /// Extension is currently active
    Active,
    /// Extension is paused
    Paused,
    /// Extension has encountered an error
    Error(String),
    /// Extension is being uninstalled
    Uninstalling,
}

/// Extension capability types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtensionCapability {
    /// Agent behavior modification
    AgentEnhancement,
    /// New command addition
    Command,
    /// Data processing pipeline
    Pipeline,
    /// UI/UX enhancement
    Interface,
    /// Integration with external services
    Integration,
    /// Monitoring and observability
    Monitoring,
    /// Security enhancement
    Security,
    /// Performance optimization
    Performance,
}

/// Extension configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Whether the extension should auto-start
    #[serde(default)]
    pub auto_start: bool,
    /// Extension-specific configuration
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
    /// Required permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Resource limits
    #[serde(default)]
    pub resource_limits: Option<ResourceLimits>,
}

/// Resource limits for extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: Option<f32>,
    /// Maximum number of threads
    pub max_threads: Option<u32>,
    /// Maximum disk space in bytes
    pub max_disk_space: Option<u64>,
}

/// Extension health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub uptime: std::time::Duration,
    pub error_count: u32,
    pub performance_score: f32,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Unknown,
}

/// Extension metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub requests_handled: u64,
    pub errors_count: u32,
    pub average_response_time: std::time::Duration,
    pub uptime: std::time::Duration,
}

/// Type-safe extension with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    /// Unique extension identifier
    pub id: String,
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: Version,
    /// Extension description
    pub description: String,
    /// Extension author/organization
    pub author: String,
    /// Extension capabilities
    pub capabilities: Vec<ExtensionCapability>,
    /// Dependencies on other extensions
    pub dependencies: Vec<ExtensionDependency>,
    /// Extension configuration
    pub config: ExtensionConfig,
    /// Current state
    pub state: ExtensionState,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Extension metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Extension entry point (for dynamic loading)
    pub entry_point: Option<String>,
    /// Manifest file path
    pub manifest_path: Option<std::path::PathBuf>,
}

/// Extension dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDependency {
    pub name: String,
    pub version_requirement: VersionRequirement,
    pub optional: bool,
}

/// Version requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionRequirement {
    Exact(Version),
    GreaterThan(Version),
    GreaterThanOrEqual(Version),
    LessThan(Version),
    LessThanOrEqual(Version),
    Compatible(Version), // Same major, >= version
    Range { min: Version, max: Version },
}

impl VersionRequirement {
    /// Check if a version satisfies this requirement
    pub fn satisfies(&self, version: &Version) -> bool {
        match self {
            Self::Exact(v) => version == v,
            Self::GreaterThan(v) => version > v,
            Self::GreaterThanOrEqual(v) => version >= v,
            Self::LessThan(v) => version < v,
            Self::LessThanOrEqual(v) => version <= v,
            Self::Compatible(v) => version.is_compatible_with(v),
            Self::Range { min, max } => version >= min && version <= max,
        }
    }
}

impl Extension {
    /// Create a new extension
    pub fn new<S: Into<String>>(
        name: S,
        version: Version,
        description: S,
        author: S,
        capabilities: Vec<ExtensionCapability>,
    ) -> Self {
        let name = name.into();
        let now = chrono::Utc::now();

        Self {
            id: format!(
                "{}-{}",
                name.to_lowercase().replace(' ', "-"),
                Uuid::new_v4()
            ),
            name,
            version,
            description: description.into(),
            author: author.into(),
            capabilities,
            dependencies: Vec::new(),
            config: ExtensionConfig::default(),
            state: ExtensionState::Installed,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            entry_point: None,
            manifest_path: None,
        }
    }

    /// Add a dependency
    pub fn add_dependency(
        &mut self,
        name: String,
        version_requirement: VersionRequirement,
        optional: bool,
    ) {
        self.dependencies.push(ExtensionDependency {
            name,
            version_requirement,
            optional,
        });
        self.touch();
    }

    /// Add metadata
    pub fn add_metadata<K: Into<String>>(&mut self, key: K, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.touch();
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ExtensionConfig) {
        self.config = config;
        self.touch();
    }

    /// Check if dependencies are satisfied by available extensions
    pub fn check_dependencies(&self, available_extensions: &[Extension]) -> Result<Vec<String>> {
        let mut missing_deps = Vec::new();

        for dep in &self.dependencies {
            if dep.optional {
                continue;
            }

            let satisfied = available_extensions
                .iter()
                .any(|ext| ext.name == dep.name && dep.version_requirement.satisfies(&ext.version));

            if !satisfied {
                missing_deps.push(dep.name.clone());
            }
        }

        if missing_deps.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(missing_deps)
        }
    }

    /// Update the last modified timestamp
    fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Check if extension can be activated
    pub fn can_activate(&self, available_extensions: &[Extension]) -> Result<bool> {
        if !matches!(self.state, ExtensionState::Loaded) {
            return Ok(false);
        }

        let missing_deps = self.check_dependencies(available_extensions)?;
        Ok(missing_deps.is_empty())
    }

    /// Get required permissions
    pub fn required_permissions(&self) -> &[String] {
        &self.config.permissions
    }

    /// Check if extension has specific capability
    pub fn has_capability(&self, capability: &ExtensionCapability) -> bool {
        self.capabilities.contains(capability)
    }
}

/// Implement common traits for Extension
impl Identifiable for Extension {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Stateful for Extension {
    type State = ExtensionState;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn is_operational(&self) -> bool {
        matches!(self.state, ExtensionState::Active | ExtensionState::Loaded)
    }
}

impl Configurable for Extension {
    type Config = ExtensionConfig;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn update_config(&mut self, config: Self::Config) -> Result<()> {
        self.config = config;
        self.touch();
        Ok(())
    }

    fn validate_config(config: &Self::Config) -> Result<()> {
        // Validate permissions are not excessive
        if config.permissions.len() > 10 {
            return Err(CCSwarmError::config("Too many permissions requested"));
        }

        // Validate resource limits are reasonable
        if let Some(ref limits) = config.resource_limits {
            if let Some(max_memory) = limits.max_memory
                && max_memory > 1024 * 1024 * 1024
            {
                // 1GB
                return Err(CCSwarmError::config("Memory limit too high"));
            }
            if let Some(max_cpu) = limits.max_cpu_percent
                && max_cpu > 100.0
            {
                return Err(CCSwarmError::config("CPU limit cannot exceed 100%"));
            }
        }

        Ok(())
    }

    fn default_config() -> Self::Config {
        ExtensionConfig::default()
    }
}

#[async_trait]
impl Monitorable for Extension {
    type HealthStatus = ExtensionHealth;
    type Metrics = ExtensionMetrics;

    async fn health_check(&self) -> Result<Self::HealthStatus> {
        let status = match &self.state {
            ExtensionState::Active => HealthStatus::Healthy,
            ExtensionState::Loaded => HealthStatus::Healthy,
            ExtensionState::Paused => HealthStatus::Degraded("Extension is paused".to_string()),
            ExtensionState::Error(e) => HealthStatus::Unhealthy(e.clone()),
            _ => HealthStatus::Unknown,
        };

        Ok(ExtensionHealth {
            status,
            last_check: chrono::Utc::now(),
            uptime: chrono::Utc::now()
                .signed_duration_since(self.created_at)
                .to_std()
                .unwrap_or_default(),
            error_count: 0,         // Would be tracked by extension runtime
            performance_score: 1.0, // Would be calculated based on metrics
        })
    }

    async fn metrics(&self) -> Result<Self::Metrics> {
        // In a real implementation, these would be collected from the running extension
        Ok(ExtensionMetrics {
            memory_usage: 0,
            cpu_usage: 0.0,
            requests_handled: 0,
            errors_count: 0,
            average_response_time: std::time::Duration::from_millis(0),
            uptime: chrono::Utc::now()
                .signed_duration_since(self.created_at)
                .to_std()
                .unwrap_or_default(),
        })
    }
}

impl Validatable for Extension {
    type ValidationResult = Vec<String>;

    fn validate(&self) -> Result<Self::ValidationResult> {
        let mut issues = Vec::new();

        // Validate name is not empty
        if self.name.trim().is_empty() {
            issues.push("Extension name cannot be empty".to_string());
        }

        // Validate description is not empty
        if self.description.trim().is_empty() {
            issues.push("Extension description cannot be empty".to_string());
        }

        // Validate author is not empty
        if self.author.trim().is_empty() {
            issues.push("Extension author cannot be empty".to_string());
        }

        // Validate capabilities are specified
        if self.capabilities.is_empty() {
            issues.push("Extension must specify at least one capability".to_string());
        }

        // Validate configuration
        if let Err(e) = Self::validate_config(&self.config) {
            issues.push(format!("Configuration validation failed: {}", e));
        }

        Ok(issues)
    }

    fn auto_fix(&mut self) -> Result<Vec<String>> {
        let mut fixes = Vec::new();

        // Auto-fix empty name
        if self.name.trim().is_empty() {
            self.name = format!("Extension-{}", Uuid::new_v4());
            fixes.push("Generated name for extension".to_string());
        }

        // Auto-fix empty description
        if self.description.trim().is_empty() {
            self.description = "No description provided".to_string();
            fixes.push("Added default description".to_string());
        }

        // Auto-fix empty author
        if self.author.trim().is_empty() {
            self.author = "Unknown".to_string();
            fixes.push("Set default author".to_string());
        }

        // Auto-fix missing capabilities
        if self.capabilities.is_empty() {
            self.capabilities.push(ExtensionCapability::Integration);
            fixes.push("Added default capability".to_string());
        }

        if !fixes.is_empty() {
            self.touch();
        }

        Ok(fixes)
    }
}

/// Extension trait for runtime behavior
#[async_trait]
pub trait ExtensionRuntime: Send + Sync {
    /// Initialize the extension
    async fn initialize(&mut self, config: &ExtensionConfig) -> Result<()>;

    /// Start the extension
    async fn start(&mut self) -> Result<()>;

    /// Stop the extension
    async fn stop(&mut self) -> Result<()>;

    /// Pause the extension
    async fn pause(&mut self) -> Result<()>;

    /// Resume the extension
    async fn resume(&mut self) -> Result<()>;

    /// Handle a request (for command extensions)
    async fn handle_request(&mut self, request: ExtensionRequest) -> Result<ExtensionResponse>;

    /// Get current metrics
    async fn get_metrics(&self) -> Result<ExtensionMetrics>;

    /// Perform health check
    async fn health_check(&self) -> Result<ExtensionHealth>;
}

/// Extension request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

/// Extension response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionResponse {
    pub id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}
