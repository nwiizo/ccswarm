use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_interfaces: Vec<AgentInterface>,
    #[serde(default)]
    pub capabilities: AgentCapabilities,
    pub version: String,
    #[serde(default)]
    pub skills: Vec<AgentSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInterface {
    pub url: String,
    pub protocol_binding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub push_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct A2AClient {
    endpoint: String,
    client: reqwest::Client,
}

impl A2AClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            endpoint: endpoint.trim().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_text(&self, text: &str) -> Result<A2AExecution> {
        let started = Instant::now();
        let request = SendMessageRequest::new(text);
        let response = self
            .client
            .post(format!("{}/message:send", self.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(reqwest::Error::without_url)
            .context("failed to send A2A message")?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .context("failed to decode A2A response")?;
        if !status.is_success() {
            anyhow::bail!("A2A endpoint returned {}: {}", status, body);
        }

        let output = extract_text(&body).unwrap_or_else(|| body.to_string());
        Ok(A2AExecution {
            output,
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct A2AExecution {
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageRequest {
    message: A2AMessage,
}

impl SendMessageRequest {
    fn new(text: &str) -> Self {
        Self {
            message: A2AMessage {
                role: "user".to_string(),
                parts: vec![A2APart::Text {
                    text: text.to_string(),
                }],
                message_id: uuid::Uuid::new_v4().to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct A2AMessage {
    role: String,
    parts: Vec<A2APart>,
    #[serde(rename = "messageId")]
    message_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum A2APart {
    Text { text: String },
}

fn extract_text(value: &Value) -> Option<String> {
    let mut fragments = Vec::new();
    collect_text(value, &mut fragments);
    if fragments.is_empty() {
        None
    } else {
        Some(fragments.join("\n"))
    }
}

fn collect_text(value: &Value, fragments: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get("text") {
                fragments.push(text.clone());
            }
            for value in map.values() {
                collect_text(value, fragments);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_text(value, fragments);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text_from_nested_a2a_response() {
        let body = serde_json::json!({
            "task": {
                "artifacts": [{
                    "parts": [{"kind": "text", "text": "done"}]
                }]
            }
        });
        assert_eq!(extract_text(&body).as_deref(), Some("done"));
    }

    #[test]
    fn normalizes_endpoint_whitespace_and_trailing_slashes() {
        let client = A2AClient::new("  https://example.test/a2a/  ");
        assert_eq!(client.endpoint, "https://example.test/a2a");
    }

    #[tokio::test]
    async fn connection_errors_do_not_expose_endpoint_credentials() {
        let client = A2AClient::new("http://user:secret@127.0.0.1:0");
        let error = client
            .send_text("test")
            .await
            .expect_err("port zero should reject the request");

        assert!(!format!("{error:#}").contains("secret"));
    }
}
