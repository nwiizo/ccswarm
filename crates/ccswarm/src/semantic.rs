use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic analysis module for code understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalyzer {
    pub symbols: HashMap<String, Symbol>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Variable,
    Module,
    Interface,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: RelationshipKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipKind {
    Calls,
    Inherits,
    Implements,
    Uses,
    Contains,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    pub fn find_dependencies(&self, symbol_name: &str) -> Vec<&Symbol> {
        self.relationships
            .iter()
            .filter(|r| r.from == symbol_name)
            .filter_map(|r| self.symbols.get(&r.to))
            .collect()
    }

    pub fn find_dependents(&self, symbol_name: &str) -> Vec<&Symbol> {
        self.relationships
            .iter()
            .filter(|r| r.to == symbol_name)
            .filter_map(|r| self.symbols.get(&r.from))
            .collect()
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}