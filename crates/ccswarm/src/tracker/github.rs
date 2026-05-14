use super::{BlockedBy, NormalizedIssue, TrackerAdapter};
use anyhow::{Context, Result, anyhow};
use chrono::DateTime;
use serde::Deserialize;

pub struct GitHubAdapter;

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    #[serde(default)]
    body: Option<String>,
    state: String,
    #[serde(default)]
    labels: Vec<GitHubLabel>,
    url: String,
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

impl GitHubIssue {
    fn into_normalized(self) -> Result<NormalizedIssue> {
        if self.title.is_empty() {
            return Err(anyhow!("Issue #{} returned no title", self.number));
        }

        let updated_at_ms = self
            .updated_at
            .as_deref()
            .map(parse_updated_at_ms)
            .transpose()?;

        Ok(NormalizedIssue {
            id: self.number.to_string(),
            identifier: self.number.to_string(),
            title: self.title,
            description: self.body,
            state: self.state,
            labels: self
                .labels
                .into_iter()
                .map(|label| label.name.to_ascii_lowercase())
                .collect(),
            blocked_by: Vec::<BlockedBy>::new(),
            url: self.url,
            priority: None,
            updated_at_ms,
        })
    }
}

fn parse_updated_at_ms(value: &str) -> Result<u64> {
    let timestamp_ms = DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("Failed to parse GitHub updatedAt timestamp `{value}`"))?
        .timestamp_millis();

    u64::try_from(timestamp_ms).context("GitHub updatedAt timestamp is before Unix epoch")
}

#[async_trait::async_trait]
impl TrackerAdapter for GitHubAdapter {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn fetch_issue(&self, identifier: &str) -> Result<NormalizedIssue> {
        let output = tokio::process::Command::new("gh")
            .args([
                "issue",
                "view",
                identifier,
                "--json",
                "number,title,body,state,labels,url,updatedAt",
            ])
            .output()
            .await
            .context("Failed to run `gh issue view`. Is GitHub CLI installed and authenticated?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh issue view failed: {}", stderr));
        }

        let issue: GitHubIssue = serde_json::from_slice(&output.stdout)
            .context("Failed to parse `gh issue view` JSON")?;
        issue.into_normalized()
    }

    async fn list_active(&self, _limit: usize) -> Result<Vec<NormalizedIssue>> {
        Err(anyhow!("GitHubAdapter list_active not yet implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_issue_maps_to_normalized_issue() -> Result<()> {
        let raw = r#"{
            "number": 42,
            "title": "Fix queue ingestion",
            "body": "Preserve issue body.",
            "state": "OPEN",
            "labels": [{"name": "Bug"}, {"name": "CLI"}],
            "url": "https://github.com/example/repo/issues/42",
            "updatedAt": "1970-01-01T00:00:01Z"
        }"#;

        let issue: GitHubIssue = serde_json::from_str(raw)?;
        let normalized = issue.into_normalized()?;

        assert_eq!(normalized.id, "42");
        assert_eq!(normalized.identifier, "42");
        assert_eq!(normalized.title, "Fix queue ingestion");
        assert_eq!(
            normalized.description.as_deref(),
            Some("Preserve issue body.")
        );
        assert_eq!(normalized.state, "OPEN");
        assert_eq!(normalized.labels, vec!["bug", "cli"]);
        assert!(normalized.blocked_by.is_empty());
        assert_eq!(
            normalized.url,
            "https://github.com/example/repo/issues/42".to_string()
        );
        assert_eq!(normalized.priority, None);
        assert_eq!(normalized.updated_at_ms, Some(1_000));
        Ok(())
    }
}
