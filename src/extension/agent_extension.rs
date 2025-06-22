//! Agent self-extension with advanced search and information gathering capabilities

use super::*;
// Remove AIProvider import since we'll define our own trait
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

/// Trait for AI providers used in extensions
#[async_trait]
pub trait ExtensionAIProvider: Send + Sync {
    /// Send a message to the AI provider and get a response
    async fn send_message(&self, message: &str) -> Result<String>;
}

/// Agent extension manager with search capabilities
pub struct AgentExtensionManager {
    agent_id: String,
    agent_role: AgentRole,
    provider: Arc<dyn ExtensionAIProvider>,
    knowledge_base: Arc<RwLock<KnowledgeBase>>,
    search_engine: SearchEngine,
    learning_assistant: LearningAssistant,
}

/// Knowledge base for storing learned information
#[derive(Debug, Default)]
pub struct KnowledgeBase {
    /// Discovered capabilities and their sources
    capabilities: HashMap<String, CapabilityInfo>,
    /// Learning resources
    resources: Vec<LearningResource>,
    /// Successful patterns
    patterns: Vec<Pattern>,
    /// Failed attempts
    failures: Vec<FailureRecord>,
}

/// Information about a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub description: String,
    pub source: String,
    pub prerequisites: Vec<String>,
    pub examples: Vec<String>,
    pub documentation_urls: Vec<String>,
    pub implementation_complexity: f64,
    pub community_adoption: f64,
}

/// Learning resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResource {
    pub title: String,
    pub url: String,
    pub resource_type: ResourceType,
    pub relevance_score: f64,
    pub last_updated: DateTime<Utc>,
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Documentation,
    Tutorial,
    BlogPost,
    Video,
    Course,
    Book,
    Repository,
}

/// Pattern learned from experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub context: String,
    pub implementation: String,
    pub benefits: Vec<String>,
    pub drawbacks: Vec<String>,
    pub success_rate: f64,
}

/// Record of a failed extension attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub extension_type: String,
    pub failure_reason: String,
    pub timestamp: DateTime<Utc>,
    pub lessons_learned: Vec<String>,
    pub avoidance_strategy: String,
}

/// Search engine for finding extension opportunities
pub struct SearchEngine {
    search_strategies: Vec<Box<dyn SearchStrategy>>,
}

/// Search strategy trait
#[async_trait]
pub trait SearchStrategy: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn name(&self) -> &str;
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub keywords: Vec<String>,
    pub context: SearchContext,
    pub filters: SearchFilters,
}

/// Context for the search
#[derive(Debug, Clone)]
pub enum SearchContext {
    CapabilityGap { current: Vec<String>, desired: Vec<String> },
    ErrorResolution { error_pattern: String },
    PerformanceOptimization { bottleneck: String },
    TechnologyTrend { trend: String },
}

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub min_relevance: f64,
    pub max_complexity: f64,
    pub preferred_sources: Vec<String>,
    pub language: Option<String>,
    pub recency_days: Option<u32>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub source: String,
    pub relevance_score: f64,
    pub summary: String,
    pub url: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Documentation search strategy
pub struct DocumentationSearchStrategy {
    doc_sources: Vec<String>,
}

#[async_trait]
impl SearchStrategy for DocumentationSearchStrategy {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        // Search through documentation sources
        for source in &self.doc_sources {
            match source.as_str() {
                "mdn" => results.extend(self.search_mdn(query).await?),
                "react" => results.extend(self.search_react_docs(query).await?),
                "rust" => results.extend(self.search_rust_docs(query).await?),
                _ => continue,
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(results)
    }
    
    fn name(&self) -> &str {
        "Documentation Search"
    }
}

impl DocumentationSearchStrategy {
    pub fn new() -> Self {
        Self {
            doc_sources: vec!["mdn".to_string(), "react".to_string(), "rust".to_string()],
        }
    }
    
    async fn search_mdn(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        use reqwest;
        use serde_json::Value;
        
        let search_terms = query.keywords.join(" ");
        let url = format!("https://developer.mozilla.org/api/v1/search?q={}", 
                         urlencoding::encode(&search_terms));
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "ccswarm/0.3.0")
            .send()
            .await?;
            
        if response.status().is_success() {
            let json: Value = response.json().await?;
            let mut results = Vec::new();
            
            if let Some(documents) = json["documents"].as_array() {
                for doc in documents.iter().take(5) {
                    if let (Some(title), Some(url), Some(summary)) = (
                        doc["title"].as_str(),
                        doc["mdn_url"].as_str(),
                        doc["summary"].as_str()
                    ) {
                        results.push(SearchResult {
                            title: title.to_string(),
                            url: Some(format!("https://developer.mozilla.org{}", url)),
                            summary: summary.to_string(),
                            relevance_score: 0.8,
                            source: "MDN".to_string(),
                            metadata: std::collections::HashMap::new(),
                        });
                    }
                }
            }
            
            Ok(results)
        } else {
            Ok(vec![])
        }
    }
    
    async fn search_react_docs(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // Simulate React docs search
        Ok(vec![])
    }
    
    async fn search_rust_docs(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // Simulate Rust docs search
        Ok(vec![])
    }
}

/// GitHub repository search strategy
pub struct GitHubSearchStrategy {
    api_token: Option<String>,
}

#[async_trait]
impl SearchStrategy for GitHubSearchStrategy {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // Search GitHub for relevant repositories, issues, and discussions
        let search_terms = query.keywords.join(" ");
        
        // Use GitHub API or gh CLI
        let output = Command::new("gh")
            .args(&["search", "repos", &search_terms, "--limit", "10", "--json", "name,description,url,stargazersCount"])
            .output()
            .await?;
            
        // Parse results
        let results = self.parse_github_results(&output.stdout)?;
        
        Ok(results)
    }
    
    fn name(&self) -> &str {
        "GitHub Search"
    }
}

impl GitHubSearchStrategy {
    pub fn new() -> Self {
        Self {
            api_token: std::env::var("GITHUB_TOKEN").ok(),
        }
    }
    
    fn parse_github_results(&self, data: &[u8]) -> Result<Vec<SearchResult>> {
        use serde_json::Value;
        
        let json_str = std::str::from_utf8(data)?;
        if json_str.trim().is_empty() {
            return Ok(vec![]);
        }
        
        let json: Value = serde_json::from_str(json_str)?;
        let mut results = Vec::new();
        
        if let Some(repos) = json.as_array() {
            for repo in repos.iter().take(5) {
                if let (Some(name), Some(description), Some(url)) = (
                    repo["name"].as_str(),
                    repo["description"].as_str(),
                    repo["url"].as_str()
                ) {
                    let stars = repo["stargazersCount"].as_u64().unwrap_or(0);
                    let relevance_score = if stars > 1000 {
                        0.9
                    } else if stars > 100 {
                        0.7
                    } else {
                        0.5
                    };
                    
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert("stars".to_string(), stars.to_string());
                    
                    results.push(SearchResult {
                        title: name.to_string(),
                        url: Some(url.to_string()),
                        summary: description.to_string(),
                        relevance_score,
                        source: "GitHub".to_string(),
                        metadata,
                    });
                }
            }
        }
        
        Ok(results)
    }
}

/// Stack Overflow search strategy
pub struct StackOverflowSearchStrategy;

#[async_trait]
impl SearchStrategy for StackOverflowSearchStrategy {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        use reqwest;
        use serde_json::Value;
        
        let search_terms = query.keywords.join(" ");
        let url = format!(
            "https://api.stackexchange.com/2.3/search?order=desc&sort=relevance&intitle={}&site=stackoverflow",
            urlencoding::encode(&search_terms)
        );
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "ccswarm/0.3.0")
            .send()
            .await?;
            
        if response.status().is_success() {
            let json: Value = response.json().await?;
            let mut results = Vec::new();
            
            if let Some(items) = json["items"].as_array() {
                for item in items.iter().take(5) {
                    if let (Some(title), Some(link), Some(score)) = (
                        item["title"].as_str(),
                        item["link"].as_str(),
                        item["score"].as_i64()
                    ) {
                        let relevance_score = if score > 10 {
                            0.9
                        } else if score > 0 {
                            0.7
                        } else {
                            0.5
                        };
                        
                        let mut metadata = std::collections::HashMap::new();
                        metadata.insert("score".to_string(), score.to_string());
                        if let Some(answer_count) = item["answer_count"].as_i64() {
                            metadata.insert("answer_count".to_string(), answer_count.to_string());
                        }
                        
                        results.push(SearchResult {
                            title: title.to_string(),
                            url: Some(link.to_string()),
                            summary: format!("Score: {}, Answers: {}", 
                                           score, 
                                           item["answer_count"].as_i64().unwrap_or(0)),
                            relevance_score,
                            source: "StackOverflow".to_string(),
                            metadata,
                        });
                    }
                }
            }
            
            Ok(results)
        } else {
            Ok(vec![])
        }
    }
    
    fn name(&self) -> &str {
        "Stack Overflow Search"
    }
}

/// Learning assistant for processing search results
pub struct LearningAssistant {
    provider: Arc<dyn ExtensionAIProvider>,
}

impl LearningAssistant {
    /// Analyze search results and extract actionable insights
    pub async fn analyze_results(
        &self,
        results: &[SearchResult],
        context: &SearchContext,
    ) -> Result<LearningAnalysis> {
        let prompt = self.build_analysis_prompt(results, context);
        
        // Use provider to analyze
        let response = self.provider.send_message(&prompt).await?;
        
        // Parse the analysis
        self.parse_analysis(&response)
    }
    
    /// Generate implementation plan from learning
    pub async fn generate_implementation_plan(
        &self,
        analysis: &LearningAnalysis,
        current_capabilities: &[String],
    ) -> Result<ImplementationPlan> {
        let prompt = format!(
            "Based on the following analysis and current capabilities, generate a detailed implementation plan:\n\n\
            Analysis: {:?}\n\n\
            Current capabilities: {:?}\n\n\
            Generate a step-by-step implementation plan with phases, tasks, and validation methods.",
            analysis, current_capabilities
        );
        
        let response = self.provider.send_message(&prompt).await?;
        
        // Parse the plan
        self.parse_implementation_plan(&response)
    }
    
    fn build_analysis_prompt(&self, results: &[SearchResult], context: &SearchContext) -> String {
        format!(
            "Analyze the following search results in the context of {:?}:\n\n\
            Results:\n{}\n\n\
            Provide:\n\
            1. Key insights\n\
            2. Recommended approach\n\
            3. Prerequisites\n\
            4. Potential challenges\n\
            5. Success criteria",
            context,
            results.iter()
                .map(|r| format!("- {} (relevance: {:.2}): {}", r.title, r.relevance_score, r.summary))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
    
    fn parse_analysis(&self, _response: &str) -> Result<LearningAnalysis> {
        // Parse the AI response into structured analysis
        Ok(LearningAnalysis {
            key_insights: vec![],
            recommended_approach: String::new(),
            prerequisites: vec![],
            challenges: vec![],
            success_criteria: vec![],
        })
    }
    
    fn parse_implementation_plan(&self, _response: &str) -> Result<ImplementationPlan> {
        // Parse the AI response into structured plan
        Ok(ImplementationPlan {
            phases: vec![],
            timeline: String::new(),
            resources_required: vec![],
            dependencies: vec![],
        })
    }
}

/// Analysis result from learning assistant
#[derive(Debug, Clone)]
pub struct LearningAnalysis {
    pub key_insights: Vec<String>,
    pub recommended_approach: String,
    pub prerequisites: Vec<String>,
    pub challenges: Vec<String>,
    pub success_criteria: Vec<String>,
}

impl AgentExtensionManager {
    pub fn new(
        agent_id: String,
        agent_role: AgentRole,
        provider: Arc<dyn ExtensionAIProvider>,
    ) -> Self {
        let search_engine = SearchEngine {
            search_strategies: vec![
                Box::new(DocumentationSearchStrategy {
                    doc_sources: vec!["mdn".to_string(), "react".to_string(), "rust".to_string()],
                }),
                Box::new(GitHubSearchStrategy { api_token: None }),
                Box::new(StackOverflowSearchStrategy),
            ],
        };
        
        let learning_assistant = LearningAssistant {
            provider: provider.clone(),
        };
        
        Self {
            agent_id,
            agent_role,
            provider,
            knowledge_base: Arc::new(RwLock::new(KnowledgeBase::default())),
            search_engine,
            learning_assistant,
        }
    }
    
    /// Discover extension opportunities through search
    pub async fn discover_opportunities(&self) -> Result<Vec<ExtensionOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Search for capability gaps
        opportunities.extend(self.search_capability_gaps().await?);
        
        // Search for performance optimizations
        opportunities.extend(self.search_performance_optimizations().await?);
        
        // Search for industry trends
        opportunities.extend(self.search_industry_trends().await?);
        
        // Rank opportunities
        self.rank_opportunities(&mut opportunities);
        
        Ok(opportunities)
    }
    
    /// Research a specific extension opportunity
    pub async fn research_extension(
        &self,
        opportunity: &ExtensionOpportunity,
    ) -> Result<ExtensionResearch> {
        let query = SearchQuery {
            keywords: opportunity.keywords.clone(),
            context: opportunity.context.clone(),
            filters: SearchFilters {
                min_relevance: 0.7,
                max_complexity: 0.8,
                ..Default::default()
            },
        };
        
        // Search across all strategies
        let mut all_results = Vec::new();
        for strategy in &self.search_engine.search_strategies {
            match strategy.search(&query).await {
                Ok(results) => all_results.extend(results),
                Err(e) => eprintln!("Search strategy {} failed: {}", strategy.name(), e),
            }
        }
        
        // Analyze results
        let analysis = self.learning_assistant
            .analyze_results(&all_results, &query.context)
            .await?;
        
        // Generate implementation plan
        let current_capabilities = self.get_current_capabilities().await?;
        let implementation_plan = self.learning_assistant
            .generate_implementation_plan(&analysis, &current_capabilities)
            .await?;
        
        // Store in knowledge base
        self.store_research(&opportunity, &all_results, &analysis).await?;
        
        let estimated_effort = self.estimate_effort(&implementation_plan);
        let confidence_score = self.calculate_confidence(&analysis);
        
        Ok(ExtensionResearch {
            opportunity: opportunity.clone(),
            search_results: all_results,
            analysis,
            implementation_plan,
            estimated_effort,
            confidence_score,
        })
    }
    
    /// Generate a self-extension proposal
    pub async fn generate_proposal(
        &self,
        research: &ExtensionResearch,
    ) -> Result<ExtensionProposal> {
        let proposal = ExtensionProposal {
            id: Uuid::new_v4(),
            proposer: self.agent_id.clone(),
            extension_type: research.opportunity.extension_type,
            title: research.opportunity.title.clone(),
            description: self.generate_description(&research).await?,
            current_state: self.assess_current_state().await?,
            proposed_state: self.define_proposed_state(&research).await?,
            implementation_plan: research.implementation_plan.clone(),
            risk_assessment: self.assess_risks(&research).await?,
            success_criteria: self.define_success_criteria(&research).await?,
            created_at: Utc::now(),
            status: ExtensionStatus::Proposed,
        };
        
        Ok(proposal)
    }
    
    async fn search_capability_gaps(&self) -> Result<Vec<ExtensionOpportunity>> {
        // Search for capabilities that peers have but this agent doesn't
        Ok(vec![])
    }
    
    async fn search_performance_optimizations(&self) -> Result<Vec<ExtensionOpportunity>> {
        // Search for performance improvement techniques
        Ok(vec![])
    }
    
    async fn search_industry_trends(&self) -> Result<Vec<ExtensionOpportunity>> {
        // Search for relevant industry trends
        Ok(vec![])
    }
    
    fn rank_opportunities(&self, opportunities: &mut Vec<ExtensionOpportunity>) {
        opportunities.sort_by(|a, b| {
            b.potential_impact.partial_cmp(&a.potential_impact).unwrap()
        });
    }
    
    async fn get_current_capabilities(&self) -> Result<Vec<String>> {
        // Get agent's current capabilities
        Ok(vec![])
    }
    
    async fn store_research(
        &self,
        _opportunity: &ExtensionOpportunity,
        results: &[SearchResult],
        _analysis: &LearningAnalysis,
    ) -> Result<()> {
        let mut kb = self.knowledge_base.write().await;
        
        // Store learning resources
        for result in results {
            if result.relevance_score > 0.7 {
                kb.resources.push(LearningResource {
                    title: result.title.clone(),
                    url: result.url.clone().unwrap_or_default(),
                    resource_type: ResourceType::Documentation,
                    relevance_score: result.relevance_score,
                    last_updated: Utc::now(),
                    key_concepts: vec![],
                });
            }
        }
        
        Ok(())
    }
    
    fn estimate_effort(&self, plan: &ImplementationPlan) -> String {
        plan.timeline.clone()
    }
    
    fn calculate_confidence(&self, _analysis: &LearningAnalysis) -> f64 {
        // Calculate confidence based on analysis quality
        0.8
    }
    
    async fn generate_description(&self, research: &ExtensionResearch) -> Result<String> {
        Ok(format!(
            "{}\n\nBased on research: {} relevant resources found with average relevance score of {:.2}",
            research.opportunity.description,
            research.search_results.len(),
            research.search_results.iter().map(|r| r.relevance_score).sum::<f64>() / research.search_results.len() as f64
        ))
    }
    
    async fn assess_current_state(&self) -> Result<CurrentState> {
        Ok(CurrentState {
            capabilities: vec![],
            limitations: vec![],
            performance_metrics: HashMap::new(),
        })
    }
    
    async fn define_proposed_state(&self, _research: &ExtensionResearch) -> Result<ProposedState> {
        Ok(ProposedState {
            new_capabilities: vec![],
            expected_improvements: vec![],
            performance_targets: HashMap::new(),
        })
    }
    
    async fn assess_risks(&self, _research: &ExtensionResearch) -> Result<RiskAssessment> {
        Ok(RiskAssessment {
            risks: vec![],
            mitigation_strategies: vec![],
            rollback_plan: "Revert to previous state".to_string(),
            overall_risk_score: 0.3,
        })
    }
    
    async fn define_success_criteria(&self, research: &ExtensionResearch) -> Result<Vec<SuccessCriterion>> {
        Ok(research.analysis.success_criteria.iter().map(|c| {
            SuccessCriterion {
                description: c.clone(),
                metric: "TBD".to_string(),
                target_value: "TBD".to_string(),
                measurement_method: "TBD".to_string(),
            }
        }).collect())
    }
}

/// Extension opportunity discovered through search
#[derive(Debug, Clone)]
pub struct ExtensionOpportunity {
    pub title: String,
    pub description: String,
    pub extension_type: ExtensionType,
    pub keywords: Vec<String>,
    pub context: SearchContext,
    pub potential_impact: f64,
    pub discovery_source: String,
}

/// Research results for an extension
#[derive(Debug, Clone)]
pub struct ExtensionResearch {
    pub opportunity: ExtensionOpportunity,
    pub search_results: Vec<SearchResult>,
    pub analysis: LearningAnalysis,
    pub implementation_plan: ImplementationPlan,
    pub estimated_effort: String,
    pub confidence_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_engine() {
        let engine = SearchEngine {
            search_strategies: vec![
                Box::new(DocumentationSearchStrategy {
                    doc_sources: vec!["mdn".to_string()],
                }),
            ],
        };
        
        let query = SearchQuery {
            keywords: vec!["react".to_string(), "server".to_string(), "components".to_string()],
            context: SearchContext::CapabilityGap {
                current: vec!["React".to_string()],
                desired: vec!["React Server Components".to_string()],
            },
            filters: Default::default(),
        };
        
        // Should not panic
        let _ = engine.search_strategies[0].search(&query).await;
    }
}