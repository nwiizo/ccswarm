use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output parser for provider and A2A task responses.
pub struct OutputParser {
    patterns: HashMap<String, regex::Regex>,
}

impl OutputParser {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert(
            "error".to_string(),
            regex::Regex::new(r"(?i)(error|exception|failure)").expect("valid regex"),
        );
        Self { patterns }
    }

    pub fn parse(&self, output: &str) -> Result<ParsedOutput> {
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

        if output.contains("tests passed") || output.contains("All tests passed") {
            return Ok(ParsedOutput::TestResults {
                passed: 1,
                failed: 0,
                details: TestDetails::default(),
            });
        }

        let cargo_re = regex::Regex::new(r"test result:.*?(\d+)\s+passed;\s+(\d+)\s+failed")
            .expect("valid regex");
        if let Some(caps) = cargo_re.captures(output) {
            return Ok(ParsedOutput::TestResults {
                passed: caps[1].parse().unwrap_or(0),
                failed: caps[2].parse().unwrap_or(0),
                details: TestDetails::default(),
            });
        }

        let playwright_re = regex::Regex::new(r"(\d+)\s+passed(?:,\s*(\d+)\s+failed)?\s*\(\d")
            .expect("valid regex");
        if let Some(caps) = playwright_re.captures(output) {
            return Ok(ParsedOutput::TestResults {
                passed: caps[1].parse().unwrap_or(0),
                failed: caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0),
                details: TestDetails::default(),
            });
        }

        let jest_re = regex::Regex::new(r"Tests?:\s*(\d+)\s+passed(?:,\s*(\d+)\s+failed)?")
            .expect("valid regex");
        if let Some(caps) = jest_re.captures(output) {
            return Ok(ParsedOutput::TestResults {
                passed: caps[1].parse().unwrap_or(0),
                failed: caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0),
                details: TestDetails::default(),
            });
        }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedOutput {
    PlainText(String),
    CodeExecution {
        result: String,
        metrics: ExecutionMetrics,
    },
    BuildOutput {
        status: BuildStatus,
        artifacts: Vec<Artifact>,
    },
    TestResults {
        passed: usize,
        failed: usize,
        details: TestDetails,
    },
    StructuredLog {
        level: LogLevel,
        message: String,
        context: LogContext,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub execution_time: std::time::Duration,
    pub memory_usage: Option<usize>,
    pub cpu_usage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Failed(String),
    Warning(String),
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub path: String,
    pub size: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestDetails {
    pub suite: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub failed_tests: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogContext {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub fields: HashMap<String, serde_json::Value>,
}
