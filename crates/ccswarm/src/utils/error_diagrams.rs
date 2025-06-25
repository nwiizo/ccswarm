//! Visual error diagrams and ASCII art for better error understanding

use colored::Colorize;

/// Visual error diagrams for common scenarios
pub struct ErrorDiagrams;

impl ErrorDiagrams {
    /// Network connectivity diagram
    pub fn network_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────┐       ┌──────────────┐       ┌─────────────┐
    │   {} │  {}  │  {} │  {}  │   {} │
    │  Computer   │──────▶│   Network    │──────▶│     API     │
    └─────────────┘       └──────────────┘       └─────────────┘
           │                      │                       │
           │                      │                       │
           ▼                      ▼                       ▼
      {}              {}            {}
      API Key            Connection           Server Status
                          Issue?                 Down?
    "#,
            "Network Connection Error:".bright_red().bold(),
            "Your".bright_white(),
            "❌".red(),
            "Internet".bright_white(),
            "❌".red(),
            "Claude".bright_white(),
            "✓ Check".green(),
            "✗ Failed".red(),
            "? Unknown".yellow()
        )
    }

    /// Session lifecycle diagram
    pub fn session_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────────────────────────────────────────────────┐
    │                    Session Lifecycle                     │
    └─────────────────────────────────────────────────────────┘
    
    {} ──▶ {} ──▶ {} ──▶ {}
         │         │         │         │
         ▼         ▼         ▼         ▼
      {}    {}    {}    {}
      
    Your session might be in any of these states.
    Use 'ccswarm session list' to check current sessions.
    "#,
            "Session State Diagram:".bright_cyan().bold(),
            "Created".green(),
            "Active".bright_green(),
            "Idle".yellow(),
            "Terminated".red(),
            "New".dimmed(),
            "Working".dimmed(),
            "Waiting".dimmed(),
            "Closed".dimmed()
        )
    }

    /// Git worktree visualization
    pub fn git_worktree_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────────────────────────────────────────────────┐
    │                     Main Repository                      │
    │  {} ◄──────────────────────────┐               │
    └─────────────────┬───────────────┼───────────────────────┘
                      │               │
         ┌────────────▼───┐      ┌────▼────────────┐
         │ {} │      │ {} │
         │   frontend     │      │    backend      │
         │  (worktree)    │      │  (worktree)     │
         └────────────────┘      └─────────────────┘
              {}                    {}
    
    Each agent works in its own isolated worktree.
    Conflicts occur when a branch is already checked out.
    "#,
            "Git Worktree Structure:".bright_yellow().bold(),
            "master branch".bright_white(),
            "Agent: Frontend".cyan(),
            "Agent: Backend".magenta(),
            "✓ Active".green(),
            "✗ Conflict".red()
        )
    }

    /// Permission hierarchy
    pub fn permission_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────────────────────────────────────────────────┐
    │                  File Permissions                        │
    └─────────────────────────────────────────────────────────┘
    
    Owner    Group    Others
    ┌─────┐  ┌─────┐  ┌─────┐
    │ {} │  │ {} │  │ {} │    {} = Read
    │ {} │  │ {} │  │ {} │    {} = Write  
    │ {} │  │ {} │  │ {} │    {} = Execute
    └─────┘  └─────┘  └─────┘
      {}      {}      {}
    
    Current user needs appropriate permissions to access files.
    "#,
            "Permission Structure:".bright_red().bold(),
            "r".green(),
            "r".green(),
            "r".dimmed(),
            "r".bright_white(),
            "w".green(),
            "w".yellow(),
            "-".dimmed(),
            "w".bright_white(),
            "x".green(),
            "-".dimmed(),
            "-".dimmed(),
            "x".bright_white(),
            "You".bright_green(),
            "Team".yellow(),
            "Others".dimmed()
        )
    }

    /// Configuration file structure
    pub fn config_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────────────────────────────────────────────────┐
    │                  ccswarm.json                            │
    ├─────────────────────────────────────────────────────────┤
    │ {{                                                       │
    │   "project": {{                                          │
    │     "name": "{}",                               │
    │     "description": "{}"                 │
    │   }},                                                    │
    │   "agents": [                                            │
    │     {{                                                   │
    │       "name": "{}",                            │
    │       "role": "{}",                            │
    │       "provider": "{}"                      │
    │     }}                                                   │
    │   ]                                                      │
    │ }}                                                       │
    └─────────────────────────────────────────────────────────┘
    
    This configuration file is required to run ccswarm.
    Use 'ccswarm init' to create it automatically.
    "#,
            "Configuration Structure:".bright_cyan().bold(),
            "MyProject".bright_green(),
            "AI orchestration project".dimmed(),
            "frontend".bright_yellow(),
            "Frontend".bright_white(),
            "claude_code".bright_magenta()
        )
    }

    /// Task flow diagram
    pub fn task_error() -> String {
        format!(
            r#"
    {}
    ┌─────────┐      ┌──────────┐      ┌─────────┐      ┌──────────┐
    │  {} │ ──▶ │ {} │ ──▶ │ {} │ ──▶ │ {} │
    └─────────┘      └──────────┘      └─────────┘      └──────────┘
         │                 │                 │                 │
         ▼                 ▼                 ▼                 ▼
    {}         {}      {}       {}
    
    Task Format: "{} [{}] [{}]"
    
    Examples:
    • "Create user authentication system [high] [feature]"
    • "Fix login bug [urgent] [bugfix]"
    • "Add unit tests for API [medium] [test]"
    "#,
            "Task Processing Flow:".bright_yellow().bold(),
            "Input".bright_white(),
            "Parse".bright_cyan(),
            "Assign".bright_green(),
            "Execute".bright_magenta(),
            "Description".dimmed(),
            "Priority".dimmed(),
            "Agent Role".dimmed(),
            "Result".dimmed(),
            "description".bright_white(),
            "priority".yellow(),
            "type".cyan()
        )
    }

    /// API key flow
    pub fn api_key_error() -> String {
        format!(
            r#"
    {}
    ┌─────────────────────────────────────────────────────────┐
    │                  API Key Setup                           │
    └─────────────────────────────────────────────────────────┘
    
    1. {} ──────▶ https://console.anthropic.com
       └─▶ Sign up / Log in
    
    2. {} ────▶ API Keys section
       └─▶ Create new key
    
    3. {} ──▶ Copy key: sk-ant-api03-...
       └─▶ Keep it secret!
    
    4. {} ─▶ Export in terminal:
       {}
    
    5. {} ──▶ Add to .env file:
       {}
    "#,
            "API Key Configuration:".bright_red().bold(),
            "Visit".bright_white(),
            "Navigate".bright_white(),
            "Generate".bright_white(),
            "Configure".bright_white(),
            "export ANTHROPIC_API_KEY=your-key-here"
                .bright_green()
                .on_black(),
            "Persist".bright_white(),
            "echo 'ANTHROPIC_API_KEY=your-key' >> .env"
                .bright_green()
                .on_black()
        )
    }

    /// Agent communication flow
    pub fn agent_error() -> String {
        format!(
            r#"
    {}
    ┌───────────────┐
    │ {} │
    │  Orchestrator │
    └───────┬───────┘
            │ {}
    ┌───────▼────────────────────────────────────┐
    │          Task Assignment                   │
    └────┬──────────┬──────────┬────────────────┘
         │          │          │
    ┌────▼───┐ ┌───▼────┐ ┌───▼────┐
    │{}│ │{}│ │{} │  {} 
    │Agent   │ │Agent   │ │Agent   │  Agent Busy!
    └────────┘ └────────┘ └────────┘
       {}        {}        {}
    
    Agents can only handle one task at a time.
    Use 'ccswarm agent status' to check availability.
    "#,
            "Agent Communication:".bright_cyan().bold(),
            "Master Claude".bright_yellow(),
            "Delegates tasks".dimmed(),
            "Frontend".cyan(),
            "Backend".magenta(),
            "DevOps".green(),
            "←── ❌".red(),
            "✓ Ready".green(),
            "✓ Ready".green(),
            "⚡ Busy".yellow()
        )
    }
}

/// Helper to display a diagram with proper formatting
pub fn show_diagram(diagram: String) {
    println!();
    for line in diagram.lines() {
        println!("{}", line);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagrams_render() {
        // Just ensure they don't panic
        let _ = ErrorDiagrams::network_error();
        let _ = ErrorDiagrams::session_error();
        let _ = ErrorDiagrams::git_worktree_error();
        let _ = ErrorDiagrams::permission_error();
        let _ = ErrorDiagrams::config_error();
        let _ = ErrorDiagrams::task_error();
        let _ = ErrorDiagrams::api_key_error();
        let _ = ErrorDiagrams::agent_error();
    }
}
