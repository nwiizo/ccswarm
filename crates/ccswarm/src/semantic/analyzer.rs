//! Semantic code analysis engine
//!
//! Provides intelligent code understanding and analysis capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{SemanticConfig, SemanticResult};

/// Represents a code symbol with semantic information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Symbol {
    /// Name of the symbol
    pub name: String,

    /// Path to the symbol (e.g., "module::class::method")
    pub path: String,

    /// Type of symbol (function, class, variable, etc.)
    pub kind: SymbolKind,

    /// File location
    pub file_path: String,

    /// Line number
    pub line: usize,

    /// Symbol body/content
    pub body: Option<String>,

    /// Dependencies and references
    pub references: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Type of symbol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Module,
    Variable,
    Constant,
    Interface,
    Trait,
    Component,
    Other(String),
}

/// Symbol change representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolChange {
    pub symbol: Symbol,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Type of change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    ApiModification,
    Refactored,
}

impl SymbolChange {
    /// Check if this change affects API
    pub fn affects_api(&self) -> bool {
        matches!(self.change_type, ChangeType::ApiModification)
            || (self.symbol.kind == SymbolKind::Function
                || self.symbol.kind == SymbolKind::Method
                || self.symbol.kind == SymbolKind::Interface)
    }
}

/// Semantic analyzer for code understanding
pub struct SemanticAnalyzer {
    _config: SemanticConfig,
    symbol_cache: Arc<RwLock<HashMap<String, Symbol>>>,
    _pattern_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub async fn new(config: SemanticConfig) -> SemanticResult<Self> {
        Ok(Self {
            _config: config,
            symbol_cache: Arc::new(RwLock::new(HashMap::new())),
            _pattern_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Find symbols relevant to a task description
    pub async fn find_relevant_symbols(&self, description: &str) -> SemanticResult<Vec<Symbol>> {
        let keywords = self.extract_keywords(description);
        let mut relevant_symbols = Vec::new();

        // Search for symbols matching keywords
        let cache = self.symbol_cache.read().await;
        for keyword in &keywords {
            for (path, symbol) in cache.iter() {
                if path.contains(keyword) || symbol.name.contains(keyword) {
                    relevant_symbols.push(symbol.clone());
                }
            }
        }

        // Sort by relevance (simple heuristic for now)
        relevant_symbols.sort_by_key(|s| {
            let name_match = if s.name.contains(description) { 0 } else { 1 };
            let path_match = if s.path.contains(description) { 0 } else { 1 };
            (name_match, path_match)
        });

        Ok(relevant_symbols)
    }

    /// Extract keywords from a task description
    fn extract_keywords(&self, description: &str) -> Vec<String> {
        // Simple keyword extraction - can be enhanced with NLP
        description
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .filter(|word| !STOP_WORDS.contains(&word.as_str()))
            .collect()
    }

    /// Find a specific symbol by name and kind
    pub async fn find_symbol(
        &self,
        name: &str,
        kind: Option<SymbolKind>,
    ) -> SemanticResult<Option<Symbol>> {
        let cache = self.symbol_cache.read().await;

        for symbol in cache.values() {
            if symbol.name == name {
                if let Some(ref k) = kind {
                    if symbol.kind == *k {
                        return Ok(Some(symbol.clone()));
                    }
                } else {
                    return Ok(Some(symbol.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Get all symbols in a file
    pub async fn get_file_symbols(&self, file_path: &str) -> SemanticResult<Vec<Symbol>> {
        let cache = self.symbol_cache.read().await;
        let symbols: Vec<Symbol> = cache
            .values()
            .filter(|s| s.file_path == file_path)
            .cloned()
            .collect();

        Ok(symbols)
    }

    /// Find symbols that reference a given symbol
    pub async fn find_references(&self, symbol: &Symbol) -> SemanticResult<Vec<Symbol>> {
        let cache = self.symbol_cache.read().await;
        let references: Vec<Symbol> = cache
            .values()
            .filter(|s| s.references.contains(&symbol.path))
            .cloned()
            .collect();

        Ok(references)
    }

    /// Analyze the impact of a symbol change
    pub async fn analyze_impact(&self, change: &SymbolChange) -> SemanticResult<ImpactAnalysis> {
        let affected_symbols = self.find_references(&change.symbol).await?;

        let severity = match change.change_type {
            ChangeType::Deleted => ImpactSeverity::High,
            ChangeType::ApiModification => ImpactSeverity::High,
            ChangeType::Modified => {
                if change.affects_api() {
                    ImpactSeverity::High
                } else {
                    ImpactSeverity::Medium
                }
            }
            ChangeType::Refactored => ImpactSeverity::Low,
            ChangeType::Added => ImpactSeverity::Low,
        };

        Ok(ImpactAnalysis {
            change: change.clone(),
            affected_symbols,
            severity,
            suggested_actions: self.generate_suggested_actions(change, &severity),
        })
    }

    /// Generate suggested actions for a change
    fn generate_suggested_actions(
        &self,
        change: &SymbolChange,
        severity: &ImpactSeverity,
    ) -> Vec<String> {
        let mut actions = Vec::new();

        match severity {
            ImpactSeverity::High => {
                actions.push("Review all dependent code".to_string());
                actions.push("Update documentation".to_string());
                actions.push("Run comprehensive tests".to_string());
                if change.affects_api() {
                    actions.push("Notify API consumers".to_string());
                    actions.push("Update API documentation".to_string());
                }
            }
            ImpactSeverity::Medium => {
                actions.push("Test affected components".to_string());
                actions.push("Review integration points".to_string());
            }
            ImpactSeverity::Low => {
                actions.push("Run unit tests".to_string());
            }
        }

        actions
    }

    /// Register a symbol in the cache
    pub async fn register_symbol(&self, symbol: Symbol) -> SemanticResult<()> {
        let mut cache = self.symbol_cache.write().await;
        cache.insert(symbol.path.clone(), symbol);
        Ok(())
    }

    /// Clear the symbol cache
    pub async fn clear_cache(&self) -> SemanticResult<()> {
        let mut cache = self.symbol_cache.write().await;
        cache.clear();
        Ok(())
    }
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub change: SymbolChange,
    pub affected_symbols: Vec<Symbol>,
    pub severity: ImpactSeverity,
    pub suggested_actions: Vec<String>,
}

/// Impact severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
}

// Common stop words to filter out from keyword extraction
const STOP_WORDS: &[&str] = &[
    "the",
    "and",
    "for",
    "with",
    "from",
    "into",
    "this",
    "that",
    "these",
    "those",
    "will",
    "would",
    "could",
    "should",
    "must",
    "can",
    "may",
    "might",
    "add",
    "create",
    "update",
    "delete",
    "modify",
    "change",
    "implement",
    "fix",
];
