/// Master Agent Task Delegation System
///
/// This module implements intelligent task delegation where a Master agent
/// analyzes tasks and assigns them to the most appropriate specialized agents.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::agent::{Priority, Task, TaskType};
use crate::identity::AgentRole;

/// Task delegation decision made by the Master agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationDecision {
    /// The task being delegated
    pub task: Task,
    /// Target agent role for execution
    pub target_agent: AgentRole,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning for the delegation
    pub reasoning: String,
    /// Priority adjustment (if any)
    pub priority_adjustment: Option<Priority>,
    /// Estimated completion time in seconds
    pub estimated_duration: Option<u32>,
    /// Dependencies on other tasks
    pub dependencies: Vec<String>,
}

/// Master delegation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationStrategy {
    /// Delegate based on task content analysis
    ContentBased,
    /// Delegate based on agent workload balancing
    LoadBalanced,
    /// Delegate based on agent expertise and past performance
    ExpertiseBased,
    /// Delegate based on task dependencies and workflow
    WorkflowBased,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

/// Agent workload and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_role: AgentRole,
    pub current_tasks: usize,
    pub completed_tasks: usize,
    pub average_completion_time: std::time::Duration,
    pub success_rate: f64,
    pub specialization_score: f64,
    pub availability: f64,
}

/// Master delegation engine
#[derive(Debug)]
pub struct MasterDelegationEngine {
    pub strategy: DelegationStrategy,
    pub agent_metrics: HashMap<String, AgentMetrics>,
    pub task_history: Vec<DelegationDecision>,
    pub delegation_rules: Vec<DelegationRule>,
}

/// Rules for task delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRule {
    pub name: String,
    pub priority: u8,
    pub condition: DelegationCondition,
    pub target_agent: AgentRole,
    pub confidence_boost: f64,
}

/// Conditions for delegation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationCondition {
    /// Task description contains keywords
    DescriptionContains(Vec<String>),
    /// Task type matches
    TaskTypeEquals(TaskType),
    /// Task priority is above threshold
    PriorityAbove(Priority),
    /// Agent has low workload
    AgentWorkloadBelow(f64),
    /// Combined conditions
    And(Vec<DelegationCondition>),
    Or(Vec<DelegationCondition>),
}

impl MasterDelegationEngine {
    /// Create a new master delegation engine
    pub fn new(strategy: DelegationStrategy) -> Self {
        Self {
            strategy,
            agent_metrics: HashMap::new(),
            task_history: Vec::new(),
            delegation_rules: Self::default_delegation_rules(),
        }
    }

    /// Create default delegation rules
    fn default_delegation_rules() -> Vec<DelegationRule> {
        vec![
            DelegationRule {
                name: "Frontend UI Tasks".to_string(),
                priority: 10,
                condition: DelegationCondition::Or(vec![
                    DelegationCondition::DescriptionContains(vec![
                        "html".to_string(),
                        "css".to_string(),
                        "javascript".to_string(),
                        "ui".to_string(),
                        "component".to_string(),
                        "frontend".to_string(),
                        "react".to_string(),
                        "vue".to_string(),
                        "angular".to_string(),
                    ]),
                    DelegationCondition::TaskTypeEquals(TaskType::Feature),
                ]),
                target_agent: AgentRole::Frontend {
                    technologies: vec![
                        "HTML".to_string(),
                        "CSS".to_string(),
                        "JavaScript".to_string(),
                    ],
                    responsibilities: vec!["UI Development".to_string()],
                    boundaries: vec!["No backend work".to_string()],
                },
                confidence_boost: 0.8,
            },
            DelegationRule {
                name: "Backend API Tasks".to_string(),
                priority: 10,
                condition: DelegationCondition::And(vec![
                    DelegationCondition::DescriptionContains(vec![
                        "api".to_string(),
                        "server".to_string(),
                        "database".to_string(),
                        "backend".to_string(),
                        "endpoint".to_string(),
                        "rest".to_string(),
                        "node".to_string(),
                        "express".to_string(),
                    ]),
                    DelegationCondition::TaskTypeEquals(TaskType::Development),
                ]),
                target_agent: AgentRole::Backend {
                    technologies: vec!["Node.js".to_string(), "Express".to_string()],
                    responsibilities: vec!["API Development".to_string()],
                    boundaries: vec!["No frontend work".to_string()],
                },
                confidence_boost: 0.8,
            },
            DelegationRule {
                name: "Testing Tasks".to_string(),
                priority: 9,
                condition: DelegationCondition::Or(vec![
                    DelegationCondition::DescriptionContains(vec![
                        "test".to_string(),
                        "testing".to_string(),
                        "qa".to_string(),
                        "quality".to_string(),
                        "validation".to_string(),
                    ]),
                    DelegationCondition::TaskTypeEquals(TaskType::Testing),
                ]),
                target_agent: AgentRole::QA {
                    technologies: vec![
                        "Jest".to_string(),
                        "Mocha".to_string(),
                        "Puppeteer".to_string(),
                    ],
                    responsibilities: vec!["Testing".to_string(), "Quality Assurance".to_string()],
                    boundaries: vec!["No production code".to_string()],
                },
                confidence_boost: 0.9,
            },
            DelegationRule {
                name: "Infrastructure Tasks".to_string(),
                priority: 9,
                condition: DelegationCondition::Or(vec![
                    DelegationCondition::DescriptionContains(vec![
                        "deploy".to_string(),
                        "ci/cd".to_string(),
                        "docker".to_string(),
                        "infrastructure".to_string(),
                        "pipeline".to_string(),
                    ]),
                    DelegationCondition::TaskTypeEquals(TaskType::Infrastructure),
                ]),
                target_agent: AgentRole::DevOps {
                    technologies: vec!["Docker".to_string(), "GitHub Actions".to_string()],
                    responsibilities: vec!["Deployment".to_string(), "Infrastructure".to_string()],
                    boundaries: vec!["No application code".to_string()],
                },
                confidence_boost: 0.9,
            },
        ]
    }

    /// Update agent metrics
    pub fn update_agent_metrics(&mut self, agent_role: AgentRole, metrics: AgentMetrics) {
        self.agent_metrics
            .insert(agent_role.name().to_string(), metrics);
    }

    /// Delegate a task to the most appropriate agent
    pub fn delegate_task(&mut self, task: Task) -> Result<DelegationDecision> {
        info!(
            "🎯 Master analyzing task for delegation: {}",
            task.description
        );

        let decision = match self.strategy {
            DelegationStrategy::ContentBased => self.delegate_content_based(&task)?,
            DelegationStrategy::LoadBalanced => self.delegate_load_balanced(&task)?,
            DelegationStrategy::ExpertiseBased => self.delegate_expertise_based(&task)?,
            DelegationStrategy::WorkflowBased => self.delegate_workflow_based(&task)?,
            DelegationStrategy::Hybrid => self.delegate_hybrid(&task)?,
        };

        // Record the delegation decision
        self.task_history.push(decision.clone());

        info!(
            "✅ Master delegated task '{}' to {} with {:.1}% confidence",
            task.description,
            decision.target_agent.name(),
            decision.confidence * 100.0
        );

        Ok(decision)
    }

    /// Content-based delegation using rules
    fn delegate_content_based(&self, task: &Task) -> Result<DelegationDecision> {
        let mut best_match: Option<(DelegationRule, f64)> = None;
        let task_lower = task.description.to_lowercase();

        for rule in &self.delegation_rules {
            if let Some(confidence) = Self::evaluate_condition(&rule.condition, task, &task_lower) {
                let total_confidence = confidence + rule.confidence_boost;

                if best_match
                    .as_ref()
                    .is_none_or(|(_, prev_confidence)| total_confidence > *prev_confidence)
                {
                    best_match = Some((rule.clone(), total_confidence.min(1.0)));
                }
            }
        }

        if let Some((rule, confidence)) = best_match {
            Ok(DelegationDecision {
                task: task.clone(),
                target_agent: rule.target_agent,
                confidence,
                reasoning: format!(
                    "Matched rule: {} with {:.1}% confidence",
                    rule.name,
                    confidence * 100.0
                ),
                priority_adjustment: None,
                estimated_duration: task.estimated_duration,
                dependencies: vec![],
            })
        } else {
            // Default to backend agent as most versatile
            Ok(DelegationDecision {
                task: task.clone(),
                target_agent: AgentRole::Backend {
                    technologies: vec!["Node.js".to_string()],
                    responsibilities: vec!["General Development".to_string()],
                    boundaries: vec![],
                },
                confidence: 0.3,
                reasoning: "No specific rule matched, defaulting to backend agent".to_string(),
                priority_adjustment: None,
                estimated_duration: task.estimated_duration,
                dependencies: vec![],
            })
        }
    }

    /// Load-balanced delegation
    fn delegate_load_balanced(&self, task: &Task) -> Result<DelegationDecision> {
        // Find agent with lowest current workload
        let mut best_agent: Option<(AgentRole, f64)> = None;

        for metrics in self.agent_metrics.values() {
            let workload = metrics.current_tasks as f64 / 10.0; // Normalize to 0-1
            let availability_score = metrics.availability * (1.0 - workload);

            if best_agent
                .as_ref()
                .is_none_or(|(_, prev_score)| availability_score > *prev_score)
            {
                best_agent = Some((metrics.agent_role.clone(), availability_score));
            }
        }

        if let Some((agent, score)) = best_agent {
            Ok(DelegationDecision {
                task: task.clone(),
                target_agent: agent,
                confidence: score,
                reasoning: format!(
                    "Load-balanced assignment with availability score {:.1}%",
                    score * 100.0
                ),
                priority_adjustment: None,
                estimated_duration: task.estimated_duration,
                dependencies: vec![],
            })
        } else {
            // Fallback to content-based
            self.delegate_content_based(task)
        }
    }

    /// Expertise-based delegation
    fn delegate_expertise_based(&self, task: &Task) -> Result<DelegationDecision> {
        let mut best_match: Option<(AgentRole, f64)> = None;

        for metrics in self.agent_metrics.values() {
            let expertise_score = metrics.specialization_score * metrics.success_rate;

            if best_match
                .as_ref()
                .is_none_or(|(_, prev_score)| expertise_score > *prev_score)
            {
                best_match = Some((metrics.agent_role.clone(), expertise_score));
            }
        }

        if let Some((agent, score)) = best_match {
            Ok(DelegationDecision {
                task: task.clone(),
                target_agent: agent,
                confidence: score,
                reasoning: format!(
                    "Expertise-based assignment with score {:.1}%",
                    score * 100.0
                ),
                priority_adjustment: None,
                estimated_duration: task.estimated_duration,
                dependencies: vec![],
            })
        } else {
            // Fallback to content-based
            self.delegate_content_based(task)
        }
    }

    /// Workflow-based delegation (considering dependencies)
    fn delegate_workflow_based(&self, task: &Task) -> Result<DelegationDecision> {
        // Analyze task dependencies and workflow
        // For now, fall back to content-based with workflow awareness
        let mut decision = self.delegate_content_based(task)?;

        // Add workflow analysis
        decision.reasoning = format!("{} (workflow-aware)", decision.reasoning);

        Ok(decision)
    }

    /// Hybrid delegation strategy
    fn delegate_hybrid(&self, task: &Task) -> Result<DelegationDecision> {
        // Combine multiple strategies
        let content_decision = self.delegate_content_based(task)?;
        let load_decision = self.delegate_load_balanced(task)?;

        // Choose the decision with higher confidence
        if content_decision.confidence >= load_decision.confidence {
            Ok(DelegationDecision {
                reasoning: format!("Hybrid: Content-based ({})", content_decision.reasoning),
                ..content_decision
            })
        } else {
            Ok(DelegationDecision {
                reasoning: format!("Hybrid: Load-based ({})", load_decision.reasoning),
                ..load_decision
            })
        }
    }

    /// Evaluate a delegation condition
    fn evaluate_condition(
        condition: &DelegationCondition,
        task: &Task,
        task_lower: &str,
    ) -> Option<f64> {
        match condition {
            DelegationCondition::DescriptionContains(keywords) => {
                let matches = keywords
                    .iter()
                    .filter(|keyword| task_lower.contains(&keyword.to_lowercase()))
                    .count();
                if matches > 0 {
                    Some(matches as f64 / keywords.len() as f64)
                } else {
                    None
                }
            }
            DelegationCondition::TaskTypeEquals(task_type) => {
                if task.task_type == *task_type {
                    Some(1.0)
                } else {
                    None
                }
            }
            DelegationCondition::PriorityAbove(priority) => {
                if task.priority as u8 >= *priority as u8 {
                    Some(0.5 + (task.priority as u8 as f64 / 10.0))
                } else {
                    None
                }
            }
            DelegationCondition::AgentWorkloadBelow(_threshold) => {
                // This would require current agent workload info
                Some(0.5) // Placeholder
            }
            DelegationCondition::And(conditions) => {
                let scores: Vec<f64> = conditions
                    .iter()
                    .filter_map(|c| Self::evaluate_condition(c, task, task_lower))
                    .collect();

                if scores.len() == conditions.len() {
                    Some(scores.iter().sum::<f64>() / scores.len() as f64)
                } else {
                    None
                }
            }
            DelegationCondition::Or(conditions) => conditions
                .iter()
                .filter_map(|c| Self::evaluate_condition(c, task, task_lower))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
        }
    }

    /// Get delegation statistics
    pub fn get_delegation_stats(&self) -> DelegationStats {
        let total_delegations = self.task_history.len();
        let mut agent_counts = HashMap::new();
        let mut total_confidence = 0.0;

        for decision in &self.task_history {
            *agent_counts
                .entry(decision.target_agent.name().to_string())
                .or_insert(0) += 1;
            total_confidence += decision.confidence;
        }

        DelegationStats {
            total_delegations,
            average_confidence: if total_delegations > 0 {
                total_confidence / total_delegations as f64
            } else {
                0.0
            },
            agent_distribution: agent_counts,
            strategy: self.strategy.clone(),
        }
    }
}

/// Delegation statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegationStats {
    pub total_delegations: usize,
    pub average_confidence: f64,
    pub agent_distribution: HashMap<String, usize>,
    pub strategy: DelegationStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_delegation_engine_creation() {
        let engine = MasterDelegationEngine::new(DelegationStrategy::ContentBased);
        assert_eq!(engine.delegation_rules.len(), 4);
        assert!(engine.task_history.is_empty());
    }

    #[test]
    fn test_content_based_delegation() {
        let mut engine = MasterDelegationEngine::new(DelegationStrategy::ContentBased);

        let task = Task::new(
            "test-1".to_string(),
            "Create React component with CSS styling for user interface".to_string(),
            Priority::High,
            TaskType::Feature,
        );

        let decision = engine.delegate_task(task).unwrap();

        assert!(matches!(decision.target_agent, AgentRole::Frontend { .. }));
        assert!(decision.confidence > 0.5);
        assert!(decision.reasoning.contains("Frontend"));
    }

    #[test]
    fn test_testing_task_delegation() {
        let mut engine = MasterDelegationEngine::new(DelegationStrategy::ContentBased);

        let task = Task::new(
            "test-2".to_string(),
            "Write unit tests for API endpoints".to_string(),
            Priority::High,
            TaskType::Testing,
        );

        let decision = engine.delegate_task(task).unwrap();

        assert!(matches!(decision.target_agent, AgentRole::QA { .. }));
        assert!(decision.confidence > 0.8);
    }
}
