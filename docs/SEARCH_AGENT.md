# Search Agent Documentation

## Overview

The Search Agent is a specialized component in the ccswarm system that provides web search capabilities to other agents and Master Claude. It integrates with the Gemini CLI tool to perform intelligent information gathering, helping agents make informed decisions and find relevant resources.

## Architecture

### System Position

The Search Agent operates as a specialized agent within the ccswarm ecosystem:

```
┌─────────────────────────────────────────────────────────────┐
│                    Master Claude (Orchestrator)              │
│  - Delegates search tasks to Search Agent                    │
│  - Receives research results for decision-making             │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┬─────────────┬────────────┬───────────┐
        │                           │             │            │           │
┌───────▼────────┐  ┌──────────────▼──┐  ┌──────▼─────┐  ┌───▼────┐  ┌───▼──────┐
│Frontend Agent  │  │Backend Agent    │  │DevOps Agent│  │QA Agent│  │Search    │
│                │  │                 │  │            │  │        │  │Agent     │
│   ←────────────┼──┼─────────────────┼──┼────────────┼──┼────────┼──→         │
│   Can request  │  │                 │  │            │  │        │  │Provides  │
│   searches     │  │                 │  │            │  │        │  │research  │
└────────────────┘  └─────────────────┘  └────────────┘  └────────┘  └──────────┘
```

### Core Components

#### 1. Search Agent (`crates/ccswarm/src/agent/search_agent.rs`)

The main agent implementation that:
- Handles search requests from other agents
- Integrates with Gemini CLI for web searches
- Manages search filters and result parsing
- Participates in the coordination bus

#### 2. Sangha Participant (`crates/ccswarm/src/sangha/search_agent_participant.rs`)

Enables autonomous decision-making by:
- Monitoring Sangha proposals
- Conducting research on proposals
- Analyzing search results to form opinions
- Casting informed votes based on evidence
- Identifying knowledge gaps and proposing initiatives

### Key Data Structures

```rust
// Search request from agents
pub struct SearchRequest {
    pub requesting_agent: String,
    pub query: String,
    pub max_results: Option<usize>,
    pub filters: Option<SearchFilters>,
    pub context: Option<String>,
}

// Search filters for targeted results
pub struct SearchFilters {
    pub domains: Option<Vec<String>>,
    pub date_range: Option<String>,
    pub language: Option<String>,
    pub file_type: Option<String>,
}

// Search result structure
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance_score: Option<f32>,
    pub metadata: Option<serde_json::Value>,
}
```

## Configuration

### Basic Setup

1. **Install Gemini CLI**:
   ```bash
   # Install gemini (example - adjust based on actual tool)
   cargo install gemini-cli
   # or
   brew install gemini
   ```

2. **Initialize Search Agent**:
   ```rust
   let search_agent = SearchAgent::new(
       "search-agent-1".to_string(),
       coordination_bus
   );
   ```

3. **Custom Gemini Path** (if needed):
   ```rust
   let search_agent = SearchAgent::with_gemini_path(
       "search-agent-1".to_string(),
       coordination_bus,
       "/usr/local/bin/gemini".to_string()
   );
   ```

### Environment Variables

- `GEMINI_API_KEY`: API key for Gemini service (if required)
- `CCSWARM_SEARCH_MAX_CONCURRENT`: Maximum concurrent searches (default: 5)
- `CCSWARM_SEARCH_TIMEOUT`: Search timeout in seconds (default: 30)

## Usage

### 1. Direct Search Requests

Other agents can request searches via the coordination bus:

```rust
// From a frontend agent needing React information
let request = SearchRequest {
    requesting_agent: "frontend-agent".to_string(),
    query: "React 18 concurrent features".to_string(),
    max_results: Some(10),
    filters: Some(SearchFilters {
        domains: Some(vec!["reactjs.org".to_string(), "beta.reactjs.org".to_string()]),
        date_range: Some("past month".to_string()),
        language: Some("en".to_string()),
        file_type: None,
    }),
    context: Some("Implementing concurrent rendering in our app".to_string()),
};

// Send via coordination bus
let message = AgentMessage::Coordination {
    from_agent: "frontend-agent".to_string(),
    to_agent: "search-agent".to_string(),
    message_type: CoordinationType::Custom("search_request".to_string()),
    payload: serde_json::to_value(&request)?,
};

coordination_bus.send_message(message).await?;
```

### 2. Task Assignment from Master Claude

Master Claude can assign search tasks:

```rust
// Task data for search
let task_data = json!({
    "query": "Rust async runtime performance comparison",
    "max_results": 15,
    "context": "Choosing the best async runtime for our backend"
});

// Master Claude assigns the task
let task_assignment = AgentMessage::TaskAssignment {
    task_id: "task-123".to_string(),
    agent_id: "search-agent".to_string(),
    task_data,
};
```

### 3. Handling Search Responses

Agents receive search results via the coordination bus:

```rust
// Listen for search responses
match message {
    AgentMessage::Coordination {
        from_agent,
        message_type: CoordinationType::Custom(msg_type),
        payload,
        ..
    } if msg_type == "search_response" => {
        let response: SearchResponse = serde_json::from_value(payload)?;
        
        // Process search results
        for result in response.results {
            println!("Found: {} - {}", result.title, result.url);
            println!("Relevance: {:?}", result.relevance_score);
        }
    }
    _ => {}
}
```

## Sangha Integration

### Autonomous Proposal Research

The Search Agent actively participates in Sangha governance:

1. **Enable Sangha Participation**:
   ```rust
   search_agent.enable_sangha_participation();
   search_agent.start_sangha_monitoring(sangha).await?;
   ```

2. **Automatic Proposal Analysis**:
   - Monitors new proposals
   - Extracts key terms and concepts
   - Conducts targeted research
   - Analyzes results to form opinions

3. **Evidence Submission**:
   ```rust
   // The agent automatically submits research findings
   let evidence = Evidence {
       evidence_type: EvidenceType::Research(ResearchEvidence {
           findings: search_results,
           methodology: "Web search and content analysis".to_string(),
           confidence_level: 0.85,
       }),
       summary: "Research on GraphQL implementation patterns".to_string(),
       relevance_score: 0.9,
       sources: vec!["https://graphql.org/learn/".to_string()],
       submitted_at: Utc::now(),
   };
   ```

4. **Informed Voting**:
   - Votes based on research findings
   - Provides reasoning with evidence
   - Abstains when insufficient information

### Knowledge Gap Detection

The Search Agent identifies areas needing research:

```rust
let identified_need = IdentifiedNeed {
    need_type: NeedType::KnowledgeGap,
    description: "No research on WebAssembly integration".to_string(),
    urgency: Urgency::High,
    supporting_data: json!({
        "affected_proposals": 3,
        "current_evidence": 0,
    }),
};

// Automatically creates proposals to address gaps
search_agent.propose_initiative(identified_need, sangha).await?;
```

## Integration Examples

### Example 1: Frontend Agent Research Flow

```rust
// Frontend agent needs to research a new UI pattern
async fn research_ui_pattern(
    coordination_bus: Arc<CoordinationBus>,
    pattern_name: &str,
) -> Result<Vec<SearchResult>> {
    // Create search request
    let request = SearchRequest {
        requesting_agent: "frontend-agent".to_string(),
        query: format!("{} implementation examples", pattern_name),
        max_results: Some(10),
        filters: Some(SearchFilters {
            domains: Some(vec![
                "codepen.io".to_string(),
                "codesandbox.io".to_string(),
                "github.com".to_string(),
            ]),
            date_range: Some("past year".to_string()),
            language: Some("en".to_string()),
            file_type: None,
        }),
        context: Some(format!("Implementing {} in our React app", pattern_name)),
    };

    // Send request
    let message = AgentMessage::Coordination {
        from_agent: "frontend-agent".to_string(),
        to_agent: "search-agent".to_string(),
        message_type: CoordinationType::Custom("search_request".to_string()),
        payload: serde_json::to_value(&request)?,
    };

    coordination_bus.send_message(message).await?;

    // Wait for response (simplified - in practice use proper async handling)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Process response
    // ... handle search results ...
    
    Ok(vec![])
}
```

### Example 2: Master Claude Delegating Research

```rust
// Master Claude identifies a task requiring research
async fn delegate_research_task(
    master_claude: &mut MasterClaude,
    task_description: &str,
) -> Result<()> {
    let task = Task {
        id: Uuid::new_v4(),
        task_type: TaskType::Research,
        description: task_description.to_string(),
        priority: TaskPriority::High,
        status: TaskStatus::Pending,
        assigned_agent: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        parent_task: None,
        tags: vec!["research".to_string()],
        context: HashMap::new(),
    };

    // Master Claude analyzes and delegates to Search Agent
    let delegation = master_claude.delegate_task(&task).await?;
    
    if delegation.recommended_agent == AgentRole::Search {
        println!("Task delegated to Search Agent with {:.0}% confidence", 
                 delegation.confidence * 100.0);
    }

    Ok(())
}
```

### Example 3: Multi-Agent Collaborative Research

```rust
// Multiple agents collaborating on research
async fn collaborative_research(
    coordination_bus: Arc<CoordinationBus>,
) -> Result<()> {
    // Backend agent requests API patterns
    let backend_request = SearchRequest {
        requesting_agent: "backend-agent".to_string(),
        query: "GraphQL vs REST API comparison 2024".to_string(),
        max_results: Some(10),
        filters: None,
        context: Some("Choosing API architecture".to_string()),
    };

    // Frontend agent requests UI libraries
    let frontend_request = SearchRequest {
        requesting_agent: "frontend-agent".to_string(),
        query: "React component libraries for GraphQL".to_string(),
        max_results: Some(10),
        filters: None,
        context: Some("Building GraphQL UI".to_string()),
    };

    // Send both requests
    for request in [backend_request, frontend_request] {
        let message = AgentMessage::Coordination {
            from_agent: request.requesting_agent.clone(),
            to_agent: "search-agent".to_string(),
            message_type: CoordinationType::Custom("search_request".to_string()),
            payload: serde_json::to_value(&request)?,
        };
        coordination_bus.send_message(message).await?;
    }

    Ok(())
}
```

## Best Practices

### 1. Query Optimization

- Be specific with search queries
- Use relevant keywords and technical terms
- Provide context to improve relevance

```rust
// Good: Specific and contextual
let query = "React Server Components hydration strategies 2024".to_string();

// Less effective: Too generic
let query = "React components".to_string();
```

### 2. Filter Usage

Use filters to improve result quality:

```rust
let filters = SearchFilters {
    // Target authoritative sources
    domains: Some(vec![
        "docs.rust-lang.org".to_string(),
        "rust-lang.github.io".to_string(),
    ]),
    // Recent information
    date_range: Some("past 6 months".to_string()),
    // Ensure readability
    language: Some("en".to_string()),
    // Get specific content types
    file_type: Some("md".to_string()), // For documentation
};
```

### 3. Result Processing

Handle results intelligently:

```rust
// Process results by relevance
let high_relevance_results: Vec<_> = response.results
    .into_iter()
    .filter(|r| r.relevance_score.unwrap_or(0.0) > 0.7)
    .collect();

// Extract key information
for result in high_relevance_results {
    // Parse metadata for additional insights
    if let Some(metadata) = result.metadata {
        // Extract publication date, author, etc.
    }
}
```

### 4. Error Handling

Always handle search failures gracefully:

```rust
match search_agent.execute_search(&request).await {
    Ok(results) => {
        if results.is_empty() {
            // Handle no results case
            log::warn!("No results found for query: {}", request.query);
            // Try alternative query or broaden search
        } else {
            // Process results
        }
    }
    Err(e) => {
        log::error!("Search failed: {}", e);
        // Implement fallback strategy
        // - Try cached results
        // - Use alternative search agent
        // - Notify requesting agent of failure
    }
}
```

## Performance Considerations

### 1. Concurrent Searches

The Search Agent handles multiple concurrent requests:

```rust
// Maximum concurrent searches (default: 5)
const MAX_CONCURRENT_SEARCHES: usize = 5;

// Requests beyond limit are queued
```

### 2. Caching

Consider implementing result caching:

```rust
// Cache recent searches to reduce API calls
struct SearchCache {
    entries: HashMap<String, (SearchResponse, Instant)>,
    ttl: Duration,
}

impl SearchCache {
    fn get(&self, query: &str) -> Option<&SearchResponse> {
        self.entries.get(query)
            .filter(|(_, timestamp)| timestamp.elapsed() < self.ttl)
            .map(|(response, _)| response)
    }
}
```

### 3. Rate Limiting

Respect API rate limits:

```rust
// Implement rate limiting
struct RateLimiter {
    requests_per_minute: u32,
    current_requests: Arc<AtomicU32>,
}
```

## Troubleshooting

### Common Issues

1. **Gemini CLI not found**:
   ```bash
   # Verify installation
   which gemini
   
   # Set custom path if needed
   export GEMINI_PATH=/usr/local/bin/gemini
   ```

2. **No search results**:
   - Check query formatting
   - Verify filters aren't too restrictive
   - Ensure Gemini API key is valid

3. **Timeout errors**:
   - Increase timeout setting
   - Reduce max_results
   - Check network connectivity

### Debug Logging

Enable detailed logging:

```bash
RUST_LOG=ccswarm::agent::search_agent=debug cargo run
```

### Monitoring

Monitor search agent performance:

```rust
// Check agent status
println!("Search Agent Status: {:?}", search_agent.status);

// View active requests
let active = search_agent.active_requests.read().await;
println!("Active searches: {}", active.len());
```

## Security Considerations

### 1. Input Validation

Always validate search queries:

```rust
fn validate_query(query: &str) -> Result<()> {
    if query.is_empty() {
        return Err(anyhow!("Empty query"));
    }
    if query.len() > 500 {
        return Err(anyhow!("Query too long"));
    }
    // Additional validation...
    Ok(())
}
```

### 2. Result Filtering

Filter potentially harmful content:

```rust
fn filter_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    results.into_iter()
        .filter(|r| !r.url.contains("malware"))
        .filter(|r| !r.snippet.contains("exploit"))
        .collect()
}
```

### 3. API Key Protection

Never expose API keys:

```rust
// Load from environment
let api_key = env::var("GEMINI_API_KEY")
    .expect("GEMINI_API_KEY must be set");

// Never log or display keys
```

## Future Enhancements

### Planned Features

1. **Multiple Search Providers**:
   - Google Custom Search API
   - Bing Search API
   - DuckDuckGo API
   - Academic search engines

2. **Advanced NLP**:
   - Query expansion
   - Semantic search
   - Result summarization
   - Entity extraction

3. **Machine Learning**:
   - Result ranking optimization
   - Query suggestion
   - Personalized filtering
   - Trend detection

4. **Enhanced Caching**:
   - Distributed cache
   - Semantic similarity matching
   - Partial result caching
   - Cache warming strategies

### Extension Points

The Search Agent is designed for extensibility:

```rust
// Custom search provider trait
#[async_trait]
trait SearchProvider {
    async fn search(&self, query: &str, filters: &SearchFilters) -> Result<Vec<SearchResult>>;
    fn name(&self) -> &str;
    fn supports_filters(&self) -> bool;
}

// Implement for new providers
struct GoogleSearchProvider { /* ... */ }
impl SearchProvider for GoogleSearchProvider { /* ... */ }
```

## API Reference

### Core Types

- `SearchAgent` - Main agent implementation
- `SearchRequest` - Request structure for searches
- `SearchFilters` - Optional search filters
- `SearchResult` - Individual search result
- `SearchResponse` - Complete response with results

### Key Methods

```rust
impl SearchAgent {
    // Create new search agent
    pub fn new(agent_id: String, coordination_bus: Arc<CoordinationBus>) -> Self;
    
    // Create with custom gemini path
    pub fn with_gemini_path(agent_id: String, coordination_bus: Arc<CoordinationBus>, gemini_path: String) -> Self;
    
    // Initialize the agent
    pub async fn initialize(&mut self) -> Result<()>;
    
    // Start listening for requests
    pub async fn start(&mut self) -> Result<()>;
    
    // Execute a search
    pub async fn execute_search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>>;
    
    // Enable Sangha participation
    pub fn enable_sangha_participation(&mut self);
    
    // Start Sangha monitoring
    pub async fn start_sangha_monitoring(&self, sangha: Arc<Sangha>) -> Result<()>;
}
```

### Message Types

- `search_request` - Request a web search
- `search_response` - Receive search results
- `search_error` - Search failure notification

## Conclusion

The Search Agent is a crucial component of the ccswarm system, providing intelligent information gathering capabilities that enhance decision-making across all agents. By integrating web search with the multi-agent architecture, ccswarm can stay informed about the latest technologies, best practices, and solutions.

For more information, see:
- [Architecture Documentation](./ARCHITECTURE.md)
- [Agent Development Guide](./AGENT_DEVELOPMENT.md)
- [Sangha System Documentation](./SANGHA.md)