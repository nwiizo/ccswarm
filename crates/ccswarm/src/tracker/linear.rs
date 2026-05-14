use super::{NormalizedIssue, TrackerAdapter};
use anyhow::{Result, anyhow};

pub struct LinearAdapter;

#[async_trait::async_trait]
impl TrackerAdapter for LinearAdapter {
    fn name(&self) -> &'static str {
        "linear"
    }

    async fn fetch_issue(&self, _: &str) -> Result<NormalizedIssue> {
        Err(anyhow!(
            "LinearAdapter not yet implemented; set CCSWARM_TRACKER=github"
        ))
    }

    async fn list_active(&self, _: usize) -> Result<Vec<NormalizedIssue>> {
        Err(anyhow!("LinearAdapter not yet implemented"))
    }
}
