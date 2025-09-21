use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

/// Claude API client for real API calls
pub struct ClaudeApiClient {
    client: Client,
    api_key: String,
    base_url: String,
}

/// Claude API message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Claude API request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeApiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Claude API response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeApiResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

impl ClaudeApiClient {
    /// Create a new Claude API client
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
            .context(
                "ANTHROPIC_API_KEY not found. Please set it in environment or provide in config.",
            )?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
        })
    }

    /// Create a completion using the Claude API
    pub async fn create_completion(
        &self,
        model: &str,
        messages: Vec<Message>,
        max_tokens: i32,
        temperature: Option<f32>,
        system: Option<String>,
    ) -> Result<ClaudeApiResponse> {
        let request = ClaudeApiRequest {
            model: model.to_string(),
            messages,
            max_tokens,
            temperature,
            system,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Claude API error ({}): {}",
                status,
                error_text
            ));
        }

        let api_response: ClaudeApiResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        Ok(api_response)
    }

    /// Create a simple completion with a single user message
    pub async fn simple_completion(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: i32,
    ) -> Result<String> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let response = self
            .create_completion(
                model,
                messages,
                max_tokens,
                Some(0.0), // Temperature 0 for deterministic responses
                None,
            )
            .await?;

        // Extract text from response
        let text = response
            .content
            .into_iter()
            .map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }
}

