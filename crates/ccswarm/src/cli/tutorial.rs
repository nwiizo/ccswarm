use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

pub struct InteractiveTutorial {
    completed_steps: Vec<String>,
    pub current_chapter: usize,
}

impl Default for InteractiveTutorial {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractiveTutorial {
    pub fn new() -> Self {
        Self {
            completed_steps: Vec::new(),
            current_chapter: 0,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.show_welcome().await?;

        loop {
            match self.current_chapter {
                0 => {
                    if self.chapter_1_basics().await? {
                        self.current_chapter += 1;
                    } else {
                        break;
                    }
                }
                1 => {
                    if self.chapter_2_agents().await? {
                        self.current_chapter += 1;
                    } else {
                        break;
                    }
                }
                2 => {
                    if self.chapter_3_tasks().await? {
                        self.current_chapter += 1;
                    } else {
                        break;
                    }
                }
                3 => {
                    self.show_completion().await?;
                    break;
                }
                _ => break,
            }
        }

        Ok(())
    }

    async fn show_welcome(&self) -> Result<()> {
        clear_screen();

        println!(
            "{}",
            "
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              🚀 Welcome to ccswarm Tutorial! 🚀              ║
║                                                               ║
║         AI-Powered Multi-Agent Orchestration System           ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
        "
            .bright_cyan()
        );

        println!();
        println!(
            "{}",
            "This interactive tutorial will teach you:".bright_white()
        );
        println!();
        println!(
            "  📚 Chapter 1: {}",
            "Setting Up Your First Project".bright_yellow()
        );
        println!(
            "  🤖 Chapter 2: {}",
            "Working with AI Agents".bright_yellow()
        );
        println!(
            "  📝 Chapter 3: {}",
            "Creating and Managing Tasks".bright_yellow()
        );
        println!("  🎯 Chapter 4: {}", "Advanced Features".bright_yellow());
        println!();
        println!("{}", "Each chapter takes ~3-5 minutes.".dimmed());
        println!();

        self.wait_for_enter("Press ENTER to begin...").await?;
        Ok(())
    }

    async fn chapter_1_basics(&mut self) -> Result<bool> {
        clear_screen();
        self.show_chapter_header(1, "Setting Up Your First Project")
            .await;

        // Step 1: Check environment
        self.show_step(1, "Environment Check").await;
        println!("First, let's make sure everything is set up correctly.");
        println!();

        self.simulate_command("ccswarm doctor").await?;

        println!("{}", "✅ Git: Installed".bright_green());
        sleep(Duration::from_millis(300)).await;
        println!("{}", "✅ Rust: 1.70+".bright_green());
        sleep(Duration::from_millis(300)).await;

        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            println!("{}", "⚠️  API Key: Not set".bright_yellow());
            println!();
            println!(
                "{}",
                "You'll need an API key from Anthropic to use AI features.".yellow()
            );
            println!(
                "Get one at: {}",
                "https://console.anthropic.com".bright_cyan().underline()
            );
        } else {
            println!("{}", "✅ API Key: Set".bright_green());
        }

        println!();
        self.wait_for_enter("Got it! Press ENTER to continue...")
            .await?;

        // Step 2: Create project
        clear_screen();
        self.show_step(2, "Creating Your Project").await;
        println!("Let's create a simple TODO app project:");
        println!();

        self.simulate_command("ccswarm init --name TodoApp --agents frontend,backend")
            .await?;

        self.animate_progress("Initializing git repository", 1000)
            .await;
        self.animate_progress("Creating project structure", 800)
            .await;
        self.animate_progress("Configuring AI agents", 1200).await;

        println!();
        println!("{}", "✅ Project created successfully!".bright_green());
        println!();
        println!("Your project structure:");
        println!(
            "{}",
            "
TodoApp/
├── ccswarm.json      # Configuration file
├── agents/           # Agent workspaces
│   ├── frontend/     # Frontend agent workspace
│   └── backend/      # Backend agent workspace
└── .ccswarm/         # Session data
        "
            .bright_white()
        );

        self.completed_steps.push("project_created".to_string());
        println!();

        if self.ask_continue().await? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn chapter_2_agents(&mut self) -> Result<bool> {
        clear_screen();
        self.show_chapter_header(2, "Working with AI Agents").await;

        // Understanding agents
        self.show_step(1, "Understanding Agents").await;
        println!("ccswarm uses specialized AI agents for different tasks:");
        println!();

        let agents = vec![
            ("🎨 Frontend Agent", "React, Vue, UI/UX, CSS", "frontend"),
            (
                "⚙️  Backend Agent",
                "APIs, databases, business logic",
                "backend",
            ),
            ("🚀 DevOps Agent", "Docker, CI/CD, deployment", "devops"),
            ("🧪 QA Agent", "Testing, quality assurance", "qa"),
        ];

        for (icon_name, desc, _) in &agents {
            println!("  {} - {}", icon_name.bright_yellow(), desc.white());
            sleep(Duration::from_millis(400)).await;
        }

        println!();
        self.wait_for_enter("Press ENTER to see agents in action...")
            .await?;

        // Starting agents
        clear_screen();
        self.show_step(2, "Starting the System").await;

        self.simulate_command("ccswarm start").await?;

        self.animate_progress("Starting Master Claude", 1500).await;
        println!(
            "{}",
            "  🧠 Master Claude: Technical lead coordinating agents".dimmed()
        );
        sleep(Duration::from_millis(500)).await;

        self.animate_progress("Starting Frontend Agent", 1000).await;
        self.animate_progress("Starting Backend Agent", 1000).await;

        println!();
        println!("{}", "✅ All systems operational!".bright_green());
        println!();

        // Show agent status
        self.simulate_command("ccswarm agent list").await?;

        println!("{}", "AGENT      STATUS    TASKS  SESSION".bright_white());
        println!("{}", "─────────────────────────────────────".dimmed());
        println!(
            "frontend   {}      0     ai-session-a1b2c3",
            "Active".bright_green()
        );
        println!(
            "backend    {}       0     ai-session-d4e5f6",
            "Active".bright_green()
        );

        self.completed_steps.push("agents_started".to_string());
        println!();

        if self.ask_continue().await? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn chapter_3_tasks(&mut self) -> Result<bool> {
        clear_screen();
        self.show_chapter_header(3, "Creating and Managing Tasks")
            .await;

        // Creating first task
        self.show_step(1, "Your First Task").await;
        println!("Let's create a task for our TODO app:");
        println!();

        self.simulate_command("ccswarm task \"Create a React component for todo items\"")
            .await?;

        self.animate_progress("Analyzing task", 800).await;
        self.animate_progress("Determining best agent", 600).await;
        println!(
            "{}",
            "  → Frontend Agent selected (React expertise)".bright_cyan()
        );
        self.animate_progress("Delegating to Frontend Agent", 1000)
            .await;

        println!();
        println!("{}", "✅ Task delegated successfully!".bright_green());
        println!("   Task ID: task-7f8g9h0i");
        println!();

        // Monitoring progress
        sleep(Duration::from_secs(1)).await;
        println!(
            "{}",
            "The Frontend Agent is now working on your task...".dimmed()
        );
        println!();

        self.simulate_typing("📝 Frontend Agent: Creating TodoItem.jsx component...")
            .await;
        sleep(Duration::from_millis(800)).await;
        self.simulate_typing("📝 Frontend Agent: Adding prop types and styling...")
            .await;
        sleep(Duration::from_millis(800)).await;
        self.simulate_typing("✅ Frontend Agent: Component created with tests!")
            .await;

        println!();
        println!();

        // Task modifiers
        self.show_step(2, "Task Modifiers").await;
        println!("You can add modifiers to your tasks:");
        println!();

        println!("  {} Priority modifiers", "•".bright_cyan());
        println!(
            "    ccswarm task \"Fix login bug\" {}",
            "[high]".bright_red()
        );
        println!();

        println!("  {} Type modifiers", "•".bright_cyan());
        println!(
            "    ccswarm task \"Add user auth\" {}",
            "[feature]".bright_green()
        );
        println!(
            "    ccswarm task \"Fix memory leak\" {}",
            "[bug]".bright_yellow()
        );
        println!();

        println!("  {} Direct delegation", "•".bright_cyan());
        println!(
            "    ccswarm task \"Setup CI/CD\" {}",
            "--agent devops".bright_white()
        );

        self.completed_steps.push("task_created".to_string());
        println!();

        if self.ask_continue().await? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn show_completion(&self) -> Result<()> {
        clear_screen();

        println!(
            "{}",
            "
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              🎉 Congratulations! 🎉                           ║
║                                                               ║
║         You've completed the ccswarm tutorial!                ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
        "
            .bright_green()
        );

        println!();
        println!("{}", "You've learned:".bright_white());
        println!();

        for step in &[
            "✓ Setting up a project",
            "✓ Working with AI agents",
            "✓ Creating and managing tasks",
        ] {
            println!("  {}", step.bright_green());
            sleep(Duration::from_millis(300)).await;
        }

        println!();
        println!("{}", "📚 What's Next?".bright_cyan());
        println!();
        println!(
            "  • Explore quality checks: {}",
            "ccswarm help quality".bright_white()
        );
        println!(
            "  • Learn about sessions: {}",
            "ccswarm help sessions".bright_white()
        );
        println!(
            "  • Try auto-create: {}",
            "ccswarm auto-create \"TODO app\"".bright_white()
        );
        println!(
            "  • Join our community: {}",
            "https://discord.gg/ccswarm".bright_cyan().underline()
        );

        println!();
        println!("{}", "Happy orchestrating! 🚀".bright_yellow());
        println!();

        Ok(())
    }

    // Helper methods
    async fn show_chapter_header(&self, num: u8, title: &str) {
        println!(
            "{}",
            format!("Chapter {} : {}", num, title).bright_cyan().bold()
        );
        println!("{}", "═".repeat(50).bright_cyan());
        println!();
    }

    async fn show_step(&self, num: u8, title: &str) {
        println!();
        println!(
            "{} {}",
            format!("Step {}:", num).bright_yellow(),
            title.bright_white().bold()
        );
        println!();
    }

    async fn simulate_command(&self, cmd: &str) -> Result<()> {
        print!("{} ", "$".bright_green());
        self.simulate_typing(cmd).await;
        println!();
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    async fn simulate_typing(&self, text: &str) {
        for ch in text.chars() {
            print!("{}", ch);
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(30)).await;
        }
    }

    async fn animate_progress(&self, message: &str, duration_ms: u64) {
        print!("  {} ", "⏳".bright_yellow());
        print!("{}", message);
        io::stdout().flush().unwrap();

        sleep(Duration::from_millis(duration_ms)).await;

        print!("\r  {} {}\n", "✅".bright_green(), message);
        io::stdout().flush().unwrap();
    }

    async fn wait_for_enter(&self, prompt: &str) -> Result<()> {
        println!();
        print!("{}", prompt.bright_cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }

    async fn ask_continue(&self) -> Result<bool> {
        println!();
        print!("{}", "Continue to next chapter? [Y/n] ".bright_cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        Ok(input.is_empty() || input == "y" || input == "yes")
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

// Quick tips that appear during operations
#[allow(dead_code)]
pub fn show_contextual_tip() {
    let tips = [
        "💡 Tip: Use 'ccswarm task list' to see all pending tasks",
        "💡 Tip: Sessions provide efficient context management through smart caching",
        "💡 Tip: Add [high] to task descriptions for priority handling",
        "💡 Tip: Quality checks run automatically every 30 seconds",
        "💡 Tip: Use 'ccswarm tui' for real-time monitoring",
        "💡 Tip: Agents work in isolated git worktrees for safety",
        "💡 Tip: Failed tasks are automatically retried with fixes",
    ];

    use rand::prelude::*;
    let mut rng = rand::rngs::ThreadRng::default();
    let tip = tips.choose(&mut rng).unwrap_or(&tips[0]);
    println!();
    println!("{}", tip.dimmed());
}
