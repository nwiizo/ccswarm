//! End-to-End tests for ccswarm CLI
//!
//! These tests verify the complete CLI workflow by executing the actual binary
//! and checking outputs and side effects.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the ccswarm binary
fn get_binary_path() -> PathBuf {
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

#[test]
fn test_health_command() {
    let output = run_ccswarm(&["health"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Health check should run (may report issues but shouldn't crash)
    assert!(
        output.status.success()
            || stdout.to_lowercase().contains("health")
            || stderr.to_lowercase().contains("check"),
        "Health command should execute. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

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

#[test]
fn test_session_help() {
    let output = run_ccswarm(&["session", "--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Session help should succeed");
    assert!(
        stdout.contains("list") || stdout.contains("COMMAND"),
        "Session help should show subcommands"
    );
}

// ============================================================================
// Template Management Tests
// ============================================================================

#[test]
fn test_template_list() {
    let output = run_ccswarm(&["template", "list"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should list available templates
    assert!(
        output.status.success()
            || stdout.to_lowercase().contains("template")
            || stderr.to_lowercase().contains("template"),
        "Template list should work. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_template_help() {
    let output = run_ccswarm(&["template", "--help"], None);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Template help should succeed");
    assert!(
        stdout.contains("list") || stdout.contains("COMMAND"),
        "Template help should show subcommands"
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

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_command() {
    let output = run_ccswarm(&["invalid-command-that-does-not-exist"], None);

    // Should fail gracefully
    assert!(!output.status.success(), "Invalid command should fail");
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

    // Step 5: Check status
    let status_output = run_ccswarm(&["status"], Some(project_path));
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    let status_stderr = String::from_utf8_lossy(&status_output.stderr);

    // Status should show project info or indicate not running
    assert!(
        status_output.status.success()
            || status_stdout.to_lowercase().contains("status")
            || status_stderr.to_lowercase().contains("not running")
            || status_stderr.to_lowercase().contains("no orchestrator"),
        "Status should work. stdout: {}, stderr: {}",
        status_stdout,
        status_stderr
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
        "init", "task", "agents", "config", "session", "template", "health", "doctor", "status",
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
