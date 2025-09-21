use serde_json::json;
use std::collections::HashMap;

/// Structured logging utilities
pub struct StructuredLogger {
    context: HashMap<String, serde_json::Value>,
}

impl StructuredLogger {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": level.as_str(),
            "message": message,
            "context": self.context,
        });

        match level {
            LogLevel::Error => eprintln!("{}", log_entry),
            _ => println!("{}", log_entry),
        }
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self::new()
    }
}