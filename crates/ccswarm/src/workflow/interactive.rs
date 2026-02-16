//! Interactive mode for Piece/Movement workflows.
//!
//! Provides four variants for user interaction before workflow execution:
//! 1. **Assistant** — AI asks clarifying questions before generating task instructions
//! 2. **Persona** — Conversation with the first movement's persona
//! 3. **Quiet** — Generates task instructions without asking questions (best-effort)
//! 4. **Passthrough** — Passes user input directly as task text

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use super::piece::Piece;

/// Interactive mode variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InteractiveMode {
    /// AI asks clarifying questions before generating task instructions
    #[default]
    Assistant,
    /// Conversation with the first movement's persona (uses its system prompt and tools)
    Persona,
    /// Generates task instructions without asking questions (best-effort)
    Quiet,
    /// Passes user input directly as task text without AI processing
    Passthrough,
}

impl std::fmt::Display for InteractiveMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assistant => write!(f, "assistant"),
            Self::Persona => write!(f, "persona"),
            Self::Quiet => write!(f, "quiet"),
            Self::Passthrough => write!(f, "passthrough"),
        }
    }
}

impl InteractiveMode {
    /// Description for display
    pub fn description(&self) -> &'static str {
        match self {
            Self::Assistant => "AI asks clarifying questions before generating task instructions",
            Self::Persona => {
                "Conversation with the first movement's persona (uses its system prompt and tools)"
            }
            Self::Quiet => "Generates task instructions without asking questions (best-effort)",
            Self::Passthrough => "Passes user input directly as task text without AI processing",
        }
    }

    /// All available modes
    pub fn all() -> Vec<Self> {
        vec![
            Self::Assistant,
            Self::Persona,
            Self::Quiet,
            Self::Passthrough,
        ]
    }
}

/// Configuration for an interactive session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveConfig {
    /// The interactive mode to use
    pub mode: InteractiveMode,
    /// Maximum number of clarification rounds (for Assistant mode)
    #[serde(default = "default_max_rounds")]
    pub max_clarification_rounds: u32,
    /// Whether to show piece selection menu
    #[serde(default = "default_true")]
    pub show_piece_selection: bool,
    /// Default piece to use (if set, skips piece selection)
    #[serde(default)]
    pub default_piece: Option<String>,
}

fn default_max_rounds() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            mode: InteractiveMode::default(),
            max_clarification_rounds: default_max_rounds(),
            show_piece_selection: true,
            default_piece: None,
        }
    }
}

/// State of an interactive session
#[derive(Debug, Clone)]
pub struct InteractiveSession {
    /// Selected piece (if any)
    pub selected_piece: Option<String>,
    /// Current mode
    pub mode: InteractiveMode,
    /// Collected user inputs
    pub user_inputs: Vec<String>,
    /// Generated clarification questions (Assistant mode)
    pub clarifications: Vec<Clarification>,
    /// Final task text to execute
    pub task_text: Option<String>,
    /// Session variables
    pub variables: HashMap<String, String>,
    /// Whether the session is ready for execution
    pub ready: bool,
}

/// A clarification question and its answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    /// The question asked
    pub question: String,
    /// User's answer (if provided)
    pub answer: Option<String>,
    /// Suggested options (if any)
    pub options: Vec<String>,
}

/// Result of processing user input in interactive mode
#[derive(Debug, Clone)]
pub enum InteractiveAction {
    /// Ask the user a question
    AskQuestion(Clarification),
    /// Display a message to the user
    ShowMessage(String),
    /// Ready to execute with the given task text
    Execute(String),
    /// User wants to exit
    Exit,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new(mode: InteractiveMode) -> Self {
        Self {
            selected_piece: None,
            mode,
            user_inputs: Vec::new(),
            clarifications: Vec::new(),
            task_text: None,
            variables: HashMap::new(),
            ready: false,
        }
    }

    /// Select a piece for this session
    pub fn select_piece(&mut self, piece_name: &str) {
        self.selected_piece = Some(piece_name.to_string());
        info!("Selected piece: {}", piece_name);
    }

    /// Process user input according to the current mode
    pub fn process_input(
        &mut self,
        input: &str,
        piece: Option<&Piece>,
    ) -> Result<InteractiveAction> {
        // Handle commands
        if let Some(action) = self.handle_command(input)? {
            return Ok(action);
        }

        self.user_inputs.push(input.to_string());

        match self.mode {
            InteractiveMode::Assistant => self.process_assistant(input, piece),
            InteractiveMode::Persona => self.process_persona(input, piece),
            InteractiveMode::Quiet => self.process_quiet(input),
            InteractiveMode::Passthrough => self.process_passthrough(input),
        }
    }

    /// Handle special commands (/go, /play, /mode, /quit)
    fn handle_command(&mut self, input: &str) -> Result<Option<InteractiveAction>> {
        let trimmed = input.trim();

        if trimmed == "/quit" || trimmed == "/exit" {
            return Ok(Some(InteractiveAction::Exit));
        }

        if trimmed == "/go" {
            if let Some(ref task) = self.task_text {
                self.ready = true;
                return Ok(Some(InteractiveAction::Execute(task.clone())));
            }
            // No task yet - build from collected inputs
            let combined = self.user_inputs.join("\n");
            self.task_text = Some(combined.clone());
            self.ready = true;
            return Ok(Some(InteractiveAction::Execute(combined)));
        }

        if let Some(task) = trimmed.strip_prefix("/play ") {
            self.task_text = Some(task.to_string());
            self.ready = true;
            return Ok(Some(InteractiveAction::Execute(task.to_string())));
        }

        if let Some(mode_str) = trimmed.strip_prefix("/mode ") {
            match mode_str.trim() {
                "assistant" => self.mode = InteractiveMode::Assistant,
                "persona" => self.mode = InteractiveMode::Persona,
                "quiet" => self.mode = InteractiveMode::Quiet,
                "passthrough" => self.mode = InteractiveMode::Passthrough,
                other => {
                    return Ok(Some(InteractiveAction::ShowMessage(format!(
                        "Unknown mode: '{}'. Available: assistant, persona, quiet, passthrough",
                        other
                    ))));
                }
            }
            return Ok(Some(InteractiveAction::ShowMessage(format!(
                "Switched to {} mode",
                self.mode
            ))));
        }

        Ok(None)
    }

    /// Assistant mode: ask clarifying questions
    fn process_assistant(
        &mut self,
        input: &str,
        _piece: Option<&Piece>,
    ) -> Result<InteractiveAction> {
        // If we have pending clarifications, record the answer
        if let Some(last_clarification) = self.clarifications.last_mut()
            && last_clarification.answer.is_none()
        {
            last_clarification.answer = Some(input.to_string());
            debug!("Recorded answer for: {}", last_clarification.question);
        }

        // Generate next clarification based on context
        let context = self.build_context();

        // After enough context, generate task
        if self.clarifications.len() >= 3 || self.has_sufficient_context() {
            let task = self.generate_task_from_context(&context);
            self.task_text = Some(task.clone());
            return Ok(InteractiveAction::ShowMessage(format!(
                "Generated task:\n{}\n\nType /go to execute or continue refining.",
                task
            )));
        }

        // Generate next question
        let question = self.generate_clarification(&context);
        let clarification = Clarification {
            question: question.clone(),
            answer: None,
            options: vec![],
        };
        self.clarifications.push(clarification.clone());

        Ok(InteractiveAction::AskQuestion(clarification))
    }

    /// Persona mode: converse with the first movement's persona
    fn process_persona(&mut self, input: &str, piece: Option<&Piece>) -> Result<InteractiveAction> {
        let persona_name = piece
            .and_then(|p| p.get_movement(&p.initial_movement))
            .and_then(|m| m.persona.as_deref())
            .unwrap_or("assistant");

        // Build response from persona perspective
        let response = format!(
            "[{}] I understand your request: \"{}\". \
             I'll incorporate this into the workflow. Type /go when ready to execute.",
            persona_name, input
        );

        // Accumulate as task text
        let current = self.task_text.clone().unwrap_or_default();
        let updated = if current.is_empty() {
            input.to_string()
        } else {
            format!("{}\n{}", current, input)
        };
        self.task_text = Some(updated);

        Ok(InteractiveAction::ShowMessage(response))
    }

    /// Quiet mode: directly generate task from input
    fn process_quiet(&mut self, input: &str) -> Result<InteractiveAction> {
        // Best-effort: take the input and enhance it into a task
        let enhanced = format!(
            "Task: {}\n\nPlease complete this task following best practices.",
            input
        );
        self.task_text = Some(enhanced.clone());
        self.ready = true;

        Ok(InteractiveAction::Execute(enhanced))
    }

    /// Passthrough mode: use input directly as task text
    fn process_passthrough(&mut self, input: &str) -> Result<InteractiveAction> {
        self.task_text = Some(input.to_string());
        self.ready = true;

        Ok(InteractiveAction::Execute(input.to_string()))
    }

    /// Build context summary from all inputs and clarifications
    fn build_context(&self) -> String {
        let mut parts = Vec::new();

        for input in &self.user_inputs {
            parts.push(format!("User: {}", input));
        }

        for clarification in &self.clarifications {
            if let Some(ref answer) = clarification.answer {
                parts.push(format!("Q: {}\nA: {}", clarification.question, answer));
            }
        }

        parts.join("\n")
    }

    /// Check if we have enough context to generate a task
    fn has_sufficient_context(&self) -> bool {
        // Heuristic: if total input length is substantial
        let total_len: usize = self.user_inputs.iter().map(|s| s.len()).sum();
        total_len > 200
    }

    /// Generate a clarification question based on context
    fn generate_clarification(&self, context: &str) -> String {
        // Generate questions based on what's missing
        let questions = [
            "What specific outcome are you looking for?",
            "Are there any constraints or requirements I should be aware of?",
            "Should I prioritize any particular aspect (speed, quality, security)?",
            "Are there any files or modules that should NOT be modified?",
            "What testing approach would you prefer?",
        ];

        let index = self.clarifications.len() % questions.len();
        let _context = context; // Used by future AI-powered question generation
        questions[index].to_string()
    }

    /// Generate a task description from collected context
    fn generate_task_from_context(&self, context: &str) -> String {
        let mut task_parts = Vec::new();
        task_parts.push("Task Summary:".to_string());

        // Include all user inputs as requirements
        for (i, input) in self.user_inputs.iter().enumerate() {
            if i == 0 {
                task_parts.push(format!("Primary goal: {}", input));
            } else {
                task_parts.push(format!("- {}", input));
            }
        }

        // Include clarification answers as constraints
        let answered: Vec<&Clarification> = self
            .clarifications
            .iter()
            .filter(|c| c.answer.is_some())
            .collect();
        if !answered.is_empty() {
            task_parts.push("\nAdditional context:".to_string());
            for c in answered {
                if let Some(ref answer) = c.answer {
                    task_parts.push(format!("- {} → {}", c.question, answer));
                }
            }
        }

        let _context = context; // Used by future AI-powered task generation
        task_parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_modes() {
        let modes = InteractiveMode::all();
        assert_eq!(modes.len(), 4);
        assert_eq!(InteractiveMode::default(), InteractiveMode::Assistant);
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(InteractiveMode::Assistant.to_string(), "assistant");
        assert_eq!(InteractiveMode::Persona.to_string(), "persona");
        assert_eq!(InteractiveMode::Quiet.to_string(), "quiet");
        assert_eq!(InteractiveMode::Passthrough.to_string(), "passthrough");
    }

    #[test]
    fn test_passthrough_mode() {
        let mut session = InteractiveSession::new(InteractiveMode::Passthrough);
        let result = session.process_input("Fix the login bug", None).unwrap();

        match result {
            InteractiveAction::Execute(task) => {
                assert_eq!(task, "Fix the login bug");
            }
            _ => panic!("Expected Execute action"),
        }
        assert!(session.ready);
    }

    #[test]
    fn test_quiet_mode() {
        let mut session = InteractiveSession::new(InteractiveMode::Quiet);
        let result = session.process_input("Add unit tests", None).unwrap();

        match result {
            InteractiveAction::Execute(task) => {
                assert!(task.contains("Add unit tests"));
            }
            _ => panic!("Expected Execute action"),
        }
        assert!(session.ready);
    }

    #[test]
    fn test_assistant_mode_asks_questions() {
        let mut session = InteractiveSession::new(InteractiveMode::Assistant);
        let result = session
            .process_input("Refactor the auth module", None)
            .unwrap();

        match result {
            InteractiveAction::AskQuestion(q) => {
                assert!(!q.question.is_empty());
            }
            _ => panic!("Expected AskQuestion action"),
        }
        assert!(!session.ready);
    }

    #[test]
    fn test_go_command() {
        let mut session = InteractiveSession::new(InteractiveMode::Assistant);
        session.process_input("Build a REST API", None).unwrap();

        let result = session.process_input("/go", None).unwrap();
        match result {
            InteractiveAction::Execute(task) => {
                assert!(task.contains("Build a REST API"));
            }
            _ => panic!("Expected Execute action"),
        }
        assert!(session.ready);
    }

    #[test]
    fn test_play_command() {
        let mut session = InteractiveSession::new(InteractiveMode::Assistant);
        let result = session
            .process_input("/play Create a login form", None)
            .unwrap();

        match result {
            InteractiveAction::Execute(task) => {
                assert_eq!(task, "Create a login form");
            }
            _ => panic!("Expected Execute action"),
        }
    }

    #[test]
    fn test_mode_switch() {
        let mut session = InteractiveSession::new(InteractiveMode::Assistant);
        let result = session.process_input("/mode quiet", None).unwrap();

        match result {
            InteractiveAction::ShowMessage(msg) => {
                assert!(msg.contains("quiet"));
            }
            _ => panic!("Expected ShowMessage action"),
        }
        assert_eq!(session.mode, InteractiveMode::Quiet);
    }

    #[test]
    fn test_quit_command() {
        let mut session = InteractiveSession::new(InteractiveMode::Assistant);
        let result = session.process_input("/quit", None).unwrap();

        match result {
            InteractiveAction::Exit => {}
            _ => panic!("Expected Exit action"),
        }
    }

    #[test]
    fn test_persona_mode() {
        let mut session = InteractiveSession::new(InteractiveMode::Persona);
        let result = session
            .process_input("Optimize the database queries", None)
            .unwrap();

        match result {
            InteractiveAction::ShowMessage(msg) => {
                assert!(msg.contains("Optimize the database queries"));
            }
            _ => panic!("Expected ShowMessage action"),
        }
        assert!(session.task_text.is_some());
    }

    #[test]
    fn test_interactive_config_default() {
        let config = InteractiveConfig::default();
        assert_eq!(config.mode, InteractiveMode::Assistant);
        assert_eq!(config.max_clarification_rounds, 5);
        assert!(config.show_piece_selection);
        assert!(config.default_piece.is_none());
    }
}
