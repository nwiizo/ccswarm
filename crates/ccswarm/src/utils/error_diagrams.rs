use std::fmt;

/// Error diagram for visual representation of errors
pub struct ErrorDiagram {
    pub title: String,
    pub description: String,
    pub steps: Vec<String>,
    pub solution: String,
}

impl ErrorDiagram {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: String::new(),
            steps: Vec::new(),
            solution: String::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn add_step(mut self, step: impl Into<String>) -> Self {
        self.steps.push(step.into());
        self
    }

    pub fn with_solution(mut self, solution: impl Into<String>) -> Self {
        self.solution = solution.into();
        self
    }
}

impl fmt::Display for ErrorDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╔══════════════════════════════════════╗")?;
        writeln!(f, "║ Error: {:^29} ║", self.title)?;
        writeln!(f, "╠══════════════════════════════════════╣")?;

        if !self.description.is_empty() {
            writeln!(f, "║ {:<36} ║", self.description)?;
            writeln!(f, "╠══════════════════════════════════════╣")?;
        }

        if !self.steps.is_empty() {
            writeln!(f, "║ Steps to reproduce:                  ║")?;
            for (i, step) in self.steps.iter().enumerate() {
                writeln!(f, "║  {}. {:<32} ║", i + 1, step)?;
            }
            writeln!(f, "╠══════════════════════════════════════╣")?;
        }

        if !self.solution.is_empty() {
            writeln!(f, "║ Solution: {:<26} ║", self.solution)?;
        }

        writeln!(f, "╚══════════════════════════════════════╝")?;
        Ok(())
    }
}

/// Show an error diagram
pub fn show_diagram(diagram: &ErrorDiagram) {
    eprintln!("{}", diagram);
}

/// Alias for ErrorDiagram for backwards compatibility
pub type ErrorDiagrams = ErrorDiagram;
