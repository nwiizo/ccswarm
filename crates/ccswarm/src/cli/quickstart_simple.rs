//! Simplified quickstart implementation without complex progress tracking

use crate::config::{AgentConfig, CcswarmConfig, ClaudeConfig, MasterClaudeConfig, ProjectConfig};
use crate::utils::user_error::CommonErrors;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::io::Write;

pub async fn handle_quickstart_simple(
    repo_path: &std::path::Path,
    name: Option<&str>,
    no_prompt: bool,
    all_agents: bool,
    with_tests: bool,
) -> Result<()> {
    println!();
    println!("{}", "🚀 ccswarm Quickstart".bright_cyan().bold());
    println!("{}", "===================".bright_cyan());
    println!(
        "{}",
        "Let's get you up and running with ccswarm in one command!".bright_white()
    );
    println!();

    // Step 1: Check system requirements
    println!("{}  Checking system requirements...", "[1/7]".bright_cyan());

    // Check Git
    if !crate::git::shell::ShellWorktreeManager::is_git_available() {
        println!("{}  Git not found", "❌".bright_red());
        CommonErrors::git_not_initialized().display();
        return Err(anyhow!("Git is required for ccswarm"));
    }
    println!("{}  Git: OK", "✅".bright_green());

    // Check API key
    let has_api_key = std::env::var("ANTHROPIC_API_KEY").is_ok();
    if !has_api_key && !no_prompt {
        println!("{}  API key not set", "⚠️".yellow());
        println!();
        println!("{}", "⚠️  ANTHROPIC_API_KEY not found".yellow());
        println!("You can:");
        println!("  1. Set it now (will not be saved to disk)");
        println!("  2. Continue without it (limited functionality)");
        println!();
        print!("Enter your choice [1/2]: ");
        std::io::stdout().flush()?;

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;

        if choice.trim() == "1" {
            print!("Enter your API key: ");
            std::io::stdout().flush()?;
            let mut key = String::new();
            std::io::stdin().read_line(&mut key)?;
            std::env::set_var("ANTHROPIC_API_KEY", key.trim());
        }
    }
    println!();

    // Step 2: Determine project name
    println!("{}  Setting up project...", "[2/7]".bright_cyan());

    let project_name = if let Some(n) = name {
        n.to_string()
    } else if no_prompt {
        // Infer from directory name
        repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("MyProject")
            .to_string()
    } else {
        print!(
            "Project name [{}]: ",
            repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("MyProject")
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("MyProject")
                .to_string()
        } else {
            input.to_string()
        }
    };

    println!(
        "{}  Project: {}",
        "✅".bright_green(),
        project_name.bright_white()
    );
    println!();

    // Step 3: Initialize Git repository
    println!("{}  Initializing Git repository...", "[3/7]".bright_cyan());
    crate::git::shell::ShellWorktreeManager::init_if_needed(repo_path)
        .await
        .inspect_err(|_| {
            println!("{}  Failed to initialize Git", "❌".bright_red());
        })?;
    println!("{}  Git repository ready", "✅".bright_green());
    println!();

    // Step 4: Determine which agents to enable
    println!("{}  Configuring agents...", "[4/7]".bright_cyan());

    let agents = if all_agents {
        vec!["frontend", "backend", "devops", "qa"]
    } else if no_prompt {
        vec!["frontend", "backend"]
    } else {
        println!("Which agents would you like to enable?");
        println!("  1. Frontend & Backend (recommended for web apps)");
        println!("  2. All agents (Frontend, Backend, DevOps, QA)");
        println!("  3. Custom selection");
        print!("Your choice [1]: ");
        std::io::stdout().flush()?;

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "2" => vec!["frontend", "backend", "devops", "qa"],
            "3" => {
                let mut selected = Vec::new();
                for agent in &["frontend", "backend", "devops", "qa"] {
                    print!("Enable {} agent? [Y/n]: ", agent);
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if !input.trim().eq_ignore_ascii_case("n") {
                        selected.push(*agent);
                    }
                }
                selected
            }
            _ => vec!["frontend", "backend"],
        }
    };

    println!(
        "{}  {} agents configured",
        "✅".bright_green(),
        agents.len()
    );
    println!();

    // Step 5: Create configuration
    println!("{}  Creating configuration...", "[5/7]".bright_cyan());

    let mut config = CcswarmConfig {
        project: ProjectConfig {
            name: project_name.clone(),
            repository: Default::default(),
            master_claude: MasterClaudeConfig {
                enable_proactive_mode: true,
                ..Default::default()
            },
        },
        agents: HashMap::new(),
        coordination: Default::default(),
    };

    // Add configured agents
    for agent_name in &agents {
        config.agents.insert(
            agent_name.to_string(),
            AgentConfig {
                specialization: agent_name.to_string(),
                worktree: format!("agents/{}-agent", agent_name),
                branch: format!("{}-agent", agent_name),
                claude_config: ClaudeConfig::for_agent(agent_name),
                claude_md_template: "default".to_string(),
            },
        );
    }

    // Enable proactive mode for better user experience
    config.project.master_claude.enable_proactive_mode = true;

    // Save configuration
    let config_file = repo_path.join("ccswarm.json");
    config.to_file(config_file.clone()).await?;
    println!("{}  Configuration saved", "✅".bright_green());
    println!();

    // Step 6: Create initial directory structure
    println!("{}  Creating project structure...", "[6/7]".bright_cyan());

    // Create agents directory
    let agents_dir = repo_path.join(".ccswarm/agents");
    tokio::fs::create_dir_all(&agents_dir).await?;

    // Create logs directory
    let logs_dir = repo_path.join(".ccswarm/logs");
    tokio::fs::create_dir_all(&logs_dir).await?;

    // Create .gitignore if it doesn't exist
    let gitignore_path = repo_path.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = r#"# ccswarm
.ccswarm/logs/
.ccswarm/sessions/
.ccswarm/*.db

# Environment
.env
*.key

# IDE
.vscode/
.idea/
*.swp
*~

# Dependencies
node_modules/
target/
dist/
build/
"#;
        tokio::fs::write(&gitignore_path, gitignore_content).await?;
    }

    // Create README if it doesn't exist
    let readme_path = repo_path.join("README.md");
    if !readme_path.exists() {
        let readme_content = format!(
            r#"# {}

An AI-orchestrated project powered by ccswarm.

## Getting Started

1. Ensure you have set your API key:
   ```bash
   export ANTHROPIC_API_KEY=your-key-here
   ```

2. Start the orchestration system:
   ```bash
   ccswarm start
   ```

3. Create your first task:
   ```bash
   ccswarm task "Create a hello world application"
   ```

4. Monitor progress:
   ```bash
   ccswarm tui
   ```

## Configured Agents

{} agents are configured for this project:
{}

## Documentation

- [ccswarm Documentation](https://github.com/nwiizo/ccswarm)
- Run `ccswarm help` for command reference
- Run `ccswarm tutorial` for interactive guide
"#,
            project_name,
            agents.len(),
            agents
                .iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n")
        );
        tokio::fs::write(&readme_path, readme_content).await?;
    }

    // Create CLAUDE.md for agent instructions
    let claude_md_path = repo_path.join("CLAUDE.md");
    if !claude_md_path.exists() {
        let claude_content = format!(
            r#"# {} - Project Instructions for AI Agents

## Project Overview
AI-orchestrated {} project

## Agent Guidelines

### All Agents
- Follow the project's coding standards
- Write comprehensive tests for all features
- Document your code thoroughly
- Communicate clearly with other agents

### Frontend Agent
- Use modern web technologies
- Ensure responsive design
- Focus on user experience
- Implement accessibility features

### Backend Agent
- Design scalable APIs
- Implement proper error handling
- Ensure data security
- Write efficient database queries

### DevOps Agent
- Automate deployment processes
- Set up CI/CD pipelines
- Monitor system performance
- Ensure infrastructure reliability

### QA Agent
- Write comprehensive test suites
- Perform security audits
- Check for performance issues
- Validate user workflows

## Success Criteria
- Code quality score > 8/10
- Test coverage > 80%
- All features documented
- No critical security issues
"#,
            project_name,
            project_name
        );
        tokio::fs::write(&claude_md_path, claude_content).await?;
    }

    println!("{}  Project structure created", "✅".bright_green());
    println!();

    // Step 7: Create initial commit
    println!("{}  Creating initial commit...", "[7/7]".bright_cyan());

    // Stage all files
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(repo_path)
        .output()?;

    // Create commit
    let commit_result = std::process::Command::new("git")
        .args(&["commit", "-m", "Initial ccswarm project setup"])
        .current_dir(repo_path)
        .output()?;

    if !commit_result.status.success() {
        // Might already have files committed, which is fine
        let output = String::from_utf8_lossy(&commit_result.stderr);
        if !output.contains("nothing to commit") {
            return Err(anyhow!("Failed to create initial commit: {}", output));
        }
    }

    println!("{}  Initial commit created", "✅".bright_green());
    println!();

    // Step 8: Run tests if requested
    if with_tests {
        println!("{}  Running initial tests...", "[8/8]".bright_cyan());

        // Test configuration loading
        match CcswarmConfig::from_file(config_file).await {
            Ok(_) => println!("    {} Configuration test", "✓".bright_green()),
            Err(e) => {
                println!("{}  Configuration test failed: {}", "⚠️".bright_yellow(), e);
            }
        }

        // Test git worktree functionality
        match crate::git::shell::ShellWorktreeManager::new(repo_path.to_path_buf()) {
            Ok(_) => println!("    {} Git worktree test", "✓".bright_green()),
            Err(e) => {
                println!("{}  Git worktree test failed: {}", "⚠️".bright_yellow(), e);
            }
        }

        println!("{}  Tests completed", "✅".bright_green());
        println!();
    }

    // Success message
    println!(
        "{}",
        "🎉 Quickstart completed successfully!"
            .bright_green()
            .bold()
    );
    println!();
    println!("{}", "Next steps:".bright_white());
    println!("  1. {}  ccswarm start", "Start system:".bright_black());
    println!(
        "  2. {}  ccswarm task \"Create hello world\"",
        "Create task:".bright_black()
    );
    println!("  3. {}  ccswarm tui", "Monitor:".bright_black());
    println!();
    println!("{}", "Happy orchestrating! 🚀".bright_cyan());

    Ok(())
}
