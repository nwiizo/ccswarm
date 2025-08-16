use crate::providers::{ExecutionResult, ProviderTrait};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiderConfig {
    pub command: String,
    pub api_key: Option<String>,
}

pub struct AiderExecutor {
    _config: AiderConfig,
}

impl AiderExecutor {
    pub fn new(config: AiderConfig) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl ProviderTrait for AiderExecutor {
    async fn execute(&self, prompt: &str) -> Result<ExecutionResult> {
        Ok(ExecutionResult {
            output: format!("Aider stub response for: {}", prompt),
            success: true,
            error: None,
        })
    }

    fn name(&self) -> &str {
        "aider"
    }
}
