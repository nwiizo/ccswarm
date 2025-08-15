//! Integration tests for semantic analysis features

use ccswarm::semantic::analyzer::{ChangeType, Symbol, SymbolChange, SymbolKind};
use ccswarm::semantic::memory::{Memory, MemoryType};
use ccswarm::semantic::subagent_integration::{AgentRole, SemanticTools};
use ccswarm::semantic::task_analyzer::{Task, TaskPriority};
use ccswarm::semantic::{
    ProjectMemory, SemanticAnalyzer, SemanticConfig, SemanticManager, SemanticSubAgent,
    SemanticTaskAnalyzer, SymbolIndex,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_semantic_manager_initialization() {
    let config = SemanticConfig::default();
    let manager = SemanticManager::new(config).await;

    assert!(
        manager.is_ok(),
        "SemanticManager should initialize successfully"
    );

    let manager = manager.unwrap();
    assert!(Arc::strong_count(&manager.analyzer()) > 0);
    assert!(Arc::strong_count(&manager.memory()) > 0);
    assert!(Arc::strong_count(&manager.symbol_index()) > 0);
}

#[tokio::test]
async fn test_semantic_analyzer_symbol_operations() {
    let config = SemanticConfig::default();
    let analyzer = SemanticAnalyzer::new(config).await.unwrap();

    // Create a test symbol
    let symbol = Symbol {
        name: "test_function".to_string(),
        path: "module::test_function".to_string(),
        kind: SymbolKind::Function,
        file_path: "src/test.rs".to_string(),
        line: 10,
        body: Some("fn test_function() {}".to_string()),
        references: vec![],
        metadata: HashMap::new(),
    };

    // Register the symbol
    analyzer.register_symbol(symbol.clone()).await.unwrap();

    // Find the symbol
    let found = analyzer
        .find_symbol("test_function", Some(SymbolKind::Function))
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test_function");

    // Test finding relevant symbols
    let relevant = analyzer
        .find_relevant_symbols("test function implementation")
        .await
        .unwrap();
    assert!(!relevant.is_empty());
}

#[tokio::test]
async fn test_project_memory_operations() {
    let memory_manager = ProjectMemory::new(10).await.unwrap();

    // Create a test memory
    let memory = Memory {
        id: "test_memory_1".to_string(),
        name: "Test Architecture Decision".to_string(),
        content: "We decided to use semantic analysis for better code understanding".to_string(),
        memory_type: MemoryType::Architecture,
        related_symbols: vec!["module::function".to_string()],
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        access_count: 0,
    };

    // Store the memory
    memory_manager.store_memory(memory.clone()).await.unwrap();

    // Retrieve the memory
    let retrieved = memory_manager
        .get_memory("Test Architecture Decision")
        .await
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, memory.content);

    // List memories
    let memories = memory_manager.list_memories().await.unwrap();
    assert!(!memories.is_empty());
}

#[tokio::test]
async fn test_symbol_index_operations() {
    let index = SymbolIndex::new().await.unwrap();

    // Add test symbols
    let symbol1 = Symbol {
        name: "MyStruct".to_string(),
        path: "module::MyStruct".to_string(),
        kind: SymbolKind::Struct,
        file_path: "src/lib.rs".to_string(),
        line: 20,
        body: None,
        references: vec![],
        metadata: HashMap::new(),
    };

    let symbol2 = Symbol {
        name: "my_function".to_string(),
        path: "module::my_function".to_string(),
        kind: SymbolKind::Function,
        file_path: "src/lib.rs".to_string(),
        line: 30,
        body: None,
        references: vec!["module::MyStruct".to_string()],
        metadata: HashMap::new(),
    };

    index.add_symbol(symbol1.clone()).await.unwrap();
    index.add_symbol(symbol2.clone()).await.unwrap();

    // Find by name
    let found = index.find_by_name("MyStruct").await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "MyStruct");

    // Find by kind
    let functions = index.find_by_kind(SymbolKind::Function).await.unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "my_function");

    // Add dependency
    index
        .add_dependency("module::my_function", "module::MyStruct")
        .await
        .unwrap();

    // Get dependencies
    let deps = index.get_dependencies("module::my_function").await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], "module::MyStruct");

    // Get dependents
    let dependents = index.get_dependents("module::MyStruct").await.unwrap();
    assert_eq!(dependents.len(), 1);
    assert_eq!(dependents[0], "module::my_function");
}

#[tokio::test]
async fn test_task_analyzer() {
    let config = SemanticConfig::default();
    let analyzer = Arc::new(SemanticAnalyzer::new(config.clone()).await.unwrap());
    let index = Arc::new(SymbolIndex::new().await.unwrap());
    let memory = Arc::new(ProjectMemory::new(10).await.unwrap());

    let task_analyzer = SemanticTaskAnalyzer::new(analyzer, index, memory);

    // Create a test task
    let task = Task {
        id: "task_1".to_string(),
        title: "Implement user authentication".to_string(),
        description: "Add user authentication with JWT tokens to the backend API".to_string(),
        priority: TaskPriority::High,
        tags: vec!["backend".to_string(), "security".to_string()],
        assigned_agent: None,
    };

    // Analyze the task
    let context = task_analyzer.analyze_task(&task).await.unwrap();

    assert_eq!(context.task.id, "task_1");
    assert!(!context.recommended_approach.steps.is_empty());
    assert_eq!(
        context.recommended_approach.recommended_agent,
        "backend-specialist"
    );
    assert!(!context
        .recommended_approach
        .required_capabilities
        .is_empty());
}

#[tokio::test]
async fn test_api_change_propagation() {
    let config = SemanticConfig::default();
    let memory = Arc::new(ProjectMemory::new(10).await.unwrap());
    let index = Arc::new(SymbolIndex::new().await.unwrap());

    let knowledge_sharing =
        ccswarm::semantic::SemanticKnowledgeSharing::new(memory.clone(), index.clone())
            .await
            .unwrap();

    // Create a frontend change that affects API
    let api_symbol = Symbol {
        name: "getUserData".to_string(),
        path: "api::getUserData".to_string(),
        kind: SymbolKind::Function,
        file_path: "src/api.ts".to_string(),
        line: 100,
        body: Some("function getUserData() {}".to_string()),
        references: vec![],
        metadata: HashMap::new(),
    };

    let change = SymbolChange {
        symbol: api_symbol,
        change_type: ChangeType::ApiModification,
        old_value: Some("function getUserData() {}".to_string()),
        new_value: Some("function getUserData(userId: string) {}".to_string()),
    };

    // Propagate the change
    let backend_tasks = knowledge_sharing
        .propagate_api_changes(&[change])
        .await
        .unwrap();

    assert!(!backend_tasks.is_empty());
    assert!(backend_tasks[0].title.contains("Update backend"));
    assert_eq!(
        backend_tasks[0].priority,
        ccswarm::semantic::knowledge_sharing::TaskPriority::High
    );
}

#[tokio::test]
async fn test_semantic_subagent_creation() {
    let config = SemanticConfig::default();
    let analyzer = Arc::new(SemanticAnalyzer::new(config.clone()).await.unwrap());
    let index = Arc::new(SymbolIndex::new().await.unwrap());
    let memory = Arc::new(ProjectMemory::new(10).await.unwrap());

    // Create semantic tools
    let symbol_manipulator = Arc::new(
        ccswarm::semantic::subagent_integration::SymbolManipulator::new(
            analyzer.clone(),
            index.clone(),
        ),
    );

    let code_searcher = Arc::new(ccswarm::semantic::subagent_integration::CodeSearcher::new(
        index.clone(),
        analyzer.clone(),
    ));

    let refactoring_advisor = Arc::new(
        ccswarm::semantic::subagent_integration::RefactoringAdvisor::new(
            analyzer.clone(),
            memory.clone(),
        ),
    );

    let dependency_analyzer =
        Arc::new(ccswarm::semantic::subagent_integration::DependencyAnalyzer::new(index.clone()));

    let semantic_tools = SemanticTools {
        symbol_manipulator,
        code_searcher,
        refactoring_advisor,
        dependency_analyzer,
    };

    let memory_access = ccswarm::semantic::subagent_integration::MemoryAccess::new(memory);

    // Create a frontend specialist
    let frontend_agent = SemanticSubAgent::new(
        "frontend-specialist".to_string(),
        AgentRole::Frontend,
        "Frontend development specialist with semantic understanding".to_string(),
        semantic_tools,
        memory_access,
    );

    assert_eq!(frontend_agent.name, "frontend-specialist");
    assert_eq!(frontend_agent.role, AgentRole::Frontend);
    assert!(!frontend_agent.capabilities.is_empty());
    assert!(frontend_agent
        .capabilities
        .contains(&"React component architecture".to_string()));
}
