//! Smoke tests for the provider layer.
//!
//! We do NOT spawn real Claude / Codex / Copilot subprocesses here — that would require
//! live credentials and would hit a provider quota every CI run. Instead we:
//! 1. Verify each provider constructs a `tokio::process::Command` with the expected
//!    program + argv pattern.
//! 2. Verify the kind round-trips through `ProviderKind::parse`.
//! 3. Verify `ProviderKind::Copilot` produces a command that exits non-zero with the
//!    friendly error, because `gh copilot suggest` cannot generate code.

use super::{ProviderKind, ProviderOptions, WorkspaceAccess, resolve};
use std::path::Path;

/// Value following `--sandbox`, if present.
fn sandbox_of(argv: &[String]) -> Option<&str> {
    argv.iter()
        .position(|a| a == "--sandbox")
        .and_then(|i| argv.get(i + 1))
        .map(String::as_str)
}

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
        max_budget: Some(0.25),
        worktree_name: Some("wt-1".to_string()),
        claude_stream_json: false,
        codex_json: false,
        workspace_access: WorkspaceAccess::Write,
    };
    let cmd = provider.build_command("do the thing", Path::new("/tmp"), &opts);
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "claude");
    assert!(argv.iter().any(|a| a == "-p"));
    assert!(argv.iter().any(|a| a == "do the thing"));
    assert!(argv.iter().any(|a| a == "--dangerously-skip-permissions"));
    assert!(argv.iter().any(|a| a == "--agent"));
    assert!(argv.iter().any(|a| a == "frontend-specialist"));
    assert!(argv.iter().any(|a| a == "--allowed-tools"));
    assert!(argv.iter().any(|a| a == "Read,Bash"));
    assert!(argv.iter().any(|a| a == "--model"));
    assert!(argv.iter().any(|a| a == "sonnet"));
    assert!(argv.iter().any(|a| a == "--append-system-prompt"));
    assert!(argv.iter().any(|a| a == "be careful"));
    // Sanity: must NOT use --system-prompt (would replace Claude Code's default).
    assert!(!argv.iter().any(|a| a == "--system-prompt"));
    assert!(argv.iter().any(|a| a == "--max-budget-usd"));
    assert!(argv.iter().any(|a| a == "--worktree"));
    assert!(argv.iter().any(|a| a == "wt-1"));
}

#[test]
fn claude_command_switches_to_stream_json_when_requested() {
    let provider = resolve(ProviderKind::Claude);
    let opts = ProviderOptions {
        claude_stream_json: true,
        ..Default::default()
    };
    let cmd = provider.build_command("hi", Path::new("/tmp"), &opts);
    let argv = argv_of(&cmd);

    assert!(argv.iter().any(|a| a == "stream-json"));
    // stream-json without --verbose only emits the final envelope; assert we
    // pair them so the bridge can actually see per-event lines.
    assert!(argv.iter().any(|a| a == "--verbose"));
    assert!(!argv.iter().any(|a| a == "text"));
}

#[test]
fn codex_command_merges_system_prompt() {
    let provider = resolve(ProviderKind::Codex);
    let opts = ProviderOptions {
        model: Some("gpt-5".to_string()),
        system_prompt: Some("SYS".to_string()),
        ..Default::default()
    };
    let cmd = provider.build_command("USER", Path::new("/tmp"), &opts);
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "codex");
    assert_eq!(argv[1], "exec");
    assert_eq!(sandbox_of(&argv), Some("workspace-write"));
    assert!(argv.iter().any(|a| a == "--skip-git-repo-check"));
    // Merged prompt is the last arg.
    let last = argv.last().expect("argv has last");
    assert!(last.contains("SYS"));
    assert!(last.contains("USER"));
    assert!(argv.iter().any(|a| a == "--model"));
    assert!(argv.iter().any(|a| a == "gpt-5"));
    assert!(!argv.iter().any(|a| a == "--json"));
}

#[test]
fn codex_read_only_access_uses_read_only_sandbox_whatever_the_tools() {
    let provider = resolve(ProviderKind::Codex);
    // Write and exec tools in the list must not widen a read-only request.
    let opts = ProviderOptions {
        allowed_tools: ["read", "bash", "edit"].map(String::from).to_vec(),
        workspace_access: WorkspaceAccess::ReadOnly,
        ..Default::default()
    };
    let argv = argv_of(&provider.build_command("review", Path::new("/tmp"), &opts));

    assert_eq!(sandbox_of(&argv), Some("read-only"));
    assert!(!argv.iter().any(|a| a == "workspace-write"));
    assert!(
        !argv
            .iter()
            .any(|a| a == "--dangerously-bypass-approvals-and-sandbox")
    );
}

#[test]
fn only_codex_enforces_read_only() {
    assert!(ProviderKind::Codex.enforces_read_only());
    assert!(!ProviderKind::Claude.enforces_read_only());
    assert!(!ProviderKind::Copilot.enforces_read_only());
}

#[test]
fn copilot_provider_emits_error_command() {
    // `gh copilot suggest` is interactive, so the provider fails fast instead.
    let provider = resolve(ProviderKind::Copilot);
    let cmd = provider.build_command("anything", Path::new("/tmp"), &ProviderOptions::default());
    let argv = argv_of(&cmd);

    assert_eq!(argv[0], "sh");
    assert_eq!(argv[1], "-c");
    assert!(argv[2].contains("not supported"));
    assert!(argv[2].contains("exit 2"));
}
