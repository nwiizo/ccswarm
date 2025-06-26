/// Example demonstrating the Search Agent usage in ccswarm
use anyhow::Result;
use ccswarm::agent::search_agent::{SearchAgent, SearchFilters, SearchRequest};
use ccswarm::coordination::{AgentMessage, CoordinationBus, CoordinationType};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize coordination bus
    let coordination_bus = Arc::new(CoordinationBus::new().await?);

    // Create search agent
    let mut search_agent =
        SearchAgent::new("search-agent-001".to_string(), coordination_bus.clone());

    // Initialize the agent
    search_agent.initialize().await?;

    // Example 1: Simple search request
    let simple_request = SearchRequest {
        requesting_agent: "master-claude".to_string(),
        query: "Rust async programming best practices".to_string(),
        max_results: Some(5),
        filters: None,
        context: Some("Looking for modern async patterns in Rust".to_string()),
    };

    // Send search request via coordination bus
    let message = AgentMessage::Coordination {
        from_agent: "master-claude".to_string(),
        to_agent: "search-agent-001".to_string(),
        message_type: CoordinationType::Custom("search_request".to_string()),
        payload: serde_json::to_value(simple_request)?,
    };

    coordination_bus.send_message(message).await?;

    // Example 2: Filtered search request
    let filtered_request = SearchRequest {
        requesting_agent: "frontend-agent".to_string(),
        query: "React hooks useState useEffect".to_string(),
        max_results: Some(10),
        filters: Some(SearchFilters {
            domains: Some(vec![
                "reactjs.org".to_string(),
                "developer.mozilla.org".to_string(),
            ]),
            date_range: Some("past month".to_string()),
            language: Some("en".to_string()),
            file_type: None,
        }),
        context: Some("Need documentation on React hooks".to_string()),
    };

    let filtered_message = AgentMessage::Coordination {
        from_agent: "frontend-agent".to_string(),
        to_agent: "search-agent-001".to_string(),
        message_type: CoordinationType::Custom("search_request".to_string()),
        payload: serde_json::to_value(filtered_request)?,
    };

    coordination_bus.send_message(filtered_message).await?;

    // Start the search agent (this will run indefinitely)
    // In production, this would be spawned as a separate task
    tokio::spawn(async move {
        if let Err(e) = search_agent.start().await {
            eprintln!("Search agent error: {}", e);
        }
    });

    // Give some time for processing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    println!("Search agent example completed");
    Ok(())
}
