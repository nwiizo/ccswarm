//! Error recovery suggestions and auto-fix capabilities

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;

/// Error recovery database with actionable fixes
pub struct ErrorRecoveryDB {
    recovery_map: HashMap<String, RecoveryAction>,
}

/// A recovery action that can be taken to fix an error
#[derive(Clone)]
pub struct RecoveryAction {
    pub description: String,
    pub steps: Vec<RecoveryStep>,
    pub can_auto_fix: bool,
    pub risk_level: RiskLevel,
}

#[derive(Clone)]
pub enum RecoveryStep {
    Command { cmd: String, description: String },
    FileCreate { path: String, content: String },
    EnvVar { name: String, example: String },
    UserAction { description: String },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "✅",
            RiskLevel::Low => "🟢",
            RiskLevel::Medium => "🟡",
            RiskLevel::High => "🔴",
        }
    }

    pub fn color(&self) -> colored::Color {
        match self {
            RiskLevel::Safe => colored::Color::Green,
            RiskLevel::Low => colored::Color::BrightGreen,
            RiskLevel::Medium => colored::Color::Yellow,
            RiskLevel::High => colored::Color::Red,
        }
    }
}

impl Default for ErrorRecoveryDB {
    fn default() -> Self {
        let mut db = Self {
            recovery_map: HashMap::new(),
        };
        db.populate_common_fixes();
        db
    }
}

impl ErrorRecoveryDB {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get recovery action for a specific error code
    pub fn get_recovery(&self, error_code: &str) -> Option<&RecoveryAction> {
        self.recovery_map.get(error_code)
    }

    /// Populate database with common error fixes
    fn populate_common_fixes(&mut self) {
        // API Key Missing (ENV001)
        self.recovery_map.insert(
            "ENV001".to_string(),
            RecoveryAction {
                description: "Configure API key for AI provider".to_string(),
                steps: vec![
                    RecoveryStep::EnvVar {
                        name: "ANTHROPIC_API_KEY".to_string(),
                        example: "sk-ant-api03-...".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "echo 'ANTHROPIC_API_KEY=your-key' >> .env".to_string(),
                        description: "Add to .env file".to_string(),
                    },
                    RecoveryStep::UserAction {
                        description: "Visit https://console.anthropic.com to get your API key"
                            .to_string(),
                    },
                ],
                can_auto_fix: false,
                risk_level: RiskLevel::Safe,
            },
        );

        // Session Not Found (SES001)
        self.recovery_map.insert(
            "SES001".to_string(),
            RecoveryAction {
                description: "Recover from missing session".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "ccswarm session list".to_string(),
                        description: "List all active sessions".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "ccswarm session create --agent <name>".to_string(),
                        description: "Create new session".to_string(),
                    },
                ],
                can_auto_fix: true,
                risk_level: RiskLevel::Safe,
            },
        );

        // Config Not Found (CFG001)
        self.recovery_map.insert(
            "CFG001".to_string(),
            RecoveryAction {
                description: "Create missing configuration".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "ccswarm init --name MyProject".to_string(),
                        description: "Initialize new project".to_string(),
                    },
                    RecoveryStep::FileCreate {
                        path: "ccswarm.json".to_string(),
                        content: r#"{
  "project": {
    "name": "MyProject",
    "description": "My ccswarm project"
  },
  "agents": [
    {
      "name": "frontend",
      "role": "Frontend",
      "provider": "claude_code"
    },
    {
      "name": "backend",
      "role": "Backend",
      "provider": "claude_code"
    }
  ]
}"#
                        .to_string(),
                    },
                ],
                can_auto_fix: true,
                risk_level: RiskLevel::Low,
            },
        );

        // Git Not Initialized (GIT001)
        self.recovery_map.insert(
            "GIT001".to_string(),
            RecoveryAction {
                description: "Initialize git repository".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "git init".to_string(),
                        description: "Initialize new git repository".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "git add .".to_string(),
                        description: "Stage all files".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "git commit -m 'Initial commit'".to_string(),
                        description: "Create initial commit".to_string(),
                    },
                ],
                can_auto_fix: true,
                risk_level: RiskLevel::Low,
            },
        );

        // Permission Denied (PRM001)
        self.recovery_map.insert(
            "PRM001".to_string(),
            RecoveryAction {
                description: "Fix file permissions".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "sudo chown -R $USER:$USER .".to_string(),
                        description: "Change ownership to current user".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "chmod -R u+rw .".to_string(),
                        description: "Grant read/write permissions".to_string(),
                    },
                ],
                can_auto_fix: true,
                risk_level: RiskLevel::Medium,
            },
        );

        // Network Error (NET001)
        self.recovery_map.insert(
            "NET001".to_string(),
            RecoveryAction {
                description: "Troubleshoot network connection".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "ping -c 3 8.8.8.8".to_string(),
                        description: "Test internet connectivity".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "curl -I https://api.anthropic.com".to_string(),
                        description: "Test API endpoint".to_string(),
                    },
                    RecoveryStep::UserAction {
                        description: "Check proxy settings if behind corporate firewall"
                            .to_string(),
                    },
                ],
                can_auto_fix: false,
                risk_level: RiskLevel::Safe,
            },
        );

        // Worktree Conflict (WRK001)
        self.recovery_map.insert(
            "WRK001".to_string(),
            RecoveryAction {
                description: "Resolve git worktree conflict".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "git worktree list".to_string(),
                        description: "List all worktrees".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "git worktree prune".to_string(),
                        description: "Clean up stale worktrees".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "ccswarm cleanup --worktrees".to_string(),
                        description: "Clean up ccswarm worktrees".to_string(),
                    },
                ],
                can_auto_fix: true,
                risk_level: RiskLevel::Low,
            },
        );

        // AI Response Error (AI001)
        self.recovery_map.insert(
            "AI001".to_string(),
            RecoveryAction {
                description: "Recover from AI provider error".to_string(),
                steps: vec![
                    RecoveryStep::Command {
                        cmd: "ccswarm doctor --check-api".to_string(),
                        description: "Check API status".to_string(),
                    },
                    RecoveryStep::UserAction {
                        description: "Check API quota at https://console.anthropic.com".to_string(),
                    },
                    RecoveryStep::Command {
                        cmd: "ccswarm config set --retry-delay 5".to_string(),
                        description: "Increase retry delay".to_string(),
                    },
                ],
                can_auto_fix: false,
                risk_level: RiskLevel::Low,
            },
        );
    }

    /// Execute auto-fix for an error if possible
    pub async fn auto_fix(&self, error_code: &str, interactive: bool) -> Result<bool> {
        let recovery = match self.get_recovery(error_code) {
            Some(r) => r,
            None => return Ok(false),
        };

        if !recovery.can_auto_fix {
            return Ok(false);
        }

        if interactive {
            println!();
            println!(
                "{} {}",
                "🔧".bright_blue(),
                "Auto-fix available!".bright_blue().bold()
            );
            println!("   {}", recovery.description);
            println!(
                "   Risk level: {} {}",
                recovery.risk_level.icon(),
                format!("{:?}", recovery.risk_level).color(recovery.risk_level.color())
            );
            println!();
            print!("   Apply auto-fix? [y/N] ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                return Ok(false);
            }
        }

        // Execute recovery steps
        for (i, step) in recovery.steps.iter().enumerate() {
            match step {
                RecoveryStep::Command { cmd, description } => {
                    println!(
                        "   {} {}",
                        format!("[{}/{}]", i + 1, recovery.steps.len()).dimmed(),
                        description
                    );
                    println!("   {} {}", "$".dimmed(), cmd.dimmed());

                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .context("Failed to execute recovery command")?;

                    if !output.status.success() {
                        eprintln!("   {} Command failed", "❌".red());
                        return Ok(false);
                    }
                }
                RecoveryStep::FileCreate { path, content } => {
                    println!(
                        "   {} Creating {}",
                        format!("[{}/{}]", i + 1, recovery.steps.len()).dimmed(),
                        path
                    );
                    std::fs::write(path, content)
                        .context(format!("Failed to create file: {}", path))?;
                }
                RecoveryStep::EnvVar { name, .. } => {
                    println!(
                        "   {} Set environment variable: {}",
                        format!("[{}/{}]", i + 1, recovery.steps.len()).dimmed(),
                        name
                    );
                }
                RecoveryStep::UserAction { description } => {
                    println!(
                        "   {} Manual step: {}",
                        format!("[{}/{}]", i + 1, recovery.steps.len()).dimmed(),
                        description
                    );
                }
            }
        }

        println!();
        println!("   {} Auto-fix completed!", "✅".green());
        Ok(true)
    }
}

/// Interactive error resolver
pub struct ErrorResolver {
    recovery_db: ErrorRecoveryDB,
}

impl Default for ErrorResolver {
    fn default() -> Self {
        Self {
            recovery_db: ErrorRecoveryDB::new(),
        }
    }
}

impl ErrorResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show interactive resolution prompt
    pub async fn resolve_interactive(&self, error_code: &str) -> Result<()> {
        let recovery = match self.recovery_db.get_recovery(error_code) {
            Some(r) => r,
            None => {
                println!(
                    "   {} No automated recovery available for this error",
                    "ℹ️".blue()
                );
                return Ok(());
            }
        };

        println!();
        println!(
            "{} {}",
            "🔍".bright_cyan(),
            "Error Resolution Steps:".bright_cyan().bold()
        );
        println!();

        for (i, step) in recovery.steps.iter().enumerate() {
            match step {
                RecoveryStep::Command { cmd, description } => {
                    println!("   {}. {} {}", i + 1, "Run:".yellow(), description);
                    println!("      {}", cmd.bright_white().on_black());
                }
                RecoveryStep::FileCreate { path, .. } => {
                    println!(
                        "   {}. {} {}",
                        i + 1,
                        "Create file:".yellow(),
                        path.bright_white()
                    );
                }
                RecoveryStep::EnvVar { name, example } => {
                    println!(
                        "   {}. {} {}",
                        i + 1,
                        "Set environment variable:".yellow(),
                        name.bright_white()
                    );
                    println!("      Example: {}={}", name, example.dimmed());
                }
                RecoveryStep::UserAction { description } => {
                    println!(
                        "   {}. {} {}",
                        i + 1,
                        "Action required:".yellow(),
                        description
                    );
                }
            }
            println!();
        }

        if recovery.can_auto_fix {
            self.recovery_db.auto_fix(error_code, true).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_db_has_common_errors() {
        let db = ErrorRecoveryDB::new();

        assert!(db.get_recovery("ENV001").is_some());
        assert!(db.get_recovery("SES001").is_some());
        assert!(db.get_recovery("CFG001").is_some());
        assert!(db.get_recovery("GIT001").is_some());
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(RiskLevel::Safe.icon(), "✅");
        assert_eq!(RiskLevel::High.icon(), "🔴");
    }
}
