//! GitHub Issue integration for Piece/Movement workflows.
//!
//! Enables creating tasks from GitHub issues and posting workflow results
//! back as comments. Inspired by takt's issue integration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// GitHub issue data used to generate a workflow task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Issue body (markdown)
    pub body: String,
    /// Labels
    pub labels: Vec<String>,
    /// Assignees
    pub assignees: Vec<String>,
    /// Repository (owner/repo)
    pub repository: String,
    /// Issue URL
    pub url: String,
}

/// Configuration for GitHub issue integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssueConfig {
    /// Repository (owner/repo)
    pub repository: String,
    /// Label-to-piece mapping (e.g., "bug" -> "review-fix")
    #[serde(default)]
    pub label_piece_mapping: HashMap<String, String>,
    /// Whether to post results as comments
    #[serde(default = "default_true")]
    pub post_results: bool,
    /// Whether to close the issue on success
    #[serde(default)]
    pub close_on_success: bool,
    /// Default piece when no label matches
    #[serde(default)]
    pub default_piece: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Result to post back to an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueResult {
    /// The issue number
    pub issue_number: u64,
    /// Whether the workflow succeeded
    pub success: bool,
    /// Summary of what was done
    pub summary: String,
    /// Detailed output
    pub details: Option<String>,
    /// Files changed
    pub files_changed: Vec<String>,
}

/// Converts a GitHub issue into a workflow task description
pub struct IssueTaskGenerator {
    config: GitHubIssueConfig,
}

impl IssueTaskGenerator {
    pub fn new(config: GitHubIssueConfig) -> Self {
        Self { config }
    }

    /// Generate a task description from a GitHub issue
    pub fn generate_task(&self, issue: &GitHubIssue) -> TaskFromIssue {
        let task_text = format!(
            "GitHub Issue #{}: {}\n\nRepository: {}\n\n{}\n\nLabels: {}",
            issue.number,
            issue.title,
            issue.repository,
            issue.body,
            issue.labels.join(", ")
        );

        // Determine which piece to use based on labels
        let piece_name = self.determine_piece(issue);

        debug!(
            "Generated task from issue #{}: piece={}",
            issue.number,
            piece_name.as_deref().unwrap_or("default")
        );

        TaskFromIssue {
            task_text,
            piece_name,
            issue_number: issue.number,
            variables: self.extract_variables(issue),
        }
    }

    /// Determine the piece to use based on issue labels
    fn determine_piece(&self, issue: &GitHubIssue) -> Option<String> {
        // Check label-to-piece mapping
        for label in &issue.labels {
            if let Some(piece) = self.config.label_piece_mapping.get(label) {
                info!(
                    "Label '{}' maps to piece '{}' for issue #{}",
                    label, piece, issue.number
                );
                return Some(piece.clone());
            }
        }

        // Fall back to default piece
        self.config.default_piece.clone()
    }

    /// Extract workflow variables from issue metadata
    fn extract_variables(&self, issue: &GitHubIssue) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("issue_number".to_string(), issue.number.to_string());
        vars.insert("issue_title".to_string(), issue.title.clone());
        vars.insert("repository".to_string(), issue.repository.clone());
        vars.insert("issue_url".to_string(), issue.url.clone());
        if !issue.labels.is_empty() {
            vars.insert("labels".to_string(), issue.labels.join(","));
        }
        if !issue.assignees.is_empty() {
            vars.insert("assignees".to_string(), issue.assignees.join(","));
        }
        vars
    }

    /// Format a workflow result as a GitHub comment
    pub fn format_result_comment(&self, result: &IssueResult) -> String {
        let status_emoji = if result.success {
            "white_check_mark"
        } else {
            "x"
        };
        let status_text = if result.success {
            "completed successfully"
        } else {
            "encountered issues"
        };

        let mut comment = format!(
            "## Workflow Result :{status_emoji}:\n\nThe automated workflow {status_text}.\n\n"
        );
        comment.push_str(&format!("### Summary\n\n{}\n", result.summary));

        if !result.files_changed.is_empty() {
            comment.push_str("\n### Files Changed\n\n");
            for file in &result.files_changed {
                comment.push_str(&format!("- `{}`\n", file));
            }
        }

        if let Some(ref details) = result.details {
            comment.push_str(&format!(
                "\n<details>\n<summary>Details</summary>\n\n{}\n</details>\n",
                details
            ));
        }

        comment.push_str("\n---\n*Generated by ccswarm*\n");
        comment
    }
}

/// A task generated from a GitHub issue
#[derive(Debug, Clone)]
pub struct TaskFromIssue {
    /// The task description text
    pub task_text: String,
    /// Recommended piece to use
    pub piece_name: Option<String>,
    /// Source issue number
    pub issue_number: u64,
    /// Variables extracted from the issue
    pub variables: HashMap<String, String>,
}

/// Parse a GitHub issue from `gh` CLI JSON output
pub fn parse_gh_issue(json: &str) -> Result<GitHubIssue> {
    let value: serde_json::Value =
        serde_json::from_str(json).context("Failed to parse GitHub issue JSON")?;

    let number = value["number"].as_u64().context("Missing issue number")?;
    let title = value["title"].as_str().unwrap_or("").to_string();
    let body = value["body"].as_str().unwrap_or("").to_string();
    let url = value["url"].as_str().unwrap_or("").to_string();

    let labels: Vec<String> = value["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let assignees: Vec<String> = value["assignees"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a["login"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let repository = value["repository"]["nameWithOwner"]
        .as_str()
        .or_else(|| value["repository"]["full_name"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(GitHubIssue {
        number,
        title,
        body,
        labels,
        assignees,
        repository,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue() -> GitHubIssue {
        GitHubIssue {
            number: 42,
            title: "Login page CSS is broken".to_string(),
            body: "The login button overlaps with the input field on mobile".to_string(),
            labels: vec!["bug".to_string(), "frontend".to_string()],
            assignees: vec!["dev1".to_string()],
            repository: "myorg/myapp".to_string(),
            url: "https://github.com/myorg/myapp/issues/42".to_string(),
        }
    }

    fn sample_config() -> GitHubIssueConfig {
        let mut mapping = HashMap::new();
        mapping.insert("bug".to_string(), "review-fix".to_string());
        mapping.insert("feature".to_string(), "default".to_string());

        GitHubIssueConfig {
            repository: "myorg/myapp".to_string(),
            label_piece_mapping: mapping,
            post_results: true,
            close_on_success: false,
            default_piece: Some("default".to_string()),
        }
    }

    #[test]
    fn test_generate_task_from_issue() {
        let generator = IssueTaskGenerator::new(sample_config());
        let task = generator.generate_task(&sample_issue());

        assert_eq!(task.issue_number, 42);
        assert!(task.task_text.contains("Login page CSS is broken"));
        assert!(task.task_text.contains("#42"));
        assert_eq!(task.piece_name, Some("review-fix".to_string()));
    }

    #[test]
    fn test_label_piece_mapping() {
        let generator = IssueTaskGenerator::new(sample_config());

        let mut issue = sample_issue();
        issue.labels = vec!["feature".to_string()];
        let task = generator.generate_task(&issue);
        assert_eq!(task.piece_name, Some("default".to_string()));
    }

    #[test]
    fn test_default_piece_fallback() {
        let generator = IssueTaskGenerator::new(sample_config());

        let mut issue = sample_issue();
        issue.labels = vec!["unknown-label".to_string()];
        let task = generator.generate_task(&issue);
        assert_eq!(task.piece_name, Some("default".to_string()));
    }

    #[test]
    fn test_extract_variables() {
        let generator = IssueTaskGenerator::new(sample_config());
        let task = generator.generate_task(&sample_issue());

        assert_eq!(task.variables.get("issue_number").unwrap(), "42");
        assert_eq!(
            task.variables.get("issue_title").unwrap(),
            "Login page CSS is broken"
        );
        assert!(task.variables.get("labels").unwrap().contains("bug"));
    }

    #[test]
    fn test_format_result_comment_success() {
        let generator = IssueTaskGenerator::new(sample_config());
        let result = IssueResult {
            issue_number: 42,
            success: true,
            summary: "Fixed the CSS overlap".to_string(),
            details: Some("Adjusted flexbox layout".to_string()),
            files_changed: vec!["src/login.css".to_string()],
        };

        let comment = generator.format_result_comment(&result);
        assert!(comment.contains("white_check_mark"));
        assert!(comment.contains("Fixed the CSS overlap"));
        assert!(comment.contains("src/login.css"));
        assert!(comment.contains("ccswarm"));
    }

    #[test]
    fn test_format_result_comment_failure() {
        let generator = IssueTaskGenerator::new(sample_config());
        let result = IssueResult {
            issue_number: 42,
            success: false,
            summary: "Could not reproduce the issue".to_string(),
            details: None,
            files_changed: vec![],
        };

        let comment = generator.format_result_comment(&result);
        assert!(comment.contains(":x:"));
        assert!(comment.contains("encountered issues"));
    }

    #[test]
    fn test_parse_gh_issue() {
        let json = r#"{
            "number": 10,
            "title": "Add dark mode",
            "body": "Need dark mode support",
            "url": "https://github.com/org/repo/issues/10",
            "labels": [{"name": "feature"}, {"name": "ui"}],
            "assignees": [{"login": "dev1"}],
            "repository": {"nameWithOwner": "org/repo"}
        }"#;

        let issue = parse_gh_issue(json).unwrap();
        assert_eq!(issue.number, 10);
        assert_eq!(issue.title, "Add dark mode");
        assert_eq!(issue.labels, vec!["feature", "ui"]);
        assert_eq!(issue.assignees, vec!["dev1"]);
    }
}
