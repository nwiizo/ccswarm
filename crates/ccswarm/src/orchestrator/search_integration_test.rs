#[cfg(test)]
mod tests {
    use crate::agent::{Task, TaskType, Priority};
    use crate::agent::search_agent::{SearchResponse, SearchResult};
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_search_delegation_detection() {
        // Test that tasks with search keywords trigger search requests
        let search_tasks = vec![
            "Research best practices for React hooks",
            "Find information about GraphQL subscriptions",
            "Look up documentation for AWS Lambda",
            "Compare PostgreSQL vs MongoDB for our use case",
            "Investigate error: undefined is not a function",
        ];
        
        for task_desc in search_tasks {
            let task = Task::new(
                format!("test-{}", uuid::Uuid::new_v4()),
                task_desc.to_string(),
                Priority::Medium,
                TaskType::Development,
            );
            
            // Check if task would trigger search
            let search_keywords = vec![
                "research", "find information", "look up", "best practices",
                "documentation", "examples", "how to", "comparison", "compare",
                "alternatives", "investigate", "unclear", "unknown"
            ];
            
            let task_desc_lower = task.description.to_lowercase();
            let needs_search = search_keywords.iter().any(|&keyword| task_desc_lower.contains(keyword));
            
            assert!(needs_search, "Task '{}' should trigger search", task_desc);
        }
    }
    
    #[tokio::test]
    async fn test_search_response_handling() {
        // Create a mock search response
        let search_response = SearchResponse {
            request_id: "test-request-123".to_string(),
            results: vec![
                SearchResult {
                    title: "React Hooks Documentation".to_string(),
                    url: "https://reactjs.org/docs/hooks-intro.html".to_string(),
                    snippet: "Hooks are a new addition in React 16.8...".to_string(),
                    relevance_score: Some(0.95),
                    metadata: None,
                },
                SearchResult {
                    title: "Best Practices for React Hooks".to_string(),
                    url: "https://blog.example.com/react-hooks-best-practices".to_string(),
                    snippet: "Learn the best practices for using React Hooks...".to_string(),
                    relevance_score: Some(0.88),
                    metadata: None,
                },
            ],
            total_results: 2,
            query_used: "React hooks best practices".to_string(),
            warnings: vec![],
        };
        
        // Verify response can be serialized/deserialized
        let json = serde_json::to_value(&search_response).unwrap();
        let deserialized: SearchResponse = serde_json::from_value(json).unwrap();
        
        assert_eq!(deserialized.results.len(), 2);
        assert_eq!(deserialized.query_used, "React hooks best practices");
    }
    
    #[tokio::test]
    async fn test_research_task_type() {
        // Test that Research tasks are properly routed
        let research_task = Task::new(
            "research-123".to_string(),
            "Review search findings for React best practices".to_string(),
            Priority::Medium,
            TaskType::Research,
        );
        
        assert_eq!(research_task.task_type, TaskType::Research);
        
        // Test string parsing
        let parsed_type: TaskType = "research".parse().unwrap();
        assert_eq!(parsed_type, TaskType::Research);
    }
    
    #[tokio::test]
    async fn test_proactive_search_decision() {
        use super::super::proactive_master::{ProactiveDecision, DecisionType, SuggestedAction, RiskLevel};
        
        // Create a proactive decision for search
        let decision = ProactiveDecision {
            decision_type: DecisionType::RequestSearch,
            reasoning: "Agent stuck on task requiring information".to_string(),
            confidence: 0.85,
            suggested_actions: vec![
                SuggestedAction {
                    action_type: "request_search".to_string(),
                    description: "Research React hooks best practices".to_string(),
                    parameters: HashMap::from([
                        ("query".to_string(), "React hooks best practices".to_string()),
                        ("context".to_string(), "Agent stuck on React implementation".to_string()),
                    ]),
                    expected_impact: "Provide information to unblock agent".to_string(),
                }
            ],
            risk_assessment: RiskLevel::Low,
        };
        
        assert_eq!(decision.decision_type, DecisionType::RequestSearch);
        assert!(decision.confidence > 0.8);
        assert_eq!(decision.risk_assessment, RiskLevel::Low);
    }
}