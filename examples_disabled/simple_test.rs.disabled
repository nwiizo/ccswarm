//! Simple test to check what's working in ccswarm

fn main() {
    println!("Testing ccswarm components...\n");
    
    // Test 1: Basic types
    println!("1. Testing basic enums and structs:");
    test_basic_types();
    
    // Test 2: Module availability
    println!("\n2. Testing module availability:");
    test_modules();
}

fn test_basic_types() {
    use ccswarm::identity::AgentRole;
    use ccswarm::agent::{Priority, TaskType};
    
    println!("  - AgentRole variants:");
    for role in [AgentRole::Frontend, AgentRole::Backend, AgentRole::DevOps, AgentRole::QA] {
        println!("    {:?}", role);
    }
    
    println!("  - Priority levels:");
    for priority in [Priority::Low, Priority::Medium, Priority::High] {
        println!("    {:?}", priority);
    }
    
    println!("  - Task types:");
    for task_type in [TaskType::Feature, TaskType::Bug, TaskType::Test, TaskType::Documentation] {
        println!("    {:?}", task_type);
    }
}

fn test_modules() {
    // Test personality module
    {
        use ccswarm::agent::personality::{SkillLevel, SkillCategory, WorkingStyle};
        println!("  ✓ Personality module loaded");
        println!("    - Skill levels: {:?}", [SkillLevel::Novice, SkillLevel::Expert]);
        println!("    - Working styles: {:?}", [WorkingStyle::Exploratory, WorkingStyle::Methodical]);
    }
    
    // Test whiteboard module
    {
        use ccswarm::agent::whiteboard::{EntryType, AnnotationMarker};
        println!("  ✓ Whiteboard module loaded");
        println!("    - Entry types: {:?}", [EntryType::Note, EntryType::Idea, EntryType::Decision]);
        println!("    - Annotations: {:?}", [AnnotationMarker::Important, AnnotationMarker::Resolved]);
    }
    
    // Test phronesis module
    {
        use ccswarm::agent::phronesis::{WisdomCategory, LearningEventType};
        println!("  ✓ Phronesis module loaded");
        println!("    - Wisdom categories: {:?}", [WisdomCategory::BestPractice, WisdomCategory::ProblemSolving]);
        println!("    - Learning events: {:?}", [LearningEventType::Success, LearningEventType::Failure]);
    }
    
    // Test config module
    {
        use ccswarm::config::ClaudeConfig;
        println!("  ✓ Config module loaded");
        let config = ClaudeConfig::default();
        println!("    - Default model: {}", config.model);
    }
    
    // Test providers
    {
        use ccswarm::providers::AIProvider;
        println!("  ✓ Providers module loaded");
        println!("    - Available providers: {:?}", 
            [AIProvider::ClaudeCode, AIProvider::Aider, AIProvider::Codex, AIProvider::Custom]);
    }
}