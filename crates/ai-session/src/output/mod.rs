//! Intelligent output management for AI sessions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output manager for intelligent processing
pub struct OutputManager {
    /// Output parser
    pub parser: OutputParser,
    /// Semantic compressor
    pub compressor: SemanticCompressor,
    /// Output cache
    _cache: HashMap<String, ParsedOutput>,
}

impl OutputManager {
    /// Create a new output manager
    pub fn new() -> Self {
        Self {
            parser: OutputParser::new(),
            compressor: SemanticCompressor::new(),
            _cache: HashMap::new(),
        }
    }

    /// Process raw output
    pub fn process_output(&mut self, raw_output: &str) -> Result<ProcessedOutput> {
        // Parse the output
        let parsed = self.parser.parse(raw_output)?;

        // Extract highlights
        let highlights = self.extract_highlights(&parsed);

        // Compress if needed
        let compressed = if raw_output.len() > 1024 {
            Some(self.compressor.compress(raw_output)?)
        } else {
            None
        };

        Ok(ProcessedOutput {
            raw: raw_output.to_string(),
            parsed: parsed.clone(),
            highlights,
            compressed,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Extract highlights from parsed output
    fn extract_highlights(&self, parsed: &ParsedOutput) -> Vec<Highlight> {
        let mut highlights = Vec::new();

        match parsed {
            ParsedOutput::CodeExecution { result: _, metrics }
                if metrics.execution_time > std::time::Duration::from_secs(5) =>
            {
                highlights.push(Highlight {
                    category: HighlightCategory::Performance,
                    message: format!("Slow execution: {:?}", metrics.execution_time),
                    severity: Severity::Warning,
                });
            }
            ParsedOutput::BuildOutput { status, .. } => match status {
                BuildStatus::Failed(error) => {
                    highlights.push(Highlight {
                        category: HighlightCategory::Error,
                        message: error.clone(),
                        severity: Severity::Error,
                    });
                }
                BuildStatus::Warning(warning) => {
                    highlights.push(Highlight {
                        category: HighlightCategory::Warning,
                        message: warning.clone(),
                        severity: Severity::Warning,
                    });
                }
                _ => {}
            },
            ParsedOutput::TestResults { failed, .. } if *failed > 0 => {
                highlights.push(Highlight {
                    category: HighlightCategory::TestFailure,
                    message: format!("{} tests failed", failed),
                    severity: Severity::Error,
                });
            }
            ParsedOutput::StructuredLog { level, message, .. } => {
                if matches!(level, LogLevel::Error | LogLevel::Warning) {
                    highlights.push(Highlight {
                        category: HighlightCategory::Log,
                        message: message.clone(),
                        severity: match level {
                            LogLevel::Error => Severity::Error,
                            LogLevel::Warning => Severity::Warning,
                            _ => Severity::Info,
                        },
                    });
                }
            }
            _ => {}
        }

        highlights
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Output parser
pub struct OutputParser {
    /// Pattern matchers
    patterns: HashMap<String, regex::Regex>,
}

impl OutputParser {
    /// Create a new parser
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Add common patterns
        patterns.insert(
            "error".to_string(),
            regex::Regex::new(r"(?i)(error|exception|failure)").unwrap(),
        );
        patterns.insert(
            "warning".to_string(),
            regex::Regex::new(r"(?i)(warning|warn)").unwrap(),
        );
        patterns.insert(
            "success".to_string(),
            regex::Regex::new(r"(?i)(success|passed|completed)").unwrap(),
        );

        Self { patterns }
    }

    /// Parse output into structured format
    pub fn parse(&self, output: &str) -> Result<ParsedOutput> {
        // Build output patterns
        if output.contains("BUILD SUCCESSFUL") || output.contains("Build succeeded") {
            return Ok(ParsedOutput::BuildOutput {
                status: BuildStatus::Success,
                artifacts: Vec::new(),
            });
        }
        if output.contains("BUILD FAILED") || output.contains("Build failed") {
            return Ok(ParsedOutput::BuildOutput {
                status: BuildStatus::Failed("Build failed".to_string()),
                artifacts: Vec::new(),
            });
        }

        // Cargo / generic "tests passed" pattern
        if output.contains("tests passed") || output.contains("All tests passed") {
            return Ok(ParsedOutput::TestResults {
                passed: 1, // Placeholder for legacy plain-string matches
                failed: 0,
                details: TestDetails::default(),
            });
        }

        // Rust `cargo test` summary: "test result: ok. 10 passed; 0 failed"
        // Covers both passing and failing runs.
        let cargo_re = regex::Regex::new(r"test result:.*?(\d+)\s+passed;\s+(\d+)\s+failed")
            .expect("valid regex");
        if let Some(caps) = cargo_re.captures(output) {
            let passed: usize = caps[1].parse().unwrap_or(0);
            let failed: usize = caps[2].parse().unwrap_or(0);
            return Ok(ParsedOutput::TestResults {
                passed,
                failed,
                details: TestDetails::default(),
            });
        }

        // Playwright output: "7 passed (3.0s)" or "5 passed, 2 failed (10s)"
        // The failed group is optional.
        let playwright_re = regex::Regex::new(r"(\d+)\s+passed(?:,\s*(\d+)\s+failed)?\s*\(\d")
            .expect("valid regex");
        if let Some(caps) = playwright_re.captures(output) {
            let passed: usize = caps[1].parse().unwrap_or(0);
            let failed: usize = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            return Ok(ParsedOutput::TestResults {
                passed,
                failed,
                details: TestDetails::default(),
            });
        }

        // npm test / Jest summary: "Tests: 3 passed, 1 failed, 4 total"
        // The failed group is optional (all-passing runs omit it).
        let jest_re = regex::Regex::new(r"Tests?:\s*(\d+)\s+passed(?:,\s*(\d+)\s+failed)?")
            .expect("valid regex");
        if let Some(caps) = jest_re.captures(output) {
            let passed: usize = caps[1].parse().unwrap_or(0);
            let failed: usize = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            return Ok(ParsedOutput::TestResults {
                passed,
                failed,
                details: TestDetails::default(),
            });
        }

        // Generic error log fallback
        if self.patterns["error"].is_match(output) {
            return Ok(ParsedOutput::StructuredLog {
                level: LogLevel::Error,
                message: output.to_string(),
                context: LogContext::default(),
            });
        }

        Ok(ParsedOutput::PlainText(output.to_string()))
    }
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic compressor
pub struct SemanticCompressor {
    /// Compression level (0.0 - 1.0)
    _compression_level: f32,
}

impl SemanticCompressor {
    /// Create a new compressor
    pub fn new() -> Self {
        Self {
            _compression_level: 0.5,
        }
    }

    /// Compress output semantically
    pub fn compress(&self, output: &str) -> Result<CompressedOutput> {
        // Simple implementation: just truncate for now
        // In a real implementation, this would use NLP techniques
        let compressed = if output.len() > 500 {
            format!("{}... (truncated)", &output[..500])
        } else {
            output.to_string()
        };

        let compressed_len = compressed.len();
        let original_len = output.len();

        Ok(CompressedOutput {
            original_size: original_len,
            compressed_size: compressed_len,
            content: compressed,
            compression_ratio: compressed_len as f32 / original_len as f32,
        })
    }
}

impl Default for SemanticCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedOutput {
    /// Plain text output
    PlainText(String),

    /// Code execution result
    CodeExecution {
        result: String,
        metrics: ExecutionMetrics,
    },

    /// Build output
    BuildOutput {
        status: BuildStatus,
        artifacts: Vec<Artifact>,
    },

    /// Test results
    TestResults {
        passed: usize,
        failed: usize,
        details: TestDetails,
    },

    /// Structured log
    StructuredLog {
        level: LogLevel,
        message: String,
        context: LogContext,
    },
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Execution time
    pub execution_time: std::time::Duration,
    /// Memory usage
    pub memory_usage: Option<usize>,
    /// CPU usage
    pub cpu_usage: Option<f32>,
}

/// Build status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Failed(String),
    Warning(String),
    InProgress,
}

/// Build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact name
    pub name: String,
    /// Artifact path
    pub path: String,
    /// Size in bytes
    pub size: usize,
}

/// Test details
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestDetails {
    /// Test suite name
    pub suite: Option<String>,
    /// Duration
    pub duration: Option<std::time::Duration>,
    /// Failed test names
    pub failed_tests: Vec<String>,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

/// Log context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogContext {
    /// Source file
    pub file: Option<String>,
    /// Line number
    pub line: Option<usize>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
}

/// Processed output
#[derive(Debug, Clone)]
pub struct ProcessedOutput {
    /// Raw output
    pub raw: String,
    /// Parsed output
    pub parsed: ParsedOutput,
    /// Highlights
    pub highlights: Vec<Highlight>,
    /// Compressed version
    pub compressed: Option<CompressedOutput>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Output highlight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    /// Category
    pub category: HighlightCategory,
    /// Message
    pub message: String,
    /// Severity
    pub severity: Severity,
}

/// Highlight category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightCategory {
    Error,
    Warning,
    Performance,
    TestFailure,
    Log,
    Success,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Compressed output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedOutput {
    /// Original size
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Compressed content
    pub content: String,
    /// Compression ratio
    pub compression_ratio: f32,
}

// Add regex dependency to Cargo.toml for this module
use once_cell::sync::Lazy;
static _REGEX_DEPENDENCY: Lazy<()> = Lazy::new(|| {
    // This is just a marker to remind about adding regex = "1.10" to Cargo.toml
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_parser() {
        let parser = OutputParser::new();

        let output = "BUILD SUCCESSFUL";
        let parsed = parser.parse(output).unwrap();

        match parsed {
            ParsedOutput::BuildOutput { status, .. } => {
                assert!(matches!(status, BuildStatus::Success));
            }
            _ => panic!("Expected BuildOutput"),
        }
    }

    #[test]
    fn test_output_manager() {
        let mut manager = OutputManager::new();

        let output = "Error: Something went wrong";
        let processed = manager.process_output(output).unwrap();

        assert!(!processed.highlights.is_empty());
        assert_eq!(processed.highlights[0].severity, Severity::Error);
    }

    // --- cargo test ---

    #[test]
    fn test_cargo_test_all_passing() {
        let parser = OutputParser::new();
        let output = "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 10);
                assert_eq!(failed, 0);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    #[test]
    fn test_cargo_test_with_failures() {
        let parser = OutputParser::new();
        let output = "test result: FAILED. 8 passed; 2 failed; 0 ignored";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 8);
                assert_eq!(failed, 2);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    // --- Playwright ---

    #[test]
    fn test_playwright_all_passing() {
        let parser = OutputParser::new();
        let output = "  7 passed (3.0s)";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 7);
                assert_eq!(failed, 0);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    #[test]
    fn test_playwright_with_failures() {
        let parser = OutputParser::new();
        let output = "  5 passed, 2 failed (10s)";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 5);
                assert_eq!(failed, 2);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    // --- npm test / Jest ---

    #[test]
    fn test_jest_all_passing() {
        let parser = OutputParser::new();
        let output = "Tests: 3 passed, 4 total";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 3);
                assert_eq!(failed, 0);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    #[test]
    fn test_jest_with_failures() {
        let parser = OutputParser::new();
        let output = "Tests: 3 passed, 1 failed, 4 total";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 3);
                assert_eq!(failed, 1);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }

    #[test]
    fn test_jest_singular_noun() {
        // "Test:" (singular) should also match
        let parser = OutputParser::new();
        let output = "Test: 1 passed, 0 failed, 1 total";
        match parser.parse(output).unwrap() {
            ParsedOutput::TestResults { passed, failed, .. } => {
                assert_eq!(passed, 1);
                assert_eq!(failed, 0);
            }
            other => panic!("expected TestResults, got {:?}", other),
        }
    }
}
