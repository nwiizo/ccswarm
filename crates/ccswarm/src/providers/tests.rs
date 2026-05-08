//! Smoke tests for the provider layer.
//!
//! We do NOT spawn real Claude / Codex / Copilot subprocesses here — that would require
//! live credentials and would hit a provider quota every CI run. Instead we:
//! 1. Verify each provider constructs a `tokio::process::Command` with the expected
//!    program + argv pattern.
//! 2. Verify the kind round-trips through `ProviderKind::parse`.
//! 3. Verify `ProviderKind::Copilot` produces a command that exits non-zero with the
//!    friendly error (codex #4 finding — `gh copilot suggest` is not usable as a code
//!    generation backend).

use super::{ProviderKind, ProviderOptions, resolve};
use std::path::Path;

fn argv_of(cmd: &tokio::process::Command) -> Vec<String> {
    let std_cmd = cmd.as_std();
    std::iter::once(std_cmd.get_program().to_string_lossy().to_string())
        .chain(std_cmd.get_args().map(|s| s.to_string_lossy().to_string()))
        .collect()
}

#[test]
fn kind_parse_round_trip() {
    for (input, expected) in [
        ("claude", ProviderKind::Claude),
        ("CLAUDE", ProviderKind::Claude),
        ("claude-code", ProviderKind::Claude),
        ("codex", ProviderKind::Codex),
        ("copilot", ProviderKind::Copilot),
        ("gh-copilot", ProviderKind::Copilot),
        ("github-copilot", ProviderKind::Copilot),
    ] {
        assert_eq!(ProviderKind::parse(input), Some(expected));
    }
    assert_eq!(ProviderKind::parse("gpt-4"), None);
    assert_eq!(ProviderKind::parse(""), None);
}

#[test]
fn claude_command_includes_expected_flags() {
    let provider = resolve(ProviderKind::Claude);
    let opts = ProviderOptions {
        allowed_tools: vec!["read".to_string(), "bash".to_string()],
        model: Some("sonnet".to_string()),
        system_prompt: Some("be careful".to_string()),
        agent_name: Some("frontend-specialist".to_string()),
        session_id: Some("abc123".to_string()),
        continue_session: false,
        max_budget: Some(0.25),
        worktree_name: Some("wt-1".to_string()),
    };
    let cmd = provider.build_command("do the thing", Path::new("/tmp"), &opts);
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "claude");
    assert!(argv.iter().any(|a| a == "-p"));
    assert!(argv.iter().any(|a| a == "do the thing"));
    assert!(argv.iter().any(|a| a == "--dangerously-skip-permissions"));
    assert!(argv.iter().any(|a| a == "--agent"));
    assert!(argv.iter().any(|a| a == "frontend-specialist"));
    assert!(argv.iter().any(|a| a == "--session-id"));
    assert!(argv.iter().any(|a| a == "abc123"));
    assert!(argv.iter().any(|a| a == "--allowed-tools"));
    assert!(argv.iter().any(|a| a == "Read,Bash"));
    assert!(argv.iter().any(|a| a == "--model"));
    assert!(argv.iter().any(|a| a == "sonnet"));
    assert!(argv.iter().any(|a| a == "--system-prompt"));
    assert!(argv.iter().any(|a| a == "be careful"));
    assert!(argv.iter().any(|a| a == "--max-budget-usd"));
    assert!(argv.iter().any(|a| a == "--worktree"));
    assert!(argv.iter().any(|a| a == "wt-1"));
}

#[test]
fn codex_command_merges_system_prompt_and_has_no_session_flag() {
    // codex #3 regression guard: Codex CLI has no `--session` flag.
    let provider = resolve(ProviderKind::Codex);
    let opts = ProviderOptions {
        model: Some("gpt-5".to_string()),
        system_prompt: Some("SYS".to_string()),
        session_id: Some("ignored".to_string()),
        ..Default::default()
    };
    let cmd = provider.build_command("USER", Path::new("/tmp"), &opts);
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "codex");
    assert_eq!(argv[1], "exec");
    assert!(argv.iter().any(|a| a == "--sandbox"));
    assert!(argv.iter().any(|a| a == "workspace-write"));
    assert!(argv.iter().any(|a| a == "--skip-git-repo-check"));
    // Merged prompt is the last arg.
    let last = argv.last().expect("argv has last");
    assert!(last.contains("SYS"));
    assert!(last.contains("USER"));
    assert!(argv.iter().any(|a| a == "--model"));
    assert!(argv.iter().any(|a| a == "gpt-5"));
    // The Codex `--session` flag does not exist; make sure we don't emit it.
    assert!(!argv.iter().any(|a| a == "--session"));
    assert!(!argv.iter().any(|a| a == "ignored"));
}

#[test]
fn copilot_provider_emits_error_command() {
    // codex #4 guard: `gh copilot suggest` is interactive and unsuitable for code gen.
    // Until GitHub ships a non-interactive CLI, the provider intentionally returns a
    // `sh -c '... exit 2'` command that fails fast with a clear message.
    let provider = resolve(ProviderKind::Copilot);
    let cmd = provider.build_command("anything", Path::new("/tmp"), &ProviderOptions::default());
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "sh");
    assert_eq!(argv[1], "-c");
    assert!(argv[2].contains("not supported"));
    assert!(argv[2].contains("exit 2"));
}
