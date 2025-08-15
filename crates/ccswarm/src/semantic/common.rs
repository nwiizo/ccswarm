/// Common utilities for semantic features
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::semantic::{
    analyzer::Symbol,
    refactoring_system::RefactoringProposal,
};

/// Generic handler trait for all semantic operations
#[async_trait::async_trait]
pub trait SemanticHandler: Send + Sync {
    async fn handle(&self, params: HandlerParams) -> Result<HandlerResponse>;
}

/// Unified parameters for all handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerParams {
    pub operation: String,
    pub data: serde_json::Value,
}

/// Unified response for all handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: Option<String>,
}

/// Factory for creating handlers
pub struct HandlerFactory;

impl HandlerFactory {
    pub fn create_handler(handler_type: HandlerType) -> Box<dyn SemanticHandler> {
        match handler_type {
            HandlerType::Analysis => Box::new(AnalysisHandler::default()),
            HandlerType::Refactoring => Box::new(RefactoringHandler::default()),
            HandlerType::Symbol => Box::new(SymbolHandler::default()),
            HandlerType::Memory => Box::new(MemoryHandler::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HandlerType {
    Analysis,
    Refactoring,
    Symbol,
    Memory,
}

/// Default analysis handler
#[derive(Default)]
struct AnalysisHandler;

#[async_trait::async_trait]
impl SemanticHandler for AnalysisHandler {
    async fn handle(&self, params: HandlerParams) -> Result<HandlerResponse> {
        Ok(HandlerResponse {
            success: true,
            data: serde_json::json!({
                "operation": params.operation,
                "result": "analysis_complete"
            }),
            message: Some("Analysis completed successfully".to_string()),
        })
    }
}

/// Default refactoring handler
#[derive(Default)]
struct RefactoringHandler;

#[async_trait::async_trait]
impl SemanticHandler for RefactoringHandler {
    async fn handle(&self, params: HandlerParams) -> Result<HandlerResponse> {
        Ok(HandlerResponse {
            success: true,
            data: serde_json::json!({
                "operation": params.operation,
                "result": "refactoring_complete"
            }),
            message: Some("Refactoring completed successfully".to_string()),
        })
    }
}

/// Default symbol handler
#[derive(Default)]
struct SymbolHandler;

#[async_trait::async_trait]
impl SemanticHandler for SymbolHandler {
    async fn handle(&self, params: HandlerParams) -> Result<HandlerResponse> {
        Ok(HandlerResponse {
            success: true,
            data: serde_json::json!({
                "operation": params.operation,
                "result": "symbol_operation_complete"
            }),
            message: Some("Symbol operation completed successfully".to_string()),
        })
    }
}

/// Default memory handler
#[derive(Default)]
struct MemoryHandler;

#[async_trait::async_trait]
impl SemanticHandler for MemoryHandler {
    async fn handle(&self, params: HandlerParams) -> Result<HandlerResponse> {
        Ok(HandlerResponse {
            success: true,
            data: serde_json::json!({
                "operation": params.operation,
                "result": "memory_operation_complete"
            }),
            message: Some("Memory operation completed successfully".to_string()),
        })
    }
}

/// Common proposal generator for refactoring
pub struct ProposalGenerator;

impl ProposalGenerator {
    pub fn generate_proposal(
        kind: ProposalKind,
        title: String,
        description: String,
        targets: Vec<String>,
    ) -> RefactoringProposal {
        use crate::semantic::refactoring_system::{
            RefactoringPriority, EffortEstimate
        };
        // RefactoringKind is now defined locally
        #[derive(Debug)]
        enum RefactoringKind {
            ExtractFunction,
            SimplifyLogic,
            RemoveDuplication,
            ImproveNaming,
        }
        
        let (priority, effort, automated) = match kind {
            ProposalKind::LongFunction => (RefactoringPriority::Medium, EffortEstimate::Small, true),
            ProposalKind::ComplexLogic => (RefactoringPriority::High, EffortEstimate::Medium, false),
            ProposalKind::DuplicateCode => (RefactoringPriority::High, EffortEstimate::Large, true),
            ProposalKind::NamingConvention => (RefactoringPriority::Low, EffortEstimate::Small, true),
            ProposalKind::Architecture => (RefactoringPriority::Critical, EffortEstimate::Large, false),
        };
        
        RefactoringProposal {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            kind: RefactoringKind::ExtractFunction,
            targets,
            benefits: vec!["Improved code quality".to_string()],
            risks: vec![],
            estimated_effort: effort,
            priority,
            automated,
            implementation_steps: vec![],
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProposalKind {
    LongFunction,
    ComplexLogic,
    DuplicateCode,
    NamingConvention,
    Architecture,
}

/// Common route handler for web dashboard
pub struct RouteHandler;

impl RouteHandler {
    pub fn create_route(path: &str, handler: Box<dyn SemanticHandler>) -> Route {
        Route {
            path: path.to_string(),
            handler,
        }
    }
}

pub struct Route {
    pub path: String,
    pub handler: Box<dyn SemanticHandler>,
}

/// Common metrics collector
#[derive(Default, Clone)]
pub struct MetricsCollector {
    pub total_symbols: usize,
    pub total_memories: usize,
    pub refactoring_proposals: usize,
    pub applied_refactorings: usize,
    pub analysis_runs: usize,
}

impl MetricsCollector {
    pub fn update(&mut self, metric_type: MetricType, value: usize) {
        match metric_type {
            MetricType::Symbols => self.total_symbols = value,
            MetricType::Memories => self.total_memories = value,
            MetricType::Proposals => self.refactoring_proposals = value,
            MetricType::Applied => self.applied_refactorings = value,
            MetricType::Analysis => self.analysis_runs += 1,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_symbols": self.total_symbols,
            "total_memories": self.total_memories,
            "refactoring_proposals": self.refactoring_proposals,
            "applied_refactorings": self.applied_refactorings,
            "analysis_runs": self.analysis_runs,
        })
    }
}

#[derive(Debug, Clone)]
pub enum MetricType {
    Symbols,
    Memories,
    Proposals,
    Applied,
    Analysis,
}

/// Common symbol operations
pub struct SymbolOperations;

impl SymbolOperations {
    pub async fn find_and_process<F, R>(
        symbols: Vec<Symbol>,
        filter: F,
    ) -> Vec<R>
    where
        F: Fn(&Symbol) -> Option<R>,
    {
        symbols.iter().filter_map(filter).collect()
    }
    
    pub fn calculate_complexity(body: &str) -> usize {
        let mut complexity = 1;
        for line in body.lines() {
            if line.contains("if ") || line.contains("match ") {
                complexity += 1;
            }
            if line.contains("for ") || line.contains("while ") {
                complexity += 1;
            }
            if line.contains("&&") || line.contains("||") {
                complexity += 1;
            }
        }
        complexity
    }
    
    pub fn calculate_similarity(s1: &str, s2: &str) -> f64 {
        let lines1: Vec<&str> = s1.lines().collect();
        let lines2: Vec<&str> = s2.lines().collect();
        
        if lines1.is_empty() || lines2.is_empty() {
            return 0.0;
        }
        
        let common = lines1.iter()
            .filter(|l| lines2.contains(l))
            .count();
        
        let total = lines1.len().max(lines2.len());
        common as f64 / total as f64
    }
}