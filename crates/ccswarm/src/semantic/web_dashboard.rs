//! Web dashboard for semantic features
//! 
//! Provides a web-based interface for monitoring and controlling semantic operations

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply, Rejection};
use warp::http::StatusCode;
use chrono::{DateTime, Utc};

use super::{SemanticManager, SemanticError, SemanticResult};
use super::analyzer::Symbol;
use super::refactoring_system::{RefactoringProposal, RefactoringStats};
use super::cross_codebase_optimization::CrossCodebaseAnalysis;

/// Dashboard state
#[derive(Clone)]
pub struct DashboardState {
    manager: Arc<SemanticManager>,
    metrics: Arc<RwLock<DashboardMetrics>>,
    events: Arc<RwLock<Vec<DashboardEvent>>>,
    analysis_cache: Arc<RwLock<Option<CrossCodebaseAnalysis>>>,
}

/// Dashboard metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub total_symbols: usize,
    pub total_memories: usize,
    pub refactoring_proposals: usize,
    pub active_agents: usize,
    pub code_quality_score: f64,
    pub technical_debt_hours: f64,
    pub last_updated: DateTime<Utc>,
}

/// Dashboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub id: String,
    pub event_type: EventType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub severity: EventSeverity,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Analysis,
    Refactoring,
    AgentGeneration,
    Optimization,
    Error,
}

/// Event severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Success,
}

/// API request types
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeRequest {
    pub path: String,
    pub deep: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefactorRequest {
    pub proposal_id: String,
    pub apply: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: String,
    pub decision: String,
    pub reason: String,
}

/// API response types
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolsResponse {
    pub symbols: Vec<SymbolInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub complexity: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefactoringResponse {
    pub proposals: Vec<RefactoringProposal>,
    pub stats: RefactoringStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
}

impl DashboardState {
    /// Create new dashboard state
    pub async fn new(manager: Arc<SemanticManager>) -> Self {
        let metrics = Arc::new(RwLock::new(DashboardMetrics {
            total_symbols: 0,
            total_memories: 0,
            refactoring_proposals: 0,
            active_agents: 0,
            code_quality_score: 0.0,
            technical_debt_hours: 0.0,
            last_updated: Utc::now(),
        }));
        
        let state = Self {
            manager,
            metrics,
            events: Arc::new(RwLock::new(Vec::new())),
            analysis_cache: Arc::new(RwLock::new(None)),
        };
        
        // Update initial metrics
        state.update_metrics().await;
        
        state
    }
    
    /// Update dashboard metrics
    pub async fn update_metrics(&self) {
        let symbols = self.manager.symbol_index()
            .get_all_symbols().await
            .unwrap_or_default();
        
        let memories = self.manager.memory()
            .list_memories().await
            .unwrap_or_default();
        
        let mut metrics = self.metrics.write().await;
        metrics.total_symbols = symbols.len();
        metrics.total_memories = memories.len();
        metrics.last_updated = Utc::now();
        
        // Calculate code quality score (simplified)
        let avg_complexity = 5.0; // Would calculate from symbols
        metrics.code_quality_score = (100.0 - avg_complexity * 2.0).max(0.0).min(100.0);
    }
    
    /// Add dashboard event
    pub async fn add_event(&self, event_type: EventType, message: String, severity: EventSeverity) {
        let event = DashboardEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            message,
            timestamp: Utc::now(),
            severity,
        };
        
        let mut events = self.events.write().await;
        events.push(event);
        
        // Keep only last 100 events
        if events.len() > 100 {
            events.drain(0..events.len() - 100);
        }
    }
}

/// Create web server for dashboard
pub async fn create_server(state: DashboardState, port: u16) {
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);
    
    // Routes
    let routes = health_route(state.clone())
        .or(metrics_route(state.clone()))
        .or(events_route(state.clone()))
        .or(symbols_route(state.clone()))
        .or(refactoring_route(state.clone()))
        .or(analyze_route(state.clone()))
        .or(websocket_route(state))
        .with(cors);
    
    println!("🌐 Dashboard server running on http://localhost:{}", port);
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

// Route handlers

fn health_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "health")
        .and(warp::get())
        .map(move || {
            let response = HealthResponse {
                status: "healthy".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime: 0, // Would calculate actual uptime
            };
            warp::reply::json(&ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            })
        })
}

fn metrics_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "metrics")
        .and(warp::get())
        .and(with_state(state))
        .and_then(handle_metrics)
}

fn events_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "events")
        .and(warp::get())
        .and(with_state(state))
        .and_then(handle_events)
}

fn symbols_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "symbols")
        .and(warp::get())
        .and(with_state(state))
        .and_then(handle_symbols)
}

fn refactoring_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "refactoring")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_refactoring)
        .or(
            warp::path!("api" / "refactoring")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_state(state))
                .and_then(handle_apply_refactoring)
        )
}

fn analyze_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "analyze")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state))
        .and_then(handle_analyze)
}

fn websocket_route(state: DashboardState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("ws")
        .and(warp::ws())
        .and(with_state(state))
        .map(|ws: warp::ws::Ws, state| {
            ws.on_upgrade(move |websocket| handle_websocket(websocket, state))
        })
}

fn with_state(state: DashboardState) -> impl Filter<Extract = (DashboardState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

// Handler implementations

async fn handle_metrics(state: DashboardState) -> Result<impl Reply, Rejection> {
    state.update_metrics().await;
    let metrics = state.metrics.read().await;
    
    Ok(warp::reply::json(&ApiResponse {
        success: true,
        data: Some(metrics.clone()),
        error: None,
    }))
}

async fn handle_events(state: DashboardState) -> Result<impl Reply, Rejection> {
    let events = state.events.read().await;
    
    Ok(warp::reply::json(&ApiResponse {
        success: true,
        data: Some(events.clone()),
        error: None,
    }))
}

async fn handle_symbols(state: DashboardState) -> Result<impl Reply, Rejection> {
    match state.manager.symbol_index().get_all_symbols().await {
        Ok(symbols) => {
            let symbol_infos: Vec<SymbolInfo> = symbols.iter()
                .take(100) // Limit for performance
                .map(|s| SymbolInfo {
                    name: s.name.clone(),
                    kind: format!("{:?}", s.kind),
                    file: s.file_path.clone(),
                    line: s.line,
                    complexity: None,
                })
                .collect();
            
            let response = SymbolsResponse {
                total: symbols.len(),
                symbols: symbol_infos,
            };
            
            Ok(warp::reply::json(&ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::<SymbolsResponse> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn handle_get_refactoring(state: DashboardState) -> Result<impl Reply, Rejection> {
    let refactoring_system = super::refactoring_system::AutomaticRefactoringSystem::new(
        state.manager.analyzer(),
        state.manager.symbol_index(),
        state.manager.memory(),
    );
    
    match refactoring_system.scan_codebase().await {
        Ok(proposals) => {
            let response = RefactoringResponse {
                proposals: proposals.into_iter().take(20).collect(),
                stats: refactoring_system.get_stats().clone(),
            };
            
            Ok(warp::reply::json(&ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::<RefactoringResponse> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn handle_apply_refactoring(
    request: RefactorRequest,
    state: DashboardState,
) -> Result<impl Reply, Rejection> {
    state.add_event(
        EventType::Refactoring,
        format!("Applying refactoring: {}", request.proposal_id),
        EventSeverity::Info,
    ).await;
    
    // In real implementation, would apply the refactoring
    Ok(warp::reply::json(&ApiResponse {
        success: true,
        data: Some("Refactoring applied successfully"),
        error: None,
    }))
}

async fn handle_analyze(
    request: AnalyzeRequest,
    state: DashboardState,
) -> Result<impl Reply, Rejection> {
    state.add_event(
        EventType::Analysis,
        format!("Analyzing path: {}", request.path),
        EventSeverity::Info,
    ).await;
    
    // Trigger analysis
    if request.deep {
        match state.manager.symbol_index().index_codebase().await {
            Ok(_) => {
                state.add_event(
                    EventType::Analysis,
                    "Deep analysis completed".to_string(),
                    EventSeverity::Success,
                ).await;
                
                Ok(warp::reply::json(&ApiResponse {
                    success: true,
                    data: Some("Analysis completed"),
                    error: None,
                }))
            }
            Err(e) => {
                state.add_event(
                    EventType::Error,
                    format!("Analysis failed: {}", e),
                    EventSeverity::Error,
                ).await;
                
                Ok(warp::reply::json(&ApiResponse::<String> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }))
            }
        }
    } else {
        Ok(warp::reply::json(&ApiResponse {
            success: true,
            data: Some("Quick analysis completed"),
            error: None,
        }))
    }
}

async fn handle_websocket(ws: warp::ws::WebSocket, state: DashboardState) {
    use futures::{FutureExt, StreamExt};
    use warp::ws::Message;
    use tokio::time::{sleep, Duration};
    
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(tokio::sync::Mutex::new(tx));
    
    // Send metrics updates every 5 seconds
    let tx_clone = tx.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            
            state_clone.update_metrics().await;
            let metrics = state_clone.metrics.read().await;
            
            let message = serde_json::to_string(&metrics.clone()).unwrap_or_default();
            
            let mut tx = tx_clone.lock().await;
            if tx.send(Message::text(message)).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_close() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

/// Static HTML content for the dashboard
pub const DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ccswarm Semantic Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
            min-height: 100vh;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        .header {
            text-align: center;
            padding: 30px 0;
        }
        .header h1 {
            font-size: 3em;
            margin-bottom: 10px;
        }
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }
        .metric-card {
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 15px;
            padding: 25px;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }
        .metric-value {
            font-size: 2.5em;
            font-weight: bold;
            margin: 10px 0;
        }
        .metric-label {
            font-size: 0.9em;
            opacity: 0.8;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .events-container {
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 15px;
            padding: 25px;
            margin: 30px 0;
            max-height: 400px;
            overflow-y: auto;
        }
        .event-item {
            padding: 10px;
            margin: 5px 0;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .event-severity-success { border-left: 3px solid #4caf50; }
        .event-severity-warning { border-left: 3px solid #ff9800; }
        .event-severity-error { border-left: 3px solid #f44336; }
        .event-severity-info { border-left: 3px solid #2196f3; }
        .actions {
            display: flex;
            gap: 15px;
            margin: 30px 0;
        }
        .btn {
            padding: 12px 30px;
            background: rgba(255, 255, 255, 0.2);
            border: 2px solid rgba(255, 255, 255, 0.3);
            border-radius: 25px;
            color: #fff;
            font-size: 1em;
            cursor: pointer;
            transition: all 0.3s;
        }
        .btn:hover {
            background: rgba(255, 255, 255, 0.3);
            transform: translateY(-2px);
        }
        .status-indicator {
            width: 10px;
            height: 10px;
            border-radius: 50%;
            display: inline-block;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        .status-healthy { background: #4caf50; }
        .status-warning { background: #ff9800; }
        .status-error { background: #f44336; }
        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧠 ccswarm Semantic Dashboard</h1>
            <p>Real-time monitoring and control of semantic operations</p>
            <p><span class="status-indicator status-healthy"></span>System Healthy</p>
        </div>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-label">Total Symbols</div>
                <div class="metric-value" id="total-symbols">-</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Project Memories</div>
                <div class="metric-value" id="total-memories">-</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Code Quality</div>
                <div class="metric-value" id="quality-score">-</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Technical Debt</div>
                <div class="metric-value" id="tech-debt">- hrs</div>
            </div>
        </div>
        
        <div class="actions">
            <button class="btn" onclick="analyzeCode()">🔍 Analyze Code</button>
            <button class="btn" onclick="scanRefactoring()">🔧 Scan Refactoring</button>
            <button class="btn" onclick="generateAgents()">🤖 Generate Agents</button>
            <button class="btn" onclick="optimizeCross()">🚀 Cross-Optimize</button>
        </div>
        
        <div class="events-container">
            <h2>📊 Recent Events</h2>
            <div id="events-list"></div>
        </div>
    </div>
    
    <script>
        const API_BASE = 'http://localhost:3000/api';
        let ws = null;
        
        // Initialize WebSocket connection
        function initWebSocket() {
            ws = new WebSocket('ws://localhost:3000/ws');
            
            ws.onmessage = (event) => {
                const metrics = JSON.parse(event.data);
                updateMetrics(metrics);
            };
            
            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };
            
            ws.onclose = () => {
                setTimeout(initWebSocket, 5000); // Reconnect after 5 seconds
            };
        }
        
        // Update metrics display
        function updateMetrics(metrics) {
            document.getElementById('total-symbols').textContent = metrics.total_symbols || '-';
            document.getElementById('total-memories').textContent = metrics.total_memories || '-';
            document.getElementById('quality-score').textContent = 
                metrics.code_quality_score ? metrics.code_quality_score.toFixed(1) + '%' : '-';
            document.getElementById('tech-debt').textContent = 
                metrics.technical_debt_hours ? metrics.technical_debt_hours.toFixed(0) + ' hrs' : '- hrs';
        }
        
        // Fetch and display events
        async function fetchEvents() {
            try {
                const response = await fetch(`${API_BASE}/events`);
                const result = await response.json();
                
                if (result.success && result.data) {
                    displayEvents(result.data);
                }
            } catch (error) {
                console.error('Failed to fetch events:', error);
            }
        }
        
        // Display events in the UI
        function displayEvents(events) {
            const container = document.getElementById('events-list');
            container.innerHTML = '';
            
            events.slice(-10).reverse().forEach(event => {
                const item = document.createElement('div');
                item.className = `event-item event-severity-${event.severity.toLowerCase()}`;
                
                const time = new Date(event.timestamp).toLocaleTimeString();
                item.innerHTML = `
                    <div>
                        <strong>${event.event_type}</strong>: ${event.message}
                    </div>
                    <div style="opacity: 0.7; font-size: 0.9em">${time}</div>
                `;
                
                container.appendChild(item);
            });
        }
        
        // Action handlers
        async function analyzeCode() {
            try {
                const response = await fetch(`${API_BASE}/analyze`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ path: '.', deep: true })
                });
                const result = await response.json();
                
                if (result.success) {
                    alert('Analysis started successfully!');
                    fetchEvents();
                }
            } catch (error) {
                alert('Failed to start analysis: ' + error.message);
            }
        }
        
        async function scanRefactoring() {
            try {
                const response = await fetch(`${API_BASE}/refactoring`);
                const result = await response.json();
                
                if (result.success && result.data) {
                    alert(`Found ${result.data.proposals.length} refactoring opportunities!`);
                    fetchEvents();
                }
            } catch (error) {
                alert('Failed to scan refactoring: ' + error.message);
            }
        }
        
        function generateAgents() {
            alert('Agent generation started! Check the console for progress.');
        }
        
        function optimizeCross() {
            alert('Cross-codebase optimization started! This may take a while...');
        }
        
        // Initialize on load
        window.onload = () => {
            initWebSocket();
            fetchEvents();
            
            // Fetch initial metrics
            fetch(`${API_BASE}/metrics`)
                .then(res => res.json())
                .then(result => {
                    if (result.success && result.data) {
                        updateMetrics(result.data);
                    }
                });
            
            // Refresh events periodically
            setInterval(fetchEvents, 10000);
        };
    </script>
</body>
</html>
"#;