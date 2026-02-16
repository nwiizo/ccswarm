use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;
use tracing::{info, warn};

use crate::agent::{Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::identity::AgentIdentity;

/// Simple persistent Claude session
#[derive(Debug, Clone)]
pub struct PersistentClaudeSession {
    /// Agent identity
    pub identity: AgentIdentity,

    /// Working directory
    pub working_dir: PathBuf,

    /// Claude configuration
    pub claude_config: ClaudeConfig,

    /// Session ID
    pub session_id: String,

    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

impl PersistentClaudeSession {
    /// Create new persistent session
    pub async fn new(
        identity: AgentIdentity,
        working_dir: PathBuf,
        claude_config: ClaudeConfig,
    ) -> Result<Self> {
        let session_id = format!("session-{}-{}", identity.agent_id, uuid::Uuid::new_v4());

        let mut env_vars = crate::utils::common::collections::new_hashmap();
        env_vars.insert("AGENT_ID".to_string(), identity.agent_id.clone());
        env_vars.insert(
            "AGENT_TYPE".to_string(),
            identity.specialization.name().to_string(),
        );
        env_vars.insert("SESSION_ID".to_string(), session_id.clone());

        Ok(Self {
            identity,
            working_dir,
            claude_config,
            session_id,
            env_vars,
        })
    }

    /// Initialize session
    pub async fn initialize(&mut self) -> Result<()> {
        info!("🚀 Initializing session: {}", self.session_id);

        // Create working directory if needed
        tokio::fs::create_dir_all(&self.working_dir).await?;

        // Skip identity prompt when using real API to avoid wasting API calls
        if !self.claude_config.use_real_api {
            let identity_prompt = format!(
                "You are a {} agent. Your workspace is {}. Your specialization is in {}.",
                self.identity.specialization.name(),
                self.working_dir.display(),
                self.identity.specialization.name()
            );
            self.execute_prompt(&identity_prompt).await?;
        }

        Ok(())
    }

    /// Execute task
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        info!("📋 Executing task {}: {}", task.id, task.description);

        let start = Instant::now();

        // Generate task prompt
        let prompt = self.generate_task_prompt(&task);

        // Execute with Claude
        let response = self.execute_prompt(&prompt).await?;

        let duration = start.elapsed();

        // Create task result
        Ok(TaskResult {
            success: true,
            output: serde_json::json!({
                "task_id": task.id,
                "agent_id": self.identity.agent_id.clone(),
                "response": response,
                "files_created": [],
                "files_modified": [],
            }),
            error: None,
            duration,
        })
    }

    /// Execute command
    pub async fn execute_command(&mut self, command: &str) -> Result<String> {
        info!("🔧 Executing command: {}", command);

        // Execute shell command in working directory
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            warn!("Command failed: {}", stderr);
        }

        Ok(format!("{}\n{}", stdout, stderr))
    }

    /// Execute prompt with Claude
    async fn execute_prompt(&self, prompt: &str) -> Result<String> {
        if self.claude_config.use_real_api {
            return self.execute_prompt_real_api(prompt).await;
        }

        info!("🤖 Claude prompt (simulated): {}", prompt);

        // Simulate response based on task content
        let response = if prompt.contains("React") || prompt.contains("frontend") {
            "Created React components with TypeScript. Files: App.tsx, TodoList.tsx, TodoItem.tsx"
        } else if prompt.contains("API") || prompt.contains("backend") {
            "Created Express API with endpoints: GET /todos, POST /todos, PUT /todos/:id, DELETE /todos/:id"
        } else if prompt.contains("test") {
            "Created test suites with Jest. Coverage: 85%"
        } else {
            "Task completed successfully"
        };

        Ok(response.to_string())
    }

    /// Execute prompt using the real Claude API
    async fn execute_prompt_real_api(&self, prompt: &str) -> Result<String> {
        use crate::providers::claude_api::{ClaudeApiClient, Message};

        info!("🤖 Claude prompt (real API): {}", prompt);

        let client = ClaudeApiClient::new(None)?;

        let system_prompt = format!(
            "You are a {} specialist agent. Workspace: {}.",
            self.identity.specialization.name(),
            self.working_dir.display()
        );

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let response = client
            .create_completion(
                &self.claude_config.model,
                messages,
                4096,
                Some(0.0),
                Some(system_prompt),
            )
            .await?;

        let text = response
            .content
            .into_iter()
            .map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }

    /// Generate task prompt
    fn generate_task_prompt(&self, task: &Task) -> String {
        format!(
            "As a {} agent, please complete the following task:\n\n\
            Task: {}\n\
            Priority: {:?}\n\
            Type: {:?}\n\
            {}\n\n\
            Please implement this task according to best practices for {}.",
            self.identity.specialization.name(),
            task.description,
            task.priority,
            task.task_type,
            task.details.as_deref().unwrap_or(""),
            self.identity.specialization.name()
        )
    }
}
