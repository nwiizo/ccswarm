//! BDD test harness for ccswarm's business-level behaviors.
//!
//! # Why BDD here
//!
//! Unit tests cover individual functions; integration tests cover end-to-end CLI
//! invocations. Neither reads naturally as "what ccswarm should do for users".
//! These cucumber features are written in declarative Gherkin:
//!
//! - What, not how. "When I queue a task", not "When I click #queue-add-button".
//! - Ubiquitous language. We use the domain words users see: *flow*, *stage*,
//!   *run*, *queue*, *drain*. No internal types leak into scenarios.
//! - One scenario = one business behavior. When a behavior breaks, exactly one
//!   scenario fails and its name tells you what broke.
//!
//! # Scope
//!
//! BDD here tests the *behaviors the user commits to*, not the provider network
//! layer. We never spawn `claude` / `codex` in these tests — provider execution
//! is covered separately by `tests/e2e_cli_test.rs` and the live verification
//! runs. This keeps BDD fast, hermetic, and usable in CI without API keys.

use cucumber::{World, given, then, when};
use std::path::PathBuf;
use tempfile::TempDir;

mod support;
use support::*;

#[derive(Debug, Default, World)]
pub struct CcswarmWorld {
    workspace: Option<TempDir>,
    task_file: Option<PathBuf>,
    last_error: Option<String>,
    last_suggest: Option<&'static str>,
    last_validation_error: Option<String>,
    last_provider_kind: Option<&'static str>,
    env_provider: Option<String>,
}

impl CcswarmWorld {
    fn repo(&self) -> PathBuf {
        self.workspace
            .as_ref()
            .expect("workspace set up in Background")
            .path()
            .to_path_buf()
    }
}

// ── Queue feature steps ─────────────────────────────────────────────────────

#[given("a ccswarm workspace")]
fn a_ccswarm_workspace(w: &mut CcswarmWorld) {
    w.workspace = Some(TempDir::new().expect("create tempdir"));
}

#[given(regex = r"^a task description file with content \x22(.+)\x22$")]
fn a_task_description_file(w: &mut CcswarmWorld, content: String) {
    let dir = w.workspace.as_ref().expect("workspace");
    let path = dir.path().join("task.md");
    std::fs::write(&path, content).unwrap();
    w.task_file = Some(path);
}

#[given(regex = r#"^a queued task "(.+)"$"#)]
fn a_queued_task(w: &mut CcswarmWorld, body: String) {
    queue_add_inline(&w.repo(), &body).expect("inline queue add");
}

#[when(regex = r#"^I queue the task "(.+)"$"#)]
fn i_queue_the_task(w: &mut CcswarmWorld, body: String) {
    queue_add_inline(&w.repo(), &body).expect("inline queue add");
}

#[when("I queue the task from that file")]
fn i_queue_from_file(w: &mut CcswarmWorld) {
    let file = w.task_file.as_ref().expect("task file in Given");
    queue_add_file(&w.repo(), file).expect("file queue add");
}

#[when(regex = r#"^I queue the task from stdin with content "(.+)"$"#)]
fn i_queue_from_stdin(w: &mut CcswarmWorld, content: String) {
    queue_add_stdin(&w.repo(), &content).expect("stdin queue add");
}

#[when("I clear the queue")]
fn i_clear_the_queue(w: &mut CcswarmWorld) {
    queue_clear(&w.repo()).expect("clear");
}

#[when("I queue a task with both --file and a positional argument")]
fn i_queue_task_with_both(w: &mut CcswarmWorld) {
    let dir = w.workspace.as_ref().expect("workspace");
    let file = dir.path().join("task.md");
    std::fs::write(&file, "from-file").unwrap();
    let err = queue_add_both_sources(&w.repo(), "from-positional", &file).unwrap_err();
    w.last_error = Some(err.to_string());
}

#[then(regex = r"^the queue shows (\d+) pending tasks?$")]
fn the_queue_shows_n_pending(w: &mut CcswarmWorld, n: usize) {
    let q = load_queue(&w.repo()).expect("load queue");
    let pending = q.tasks.iter().filter(|t| t.state == "pending").count();
    assert_eq!(pending, n, "expected {n} pending tasks");
}

#[then(regex = r#"^the most recent queued task mentions "(.+)"$"#)]
fn the_most_recent_queued_task(w: &mut CcswarmWorld, needle: String) {
    let q = load_queue(&w.repo()).expect("load queue");
    let last = q.tasks.last().expect("queue has at least one task");
    assert!(
        last.task.contains(&needle),
        "task body {:?} did not contain {:?}",
        last.task,
        needle
    );
}

#[then("queuing fails with a helpful error")]
fn queuing_fails(w: &mut CcswarmWorld) {
    let err = w.last_error.as_deref().expect("expected an error");
    assert!(
        err.contains("choose only one"),
        "error message {:?} did not mention mutually exclusive sources",
        err
    );
}

// ── Flow suggest feature steps ──────────────────────────────────────────────

#[when(regex = r#"^I ask ccswarm to suggest a flow for "(.+)"$"#)]
fn i_ask_suggest(w: &mut CcswarmWorld, task: String) {
    let (flow, _) = ccswarm::bdd_api::suggest_flow_for_task(&task);
    w.last_suggest = Some(flow);
}

#[then(regex = r#"^the suggested flow is "(.+)"$"#)]
fn the_suggested_flow_is(w: &mut CcswarmWorld, expected: String) {
    let actual = w.last_suggest.expect("Suggest step must run first");
    assert_eq!(actual, expected.as_str(), "suggested flow mismatch");
}

// ── Run safety feature steps ────────────────────────────────────────────────

#[given("a ccswarm workspace with a runs directory")]
fn a_workspace_with_runs(w: &mut CcswarmWorld) {
    w.workspace = Some(TempDir::new().expect("tempdir"));
    let runs_dir = w.repo().join(".ccswarm").join("runs");
    std::fs::create_dir_all(&runs_dir).unwrap();
}

#[when(regex = r#"^I ask for details of run "(.+)"$"#)]
fn i_ask_for_run(w: &mut CcswarmWorld, id: String) {
    w.last_validation_error = ccswarm::bdd_api::validate_run_id(&id)
        .err()
        .map(|e| e.to_string());
}

#[then(regex = r#"^the request is rejected with "(.+)"$"#)]
fn the_request_is_rejected(w: &mut CcswarmWorld, needle: String) {
    let err = w
        .last_validation_error
        .as_deref()
        .expect("expected validation to fail");
    assert!(
        err.contains(&needle),
        "validation error {:?} did not contain {:?}",
        err,
        needle
    );
}

#[then("the run ID passes validation")]
fn the_run_id_passes(w: &mut CcswarmWorld) {
    assert!(
        w.last_validation_error.is_none(),
        "expected validation to pass, got {:?}",
        w.last_validation_error
    );
}

// ── Providers feature steps ─────────────────────────────────────────────────

#[given(regex = r#"^the environment variable CCSWARM_PROVIDER is "(.+)"$"#)]
fn env_ccswarm_provider_is(w: &mut CcswarmWorld, value: String) {
    w.env_provider = Some(value);
}

#[when("I resolve the provider for a flow that doesn't specify one")]
fn resolve_no_stage_provider(w: &mut CcswarmWorld) {
    let kind = resolve_provider(None, w.env_provider.as_deref());
    w.last_provider_kind = Some(kind);
}

#[when(regex = r#"^I resolve the provider for a flow stage declaring "(.+)"$"#)]
fn resolve_with_stage_provider(w: &mut CcswarmWorld, stage_provider: String) {
    let kind = resolve_provider(Some(&stage_provider), w.env_provider.as_deref());
    w.last_provider_kind = Some(kind);
}

#[then(regex = r#"^the resolved provider is "(.+)"$"#)]
fn the_resolved_provider_is(w: &mut CcswarmWorld, expected: String) {
    let actual = w
        .last_provider_kind
        .expect("resolution step must run first");
    assert_eq!(actual, expected.as_str(), "provider resolution mismatch");
}

#[when(regex = r#"^I parse provider name "(.+)"$"#)]
fn i_parse_provider_name(w: &mut CcswarmWorld, input: String) {
    w.last_provider_kind = Some(parse_provider_kind(&input).expect("unknown provider alias"));
}

#[then(regex = r#"^the provider kind is "(.+)"$"#)]
fn the_provider_kind_is(w: &mut CcswarmWorld, expected: String) {
    let actual = w.last_provider_kind.expect("parse step must run first");
    assert_eq!(actual, expected.as_str(), "provider kind mismatch");
}

fn main() {
    futures::executor::block_on(
        CcswarmWorld::cucumber()
            .fail_on_skipped()
            .run_and_exit("tests/bdd/features"),
    );
}
