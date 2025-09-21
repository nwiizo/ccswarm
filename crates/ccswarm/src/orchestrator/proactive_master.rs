use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::agent::search_agent::{SearchRequest, SearchResponse};
use crate::agent::{AgentStatus, ClaudeCodeAgent, Priority, Task, TaskResult, TaskType};
use crate::coordination::{AgentMessage, CoordinationBus, CoordinationType};

/// Proactive Master Claude intelligence system
pub struct ProactiveMaster {
    /// Project context and goals
    project_context: Arc<RwLock<ProjectContext>>,

    /// Task dependency graph
    dependency_graph: Arc<RwLock<DependencyGraph>>,

    /// Progress analyzer
    progress_analyzer: Arc<RwLock<ProgressAnalyzer>>,

    /// Task predictor
    task_predictor: Arc<RwLock<TaskPredictor>>,

    /// Goal tracker
    goal_tracker: Arc<RwLock<GoalTracker>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_type: String,
    pub tech_stack: Vec<String>,
    pub features: Vec<String>,
    pub current_phase: DevelopmentPhase,
    pub milestones: Vec<Milestone>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopmentPhase {
    Planning,
    Setup,
    Development,
    Testing,
    Deployment,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub deadline: Option<DateTime<Utc>>,
    pub completion_percentage: f64,
    pub dependencies: Vec<String>,
    pub critical_path: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, TaskNode>,
    pub edges: HashMap<String, Vec<String>>, // task_id -> dependent_task_ids
    pub reverse_edges: HashMap<String, Vec<String>>, // task_id -> prerequisite_task_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub task_id: String,
    pub task_type: TaskType,
    pub status: TaskNodeStatus,
    pub estimated_duration: u64, // minutes
    pub actual_duration: Option<u64>,
    pub agent_id: Option<String>,
    pub priority: Priority,
    pub blocking_others: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskNodeStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressAnalyzer {
    pub velocity_history: Vec<VelocityPoint>,
    pub bottlenecks: Vec<Bottleneck>,
    pub efficiency_metrics: HashMap<String, f64>, // agent_id -> efficiency
    pub prediction_accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityPoint {
    pub timestamp: DateTime<Utc>,
    pub tasks_completed: u32,
    pub story_points: u32,
    pub team_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub agent_id: String,
    pub task_type: TaskType,
    pub queue_length: u32,
    pub average_wait_time: u64, // minutes
    pub severity: BottleneckSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPredictor {
    pub pattern_library: HashMap<String, TaskPattern>,
    pub completion_patterns: Vec<CompletionPattern>,
    pub feature_templates: HashMap<String, FeatureTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub pattern_id: String,
    pub trigger_conditions: Vec<String>,
    pub generated_tasks: Vec<TaskTemplate>,
    pub confidence: f64,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionPattern {
    pub completed_task_type: TaskType,
    pub follow_up_tasks: Vec<TaskTemplate>,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub description_template: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub estimated_duration: u64,
    pub required_agent_type: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureTemplate {
    pub feature_name: String,
    pub required_tasks: Vec<TaskTemplate>,
    pub optional_tasks: Vec<TaskTemplate>,
    pub typical_duration: u64, // hours
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTracker {
    pub objectives: Vec<Objective>,
    pub key_results: Vec<KeyResult>,
    pub current_sprint: Option<Sprint>,
    pub backlog: Vec<BacklogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub title: String,
    pub description: String,
    pub deadline: Option<DateTime<Utc>>,
    pub progress: f64,            // 0.0 to 1.0
    pub key_results: Vec<String>, // IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResult {
    pub id: String,
    pub objective_id: String,
    pub description: String,
    pub target_value: f64,
    pub current_value: f64,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Percentage,
    Count,
    Duration,
    Quality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub tasks: Vec<String>, // task IDs
    pub velocity_target: u32,
    pub current_velocity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub story_points: u32,
    pub estimated_tasks: Vec<TaskTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveDecision {
    pub decision_type: DecisionType,
    pub reasoning: String,
    pub confidence: f64,
    pub suggested_actions: Vec<SuggestedAction>,
    pub risk_assessment: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionType {
    GenerateTask,
    ReassignTask,
    ScaleTeam,
    ChangeStrategy,
    RequestIntervention,
    RequestSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub expected_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl ProactiveMaster {
    pub async fn new() -> Result<Self> {
        let project_context = Arc::new(RwLock::new(ProjectContext {
            project_type: "web_application".to_string(),
            tech_stack: vec![
                "React".to_string(),
                "Node.js".to_string(),
                "PostgreSQL".to_string(),
            ],
            features: vec![],
            current_phase: DevelopmentPhase::Planning,
            milestones: vec![],
            constraints: vec![],
        }));

        let dependency_graph = Arc::new(RwLock::new(DependencyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }));

        let progress_analyzer = Arc::new(RwLock::new(ProgressAnalyzer {
            velocity_history: vec![],
            bottlenecks: vec![],
            efficiency_metrics: HashMap::new(),
            prediction_accuracy: 0.8,
        }));

        let task_predictor = Arc::new(RwLock::new(TaskPredictor {
            pattern_library: Self::initialize_pattern_library(),
            completion_patterns: Self::initialize_completion_patterns(),
            feature_templates: Self::initialize_feature_templates(),
        }));

        let goal_tracker = Arc::new(RwLock::new(GoalTracker {
            objectives: vec![],
            key_results: vec![],
            current_sprint: None,
            backlog: vec![],
        }));

        Ok(Self {
            project_context,
            dependency_graph,
            progress_analyzer,
            task_predictor,
            goal_tracker,
        })
    }

    /// Initialize predefined task patterns
    fn initialize_pattern_library() -> HashMap<String, TaskPattern> {
        let mut patterns = HashMap::new();

        // Frontend component pattern
        patterns.insert(
            "frontend_component".to_string(),
            TaskPattern {
                pattern_id: "frontend_component".to_string(),
                trigger_conditions: vec!["component created".to_string()],
                generated_tasks: vec![
                    TaskTemplate {
                        description_template: "Write unit tests for {component_name} component"
                            .to_string(),
                        task_type: TaskType::Testing,
                        priority: Priority::High,
                        estimated_duration: 30,
                        required_agent_type: "QA".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Add {component_name} to component library docs"
                            .to_string(),
                        task_type: TaskType::Documentation,
                        priority: Priority::Medium,
                        estimated_duration: 15,
                        required_agent_type: "Frontend".to_string(),
                        variables: HashMap::new(),
                    },
                ],
                confidence: 0.95,
                usage_count: 0,
            },
        );

        // API endpoint pattern
        patterns.insert(
            "api_endpoint".to_string(),
            TaskPattern {
                pattern_id: "api_endpoint".to_string(),
                trigger_conditions: vec!["API endpoint created".to_string()],
                generated_tasks: vec![
                    TaskTemplate {
                        description_template: "Write integration tests for {endpoint_name} API"
                            .to_string(),
                        task_type: TaskType::Testing,
                        priority: Priority::High,
                        estimated_duration: 45,
                        required_agent_type: "QA".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Update API documentation for {endpoint_name}"
                            .to_string(),
                        task_type: TaskType::Documentation,
                        priority: Priority::Medium,
                        estimated_duration: 20,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Add rate limiting to {endpoint_name} endpoint"
                            .to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::Medium,
                        estimated_duration: 25,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                ],
                confidence: 0.9,
                usage_count: 0,
            },
        );

        patterns
    }

    /// Initialize completion patterns
    fn initialize_completion_patterns() -> Vec<CompletionPattern> {
        vec![
            CompletionPattern {
                completed_task_type: TaskType::Development,
                follow_up_tasks: vec![TaskTemplate {
                    description_template: "Test the implemented functionality".to_string(),
                    task_type: TaskType::Testing,
                    priority: Priority::High,
                    estimated_duration: 30,
                    required_agent_type: "QA".to_string(),
                    variables: HashMap::new(),
                }],
                probability: 0.85,
            },
            CompletionPattern {
                completed_task_type: TaskType::Testing,
                follow_up_tasks: vec![TaskTemplate {
                    description_template: "Update documentation with test results".to_string(),
                    task_type: TaskType::Documentation,
                    priority: Priority::Low,
                    estimated_duration: 15,
                    required_agent_type: "QA".to_string(),
                    variables: HashMap::new(),
                }],
                probability: 0.6,
            },
        ]
    }

    /// Initialize feature templates
    fn initialize_feature_templates() -> HashMap<String, FeatureTemplate> {
        let mut templates = HashMap::new();

        templates.insert(
            "user_authentication".to_string(),
            FeatureTemplate {
                feature_name: "User Authentication".to_string(),
                required_tasks: vec![
                    TaskTemplate {
                        description_template: "Create user registration API endpoint".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::High,
                        estimated_duration: 120,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Create user login API endpoint".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::High,
                        estimated_duration: 90,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Create registration form component".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::High,
                        estimated_duration: 60,
                        required_agent_type: "Frontend".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Create login form component".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::High,
                        estimated_duration: 45,
                        required_agent_type: "Frontend".to_string(),
                        variables: HashMap::new(),
                    },
                ],
                optional_tasks: vec![
                    TaskTemplate {
                        description_template: "Add social login integration".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::Medium,
                        estimated_duration: 180,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                    TaskTemplate {
                        description_template: "Add password reset functionality".to_string(),
                        task_type: TaskType::Development,
                        priority: Priority::Medium,
                        estimated_duration: 90,
                        required_agent_type: "Backend".to_string(),
                        variables: HashMap::new(),
                    },
                ],
                typical_duration: 8, // hours
            },
        );

        templates
    }

    /// Analyze current progress and make proactive decisions
    pub async fn analyze_and_decide(
        &self,
        agents: &DashMap<String, ClaudeCodeAgent>,
        coordination_bus: &CoordinationBus,
    ) -> Result<Vec<ProactiveDecision>> {
        debug!("Starting proactive analysis");

        let mut decisions = Vec::new();

        // 1. Analyze agent progress
        let progress_decisions = self.analyze_agent_progress(agents).await?;
        decisions.extend(progress_decisions);

        // 2. Check for blocked dependencies
        let dependency_decisions = self.resolve_dependencies().await?;
        decisions.extend(dependency_decisions);

        // 3. Predict next tasks based on completion patterns
        let prediction_decisions = self.predict_next_tasks(agents).await?;
        decisions.extend(prediction_decisions);

        // 4. Monitor goal progress
        let goal_decisions = self.monitor_goals().await?;
        decisions.extend(goal_decisions);

        // 5. Analyze for search needs
        let search_decisions = self.analyze_search_needs(agents).await?;
        decisions.extend(search_decisions);

        // Execute high-confidence decisions automatically
        for decision in &decisions {
            if decision.confidence > 0.8 && decision.risk_assessment == RiskLevel::Low {
                self.execute_decision(decision, coordination_bus).await?;
            }
        }

        info!("Generated {} proactive decisions", decisions.len());
        Ok(decisions)
    }

    /// Analyze individual agent progress
    async fn analyze_agent_progress(
        &self,
        agents: &DashMap<String, ClaudeCodeAgent>,
    ) -> Result<Vec<ProactiveDecision>> {
        let mut decisions = Vec::new();
        let mut analyzer = self.progress_analyzer.write().await;

        for entry in agents.iter() {
            let agent = entry.value();
            let agent_id = &agent.identity.agent_id;

            // Check if agent is stuck
            if matches!(agent.status, AgentStatus::Working) {
                let time_since_activity = Utc::now() - agent.last_activity;
                if time_since_activity.num_minutes() > 15 {
                    decisions.push(ProactiveDecision {
                        decision_type: DecisionType::RequestIntervention,
                        reasoning: format!(
                            "Agent {} has been working on the same task for {} minutes without progress",
                            agent_id, time_since_activity.num_minutes()
                        ),
                        confidence: 0.9,
                        suggested_actions: vec![
                            SuggestedAction {
                                action_type: "check_agent_status".to_string(),
                                description: "Check if agent needs assistance".to_string(),
                                parameters: HashMap::from([
                                    ("agent_id".to_string(), agent_id.clone()),
                                ]),
                                expected_impact: "Unblock agent or reassign task".to_string(),
                            },
                        ],
                        risk_assessment: RiskLevel::Low,
                    });
                }
            }

            // Analyze agent efficiency
            let recent_tasks: Vec<_> = agent.task_history.iter().rev().take(5).collect();
            if recent_tasks.len() >= 3 {
                let avg_completion_time: f64 = recent_tasks
                    .iter()
                    .filter_map(|(_task, result)| {
                        if result.success {
                            Some(result.duration.as_secs() as f64 / 60.0)
                        } else {
                            None
                        }
                    })
                    .sum::<f64>()
                    / recent_tasks.len() as f64;

                analyzer
                    .efficiency_metrics
                    .insert(agent_id.clone(), avg_completion_time);
            }
        }

        Ok(decisions)
    }

    /// Resolve dependency conflicts
    async fn resolve_dependencies(&self) -> Result<Vec<ProactiveDecision>> {
        let mut decisions = Vec::new();
        let graph = self.dependency_graph.read().await;

        // Find blocked tasks
        for (task_id, node) in &graph.nodes {
            if node.status == TaskNodeStatus::Blocked {
                // Check if blocking dependencies are resolved
                if let Some(prerequisites) = graph.reverse_edges.get(task_id) {
                    let unresolved: Vec<_> = prerequisites
                        .iter()
                        .filter(|&prereq_id| {
                            graph
                                .nodes
                                .get(prereq_id)
                                .map(|n| n.status != TaskNodeStatus::Completed)
                                .unwrap_or(true)
                        })
                        .collect();

                    if unresolved.is_empty() {
                        // All dependencies resolved, task can be unblocked
                        decisions.push(ProactiveDecision {
                            decision_type: DecisionType::GenerateTask,
                            reasoning: format!(
                                "Task {} can be unblocked - all dependencies completed",
                                task_id
                            ),
                            confidence: 0.95,
                            suggested_actions: vec![SuggestedAction {
                                action_type: "unblock_task".to_string(),
                                description: "Move task to ready queue".to_string(),
                                parameters: HashMap::from([(
                                    "task_id".to_string(),
                                    task_id.clone(),
                                )]),
                                expected_impact: "Enable task execution".to_string(),
                            }],
                            risk_assessment: RiskLevel::Low,
                        });
                    }
                }
            }
        }

        Ok(decisions)
    }

    /// Predict next tasks based on patterns
    async fn predict_next_tasks(
        &self,
        agents: &DashMap<String, ClaudeCodeAgent>,
    ) -> Result<Vec<ProactiveDecision>> {
        let mut decisions = Vec::new();
        let predictor = self.task_predictor.read().await;

        // Analyze recent completions
        for entry in agents.iter() {
            let agent = entry.value();

            // Check last completed task
            if let Some((last_task, last_result)) = agent.task_history.last() {
                if last_result.success {
                    // Find matching completion patterns
                    for pattern in &predictor.completion_patterns {
                        if pattern.completed_task_type == last_task.task_type
                            && pattern.probability > 0.7
                        {
                            for task_template in &pattern.follow_up_tasks {
                                decisions.push(ProactiveDecision {
                                    decision_type: DecisionType::GenerateTask,
                                    reasoning: format!(
                                        "Pattern match: {:?} completion typically requires {}",
                                        last_task.task_type, task_template.description_template
                                    ),
                                    confidence: pattern.probability,
                                    suggested_actions: vec![SuggestedAction {
                                        action_type: "create_task".to_string(),
                                        description: format!(
                                            "Create follow-up task: {}",
                                            task_template.description_template
                                        ),
                                        parameters: HashMap::from([
                                            (
                                                "template".to_string(),
                                                serde_json::to_string(task_template)?,
                                            ),
                                            ("parent_task".to_string(), last_task.id.clone()),
                                        ]),
                                        expected_impact: "Maintain development momentum"
                                            .to_string(),
                                    }],
                                    risk_assessment: RiskLevel::Low,
                                });
                            }
                        }
                    }

                    // Check for pattern triggers in task description
                    let description_lower = last_task.description.to_lowercase();
                    for (pattern_id, pattern) in &predictor.pattern_library {
                        for trigger in &pattern.trigger_conditions {
                            if description_lower.contains(&trigger.to_lowercase()) {
                                for task_template in &pattern.generated_tasks {
                                    decisions.push(ProactiveDecision {
                                        decision_type: DecisionType::GenerateTask,
                                        reasoning: format!(
                                            "Pattern '{}' triggered by: {}",
                                            pattern_id, trigger
                                        ),
                                        confidence: pattern.confidence,
                                        suggested_actions: vec![
                                            SuggestedAction {
                                                action_type: "create_task".to_string(),
                                                description: format!(
                                                    "Auto-generate: {}",
                                                    task_template.description_template
                                                ),
                                                parameters: HashMap::from([
                                                    ("template".to_string(), serde_json::to_string(task_template)?),
                                                    ("trigger_task".to_string(), last_task.id.clone()),
                                                ]),
                                                expected_impact: "Ensure complete feature implementation".to_string(),
                                            },
                                        ],
                                        risk_assessment: RiskLevel::Low,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(decisions)
    }

    /// Monitor goal progress
    async fn monitor_goals(&self) -> Result<Vec<ProactiveDecision>> {
        let mut decisions = Vec::new();
        let goals = self.goal_tracker.read().await;

        // Check objective progress
        for objective in &goals.objectives {
            if objective.progress < 0.5 {
                if let Some(deadline) = objective.deadline {
                    let time_remaining = deadline - Utc::now();
                    let days_remaining = time_remaining.num_days();

                    if days_remaining <= 7 && objective.progress < 0.8 {
                        decisions.push(ProactiveDecision {
                            decision_type: DecisionType::ChangeStrategy,
                            reasoning: format!(
                                "Objective '{}' is behind schedule: {}% complete with {} days remaining",
                                objective.title, (objective.progress * 100.0) as u32, days_remaining
                            ),
                            confidence: 0.85,
                            suggested_actions: vec![
                                SuggestedAction {
                                    action_type: "reprioritize_tasks".to_string(),
                                    description: "Focus resources on critical objective".to_string(),
                                    parameters: HashMap::from([
                                        ("objective_id".to_string(), objective.id.clone()),
                                    ]),
                                    expected_impact: "Improve deadline adherence".to_string(),
                                },
                            ],
                            risk_assessment: RiskLevel::Medium,
                        });
                    }
                }
            }
        }

        // Check sprint velocity
        if let Some(sprint) = &goals.current_sprint {
            let sprint_progress = (Utc::now() - sprint.start_date).num_days() as f64
                / (sprint.end_date - sprint.start_date).num_days() as f64;

            let velocity_progress = sprint.current_velocity as f64 / sprint.velocity_target as f64;

            if sprint_progress > 0.5 && velocity_progress < 0.3 {
                decisions.push(ProactiveDecision {
                    decision_type: DecisionType::ScaleTeam,
                    reasoning: format!(
                        "Sprint velocity is significantly behind: {}% of target with {}% time remaining",
                        (velocity_progress * 100.0) as u32,
                        ((1.0 - sprint_progress) * 100.0) as u32
                    ),
                    confidence: 0.7,
                    suggested_actions: vec![
                        SuggestedAction {
                            action_type: "add_agent".to_string(),
                            description: "Consider adding additional agents".to_string(),
                            parameters: HashMap::new(),
                            expected_impact: "Increase development velocity".to_string(),
                        },
                    ],
                    risk_assessment: RiskLevel::Medium,
                });
            }
        }

        Ok(decisions)
    }

    /// Execute a proactive decision
    async fn execute_decision(
        &self,
        decision: &ProactiveDecision,
        coordination_bus: &CoordinationBus,
    ) -> Result<()> {
        info!("Executing proactive decision: {:?}", decision.decision_type);

        for action in &decision.suggested_actions {
            match action.action_type.as_str() {
                "create_task" => {
                    if let Some(template_json) = action.parameters.get("template") {
                        let template: TaskTemplate = serde_json::from_str(template_json)?;
                        let task = self.create_task_from_template(&template).await?;

                        // Send task creation message
                        coordination_bus
                            .send_message(AgentMessage::TaskGenerated {
                                task_id: task.id.clone(),
                                description: task.description.clone(),
                                reasoning: decision.reasoning.clone(),
                            })
                            .await?;

                        info!("Auto-generated task: {}", task.description);
                    }
                }
                "unblock_task" => {
                    if let Some(task_id) = action.parameters.get("task_id") {
                        self.unblock_task(task_id).await?;
                    }
                }
                "request_search" => {
                    if let Some(query) = action.parameters.get("query") {
                        if let Some(context) = action.parameters.get("context") {
                            self.request_search(query, context, coordination_bus)
                                .await?;
                        }
                    }
                }
                _ => {
                    debug!("Skipping execution of action: {}", action.action_type);
                }
            }
        }

        Ok(())
    }

    /// Create a task from a template
    async fn create_task_from_template(&self, template: &TaskTemplate) -> Result<Task> {
        let task_id = format!("auto-{}", Uuid::new_v4());
        let description = template.description_template.clone();

        Ok(
            Task::new(task_id, description, template.priority, template.task_type)
                .with_duration((template.estimated_duration * 60) as u32),
        ) // convert minutes to seconds
    }

    /// Unblock a task in the dependency graph
    async fn unblock_task(&self, task_id: &str) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;

        if let Some(node) = graph.nodes.get_mut(task_id) {
            node.status = TaskNodeStatus::NotStarted;
            info!("Unblocked task: {}", task_id);
        }

        Ok(())
    }

    /// Update project context based on completed work
    pub async fn update_context_from_completion(
        &self,
        task: &Task,
        result: &TaskResult,
    ) -> Result<()> {
        if result.success {
            let mut context = self.project_context.write().await;

            // Analyze task to understand what was built
            if task.description.to_lowercase().contains("component") {
                let component_name = self.extract_component_name(&task.description);
                if !context.features.contains(&component_name) {
                    context.features.push(component_name);
                }
            }

            // Update dependency graph
            let mut graph = self.dependency_graph.write().await;
            if let Some(node) = graph.nodes.get_mut(&task.id) {
                node.status = TaskNodeStatus::Completed;
                node.actual_duration = Some(result.duration.as_secs() / 60); // convert to minutes
            }
        }

        Ok(())
    }

    /// Extract component name from task description
    fn extract_component_name(&self, description: &str) -> String {
        // Simple heuristic to extract component names
        let words: Vec<&str> = description.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase() == "component" && i > 0 {
                return words[i - 1].to_string();
            }
        }

        "Unknown Component".to_string()
    }

    /// Add a milestone to track
    pub async fn add_milestone(&self, milestone: Milestone) -> Result<()> {
        let mut context = self.project_context.write().await;
        context.milestones.push(milestone);
        Ok(())
    }

    /// Set project goal
    pub async fn set_objective(&self, objective: Objective) -> Result<()> {
        let mut goals = self.goal_tracker.write().await;
        goals.objectives.push(objective);
        Ok(())
    }

    /// Analyze if search is needed for current tasks
    async fn analyze_search_needs(
        &self,
        agents: &DashMap<String, ClaudeCodeAgent>,
    ) -> Result<Vec<ProactiveDecision>> {
        let mut decisions = Vec::new();

        // Patterns that indicate search might be helpful
        let search_indicators = vec![
            ("research", "Researching information about"),
            ("find information", "Finding information about"),
            ("look up", "Looking up"),
            ("best practices", "Discovering best practices for"),
            ("documentation", "Finding documentation for"),
            ("examples", "Finding examples of"),
            ("how to", "Understanding how to"),
            ("comparison", "Comparing technologies"),
            ("alternatives", "Finding alternatives to"),
            ("error", "Investigating error"),
            ("unknown", "Clarifying unknown concept"),
            ("investigate", "Investigating"),
        ];

        // Check recent agent activities and errors
        for entry in agents.iter() {
            let agent = entry.value();

            // Check if agent is stuck and might need information
            if matches!(agent.status, AgentStatus::Working) {
                let time_since_activity = Utc::now() - agent.last_activity;

                // If stuck for more than 10 minutes, suggest search
                if time_since_activity.num_minutes() > 10 {
                    // Look at current task context
                    if let Some((current_task, _)) = agent.task_history.last() {
                        let task_desc_lower = current_task.description.to_lowercase();

                        // Check if task involves research or information gathering
                        for (indicator, search_prefix) in &search_indicators {
                            if task_desc_lower.contains(indicator) {
                                decisions.push(ProactiveDecision {
                                    decision_type: DecisionType::RequestSearch,
                                    reasoning: format!(
                                        "Agent {} appears stuck on task requiring information: '{}'",
                                        agent.identity.agent_id, current_task.description
                                    ),
                                    confidence: 0.85,
                                    suggested_actions: vec![SuggestedAction {
                                        action_type: "request_search".to_string(),
                                        description: format!("{} {}", search_prefix, current_task.description),
                                        parameters: HashMap::from([
                                            ("query".to_string(), current_task.description.clone()),
                                            ("context".to_string(), format!("Agent {} stuck on task", agent.identity.agent_id)),
                                            ("requesting_agent".to_string(), agent.identity.agent_id.clone()),
                                        ]),
                                        expected_impact: "Provide information to unblock agent".to_string(),
                                    }],
                                    risk_assessment: RiskLevel::Low,
                                });
                                break;
                            }
                        }
                    }
                }
            }

            // Check recent failed tasks for missing information
            let recent_failures: Vec<_> = agent
                .task_history
                .iter()
                .rev()
                .take(3)
                .filter(|(_, result)| !result.success)
                .collect();

            for (failed_task, result) in recent_failures {
                if let Some(error) = &result.error {
                    let error_lower = error.to_lowercase();

                    // Common error patterns that might benefit from search
                    if error_lower.contains("not found")
                        || error_lower.contains("unknown")
                        || error_lower.contains("missing documentation")
                        || error_lower.contains("unclear")
                        || error_lower.contains("deprecat")
                        || error_lower.contains("no examples")
                    {
                        decisions.push(ProactiveDecision {
                            decision_type: DecisionType::RequestSearch,
                            reasoning: format!(
                                "Task {} failed with error suggesting missing information: {}",
                                failed_task.id, error
                            ),
                            confidence: 0.9,
                            suggested_actions: vec![SuggestedAction {
                                action_type: "request_search".to_string(),
                                description: format!("Search for solution to: {}", error),
                                parameters: HashMap::from([
                                    (
                                        "query".to_string(),
                                        format!("{} {}", failed_task.description, error),
                                    ),
                                    (
                                        "context".to_string(),
                                        format!("Error resolution for task {}", failed_task.id),
                                    ),
                                    (
                                        "requesting_agent".to_string(),
                                        agent.identity.agent_id.clone(),
                                    ),
                                ]),
                                expected_impact: "Find solution to resolve error".to_string(),
                            }],
                            risk_assessment: RiskLevel::Low,
                        });
                    }
                }
            }
        }

        // Check project context for technology research needs
        let context = self.project_context.read().await;

        // If in planning or setup phase, suggest research for tech stack
        if matches!(
            context.current_phase,
            DevelopmentPhase::Planning | DevelopmentPhase::Setup
        ) {
            for tech in &context.tech_stack {
                decisions.push(ProactiveDecision {
                    decision_type: DecisionType::RequestSearch,
                    reasoning: format!(
                        "Project in {} phase - gathering best practices for {}",
                        match context.current_phase {
                            DevelopmentPhase::Planning => "planning",
                            DevelopmentPhase::Setup => "setup",
                            _ => "early",
                        },
                        tech
                    ),
                    confidence: 0.75,
                    suggested_actions: vec![SuggestedAction {
                        action_type: "request_search".to_string(),
                        description: format!("{} best practices and setup guide", tech),
                        parameters: HashMap::from([
                            (
                                "query".to_string(),
                                format!("{} best practices setup guide tutorial", tech),
                            ),
                            (
                                "context".to_string(),
                                "Project setup and architecture planning".to_string(),
                            ),
                            ("requesting_agent".to_string(), "master-claude".to_string()),
                        ]),
                        expected_impact: "Ensure proper setup and architecture".to_string(),
                    }],
                    risk_assessment: RiskLevel::Low,
                });
            }
        }

        Ok(decisions)
    }

    /// Request a search from the search agent
    async fn request_search(
        &self,
        query: &str,
        context: &str,
        coordination_bus: &CoordinationBus,
    ) -> Result<()> {
        info!("Requesting search for: {}", query);

        let search_request = SearchRequest {
            requesting_agent: "master-claude".to_string(),
            query: query.to_string(),
            max_results: Some(10),
            filters: None,
            context: Some(context.to_string()),
        };

        let message = AgentMessage::Coordination {
            from_agent: "master-claude".to_string(),
            to_agent: "search".to_string(),
            message_type: CoordinationType::Custom("search_request".to_string()),
            payload: serde_json::to_value(search_request)?,
        };

        coordination_bus.send_message(message).await?;

        Ok(())
    }

    /// Handle search response from search agent
    pub async fn handle_search_response(
        &self,
        response: SearchResponse,
        coordination_bus: &CoordinationBus,
    ) -> Result<()> {
        info!(
            "Received search response with {} results",
            response.results.len()
        );

        // Analyze search results and create appropriate tasks or insights
        if !response.results.is_empty() {
            let mut insights = Vec::new();

            for (i, result) in response.results.iter().take(5).enumerate() {
                insights.push(format!(
                    "{}. {} - {} (relevance: {:?})",
                    i + 1,
                    result.title,
                    result.snippet,
                    result.relevance_score
                ));
            }

            // Create a task to review and apply the findings
            let review_task = Task::new(
                format!("review-search-{}", Uuid::new_v4()),
                format!("Review search findings for: {}", response.query_used),
                Priority::Medium,
                TaskType::Research,
            )
            .with_details(format!(
                "Search query: {}\nTop findings:\n{}\n\nPlease review these findings and apply relevant insights to the current work.",
                response.query_used,
                insights.join("\n")
            ))
            .with_duration(1200); // 20 minutes

            // Send task generation message
            coordination_bus
                .send_message(AgentMessage::TaskGenerated {
                    task_id: review_task.id.clone(),
                    description: review_task.description.clone(),
                    reasoning: format!(
                        "Search completed with {} relevant results",
                        response.results.len()
                    ),
                })
                .await?;
        }

        Ok(())
    }
}

// Extend AgentMessage to include task generation
impl AgentMessage {
    pub fn task_generated(task_id: String, description: String, reasoning: String) -> Self {
        AgentMessage::TaskGenerated {
            task_id,
            description,
            reasoning,
        }
    }
}

/// Custom agent message for task generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProactiveAgentMessage {
    TaskGenerated {
        task_id: String,
        description: String,
        reasoning: String,
    },
}
