/// Automatic refactoring system - Optimized version
use super::common::{ProposalGenerator, ProposalKind, SymbolOperations, MetricsCollector, MetricType};
use crate::semantic::{
    analyzer::{SemanticAnalyzer, Symbol, SymbolKind},
    memory::{Memory, MemoryType, ProjectMemory},
    symbol_index::SymbolIndex,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RefactoringPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortEstimate {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringKind {
    ExtractFunction,
    SimplifyLogic,
    RemoveDuplication,
    ImproveNaming,
    OptimizePerformance,
    ImproveTestability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefactoringProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: RefactoringKind,
    pub targets: Vec<String>,
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub estimated_effort: EffortEstimate,
    pub priority: RefactoringPriority,
    pub automated: bool,
    pub implementation_steps: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct AutomaticRefactoringSystem {
    analyzer: Arc<SemanticAnalyzer>,
    symbol_index: Arc<SymbolIndex>,
    memory: Arc<ProjectMemory>,
    proposals: Arc<RwLock<Vec<RefactoringProposal>>>,
    metrics: Arc<RwLock<MetricsCollector>>,
}

impl AutomaticRefactoringSystem {
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        symbol_index: Arc<SymbolIndex>,
        memory: Arc<ProjectMemory>,
    ) -> Self {
        Self {
            analyzer,
            symbol_index,
            memory,
            proposals: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(MetricsCollector::default())),
        }
    }

    pub async fn scan_codebase(&mut self) -> Result<Vec<RefactoringProposal>> {
        let mut proposals = Vec::new();
        let symbols = self.symbol_index.get_all_symbols().await?;
        
        // Use a unified check method with different strategies
        for symbol in &symbols {
            if let Some(proposal) = self.check_symbol(symbol).await? {
                proposals.push(proposal);
            }
        }
        
        self.metrics.write().await.update(MetricType::Proposals, proposals.len());
        self.metrics.write().await.update(MetricType::Analysis, 1);
        
        let mut stored = self.proposals.write().await;
        stored.extend(proposals.clone());
        
        Ok(proposals)
    }

    async fn check_symbol(&self, symbol: &Symbol) -> Result<Option<RefactoringProposal>> {
        if let Some(body) = &symbol.body {
            let lines = body.lines().count();
            let complexity = SymbolOperations::calculate_complexity(body);
            
            // Unified check logic
            if lines > 50 {
                return Ok(Some(ProposalGenerator::generate_proposal(
                    ProposalKind::LongFunction,
                    format!("Extract functions from {}", symbol.name),
                    format!("Function {} has {} lines (recommended: max 50)", symbol.name, lines),
                    vec![symbol.path.clone()],
                )));
            }
            
            if complexity > 10 {
                return Ok(Some(ProposalGenerator::generate_proposal(
                    ProposalKind::ComplexLogic,
                    format!("Simplify complex logic in {}", symbol.name),
                    format!("Function {} has cyclomatic complexity of {} (recommended: max 10)", symbol.name, complexity),
                    vec![symbol.path.clone()],
                )));
            }
            
            if !self.check_naming(&symbol.name) {
                return Ok(Some(ProposalGenerator::generate_proposal(
                    ProposalKind::NamingConvention,
                    format!("Rename {} to follow convention", symbol.name),
                    format!("Name '{}' doesn't follow Rust naming conventions", symbol.name),
                    vec![symbol.path.clone()],
                )));
            }
        }
        
        Ok(None)
    }

    fn check_naming(&self, name: &str) -> bool {
        // Simple naming convention check
        match name.chars().next() {
            Some(c) if c.is_uppercase() => name.chars().all(|c| c.is_alphanumeric() || c == '_'),
            Some(c) if c.is_lowercase() => name.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric()),
            _ => false,
        }
    }

    pub async fn apply_proposal(&mut self, proposal_id: &str) -> Result<()> {
        let proposals = self.proposals.read().await;
        
        if let Some(proposal) = proposals.iter().find(|p| p.id == proposal_id) {
            if proposal.automated {
                // Store in memory for tracking
                let memory = Memory {
                    id: format!("refactoring_{}", proposal_id),
                    name: format!("Applied: {}", proposal.title),
                    content: serde_json::to_string(&proposal)?,
                    memory_type: MemoryType::Decision,
                    related_symbols: proposal.targets.clone(),
                    metadata: HashMap::new(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    access_count: 0,
                };
                
                self.memory.store_memory(memory).await?;
                self.metrics.write().await.update(MetricType::Applied, 1);
            }
        }
        
        Ok(())
    }

    pub fn get_stats(&self) -> RefactoringStats {
        let metrics = self.metrics.blocking_read();
        RefactoringStats {
            total_proposals: metrics.refactoring_proposals,
            applied_proposals: metrics.applied_refactorings,
            time_saved_hours: metrics.applied_refactorings as f64 * 0.5,
            lines_refactored: metrics.applied_refactorings * 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringStats {
    pub total_proposals: usize,
    pub applied_proposals: usize,
    pub time_saved_hours: f64,
    pub lines_refactored: usize,
}