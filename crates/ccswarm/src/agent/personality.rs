use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent personality traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub traits: Vec<String>,
    pub style: String,
    pub approach: String,
    pub skills: HashMap<String, Skill>,
    pub working_style: String,
}

impl Personality {
    pub fn new(style: String) -> Self {
        Self {
            traits: Vec::new(),
            style,
            approach: String::from("collaborative"),
            skills: HashMap::new(),
            working_style: String::from("balanced"),
        }
    }

    pub fn add_trait(&mut self, trait_name: String) {
        self.traits.push(trait_name);
    }

    pub fn set_approach(&mut self, approach: String) {
        self.approach = approach;
    }

    pub fn describe_personality(&self) -> String {
        format!(
            "Style: {}, Approach: {}, Traits: {:?}, Skills: {:?}",
            self.style,
            self.approach,
            self.traits,
            self.skills.keys().collect::<Vec<_>>()
        )
    }

    pub fn composability_score(&self) -> f32 {
        // Calculate composability based on collaborative traits
        let collaborative_traits = ["cooperative", "flexible", "team-oriented"];
        let score = self.traits.iter()
            .filter(|t| collaborative_traits.contains(&t.as_str()))
            .count() as f32;
        (score / collaborative_traits.len() as f32).min(1.0)
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self::new(String::from("neutral"))
    }
}

/// Agent personality configuration
pub type AgentPersonality = Personality;

/// Personality traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalityTraits {
    Analytical,
    Creative,
    Methodical,
    Collaborative,
    Innovative,
}

/// Skill level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub level: SkillLevel,
    pub experience: f32,
    pub experience_points: u32,
    pub relevance_score: f32,
    pub language: String,
    pub framework: String,
}

impl Skill {
    pub fn new(name: String, level: SkillLevel) -> Self {
        Self {
            name: name.clone(),
            level,
            experience: 0.0,
            experience_points: 0,
            relevance_score: 0.5,
            language: String::new(),
            framework: String::new(),
        }
    }

    pub fn add_experience(&mut self, exp: f32) {
        self.experience += exp;
    }
}

/// Skill level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Task approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskApproach {
    Sequential,
    Parallel,
    Hybrid,
}

/// Working style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkingStyle {
    Independent,
    TeamPlayer,
    Leader,
    Support,
}