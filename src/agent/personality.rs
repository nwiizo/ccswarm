//! Agent personality system integrated with ccswarm
//! Provides personality traits, skills, and adaptive behavior for agents

//use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::identity::AgentRole;

/// Agent personality traits that influence behavior and decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    pub agent_id: String,
    pub skills: HashMap<String, Skill>,
    pub traits: PersonalityTraits,
    pub working_style: WorkingStyle,
    pub motto: Option<String>,
    pub experience_points: u32,
    pub adaptation_history: Vec<AdaptationRecord>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Individual skill with level and experience
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub level: SkillLevel,
    pub experience_points: u32,
    pub last_used: Option<DateTime<Utc>>,
    pub success_rate: f32,
    pub improvement_rate: f32,
}

/// Skill proficiency levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillLevel {
    Novice,      // 0-100 XP
    Beginner,    // 101-300 XP
    Intermediate, // 301-700 XP
    Advanced,    // 701-1500 XP
    Expert,      // 1501-3000 XP
    Master,      // 3000+ XP
}

/// Core personality traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub curiosity: f32,        // 0.0-1.0: Willingness to explore new approaches
    pub persistence: f32,      // 0.0-1.0: Tenacity in solving problems
    pub collaboration: f32,    // 0.0-1.0: Preference for team work
    pub risk_tolerance: f32,   // 0.0-1.0: Comfort with uncertainty
    pub attention_to_detail: f32, // 0.0-1.0: Focus on precision
    pub innovation: f32,       // 0.0-1.0: Tendency to try creative solutions
    pub communication_style: CommunicationStyle,
}

/// Communication style preferences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Direct,         // Clear, concise communication
    Collaborative,  // Consultative, team-oriented
    Analytical,     // Data-driven, detailed explanations
    Supportive,     // Encouraging, helpful tone
    Questioning,    // Inquisitive, explores options
}

impl CommunicationStyle {
    pub fn collaboration_factor(&self) -> f32 {
        match self {
            CommunicationStyle::Direct => 0.6,
            CommunicationStyle::Collaborative => 0.9,
            CommunicationStyle::Analytical => 0.7,
            CommunicationStyle::Supportive => 0.8,
            CommunicationStyle::Questioning => 0.7,
        }
    }
}

/// Working style and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingStyle {
    pub preferred_task_size: TaskSizePreference,
    pub work_rhythm: WorkRhythm,
    pub quality_vs_speed: f32, // 0.0 = speed focused, 1.0 = quality focused
    pub documentation_preference: DocumentationStyle,
    pub testing_approach: TestingApproach,
}

impl WorkingStyle {
    pub fn collaboration_factor(&self) -> f32 {
        let base_factor = match self.work_rhythm {
            WorkRhythm::Steady => 0.8,
            WorkRhythm::Sprint => 0.5,
            WorkRhythm::Iterative => 0.9,
            WorkRhythm::Exploratory => 0.7,
        };
        
        let size_factor = match self.preferred_task_size {
            TaskSizePreference::Small => 0.6,
            TaskSizePreference::Medium => 0.8,
            TaskSizePreference::Large => 0.7,
            TaskSizePreference::Adaptive => 0.9,
        };
        
        (base_factor + size_factor) / 2.0
    }
}

/// Task size preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSizePreference {
    Small,      // Prefers small, focused tasks
    Medium,     // Balanced approach
    Large,      // Prefers comprehensive tasks
    Adaptive,   // Adapts to context
}

/// Work rhythm patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkRhythm {
    Steady,     // Consistent pace
    Sprint,     // Intense bursts
    Iterative,  // Gradual refinement
    Exploratory, // Research-first approach
}

/// Documentation style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationStyle {
    Minimal,     // Essential documentation only
    Thorough,    // Comprehensive documentation
    UserFocused, // User-centric documentation
    Technical,   // Developer-focused details
}

/// Testing approach preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestingApproach {
    TestFirst,   // TDD approach
    TestLast,    // Implementation first
    Continuous,  // Testing throughout
    Pragmatic,   // Context-dependent
}

/// Record of personality adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRecord {
    pub timestamp: DateTime<Utc>,
    pub trigger: AdaptationTrigger,
    pub changes: Vec<PersonalityChange>,
    pub success_outcome: Option<bool>,
    pub learning_value: f32,
}

/// What triggered the adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptationTrigger {
    TaskSuccess,
    TaskFailure,
    FeedbackReceived,
    CollaborationExperience,
    SkillImprovement,
    EnvironmentChange,
}

/// Specific personality changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalityChange {
    SkillLevelUp { skill: String },
    TraitAdjustment { trait_name: String, old_value: f32, new_value: f32 },
    StyleChange { aspect: String, old_style: String, new_style: String },
    NewSkillAcquired { skill: String },
}

impl AgentPersonality {
    /// Create a new personality based on agent role
    pub fn new(agent_id: String, role: &AgentRole) -> Self {
        let (skills, traits, working_style, motto) = Self::create_role_based_personality(role);
        
        Self {
            agent_id,
            skills,
            traits,
            working_style,
            motto,
            experience_points: 0,
            adaptation_history: Vec::new(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    /// Create personality traits based on agent role
    fn create_role_based_personality(role: &AgentRole) -> (
        HashMap<String, Skill>,
        PersonalityTraits,
        WorkingStyle,
        Option<String>,
    ) {
        match role {
            AgentRole::Frontend { technologies, .. } => {
                let mut skills = HashMap::new();
                skills.insert("react".to_string(), Skill::new("React".to_string(), 150));
                skills.insert("typescript".to_string(), Skill::new("TypeScript".to_string(), 120));
                skills.insert("css".to_string(), Skill::new("CSS/Styling".to_string(), 200));
                skills.insert("ux_design".to_string(), Skill::new("UX Design".to_string(), 80));
                
                // Add role-specific technologies
                for tech in technologies {
                    skills.insert(
                        tech.to_lowercase(),
                        Skill::new(tech.clone(), 100),
                    );
                }

                (
                    skills,
                    PersonalityTraits {
                        curiosity: 0.8,
                        persistence: 0.7,
                        collaboration: 0.9,
                        risk_tolerance: 0.6,
                        attention_to_detail: 0.8,
                        innovation: 0.9,
                        communication_style: CommunicationStyle::Collaborative,
                    },
                    WorkingStyle {
                        preferred_task_size: TaskSizePreference::Medium,
                        work_rhythm: WorkRhythm::Iterative,
                        quality_vs_speed: 0.7,
                        documentation_preference: DocumentationStyle::UserFocused,
                        testing_approach: TestingApproach::Continuous,
                    },
                    Some("Beautiful, functional user experiences".to_string()),
                )
            }
            AgentRole::Backend { technologies, .. } => {
                let mut skills = HashMap::new();
                skills.insert("rust".to_string(), Skill::new("Rust".to_string(), 180));
                skills.insert("databases".to_string(), Skill::new("Database Design".to_string(), 160));
                skills.insert("api_design".to_string(), Skill::new("API Design".to_string(), 140));
                skills.insert("performance".to_string(), Skill::new("Performance Optimization".to_string(), 120));
                
                for tech in technologies {
                    skills.insert(
                        tech.to_lowercase(),
                        Skill::new(tech.clone(), 100),
                    );
                }

                (
                    skills,
                    PersonalityTraits {
                        curiosity: 0.7,
                        persistence: 0.9,
                        collaboration: 0.6,
                        risk_tolerance: 0.4,
                        attention_to_detail: 0.9,
                        innovation: 0.6,
                        communication_style: CommunicationStyle::Analytical,
                    },
                    WorkingStyle {
                        preferred_task_size: TaskSizePreference::Large,
                        work_rhythm: WorkRhythm::Steady,
                        quality_vs_speed: 0.8,
                        documentation_preference: DocumentationStyle::Technical,
                        testing_approach: TestingApproach::TestFirst,
                    },
                    Some("Robust, scalable systems".to_string()),
                )
            }
            AgentRole::DevOps { technologies, .. } => {
                let mut skills = HashMap::new();
                skills.insert("docker".to_string(), Skill::new("Docker".to_string(), 170));
                skills.insert("kubernetes".to_string(), Skill::new("Kubernetes".to_string(), 140));
                skills.insert("ci_cd".to_string(), Skill::new("CI/CD".to_string(), 160));
                skills.insert("monitoring".to_string(), Skill::new("Monitoring".to_string(), 130));
                
                for technology in technologies {
                    skills.insert(
                        technology.to_lowercase(),
                        Skill::new(technology.clone(), 100),
                    );
                }

                (
                    skills,
                    PersonalityTraits {
                        curiosity: 0.6,
                        persistence: 0.8,
                        collaboration: 0.7,
                        risk_tolerance: 0.3,
                        attention_to_detail: 0.95,
                        innovation: 0.5,
                        communication_style: CommunicationStyle::Direct,
                    },
                    WorkingStyle {
                        preferred_task_size: TaskSizePreference::Medium,
                        work_rhythm: WorkRhythm::Steady,
                        quality_vs_speed: 0.9,
                        documentation_preference: DocumentationStyle::Thorough,
                        testing_approach: TestingApproach::Continuous,
                    },
                    Some("Reliable, automated infrastructure".to_string()),
                )
            }
            AgentRole::QA { technologies, .. } => {
                let mut skills = HashMap::new();
                skills.insert("test_automation".to_string(), Skill::new("Test Automation".to_string(), 160));
                skills.insert("manual_testing".to_string(), Skill::new("Manual Testing".to_string(), 180));
                skills.insert("bug_analysis".to_string(), Skill::new("Bug Analysis".to_string(), 170));
                skills.insert("quality_assurance".to_string(), Skill::new("Quality Assurance".to_string(), 150));
                
                for technology in technologies {
                    skills.insert(
                        technology.to_lowercase(),
                        Skill::new(technology.clone(), 100),
                    );
                }

                (
                    skills,
                    PersonalityTraits {
                        curiosity: 0.8,
                        persistence: 0.9,
                        collaboration: 0.8,
                        risk_tolerance: 0.2,
                        attention_to_detail: 0.95,
                        innovation: 0.7,
                        communication_style: CommunicationStyle::Supportive,
                    },
                    WorkingStyle {
                        preferred_task_size: TaskSizePreference::Small,
                        work_rhythm: WorkRhythm::Iterative,
                        quality_vs_speed: 0.95,
                        documentation_preference: DocumentationStyle::Thorough,
                        testing_approach: TestingApproach::TestFirst,
                    },
                    Some("Zero defects, maximum quality".to_string()),
                )
            }
            AgentRole::Master { .. } => {
                let mut skills = HashMap::new();
                skills.insert("coordination".to_string(), Skill::new("Team Coordination".to_string(), 200));
                skills.insert("decision_making".to_string(), Skill::new("Decision Making".to_string(), 180));
                skills.insert("resource_management".to_string(), Skill::new("Resource Management".to_string(), 160));
                skills.insert("strategic_thinking".to_string(), Skill::new("Strategic Thinking".to_string(), 170));

                (
                    skills,
                    PersonalityTraits {
                        curiosity: 0.7,
                        persistence: 0.8,
                        collaboration: 0.95,
                        risk_tolerance: 0.5,
                        attention_to_detail: 0.7,
                        innovation: 0.8,
                        communication_style: CommunicationStyle::Direct,
                    },
                    WorkingStyle {
                        preferred_task_size: TaskSizePreference::Adaptive,
                        work_rhythm: WorkRhythm::Sprint,
                        quality_vs_speed: 0.6,
                        documentation_preference: DocumentationStyle::Minimal,
                        testing_approach: TestingApproach::Pragmatic,
                    },
                    Some("Orchestrating excellence through collaboration".to_string()),
                )
            }
        }
    }

    /// Update experience for a skill after task completion
    pub fn update_skill_experience(&mut self, skill_name: &str, experience_gained: u32, success: bool) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.experience_points += experience_gained;
            skill.last_used = Some(Utc::now());
            
            // Update success rate
            let success_factor = if success { 1.0 } else { 0.5 };
            skill.success_rate = (skill.success_rate * 0.9) + (success_factor * 0.1);
            
            // Check for level up
            let old_level = skill.level.clone();
            skill.level = SkillLevel::from_experience(skill.experience_points);
            
            if skill.level != old_level {
                self.adaptation_history.push(AdaptationRecord {
                    timestamp: Utc::now(),
                    trigger: AdaptationTrigger::SkillImprovement,
                    changes: vec![PersonalityChange::SkillLevelUp { 
                        skill: skill_name.to_string() 
                    }],
                    success_outcome: Some(true),
                    learning_value: 0.8,
                });
            }
        }
        
        self.experience_points += experience_gained;
        self.last_updated = Utc::now();
    }

    /// Adapt personality based on task outcome
    pub fn adapt_from_task_outcome(&mut self, task_type: &str, success: bool, feedback: Option<&str>) {
        let trigger = if success {
            AdaptationTrigger::TaskSuccess
        } else {
            AdaptationTrigger::TaskFailure
        };

        let mut changes = Vec::new();

        // Adjust traits based on outcome
        if success {
            // Reinforce successful patterns
            match task_type {
                "complex" | "large" => {
                    if self.traits.persistence < 0.9 {
                        let old_value = self.traits.persistence;
                        self.traits.persistence = (self.traits.persistence + 0.05).min(1.0);
                        changes.push(PersonalityChange::TraitAdjustment {
                            trait_name: "persistence".to_string(),
                            old_value,
                            new_value: self.traits.persistence,
                        });
                    }
                }
                "collaborative" => {
                    if self.traits.collaboration < 0.9 {
                        let old_value = self.traits.collaboration;
                        self.traits.collaboration = (self.traits.collaboration + 0.03).min(1.0);
                        changes.push(PersonalityChange::TraitAdjustment {
                            trait_name: "collaboration".to_string(),
                            old_value,
                            new_value: self.traits.collaboration,
                        });
                    }
                }
                _ => {}
            }
        } else {
            // Learn from failures
            if let Some(feedback_text) = feedback {
                if feedback_text.contains("attention") || feedback_text.contains("detail") {
                    let old_value = self.traits.attention_to_detail;
                    self.traits.attention_to_detail = (self.traits.attention_to_detail + 0.1).min(1.0);
                    changes.push(PersonalityChange::TraitAdjustment {
                        trait_name: "attention_to_detail".to_string(),
                        old_value,
                        new_value: self.traits.attention_to_detail,
                    });
                }
            }
        }

        if !changes.is_empty() {
            self.adaptation_history.push(AdaptationRecord {
                timestamp: Utc::now(),
                trigger,
                changes,
                success_outcome: Some(success),
                learning_value: if success { 0.6 } else { 0.8 }, // Learn more from failures
            });
        }

        self.last_updated = Utc::now();
    }

    /// Get personality-influenced task approach
    pub fn get_task_approach(&self, task_description: &str) -> TaskApproach {
        let complexity_score = Self::estimate_task_complexity(task_description);
        
        TaskApproach {
            preferred_method: self.select_method(complexity_score),
            estimated_effort: self.estimate_effort(complexity_score),
            quality_focus: self.working_style.quality_vs_speed,
            collaboration_likelihood: self.traits.collaboration,
            innovation_factor: self.traits.innovation,
            risk_assessment: self.assess_risk(task_description),
        }
    }

    /// Estimate task complexity based on description
    fn estimate_task_complexity(description: &str) -> f32 {
        let complexity_indicators = [
            ("refactor", 0.8),
            ("implement", 0.6),
            ("fix", 0.4),
            ("optimize", 0.7),
            ("design", 0.8),
            ("integrate", 0.9),
            ("migrate", 0.9),
            ("test", 0.3),
            ("document", 0.2),
        ];

        let lower_desc = description.to_lowercase();
        let mut max_complexity: f32 = 0.2; // Base complexity

        for (indicator, complexity) in &complexity_indicators {
            if lower_desc.contains(indicator) {
                max_complexity = max_complexity.max(*complexity);
            }
        }

        // Length also affects complexity
        let length_factor = (description.len() as f32 / 100.0).min(0.3);
        (max_complexity + length_factor).min(1.0)
    }

    fn select_method(&self, complexity: f32) -> String {
        match (&self.working_style.work_rhythm, complexity) {
            (WorkRhythm::Sprint, c) if c < 0.5 => "Quick implementation".to_string(),
            (WorkRhythm::Iterative, _) => "Iterative development".to_string(),
            (WorkRhythm::Exploratory, c) if c > 0.7 => "Research-first approach".to_string(),
            (WorkRhythm::Steady, _) => "Systematic implementation".to_string(),
            _ => "Adaptive approach".to_string(),
        }
    }

    fn estimate_effort(&self, complexity: f32) -> f32 {
        let base_effort = complexity;
        let persistence_factor = 1.0 - (self.traits.persistence * 0.2);
        let detail_factor = 1.0 + (self.traits.attention_to_detail * 0.3);
        
        base_effort * persistence_factor * detail_factor
    }

    fn assess_risk(&self, description: &str) -> f32 {
        let risk_keywords = ["breaking", "major", "critical", "legacy", "production"];
        let has_risk_keywords = risk_keywords.iter()
            .any(|&keyword| description.to_lowercase().contains(keyword));
        
        let base_risk = if has_risk_keywords { 0.7 } else { 0.3 };
        let tolerance_adjustment = self.traits.risk_tolerance * 0.4;
        
        (base_risk - tolerance_adjustment).clamp(0.1, 1.0)
    }

    /// Get skills relevant to a task
    pub fn get_relevant_skills(&self, task_description: &str) -> Vec<&Skill> {
        let lower_desc = task_description.to_lowercase();
        
        self.skills.values()
            .filter(|skill| {
                let skill_name_lower = skill.name.to_lowercase();
                lower_desc.contains(&skill_name_lower) ||
                skill_name_lower.split_whitespace().any(|word| lower_desc.contains(word))
            })
            .collect()
    }

    /// Generate personality summary for reporting
    pub fn generate_summary(&self) -> PersonalitySummary {
        PersonalitySummary {
            agent_id: self.agent_id.clone(),
            dominant_traits: self.get_dominant_traits(),
            skill_levels: self.skills.iter()
                .map(|(name, skill)| (name.clone(), skill.level.clone()))
                .collect(),
            adaptations_count: self.adaptation_history.len(),
            last_adaptation: self.adaptation_history.last()
                .map(|record| record.timestamp),
            motto: self.motto.clone(),
            experience_points: self.experience_points,
        }
    }

    fn get_dominant_traits(&self) -> Vec<String> {
        let mut traits = vec![
            ("curiosity", self.traits.curiosity),
            ("persistence", self.traits.persistence),
            ("collaboration", self.traits.collaboration),
            ("risk_tolerance", self.traits.risk_tolerance),
            ("attention_to_detail", self.traits.attention_to_detail),
            ("innovation", self.traits.innovation),
        ];
        
        traits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        traits.into_iter()
            .take(3)
            .map(|(name, _)| name.to_string())
            .collect()
    }
    
    /// Generate a description of the agent's personality
    pub fn describe_personality(&self) -> String {
        let skill_count = self.skills.len();
        let avg_skill_level: f32 = self.skills.values()
            .map(|s| s.experience_points as f32)
            .sum::<f32>() / skill_count as f32;
        
        format!(
            "Agent with {} skills (avg. {} XP), {:?} communication style, {:?} work rhythm",
            skill_count,
            avg_skill_level as u32,
            self.traits.communication_style,
            self.working_style.work_rhythm
        )
    }
    
    /// Calculate composability score (how well this agent works with others)
    pub fn composability_score(&self) -> f32 {
        let base_score = (self.traits.collaboration + 
                         self.traits.communication_style.collaboration_factor() +
                         self.working_style.collaboration_factor()) / 3.0;
        
        // Adjust based on experience
        let experience_factor = (self.experience_points as f32 / 1000.0).min(1.0);
        base_score * (0.5 + 0.5 * experience_factor)
    }
}

/// Task approach influenced by personality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskApproach {
    pub preferred_method: String,
    pub estimated_effort: f32,
    pub quality_focus: f32,
    pub collaboration_likelihood: f32,
    pub innovation_factor: f32,
    pub risk_assessment: f32,
}

/// Personality summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalitySummary {
    pub agent_id: String,
    pub dominant_traits: Vec<String>,
    pub skill_levels: HashMap<String, SkillLevel>,
    pub adaptations_count: usize,
    pub last_adaptation: Option<DateTime<Utc>>,
    pub motto: Option<String>,
    pub experience_points: u32,
}

impl Skill {
    pub fn new(name: String, initial_xp: u32) -> Self {
        Self {
            level: SkillLevel::from_experience(initial_xp),
            name,
            experience_points: initial_xp,
            last_used: None,
            success_rate: 0.7, // Start with moderate success rate
            improvement_rate: 1.0,
        }
    }
    
    /// Add experience points to the skill
    pub fn add_experience(&mut self, points: u32) {
        self.experience_points += points;
        self.level = SkillLevel::from_experience(self.experience_points);
        self.last_used = Some(Utc::now());
    }
}

impl SkillLevel {
    pub fn from_experience(xp: u32) -> Self {
        match xp {
            0..=100 => SkillLevel::Novice,
            101..=300 => SkillLevel::Beginner,
            301..=700 => SkillLevel::Intermediate,
            701..=1500 => SkillLevel::Advanced,
            1501..=3000 => SkillLevel::Expert,
            _ => SkillLevel::Master,
        }
    }

    pub fn to_multiplier(&self) -> f32 {
        match self {
            SkillLevel::Novice => 0.5,
            SkillLevel::Beginner => 0.7,
            SkillLevel::Intermediate => 1.0,
            SkillLevel::Advanced => 1.3,
            SkillLevel::Expert => 1.6,
            SkillLevel::Master => 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{default_frontend_role, default_backend_role};

    #[test]
    fn test_personality_creation() {
        let frontend_role = default_frontend_role();
        let personality = AgentPersonality::new("test-agent".to_string(), &frontend_role);
        
        assert_eq!(personality.agent_id, "test-agent");
        assert!(!personality.skills.is_empty());
        assert!(personality.skills.contains_key("react"));
        assert_eq!(personality.traits.communication_style, CommunicationStyle::Collaborative);
    }

    #[test]
    fn test_skill_experience_update() {
        let mut personality = AgentPersonality::new("test-agent".to_string(), &default_frontend_role());
        let initial_xp = personality.skills["react"].experience_points;
        
        personality.update_skill_experience("react", 50, true);
        
        assert_eq!(personality.skills["react"].experience_points, initial_xp + 50);
        assert!(personality.skills["react"].success_rate > 0.7);
    }

    #[test]
    fn test_task_approach() {
        let personality = AgentPersonality::new("test-agent".to_string(), &default_backend_role());
        let approach = personality.get_task_approach("Implement complex API with database integration");
        
        assert!(approach.estimated_effort > 0.5);
        assert!(approach.quality_focus > 0.7); // Backend focuses on quality
    }

    #[test]
    fn test_relevant_skills() {
        let personality = AgentPersonality::new("test-agent".to_string(), &default_frontend_role());
        let skills = personality.get_relevant_skills("Create React component with TypeScript");
        
        assert!(skills.iter().any(|s| s.name.contains("React")));
        assert!(skills.iter().any(|s| s.name.contains("TypeScript")));
    }
}