//! Error help topics and documentation

use colored::Colorize;
use std::collections::HashMap;

/// Error help topics
pub struct ErrorHelp;

impl ErrorHelp {
    /// Get all error codes with descriptions
    pub fn all_error_codes() -> HashMap<&'static str, ErrorInfo> {
        let mut codes = HashMap::new();

        codes.insert(
            "ENV001",
            ErrorInfo {
                title: "API Key Missing",
                category: "Environment",
                description: "The AI provider requires an API key to function",
                common_causes: vec![
                    "API key not set in environment",
                    "Incorrect environment variable name",
                    ".env file not loaded",
                ],
                quick_fix: Some("export ANTHROPIC_API_KEY=your-key-here"),
            },
        );

        codes.insert(
            "SES001",
            ErrorInfo {
                title: "Session Not Found",
                category: "Session Management",
                description: "The requested session does not exist or has been terminated",
                common_causes: vec![
                    "Session timed out",
                    "Session manually terminated",
                    "Session ID typo",
                    "System restart cleared sessions",
                ],
                quick_fix: Some("ccswarm session create --agent <name>"),
            },
        );

        codes.insert(
            "CFG001",
            ErrorInfo {
                title: "Configuration Not Found",
                category: "Configuration",
                description: "ccswarm.json configuration file is missing",
                common_causes: vec![
                    "Not in a ccswarm project directory",
                    "Configuration file deleted",
                    "Wrong working directory",
                ],
                quick_fix: Some("ccswarm init --name MyProject"),
            },
        );

        codes.insert(
            "GIT001",
            ErrorInfo {
                title: "Git Not Initialized",
                category: "Version Control",
                description: "ccswarm requires a git repository for agent isolation",
                common_causes: vec![
                    "Directory is not a git repository",
                    "Git not installed on system",
                    ".git directory corrupted",
                ],
                quick_fix: Some("git init"),
            },
        );

        codes.insert(
            "PRM001",
            ErrorInfo {
                title: "Permission Denied",
                category: "File System",
                description: "Cannot access file or directory due to permissions",
                common_causes: vec![
                    "File owned by different user",
                    "Insufficient permissions",
                    "Directory is read-only",
                ],
                quick_fix: Some("sudo chown -R $USER:$USER ."),
            },
        );

        codes.insert(
            "NET001",
            ErrorInfo {
                title: "Network Connection Failed",
                category: "Network",
                description: "Cannot connect to external service",
                common_causes: vec![
                    "No internet connection",
                    "Firewall blocking connection",
                    "Proxy configuration needed",
                    "Service temporarily down",
                ],
                quick_fix: None,
            },
        );

        codes.insert(
            "WRK001",
            ErrorInfo {
                title: "Git Worktree Conflict",
                category: "Version Control",
                description: "Branch is already checked out in another worktree",
                common_causes: vec![
                    "Branch already in use",
                    "Stale worktree references",
                    "Multiple agents using same branch",
                ],
                quick_fix: Some("git worktree prune"),
            },
        );

        codes.insert(
            "AGT001",
            ErrorInfo {
                title: "Agent Busy",
                category: "Agent Management",
                description: "Agent is currently processing another task",
                common_causes: vec![
                    "Previous task still running",
                    "Agent in error state",
                    "Resource locked by agent",
                ],
                quick_fix: None,
            },
        );

        codes.insert(
            "TSK001",
            ErrorInfo {
                title: "Invalid Task Format",
                category: "Task Management",
                description: "Task description doesn't meet requirements",
                common_causes: vec![
                    "Missing task description",
                    "Invalid priority level",
                    "Unknown task type",
                ],
                quick_fix: None,
            },
        );

        codes.insert(
            "AI001",
            ErrorInfo {
                title: "AI Response Error",
                category: "AI Provider",
                description: "AI provider returned unexpected response",
                common_causes: vec![
                    "API quota exceeded",
                    "Invalid API key",
                    "Request timeout",
                    "Provider service issue",
                ],
                quick_fix: None,
            },
        );

        codes
    }

    /// Display all error codes
    pub fn show_all_errors() {
        println!("{}", "📚 ccswarm Error Reference".bright_cyan().bold());
        println!("{}", "==========================".bright_cyan());
        println!();

        let codes = Self::all_error_codes();
        let mut by_category: HashMap<&str, Vec<(&str, &ErrorInfo)>> = HashMap::new();

        // Group by category
        for (code, info) in &codes {
            by_category
                .entry(info.category)
                .or_default()
                .push((code, info));
        }

        // Display by category
        for (category, mut errors) in by_category {
            println!("{}:", category.bright_yellow().bold());
            errors.sort_by_key(|&(code, _)| code);

            for (code, info) in errors {
                println!("  {} - {}", code.bright_white(), info.title);
                if let Some(fix) = &info.quick_fix {
                    println!("    {} {}", "Quick fix:".dimmed(), fix.bright_green());
                }
            }
            println!();
        }

        println!("{}:", "For detailed error diagnosis".dimmed());
        println!("  ccswarm doctor --error <CODE>");
        println!();
        println!("{}:", "To auto-fix errors".dimmed());
        println!("  ccswarm doctor --error <CODE> --fix");
    }
}

/// Error information structure
#[allow(dead_code)]
pub struct ErrorInfo {
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub common_causes: Vec<&'static str>,
    pub quick_fix: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_error_codes_populated() {
        let codes = ErrorHelp::all_error_codes();
        assert!(codes.len() >= 10);
        assert!(codes.contains_key("ENV001"));
        assert!(codes.contains_key("SES001"));
    }
}
