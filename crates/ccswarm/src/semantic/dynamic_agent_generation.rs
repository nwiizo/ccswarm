//! Dynamic subagent generation with semantic support
//!
//! Automatically generates specialized subagents based on project needs

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use super::analyzer::{SemanticAnalyzer, Symbol, SymbolKind};
use super::memory::{Memory, MemoryType, ProjectMemory};
use super::subagent_integration::AgentRole;
use super::symbol_index::SymbolIndex;
use super::task_analyzer::{Task, TaskContext};
use super::{SemanticError, SemanticResult};

/// Agent template for dynamic generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub name: String,
    pub role: AgentRole,
    pub description: String,
    pub tools: AgentTools,
    pub capabilities: Vec<String>,
    pub expertise_areas: Vec<String>,
    pub coordination_rules: Vec<String>,
}

/// Tools configuration for agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTools {
    pub standard: Vec<String>,
    pub semantic: Vec<String>,
    pub memory: Vec<String>,
    pub custom: Vec<String>,
}

/// Agent generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenerationRequest {
    pub task_context: TaskContext,
    pub project_characteristics: ProjectCharacteristics,
    pub existing_agents: Vec<String>,
    pub requirements: Vec<String>,
}

/// Project complexity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProjectComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Project characteristics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCharacteristics {
    pub primary_language: String,
    pub frameworks: Vec<String>,
    pub architecture_style: String,
    pub complexity: ProjectComplexityLevel,
    pub domain: String,
    pub team_size: usize,
}

/// Generated agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAgent {
    pub template: AgentTemplate,
    pub configuration: AgentConfiguration,
    pub markdown_definition: String,
    pub justification: String,
}

/// Agent configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfiguration {
    pub auto_accept_enabled: bool,
    pub risk_threshold: u8,
    pub max_concurrent_tasks: usize,
    pub coordination_mode: CoordinationMode,
    pub memory_quota: usize,
}

/// Coordination mode for agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoordinationMode {
    Independent,
    Collaborative,
    Supervised,
    LeaderFollower,
}

/// Dynamic agent generator
pub struct DynamicAgentGenerator {
    analyzer: Arc<SemanticAnalyzer>,
    index: Arc<SymbolIndex>,
    memory: Arc<ProjectMemory>,
    templates_dir: PathBuf,
    agents_dir: PathBuf,
}

impl DynamicAgentGenerator {
    /// Create a new dynamic agent generator
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        index: Arc<SymbolIndex>,
        memory: Arc<ProjectMemory>,
    ) -> Self {
        Self {
            analyzer,
            index,
            memory,
            templates_dir: PathBuf::from(".claude/agent_templates"),
            agents_dir: PathBuf::from(".claude/agents"),
        }
    }

    /// Analyze project and determine needed agents
    pub async fn analyze_agent_needs(&self) -> SemanticResult<Vec<AgentGenerationRequest>> {
        let mut requests = Vec::new();

        // Analyze codebase structure
        let characteristics = self.analyze_project_characteristics().await?;

        // Get all symbols to understand project scope
        let all_symbols = self.index.get_all_symbols().await?;

        // Analyze symbol distribution
        let distribution = self.analyze_symbol_distribution(&all_symbols);

        // Determine needed specializations
        if distribution.frontend_symbols > 50 && !self.has_agent("frontend-specialist").await {
            requests.push(self.create_frontend_request(&characteristics).await?);
        }

        if distribution.backend_symbols > 50 && !self.has_agent("backend-specialist").await {
            requests.push(self.create_backend_request(&characteristics).await?);
        }

        if distribution.test_symbols < distribution.total_symbols / 10 {
            requests.push(self.create_qa_request(&characteristics).await?);
        }

        if characteristics.complexity == ProjectComplexityLevel::VeryComplex {
            requests.push(self.create_architect_request(&characteristics).await?);
        }

        // Check for domain-specific needs
        if characteristics.domain.contains("ml") || characteristics.domain.contains("ai") {
            requests.push(self.create_ml_specialist_request(&characteristics).await?);
        }

        if characteristics
            .frameworks
            .iter()
            .any(|f| f.contains("blockchain"))
        {
            requests.push(
                self.create_blockchain_specialist_request(&characteristics)
                    .await?,
            );
        }

        Ok(requests)
    }

    /// Generate a new agent based on request
    pub async fn generate_agent(
        &self,
        request: &AgentGenerationRequest,
    ) -> SemanticResult<GeneratedAgent> {
        // Determine agent role and specialization
        let role = self.determine_role(&request.task_context, &request.project_characteristics);

        // Generate capabilities based on project needs
        let capabilities = self.generate_capabilities(&request.task_context, &request.requirements);

        // Create agent template
        let template = AgentTemplate {
            name: self.generate_agent_name(&role, &request.project_characteristics),
            role: role.clone(),
            description: self.generate_description(&role, &capabilities),
            tools: self.select_tools(&role, &request.requirements),
            capabilities: capabilities.clone(),
            expertise_areas: self.determine_expertise_areas(&request.project_characteristics),
            coordination_rules: self.generate_coordination_rules(&request.existing_agents),
        };

        // Generate configuration
        let configuration = self.generate_configuration(&role, &request.project_characteristics);

        // Generate markdown definition
        let markdown = self
            .generate_markdown_definition(&template, &configuration)
            .await?;

        // Generate justification
        let justification = self.generate_justification(&template, request);

        Ok(GeneratedAgent {
            template,
            configuration,
            markdown_definition: markdown,
            justification,
        })
    }

    /// Deploy a generated agent
    pub async fn deploy_agent(&self, agent: &GeneratedAgent) -> SemanticResult<()> {
        // Save agent definition
        let agent_file = self.agents_dir.join(format!("{}.md", agent.template.name));
        fs::write(&agent_file, &agent.markdown_definition)
            .await
            .map_err(|e| {
                SemanticError::Other(format!("Failed to write agent definition: {}", e))
            })?;

        // Record in project memory
        let memory = Memory {
            id: format!("agent_{}", Utc::now().timestamp()),
            name: format!("Agent: {}", agent.template.name),
            content: serde_json::to_string(&agent.template)?,
            memory_type: MemoryType::Other("AgentDefinition".to_string()),
            related_symbols: Vec::new(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("role".to_string(), format!("{:?}", agent.template.role));
                meta.insert("deployed_at".to_string(), Utc::now().to_rfc3339());
                meta
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.memory.store_memory(memory).await?;

        log::info!("Deployed agent: {}", agent.template.name);
        Ok(())
    }

    /// Analyze project characteristics
    async fn analyze_project_characteristics(&self) -> SemanticResult<ProjectCharacteristics> {
        let all_symbols = self.index.get_all_symbols().await?;

        // Detect primary language (simplified - assumes Rust for now)
        let primary_language = "Rust".to_string();

        // Detect frameworks
        let mut frameworks = Vec::new();
        for symbol in &all_symbols {
            if symbol.path.contains("tokio") {
                frameworks.push("Tokio".to_string());
            }
            if symbol.path.contains("actix") {
                frameworks.push("Actix".to_string());
            }
            if symbol.path.contains("rocket") {
                frameworks.push("Rocket".to_string());
            }
        }
        frameworks.sort();
        frameworks.dedup();

        // Determine architecture style
        let architecture_style = if all_symbols.len() > 1000 {
            "Microservices".to_string()
        } else if all_symbols.len() > 500 {
            "Modular Monolith".to_string()
        } else {
            "Monolithic".to_string()
        };

        // Calculate complexity
        let complexity = if all_symbols.len() > 2000 {
            ProjectComplexityLevel::VeryComplex
        } else if all_symbols.len() > 1000 {
            ProjectComplexityLevel::Complex
        } else if all_symbols.len() > 500 {
            ProjectComplexityLevel::Moderate
        } else {
            ProjectComplexityLevel::Simple
        };

        // Detect domain
        let domain = self.detect_domain(&all_symbols);

        Ok(ProjectCharacteristics {
            primary_language,
            frameworks,
            architecture_style,
            complexity,
            domain,
            team_size: 5, // Default assumption
        })
    }

    /// Detect project domain from symbols
    fn detect_domain(&self, symbols: &[Symbol]) -> String {
        let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.to_lowercase()).collect();

        if symbol_names
            .iter()
            .any(|n| n.contains("neural") || n.contains("tensor") || n.contains("model"))
        {
            "Machine Learning".to_string()
        } else if symbol_names
            .iter()
            .any(|n| n.contains("order") || n.contains("payment") || n.contains("cart"))
        {
            "E-commerce".to_string()
        } else if symbol_names
            .iter()
            .any(|n| n.contains("user") || n.contains("auth") || n.contains("session"))
        {
            "Web Application".to_string()
        } else if symbol_names
            .iter()
            .any(|n| n.contains("block") || n.contains("chain") || n.contains("ledger"))
        {
            "Blockchain".to_string()
        } else {
            "General Purpose".to_string()
        }
    }

    /// Analyze symbol distribution
    fn analyze_symbol_distribution(&self, symbols: &[Symbol]) -> SymbolDistribution {
        let mut distribution = SymbolDistribution::default();

        for symbol in symbols {
            distribution.total_symbols += 1;

            // Categorize symbols
            if symbol.file_path.contains("frontend") || symbol.file_path.contains("ui") {
                distribution.frontend_symbols += 1;
            } else if symbol.file_path.contains("backend") || symbol.file_path.contains("api") {
                distribution.backend_symbols += 1;
            } else if symbol.file_path.contains("test") {
                distribution.test_symbols += 1;
            }

            match symbol.kind {
                SymbolKind::Function | SymbolKind::Method => distribution.function_symbols += 1,
                SymbolKind::Struct | SymbolKind::Class => distribution.type_symbols += 1,
                _ => distribution.other_symbols += 1,
            }
        }

        distribution
    }

    /// Check if an agent exists
    async fn has_agent(&self, name: &str) -> bool {
        let agent_file = self.agents_dir.join(format!("{}.md", name));
        agent_file.exists()
    }

    /// Determine role for the agent
    fn determine_role(
        &self,
        task_context: &TaskContext,
        characteristics: &ProjectCharacteristics,
    ) -> AgentRole {
        // Analyze task to determine best role
        if task_context
            .task
            .tags
            .iter()
            .any(|t| t.contains("frontend"))
        {
            AgentRole::Frontend
        } else if task_context.task.tags.iter().any(|t| t.contains("backend")) {
            AgentRole::Backend
        } else if task_context.task.tags.iter().any(|t| t.contains("test")) {
            AgentRole::QA
        } else if task_context
            .task
            .tags
            .iter()
            .any(|t| t.contains("security"))
        {
            AgentRole::Security
        } else if characteristics.complexity == ProjectComplexityLevel::VeryComplex {
            AgentRole::Custom("Architect".to_string())
        } else {
            AgentRole::Custom("Generalist".to_string())
        }
    }

    /// Generate capabilities for the agent
    fn generate_capabilities(
        &self,
        task_context: &TaskContext,
        requirements: &[String],
    ) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Add task-specific capabilities
        for step in &task_context.recommended_approach.steps {
            capabilities.push(format!("{:?} operations", step.action_type));
        }

        // Add required capabilities
        capabilities.extend(
            task_context
                .recommended_approach
                .required_capabilities
                .clone(),
        );

        // Add custom requirements
        capabilities.extend(requirements.iter().cloned());

        // Deduplicate
        capabilities.sort();
        capabilities.dedup();

        capabilities
    }

    /// Generate agent name
    fn generate_agent_name(
        &self,
        role: &AgentRole,
        characteristics: &ProjectCharacteristics,
    ) -> String {
        match role {
            AgentRole::Frontend => "frontend-specialist".to_string(),
            AgentRole::Backend => "backend-specialist".to_string(),
            AgentRole::DevOps => "devops-specialist".to_string(),
            AgentRole::QA => "qa-specialist".to_string(),
            AgentRole::Security => "security-specialist".to_string(),
            AgentRole::Search => "search-specialist".to_string(),
            AgentRole::Refactoring => "refactoring-specialist".to_string(),
            AgentRole::Custom(name) => format!("{}-specialist", name.to_lowercase()),
        }
    }

    /// Generate agent description
    fn generate_description(&self, role: &AgentRole, capabilities: &[String]) -> String {
        format!(
            "{:?} specialist with expertise in: {}. Enhanced with semantic code understanding.",
            role,
            capabilities.join(", ")
        )
    }

    /// Select tools for the agent
    fn select_tools(&self, role: &AgentRole, requirements: &[String]) -> AgentTools {
        let mut tools = AgentTools {
            standard: vec![
                "write_file".to_string(),
                "read_file".to_string(),
                "execute_command".to_string(),
            ],
            semantic: vec![
                "find_symbol".to_string(),
                "replace_symbol_body".to_string(),
                "find_referencing_symbols".to_string(),
                "search_for_pattern".to_string(),
            ],
            memory: vec![
                "read_memory".to_string(),
                "write_memory".to_string(),
                "list_memories".to_string(),
            ],
            custom: Vec::new(),
        };

        // Add role-specific tools
        match role {
            AgentRole::Frontend => {
                tools.standard.push("browser_action".to_string());
                tools.custom.push("component_generator".to_string());
            }
            AgentRole::Backend => {
                tools.standard.push("database_query".to_string());
                tools.custom.push("api_generator".to_string());
            }
            AgentRole::DevOps => {
                tools.standard.push("docker_command".to_string());
                tools.custom.push("deployment_manager".to_string());
            }
            AgentRole::QA => {
                tools.standard.push("test_runner".to_string());
                tools.custom.push("coverage_analyzer".to_string());
            }
            _ => {}
        }

        // Add requirement-specific tools
        for req in requirements {
            if req.contains("database") {
                tools.standard.push("database_query".to_string());
            }
            if req.contains("api") {
                tools.custom.push("api_tester".to_string());
            }
        }

        tools
    }

    /// Determine expertise areas
    fn determine_expertise_areas(&self, characteristics: &ProjectCharacteristics) -> Vec<String> {
        let mut areas = vec![characteristics.primary_language.clone()];
        areas.extend(characteristics.frameworks.clone());
        areas.push(characteristics.architecture_style.clone());
        areas.push(characteristics.domain.clone());
        areas
    }

    /// Generate coordination rules
    fn generate_coordination_rules(&self, existing_agents: &[String]) -> Vec<String> {
        let mut rules = vec![
            "Notify relevant agents of API changes".to_string(),
            "Share knowledge through project memory".to_string(),
            "Coordinate through semantic analysis".to_string(),
        ];

        for agent in existing_agents {
            if agent.contains("frontend") {
                rules.push("Coordinate UI changes with frontend-specialist".to_string());
            }
            if agent.contains("backend") {
                rules.push("Sync API contracts with backend-specialist".to_string());
            }
        }

        rules
    }

    /// Generate agent configuration
    fn generate_configuration(
        &self,
        role: &AgentRole,
        characteristics: &ProjectCharacteristics,
    ) -> AgentConfiguration {
        let (auto_accept, risk_threshold) = match role {
            AgentRole::QA => (false, 3),       // QA should be careful
            AgentRole::Security => (false, 2), // Security even more careful
            _ => (true, 5),                    // Others can be more autonomous
        };

        let max_concurrent_tasks = match characteristics.complexity {
            ProjectComplexityLevel::VeryComplex => 1,
            ProjectComplexityLevel::Complex => 2,
            _ => 3,
        };

        let coordination_mode = if characteristics.team_size > 10 {
            CoordinationMode::Supervised
        } else if characteristics.team_size > 5 {
            CoordinationMode::Collaborative
        } else {
            CoordinationMode::Independent
        };

        AgentConfiguration {
            auto_accept_enabled: auto_accept,
            risk_threshold,
            max_concurrent_tasks,
            coordination_mode,
            memory_quota: 100,
        }
    }

    /// Generate markdown definition for the agent
    async fn generate_markdown_definition(
        &self,
        template: &AgentTemplate,
        configuration: &AgentConfiguration,
    ) -> SemanticResult<String> {
        let tools_standard = template.tools.standard.join(",");
        let tools_semantic = template.tools.semantic.join(",");
        let tools_memory = template.tools.memory.join(",");
        let tools_custom = template.tools.custom.join(",");

        let markdown = format!(
            r#"---
name: {}
description: |
  {}
  MUST BE USED PROACTIVELY for relevant tasks.
tools: 
  - standard: {}
  - semantic: {}
  - memory: {}
  - custom: {}
capabilities: {}
expertise_areas: {}
coordination_mode: {:?}
auto_accept: {}
risk_threshold: {}
---

# {} with Semantic Intelligence

You are a specialized agent with deep understanding of code semantics and project architecture.

## Core Responsibilities

{}

## Expertise Areas

{}

## Semantic Analysis Guidelines

### 1. Code Exploration Strategy
NEVER read entire files. Instead:
1. Use `get_symbols_overview` to understand structure
2. Use `find_symbol` to locate specific elements
3. Use `find_referencing_symbols` to trace usage
4. Only read symbol bodies when necessary

### 2. Implementation Workflow
1. **Analyze existing patterns**
2. **Implement with context**
3. **Validate changes**
4. **Share knowledge**

## Coordination Rules

{}

## Configuration
- Auto-accept: {}
- Risk threshold: {}
- Max concurrent tasks: {}
- Coordination mode: {:?}

## Best Practices
1. Always use semantic tools for code understanding
2. Share discoveries through project memory
3. Coordinate with other agents
4. Maintain consistency with project patterns
5. Document important decisions
"#,
            template.name,
            template.description,
            tools_standard,
            tools_semantic,
            tools_memory,
            tools_custom,
            template.capabilities.join(", "),
            template.expertise_areas.join(", "),
            configuration.coordination_mode,
            configuration.auto_accept_enabled,
            configuration.risk_threshold,
            template.name,
            template.description,
            template
                .expertise_areas
                .iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n"),
            template
                .coordination_rules
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
            configuration.auto_accept_enabled,
            configuration.risk_threshold,
            configuration.max_concurrent_tasks,
            configuration.coordination_mode
        );

        Ok(markdown)
    }

    /// Generate justification for the agent
    fn generate_justification(
        &self,
        template: &AgentTemplate,
        request: &AgentGenerationRequest,
    ) -> String {
        format!(
            "Generated {} to handle {} complexity tasks in {}. \
            Project has {} symbols requiring specialized expertise in {}. \
            Agent will coordinate with {} existing agents using {:?} mode.",
            template.name,
            format!("{:?}", request.project_characteristics.complexity).to_lowercase(),
            request.project_characteristics.domain,
            request.task_context.related_symbols.len(),
            template.expertise_areas.join(", "),
            request.existing_agents.len(),
            template.role
        )
    }

    // Helper methods for creating specific agent requests
    async fn create_frontend_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Frontend Development"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "React component development".to_string(),
                "TypeScript type safety".to_string(),
                "UI/UX optimization".to_string(),
            ],
        })
    }

    async fn create_backend_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Backend Development"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "API design and implementation".to_string(),
                "Database optimization".to_string(),
                "Business logic implementation".to_string(),
            ],
        })
    }

    async fn create_qa_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Quality Assurance"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "Test coverage improvement".to_string(),
                "Test generation".to_string(),
                "Quality metrics tracking".to_string(),
            ],
        })
    }

    async fn create_architect_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Architecture Design"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "System design".to_string(),
                "Pattern enforcement".to_string(),
                "Technical debt management".to_string(),
            ],
        })
    }

    async fn create_ml_specialist_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Machine Learning"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "Model development".to_string(),
                "Data pipeline optimization".to_string(),
                "Performance tuning".to_string(),
            ],
        })
    }

    async fn create_blockchain_specialist_request(
        &self,
        characteristics: &ProjectCharacteristics,
    ) -> SemanticResult<AgentGenerationRequest> {
        Ok(AgentGenerationRequest {
            task_context: self.create_mock_task_context("Blockchain Development"),
            project_characteristics: characteristics.clone(),
            existing_agents: self.get_existing_agents().await?,
            requirements: vec![
                "Smart contract development".to_string(),
                "Consensus mechanism".to_string(),
                "Security auditing".to_string(),
            ],
        })
    }

    fn create_mock_task_context(&self, title: &str) -> TaskContext {
        TaskContext {
            task: Task {
                id: format!("mock_{}", Utc::now().timestamp()),
                title: title.to_string(),
                description: format!("Handle {} tasks", title.to_lowercase()),
                priority: super::task_analyzer::TaskPriority::High,
                tags: vec![title.to_lowercase().replace(" ", "_")],
                assigned_agent: None,
            },
            related_symbols: Vec::new(),
            impact: super::analyzer::ImpactAnalysis {
                change: super::analyzer::SymbolChange {
                    symbol: Symbol {
                        name: "mock".to_string(),
                        path: "mock".to_string(),
                        kind: SymbolKind::Other("mock".to_string()),
                        file_path: String::new(),
                        line: 0,
                        body: None,
                        references: Vec::new(),
                        metadata: HashMap::new(),
                    },
                    change_type: super::analyzer::ChangeType::Added,
                    old_value: None,
                    new_value: None,
                },
                affected_symbols: Vec::new(),
                severity: super::analyzer::ImpactSeverity::Low,
                suggested_actions: Vec::new(),
            },
            recommended_approach: super::task_analyzer::TaskApproach {
                steps: Vec::new(),
                complexity: super::task_analyzer::ComplexityLevel::Simple,
                recommended_agent: title.to_lowercase().replace(" ", "-"),
                required_capabilities: Vec::new(),
                risks: Vec::new(),
            },
        }
    }

    async fn get_existing_agents(&self) -> SemanticResult<Vec<String>> {
        let mut agents = Vec::new();

        if self.agents_dir.exists() {
            let mut entries = fs::read_dir(&self.agents_dir).await.map_err(|e| {
                SemanticError::Other(format!("Failed to read agents directory: {}", e))
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                SemanticError::Other(format!("Failed to read directory entry: {}", e))
            })? {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".md") {
                        agents.push(name.trim_end_matches(".md").to_string());
                    }
                }
            }
        }

        Ok(agents)
    }
}

/// Symbol distribution analysis
#[derive(Debug, Default)]
struct SymbolDistribution {
    total_symbols: usize,
    frontend_symbols: usize,
    backend_symbols: usize,
    test_symbols: usize,
    function_symbols: usize,
    type_symbols: usize,
    other_symbols: usize,
}
