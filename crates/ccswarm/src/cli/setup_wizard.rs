use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::{
    AgentConfig, CcswarmConfig, ClaudeConfig, CoordinationConfig, MasterClaudeConfig,
    ProjectConfig, RepositoryConfig, ThinkMode,
};

pub struct SetupWizard;

impl SetupWizard {
    pub async fn run() -> Result<CcswarmConfig> {
        println!();
        println!("{}", "🚀 Welcome to ccswarm!".bright_cyan().bold());
        println!(
            "{}",
            "Let's set up your AI-powered multi-agent orchestration system.".bright_white()
        );
        println!();

        // Project basics
        let project_name = Self::prompt(
            "What's your project name?",
            "MyProject",
            Some("This will be used to identify your project"),
        )?;

        // Repository setup
        println!();
        println!("{}", "📁 Repository Configuration".bright_yellow());
        let repo_url = Self::prompt(
            "Repository URL (or '.' for current directory)",
            ".",
            Some("Where your code lives - can be a GitHub URL or local path"),
        )?;

        // Agent selection
        println!();
        println!("{}", "🤖 Agent Selection".bright_yellow());
        println!("Which specialized agents do you want to enable?");
        println!();

        let enable_frontend =
            Self::confirm("Frontend Agent", true, Some("React, Vue, UI development"))?;
        let enable_backend =
            Self::confirm("Backend Agent", true, Some("APIs, databases, server logic"))?;
        let enable_devops = Self::confirm("DevOps Agent", true, Some("Docker, Kubernetes, CI/CD"))?;
        let enable_qa = Self::confirm("QA Agent", false, Some("Testing, quality assurance"))?;

        // AI Provider selection
        println!();
        println!("{}", "🧠 AI Provider Configuration".bright_yellow());
        println!("Which AI provider will you use?");
        println!();
        println!("  1. Claude Code (Recommended) - Best integration");
        println!("  2. Aider - Open source alternative");
        println!("  3. Custom - Your own provider");

        let provider_choice = Self::prompt_number("Select provider", 1, 1, 3)?;
        let _use_real_api = provider_choice != 3;

        // Advanced options
        println!();
        let advanced = Self::confirm(
            "Configure advanced options?",
            false,
            Some("Proactive mode, quality thresholds, auto-accept"),
        )?;

        let (enable_proactive, quality_threshold, think_mode) = if advanced {
            println!();
            println!("{}", "⚙️  Advanced Configuration".bright_yellow());

            let proactive = Self::confirm(
                "Enable proactive mode?",
                true,
                Some("AI predicts and suggests next tasks automatically"),
            )?;

            let quality =
                Self::prompt_number("Quality threshold (0.0-1.0)", 85, 0, 100)? as f64 / 100.0;

            println!();
            println!("Thinking modes:");
            println!("  1. Think - Fast responses");
            println!("  2. Think Hard - Better reasoning");
            println!("  3. Ultra Think - Maximum intelligence");

            let think_choice = Self::prompt_number("Select thinking mode", 2, 1, 3)?;
            let mode = match think_choice {
                1 => ThinkMode::Think,
                3 => ThinkMode::UltraThink,
                _ => ThinkMode::ThinkHard,
            };

            (proactive, quality, mode)
        } else {
            (true, 0.85, ThinkMode::ThinkHard)
        };

        // Build configuration
        let mut agents = std::collections::HashMap::new();

        if enable_frontend {
            agents.insert(
                "frontend".to_string(),
                AgentConfig {
                    specialization: "frontend".to_string(),
                    worktree: "agents/frontend".to_string(),
                    branch: "feature/frontend".to_string(),
                    claude_config: ClaudeConfig::for_agent("frontend"),
                    claude_md_template: "frontend_specialist".to_string(),
                },
            );
        }

        if enable_backend {
            agents.insert(
                "backend".to_string(),
                AgentConfig {
                    specialization: "backend".to_string(),
                    worktree: "agents/backend".to_string(),
                    branch: "feature/backend".to_string(),
                    claude_config: ClaudeConfig::for_agent("backend"),
                    claude_md_template: "backend_specialist".to_string(),
                },
            );
        }

        if enable_devops {
            agents.insert(
                "devops".to_string(),
                AgentConfig {
                    specialization: "devops".to_string(),
                    worktree: "agents/devops".to_string(),
                    branch: "feature/devops".to_string(),
                    claude_config: ClaudeConfig::for_agent("devops"),
                    claude_md_template: "devops_specialist".to_string(),
                },
            );
        }

        if enable_qa {
            agents.insert(
                "qa".to_string(),
                AgentConfig {
                    specialization: "qa".to_string(),
                    worktree: "agents/qa".to_string(),
                    branch: "feature/qa".to_string(),
                    claude_config: ClaudeConfig::for_agent("qa"),
                    claude_md_template: "qa_specialist".to_string(),
                },
            );
        }

        let config = CcswarmConfig {
            project: ProjectConfig {
                name: project_name,
                repository: RepositoryConfig {
                    url: repo_url,
                    main_branch: "main".to_string(),
                },
                master_claude: MasterClaudeConfig {
                    role: "technical_lead".to_string(),
                    quality_threshold,
                    think_mode,
                    permission_level: "supervised".to_string(),
                    claude_config: ClaudeConfig::for_master(),
                    enable_proactive_mode: enable_proactive,
                    proactive_frequency: 30,
                    high_frequency: 15,
                },
            },
            agents,
            coordination: CoordinationConfig {
                communication_method: "json_files".to_string(),
                sync_interval: 30,
                quality_gate_frequency: "on_commit".to_string(),
                master_review_trigger: "all_tasks_complete".to_string(),
            },
        };

        // Save configuration
        println!();
        println!("{}", "💾 Configuration Summary".bright_green());
        println!();
        println!("Project: {}", config.project.name.bright_white());
        println!("Agents: {} enabled", config.agents.len());
        println!(
            "Proactive Mode: {}",
            if enable_proactive { "✓" } else { "✗" }
        );
        println!("Quality Threshold: {}%", (quality_threshold * 100.0) as u8);
        println!();

        let save = Self::confirm("Save this configuration?", true, None)?;
        if save {
            let config_path = PathBuf::from("ccswarm.json");
            config.to_file(config_path).await?;
            println!();
            println!(
                "{}",
                "✅ Configuration saved to ccswarm.json".bright_green()
            );

            // Show next steps
            println!();
            println!("{}", "🎯 Next Steps:".bright_cyan());
            println!();
            println!("  1. Set your API keys:");
            println!(
                "     {} ANTHROPIC_API_KEY=your-key",
                "export".bright_white()
            );
            println!();
            println!("  2. Initialize your project:");
            println!("     {} init", "ccswarm".bright_white());
            println!();
            println!("  3. Start orchestrating:");
            println!("     {} start", "ccswarm".bright_white());
            println!();
            println!("Need help? Run: {} help", "ccswarm".bright_white());
        }

        Ok(config)
    }

    fn prompt(question: &str, default: &str, hint: Option<&str>) -> Result<String> {
        print!("{} ", question.bright_cyan());
        if !default.is_empty() {
            print!("[{}] ", default.dimmed());
        }

        if let Some(hint_text) = hint {
            println!();
            println!("  {}", hint_text.dimmed());
            print!("> ");
        }

        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        Ok(if input.is_empty() {
            default.to_string()
        } else {
            input.to_string()
        })
    }

    fn prompt_number(question: &str, default: u32, min: u32, max: u32) -> Result<u32> {
        loop {
            let input = Self::prompt(question, &default.to_string(), None)?;
            match input.parse::<u32>() {
                Ok(num) if num >= min && num <= max => return Ok(num),
                _ => {
                    println!(
                        "{}",
                        format!("Please enter a number between {} and {}", min, max).red()
                    );
                }
            }
        }
    }

    fn confirm(question: &str, default: bool, hint: Option<&str>) -> Result<bool> {
        print!("{} ", question.bright_cyan());
        print!("[{}] ", if default { "Y/n" } else { "y/N" }.dimmed());

        if let Some(hint_text) = hint {
            print!("- {}", hint_text.dimmed());
        }

        print!(": ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        Ok(if input.is_empty() {
            default
        } else {
            input == "y" || input == "yes"
        })
    }
}
