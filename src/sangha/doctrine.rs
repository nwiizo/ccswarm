//! Doctrine management for Sangha principles and rules

use super::*;
use crate::sangha::proposal::DoctrineCategory;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Manages the doctrines (principles and rules) of the Sangha
#[derive(Debug)]
pub struct DoctrineManager {
    doctrines: Arc<RwLock<HashMap<Uuid, Doctrine>>>,
    categories: Arc<RwLock<HashMap<DoctrineCategory, Vec<Uuid>>>>,
}

/// Represents a doctrine or principle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doctrine {
    pub id: Uuid,
    pub category: DoctrineCategory,
    pub title: String,
    pub content: String,
    pub rationale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
    pub version: u32,
    pub status: DoctrineStatus,
    pub precedence: u32, // Higher number = higher precedence
    pub related_doctrines: Vec<Uuid>,
    pub examples: Vec<DoctrineExample>,
}

/// Status of a doctrine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctrineStatus {
    Draft,
    Active,
    Deprecated,
    Superseded,
}

/// Example demonstrating a doctrine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineExample {
    pub description: String,
    pub scenario: String,
    pub correct_application: String,
    pub incorrect_application: Option<String>,
}

/// Core principles that are immutable
#[derive(Debug, Clone)]
pub struct CorePrinciples {
    pub agent_autonomy: String,
    pub collective_benefit: String,
    pub continuous_improvement: String,
    pub transparency: String,
    pub accountability: String,
}

impl Default for CorePrinciples {
    fn default() -> Self {
        Self {
            agent_autonomy: "Each agent maintains autonomy within their domain while respecting collective decisions".to_string(),
            collective_benefit: "Decisions should benefit the system as a whole, not individual agents".to_string(),
            continuous_improvement: "The system should constantly evolve and improve through learning".to_string(),
            transparency: "All decisions and processes should be transparent and auditable".to_string(),
            accountability: "Agents are accountable for their actions and decisions".to_string(),
        }
    }
}

impl DoctrineManager {
    pub fn new() -> Self {
        let manager = Self {
            doctrines: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize with core principles
        // Note: We're not initializing in the constructor to avoid ownership issues
        // Call initialize_core_principles() separately after creation
        
        manager
    }

    /// Initialize core principles
    pub async fn initialize_core_principles(&self) -> Result<()> {
        let principles = CorePrinciples::default();
        
        // Agent Autonomy
        self.add_doctrine(Doctrine {
            id: Uuid::new_v4(),
            category: DoctrineCategory::CorePrinciple,
            title: "Agent Autonomy".to_string(),
            content: principles.agent_autonomy,
            rationale: "Ensures specialization and efficient task execution".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: "system".to_string(),
            version: 1,
            status: DoctrineStatus::Active,
            precedence: 100,
            related_doctrines: vec![],
            examples: vec![
                DoctrineExample {
                    description: "Frontend agent making UI decisions".to_string(),
                    scenario: "Choosing between React components".to_string(),
                    correct_application: "Frontend agent independently selects appropriate component".to_string(),
                    incorrect_application: Some("Backend agent dictating UI choices".to_string()),
                }
            ],
        }).await?;

        // Collective Benefit
        self.add_doctrine(Doctrine {
            id: Uuid::new_v4(),
            category: DoctrineCategory::CorePrinciple,
            title: "Collective Benefit".to_string(),
            content: principles.collective_benefit,
            rationale: "Prevents selfish behavior and ensures system cohesion".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: "system".to_string(),
            version: 1,
            status: DoctrineStatus::Active,
            precedence: 100,
            related_doctrines: vec![],
            examples: vec![],
        }).await?;

        Ok(())
    }

    /// Add a new doctrine
    pub async fn add_doctrine(&self, doctrine: Doctrine) -> Result<()> {
        let mut doctrines = self.doctrines.write().await;
        let mut categories = self.categories.write().await;
        
        categories.entry(doctrine.category)
            .or_insert_with(Vec::new)
            .push(doctrine.id);
            
        doctrines.insert(doctrine.id, doctrine);
        
        Ok(())
    }

    /// Update an existing doctrine
    pub async fn update_doctrine(&self, id: Uuid, updates: DoctrineUpdate) -> Result<()> {
        let mut doctrines = self.doctrines.write().await;
        
        let doctrine = doctrines.get_mut(&id)
            .context("Doctrine not found")?;
            
        if let Some(content) = updates.content {
            doctrine.content = content;
        }
        
        if let Some(rationale) = updates.rationale {
            doctrine.rationale = rationale;
        }
        
        if let Some(examples) = updates.examples {
            doctrine.examples = examples;
        }
        
        doctrine.updated_at = Utc::now();
        doctrine.version += 1;
        
        Ok(())
    }

    /// Get all active doctrines
    pub async fn get_active_doctrines(&self) -> Vec<Doctrine> {
        let doctrines = self.doctrines.read().await;
        doctrines.values()
            .filter(|d| d.status == DoctrineStatus::Active)
            .cloned()
            .collect()
    }

    /// Get doctrines by category
    pub async fn get_by_category(&self, category: DoctrineCategory) -> Vec<Doctrine> {
        let categories = self.categories.read().await;
        let doctrines = self.doctrines.read().await;
        
        if let Some(ids) = categories.get(&category) {
            ids.iter()
                .filter_map(|id| doctrines.get(id))
                .filter(|d| d.status == DoctrineStatus::Active)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    /// Check if an action complies with doctrines
    pub async fn check_compliance(&self, action: &ProposedAction) -> ComplianceResult {
        let doctrines = self.get_active_doctrines().await;
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        
        for doctrine in &doctrines {
            let check = self.check_doctrine_compliance(&doctrine, action);
            match check {
                ComplianceCheck::Compliant => continue,
                ComplianceCheck::Warning(msg) => warnings.push(msg),
                ComplianceCheck::Violation(msg) => violations.push(msg),
            }
        }
        
        ComplianceResult {
            compliant: violations.is_empty(),
            violations,
            warnings,
        }
    }

    /// Check compliance with a specific doctrine
    fn check_doctrine_compliance(&self, _doctrine: &Doctrine, _action: &ProposedAction) -> ComplianceCheck {
        // This would contain actual compliance logic
        // For now, return compliant
        ComplianceCheck::Compliant
    }

    /// Get doctrine history
    pub async fn get_doctrine_history(&self, _id: Uuid) -> Vec<DoctrineVersion> {
        // In a real implementation, this would retrieve from version storage
        vec![]
    }

    /// Deprecate a doctrine
    pub async fn deprecate_doctrine(&self, id: Uuid, _reason: String) -> Result<()> {
        let mut doctrines = self.doctrines.write().await;
        
        let doctrine = doctrines.get_mut(&id)
            .context("Doctrine not found")?;
            
        doctrine.status = DoctrineStatus::Deprecated;
        doctrine.updated_at = Utc::now();
        
        Ok(())
    }
}

/// Update structure for doctrines
#[derive(Debug, Clone)]
pub struct DoctrineUpdate {
    pub content: Option<String>,
    pub rationale: Option<String>,
    pub examples: Option<Vec<DoctrineExample>>,
}

/// Proposed action to check against doctrines
#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub action_type: String,
    pub agent_id: String,
    pub description: String,
    pub affects: Vec<String>,
}

/// Result of compliance check
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

/// Compliance check result
#[derive(Debug, Clone)]
enum ComplianceCheck {
    Compliant,
    Warning(String),
    Violation(String),
}

/// Version history of a doctrine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineVersion {
    pub version: u32,
    pub content: String,
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
    pub change_reason: String,
}

/// Doctrine interpreter for natural language queries
pub struct DoctrineInterpreter {
    doctrines: Arc<DoctrineManager>,
}

impl DoctrineInterpreter {
    pub fn new(doctrines: Arc<DoctrineManager>) -> Self {
        Self { doctrines }
    }

    /// Interpret a query about doctrines
    pub async fn interpret_query(&self, _query: &str) -> InterpretationResult {
        // This would use NLP to understand the query
        // For now, return a simple result
        InterpretationResult {
            relevant_doctrines: vec![],
            interpretation: "Query interpretation not implemented".to_string(),
            confidence: 0.0,
        }
    }

    /// Find relevant doctrines for a situation
    pub async fn find_relevant_doctrines(&self, _situation: &str) -> Vec<Doctrine> {
        // This would use semantic search
        vec![]
    }
}

/// Result of doctrine interpretation
#[derive(Debug, Clone)]
pub struct InterpretationResult {
    pub relevant_doctrines: Vec<Uuid>,
    pub interpretation: String,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_doctrine_manager() {
        let manager = DoctrineManager::new();
        
        let doctrine = Doctrine {
            id: Uuid::new_v4(),
            category: DoctrineCategory::OperationalGuideline,
            title: "Test Doctrine".to_string(),
            content: "Test content".to_string(),
            rationale: "Test rationale".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: "test".to_string(),
            version: 1,
            status: DoctrineStatus::Active,
            precedence: 50,
            related_doctrines: vec![],
            examples: vec![],
        };
        
        manager.add_doctrine(doctrine.clone()).await.unwrap();
        
        let active = manager.get_active_doctrines().await;
        assert!(!active.is_empty());
        
        let by_category = manager.get_by_category(DoctrineCategory::OperationalGuideline).await;
        assert!(!by_category.is_empty());
    }
}