//! Semantic code analysis integration layer for ccswarm
//!
//! This module provides integration with serena's semantic code analysis capabilities,
//! enabling intelligent code understanding, symbol-level operations, and project memory management.

pub mod common;
pub mod analyzer;
pub mod cross_codebase_optimization;
pub mod memory;
pub mod refactoring_system;
pub mod sangha_voting;
pub mod symbol_index;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use analyzer::SemanticAnalyzer;
pub use memory::ProjectMemory;
pub use symbol_index::SymbolIndex;

/// Configuration for semantic features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConfig {
    /// Enable semantic code analysis
    pub enabled: bool,

    /// Maximum cache size for semantic analysis
    pub cache_size: String,

    /// Index codebase on startup
    pub index_on_startup: bool,

    /// Enable MCP protocol server
    pub mcp_enabled: bool,

    /// MCP server port
    pub mcp_port: u16,

    /// Enable project memory
    pub memory_enabled: bool,

    /// Maximum number of memories
    pub max_memories: usize,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size: "1GB".to_string(),
            index_on_startup: true,
            mcp_enabled: true,
            mcp_port: 8080,
            memory_enabled: true,
            max_memories: 100,
        }
    }
}

/// Main semantic integration manager
pub struct SemanticManager {
    config: SemanticConfig,
    analyzer: Arc<SemanticAnalyzer>,
    memory: Arc<ProjectMemory>,
    symbol_index: Arc<SymbolIndex>,
}

impl SemanticManager {
    /// Create a new semantic manager
    pub async fn new(config: SemanticConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let analyzer = Arc::new(SemanticAnalyzer::new(config.clone()).await?);
        let memory = Arc::new(ProjectMemory::new(config.max_memories).await?);
        let symbol_index = Arc::new(SymbolIndex::new().await?);

        Ok(Self {
            config,
            analyzer,
            memory,
            symbol_index,
        })
    }

    /// Initialize semantic features for the project
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.index_on_startup {
            log::info!("Indexing codebase for semantic analysis...");
            self.symbol_index.index_codebase().await?;
        }

        if self.config.mcp_enabled {
            log::info!(
                "Starting MCP protocol server on port {}",
                self.config.mcp_port
            );
            // MCP server initialization will be handled by the mcp module
        }

        if self.config.memory_enabled {
            log::info!("Loading project memories...");
            self.memory.load_memories().await?;
        }

        Ok(())
    }

    /// Get the semantic analyzer
    pub fn analyzer(&self) -> Arc<SemanticAnalyzer> {
        self.analyzer.clone()
    }

    /// Get the project memory
    pub fn memory(&self) -> Arc<ProjectMemory> {
        self.memory.clone()
    }

    /// Get the symbol index
    pub fn symbol_index(&self) -> Arc<SymbolIndex> {
        self.symbol_index.clone()
    }

}

/// Result type for semantic operations
pub type SemanticResult<T> = Result<T, SemanticError>;

/// Error type for semantic operations
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Memory operation failed: {0}")]
    MemoryError(String),

    #[error("Index operation failed: {0}")]
    IndexError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}
