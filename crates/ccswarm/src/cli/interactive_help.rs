use colored::Colorize;
use std::collections::HashMap;

pub struct InteractiveHelp {
    topics: HashMap<String, HelpTopic>,
}

pub struct HelpTopic {
    pub title: String,
    pub description: String,
    pub examples: Vec<Example>,
    pub related: Vec<String>,
    pub tips: Vec<String>,
}

pub struct Example {
    pub command: String,
    pub description: String,
    pub output: Option<String>,
}

impl Default for InteractiveHelp {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractiveHelp {
    pub fn new() -> Self {
        let mut help = Self {
            topics: HashMap::new(),
        };

        help.populate_topics();
        help
    }

    fn populate_topics(&mut self) {
        // Getting Started
        self.topics.insert(
            "getting-started".to_string(),
            HelpTopic {
                title: "Getting Started with ccswarm".to_string(),
                description: "Learn how to set up and run your first AI orchestration".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm setup".to_string(),
                        description: "Interactive setup wizard for new projects".to_string(),
                        output: Some("🚀 Welcome to ccswarm!\nLet's set up your AI-powered multi-agent orchestration system.".to_string()),
                    },
                    Example {
                        command: "ccswarm init --name TodoApp --agents frontend,backend".to_string(),
                        description: "Quick initialization with specific agents".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm start".to_string(),
                        description: "Start the orchestration system".to_string(),
                        output: Some("✅ Master Claude initialized\n✅ 2 agents ready\n🎯 System ready for tasks".to_string()),
                    },
                ],
                related: vec!["agents".to_string(), "tasks".to_string()],
                tips: vec![
                    "Use 'ccswarm setup' for guided configuration".to_string(),
                    "Set ANTHROPIC_API_KEY before starting".to_string(),
                    "Run 'ccswarm doctor' to check system requirements".to_string(),
                ],
            },
        );

        // Working with Tasks
        self.topics.insert(
            "tasks".to_string(),
            HelpTopic {
                title: "Task Management".to_string(),
                description: "Create, delegate, and monitor tasks across agents".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm task \"Create user authentication\"".to_string(),
                        description: "Create a simple task".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm task \"Add login form\" --priority high --type feature"
                            .to_string(),
                        description: "Task with priority and type".to_string(),
                        output: Some(
                            "✅ Task created: task-a4b2c3d4\n🤖 Delegating to frontend agent..."
                                .to_string(),
                        ),
                    },
                    Example {
                        command: "ccswarm task list --status pending".to_string(),
                        description: "View pending tasks".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm delegate task \"Setup CI/CD\" --agent devops".to_string(),
                        description: "Delegate directly to specific agent".to_string(),
                        output: None,
                    },
                ],
                related: vec!["agents".to_string(), "quality".to_string()],
                tips: vec![
                    "Use clear, actionable task descriptions".to_string(),
                    "Add [high], [bug], or [test] modifiers in task text".to_string(),
                    "Tasks are automatically routed to appropriate agents".to_string(),
                ],
            },
        );

        // Agent Management
        self.topics.insert(
            "agents".to_string(),
            HelpTopic {
                title: "Agent Management".to_string(),
                description: "Control and monitor specialized AI agents".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm agent list".to_string(),
                        description: "View all configured agents".to_string(),
                        output: Some("🤖 Frontend Agent (Active) - 5 tasks completed\n🤖 Backend Agent (Idle) - 3 tasks completed".to_string()),
                    },
                    Example {
                        command: "ccswarm agent status frontend".to_string(),
                        description: "Detailed agent status".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm session create --agent backend".to_string(),
                        description: "Create new agent session".to_string(),
                        output: None,
                    },
                ],
                related: vec!["sessions".to_string(), "tasks".to_string()],
                tips: vec![
                    "Each agent specializes in specific domains".to_string(),
                    "Agents work in isolated git worktrees".to_string(),
                    "Use 'ccswarm agent logs' to debug issues".to_string(),
                ],
            },
        );

        // Sessions
        self.topics.insert(
            "sessions".to_string(),
            HelpTopic {
                title: "Session Management".to_string(),
                description: "Manage AI-powered terminal sessions".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm session list".to_string(),
                        description: "List all active sessions".to_string(),
                        output: Some("📍 frontend-a1b2c3 (Active) - Frontend Agent\n📍 backend-d4e5f6 (Idle) - Backend Agent".to_string()),
                    },
                    Example {
                        command: "ccswarm session attach frontend-a1b2c3".to_string(),
                        description: "Attach to agent session".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm session stats --show-savings".to_string(),
                        description: "View token savings statistics".to_string(),
                        output: Some("📊 Session Efficiency: Optimized\n🔧 Context Compression: Active".to_string()),
                    },
                ],
                related: vec!["agents".to_string()],
                tips: vec![
                    "Sessions persist context across tasks".to_string(),
                    "AI-session provides intelligent context management".to_string(),
                    "Sessions auto-recover after crashes".to_string(),
                ],
            },
        );

        // Quality & Review
        self.topics.insert(
            "quality".to_string(),
            HelpTopic {
                title: "Quality Assurance".to_string(),
                description: "Automated code quality and review system".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm quality check".to_string(),
                        description: "Run all quality checks".to_string(),
                        output: Some("🔍 Running quality checks...\n✅ Format: Passed\n✅ Lint: Passed\n⚠️ Test Coverage: 78% (threshold: 80%)".to_string()),
                    },
                    Example {
                        command: "ccswarm review trigger --all".to_string(),
                        description: "Trigger code review".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm quality test --coverage".to_string(),
                        description: "Run tests with coverage".to_string(),
                        output: None,
                    },
                ],
                related: vec!["tasks".to_string()],
                tips: vec![
                    "Quality checks run automatically every 30s".to_string(),
                    "Failed reviews create remediation tasks".to_string(),
                    "Customize thresholds in ccswarm.json".to_string(),
                ],
            },
        );

        // Troubleshooting
        self.topics.insert(
            "troubleshooting".to_string(),
            HelpTopic {
                title: "Troubleshooting Common Issues".to_string(),
                description: "Solutions for common problems".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm doctor".to_string(),
                        description: "Diagnose system issues".to_string(),
                        output: Some("🏥 System Diagnosis\n✅ Git: OK\n✅ API Keys: Set\n❌ Port 8080: In use".to_string()),
                    },
                    Example {
                        command: "RUST_LOG=debug ccswarm start".to_string(),
                        description: "Enable debug logging".to_string(),
                        output: None,
                    },
                    Example {
                        command: "ccswarm cleanup --force".to_string(),
                        description: "Clean up stuck resources".to_string(),
                        output: None,
                    },
                ],
                related: vec!["getting-started".to_string(), "errors".to_string()],
                tips: vec![
                    "Check logs in ~/.ccswarm/logs/".to_string(),
                    "Use --verbose flag for detailed output".to_string(),
                    "Join Discord for community support".to_string(),
                ],
            },
        );

        // Error Codes and Recovery
        self.topics.insert(
            "errors".to_string(),
            HelpTopic {
                title: "Error Codes and Recovery".to_string(),
                description: "Understand and fix ccswarm errors with visual diagrams".to_string(),
                examples: vec![
                    Example {
                        command: "ccswarm help errors".to_string(),
                        description: "List all error codes".to_string(),
                        output: Some("📚 Error Reference\n\nEnvironment:\n  ENV001 - API Key Missing\n\nSession Management:\n  SES001 - Session Not Found".to_string()),
                    },
                    Example {
                        command: "ccswarm doctor --error ENV001".to_string(),
                        description: "Diagnose specific error".to_string(),
                        output: Some("🔍 Error Code: ENV001\n📋 Configure API key for AI provider\n\nRecovery steps:\n1. Set environment variable\n2. Add to .env file".to_string()),
                    },
                    Example {
                        command: "ccswarm doctor --error SES001 --fix".to_string(),
                        description: "Auto-fix session error".to_string(),
                        output: Some("🔧 Auto-fix available!\nApplying fix...\n✅ Session created successfully".to_string()),
                    },
                    Example {
                        command: "ccswarm start --fix".to_string(),
                        description: "Run command with auto-fix enabled".to_string(),
                        output: None,
                    },
                ],
                related: vec!["troubleshooting".to_string()],
                tips: vec![
                    "Most errors show visual diagrams to explain the problem".to_string(),
                    "Use --fix flag to attempt automatic recovery".to_string(),
                    "Error codes help support diagnose issues quickly".to_string(),
                    "Run 'ccswarm doctor --check-api' to test connectivity".to_string(),
                ],
            },
        );
    }

    pub fn show_topic(&self, topic: &str) {
        // Special handling for errors topic
        if topic == "errors" {
            use crate::cli::error_help::ErrorHelp;
            ErrorHelp::show_all_errors();
            return;
        }

        if let Some(help_topic) = self.topics.get(topic) {
            self.display_topic(help_topic);
        } else {
            self.show_topic_list();
        }
    }

    pub fn show_topic_list(&self) {
        println!();
        println!("{}", "📚 Available Help Topics".bright_cyan().bold());
        println!();

        let topics = vec![
            (
                "getting-started",
                "Learn the basics and set up your first project",
            ),
            ("tasks", "Create and manage tasks for AI agents"),
            ("agents", "Work with specialized AI agents"),
            ("sessions", "Manage AI-powered terminal sessions"),
            ("quality", "Code quality and automated reviews"),
            ("troubleshooting", "Fix common problems"),
            ("errors", "Error codes and recovery procedures"),
        ];

        for (key, desc) in topics {
            if self.topics.contains_key(key) {
                println!(
                    "  {} {}",
                    format!("{:>16}", key).bright_white(),
                    format!("- {}", desc).dimmed()
                );
            }
        }

        println!();
        println!("{}", "💡 Usage:".bright_yellow());
        println!("  ccswarm help <topic>     View detailed help");
        println!("  ccswarm help search <q>  Search help topics");
        println!();
    }

    fn display_topic(&self, topic: &HelpTopic) {
        println!();
        println!("{}", topic.title.bright_cyan().bold());
        println!("{}", "─".repeat(topic.title.len()).bright_cyan());
        println!();
        println!("{}", topic.description.white());

        if !topic.examples.is_empty() {
            println!();
            println!("{}", "📝 Examples:".bright_yellow());
            println!();

            for example in &topic.examples {
                println!("  {}", example.command.bright_white());
                println!("  {}", example.description.dimmed());

                if let Some(output) = &example.output {
                    println!();
                    for line in output.lines() {
                        println!("  {}", line.bright_green());
                    }
                }
                println!();
            }
        }

        if !topic.tips.is_empty() {
            println!("{}", "💡 Tips:".bright_yellow());
            for tip in &topic.tips {
                println!("  • {}", tip.white());
            }
            println!();
        }

        if !topic.related.is_empty() {
            println!("{}", "🔗 Related topics:".bright_cyan());
            print!("  ");
            for (i, related) in topic.related.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", related.bright_white());
            }
            println!("\n");
        }
    }

    pub fn search(&self, query: &str) -> Vec<(&str, &HelpTopic)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (key, topic) in &self.topics {
            let score = self.calculate_relevance(topic, &query_lower);
            if score > 0 {
                results.push((key.as_str(), topic, score));
            }
        }

        results.sort_by(|a, b| b.2.cmp(&a.2));
        results.into_iter().map(|(k, t, _)| (k, t)).collect()
    }

    fn calculate_relevance(&self, topic: &HelpTopic, query: &str) -> u32 {
        let mut score = 0;

        // Title match (highest weight)
        if topic.title.to_lowercase().contains(query) {
            score += 100;
        }

        // Description match
        if topic.description.to_lowercase().contains(query) {
            score += 50;
        }

        // Example matches
        for example in &topic.examples {
            if example.command.to_lowercase().contains(query) {
                score += 30;
            }
            if example.description.to_lowercase().contains(query) {
                score += 20;
            }
        }

        // Tips matches
        for tip in &topic.tips {
            if tip.to_lowercase().contains(query) {
                score += 10;
            }
        }

        score
    }
}

/// Quick contextual help
pub fn show_quick_help(context: &str) {
    let tips = match context {
        "task-created" => vec![
            "View task progress: ccswarm task status <id>",
            "List all tasks: ccswarm task list",
            "Tasks are automatically delegated to the best agent",
        ],
        "session-created" => vec![
            "Attach to session: ccswarm session attach <id>",
            "Sessions provide efficient context management",
            "Sessions persist across ccswarm restarts",
        ],
        "agent-busy" => vec![
            "Check agent status: ccswarm agent status",
            "Queue tasks will be processed in order",
            "Use --force to interrupt (not recommended)",
        ],
        "error-occurred" => vec![
            "Enable debug logs: RUST_LOG=debug ccswarm <command>",
            "Check system health: ccswarm doctor",
            "View detailed help: ccswarm help troubleshooting",
        ],
        _ => vec![
            "Get help: ccswarm help",
            "View examples: ccswarm help <topic>",
            "Join our Discord for support",
        ],
    };

    println!();
    println!("{}", "💡 Quick tips:".bright_yellow());
    for tip in tips {
        println!("  • {}", tip.dimmed());
    }
}
