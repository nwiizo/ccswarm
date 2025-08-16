//! Project memory system for persistent knowledge management
//!
//! Stores and manages project-specific knowledge that can be shared across agents

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

use super::analyzer::{ImpactAnalysis, Symbol, SymbolChange};
use super::{SemanticError, SemanticResult};

/// A memory entry containing project knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier for the memory
    pub id: String,

    /// Name of the memory (meaningful identifier)
    pub name: String,

    /// Content of the memory
    pub content: String,

    /// Type of memory
    pub memory_type: MemoryType,

    /// Associated symbols
    pub related_symbols: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Access count for relevance tracking
    pub access_count: usize,
}

/// Type of memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Architecture,
    CodingConvention,
    DomainKnowledge,
    ApiChange,
    Refactoring,
    BugPattern,
    Performance,
    Security,
    Decision,
    Other(String),
}

/// Context retrieved from project memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Relevant memories for the current task
    pub memories: Vec<Memory>,

    /// Aggregated knowledge summary
    pub summary: String,

    /// Suggested patterns and conventions
    pub patterns: Vec<String>,

    /// Related API changes
    pub api_changes: Vec<SymbolChange>,
}

/// Project memory manager
pub struct ProjectMemory {
    /// Maximum number of memories to retain
    max_memories: usize,

    /// Memory storage
    memories: RwLock<HashMap<String, Memory>>,

    /// Memory directory path
    memory_dir: PathBuf,
}

impl ProjectMemory {
    /// Create a new project memory manager
    pub async fn new(max_memories: usize) -> SemanticResult<Self> {
        let memory_dir = PathBuf::from(".claude/memories");

        // Create memory directory if it doesn't exist
        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir).await.map_err(|e| {
                SemanticError::MemoryError(format!("Failed to create memory directory: {}", e))
            })?;
        }

        Ok(Self {
            max_memories,
            memories: RwLock::new(HashMap::new()),
            memory_dir,
        })
    }

    /// Load memories from disk
    pub async fn load_memories(&self) -> SemanticResult<()> {
        let mut memories = self.memories.write().await;
        memories.clear();

        // Read all memory files
        let mut entries = fs::read_dir(&self.memory_dir).await.map_err(|e| {
            SemanticError::MemoryError(format!("Failed to read memory directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            SemanticError::MemoryError(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await.map_err(|e| {
                    SemanticError::MemoryError(format!("Failed to read memory file: {}", e))
                })?;

                if let Ok(memory) = serde_json::from_str::<Memory>(&content) {
                    memories.insert(memory.id.clone(), memory);
                }
            }
        }

        log::info!("Loaded {} memories from disk", memories.len());
        Ok(())
    }

    /// Save a memory to disk
    async fn save_memory(&self, memory: &Memory) -> SemanticResult<()> {
        let file_path = self.memory_dir.join(format!("{}.json", memory.id));
        let content = serde_json::to_string_pretty(memory)?;

        fs::write(&file_path, content)
            .await
            .map_err(|e| SemanticError::MemoryError(format!("Failed to save memory: {}", e)))?;

        Ok(())
    }

    /// Store a new memory
    pub async fn store_memory(&self, memory: Memory) -> SemanticResult<()> {
        // Check memory limit
        let mut memories = self.memories.write().await;

        if memories.len() >= self.max_memories {
            // Remove least accessed memory
            if let Some(least_used_id) = memories
                .values()
                .min_by_key(|m| m.access_count)
                .map(|m| m.id.clone())
            {
                memories.remove(&least_used_id);

                // Delete from disk
                let file_path = self.memory_dir.join(format!("{}.json", least_used_id));
                let _ = fs::remove_file(&file_path).await;
            }
        }

        // Save to disk
        self.save_memory(&memory).await?;

        // Store in memory
        memories.insert(memory.id.clone(), memory);

        Ok(())
    }

    /// Retrieve a specific memory
    pub async fn get_memory(&self, name: &str) -> SemanticResult<Option<Memory>> {
        let mut memories = self.memories.write().await;

        if let Some(memory) = memories.values_mut().find(|m| m.name == name) {
            memory.access_count += 1;
            memory.updated_at = Utc::now();
            let result = memory.clone();

            // Update on disk
            self.save_memory(&result).await?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// List all available memories
    pub async fn list_memories(&self) -> SemanticResult<Vec<String>> {
        let memories = self.memories.read().await;
        Ok(memories.values().map(|m| m.name.clone()).collect())
    }

    /// Delete a memory
    pub async fn delete_memory(&self, name: &str) -> SemanticResult<()> {
        let mut memories = self.memories.write().await;

        if let Some(memory) = memories.values().find(|m| m.name == name) {
            let id = memory.id.clone();
            memories.remove(&id);

            // Delete from disk
            let file_path = self.memory_dir.join(format!("{}.json", id));
            fs::remove_file(&file_path).await.map_err(|e| {
                SemanticError::MemoryError(format!("Failed to delete memory file: {}", e))
            })?;
        }

        Ok(())
    }

    /// Retrieve context relevant to given symbols
    pub async fn retrieve_context(&self, symbols: &[Symbol]) -> SemanticResult<ProjectContext> {
        let memories = self.memories.read().await;
        let mut relevant_memories = Vec::new();
        let mut api_changes = Vec::new();
        let mut patterns = Vec::new();

        // Find relevant memories
        for memory in memories.values() {
            let is_relevant = symbols.iter().any(|symbol| {
                memory.related_symbols.contains(&symbol.path)
                    || memory.content.contains(&symbol.name)
            });

            if is_relevant {
                relevant_memories.push(memory.clone());

                // Extract patterns from coding conventions
                if memory.memory_type == MemoryType::CodingConvention {
                    if let Some(pattern_str) = memory.metadata.get("patterns") {
                        patterns.extend(pattern_str.split(',').map(|s| s.trim().to_string()));
                    }
                }

                // Extract API changes
                if memory.memory_type == MemoryType::ApiChange {
                    if let Ok(change) = serde_json::from_str::<SymbolChange>(&memory.content) {
                        api_changes.push(change);
                    }
                }
            }
        }

        // Generate summary
        let summary = self.generate_context_summary(&relevant_memories);

        Ok(ProjectContext {
            memories: relevant_memories,
            summary,
            patterns,
            api_changes,
        })
    }

    /// Generate a summary of relevant memories
    fn generate_context_summary(&self, memories: &[Memory]) -> String {
        if memories.is_empty() {
            return "No relevant project knowledge found.".to_string();
        }

        let mut summary = format!("Found {} relevant memories:\n", memories.len());

        // Group by type
        let mut by_type: HashMap<String, Vec<&Memory>> = HashMap::new();
        for memory in memories {
            let type_key = format!("{:?}", memory.memory_type);
            by_type.entry(type_key).or_default().push(memory);
        }

        for (memory_type, mems) in by_type {
            summary.push_str(&format!("\n{}: {} items\n", memory_type, mems.len()));
            for mem in mems.iter().take(3) {
                summary.push_str(&format!("  - {}\n", mem.name));
            }
        }

        summary
    }

    /// Record an API change in memory
    pub async fn record_api_change(
        &self,
        change: &SymbolChange,
        impact: &ImpactAnalysis,
    ) -> SemanticResult<()> {
        let memory = Memory {
            id: format!("api_change_{}", Utc::now().timestamp()),
            name: format!("API Change: {}", change.symbol.name),
            content: serde_json::to_string(change)?,
            memory_type: MemoryType::ApiChange,
            related_symbols: vec![change.symbol.path.clone()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("severity".to_string(), format!("{:?}", impact.severity));
                meta.insert(
                    "affected_count".to_string(),
                    impact.affected_symbols.len().to_string(),
                );
                meta
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.store_memory(memory).await
    }

    /// Record a refactoring pattern
    pub async fn record_refactoring(
        &self,
        name: &str,
        description: &str,
        before_pattern: &str,
        after_pattern: &str,
        related_symbols: Vec<String>,
    ) -> SemanticResult<()> {
        let content = format!(
            "Refactoring: {}\n\nDescription: {}\n\nBefore:\n{}\n\nAfter:\n{}",
            name, description, before_pattern, after_pattern
        );

        let memory = Memory {
            id: format!("refactoring_{}", Utc::now().timestamp()),
            name: format!("Refactoring: {}", name),
            content,
            memory_type: MemoryType::Refactoring,
            related_symbols,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.store_memory(memory).await
    }
}
