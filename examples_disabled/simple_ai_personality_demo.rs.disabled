//! Simple demonstration of AI agent personality systems
//! This runs without external dependencies

use std::collections::HashMap;

// Skill system
#[derive(Debug, Clone, PartialEq)]
pub enum SkillLevel {
    Novice,
    Competent,
    Proficient,
    Expert,
    Master,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub category: SkillCategory,
    pub level: SkillLevel,
    pub experience_points: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillCategory {
    Technical,
    Creative,
    Analytical,
    Communication,
    Leadership,
}

// Personality system
#[derive(Debug, Clone)]
pub struct Personality {
    pub agent_id: String,
    pub skills: HashMap<String, Skill>,
    pub working_style: WorkingStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkingStyle {
    Methodical,
    Creative,
    Analytical,
    Collaborative,
    Balanced,
}

// Whiteboard entry
#[derive(Debug, Clone)]
pub enum ThoughtType {
    Hypothesis { statement: String, confidence: f32 },
    Calculation { expression: String, result: String },
    ThoughtChain { steps: Vec<String> },
}

// Practical wisdom
#[derive(Debug, Clone)]
pub struct Wisdom {
    pub insight: String,
    pub confidence: f32,
}

// Simple agent
pub struct Agent {
    pub personality: Personality,
    pub thoughts: Vec<ThoughtType>,
    pub wisdom: Vec<Wisdom>,
}

impl Agent {
    pub fn new(id: String, role: &str) -> Self {
        let mut skills = HashMap::new();
        
        match role {
            "frontend" => {
                skills.insert("React".to_string(), Skill {
                    name: "React".to_string(),
                    category: SkillCategory::Technical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
                skills.insert("Design".to_string(), Skill {
                    name: "Design".to_string(),
                    category: SkillCategory::Creative,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
            }
            "backend" => {
                skills.insert("API".to_string(), Skill {
                    name: "API".to_string(),
                    category: SkillCategory::Technical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
                skills.insert("Database".to_string(), Skill {
                    name: "Database".to_string(),
                    category: SkillCategory::Analytical,
                    level: SkillLevel::Novice,
                    experience_points: 0,
                });
            }
            _ => {}
        }
        
        Self {
            personality: Personality {
                agent_id: id,
                skills,
                working_style: WorkingStyle::Balanced,
            },
            thoughts: Vec::new(),
            wisdom: Vec::new(),
        }
    }
    
    pub fn gain_experience(&mut self, skill: &str, points: u32) {
        if let Some(s) = self.personality.skills.get_mut(skill) {
            s.experience_points += points;
            
            let new_level = match s.experience_points {
                0..=99 => SkillLevel::Novice,
                100..=299 => SkillLevel::Competent,
                300..=599 => SkillLevel::Proficient,
                600..=999 => SkillLevel::Expert,
                _ => SkillLevel::Master,
            };
            
            if s.level != new_level {
                println!("🎉 {} leveled up to {:?}!", skill, new_level);
                s.level = new_level;
            }
        }
    }
    
    pub fn think(&mut self, thought: ThoughtType) {
        self.thoughts.push(thought);
    }
    
    pub fn learn(&mut self, insight: String) {
        self.wisdom.push(Wisdom {
            insight,
            confidence: 0.8,
        });
    }
    
    pub fn display(&self) {
        println!("\n=== Agent: {} ===", self.personality.agent_id);
        println!("Style: {:?}", self.personality.working_style);
        
        println!("\nSkills:");
        for (name, skill) in &self.personality.skills {
            println!("  {} [{:?}]: Level {:?} ({}xp)", 
                name, 
                skill.category,
                skill.level, 
                skill.experience_points
            );
        }
        
        println!("\nRecent Thoughts:");
        for thought in self.thoughts.iter().rev().take(3) {
            match thought {
                ThoughtType::Hypothesis { statement, confidence } => {
                    println!("  💭 Hypothesis: {} ({}% confident)", 
                        statement, 
                        (confidence * 100.0) as u32
                    );
                }
                ThoughtType::Calculation { expression, result } => {
                    println!("  🧮 Calculated: {} = {}", expression, result);
                }
                ThoughtType::ThoughtChain { steps } => {
                    println!("  🔗 Thought chain: {}", steps.join(" → "));
                }
            }
        }
        
        println!("\nWisdom:");
        for w in &self.wisdom {
            println!("  💡 {}", w.insight);
        }
    }
}

fn main() {
    println!("🤖 AI Agent Personality Demo");
    println!("============================\n");
    
    // Create agents
    let mut frontend = Agent::new("agent-001".to_string(), "frontend");
    let mut backend = Agent::new("agent-002".to_string(), "backend");
    
    // Simulate activities
    println!("📍 Phase 1: Initial Tasks");
    
    // Frontend agent activities
    frontend.think(ThoughtType::Hypothesis {
        statement: "CSS Grid will improve layout flexibility".to_string(),
        confidence: 0.8,
    });
    frontend.gain_experience("React", 50);
    frontend.gain_experience("Design", 30);
    
    // Backend agent activities
    backend.think(ThoughtType::Calculation {
        expression: "Cache hit rate".to_string(),
        result: "85%".to_string(),
    });
    backend.gain_experience("API", 80);
    backend.gain_experience("Database", 60);
    
    // Phase 2: Learning
    println!("\n📍 Phase 2: Learning from Experience");
    
    frontend.think(ThoughtType::ThoughtChain {
        steps: vec![
            "User needs responsive design".to_string(),
            "Mobile-first is best practice".to_string(),
            "Use breakpoints at 768px and 1024px".to_string(),
        ],
    });
    frontend.learn("Mobile-first design reduces rework".to_string());
    
    backend.think(ThoughtType::Hypothesis {
        statement: "Connection pooling will reduce latency".to_string(),
        confidence: 0.9,
    });
    backend.learn("Always use pagination for large datasets".to_string());
    
    // Phase 3: Collaboration
    println!("\n📍 Phase 3: Collaborative Project");
    
    // Simulate multiple iterations
    for i in 1..=5 {
        println!("  Working on iteration {}...", i);
        
        // Frontend gains experience
        frontend.gain_experience("React", 40 + i * 10);
        frontend.gain_experience("Design", 20 + i * 5);
        
        // Backend gains experience  
        backend.gain_experience("API", 50 + i * 10);
        backend.gain_experience("Database", 30 + i * 5);
    }
    
    // Add collaborative insights
    frontend.think(ThoughtType::ThoughtChain {
        steps: vec![
            "API contract defined".to_string(),
            "UI components mapped to endpoints".to_string(),
            "Error handling implemented".to_string(),
        ],
    });
    frontend.learn("Clear API contracts speed up frontend development".to_string());
    
    backend.think(ThoughtType::Calculation {
        expression: "Average response time".to_string(),
        result: "45ms".to_string(),
    });
    backend.learn("GraphQL reduces over-fetching".to_string());
    
    // Display final status
    println!("\n🏁 Final Agent Status");
    frontend.display();
    backend.display();
    
    println!("\n✨ Demo Complete!");
    println!("The agents have developed unique skills and insights through experience.");
}