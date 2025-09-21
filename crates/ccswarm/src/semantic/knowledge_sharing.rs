//! Knowledge sharing system for inter-agent communication
//!
//! Enables semantic knowledge sharing between subagents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::analyzer::{ChangeType, Symbol, SymbolChange};
use super::memory::{Memory, MemoryType, ProjectMemory};
use super::symbol_index::SymbolIndex;
use super::SemanticResult;

/// Pattern library for code patterns
#[derive(Debug)]
pub struct PatternLibrary {
    patterns: RwLock<HashMap<String, CodePattern>>,
}

/// Code pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub name: String,
    pub description: String,
    pub language: String,
    pub pattern: String,
    pub example: String,
    pub tags: Vec<String>,
}

/// Shared symbol registry
#[derive(Debug)]
pub struct SymbolRegistry {
    symbols: RwLock<HashMap<String, RegisteredSymbol>>,
}

/// Registered symbol with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSymbol {
    pub symbol: Symbol,
    pub owner_agent: String,
    pub visibility: SymbolVisibility,
    pub contracts: Vec<Contract>,
}

/// Symbol visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolVisibility {
    Public,
    Internal,
    Private,
}

/// Contract definition for symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub description: String,
    pub constraints: Vec<String>,
}

/// API impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiImpact {
    pub change: SymbolChange,
    pub affected_agents: Vec<String>,
    pub breaking_changes: Vec<BreakingChange>,
    pub migration_steps: Vec<String>,
}

/// Breaking change definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub description: String,
    pub severity: BreakingSeverity,
    pub affected_symbols: Vec<String>,
}

/// Breaking change severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BreakingSeverity {
    Minor,
    Major,
    Critical,
}

/// Backend task generated from frontend changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendTask {
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub api_changes: Vec<ApiChange>,
    pub estimated_effort: String,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// API change details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChange {
    pub endpoint: String,
    pub method: String,
    pub change_type: ApiChangeType,
    pub description: String,
}

/// Type of API change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiChangeType {
    NewEndpoint,
    ModifiedEndpoint,
    DeletedEndpoint,
    ParameterChange,
    ResponseChange,
}

/// Knowledge sharing coordinator
pub struct SemanticKnowledgeSharing {
    shared_memory: Arc<ProjectMemory>,
    symbol_registry: Arc<RwLock<SymbolRegistry>>,
    pattern_library: Arc<RwLock<PatternLibrary>>,
    symbol_index: Arc<SymbolIndex>,
}

impl SemanticKnowledgeSharing {
    /// Create a new knowledge sharing system
    pub async fn new(
        shared_memory: Arc<ProjectMemory>,
        symbol_index: Arc<SymbolIndex>,
    ) -> SemanticResult<Self> {
        Ok(Self {
            shared_memory,
            symbol_registry: Arc::new(RwLock::new(SymbolRegistry {
                symbols: RwLock::new(HashMap::new()),
            })),
            pattern_library: Arc::new(RwLock::new(PatternLibrary {
                patterns: RwLock::new(HashMap::new()),
            })),
            symbol_index,
        })
    }

    /// Register a symbol for sharing
    pub async fn register_symbol(
        &self,
        symbol: Symbol,
        owner_agent: String,
        visibility: SymbolVisibility,
    ) -> SemanticResult<()> {
        let registry = self.symbol_registry.read().await;
        let mut symbols = registry.symbols.write().await;

        let registered = RegisteredSymbol {
            symbol: symbol.clone(),
            owner_agent,
            visibility,
            contracts: Vec::new(),
        };

        symbols.insert(symbol.path.clone(), registered);
        Ok(())
    }

    /// Propagate API changes from frontend to backend
    pub async fn propagate_api_changes(
        &self,
        frontend_changes: &[SymbolChange],
    ) -> SemanticResult<Vec<BackendTask>> {
        let mut backend_tasks = Vec::new();

        for change in frontend_changes {
            if change.affects_api() {
                // Analyze API impact
                let api_impact = self.analyze_api_impact(change).await?;

                // Generate backend task
                let task = self.generate_backend_task(&api_impact).await?;
                backend_tasks.push(task);

                // Record in shared memory
                self.shared_memory
                    .record_api_change(
                        change,
                        &super::analyzer::ImpactAnalysis {
                            change: change.clone(),
                            affected_symbols: Vec::new(),
                            severity: super::analyzer::ImpactSeverity::High,
                            suggested_actions: api_impact.migration_steps.clone(),
                        },
                    )
                    .await?;
            }
        }

        Ok(backend_tasks)
    }

    /// Analyze API impact of a change
    pub async fn analyze_api_impact(&self, change: &SymbolChange) -> SemanticResult<ApiImpact> {
        let mut affected_agents = Vec::new();
        let mut breaking_changes = Vec::new();
        let mut migration_steps = Vec::new();

        // Determine affected agents
        match change.symbol.kind {
            super::analyzer::SymbolKind::Interface
            | super::analyzer::SymbolKind::Function
            | super::analyzer::SymbolKind::Method => {
                affected_agents.push("backend".to_string());
                if change.change_type == ChangeType::Deleted {
                    affected_agents.push("qa".to_string());
                }
            }
            _ => {}
        }

        // Identify breaking changes
        if change.change_type == ChangeType::Deleted {
            breaking_changes.push(BreakingChange {
                description: format!("Symbol {} has been deleted", change.symbol.name),
                severity: BreakingSeverity::Critical,
                affected_symbols: vec![change.symbol.path.clone()],
            });
            migration_steps.push("Remove all references to the deleted symbol".to_string());
        } else if change.change_type == ChangeType::ApiModification {
            breaking_changes.push(BreakingChange {
                description: format!("API signature changed for {}", change.symbol.name),
                severity: BreakingSeverity::Major,
                affected_symbols: vec![change.symbol.path.clone()],
            });
            migration_steps.push("Update all calls to match new signature".to_string());
        }

        Ok(ApiImpact {
            change: change.clone(),
            affected_agents,
            breaking_changes,
            migration_steps,
        })
    }

    /// Generate backend task from API impact
    async fn generate_backend_task(&self, impact: &ApiImpact) -> SemanticResult<BackendTask> {
        let priority = if impact
            .breaking_changes
            .iter()
            .any(|bc| bc.severity == BreakingSeverity::Critical)
        {
            TaskPriority::Critical
        } else if impact
            .breaking_changes
            .iter()
            .any(|bc| bc.severity == BreakingSeverity::Major)
        {
            TaskPriority::High
        } else {
            TaskPriority::Medium
        };

        let api_changes: Vec<ApiChange> = impact
            .breaking_changes
            .iter()
            .map(|bc| ApiChange {
                endpoint: format!("/api/{}", impact.change.symbol.name.to_lowercase()),
                method: "POST".to_string(),
                change_type: match impact.change.change_type {
                    ChangeType::Added => ApiChangeType::NewEndpoint,
                    ChangeType::Modified | ChangeType::ApiModification => {
                        ApiChangeType::ModifiedEndpoint
                    }
                    ChangeType::Deleted => ApiChangeType::DeletedEndpoint,
                    _ => ApiChangeType::ParameterChange,
                },
                description: bc.description.clone(),
            })
            .collect();

        Ok(BackendTask {
            title: format!("Update backend for {} changes", impact.change.symbol.name),
            description: format!(
                "Frontend component {} has been modified. Backend updates required:\n{}",
                impact.change.symbol.name,
                impact.migration_steps.join("\n")
            ),
            priority,
            api_changes,
            estimated_effort: self.estimate_effort(&impact),
        })
    }

    /// Estimate effort for a task
    fn estimate_effort(&self, impact: &ApiImpact) -> String {
        let critical_count = impact
            .breaking_changes
            .iter()
            .filter(|bc| bc.severity == BreakingSeverity::Critical)
            .count();
        let major_count = impact
            .breaking_changes
            .iter()
            .filter(|bc| bc.severity == BreakingSeverity::Major)
            .count();

        if critical_count > 0 {
            "High (4-8 hours)".to_string()
        } else if major_count > 0 {
            "Medium (2-4 hours)".to_string()
        } else {
            "Low (1-2 hours)".to_string()
        }
    }

    /// Share a code pattern
    pub async fn share_pattern(&self, pattern: CodePattern) -> SemanticResult<()> {
        let library = self.pattern_library.read().await;
        let mut patterns = library.patterns.write().await;
        patterns.insert(pattern.name.clone(), pattern.clone());

        // Store in memory for persistence
        let memory = Memory {
            id: format!("pattern_{}", chrono::Utc::now().timestamp()),
            name: format!("Pattern: {}", pattern.name),
            content: serde_json::to_string(&pattern)?,
            memory_type: MemoryType::CodingConvention,
            related_symbols: Vec::new(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("language".to_string(), pattern.language);
                meta.insert("tags".to_string(), pattern.tags.join(","));
                meta
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            access_count: 0,
        };

        self.shared_memory.store_memory(memory).await?;
        Ok(())
    }

    /// Get shared patterns by tag
    pub async fn get_patterns_by_tag(&self, tag: &str) -> SemanticResult<Vec<CodePattern>> {
        let library = self.pattern_library.read().await;
        let patterns = library.patterns.read().await;

        let matching: Vec<CodePattern> = patterns
            .values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .cloned()
            .collect();

        Ok(matching)
    }

    /// Share knowledge between agents
    pub async fn share_knowledge(
        &self,
        from_agent: &str,
        to_agent: &str,
        knowledge: &str,
    ) -> SemanticResult<()> {
        let memory = Memory {
            id: format!("shared_{}", chrono::Utc::now().timestamp()),
            name: format!("Shared: {} -> {}", from_agent, to_agent),
            content: knowledge.to_string(),
            memory_type: MemoryType::Other("SharedKnowledge".to_string()),
            related_symbols: Vec::new(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("from_agent".to_string(), from_agent.to_string());
                meta.insert("to_agent".to_string(), to_agent.to_string());
                meta
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            access_count: 0,
        };

        self.shared_memory.store_memory(memory).await
    }
}
