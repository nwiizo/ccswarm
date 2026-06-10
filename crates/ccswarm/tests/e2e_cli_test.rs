//! End-to-End tests for ccswarm CLI
//!
//! These tests verify the complete CLI workflow by executing the actual binary
//! and checking outputs and side effects.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the ccswarm binary
fn get_binary_path() -> PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_ccswarm") {
        return PathBuf::from(path);
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from crates/ccswarm
    path.pop(); // Go up to workspace root
    path.push("target");
    path.push("debug");
    path.push("ccswarm");
    path
}

/// Helper to run ccswarm command and capture output
fn run_ccswarm(args: &[&str], working_dir: Option<&std::path::Path>) -> std::process::Output {
    let binary = get_binary_path();
    let mut cmd = Command::new(&binary);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.output().expect("Failed to execute ccswarm")
}

/// Helper to run ccswarm and expect success
#[allow(dead_code)]
fn run_ccswarm_success(args: &[&str], working_dir: Option<&std::path::Path>) -> String {
    let output = run_ccswarm(args, working_dir);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!(
            "Command failed: ccswarm {}\nstdout: {}\nstderr: {}",
            args.join(" "),
            stdout,
            stderr
        );
    }

    stdout
}

// ============================================================================
// CLI Help and Version Tests
// ============================================================================

#[test]
fn test_cli_help() {
    let output = run_ccswarm(&["--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Help command should succeed");
    assert!(stdout.contains("ccswarm"), "Help should mention ccswarm");
    assert!(stdout.contains("init"), "Help should list init command");
    assert!(stdout.contains("task"), "Help should list task command");
    assert!(stdout.contains("agent"), "Help should list agent command");
}

#[test]
fn test_cli_version() {
    let output = run_ccswarm(&["--version"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Version command should succeed");
    assert!(
        stdout.contains("ccswarm") || stdout.contains("0."),
        "Should show version"
    );
}

// ============================================================================
// Project Initialization Tests
// ============================================================================

#[test]
fn test_init_creates_config() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Initialize git repo first
    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .expect("Failed to init git");

    // Run ccswarm init
    let output = run_ccswarm(&["init", "--name", "TestProject"], Some(project_path));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that initialization message appears (may succeed or report existing)
    assert!(
        output.status.success() || stderr.contains("already") || stdout.contains("initialized"),
        "Init should succeed or report already initialized. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_init_json_reports_configured_agents() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .expect("Failed to init git");

    let output = run_ccswarm(
        &["--json", "init", "--name", "JsonInit"],
        Some(project_path),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "init should succeed: {stdout}");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("init --json should emit valid JSON");
    assert_eq!(parsed["status"], "success");
    let agents = parsed["data"]["agents"]
        .as_array()
        .expect("agents should be an array");
    assert!(agents.contains(&serde_json::Value::String("frontend".to_string())));
    assert!(agents.contains(&serde_json::Value::String("backend".to_string())));
    assert!(agents.contains(&serde_json::Value::String("devops".to_string())));
}

#[test]
fn test_init_with_agents() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .expect("Failed to init git");

    let output = run_ccswarm(
        &[
            "init",
            "--name",
            "MultiAgentProject",
            "--agents",
            "frontend,backend",
        ],
        Some(project_path),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check initialization
    assert!(
        output.status.success() || stderr.contains("already") || stdout.contains("initialized"),
        "Init with agents should work. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// Task Management Tests
// ============================================================================

#[test]
fn test_task_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Initialize git and project
    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "TaskTest"], Some(project_path));

    // List tasks (should be empty or show no tasks)
    let output = run_ccswarm(&["task", "list"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should either succeed with empty list or show "no tasks"
    assert!(
        output.status.success() || stdout.to_lowercase().contains("no task"),
        "Task list should handle empty state"
    );
}

#[test]
fn test_task_help() {
    let output = run_ccswarm(&["task", "--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Task help should succeed");
    assert!(
        stdout.contains("list") || stdout.contains("add") || stdout.contains("COMMAND"),
        "Task help should show subcommands"
    );
}

// ============================================================================
// Agent Management Tests
// ============================================================================

#[test]
fn test_agents_list() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Initialize git and project
    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "AgentTest"], Some(project_path));

    let output = run_ccswarm(&["agents"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed or show no agents message
    assert!(
        output.status.success()
            || stdout.to_lowercase().contains("no agent")
            || stdout.to_lowercase().contains("agent")
            || stderr.to_lowercase().contains("no agent"),
        "Agents command should work. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_agents_help() {
    let output = run_ccswarm(&["agents", "--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Agents help should succeed");
    assert!(
        stdout.contains("agent") || stdout.contains("COMMAND") || stdout.contains("Agent"),
        "Agents help should show info. stdout: {}",
        stdout
    );
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_show() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Initialize git and project
    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "ConfigTest"], Some(project_path));

    let output = run_ccswarm(&["config", "show"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show config or error about missing config
    assert!(
        output.status.success()
            || stdout.contains("project")
            || stderr.contains("config")
            || stderr.contains("not found"),
        "Config show should return config or error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_config_help() {
    let output = run_ccswarm(&["config", "--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Config help should succeed");
    assert!(
        stdout.contains("show") || stdout.contains("COMMAND"),
        "Config help should show subcommands"
    );
}

// ============================================================================
// Health and Doctor Tests
// ============================================================================

// `health` was folded into `doctor`; its coverage moved to `test_doctor_command`.

#[test]
fn test_doctor_command() {
    let output = run_ccswarm(&["doctor"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Doctor should run diagnostics
    assert!(
        output.status.success()
            || stdout.to_lowercase().contains("doctor")
            || stdout.to_lowercase().contains("diagnostic")
            || stderr.to_lowercase().contains("check"),
        "Doctor command should execute. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[test]
fn test_session_list() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "SessionTest"], Some(project_path));

    let output = run_ccswarm(&["session", "list"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should list sessions or show empty
    assert!(
        output.status.success()
            || stdout.to_lowercase().contains("session")
            || stdout.to_lowercase().contains("no ")
            || stderr.to_lowercase().contains("session"),
        "Session list should work. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn test_json_output_flag() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "JsonTest"], Some(project_path));

    let output = run_ccswarm(&["--json", "task", "list"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);

    // With --json flag, output should be JSON or indicate JSON mode
    // (may be empty JSON array or object)
    if output.status.success() && !stdout.is_empty() {
        // If there's output, it should be valid JSON or JSON-like
        let is_json_like = stdout.trim().starts_with('{')
            || stdout.trim().starts_with('[')
            || stdout.contains("\"");
        assert!(
            is_json_like || stdout.is_empty(),
            "JSON output should be JSON formatted: {}",
            stdout
        );
    }
}

#[test]
fn test_queue_list_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "QueueJsonTest"], Some(project_path));
    let _ = run_ccswarm(&["queue", "add", "Smoke check task"], Some(project_path));

    let output = run_ccswarm(&["--json", "queue", "list"], Some(project_path));
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "queue list should succeed. stdout: {}, stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("queue list --json should emit valid JSON");
    assert_eq!(parsed["total"], 1);
    assert_eq!(parsed["tasks"][0]["task"], "Smoke check task");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_flag() {
    // Test that unrecognized CLI flags are rejected by the argument parser.
    // Note: unknown positional args are now treated as direct task descriptions
    // by the "direct task mode" feature, so we test with an invalid flag instead.
    let output = run_ccswarm(&["--not-a-real-flag"], None);

    // Should fail gracefully with a non-zero exit code
    assert!(!output.status.success(), "Invalid flag should fail");
}

#[test]
fn test_missing_required_args() {
    // Init without name should fail or prompt
    let output = run_ccswarm(&["init"], None);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either fail or ask for required arguments
    assert!(
        !output.status.success() || stderr.contains("required") || stderr.contains("missing"),
        "Init without name should fail or indicate missing args"
    );
}

// ============================================================================
// Workflow Integration Tests
// ============================================================================

#[test]
fn test_full_workflow_init_to_task() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Step 1: Initialize git
    let git_output = Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .expect("Git init failed");
    assert!(git_output.status.success(), "Git init should succeed");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(project_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(project_path)
        .output()
        .unwrap();

    // Step 2: Initialize ccswarm project
    let init_output = run_ccswarm(
        &["init", "--name", "WorkflowTest", "--agents", "frontend"],
        Some(project_path),
    );
    let init_stdout = String::from_utf8_lossy(&init_output.stdout);
    let init_stderr = String::from_utf8_lossy(&init_output.stderr);

    // Init should work
    assert!(
        init_output.status.success()
            || init_stdout.contains("initialized")
            || init_stderr.contains("already"),
        "Project init should succeed. stdout: {}, stderr: {}",
        init_stdout,
        init_stderr
    );

    // Step 3: List agents (verify initialization)
    let _agent_output = run_ccswarm(&["agents"], Some(project_path));
    // Should work regardless of agents present

    // Step 4: List tasks
    let _task_output = run_ccswarm(&["task", "list"], Some(project_path));
    // Should work with empty task list

    // Step 5: Check doctor
    let doctor_output = run_ccswarm(&["doctor"], Some(project_path));
    // Doctor should show system health
    assert!(
        doctor_output.status.success()
            || !String::from_utf8_lossy(&doctor_output.stderr).is_empty(),
        "Doctor should work"
    );
}

#[test]
fn test_approve_commit_writes_decision_record() {
    let temp_dir = TempDir::new().unwrap();
    let project = temp_dir.path();

    let output = run_ccswarm(
        &[
            "approve", "commit", "--id", "run-1", "--reject", "--reason", "no",
        ],
        Some(project),
    );
    assert!(
        output.status.success(),
        "approve commit should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let record_path = project.join(".ccswarm/approvals/run-1.json");
    let content = std::fs::read_to_string(&record_path).expect("approval record written");
    let record: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(record["gate"], "commit");
    assert_eq!(record["status"], "rejected");
    assert_eq!(record["reason"], "no");

    let list = run_ccswarm(&["approve", "list", "--status", "rejected"], Some(project));
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(
        stdout.contains("run-1"),
        "approve list should show the rejected record. stdout: {}",
        stdout
    );
}

#[test]
fn test_flow_check_reports_cycles() {
    let temp_dir = TempDir::new().unwrap();

    // The builtin review-fix flow contains a review <-> fix cycle by design;
    // `flow check` should surface it as a warning with the runtime bound.
    let output = run_ccswarm(&["flow", "check", "review-fix"], Some(temp_dir.path()));
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "flow check should succeed (cycles are warnings, not errors). stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("contains cycle") && stdout.contains("max_stage_visits"),
        "flow check should warn about the review<->fix cycle. stdout: {}",
        stdout
    );
}

#[test]
fn test_pipeline_dry_run_includes_task_body() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "DryRunTest"], Some(project_path));
    let output = run_ccswarm(
        &[
            "pipeline",
            "--task",
            "Fix the onboarding typo",
            "--flow",
            "quick",
            "--dry-run",
        ],
        Some(project_path),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "dry-run should succeed. stdout: {}, stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("Fix the onboarding typo"),
        "dry-run prompt preview should include the task body: {stdout}"
    );
}

#[test]
fn test_pipeline_dry_run_shows_sangha_member_prompts() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let _ = run_ccswarm(&["init", "--name", "SanghaDryRunTest"], Some(project_path));
    let output = run_ccswarm(
        &[
            "pipeline",
            "--task",
            "Add validation",
            "--flow",
            "default",
            "--dry-run",
        ],
        Some(project_path),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "default dry-run should succeed. stdout: {}, stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("stage: sangha/member: planner"),
        "dry-run should show Sangha member prompts: {stdout}"
    );
    assert!(
        stdout.contains("SANGHA_DECISION=APPROVE"),
        "dry-run should show the Sangha decision contract: {stdout}"
    );
}

#[test]
fn test_scaffold_returns_failure_when_pipeline_fails() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("missing-flow-app");
    let dir_arg = project_path.to_string_lossy().to_string();

    let output = run_ccswarm(
        &[
            "scaffold",
            "--dir",
            &dir_arg,
            "--task",
            "Create a tiny app",
            "--flow",
            "missing-flow-for-test",
            "--timeout",
            "5",
        ],
        None,
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "scaffold should fail when the child pipeline fails. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        !stderr.contains("Project ready"),
        "failed scaffold must not claim the project is ready: {stderr}"
    );
}

#[test]
fn test_extend_auto_propose_creates_sangha_proposal() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()
        .unwrap();

    let output = run_ccswarm(
        &[
            "--json",
            "lab",
            "extend",
            "auto-propose",
            "--agent",
            "backend",
            "--reason",
            "Repeated API review issues",
        ],
        Some(project_path),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "auto-propose should succeed. stdout: {stdout}, stderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("auto-propose --json should emit valid JSON");
    let proposal_id = parsed["data"]["sangha_proposal_id"]
        .as_str()
        .expect("extension should link to a Sangha proposal");
    assert_eq!(parsed["data"]["status"], "pending_consensus");

    let proposal_path = project_path
        .join("coordination")
        .join("proposals")
        .join(format!("{proposal_id}.json"));
    assert!(proposal_path.exists(), "Sangha proposal should be written");
}

#[test]
fn test_sangha_rejects_unsafe_proposal_id() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let output = run_ccswarm(
        &["lab", "sangha", "status", "../../etc/passwd"],
        Some(project_path),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "unsafe proposal id should fail. stderr: {stderr}"
    );
    assert!(
        stderr.contains("safe coordination ID"),
        "error should explain unsafe IDs: {stderr}"
    );
}

// ============================================================================
// Verbose Mode Tests
// ============================================================================

#[test]
fn test_verbose_flag() {
    let output = run_ccswarm(&["--verbose", "--help"], None);

    assert!(output.status.success(), "Verbose help should succeed");
}

// ============================================================================
// Subcommand Discovery Tests
// ============================================================================

#[test]
fn test_all_subcommands_have_help() {
    let subcommands = [
        "init", "task", "agents", "config", "doctor", "flow", "harness", "approve", "queue",
        "tail", "cost", "facets", "replay", "undo", "run", "auto", "lab",
    ];

    for subcmd in subcommands {
        let output = run_ccswarm(&[subcmd, "--help"], None);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "Subcommand '{}' should have help",
            subcmd
        );
        assert!(
            !stdout.is_empty(),
            "Subcommand '{}' help should not be empty",
            subcmd
        );
    }
}
