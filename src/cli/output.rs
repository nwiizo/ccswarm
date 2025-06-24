//! Output formatting utilities for CLI

use serde::Serialize;
use serde_json::Value;

/// Enum to hold different formatter types
pub enum OutputFormatter {
    Json(JsonFormatter),
    Human(HumanFormatter),
}

impl OutputFormatter {
    /// Format a success message
    pub fn format_success(&self, message: &str, data: Option<Value>) -> String {
        match self {
            OutputFormatter::Json(f) => f.format_success(message, data),
            OutputFormatter::Human(f) => f.format_success(message, data),
        }
    }

    /// Format an error message
    #[allow(dead_code)]
    pub fn format_error(&self, error: &str) -> String {
        match self {
            OutputFormatter::Json(f) => f.format_error(error),
            OutputFormatter::Human(f) => f.format_error(error),
        }
    }

    /// Format a list of items
    #[allow(dead_code)]
    pub fn format_list<T: Serialize>(&self, items: &[T], title: &str) -> String {
        match self {
            OutputFormatter::Json(f) => f.format_list(items, title),
            OutputFormatter::Human(f) => f.format_list(items, title),
        }
    }

    /// Format a status report
    #[allow(dead_code)]
    pub fn format_status<T: Serialize>(&self, status: &T, title: &str) -> String {
        match self {
            OutputFormatter::Json(f) => f.format_status(status, title),
            OutputFormatter::Human(f) => f.format_status(status, title),
        }
    }
}

/// JSON output formatter
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format_success(&self, message: &str, data: Option<Value>) -> String {
        let mut result = serde_json::json!({
            "status": "success",
            "message": message
        });

        if let Some(data) = data {
            result["data"] = data;
        }

        serde_json::to_string_pretty(&result).unwrap_or_else(|_| message.to_string())
    }

    #[allow(dead_code)]
    pub fn format_error(&self, error: &str) -> String {
        let result = serde_json::json!({
            "status": "error",
            "error": error
        });

        serde_json::to_string_pretty(&result).unwrap_or_else(|_| error.to_string())
    }

    #[allow(dead_code)]
    pub fn format_list<T: Serialize>(&self, items: &[T], title: &str) -> String {
        let result = serde_json::json!({
            "title": title,
            "count": items.len(),
            "items": items
        });

        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| format!("{}: {} items", title, items.len()))
    }

    #[allow(dead_code)]
    pub fn format_status<T: Serialize>(&self, status: &T, title: &str) -> String {
        let result = serde_json::json!({
            "title": title,
            "status": status
        });

        serde_json::to_string_pretty(&result).unwrap_or_else(|_| title.to_string())
    }
}

/// Human-readable output formatter
pub struct HumanFormatter;

impl HumanFormatter {
    pub fn format_success(&self, message: &str, data: Option<Value>) -> String {
        if let Some(data) = data {
            format!(
                "✅ {}\n\n{}",
                message,
                serde_json::to_string_pretty(&data).unwrap_or_default()
            )
        } else {
            format!("✅ {}", message)
        }
    }

    #[allow(dead_code)]
    pub fn format_error(&self, error: &str) -> String {
        format!("❌ Error: {}", error)
    }

    #[allow(dead_code)]
    pub fn format_list<T: Serialize>(&self, items: &[T], title: &str) -> String {
        let mut output = format!("📋 {} ({} items):\n", title, items.len());

        for (i, item) in items.iter().enumerate() {
            if let Ok(json) = serde_json::to_value(item) {
                output.push_str(&format!("  {}. {}\n", i + 1, format_item(&json)));
            }
        }

        output
    }

    #[allow(dead_code)]
    pub fn format_status<T: Serialize>(&self, status: &T, title: &str) -> String {
        format!(
            "📊 {}:\n{}",
            title,
            serde_json::to_string_pretty(status).unwrap_or_default()
        )
    }
}

/// Format a JSON value as a human-readable string
#[allow(dead_code)]
fn format_item(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(obj) => {
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                if let Some(status) = obj.get("status").and_then(|v| v.as_str()) {
                    format!("{} ({})", name, status)
                } else {
                    name.to_string()
                }
            } else if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                id.to_string()
            } else {
                serde_json::to_string(value).unwrap_or_default()
            }
        }
        _ => value.to_string(),
    }
}

/// Create a formatter based on output format preference
pub fn create_formatter(json: bool) -> OutputFormatter {
    if json {
        OutputFormatter::Json(JsonFormatter)
    } else {
        OutputFormatter::Human(HumanFormatter)
    }
}
