/// Search Agent implementation for ccswarm
///
/// This module provides a specialized search agent that integrates with gemini CLI
/// to handle web search requests from other agents and Master Claude.
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::agent::{AgentStatus, TaskResult};
use crate::coordination::{AgentMessage, CoordinationBus, CoordinationType};
use crate::identity::AgentRole;

/// Search request message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// ID of the requesting agent
    pub requesting_agent: String,
    /// Search query
    pub query: String,
    /// Maximum number of results to return
    pub max_results: Option<usize>,
    /// Optional filters (e.g., date range, domain)
    pub filters: Option<SearchFilters>,
    /// Context about why the search is needed
    pub context: Option<String>,
}

/// Search filters for more targeted results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Restrict to specific domains
    pub domains: Option<Vec<String>>,
    /// Date range filter (e.g., "past week", "past month")
    pub date_range: Option<String>,
    /// Language filter
    pub language: Option<String>,
    /// File type filter (e.g., "pdf", "doc")
    pub file_type: Option<String>,
}

/// Search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the result
    pub title: String,
    /// URL of the result
    pub url: String,
    /// Snippet/summary of the content
    pub snippet: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: Option<f32>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Response to a search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Original request ID
    pub request_id: String,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results found
    pub total_results: usize,
    /// Search query used (may be refined from original)
    pub query_used: String,
    /// Any errors or warnings
    pub warnings: Vec<String>,
}

/// Search Agent that handles web search requests
pub struct SearchAgent {
    /// Agent ID
    pub agent_id: String,
    /// Current status
    pub status: AgentStatus,
    /// Coordination bus for communication
    pub coordination_bus: Arc<CoordinationBus>,
    /// Active search requests
    active_requests: Arc<RwLock<Vec<SearchRequest>>>,
    /// Path to gemini CLI (defaults to "gemini" in PATH)
    gemini_path: String,
    /// Optional Sangha participant for autonomous decision-making
    sangha_participant: Option<
        Arc<
            tokio::sync::Mutex<
                Box<dyn crate::sangha::search_agent_participant::SanghaParticipant + Send>,
            >,
        >,
    >,
}

impl SearchAgent {
    /// Create a new search agent
    pub fn new(agent_id: String, coordination_bus: Arc<CoordinationBus>) -> Self {
        Self {
            agent_id,
            status: AgentStatus::Initializing,
            coordination_bus,
            active_requests: Arc::new(RwLock::new(Vec::new())),
            gemini_path: "gemini".to_string(),
            sangha_participant: None,
        }
    }

    /// Create a new search agent with custom gemini path
    pub fn with_gemini_path(
        agent_id: String,
        coordination_bus: Arc<CoordinationBus>,
        gemini_path: String,
    ) -> Self {
        Self {
            agent_id,
            status: AgentStatus::Initializing,
            coordination_bus,
            active_requests: Arc::new(RwLock::new(Vec::new())),
            gemini_path,
            sangha_participant: None,
        }
    }

    /// Enable Sangha participation for autonomous decision-making
    pub fn enable_sangha_participation(&mut self) {
        let participant = crate::sangha::search_agent_participant::create_search_agent_participant(
            self.agent_id.clone(),
            self.coordination_bus.clone(),
        );
        self.sangha_participant = Some(Arc::new(tokio::sync::Mutex::new(participant)));
    }

    /// Start Sangha monitoring in background
    pub async fn start_sangha_monitoring(&self, sangha: Arc<crate::sangha::Sangha>) -> Result<()> {
        if let Some(participant) = &self.sangha_participant {
            let participant_clone = participant.clone();
            let sangha_clone = sangha.clone();

            // Spawn background task for monitoring
            tokio::spawn(async move {
                let mut participant = participant_clone.lock().await;
                if let Err(e) = participant.monitor_proposals(sangha_clone).await {
                    error!("Sangha monitoring error: {}", e);
                }
            });

            info!(
                "Started Sangha monitoring for search agent {}",
                self.agent_id
            );
        } else {
            warn!(
                "Sangha participation not enabled for search agent {}",
                self.agent_id
            );
        }

        Ok(())
    }

    /// Initialize the search agent
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing search agent: {}", self.agent_id);

        // Verify gemini CLI is available
        self.verify_gemini_cli().await?;

        // Register with coordination bus
        self.register_with_coordination_bus().await?;

        self.status = AgentStatus::Available;
        info!("Search agent {} initialized successfully", self.agent_id);

        Ok(())
    }

    /// Verify gemini CLI is installed and accessible
    async fn verify_gemini_cli(&self) -> Result<()> {
        let output = Command::new(&self.gemini_path)
            .arg("--version")
            .output()
            .await
            .context("Failed to execute gemini CLI - is it installed?")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Gemini CLI verification failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Gemini CLI verified successfully");
        Ok(())
    }

    /// Register the search agent with the coordination bus
    async fn register_with_coordination_bus(&self) -> Result<()> {
        let registration = AgentMessage::Registration {
            agent_id: self.agent_id.clone(),
            capabilities: vec![
                "web_search".to_string(),
                "filtered_search".to_string(),
                "multi_result_search".to_string(),
            ],
            metadata: serde_json::json!({
                "agent_type": "search",
                "provider": "gemini",
                "max_concurrent_searches": 5,
            }),
        };

        self.coordination_bus
            .send_message(registration)
            .await
            .context("Failed to register with coordination bus")?;

        Ok(())
    }

    /// Start listening for search requests
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Search agent {} starting to listen for requests",
            self.agent_id
        );
        self.status = AgentStatus::Available;

        loop {
            // Check for messages from coordination bus
            if let Some(message) = self.coordination_bus.try_receive_message() {
                if let Err(e) = self.handle_message(message).await {
                    error!("Error handling message: {}", e);
                }
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Handle incoming messages
    async fn handle_message(&mut self, message: AgentMessage) -> Result<()> {
        match message {
            AgentMessage::Coordination {
                from_agent,
                to_agent,
                message_type,
                payload,
            } if to_agent == self.agent_id || to_agent == "search" => match message_type {
                CoordinationType::Custom(msg_type) if msg_type == "search_request" => {
                    let request: SearchRequest = serde_json::from_value(payload)
                        .context("Failed to parse search request")?;
                    self.handle_search_request(from_agent, request).await?;
                }
                _ => {
                    warn!(
                        "Received unknown coordination message type: {:?}",
                        message_type
                    );
                }
            },
            AgentMessage::TaskAssignment {
                task_id,
                agent_id,
                task_data,
            } if agent_id == self.agent_id => {
                self.handle_task_assignment(task_id, task_data).await?;
            }
            _ => {
                // Ignore messages not for this agent
            }
        }

        Ok(())
    }

    /// Handle a search request
    async fn handle_search_request(
        &mut self,
        requesting_agent: String,
        mut request: SearchRequest,
    ) -> Result<()> {
        info!(
            "Received search request from {}: {}",
            requesting_agent, request.query
        );

        // Add to active requests
        {
            let mut active = self.active_requests.write().await;
            request.requesting_agent = requesting_agent.clone();
            active.push(request.clone());
        }

        self.status = AgentStatus::Working;

        // Execute the search
        let results = self.execute_search(&request).await?;

        // Send response back to requesting agent
        let response = SearchResponse {
            request_id: uuid::Uuid::new_v4().to_string(),
            results: results.clone(),
            total_results: results.len(),
            query_used: request.query.clone(),
            warnings: vec![],
        };

        self.send_search_response(&requesting_agent, response)
            .await?;

        // Remove from active requests
        {
            let mut active = self.active_requests.write().await;
            active.retain(|r| r.requesting_agent != requesting_agent || r.query != request.query);
        }

        self.status = AgentStatus::Available;

        Ok(())
    }

    /// Execute a search using gemini CLI
    pub async fn execute_search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>> {
        let mut cmd = Command::new(&self.gemini_path);
        cmd.arg("search").arg(&request.query);

        // Add filters if provided
        if let Some(filters) = &request.filters {
            if let Some(domains) = &filters.domains {
                for domain in domains {
                    cmd.arg("--site").arg(domain);
                }
            }
            if let Some(date_range) = &filters.date_range {
                cmd.arg("--time").arg(date_range);
            }
            if let Some(language) = &filters.language {
                cmd.arg("--lang").arg(language);
            }
            if let Some(file_type) = &filters.file_type {
                cmd.arg("--filetype").arg(file_type);
            }
        }

        // Set max results
        let max_results = request.max_results.unwrap_or(10);
        cmd.arg("--max-results").arg(max_results.to_string());

        // Execute command with streaming output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .context("Failed to spawn gemini search process")?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        let mut results = Vec::new();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Parse gemini output (assuming JSON lines format)
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON result
            match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(json) => {
                    if let Ok(result) = self.parse_gemini_result(json) {
                        results.push(result);
                    }
                }
                Err(_) => {
                    // If not JSON, might be plain text format
                    if let Some(result) = self.parse_plain_text_result(&line) {
                        results.push(result);
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().await?;
        if !status.success() {
            warn!("Gemini search process exited with non-zero status");
        }

        info!("Search completed with {} results", results.len());
        Ok(results)
    }

    /// Parse a JSON result from gemini
    fn parse_gemini_result(&self, json: serde_json::Value) -> Result<SearchResult> {
        Ok(SearchResult {
            title: json["title"].as_str().unwrap_or("Untitled").to_string(),
            url: json["url"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing URL in result"))?
                .to_string(),
            snippet: json["snippet"].as_str().unwrap_or("").to_string(),
            relevance_score: json["score"].as_f64().map(|s| s as f32),
            metadata: Some(json),
        })
    }

    /// Parse a plain text result (fallback)
    fn parse_plain_text_result(&self, line: &str) -> Option<SearchResult> {
        // Simple parsing for plain text format
        // Format: "Title | URL | Snippet"
        let parts: Vec<&str> = line.split(" | ").collect();
        if parts.len() >= 2 {
            Some(SearchResult {
                title: parts[0].to_string(),
                url: parts[1].to_string(),
                snippet: parts.get(2).unwrap_or(&"").to_string(),
                relevance_score: None,
                metadata: None,
            })
        } else {
            None
        }
    }

    /// Send search response back to requesting agent
    async fn send_search_response(
        &self,
        requesting_agent: &str,
        response: SearchResponse,
    ) -> Result<()> {
        let message = AgentMessage::Coordination {
            from_agent: self.agent_id.clone(),
            to_agent: requesting_agent.to_string(),
            message_type: CoordinationType::Custom("search_response".to_string()),
            payload: serde_json::to_value(response)?,
        };

        self.coordination_bus
            .send_message(message)
            .await
            .context("Failed to send search response")?;

        Ok(())
    }

    /// Handle task assignment (alternative to direct search requests)
    async fn handle_task_assignment(
        &mut self,
        task_id: String,
        task_data: serde_json::Value,
    ) -> Result<()> {
        // Extract search query from task data
        let query = task_data["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query in task data"))?;

        let request = SearchRequest {
            requesting_agent: "task_system".to_string(),
            query: query.to_string(),
            max_results: task_data["max_results"].as_u64().map(|n| n as usize),
            filters: None,
            context: task_data["context"].as_str().map(|s| s.to_string()),
        };

        // Execute search
        let results = self.execute_search(&request).await?;

        // Send task completion
        let task_result = TaskResult {
            success: true,
            output: serde_json::json!({
                "results": results,
                "total_found": results.len(),
                "query": query,
            }),
            error: None,
            duration: std::time::Duration::from_secs(1), // Placeholder
        };

        let completion = AgentMessage::TaskCompleted {
            agent_id: self.agent_id.clone(),
            task_id,
            result: task_result,
        };

        self.coordination_bus.send_message(completion).await?;

        Ok(())
    }
}

/// Create search role for AgentRole enum
pub fn search_agent_role() -> AgentRole {
    AgentRole::Search {
        technologies: vec![
            "Gemini CLI".to_string(),
            "Web Search".to_string(),
            "Information Retrieval".to_string(),
        ],
        responsibilities: vec![
            "Web Search".to_string(),
            "Information Gathering".to_string(),
            "Result Filtering".to_string(),
            "Query Optimization".to_string(),
        ],
        boundaries: vec![
            "No code implementation".to_string(),
            "No direct file modifications".to_string(),
            "Read-only information gathering".to_string(),
            "No execution of found code".to_string(),
        ],
    }
}

/// Trait for search-capable agents
#[async_trait]
pub trait SearchCapable {
    /// Perform a web search
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;

    /// Perform a filtered search
    async fn search_with_filters(
        &self,
        query: &str,
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_agent_creation() {
        let coordination_bus = Arc::new(CoordinationBus::new().await.unwrap());
        let agent = SearchAgent::new("search-agent-1".to_string(), coordination_bus);

        assert_eq!(agent.agent_id, "search-agent-1");
        assert_eq!(agent.status, AgentStatus::Initializing);
    }

    #[tokio::test]
    async fn test_search_request_parsing() {
        let request_json = serde_json::json!({
            "requesting_agent": "frontend-agent",
            "query": "React hooks best practices",
            "max_results": 5,
            "filters": {
                "domains": ["reactjs.org", "developer.mozilla.org"],
                "date_range": "past month"
            }
        });

        let request: SearchRequest = serde_json::from_value(request_json).unwrap();
        assert_eq!(request.query, "React hooks best practices");
        assert_eq!(request.max_results, Some(5));
        assert!(request.filters.is_some());
    }

    #[tokio::test]
    async fn test_search_result_creation() {
        let result = SearchResult {
            title: "React Hooks Documentation".to_string(),
            url: "https://reactjs.org/docs/hooks-intro.html".to_string(),
            snippet: "Hooks are a new addition in React 16.8...".to_string(),
            relevance_score: Some(0.95),
            metadata: None,
        };

        assert_eq!(result.title, "React Hooks Documentation");
        assert_eq!(result.relevance_score, Some(0.95));
    }
}
