//! AI Agent Memory System Demo
//! Implements four types of memory: Working, Episodic, Semantic, and Procedural

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

// Memory types
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    capacity: usize,
    items: VecDeque<MemoryItem>,
    attention_focus: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    episodes: Vec<Episode>,
    max_episodes: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticMemory {
    concepts: HashMap<String, Concept>,
    relationships: Vec<Relationship>,
}

#[derive(Debug, Clone)]
pub struct ProceduralMemory {
    procedures: HashMap<String, Procedure>,
    skill_levels: HashMap<String, f32>,
}

// Memory components
#[derive(Debug, Clone)]
pub struct MemoryItem {
    content: String,
    importance: f32,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct Episode {
    id: String,
    context: String,
    events: Vec<String>,
    outcome: String,
    emotion: EmotionalTone,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct Concept {
    name: String,
    definition: String,
    examples: Vec<String>,
    confidence: f32,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    concept_a: String,
    concept_b: String,
    relation_type: RelationType,
    strength: f32,
}

#[derive(Debug, Clone)]
pub struct Procedure {
    name: String,
    steps: Vec<String>,
    success_rate: f32,
    last_used: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmotionalTone {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

#[derive(Debug, Clone)]
pub enum RelationType {
    IsA,
    PartOf,
    UsedFor,
    Similar,
    Opposite,
}

// Memory-enabled Agent
pub struct MemoryAgent {
    id: String,
    working_memory: WorkingMemory,
    episodic_memory: EpisodicMemory,
    semantic_memory: SemanticMemory,
    procedural_memory: ProceduralMemory,
}

impl MemoryAgent {
    pub fn new(id: String) -> Self {
        Self {
            id,
            working_memory: WorkingMemory {
                capacity: 7, // Miller's magical number
                items: VecDeque::new(),
                attention_focus: None,
            },
            episodic_memory: EpisodicMemory {
                episodes: Vec::new(),
                max_episodes: 100,
            },
            semantic_memory: SemanticMemory {
                concepts: HashMap::new(),
                relationships: Vec::new(),
            },
            procedural_memory: ProceduralMemory {
                procedures: HashMap::new(),
                skill_levels: HashMap::new(),
            },
        }
    }

    // Working Memory operations
    pub fn focus_attention(&mut self, topic: &str) {
        self.working_memory.attention_focus = Some(topic.to_string());
        self.add_to_working_memory(format!("Focusing on: {}", topic), 1.0);
    }

    pub fn add_to_working_memory(&mut self, content: String, importance: f32) {
        let item = MemoryItem {
            content,
            importance,
            timestamp: Instant::now(),
        };

        self.working_memory.items.push_back(item);

        // Maintain capacity limit
        while self.working_memory.items.len() > self.working_memory.capacity {
            self.working_memory.items.pop_front();
        }
    }

    // Episodic Memory operations
    pub fn record_episode(&mut self, context: String, events: Vec<String>, outcome: String, emotion: EmotionalTone) {
        let episode = Episode {
            id: format!("ep_{}", self.episodic_memory.episodes.len()),
            context,
            events,
            outcome,
            emotion,
            timestamp: Instant::now(),
        };

        self.episodic_memory.episodes.push(episode);

        // Maintain episode limit
        if self.episodic_memory.episodes.len() > self.episodic_memory.max_episodes {
            self.episodic_memory.episodes.remove(0);
        }
    }

    pub fn recall_similar_episodes(&self, context: &str) -> Vec<&Episode> {
        self.episodic_memory.episodes.iter()
            .filter(|ep| ep.context.contains(context))
            .collect()
    }

    // Semantic Memory operations
    pub fn learn_concept(&mut self, name: String, definition: String) {
        let concept = Concept {
            name: name.clone(),
            definition,
            examples: Vec::new(),
            confidence: 0.5,
        };
        self.semantic_memory.concepts.insert(name, concept);
    }

    pub fn add_example(&mut self, concept_name: &str, example: String) {
        if let Some(concept) = self.semantic_memory.concepts.get_mut(concept_name) {
            concept.examples.push(example);
            concept.confidence = (concept.confidence + 0.1).min(1.0);
        }
    }

    pub fn relate_concepts(&mut self, concept_a: String, concept_b: String, relation: RelationType) {
        let relationship = Relationship {
            concept_a,
            concept_b,
            relation_type: relation,
            strength: 0.5,
        };
        self.semantic_memory.relationships.push(relationship);
    }

    // Procedural Memory operations
    pub fn learn_procedure(&mut self, name: String, steps: Vec<String>) {
        let procedure = Procedure {
            name: name.clone(),
            steps,
            success_rate: 0.5,
            last_used: None,
        };
        self.procedural_memory.procedures.insert(name.clone(), procedure);
        self.procedural_memory.skill_levels.insert(name, 0.1);
    }

    pub fn execute_procedure(&mut self, name: &str, success: bool) -> Option<Vec<String>> {
        if let Some(procedure) = self.procedural_memory.procedures.get_mut(name) {
            procedure.last_used = Some(Instant::now());
            
            // Update success rate with exponential moving average
            let alpha = 0.2;
            procedure.success_rate = alpha * (if success { 1.0 } else { 0.0 }) 
                + (1.0 - alpha) * procedure.success_rate;

            // Improve skill level
            if let Some(skill) = self.procedural_memory.skill_levels.get_mut(name) {
                *skill = (*skill + if success { 0.05 } else { 0.01 }).min(1.0);
            }

            return Some(procedure.steps.clone());
        }
        None
    }

    // Memory consolidation
    pub fn consolidate_memories(&mut self) {
        // Transfer important working memory to episodic
        let important_items: Vec<_> = self.working_memory.items.iter()
            .filter(|item| item.importance > 0.7)
            .cloned()
            .collect();

        if !important_items.is_empty() {
            let events: Vec<String> = important_items.iter()
                .map(|item| item.content.clone())
                .collect();
            
            let context = self.working_memory.attention_focus.clone()
                .unwrap_or_else(|| "General processing".to_string());
            
            self.record_episode(
                context,
                events,
                "Consolidated from working memory".to_string(),
                EmotionalTone::Neutral,
            );
        }

        // Extract patterns from episodes to semantic memory
        let new_concepts: Vec<(String, String)> = self.episodic_memory.episodes.iter()
            .flat_map(|episode| {
                episode.events.iter().filter_map(|event| {
                    if event.contains("learned") || event.contains("discovered") {
                        let parts: Vec<&str> = event.split_whitespace().collect();
                        if parts.len() > 2 {
                            let concept_name = parts[1..3].join(" ");
                            if !self.semantic_memory.concepts.contains_key(&concept_name) {
                                return Some((concept_name, format!("Extracted from episode: {}", episode.id)));
                            }
                        }
                    }
                    None
                })
            })
            .collect();
        
        // Now add the new concepts
        for (name, definition) in new_concepts {
            self.learn_concept(name, definition);
        }
    }

    // Display memory state
    pub fn display_memory_state(&self) {
        println!("\n=== Memory State for Agent {} ===", self.id);
        
        println!("\n📝 Working Memory (capacity: {}/{}):", 
            self.working_memory.items.len(), 
            self.working_memory.capacity
        );
        if let Some(focus) = &self.working_memory.attention_focus {
            println!("  🎯 Attention: {}", focus);
        }
        for item in &self.working_memory.items {
            println!("  • {} (importance: {:.1})", item.content, item.importance);
        }

        println!("\n📚 Episodic Memory ({} episodes):", self.episodic_memory.episodes.len());
        for episode in self.episodic_memory.episodes.iter().rev().take(3) {
            println!("  Episode {}: {} [{:?}]", episode.id, episode.context, episode.emotion);
            println!("    Events: {}", episode.events.join(" → "));
            println!("    Outcome: {}", episode.outcome);
        }

        println!("\n🧠 Semantic Memory ({} concepts):", self.semantic_memory.concepts.len());
        for (name, concept) in self.semantic_memory.concepts.iter().take(5) {
            println!("  • {}: {} (confidence: {:.1})", 
                name, 
                concept.definition, 
                concept.confidence
            );
            if !concept.examples.is_empty() {
                println!("    Examples: {}", concept.examples.join(", "));
            }
        }

        println!("\n⚙️ Procedural Memory ({} procedures):", self.procedural_memory.procedures.len());
        for (name, procedure) in &self.procedural_memory.procedures {
            let skill = self.procedural_memory.skill_levels.get(name).unwrap_or(&0.0);
            println!("  • {}: {:.0}% success rate, skill level: {:.1}", 
                name, 
                procedure.success_rate * 100.0,
                skill
            );
        }
    }
}

// Demonstration
fn main() {
    println!("🧠 AI Agent Memory System Demo");
    println!("==============================\n");

    let mut agent = MemoryAgent::new("Agent-Alpha".to_string());

    // Phase 1: Learning basics
    println!("📍 Phase 1: Learning Basic Concepts");
    
    agent.focus_attention("Web Development");
    agent.add_to_working_memory("Learning about React components".to_string(), 0.8);
    agent.add_to_working_memory("Components are reusable".to_string(), 0.9);
    agent.add_to_working_memory("Props pass data to components".to_string(), 0.9);
    
    // Learn concepts
    agent.learn_concept("React Component".to_string(), "A reusable piece of UI".to_string());
    agent.add_example("React Component", "Button component".to_string());
    agent.add_example("React Component", "Navigation bar".to_string());
    
    // Learn procedure
    agent.learn_procedure(
        "Create Component".to_string(),
        vec![
            "Define component function".to_string(),
            "Add props parameter".to_string(),
            "Return JSX".to_string(),
            "Export component".to_string(),
        ]
    );

    // Phase 2: Practical experience
    println!("\n📍 Phase 2: Gaining Experience");
    
    // Execute procedure successfully
    if let Some(steps) = agent.execute_procedure("Create Component", true) {
        println!("  ✅ Successfully executed procedure");
        agent.record_episode(
            "Creating login component".to_string(),
            steps,
            "Component created successfully".to_string(),
            EmotionalTone::Positive,
        );
    }

    // Add more working memory items
    agent.add_to_working_memory("useState manages component state".to_string(), 0.85);
    agent.add_to_working_memory("useEffect handles side effects".to_string(), 0.85);
    agent.add_to_working_memory("Custom hooks share logic".to_string(), 0.7);
    
    // Execute procedure with mixed results
    agent.execute_procedure("Create Component", false);
    agent.record_episode(
        "Creating complex form".to_string(),
        vec![
            "Attempted complex validation".to_string(),
            "Encountered type errors".to_string(),
            "Fixed after debugging".to_string(),
        ],
        "Learned importance of TypeScript".to_string(),
        EmotionalTone::Mixed,
    );

    // Phase 3: Advanced learning
    println!("\n📍 Phase 3: Advanced Concepts");
    
    agent.focus_attention("Performance Optimization");
    agent.learn_concept("Memoization".to_string(), "Caching computation results".to_string());
    agent.relate_concepts(
        "React Component".to_string(), 
        "Memoization".to_string(), 
        RelationType::UsedFor
    );
    
    // More procedures
    agent.learn_procedure(
        "Optimize Component".to_string(),
        vec![
            "Identify re-render causes".to_string(),
            "Apply React.memo".to_string(),
            "Use useMemo for expensive calculations".to_string(),
            "Test performance impact".to_string(),
        ]
    );
    
    agent.execute_procedure("Optimize Component", true);
    agent.execute_procedure("Create Component", true);
    agent.execute_procedure("Optimize Component", true);

    // Phase 4: Memory consolidation
    println!("\n📍 Phase 4: Memory Consolidation");
    agent.consolidate_memories();

    // Test recall
    println!("\n🔍 Testing Memory Recall");
    let similar = agent.recall_similar_episodes("component");
    println!("  Found {} similar episodes about components", similar.len());

    // Display final state
    agent.display_memory_state();

    println!("\n✨ Demo Complete!");
    println!("The agent has developed a rich memory system with:");
    println!("  • Short-term working memory for immediate tasks");
    println!("  • Episodic memory of past experiences");
    println!("  • Semantic understanding of concepts");
    println!("  • Procedural knowledge that improves with practice");
}