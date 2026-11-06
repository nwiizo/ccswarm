//! Success message formatting for human or `--json` output.

use serde_json::Value;

pub enum OutputFormatter {
    Json,
    Human,
}

impl OutputFormatter {
    /// Format a success message with optional structured data.
    pub fn format_success(&self, message: &str, data: Option<Value>) -> String {
        match self {
            OutputFormatter::Json => {
                let mut result = serde_json::json!({
                    "status": "success",
                    "message": message
                });
                if let Some(data) = data {
                    result["data"] = data;
                }
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| message.to_string())
            }
            OutputFormatter::Human => match data {
                Some(data) => format!(
                    "✅ {}\n\n{}",
                    message,
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                ),
                None => format!("✅ {}", message),
            },
        }
    }
}

/// Create a formatter based on output format preference
pub fn create_formatter(json: bool) -> OutputFormatter {
    if json {
        OutputFormatter::Json
    } else {
        OutputFormatter::Human
    }
}
