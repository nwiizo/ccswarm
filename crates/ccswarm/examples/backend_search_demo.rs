/// Backend Agent Search Integration Demo
///
/// This example demonstrates how backend agents interact with the search agent
/// to solve real-world development challenges. Shows the complete workflow
/// from problem identification to solution implementation.
use anyhow::Result;
use ccswarm::agent::search_agent::{
    SearchAgent, SearchFilters, SearchRequest, SearchResponse, SearchResult,
};
use ccswarm::agent::{Agent, AgentRole, AgentStatus, Task, TaskPriority};
use ccswarm::coordination::{AgentMessage, CoordinationBus, CoordinationType};
use ccswarm::orchestrator::MasterClaude;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Simulated search response for demo purposes
fn create_mock_search_response(query: &str) -> SearchResponse {
    let results = match query {
        q if q.contains("database") && q.contains("optimization") => vec![
            SearchResult {
                title: "PostgreSQL Performance Tuning".to_string(),
                url: "https://postgresql.org/docs/current/performance-tips.html".to_string(),
                snippet: "Key techniques: proper indexing, query planning, vacuum optimization..."
                    .to_string(),
                relevance_score: Some(0.95),
                metadata: Some(json!({
                    "source": "official_docs",
                    "last_updated": "2024-01-15"
                })),
            },
            SearchResult {
                title: "Solving Slow Queries in PostgreSQL".to_string(),
                url: "https://dba.stackexchange.com/questions/slow-queries".to_string(),
                snippet: "Use EXPLAIN ANALYZE to identify bottlenecks, create partial indexes..."
                    .to_string(),
                relevance_score: Some(0.88),
                metadata: None,
            },
        ],
        q if q.contains("Redis") && q.contains("caching") => vec![SearchResult {
            title: "Redis Caching Strategies".to_string(),
            url: "https://redis.io/docs/manual/patterns/".to_string(),
            snippet: "Cache-aside, write-through, and write-behind patterns explained..."
                .to_string(),
            relevance_score: Some(0.92),
            metadata: Some(json!({"pattern_type": "architectural"})),
        }],
        q if q.contains("API") && q.contains("versioning") => vec![SearchResult {
            title: "REST API Versioning Best Practices".to_string(),
            url: "https://restfulapi.net/versioning/".to_string(),
            snippet: "URL versioning vs header versioning, backward compatibility...".to_string(),
            relevance_score: Some(0.85),
            metadata: None,
        }],
        _ => vec![SearchResult {
            title: "General Development Resource".to_string(),
            url: "https://developer.mozilla.org/".to_string(),
            snippet: "Comprehensive web development documentation...".to_string(),
            relevance_score: Some(0.70),
            metadata: None,
        }],
    };

    SearchResponse {
        request_id: uuid::Uuid::new_v4().to_string(),
        results,
        total_results: results.len(),
        query_used: query.to_string(),
        warnings: vec![],
    }
}

/// Backend agent workflow that uses search
struct BackendSearchWorkflow {
    agent: Agent,
    search_agent_id: String,
    coordination_bus: Arc<CoordinationBus>,
    pending_searches: Arc<RwLock<Vec<String>>>,
}

impl BackendSearchWorkflow {
    fn new(agent: Agent, search_agent_id: String, coordination_bus: Arc<CoordinationBus>) -> Self {
        Self {
            agent,
            search_agent_id,
            coordination_bus,
            pending_searches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process a task that requires search assistance
    async fn process_task_with_search(&mut self, task: &Task) -> Result<()> {
        info!("Backend agent processing task: {}", task.description);

        // Analyze task to determine if search is needed
        let search_query = self.analyze_task_for_search(task).await?;

        if let Some(query) = search_query {
            info!("Task requires search assistance. Query: {}", query);

            // Send search request
            let search_request = SearchRequest {
                requesting_agent: self.agent.name.clone(),
                query: query.clone(),
                max_results: Some(5),
                filters: Some(SearchFilters {
                    domains: None,
                    date_range: Some("past year".to_string()),
                    language: Some("en".to_string()),
                    file_type: None,
                }),
                context: Some(format!("Task: {}", task.description)),
            };

            self.send_search_request(search_request).await?;

            // Track pending search
            self.pending_searches.write().await.push(query);

            // Wait for response (in real implementation, this would be async)
            sleep(Duration::from_secs(2)).await;

            // Process search results
            let mock_response = create_mock_search_response(&query);
            self.process_search_results(&mock_response, task).await?;
        } else {
            info!("Task doesn't require search assistance, proceeding with implementation");
            self.implement_task_directly(task).await?;
        }

        Ok(())
    }

    /// Analyze task to determine if search is needed
    async fn analyze_task_for_search(&self, task: &Task) -> Result<Option<String>> {
        let description = task.description.to_lowercase();

        // Keywords that indicate search might be helpful
        let search_indicators = vec![
            (
                "optimize",
                "performance",
                "database optimization techniques",
            ),
            (
                "implement",
                "authentication",
                "authentication best practices",
            ),
            ("cache", "strategy", "caching patterns Redis Memcached"),
            ("api", "versioning", "REST API versioning strategies"),
            ("error", "handling", "error handling patterns"),
            (
                "security",
                "vulnerabilities",
                "OWASP security best practices",
            ),
            ("scale", "performance", "horizontal scaling techniques"),
            ("microservices", "communication", "microservices patterns"),
        ];

        for (keyword1, keyword2, query) in search_indicators {
            if description.contains(keyword1) && description.contains(keyword2) {
                return Ok(Some(query.to_string()));
            }
        }

        // Check for explicit research tasks
        if description.contains("research") || description.contains("investigate") {
            let query = description
                .replace("research", "")
                .replace("investigate", "")
                .trim()
                .to_string();
            return Ok(Some(query));
        }

        Ok(None)
    }

    /// Send search request to search agent
    async fn send_search_request(&self, request: SearchRequest) -> Result<()> {
        let message = AgentMessage::Coordination {
            from_agent: self.agent.name.clone(),
            to_agent: self.search_agent_id.clone(),
            message_type: CoordinationType::Custom("search_request".to_string()),
            payload: serde_json::to_value(request)?,
        };

        self.coordination_bus.send_message(message).await?;
        info!("Sent search request to {}", self.search_agent_id);

        Ok(())
    }

    /// Process search results and apply to task
    async fn process_search_results(&self, response: &SearchResponse, task: &Task) -> Result<()> {
        info!("Processing {} search results", response.results.len());

        // Analyze results and extract key insights
        let mut insights = Vec::new();

        for result in &response.results {
            if let Some(score) = result.relevance_score {
                if score > 0.8 {
                    insights.push(format!(
                        "High relevance ({}): {} - {}",
                        score, result.title, result.snippet
                    ));
                }
            }

            debug!("Search result: {} ({})", result.title, result.url);
        }

        // Generate implementation based on insights
        if !insights.is_empty() {
            info!("Found {} high-relevance insights", insights.len());
            self.implement_with_insights(task, insights).await?;
        } else {
            warn!("No high-relevance results found, using general approach");
            self.implement_task_directly(task).await?;
        }

        Ok(())
    }

    /// Implement task using search insights
    async fn implement_with_insights(&self, task: &Task, insights: Vec<String>) -> Result<()> {
        info!("Implementing task with {} insights", insights.len());

        // Example: Generate code based on search results
        let implementation = match task.description.to_lowercase() {
            desc if desc.contains("database") && desc.contains("optimize") => {
                // Database optimization based on search
                r#"
-- Based on search insights: PostgreSQL optimization
-- Creating partial indexes for frequently queried columns
CREATE INDEX CONCURRENTLY idx_users_active_recent 
    ON users (created_at) 
    WHERE active = true AND created_at > NOW() - INTERVAL '90 days';

-- Optimizing query planner statistics
ANALYZE users;

-- Adjusting work memory for complex queries
SET work_mem = '256MB';
"#
            }
            desc if desc.contains("cache") => {
                // Caching implementation based on search
                r#"
// Redis caching layer with cache-aside pattern
use redis::{Client, Commands};

pub struct CacheLayer {
    redis: Client,
    ttl: usize,
}

impl CacheLayer {
    pub async fn get_or_compute<F, T>(&self, key: &str, compute: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Serialize + DeserializeOwned,
    {
        let mut conn = self.redis.get_connection()?;
        
        // Try cache first
        if let Ok(cached) = conn.get::<_, String>(key) {
            return Ok(serde_json::from_str(&cached)?);
        }
        
        // Compute and cache
        let value = compute()?;
        let serialized = serde_json::to_string(&value)?;
        conn.setex(key, self.ttl, serialized)?;
        
        Ok(value)
    }
}
"#
            }
            _ => "// Generic implementation based on search insights",
        };

        info!("Generated implementation:\n{}", implementation);

        // Log insights used
        for insight in insights {
            debug!("Applied insight: {}", insight);
        }

        Ok(())
    }

    /// Implement task without search assistance
    async fn implement_task_directly(&self, task: &Task) -> Result<()> {
        info!(
            "Implementing task directly without search: {}",
            task.description
        );
        // Direct implementation logic here
        Ok(())
    }
}

/// Demonstrate the complete workflow
async fn demonstrate_backend_search_workflow() -> Result<()> {
    println!("\n=== Backend Agent Search Workflow Demo ===\n");

    // Initialize coordination bus
    let coordination_bus = Arc::new(CoordinationBus::new().await?);

    // Create backend agent
    let backend_agent = Agent::new(
        "backend-specialist".to_string(),
        AgentRole::Backend,
        coordination_bus.clone(),
    );

    // Create workflow handler
    let mut workflow = BackendSearchWorkflow::new(
        backend_agent,
        "search-agent-001".to_string(),
        coordination_bus.clone(),
    );

    // Example tasks that trigger search
    let tasks = vec![
        Task {
            id: "task-001".to_string(),
            description: "Optimize database queries for user table performance".to_string(),
            priority: TaskPriority::High,
            assigned_agent: Some("backend-specialist".to_string()),
            status: ccswarm::agent::TaskStatus::Pending,
            dependencies: vec![],
            metadata: json!({}),
        },
        Task {
            id: "task-002".to_string(),
            description: "Implement caching strategy for API responses".to_string(),
            priority: TaskPriority::Medium,
            assigned_agent: Some("backend-specialist".to_string()),
            status: ccswarm::agent::TaskStatus::Pending,
            dependencies: vec![],
            metadata: json!({}),
        },
        Task {
            id: "task-003".to_string(),
            description: "Research and implement API versioning".to_string(),
            priority: TaskPriority::Medium,
            assigned_agent: Some("backend-specialist".to_string()),
            status: ccswarm::agent::TaskStatus::Pending,
            dependencies: vec![],
            metadata: json!({}),
        },
        Task {
            id: "task-004".to_string(),
            description: "Create user authentication endpoint".to_string(),
            priority: TaskPriority::High,
            assigned_agent: Some("backend-specialist".to_string()),
            status: ccswarm::agent::TaskStatus::Pending,
            dependencies: vec![],
            metadata: json!({}),
        },
    ];

    // Process each task
    for task in tasks {
        println!("\n--- Processing Task: {} ---", task.id);
        workflow.process_task_with_search(&task).await?;
        sleep(Duration::from_secs(1)).await;
    }

    // Summary
    let pending_searches = workflow.pending_searches.read().await;
    println!("\n=== Workflow Summary ===");
    println!("Total searches performed: {}", pending_searches.len());
    println!("Search queries:");
    for query in pending_searches.iter() {
        println!("  - {}", query);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Run the demonstration
    demonstrate_backend_search_workflow().await?;

    println!("\n=== Demo Complete ===");
    println!("This demo showed how backend agents:");
    println!("1. Analyze tasks to determine if search is needed");
    println!("2. Formulate appropriate search queries");
    println!("3. Send requests to the search agent");
    println!("4. Process search results");
    println!("5. Apply insights to generate better implementations");

    Ok(())
}
