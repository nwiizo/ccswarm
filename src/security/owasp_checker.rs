use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, warn};

use super::security_agent::{SecurityCheck, SecurityViolation, ViolationSeverity};

/// OWASP Top 10 checker for identifying common web application security risks
pub struct OwaspChecker {
    /// Pre-compiled regex patterns for various checks
    patterns: HashMap<OwaspCategory, Vec<SecurityPattern>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OwaspCategory {
    /// A01:2021 - Broken Access Control
    BrokenAccessControl,

    /// A02:2021 - Cryptographic Failures
    CryptographicFailures,

    /// A03:2021 - Injection
    InjectionFlaws,

    /// A04:2021 - Insecure Design
    InsecureDesign,

    /// A05:2021 - Security Misconfiguration
    SecurityMisconfiguration,

    /// A06:2021 - Vulnerable and Outdated Components
    VulnerableComponents,

    /// A07:2021 - Identification and Authentication Failures
    BrokenAuthentication,

    /// A08:2021 - Software and Data Integrity Failures
    IntegrityFailures,

    /// A09:2021 - Security Logging and Monitoring Failures
    InsufficientLogging,

    /// Insecure Deserialization (Legacy OWASP Top 10)
    InsecureDeserialization,

    /// A10:2021 - Server-Side Request Forgery (SSRF)
    ServerSideRequestForgery,

    // Legacy categories still relevant
    /// Cross-Site Scripting (XSS)
    CrossSiteScripting,

    /// XML External Entity (XXE)
    XmlExternalEntity,

    /// Sensitive Data Exposure
    SensitiveDataExposure,
}

#[derive(Debug, Clone)]
struct SecurityPattern {
    /// Regex pattern to match
    pattern: Regex,

    /// Description of what this pattern detects
    description: String,

    /// Suggested fix for this issue
    suggested_fix: String,

    /// Severity of this issue
    severity: ViolationSeverity,

    /// Associated CWE ID
    cwe_id: Option<u32>,

    /// Security check type
    check_type: SecurityCheck,
}

impl Default for OwaspChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl OwaspChecker {
    /// Create a new OWASP checker with predefined patterns
    pub fn new() -> Self {
        let mut checker = Self {
            patterns: HashMap::new(),
        };

        checker.initialize_patterns();
        checker
    }

    /// Initialize security patterns for different OWASP categories
    fn initialize_patterns(&mut self) {
        // A03:2021 - Injection Flaws
        self.add_injection_patterns();

        // A07:2021 - Authentication Failures
        self.add_authentication_patterns();

        // A02:2021 - Cryptographic Failures (formerly Sensitive Data Exposure)
        self.add_cryptographic_patterns();

        // A05:2021 - Security Misconfiguration
        self.add_misconfiguration_patterns();

        // Cross-Site Scripting
        self.add_xss_patterns();

        // A01:2021 - Broken Access Control
        self.add_access_control_patterns();

        // A09:2021 - Insufficient Logging
        self.add_logging_patterns();

        // A10:2021 - SSRF
        self.add_ssrf_patterns();
    }

    /// Add SQL injection and other injection patterns
    fn add_injection_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(
                    r#"(?i)(select|insert|update|delete|drop|create|alter)\s+.*\s*\+\s*.*"#,
                )
                .unwrap(),
                description:
                    "Potential SQL injection vulnerability - string concatenation in SQL query"
                        .to_string(),
                suggested_fix: "Use parameterized queries or prepared statements".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(89),
                check_type: SecurityCheck::SqlInjection,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)query\s*\(\s*["'].*\$\{.*\}.*["']\s*\)"#).unwrap(),
                description: "SQL injection risk - template literals in SQL queries".to_string(),
                suggested_fix: "Use parameterized queries instead of template literals".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(89),
                check_type: SecurityCheck::SqlInjection,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)exec\s*\(\s*["'].*\$\{.*\}.*["']\s*\)"#).unwrap(),
                description: "Command injection vulnerability - dynamic command execution"
                    .to_string(),
                suggested_fix: "Validate and sanitize input, use allowlists for commands"
                    .to_string(),
                severity: ViolationSeverity::Critical,
                cwe_id: Some(78),
                check_type: SecurityCheck::SqlInjection,
            },
        ];

        self.patterns
            .insert(OwaspCategory::InjectionFlaws, patterns);
    }

    /// Add authentication-related patterns
    fn add_authentication_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)password\s*=\s*["'][^"']*["']"#).unwrap(),
                description: "Hardcoded password detected".to_string(),
                suggested_fix: "Use environment variables or secure configuration management"
                    .to_string(),
                severity: ViolationSeverity::Critical,
                cwe_id: Some(798),
                check_type: SecurityCheck::BrokenAuthentication,
            },
            SecurityPattern {
                pattern: Regex::new(
                    r#"(?i)(api_key|apikey|secret|token)\s*=\s*["'][^"']{10,}["']"#,
                )
                .unwrap(),
                description: "Hardcoded API key or secret detected".to_string(),
                suggested_fix: "Store secrets in environment variables or secure vault".to_string(),
                severity: ViolationSeverity::Critical,
                cwe_id: Some(798),
                check_type: SecurityCheck::HardcodedSecrets,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)auth\s*=\s*false"#).unwrap(),
                description: "Authentication disabled".to_string(),
                suggested_fix: "Enable proper authentication mechanisms".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(287),
                check_type: SecurityCheck::BrokenAuthentication,
            },
        ];

        self.patterns
            .insert(OwaspCategory::BrokenAuthentication, patterns);
    }

    /// Add cryptographic failure patterns
    fn add_cryptographic_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)(md5|sha1)\s*\("#).unwrap(),
                description: "Weak cryptographic hash function detected".to_string(),
                suggested_fix: "Use SHA-256, SHA-3, or bcrypt for password hashing".to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(327),
                check_type: SecurityCheck::SensitiveDataExposure,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)crypto\s*\.\s*createCipher\s*\(\s*["']des["']"#)
                    .unwrap(),
                description: "Weak encryption algorithm (DES) detected".to_string(),
                suggested_fix: "Use AES-256 or other strong encryption algorithms".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(327),
                check_type: SecurityCheck::SensitiveDataExposure,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)ssl.*verify\s*=\s*false"#).unwrap(),
                description: "SSL certificate verification disabled".to_string(),
                suggested_fix: "Enable SSL certificate verification".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(295),
                check_type: SecurityCheck::SecurityMisconfiguration,
            },
        ];

        self.patterns
            .insert(OwaspCategory::CryptographicFailures, patterns);
    }

    /// Add security misconfiguration patterns
    fn add_misconfiguration_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)debug\s*=\s*true"#).unwrap(),
                description: "Debug mode enabled in production".to_string(),
                suggested_fix: "Disable debug mode in production environments".to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(489),
                check_type: SecurityCheck::SecurityMisconfiguration,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)cors\s*\(\s*\{\s*origin\s*:\s*["']\*["']"#).unwrap(),
                description: "CORS configured to allow all origins".to_string(),
                suggested_fix: "Restrict CORS to specific trusted domains".to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(942),
                check_type: SecurityCheck::SecurityMisconfiguration,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)helmet\s*\(\s*\{\s*contentSecurityPolicy\s*:\s*false"#)
                    .unwrap(),
                description: "Content Security Policy disabled".to_string(),
                suggested_fix: "Enable and properly configure Content Security Policy".to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(693),
                check_type: SecurityCheck::SecurityMisconfiguration,
            },
        ];

        self.patterns
            .insert(OwaspCategory::SecurityMisconfiguration, patterns);
    }

    /// Add XSS patterns
    fn add_xss_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)innerHTML\s*=\s*.*\+.*"#).unwrap(),
                description: "Potential XSS vulnerability - dynamic innerHTML assignment"
                    .to_string(),
                suggested_fix: "Use textContent or properly sanitize HTML content".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(79),
                check_type: SecurityCheck::CrossSiteScripting,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)document\.write\s*\(\s*.*\+.*\)"#).unwrap(),
                description: "XSS risk - dynamic content in document.write".to_string(),
                suggested_fix: "Avoid document.write or sanitize dynamic content".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(79),
                check_type: SecurityCheck::CrossSiteScripting,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)eval\s*\(\s*.*req\."#).unwrap(),
                description: "Code injection risk - eval with user input".to_string(),
                suggested_fix: "Never use eval() with user input, use JSON.parse() instead"
                    .to_string(),
                severity: ViolationSeverity::Critical,
                cwe_id: Some(94),
                check_type: SecurityCheck::CrossSiteScripting,
            },
        ];

        self.patterns
            .insert(OwaspCategory::CrossSiteScripting, patterns);
    }

    /// Add access control patterns
    fn add_access_control_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)app\.use\s*\(\s*["']/admin["'].*\)"#).unwrap(),
                description: "Admin route without authentication middleware".to_string(),
                suggested_fix: "Add authentication and authorization middleware".to_string(),
                severity: ViolationSeverity::High,
                cwe_id: Some(862),
                check_type: SecurityCheck::BrokenAccessControl,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)role\s*===?\s*["']admin["'].*return true"#).unwrap(),
                description: "Simple role-based access control without proper validation"
                    .to_string(),
                suggested_fix: "Implement proper role validation and session management"
                    .to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(863),
                check_type: SecurityCheck::BrokenAccessControl,
            },
        ];

        self.patterns
            .insert(OwaspCategory::BrokenAccessControl, patterns);
    }

    /// Add logging patterns
    fn add_logging_patterns(&mut self) {
        let patterns = vec![
            SecurityPattern {
                pattern: Regex::new(r#"(?i)console\.log\s*\(\s*.*password.*\)"#).unwrap(),
                description: "Sensitive information logged to console".to_string(),
                suggested_fix: "Remove sensitive data from log statements".to_string(),
                severity: ViolationSeverity::Medium,
                cwe_id: Some(532),
                check_type: SecurityCheck::InsufficientLogging,
            },
            SecurityPattern {
                pattern: Regex::new(r#"(?i)try\s*\{[\s\S]*\}\s*catch\s*\([^)]+\)\s*\{\s*\}"#)
                    .unwrap(),
                description: "Empty catch block - security events not logged".to_string(),
                suggested_fix: "Log security-relevant exceptions and errors".to_string(),
                severity: ViolationSeverity::Low,
                cwe_id: Some(778),
                check_type: SecurityCheck::InsufficientLogging,
            },
        ];

        self.patterns
            .insert(OwaspCategory::InsufficientLogging, patterns);
    }

    /// Add SSRF patterns
    fn add_ssrf_patterns(&mut self) {
        let patterns = vec![SecurityPattern {
            pattern: Regex::new(r#"(?i)(fetch|axios|request)\s*\(\s*.*req\.(query|body|params)"#)
                .unwrap(),
            description: "SSRF vulnerability - HTTP request with user-controlled URL".to_string(),
            suggested_fix: "Validate and allowlist URLs, use URL parsing to check domains"
                .to_string(),
            severity: ViolationSeverity::High,
            cwe_id: Some(918),
            check_type: SecurityCheck::CrossSiteRequestForgery,
        }];

        self.patterns
            .insert(OwaspCategory::ServerSideRequestForgery, patterns);
    }

    /// Check a directory for OWASP violations
    pub async fn check_directory(
        &self,
        path: &Path,
        categories: &[OwaspCategory],
    ) -> Result<Vec<SecurityViolation>> {
        let mut violations = Vec::new();

        // Recursively scan all files
        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Skip common directories that don't need security scanning
                if let Some(dir_name) = entry_path.file_name() {
                    if matches!(
                        dir_name.to_string_lossy().as_ref(),
                        "node_modules" | "target" | ".git" | "dist" | "build"
                    ) {
                        continue;
                    }
                }

                // Recursively check subdirectory
                let sub_violations =
                    Box::pin(self.check_directory(&entry_path, categories)).await?;
                violations.extend(sub_violations);
            } else {
                // Check individual file
                for category in categories {
                    let file_violations = self.check_file(&entry_path, category).await?;
                    violations.extend(file_violations);
                }
            }
        }

        Ok(violations)
    }

    /// Check a single file for OWASP violations
    pub async fn check_file(
        &self,
        file_path: &Path,
        category: &OwaspCategory,
    ) -> Result<Vec<SecurityViolation>> {
        // Skip non-code files
        if !self.is_code_file(file_path) {
            return Ok(Vec::new());
        }

        let patterns = match self.patterns.get(category) {
            Some(patterns) => patterns,
            None => {
                debug!("No patterns defined for OWASP category: {:?}", category);
                return Ok(Vec::new());
            }
        };

        let content = match fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(_) => {
                debug!("Could not read file: {}", file_path.display());
                return Ok(Vec::new());
            }
        };

        let mut violations = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            for pattern in patterns {
                if pattern.pattern.is_match(line) {
                    violations.push(SecurityViolation {
                        check_type: pattern.check_type.clone(),
                        severity: pattern.severity.clone(),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: Some((line_number + 1) as u32),
                        description: pattern.description.clone(),
                        suggested_fix: pattern.suggested_fix.clone(),
                        cwe_id: pattern.cwe_id,
                        owasp_category: Some(category.clone()),
                    });
                }
            }
        }

        if !violations.is_empty() {
            warn!(
                "Found {} security violations in {}",
                violations.len(),
                file_path.display()
            );
        }

        Ok(violations)
    }

    /// Check if a file is a code file that should be scanned
    fn is_code_file(&self, file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(
                ext.as_str(),
                "js" | "ts"
                    | "jsx"
                    | "tsx"
                    | "py"
                    | "java"
                    | "php"
                    | "rb"
                    | "go"
                    | "rs"
                    | "cpp"
                    | "c"
                    | "cs"
                    | "swift"
                    | "kt"
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_sql_injection_detection() {
        let checker = OwaspChecker::new();

        // Create a temporary file with SQL injection vulnerability
        let temp_file = NamedTempFile::with_suffix(".js").unwrap();
        let content = r#"
const query = "SELECT * FROM users WHERE id = " + userId;
        "#;

        tokio::fs::write(temp_file.path(), content).await.unwrap();

        let violations = checker
            .check_file(temp_file.path(), &OwaspCategory::InjectionFlaws)
            .await
            .unwrap();

        assert!(!violations.is_empty());
        assert_eq!(violations[0].check_type, SecurityCheck::SqlInjection);
        assert_eq!(violations[0].severity, ViolationSeverity::High);
    }

    #[tokio::test]
    async fn test_hardcoded_password_detection() {
        let checker = OwaspChecker::new();

        let temp_file = NamedTempFile::with_suffix(".js").unwrap();
        let content = r#"
const password = "mySecretPassword123";
        "#;

        tokio::fs::write(temp_file.path(), content).await.unwrap();

        let violations = checker
            .check_file(temp_file.path(), &OwaspCategory::BrokenAuthentication)
            .await
            .unwrap();

        assert!(!violations.is_empty());
        assert_eq!(
            violations[0].check_type,
            SecurityCheck::BrokenAuthentication
        );
        assert_eq!(violations[0].severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_is_code_file() {
        let checker = OwaspChecker::new();

        assert!(checker.is_code_file(Path::new("test.js")));
        assert!(checker.is_code_file(Path::new("app.py")));
        assert!(checker.is_code_file(Path::new("main.rs")));
        assert!(!checker.is_code_file(Path::new("README.md")));
        assert!(!checker.is_code_file(Path::new("data.json")));
    }
}
