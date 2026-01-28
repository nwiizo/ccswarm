use anyhow::Result;
use ccswarm::session::{AISessionConfig, AISessionManager};
use tokio::time::{Duration, sleep};

/// Ensure ccswarm can spin up an ai-session managed shell even when PTYs are unavailable.
#[tokio::test]
async fn ccswarm_starts_ai_session_via_headless_backend() -> Result<()> {
    let manager = AISessionManager::new();
    let mut config = AISessionConfig::default();
    config.force_headless = true;
    config.allow_headless_fallback = true;
    config.shell = Some("/bin/sh".to_string());
    config.environment.clear();

    let session = manager.create_session_with_config(config).await?;
    session.start().await?;

    session.send_input("echo 'hello from ccswarm'\n").await?;
    let mut aggregated = String::new();
    for _ in 0..10 {
        sleep(Duration::from_millis(100)).await;
        let chunk = session.read_output().await?;
        if chunk.is_empty() {
            continue;
        }
        aggregated.push_str(&String::from_utf8_lossy(&chunk));
        if aggregated.contains("hello from ccswarm") {
            break;
        }
    }

    assert!(
        aggregated.contains("hello from ccswarm"),
        "expected output to contain greeting, got: {}",
        aggregated
    );

    session.stop().await?;
    Ok(())
}
