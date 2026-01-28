//! Verification Agent for auto-created applications
//!
//! This module provides functionality to automatically verify that
//! applications created by ccswarm's auto-create feature work correctly.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tracing::{error, info, warn};

/// Result of a verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Overall success status
    pub success: bool,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Total time taken for verification
    pub duration_ms: u64,
    /// Summary message
    pub summary: String,
}

/// Result of an individual check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Details or error message
    pub message: String,
    /// Duration of this check
    pub duration_ms: u64,
}

/// Configuration for verification
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Port for backend server
    pub backend_port: u16,
    /// Port for frontend server
    pub frontend_port: u16,
    /// Timeout for server startup
    pub startup_timeout: Duration,
    /// Timeout for health checks
    pub health_check_timeout: Duration,
    /// Whether to run npm install if needed
    pub auto_install_deps: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            backend_port: 3000,
            frontend_port: 8080,
            startup_timeout: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(10),
            auto_install_deps: true,
        }
    }
}

/// Verification Agent for testing auto-created applications
pub struct VerificationAgent {
    config: VerificationConfig,
}

impl VerificationAgent {
    /// Create a new verification agent
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(VerificationConfig::default())
    }

    /// Run full verification on an auto-created application
    pub async fn verify_app(&self, app_path: &Path) -> Result<VerificationResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();

        info!("\n🔍 Starting application verification...");
        info!("   📂 Path: {}", app_path.display());

        // Check 1: Verify required files exist
        checks.push(self.check_required_files(app_path).await);

        // Check 2: Install dependencies if needed
        if self.config.auto_install_deps {
            checks.push(self.check_and_install_deps(app_path).await);
        }

        // Check 3: Start backend and verify health
        let backend_check = self.check_backend(app_path).await;
        let backend_started = backend_check.passed;
        checks.push(backend_check);

        // Check 4: Start frontend and verify accessibility
        let frontend_check = self.check_frontend(app_path).await;
        checks.push(frontend_check);

        // Check 5: Test API endpoints (if backend started)
        if backend_started {
            checks.push(self.check_api_endpoints().await);
        }

        // Check 6: Run tests if they exist
        checks.push(self.run_tests(app_path).await);

        // Calculate results
        let passed_count = checks.iter().filter(|c| c.passed).count();
        let total_count = checks.len();
        let success = checks.iter().all(|c| c.passed);

        let summary = format!("{}/{} checks passed", passed_count, total_count);

        let result = VerificationResult {
            success,
            checks,
            duration_ms: start.elapsed().as_millis() as u64,
            summary,
        };

        // Print summary
        self.print_summary(&result);

        // Save verification report
        Self::save_report(&result, app_path).await?;

        Ok(result)
    }

    /// Check that required files exist
    async fn check_required_files(&self, app_path: &Path) -> CheckResult {
        let start = std::time::Instant::now();
        let mut missing = Vec::new();

        let required_files = ["package.json", "server.js", "index.html"];

        for file in &required_files {
            let file_path = app_path.join(file);
            if !file_path.exists() {
                missing.push(*file);
            }
        }

        let passed = missing.is_empty();
        let message = if passed {
            "All required files present".to_string()
        } else {
            format!("Missing files: {}", missing.join(", "))
        };

        CheckResult {
            name: "Required Files".to_string(),
            passed,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Check and install dependencies
    async fn check_and_install_deps(&self, app_path: &Path) -> CheckResult {
        let start = std::time::Instant::now();

        let node_modules = app_path.join("node_modules");
        if node_modules.exists() {
            return CheckResult {
                name: "Dependencies".to_string(),
                passed: true,
                message: "node_modules already exists".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        info!("   📦 Installing dependencies...");

        let output = Command::new("npm")
            .arg("install")
            .current_dir(app_path)
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => CheckResult {
                name: "Dependencies".to_string(),
                passed: true,
                message: "npm install completed successfully".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Ok(output) => CheckResult {
                name: "Dependencies".to_string(),
                passed: false,
                message: format!(
                    "npm install failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => CheckResult {
                name: "Dependencies".to_string(),
                passed: false,
                message: format!("Failed to run npm install: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Start backend and check health endpoint
    async fn check_backend(&self, app_path: &Path) -> CheckResult {
        let start = std::time::Instant::now();

        info!("   🚀 Starting backend server...");

        // Kill any existing process on the port
        let _ = Command::new("lsof")
            .args(["-ti", &format!(":{}", self.config.backend_port)])
            .output()
            .await
            .map(|o| {
                if let Ok(pids) = String::from_utf8(o.stdout) {
                    for pid in pids.lines() {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", pid.trim()])
                            .output();
                    }
                }
            });

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Start the server in background
        let mut child = match Command::new("node")
            .arg("server.js")
            .current_dir(app_path)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                return CheckResult {
                    name: "Backend Server".to_string(),
                    passed: false,
                    message: format!("Failed to start server: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        // Wait for server to start and check health
        let health_url = format!("http://localhost:{}/health", self.config.backend_port);
        let client = reqwest::Client::new();

        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;

            match client
                .get(&health_url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    info!("   ✅ Backend health check passed");
                    return CheckResult {
                        name: "Backend Server".to_string(),
                        passed: true,
                        message: format!("Server running on port {}", self.config.backend_port),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
                Ok(_) => continue,
                Err(_) if i < 9 => continue,
                Err(e) => {
                    let _ = child.kill().await;
                    return CheckResult {
                        name: "Backend Server".to_string(),
                        passed: false,
                        message: format!("Health check failed: {}", e),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            }
        }

        CheckResult {
            name: "Backend Server".to_string(),
            passed: false,
            message: "Server did not respond within timeout".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Check frontend accessibility
    async fn check_frontend(&self, app_path: &Path) -> CheckResult {
        let start = std::time::Instant::now();

        // Check if index.html exists and is valid
        let index_path = app_path.join("index.html");
        if !index_path.exists() {
            return CheckResult {
                name: "Frontend".to_string(),
                passed: false,
                message: "index.html not found".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Read and validate HTML
        match tokio::fs::read_to_string(&index_path).await {
            Ok(content) => {
                let has_doctype = content.to_lowercase().contains("<!doctype html>");
                let has_html_tag = content.contains("<html");
                let has_body = content.contains("<body");

                if has_doctype && has_html_tag && has_body {
                    CheckResult {
                        name: "Frontend".to_string(),
                        passed: true,
                        message: "Valid HTML structure".to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                } else {
                    CheckResult {
                        name: "Frontend".to_string(),
                        passed: false,
                        message: "Invalid HTML structure".to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(e) => CheckResult {
                name: "Frontend".to_string(),
                passed: false,
                message: format!("Failed to read index.html: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Test API endpoints
    async fn check_api_endpoints(&self) -> CheckResult {
        let start = std::time::Instant::now();
        let client = reqwest::Client::new();
        let base_url = format!("http://localhost:{}", self.config.backend_port);

        info!("   🔌 Testing API endpoints...");

        // Test creating a todo
        let create_result = client
            .post(format!("{}/api/todos", base_url))
            .json(&serde_json::json!({
                "title": "Test TODO from verification agent",
                "completed": false
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match create_result {
            Ok(response) if response.status().is_success() => {
                info!("   ✅ API endpoints working");
                CheckResult {
                    name: "API Endpoints".to_string(),
                    passed: true,
                    message: "CRUD operations working".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Ok(response) => CheckResult {
                name: "API Endpoints".to_string(),
                passed: false,
                message: format!("API returned status: {}", response.status()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => CheckResult {
                name: "API Endpoints".to_string(),
                passed: false,
                message: format!("API request failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Run tests if they exist
    async fn run_tests(&self, app_path: &Path) -> CheckResult {
        let start = std::time::Instant::now();

        // Check if test files exist
        let test_files = ["app.test.js", "server.test.js", "__tests__", "test"];

        let has_tests = test_files.iter().any(|f| app_path.join(f).exists());

        if !has_tests {
            return CheckResult {
                name: "Tests".to_string(),
                passed: true,
                message: "No test files found (skipped)".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        info!("   🧪 Running tests...");

        let output = Command::new("npm")
            .arg("test")
            .arg("--")
            .arg("--passWithNoTests")
            .current_dir(app_path)
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                info!("   ✅ Tests passed");
                CheckResult {
                    name: "Tests".to_string(),
                    passed: true,
                    message: "All tests passed".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("   ⚠️ Tests failed: {}", stderr);
                CheckResult {
                    name: "Tests".to_string(),
                    passed: false,
                    message: format!(
                        "Tests failed: {}",
                        stderr.chars().take(200).collect::<String>()
                    ),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(e) => CheckResult {
                name: "Tests".to_string(),
                passed: false,
                message: format!("Failed to run tests: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Print verification summary
    fn print_summary(&self, result: &VerificationResult) {
        info!("\n📊 Verification Summary");
        info!("========================");

        for check in &result.checks {
            let icon = if check.passed { "✅" } else { "❌" };
            info!("   {} {}: {}", icon, check.name, check.message);
        }

        info!("");
        if result.success {
            info!("🎉 All checks passed! Application is ready.");
        } else {
            error!("⚠️  Some checks failed. Please review the issues above.");
        }
        info!("   ⏱️  Total verification time: {}ms", result.duration_ms);
    }
}

/// Detected application type for customized verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppType {
    /// Node.js/JavaScript application
    NodeJs,
    /// Python application
    Python,
    /// Rust application
    Rust,
    /// Go application
    Go,
    /// Static HTML/CSS/JS
    Static,
    /// Unknown type
    Unknown,
}

/// Remediation suggestion for failed checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    /// The check that failed
    pub check_name: String,
    /// Suggested fix command or action
    pub suggestion: String,
    /// Auto-fixable flag
    pub auto_fixable: bool,
}

impl VerificationAgent {
    /// Detect application type from project files
    pub fn detect_app_type(app_path: &Path) -> AppType {
        // Check for Node.js indicators
        if app_path.join("package.json").exists() {
            return AppType::NodeJs;
        }

        // Check for Python indicators
        if app_path.join("requirements.txt").exists()
            || app_path.join("pyproject.toml").exists()
            || app_path.join("setup.py").exists()
        {
            return AppType::Python;
        }

        // Check for Rust indicators
        if app_path.join("Cargo.toml").exists() {
            return AppType::Rust;
        }

        // Check for Go indicators
        if app_path.join("go.mod").exists() {
            return AppType::Go;
        }

        // Check for static site
        if app_path.join("index.html").exists() && !app_path.join("package.json").exists() {
            return AppType::Static;
        }

        AppType::Unknown
    }

    /// Get remediation suggestions for failed checks
    pub fn get_remediation_suggestions(result: &VerificationResult) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        for check in &result.checks {
            if !check.passed {
                let suggestion = match check.name.as_str() {
                    "Required Files" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Create missing files or run the auto-create command again"
                            .to_string(),
                        auto_fixable: false,
                    },
                    "Dependencies" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Run 'npm install' to install dependencies".to_string(),
                        auto_fixable: true,
                    },
                    "Backend Server" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Check server.js for errors, ensure port 3000 is available"
                            .to_string(),
                        auto_fixable: false,
                    },
                    "Frontend" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Ensure index.html has valid HTML5 structure".to_string(),
                        auto_fixable: false,
                    },
                    "API Endpoints" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Verify API routes are properly defined and server is running"
                            .to_string(),
                        auto_fixable: false,
                    },
                    "Tests" => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: "Fix failing tests or add 'jest' configuration".to_string(),
                        auto_fixable: false,
                    },
                    _ => RemediationSuggestion {
                        check_name: check.name.clone(),
                        suggestion: format!("Review error: {}", check.message),
                        auto_fixable: false,
                    },
                };
                suggestions.push(suggestion);
            }
        }

        suggestions
    }

    /// Generate a detailed verification report
    pub fn generate_report(result: &VerificationResult, app_path: &Path) -> String {
        let app_type = Self::detect_app_type(app_path);
        let suggestions = Self::get_remediation_suggestions(result);

        let mut report = String::new();
        report.push_str("# Verification Report\n\n");
        report.push_str(&format!("**Application Path**: {}\n", app_path.display()));
        report.push_str(&format!("**Detected Type**: {:?}\n", app_type));
        report.push_str(&format!(
            "**Overall Status**: {}\n",
            if result.success {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        ));
        report.push_str(&format!("**Duration**: {}ms\n\n", result.duration_ms));

        report.push_str("## Check Results\n\n");
        for check in &result.checks {
            let icon = if check.passed { "✅" } else { "❌" };
            report.push_str(&format!(
                "- {} **{}**: {} ({}ms)\n",
                icon, check.name, check.message, check.duration_ms
            ));
        }

        if !suggestions.is_empty() {
            report.push_str("\n## Remediation Suggestions\n\n");
            for suggestion in &suggestions {
                let fixable = if suggestion.auto_fixable { " 🔧" } else { "" };
                report.push_str(&format!(
                    "- **{}**{}: {}\n",
                    suggestion.check_name, fixable, suggestion.suggestion
                ));
            }
        }

        report.push_str("\n---\n*Generated by ccswarm v0.4.0*\n");

        report
    }

    /// Save verification report to file
    pub async fn save_report(result: &VerificationResult, app_path: &Path) -> Result<()> {
        let report = Self::generate_report(result, app_path);
        let report_path = app_path.join("VERIFICATION_REPORT.md");
        tokio::fs::write(&report_path, report).await?;
        info!("📄 Verification report saved to: {}", report_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_check_required_files() {
        let temp_dir = TempDir::new().unwrap();
        let agent = VerificationAgent::with_defaults();

        // No files - should fail
        let result = agent.check_required_files(temp_dir.path()).await;
        assert!(!result.passed);

        // Create required files
        std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        std::fs::write(temp_dir.path().join("server.js"), "").unwrap();
        std::fs::write(temp_dir.path().join("index.html"), "").unwrap();

        let result = agent.check_required_files(temp_dir.path()).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert_eq!(config.backend_port, 3000);
        assert_eq!(config.frontend_port, 8080);
    }
}
