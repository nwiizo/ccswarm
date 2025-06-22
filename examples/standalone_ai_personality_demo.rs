//! Standalone demonstration of AI agent personality systems
//! This runs independently without requiring the full ccswarm compilation

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use colored::*;
use rand::Rng;

// Skill system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillLevel {
    Novice,
    Competent,
    Proficient,
    Expert,
    Master,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub category: SkillCategory,
    pub level: SkillLevel,
    pub experience_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillCategory {
    Technical,
    Creative,
    Analytical,
    Communication,
    Leadership,
}

// Personality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub agent_id: String,
    pub skills: HashMap<String, Skill>,
    pub traits: PersonalityTraits,
    pub working_style: WorkingStyle,
    pub motto: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub creativity: f32,
    pub analytical_thinking: f32,
    pub risk_tolerance: f32,
    pub collaboration: f32,
    pub attention_to_detail: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkingStyle {
    Methodical,
    Creative,
    Analytical,
    Collaborative,
    Balanced,
}

// Whiteboard for thought visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: EntryType,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Calculation { expression: String, result: Option<String> },
    Hypothesis { statement: String, confidence: f32 },
    ThoughtTrace { thoughts: Vec<String>, conclusion: Option<String> },
    Diagram { title: String, elements: Vec<String> },
}

// Phronesis (practical wisdom)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticalWisdom {
    pub category: WisdomCategory,
    pub insight: String,
    pub confidence: f32,
    pub applications: Vec<WisdomApplication>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WisdomCategory {
    TaskExecution,
    ErrorHandling,
    PatternRecognition,
    Collaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WisdomApplication {
    pub context: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

// Agent implementation
pub struct AIAgent {
    pub personality: Personality,
    pub whiteboard: Vec<WhiteboardEntry>,
    pub wisdom: Vec<PracticalWisdom>,
}

impl AIAgent {
    pub fn new(agent_id: String, role: &str) -> Self {
        let mut skills = HashMap::new();
        
        // Initialize skills based on role
        match role {
            "frontend" => {
                skills.insert("React Development".to_string(), Skill {
                    name: "React Development".to_string(),
                    category: SkillCategory::Technical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
                skills.insert("UI Design".to_string(), Skill {
                    name: "UI Design".to_string(),
                    category: SkillCategory::Creative,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
            }
            "backend" => {
                skills.insert("API Design".to_string(), Skill {
                    name: "API Design".to_string(),
                    category: SkillCategory::Technical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
                skills.insert("Database Optimization".to_string(), Skill {
                    name: "Database Optimization".to_string(),
                    category: SkillCategory::Analytical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
            }
            _ => {}
        }
        
        let personality = Personality {
            agent_id: agent_id.clone(),
            skills,
            traits: PersonalityTraits {
                creativity: 0.5,
                analytical_thinking: 0.5,
                risk_tolerance: 0.5,
                collaboration: 0.5,
                attention_to_detail: 0.5,
            },
            working_style: WorkingStyle::Balanced,
            motto: Some(format!("Excellence in {}", role)),
        };
        
        Self {
            personality,
            whiteboard: Vec::new(),
            wisdom: Vec::new(),
        }
    }
    
    pub fn gain_experience(&mut self, skill_name: &str, points: u32) {
        if let Some(skill) = self.personality.skills.get_mut(skill_name) {
            skill.experience_points += points;
            
            // Level up logic
            let new_level = match skill.experience_points {
                0..=99 => SkillLevel::Novice,
                100..=299 => SkillLevel::Competent,
                300..=599 => SkillLevel::Proficient,
                600..=999 => SkillLevel::Expert,
                _ => SkillLevel::Master,
            };
            
            if skill.level != new_level {
                println!("{} {} leveled up to {:?}!", 
                    "🎉".yellow(), 
                    skill_name, 
                    new_level
                );
                skill.level = new_level;
            }
        }
    }
    
    pub fn add_thought(&mut self, thought_type: EntryType) {
        let entry = WhiteboardEntry {
            id: format!("thought-{}", self.whiteboard.len()),
            timestamp: Utc::now(),
            entry_type: thought_type,
            annotations: Vec::new(),
        };
        self.whiteboard.push(entry);
    }
    
    pub fn learn_from_experience(&mut self, insight: String, category: WisdomCategory) {
        let wisdom = PracticalWisdom {
            category,
            insight,
            confidence: 0.7,
            applications: Vec::new(),
        };
        self.wisdom.push(wisdom);
    }
    
    pub fn display_status(&self) {
        println!("\n{} Agent Status", "📊".blue());
        println!("ID: {}", self.personality.agent_id);
        println!("Working Style: {:?}", self.personality.working_style);
        
        println!("\n{} Skills:", "💪".yellow());
        for (name, skill) in &self.personality.skills {
            let bar_length = (skill.experience_points as f32 / 100.0).min(10.0) as usize;
            let progress_bar = "█".repeat(bar_length).green().to_string() 
                + &"░".repeat(10 - bar_length).dimmed().to_string();
            println!("  {} - {:?} {} ({}xp)", 
                name, 
                skill.level, 
                progress_bar,
                skill.experience_points
            );
        }
        
        println!("\n{} Recent Thoughts:", "💭".cyan());
        for entry in self.whiteboard.iter().rev().take(3) {
            match &entry.entry_type {
                EntryType::Hypothesis { statement, confidence } => {
                    println!("  • Hypothesis: {} ({}% confident)", 
                        statement, 
                        (confidence * 100.0) as u32
                    );
                }
                EntryType::ThoughtTrace { thoughts, conclusion } => {
                    println!("  • Thought: {}", thoughts.join(" → "));
                    if let Some(c) = conclusion {
                        println!("    Conclusion: {}", c);
                    }
                }
                _ => {}
            }
        }
        
        println!("\n{} Wisdom Gained:", "🧠".magenta());
        for wisdom in &self.wisdom {
            println!("  • {}: {}", 
                match wisdom.category {
                    WisdomCategory::TaskExecution => "Task",
                    WisdomCategory::ErrorHandling => "Error",
                    WisdomCategory::PatternRecognition => "Pattern",
                    WisdomCategory::Collaboration => "Collab",
                },
                wisdom.insight
            );
        }
    }
}

fn main() {
    println!("{}", "🤖 AI Agent Personality & Learning Demo".bold().cyan());
    println!("{}", "=====================================\n".dimmed());
    
    // Create agents
    let mut frontend_agent = AIAgent::new("agent-001".to_string(), "frontend");
    let mut backend_agent = AIAgent::new("agent-002".to_string(), "backend");
    
    // Simulate task execution
    println!("{} Simulating agent activities...\n", "🚀".green());
    
    // Frontend agent works on UI tasks
    frontend_agent.add_thought(EntryType::Hypothesis {
        statement: "Using CSS Grid will improve layout flexibility".to_string(),
        confidence: 0.8,
    });
    
    frontend_agent.gain_experience("React Development", 50);
    frontend_agent.gain_experience("UI Design", 30);
    
    frontend_agent.add_thought(EntryType::ThoughtTrace {
        thoughts: vec![
            "User needs responsive design".to_string(),
            "Mobile-first approach is best".to_string(),
            "Use breakpoints at 768px and 1024px".to_string(),
        ],
        conclusion: Some("Implement adaptive layout system".to_string()),
    });
    
    frontend_agent.learn_from_experience(
        "Mobile-first design reduces rework".to_string(),
        WisdomCategory::PatternRecognition
    );
    
    // Backend agent works on API tasks
    backend_agent.add_thought(EntryType::Hypothesis {
        statement: "Caching frequently accessed data will improve performance".to_string(),
        confidence: 0.9,
    });
    
    backend_agent.gain_experience("API Design", 80);
    backend_agent.gain_experience("Database Optimization", 120);
    
    backend_agent.learn_from_experience(
        "Use pagination for large datasets".to_string(),
        WisdomCategory::TaskExecution
    );
    
    // Simulate more complex interaction
    println!("{} Agents collaborating on feature...\n", "🤝".yellow());
    
    let mut rng = rand::thread_rng();
    for i in 0..5 {
        println!("  {} Working on iteration {}...", "⚡".blue(), i + 1);
        
        // Frontend gains experience
        let ui_exp = rng.gen_range(20..60);
        frontend_agent.gain_experience("React Development", ui_exp);
        frontend_agent.gain_experience("UI Design", ui_exp / 2);
        
        // Backend gains experience
        let api_exp = rng.gen_range(30..70);
        backend_agent.gain_experience("API Design", api_exp);
        backend_agent.gain_experience("Database Optimization", api_exp / 3);
        
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    // Display final status
    println!("\n{}", "=== Final Agent Status ===".bold().green());
    frontend_agent.display_status();
    backend_agent.display_status();
    
    println!("\n{} Demo completed!", "✨".green());
    println!("{}", "Agents have developed unique personalities through experience.".italic());
}