use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

use super::owasp_checker::{OwaspCategory, OwaspChecker};
use super::vulnerability_scanner::{Vulnerability, VulnerabilityScanner};

/// Security Agent for automated security analysis and vulnerability detection
pub struct SecurityAgent {
    /// OWASP Top 10 checker
    owasp_checker: OwaspChecker,

    /// Dependency vulnerability scanner
    vulnerability_scanner: VulnerabilityScanner,

    /// Security check configuration
    config: SecurityConfig,

    /// Security scan history
    scan_history: Vec<SecurityScanResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Minimum severity level to report
    pub min_severity: ViolationSeverity,

    /// Whether to fail on any security violations
    pub fail_on_violations: bool,

    /// OWASP categories to check
    pub enabled_owasp_categories: Vec<OwaspCategory>,

    /// Whether to scan dependencies for vulnerabilities
    pub scan_dependencies: bool,

    /// Maximum age of vulnerability database (days)
    pub max_vuln_db_age: u32,

    /// Paths to exclude from scanning
    pub excluded_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    /// Scan timestamp
    pub timestamp: DateTime<Utc>,

    /// Path scanned
    pub path: String,

    /// Security violations found
    pub violations: Vec<SecurityViolation>,

    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,

    /// Overall security score (0.0 to 1.0)
    pub security_score: f64,

    /// Scan duration in milliseconds
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    /// Type of security check that failed
    pub check_type: SecurityCheck,

    /// Severity of the violation
    pub severity: ViolationSeverity,

    /// File where violation was found
    pub file_path: String,

    /// Line number (if applicable)
    pub line_number: Option<u32>,

    /// Description of the violation
    pub description: String,

    /// Suggested fix
    pub suggested_fix: String,

    /// CWE (Common Weakness Enumeration) ID if applicable
    pub cwe_id: Option<u32>,

    /// OWASP category if applicable
    pub owasp_category: Option<OwaspCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityCheck {
    /// SQL injection vulnerability check
    SqlInjection,

    /// Cross-Site Scripting (XSS) check
    CrossSiteScripting,

    /// Broken authentication check
    BrokenAuthentication,

    /// Sensitive data exposure check
    SensitiveDataExposure,

    /// XML external entity (XXE) check
    XmlExternalEntity,

    /// Broken access control check
    BrokenAccessControl,

    /// Security misconfiguration check
    SecurityMisconfiguration,

    /// Cross-Site Request Forgery (CSRF) check
    CrossSiteRequestForgery,

    /// Using components with known vulnerabilities
    VulnerableComponents,

    /// Insufficient logging and monitoring
    InsufficientLogging,

    /// Hardcoded secrets check
    HardcodedSecrets,

    /// Insecure dependencies
    InsecureDependencies,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            min_severity: ViolationSeverity::Medium,
            fail_on_violations: false,
            enabled_owasp_categories: vec![
                OwaspCategory::InjectionFlaws,
                OwaspCategory::BrokenAuthentication,
                OwaspCategory::SensitiveDataExposure,
                OwaspCategory::XmlExternalEntity,
                OwaspCategory::BrokenAccessControl,
                OwaspCategory::SecurityMisconfiguration,
                OwaspCategory::CrossSiteScripting,
                OwaspCategory::InsecureDeserialization,
                OwaspCategory::VulnerableComponents,
                OwaspCategory::InsufficientLogging,
            ],
            scan_dependencies: true,
            max_vuln_db_age: 7, // 1 week
            excluded_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "*.test.js".to_string(),
                "*.spec.ts".to_string(),
            ],
        }
    }
}

impl SecurityAgent {
    /// Create a new Security Agent instance
    pub async fn new() -> Result<Self> {
        Self::with_config(SecurityConfig::default()).await
    }

    /// Create a new Security Agent with custom configuration
    pub async fn with_config(config: SecurityConfig) -> Result<Self> {
        let owasp_checker = OwaspChecker::new();
        let vulnerability_scanner = VulnerabilityScanner::new().await?;

        Ok(Self {
            owasp_checker,
            vulnerability_scanner,
            config,
            scan_history: Vec::new(),
        })
    }

    /// Perform comprehensive security scan on a directory
    pub async fn scan_directory(&mut self, path: &Path) -> Result<SecurityScanResult> {
        let start_time = std::time::Instant::now();
        info!("Starting security scan of directory: {}", path.display());

        let mut violations = Vec::new();
        let mut vulnerabilities = Vec::new();

        // Run OWASP checks
        if !self.config.enabled_owasp_categories.is_empty() {
            let owasp_violations = self
                .owasp_checker
                .check_directory(path, &self.config.enabled_owasp_categories)
                .await?;
            violations.extend(owasp_violations);
        }

        // Run dependency vulnerability scan
        if self.config.scan_dependencies {
            let dep_vulnerabilities = self.vulnerability_scanner.scan_dependencies(path).await?;
            vulnerabilities.extend(dep_vulnerabilities);
        }

        // Calculate security score
        let security_score = self.calculate_security_score(&violations, &vulnerabilities);

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let result = SecurityScanResult {
            timestamp: Utc::now(),
            path: path.to_string_lossy().to_string(),
            violations,
            vulnerabilities,
            security_score,
            duration_ms,
        };

        // Store scan result in history
        self.scan_history.push(result.clone());

        // Keep only last 100 scan results
        if self.scan_history.len() > 100 {
            self.scan_history.drain(0..self.scan_history.len() - 100);
        }

        info!(
            "Security scan completed in {}ms. Score: {:.2}, Violations: {}, Vulnerabilities: {}",
            duration_ms,
            security_score,
            result.violations.len(),
            result.vulnerabilities.len()
        );

        Ok(result)
    }

    /// Perform quick security check on a single file
    pub async fn scan_file(&self, file_path: &Path) -> Result<Vec<SecurityViolation>> {
        info!("Scanning file for security issues: {}", file_path.display());

        // Check if file should be excluded
        if self.should_exclude_path(file_path) {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Run OWASP checks on the file
        for category in &self.config.enabled_owasp_categories {
            let file_violations = self.owasp_checker.check_file(file_path, category).await?;
            violations.extend(file_violations);
        }

        // Filter by minimum severity
        violations.retain(|v| v.severity >= self.config.min_severity);

        Ok(violations)
    }

    /// Calculate overall security score based on violations and vulnerabilities
    fn calculate_security_score(
        &self,
        violations: &[SecurityViolation],
        vulnerabilities: &[Vulnerability],
    ) -> f64 {
        if violations.is_empty() && vulnerabilities.is_empty() {
            return 1.0; // Perfect score
        }

        let mut penalty = 0.0;

        // Penalize based on violation severity
        for violation in violations {
            penalty += match violation.severity {
                ViolationSeverity::Low => 0.05,
                ViolationSeverity::Medium => 0.1,
                ViolationSeverity::High => 0.2,
                ViolationSeverity::Critical => 0.4,
            };
        }

        // Penalize based on vulnerability severity
        for vulnerability in vulnerabilities {
            penalty += match vulnerability.severity.as_str() {
                "LOW" => 0.05,
                "MEDIUM" => 0.1,
                "HIGH" => 0.2,
                "CRITICAL" => 0.4,
                _ => 0.1, // Default for unknown severity
            };
        }

        // Ensure score is between 0.0 and 1.0
        (1.0_f64 - penalty).clamp(0.0_f64, 1.0_f64)
    }

    /// Check if a path should be excluded from scanning
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.excluded_paths {
            if pattern.contains('*') {
                // Simple glob matching
                if let Some(extension) = pattern.strip_prefix("*.") {
                    if path_str.ends_with(extension) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Generate security report
    pub fn generate_security_report(&self) -> SecurityReport {
        let total_scans = self.scan_history.len();

        if total_scans == 0 {
            return SecurityReport::default();
        }

        let recent_scans = self.scan_history.iter().rev().take(10).collect::<Vec<_>>();

        let avg_score =
            recent_scans.iter().map(|s| s.security_score).sum::<f64>() / recent_scans.len() as f64;

        let total_violations = recent_scans
            .iter()
            .map(|s| s.violations.len())
            .sum::<usize>();

        let total_vulnerabilities = recent_scans
            .iter()
            .map(|s| s.vulnerabilities.len())
            .sum::<usize>();

        let critical_violations = recent_scans
            .iter()
            .flat_map(|s| &s.violations)
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .count();

        let high_violations = recent_scans
            .iter()
            .flat_map(|s| &s.violations)
            .filter(|v| v.severity == ViolationSeverity::High)
            .count();

        // Get most common violation types
        let mut violation_counts: HashMap<String, usize> = HashMap::new();
        for scan in &recent_scans {
            for violation in &scan.violations {
                let violation_type = format!("{:?}", violation.check_type);
                *violation_counts.entry(violation_type).or_insert(0) += 1;
            }
        }

        let mut common_violations: Vec<_> = violation_counts.into_iter().collect();
        common_violations.sort_by(|a, b| b.1.cmp(&a.1));
        common_violations.truncate(5); // Top 5

        SecurityReport {
            total_scans,
            average_security_score: avg_score,
            total_violations,
            total_vulnerabilities,
            critical_violations,
            high_violations,
            most_common_violations: common_violations,
            last_scan_time: self.scan_history.last().map(|s| s.timestamp),
        }
    }

    /// Get scan history
    pub fn get_scan_history(&self) -> &[SecurityScanResult] {
        &self.scan_history
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }

    /// Check if the agent should fail based on scan results
    pub fn should_fail_build(&self, scan_result: &SecurityScanResult) -> bool {
        if !self.config.fail_on_violations {
            return false;
        }

        // Fail if there are critical violations
        scan_result.violations.iter().any(|v| v.severity == ViolationSeverity::Critical) ||
        // Fail if security score is too low
        scan_result.security_score < 0.7
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub total_scans: usize,
    pub average_security_score: f64,
    pub total_violations: usize,
    pub total_vulnerabilities: usize,
    pub critical_violations: usize,
    pub high_violations: usize,
    pub most_common_violations: Vec<(String, usize)>,
    pub last_scan_time: Option<DateTime<Utc>>,
}

impl Default for SecurityReport {
    fn default() -> Self {
        Self {
            total_scans: 0,
            average_security_score: 1.0,
            total_violations: 0,
            total_vulnerabilities: 0,
            critical_violations: 0,
            high_violations: 0,
            most_common_violations: Vec::new(),
            last_scan_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_agent_creation() {
        let agent = SecurityAgent::new().await.unwrap();
        assert_eq!(agent.scan_history.len(), 0);
    }

    #[tokio::test]
    async fn test_should_exclude_path() {
        let agent = SecurityAgent::new().await.unwrap();

        assert!(agent.should_exclude_path(Path::new("node_modules/some/file.js")));
        assert!(agent.should_exclude_path(Path::new("test.spec.ts")));
        assert!(!agent.should_exclude_path(Path::new("src/main.rs")));
    }

    #[tokio::test]
    async fn test_security_score_calculation() {
        let agent = SecurityAgent::new().await.unwrap();

        // Perfect score with no violations
        let score = agent.calculate_security_score(&[], &[]);
        assert_eq!(score, 1.0);

        // Score with violations
        let violations = vec![SecurityViolation {
            check_type: SecurityCheck::SqlInjection,
            severity: ViolationSeverity::High,
            file_path: "test.js".to_string(),
            line_number: Some(10),
            description: "SQL injection vulnerability".to_string(),
            suggested_fix: "Use parameterized queries".to_string(),
            cwe_id: Some(89),
            owasp_category: Some(OwaspCategory::InjectionFlaws),
        }];

        let score = agent.calculate_security_score(&violations, &[]);
        assert_eq!(score, 0.8); // 1.0 - 0.2 (high violation penalty)
    }
}
