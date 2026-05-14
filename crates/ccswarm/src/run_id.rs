//! Run ID validation shared by CLI handlers and event recording.

use anyhow::{Result, anyhow};

/// Validate that a run ID only contains characters allowed in UUIDs / short hex.
///
/// Rejects anything that could escape `.ccswarm/runs/` via `../`, embedded path
/// separators, or null bytes.
pub fn validate_run_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(anyhow!("run ID is empty"));
    }
    if id.len() > 128 {
        return Err(anyhow!("run ID is suspiciously long ({} bytes)", id.len()));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "run ID '{}' contains illegal characters (allowed: [A-Za-z0-9_-])",
            id
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_run_id("../etc/passwd").is_err());
        assert!(validate_run_id("..").is_err());
        assert!(validate_run_id("a/b").is_err());
        assert!(validate_run_id("a\\b").is_err());
        assert!(validate_run_id("a\0b").is_err());
    }

    #[test]
    fn accepts_uuid_ish() {
        assert!(validate_run_id("97fc1ccd-ea1b-4119-9a3c-2c1a550d71ee").is_ok());
        assert!(validate_run_id("abc123").is_ok());
        assert!(validate_run_id("run_42").is_ok());
    }

    #[test]
    fn rejects_empty_and_long() {
        assert!(validate_run_id("").is_err());
        assert!(validate_run_id(&"a".repeat(200)).is_err());
    }
}
