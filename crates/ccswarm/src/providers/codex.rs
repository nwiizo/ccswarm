use crate::providers::{ExecutionResult, ProviderTrait};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub api_key: String,
    pub model: Option<String>,
}

pub struct CodexExecutor {
    config: CodexConfig,
}

impl CodexExecutor {
    pub fn new(config: CodexConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ProviderTrait for CodexExecutor {
    async fn execute(&self, prompt: &str) -> Result<ExecutionResult> {
        Ok(ExecutionResult {
            output: format!("Codex stub response for: {}", prompt),
            success: true,
            error: None,
        })
    }

    fn name(&self) -> &str {
        "codex"
    }
}
