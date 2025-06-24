//! Demo of working features in ccswarm
//! This example demonstrates the personality, whiteboard, and phronesis systems

use ccswarm::agent::personality::{
    Personality, PersonalityTraits, Skill, SkillCategory, SkillLevel, WorkingStyle,
};
use ccswarm::agent::whiteboard::{Whiteboard, WhiteboardEntry, EntryType, AnnotationMarker};
use ccswarm::agent::phronesis::{PhronesisManager, PracticalWisdom, WisdomCategory, LearningEventType};
use ccswarm::identity::{AgentRole, AgentIdentity, Capabilities};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("🎭 ccswarm Working Features Demo\n");
    
    // 1. Personality System Demo
    println!("=== 1. Personality System ===");
    demo_personality_system();
    
    // 2. Whiteboard System Demo
    println!("\n=== 2. Whiteboard System ===");
    demo_whiteboard_system();
    
    // 3. Phronesis (Practical Wisdom) System Demo
    println!("\n=== 3. Phronesis System ===");
    demo_phronesis_system();
    
    // 4. Identity System Demo
    println!("\n=== 4. Identity System ===");
    demo_identity_system();
}

fn demo_personality_system() {
    // Create skills
    let mut skills = HashMap::new();
    
    skills.insert(
        "React Development".to_string(),
        Skill {
            name: "React Development".to_string(),
            category: SkillCategory::Technical,
            level: SkillLevel::Expert,
            description: "Building modern React applications".to_string(),
            experience_points: 8500,
        },
    );
    
    skills.insert(
        "UI Design".to_string(),
        Skill {
            name: "UI Design".to_string(),
            category: SkillCategory::Creative,
            level: SkillLevel::Proficient,
            description: "Creating beautiful user interfaces".to_string(),
            experience_points: 5200,
        },
    );
    
    // Create personality traits
    let traits = PersonalityTraits {
        curiosity: 0.8,
        attention_to_detail: 0.9,
        risk_taking: 0.4,
        collaboration: 0.7,
        innovation: 0.75,
    };
    
    // Create personality
    let personality = Personality {
        traits,
        skills,
        working_style: WorkingStyle::Methodical,
        preferred_tasks: vec!["UI Development".to_string(), "Component Design".to_string()],
        learning_preferences: vec!["Visual Learning".to_string(), "Hands-on Practice".to_string()],
    };
    
    println!("Frontend Agent Personality:");
    println!("  {}", personality.describe_personality());
    println!("  Working Style: {:?}", personality.working_style);
    println!("  Dominant Trait: {:?}", personality.get_dominant_trait());
    
    // Check capabilities
    println!("\nCapability Checks:");
    println!("  Can do UI tasks: {}", personality.is_capable_of("UI"));
    println!("  Can do backend tasks: {}", personality.is_capable_of("backend"));
}

fn demo_whiteboard_system() {
    let mut whiteboard = Whiteboard::new("frontend-agent");
    
    // Add a note
    let note_id = whiteboard.add_entry(
        EntryType::Note,
        "Need to optimize React component rendering".to_string(),
        None,
    );
    println!("Added note: {}", note_id);
    
    // Add an idea
    let idea_id = whiteboard.add_entry(
        EntryType::Idea,
        "Use React.memo for expensive components".to_string(),
        Some(vec![note_id.clone()]),
    );
    
    // Add a question
    let question_id = whiteboard.add_entry(
        EntryType::Question,
        "Should we use useMemo or useCallback here?".to_string(),
        Some(vec![idea_id.clone()]),
    );
    
    // Annotate with importance
    whiteboard.annotate_entry(&idea_id, AnnotationMarker::Important);
    
    // Add a decision
    let decision_id = whiteboard.add_entry(
        EntryType::Decision,
        "Implement React.memo with custom comparison function".to_string(),
        Some(vec![question_id]),
    );
    whiteboard.annotate_entry(&decision_id, AnnotationMarker::Resolved);
    
    // Display whiteboard
    println!("\nWhiteboard Contents:");
    for entry in whiteboard.get_all_entries() {
        println!("  [{:?}] {}: {}", 
            entry.entry_type, 
            entry.id.chars().take(8).collect::<String>(),
            entry.content
        );
        if !entry.annotations.is_empty() {
            println!("    Annotations: {:?}", entry.annotations);
        }
    }
    
    // Show related entries
    let related = whiteboard.get_related_entries(&idea_id);
    println!("\nEntries related to optimization idea: {}", related.len());
}

fn demo_phronesis_system() {
    let mut phronesis = PhronesisManager::new("backend-agent");
    
    // Record some practical wisdom
    let wisdom1 = phronesis.record_wisdom(
        WisdomCategory::ProblemSolving,
        "Always validate input at the API boundary".to_string(),
        "Prevents security vulnerabilities and improves error handling".to_string(),
        1.0, // High confidence
    );
    
    let wisdom2 = phronesis.record_wisdom(
        WisdomCategory::BestPractice,
        "Use connection pooling for database access".to_string(),
        "Improves performance by reusing connections".to_string(),
        0.9,
    );
    
    // Record a learning event
    phronesis.record_learning_event(
        LearningEventType::Success,
        "Implemented caching layer".to_string(),
        "Reduced API response time by 60%".to_string(),
        vec![wisdom2],
    );
    
    // Get wisdom for a category
    let problem_solving_wisdom = phronesis.get_wisdom_by_category(&WisdomCategory::ProblemSolving);
    println!("Problem Solving Wisdom:");
    for wisdom in problem_solving_wisdom {
        println!("  - {}: {} (confidence: {:.1})", 
            wisdom.principle, 
            wisdom.rationale,
            wisdom.confidence_score
        );
    }
    
    // Get highly confident wisdom
    let high_confidence = phronesis.get_high_confidence_wisdom(0.8);
    println!("\nHigh Confidence Wisdom (>0.8):");
    for wisdom in high_confidence {
        println!("  - {:?}: {}", wisdom.category, wisdom.principle);
    }
    
    // Show decision making capability
    let context = "Need to handle high traffic API endpoint";
    let decisions = phronesis.make_decision(context);
    println!("\nDecisions for '{}': {} relevant principles", context, decisions.len());
}

fn demo_identity_system() {
    // Create a frontend identity
    let capabilities = Capabilities {
        languages: vec!["TypeScript".to_string(), "JavaScript".to_string()],
        frameworks: vec!["React".to_string(), "Next.js".to_string()],
        tools: vec!["Webpack".to_string(), "ESLint".to_string()],
        domains: vec!["UI/UX".to_string(), "State Management".to_string()],
    };
    
    let identity = AgentIdentity {
        id: Uuid::new_v4(),
        name: "frontend-specialist".to_string(),
        role: AgentRole::Frontend,
        capabilities,
        constraints: vec![
            "Cannot modify backend code".to_string(),
            "Must follow design system guidelines".to_string(),
        ],
        created_at: Utc::now(),
    };
    
    println!("Agent Identity:");
    println!("  Name: {}", identity.name);
    println!("  Role: {:?}", identity.role);
    println!("  Languages: {:?}", identity.capabilities.languages);
    println!("  Frameworks: {:?}", identity.capabilities.frameworks);
    println!("  Constraints: {:?}", identity.constraints);
    
    // Role descriptions
    println!("\nRole Descriptions:");
    for role in [AgentRole::Frontend, AgentRole::Backend, AgentRole::DevOps, AgentRole::QA] {
        println!("  {:?}: {}", role, get_role_description(&role));
    }
}

fn get_role_description(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Frontend => "Specializes in user interfaces, React, and client-side development",
        AgentRole::Backend => "Handles APIs, databases, and server-side logic",
        AgentRole::DevOps => "Manages infrastructure, deployment, and CI/CD pipelines",
        AgentRole::QA => "Focuses on testing, quality assurance, and test automation",
    }
}