pub mod github;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<BlockedBy>,
    pub url: String,
    #[serde(default)]
    pub priority: Option<u8>,
    #[serde(default)]
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockedBy {
    pub identifier: String,
    pub state: String,
}

#[async_trait::async_trait]
pub trait TrackerAdapter: Send + Sync {
    async fn fetch_issue(&self, identifier: &str) -> Result<NormalizedIssue>;
}

pub fn resolve_tracker(name: &str) -> Result<Box<dyn TrackerAdapter>> {
    match name {
        "github" => Ok(Box::new(github::GitHubAdapter)),
        other => Err(anyhow::anyhow!(
            "unknown tracker: {} (supported: github)",
            other
        )),
    }
}

pub fn default_tracker_name() -> String {
    std::env::var("CCSWARM_TRACKER").unwrap_or_else(|_| "github".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_issue_round_trip() -> Result<()> {
        let issue = NormalizedIssue {
            id: "issue-id".to_string(),
            identifier: "ENG-123".to_string(),
            title: "Add tracker normalization".to_string(),
            description: Some("Normalize issue data across trackers.".to_string()),
            state: "OPEN".to_string(),
            labels: vec!["backend".to_string(), "tracker".to_string()],
            blocked_by: vec![BlockedBy {
                identifier: "ENG-100".to_string(),
                state: "OPEN".to_string(),
            }],
            url: "https://linear.app/example/issue/ENG-123".to_string(),
            priority: Some(2),
            updated_at_ms: Some(1_700_000_000_000),
        };

        let serialized = serde_json::to_string(&issue)?;
        let round_tripped: NormalizedIssue = serde_json::from_str(&serialized)?;

        assert_eq!(round_tripped, issue);
        Ok(())
    }

    #[test]
    fn test_resolve_tracker_default_is_github() {
        assert!(resolve_tracker("github").is_ok());
    }

    #[test]
    fn test_resolve_tracker_unknown_returns_error() {
        let Err(err) = resolve_tracker("unknown") else {
            panic!("unknown tracker should fail");
        };

        assert_eq!(
            err.to_string(),
            "unknown tracker: unknown (supported: github)"
        );
    }
}
