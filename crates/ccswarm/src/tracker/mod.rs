pub mod github;
pub mod linear;

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
    fn name(&self) -> &'static str;
    async fn fetch_issue(&self, identifier: &str) -> Result<NormalizedIssue>;
    async fn list_active(&self, limit: usize) -> Result<Vec<NormalizedIssue>>;
}

pub fn resolve_tracker(name: &str) -> Result<Box<dyn TrackerAdapter>> {
    match name {
        "github" => Ok(Box::new(github::GitHubAdapter)),
        "linear" => Ok(Box::new(linear::LinearAdapter)),
        other => Err(anyhow::anyhow!(
            "unknown tracker: {} (supported: github, linear)",
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
    fn test_resolve_tracker_default_is_github() -> Result<()> {
        let tracker = resolve_tracker("github")?;

        assert_eq!(tracker.name(), "github");
        Ok(())
    }

    #[test]
    fn test_resolve_tracker_unknown_returns_error() {
        let err = match resolve_tracker("unknown") {
            Ok(tracker) => panic!("unknown tracker should fail, got {}", tracker.name()),
            Err(err) => err,
        };

        assert_eq!(
            err.to_string(),
            "unknown tracker: unknown (supported: github, linear)"
        );
    }

    #[tokio::test]
    async fn test_linear_adapter_returns_not_implemented() {
        let tracker = linear::LinearAdapter;
        let err = tracker
            .fetch_issue("ENG-123")
            .await
            .expect_err("linear fetch should not be implemented yet");

        assert_eq!(
            err.to_string(),
            "LinearAdapter not yet implemented; set CCSWARM_TRACKER=github"
        );
    }
}
