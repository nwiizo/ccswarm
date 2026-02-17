//! Cross-module integration tests for the workflow engine.
//!
//! Each test exercises at least 2 workflow modules working together.
//! No external API calls; all execution is simulated.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use ccswarm::workflow::{
    ArpeggioConfig, ArpeggioExecutor, ArpeggioItem, ChangeType, CycleDetector, FacetRegistry,
    FileChange, GitHubIssue, GitHubIssueConfig, I18nManager, InteractiveAction, InteractiveMode,
    InteractiveSession, IssueResult, IssueTaskGenerator, Language, LocaleBundle, LoopStrategy,
    MatchMethod, Movement, MovementJudge, MovementPermission, MovementRule, PermissionEnforcer,
    Piece, PieceEngine, PieceStatus, PipelineConfig, PipelineExitCode, PipelineRunner,
    PipelineStatus, RuleCondition, WatchConfig, WatchController, WatchState, builtin_personas,
    builtin_pieces, builtin_policies, parse_gh_issue,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_movement(id: &str, rules: Vec<(&str, &str)>) -> Movement {
    Movement {
        id: id.to_string(),
        persona: None,
        policy: None,
        knowledge: None,
        provider: None,
        model: None,
        instruction: format!("Instruction for {}", id),
        tools: vec![],
        permission: MovementPermission::Readonly,
        rules: rules
            .into_iter()
            .map(|(cond, next)| MovementRule {
                condition: RuleCondition::Simple(cond.to_string()),
                next: next.to_string(),
                priority: 0,
            })
            .collect(),
        parallel: false,
        sub_movements: vec![],
        output_contract: None,
        timeout: None,
        max_retries: 0,
    }
}

fn sample_issue(number: u64, title: &str, labels: Vec<&str>) -> GitHubIssue {
    GitHubIssue {
        number,
        title: title.to_string(),
        body: format!("Body for issue #{}", number),
        labels: labels.into_iter().map(String::from).collect(),
        assignees: vec!["dev".to_string()],
        repository: "org/repo".to_string(),
        url: format!("https://github.com/org/repo/issues/{}", number),
    }
}

fn sample_issue_config() -> GitHubIssueConfig {
    let mut mapping = HashMap::new();
    mapping.insert("bug".to_string(), "review-fix".to_string());
    mapping.insert("feature".to_string(), "default".to_string());
    mapping.insert("research".to_string(), "research".to_string());

    GitHubIssueConfig {
        repository: "org/repo".to_string(),
        label_piece_mapping: mapping,
        post_results: true,
        close_on_success: false,
        default_piece: Some("default".to_string()),
    }
}

// ---------------------------------------------------------------------------
// 1. piece + facets + judge
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_piece_engine_faceted_execution() {
    // Build a YAML-style piece with persona + policy references
    let yaml = r#"
name: faceted-test
description: "Tests faceted prompt composition through PieceEngine"
max_movements: 10
initial_movement: plan

movements:
  - id: plan
    persona: planner
    policy: coding
    instruction: "Analyze the task and create a plan"
    tools: [read, grep]
    permission: readonly
    rules:
      - condition: success
        next: implement
  - id: implement
    persona: coder
    policy: coding
    instruction: "Implement the plan"
    tools: [read, write, edit]
    permission: edit
    rules: []
"#;
    let piece = Piece::from_yaml(yaml).expect("parse failed");

    let mut engine = PieceEngine::new();
    engine.register_piece(piece);

    // Execute — PieceEngine internally composes faceted prompts and routes via judge
    let state = engine
        .execute_piece("faceted-test")
        .await
        .expect("execution failed");

    assert_eq!(state.status, PieceStatus::Completed);
    assert_eq!(state.movement_count, 2);

    // Verify that movement outputs were stored as variables
    assert!(state.variables.contains_key("plan_output"));
    assert!(state.variables.contains_key("implement_output"));

    // The plan_output should contain the composed prompt (includes persona + policy tags)
    let plan_out = state.variables.get("plan_output").unwrap();
    let plan_str = serde_json::to_string(plan_out).unwrap();
    assert!(plan_str.contains("plan"));
}

// ---------------------------------------------------------------------------
// 2. pipeline + piece + judge
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_pipeline_executes_builtin_piece() {
    let mut runner = PipelineRunner::new();

    // Register the builtin "research" piece into the runner's engine
    for piece in builtin_pieces() {
        runner.engine_mut().register_piece(piece);
    }

    let config = PipelineConfig::builder()
        .piece_name("research")
        .task_text("Investigate Rust async patterns")
        .output_format("json")
        .timeout(Duration::from_secs(30))
        .verbose(true)
        .build()
        .expect("config build failed");

    let output = runner.execute(config).await.expect("pipeline failed");

    assert_eq!(output.exit_code(), PipelineExitCode::Success);
    assert_eq!(output.status, PipelineStatus::Success);
    assert!(output.is_success());
    assert!(output.movement_count >= 2); // investigate -> summarize
    assert!(output.error.is_none());

    // JSON format should be parseable
    let json = output.format_json().expect("json format failed");
    assert!(json.contains("\"exit_code\""));

    // Verbose details should be present
    assert!(output.details.is_some());
    let details = output.details.as_ref().unwrap();
    assert_eq!(details.piece_name, "research");
}

// ---------------------------------------------------------------------------
// 3. cycle + piece
// ---------------------------------------------------------------------------

#[test]
fn test_cycle_detector_on_builtin_pieces() {
    let detector = CycleDetector::new(LoopStrategy::AllowN(3));
    let pieces = builtin_pieces();

    for piece in &pieces {
        let analysis = detector
            .analyze_piece(piece)
            .unwrap_or_else(|e| panic!("Failed to analyze piece '{}': {}", piece.name, e));

        match piece.name.as_str() {
            "default" => {
                // default piece has review <-> fix cycle
                assert!(
                    analysis.has_cycles,
                    "default piece should have cycles (review<->fix)"
                );
                assert!(
                    !analysis.cyclic_movements.is_empty(),
                    "should identify cyclic movements"
                );
            }
            "research" => {
                // research is linear: investigate -> summarize
                assert!(!analysis.has_cycles, "research piece should be acyclic");
                assert_eq!(analysis.max_depth, 1);
            }
            "review-fix" => {
                // review-fix has review <-> fix cycle
                assert!(analysis.has_cycles, "review-fix piece should have cycles");
                assert!(analysis.cyclic_movements.contains("review"));
                assert!(analysis.cyclic_movements.contains("fix"));
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// 4. cycle (LoopTracker runtime monitoring)
// ---------------------------------------------------------------------------

#[test]
fn test_loop_tracker_monitors_cyclic_piece() {
    let detector = CycleDetector::new(LoopStrategy::AllowN(2));
    let mut tracker = detector.create_tracker();

    // Simulate a review <-> fix cycle
    assert!(!tracker.record_visit("review")); // visit 1
    assert!(!tracker.record_visit("fix")); // visit 1
    assert!(!tracker.record_visit("review")); // visit 2
    assert!(!tracker.record_visit("fix")); // visit 2
    assert!(tracker.record_visit("review")); // visit 3 -> exceeds AllowN(2)

    assert_eq!(tracker.visit_count("review"), 3);
    assert_eq!(tracker.visit_count("fix"), 2);

    // Pattern detection should find [review, fix] repeating
    let pattern = tracker.detect_pattern();
    assert!(
        pattern.is_some(),
        "should detect repeating review-fix pattern"
    );
    let pat = pattern.unwrap();
    assert!(
        pat.contains(&"review".to_string()) && pat.contains(&"fix".to_string()),
        "pattern should contain review and fix"
    );
}

// ---------------------------------------------------------------------------
// 5. permissions + piece
// ---------------------------------------------------------------------------

#[test]
fn test_permission_enforcer_with_piece_movements() {
    let yaml = r#"
name: perm-test
initial_movement: scan
movements:
  - id: scan
    instruction: "Scan codebase"
    tools: [read, grep, glob]
    permission: readonly
    rules:
      - condition: success
        next: modify
  - id: modify
    instruction: "Apply changes"
    tools: [read, write, edit, bash]
    permission: full
    rules: []
"#;
    let piece = Piece::from_yaml(yaml).expect("parse failed");

    // Test readonly movement permissions
    let scan = piece.get_movement("scan").unwrap();
    let readonly_enforcer = PermissionEnforcer::from_movement(scan.permission.clone(), &scan.tools);
    assert!(readonly_enforcer.check_tool("read").allowed);
    assert!(readonly_enforcer.check_tool("grep").allowed);
    assert!(readonly_enforcer.check_tool("glob").allowed);
    assert!(!readonly_enforcer.check_tool("bash").allowed);
    assert!(!readonly_enforcer.check_tool("write").allowed);
    assert!(
        !readonly_enforcer
            .check_file_access("src/main.rs", true)
            .allowed,
        "readonly should deny write access"
    );
    assert!(
        readonly_enforcer
            .check_file_access("src/main.rs", false)
            .allowed,
        "readonly should allow read access"
    );
    assert!(
        !readonly_enforcer
            .check_command_execution("cargo test")
            .allowed,
        "readonly should deny command execution"
    );

    // Test full permission movement
    let modify = piece.get_movement("modify").unwrap();
    let full_enforcer = PermissionEnforcer::from_movement(modify.permission.clone(), &modify.tools);
    assert!(full_enforcer.check_tool("read").allowed);
    assert!(full_enforcer.check_tool("write").allowed);
    assert!(full_enforcer.check_tool("edit").allowed);
    assert!(full_enforcer.check_tool("bash").allowed);
    assert!(
        full_enforcer.check_file_access("src/main.rs", true).allowed,
        "full should allow write access"
    );
    assert!(
        full_enforcer.check_command_execution("cargo test").allowed,
        "full should allow command execution"
    );

    // Protected files should still be denied even at full permission
    assert!(
        !full_enforcer.check_file_access(".env", false).allowed,
        "protected files denied even at full level"
    );
}

// ---------------------------------------------------------------------------
// 6. interactive + piece
// ---------------------------------------------------------------------------

#[test]
fn test_interactive_session_with_piece() {
    let yaml = r#"
name: interactive-piece
initial_movement: plan
movements:
  - id: plan
    persona: planner
    instruction: "Create a plan"
    rules:
      - condition: success
        next: done
  - id: done
    instruction: "Complete"
"#;
    let piece = Piece::from_yaml(yaml).expect("parse failed");

    // Persona mode: should respond with persona name
    let mut session = InteractiveSession::new(InteractiveMode::Persona);
    session.select_piece(&piece.name);

    let action = session
        .process_input("Implement a REST API", Some(&piece))
        .expect("process failed");
    match &action {
        InteractiveAction::ShowMessage(msg) => {
            assert!(
                msg.contains("planner"),
                "persona mode should reference the initial movement's persona"
            );
            assert!(msg.contains("REST API"));
        }
        _ => panic!("Expected ShowMessage in persona mode, got {:?}", action),
    }
    assert!(session.task_text.is_some());

    // /go command should transition to Execute
    let go_action = session
        .process_input("/go", Some(&piece))
        .expect("go failed");
    match go_action {
        InteractiveAction::Execute(task) => {
            assert!(
                task.contains("REST API"),
                "execute task should contain original input"
            );
        }
        _ => panic!("Expected Execute after /go"),
    }
    assert!(session.ready);
}

// ---------------------------------------------------------------------------
// 7. github_issue + pipeline + piece
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_github_issue_to_pipeline_workflow() {
    // Parse issue from JSON (simulating `gh issue view --json`)
    let json = r#"{
        "number": 99,
        "title": "Fix broken login flow",
        "body": "The login page returns 500 on invalid credentials",
        "url": "https://github.com/org/repo/issues/99",
        "labels": [{"name": "bug"}, {"name": "high-priority"}],
        "assignees": [{"login": "dev1"}],
        "repository": {"nameWithOwner": "org/repo"}
    }"#;

    let issue = parse_gh_issue(json).expect("parse failed");
    assert_eq!(issue.number, 99);
    assert!(issue.labels.contains(&"bug".to_string()));

    // Generate task from issue
    let generator = IssueTaskGenerator::new(sample_issue_config());
    let task = generator.generate_task(&issue);
    assert_eq!(task.piece_name, Some("review-fix".to_string())); // bug -> review-fix
    assert!(task.task_text.contains("#99"));
    assert!(task.task_text.contains("Fix broken login flow"));

    // Set up pipeline with builtin pieces
    let mut runner = PipelineRunner::new();
    for piece in builtin_pieces() {
        runner.engine_mut().register_piece(piece);
    }

    let config = PipelineConfig::builder()
        .piece_name(task.piece_name.as_deref().unwrap_or("default"))
        .task_text(&task.task_text)
        .output_format("text")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("config build");

    let output = runner.execute(config).await.expect("pipeline exec");
    // review-fix will run review -> (either done or fix cycle) eventually completing
    assert!(output.movement_count >= 1);

    // Format result comment for posting back
    let result = IssueResult {
        issue_number: issue.number,
        success: output.is_success(),
        summary: format!("Pipeline completed with status: {:?}", output.status),
        details: Some(output.output.clone()),
        files_changed: vec!["src/auth.rs".to_string()],
    };
    let comment = generator.format_result_comment(&result);
    assert!(comment.contains("ccswarm"));
    assert!(comment.contains("src/auth.rs"));
}

// ---------------------------------------------------------------------------
// 8. watch + cycle + i18n + piece
// ---------------------------------------------------------------------------

#[test]
fn test_watch_cycle_i18n_integration() {
    // Set up i18n with Japanese
    let mut i18n = I18nManager::new();
    let mut ja_strings = HashMap::new();
    ja_strings.insert(
        "watch.change_detected".to_string(),
        "変更検出: {count}ファイル".to_string(),
    );
    i18n.register_bundle(LocaleBundle {
        language: Language::Ja,
        strings: ja_strings,
    });
    i18n.set_language(Language::Ja);

    // Verify localized message
    let mut vars = HashMap::new();
    vars.insert("count".to_string(), "3".to_string());
    let msg = i18n.format("watch.change_detected", &vars);
    assert_eq!(msg, "変更検出: 3ファイル");

    // Language instruction for agents
    let lang_instruction = i18n.agent_language_instruction();
    assert!(lang_instruction.contains("日本語"));

    // Cycle analysis on a piece that the watch mode would trigger
    let piece = Piece {
        name: "watched-workflow".to_string(),
        description: "Workflow triggered by file changes".to_string(),
        max_movements: 20,
        initial_movement: "lint".to_string(),
        movements: vec![
            make_movement("lint", vec![("success", "test"), ("error", "fix")]),
            make_movement("test", vec![("success", "done")]),
            make_movement("fix", vec![("success", "lint")]),
            make_movement("done", vec![]),
        ],
        variables: HashMap::new(),
        metadata: HashMap::new(),
        interactive_mode: None,
    };

    let detector = CycleDetector::new(LoopStrategy::AllowN(3));
    let analysis = detector.analyze_piece(&piece).expect("analysis failed");
    assert!(
        analysis.has_cycles,
        "lint->fix->lint should be detected as a cycle"
    );
    assert!(analysis.cyclic_movements.contains("lint"));
    assert!(analysis.cyclic_movements.contains("fix"));

    // Watch controller: debounce_ms=0 for deterministic test
    let config = WatchConfig {
        debounce_ms: 0,
        piece_name: "watched-workflow".to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_consecutive_runs: 3,
        ..WatchConfig::default()
    };
    let mut watcher = WatchController::new(config);

    // Record a change and poll
    watcher.record_change(FileChange {
        path: PathBuf::from("src/lib.rs"),
        change_type: ChangeType::Modified,
        detected_at: std::time::SystemTime::now(),
    });
    assert_eq!(*watcher.state(), WatchState::Debouncing);

    std::thread::sleep(Duration::from_millis(1));
    let changes = watcher.poll();
    assert!(changes.is_some());
    assert_eq!(*watcher.state(), WatchState::Executing);
    assert_eq!(changes.unwrap().len(), 1);

    watcher.execution_complete();
    assert_eq!(*watcher.state(), WatchState::Idle);
}

// ---------------------------------------------------------------------------
// 9. arpeggio + github_issue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_arpeggio_batch_from_issues() {
    let issues = vec![
        sample_issue(1, "Fix header alignment", vec!["bug", "frontend"]),
        sample_issue(2, "Add user avatar upload", vec!["feature"]),
        sample_issue(3, "Research caching strategies", vec!["research"]),
    ];

    let generator = IssueTaskGenerator::new(sample_issue_config());

    // Convert issues to ArpeggioItems
    let items: Vec<ArpeggioItem> = issues
        .iter()
        .map(|issue| {
            let task = generator.generate_task(issue);
            ArpeggioItem {
                id: format!("issue-{}", issue.number),
                task_text: task.task_text,
                variables: task.variables.into_iter().collect(),
            }
        })
        .collect();

    assert_eq!(items.len(), 3);
    assert!(items[0].task_text.contains("#1"));
    assert!(items[1].task_text.contains("user avatar"));
    assert!(items[2].task_text.contains("caching strategies"));

    // Execute batch
    let config = ArpeggioConfig {
        piece_name: "default".to_string(),
        max_concurrency: 1,
        fail_fast: false,
        ..ArpeggioConfig::default()
    };
    let executor = ArpeggioExecutor::new(config);
    let result = executor.execute(items).await.expect("arpeggio failed");

    assert_eq!(result.total, 3);
    assert_eq!(result.succeeded, 3);
    assert_eq!(result.failed, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.all_succeeded());
    assert_eq!(result.success_rate(), 100.0);
    assert!(result.total_duration_ms < 5000); // Should be fast (simulated)

    // Verify each item result
    assert_eq!(result.items[0].id, "issue-1");
    assert!(result.items[0].success);
    assert_eq!(result.items[2].id, "issue-3");
}

// ---------------------------------------------------------------------------
// 10. facets + i18n + judge + piece
// ---------------------------------------------------------------------------

#[test]
fn test_facets_i18n_judge_prompt_composition() {
    // Set up FacetRegistry with builtins
    let mut registry = FacetRegistry::new();
    for persona in builtin_personas() {
        registry.register_persona(persona);
    }
    for policy in builtin_policies() {
        registry.register_policy(policy);
    }

    // Set up i18n for Japanese
    let mut i18n = I18nManager::new();
    i18n.set_language(Language::Ja);
    let lang_instruction = i18n.agent_language_instruction();

    // Compose prompt with persona=coder, policy=coding, + i18n instruction
    let composed = registry.compose(
        Some("coder"),
        Some("coding"),
        None,
        &format!("Implement feature X\n\n{}", lang_instruction),
        Some("Return JSON with {status, files_changed}"),
    );

    // System prompt should contain coder persona
    assert!(
        composed.system.contains("coder"),
        "system should have persona"
    );
    assert!(
        composed.system.contains("Senior software engineer"),
        "should use builtin coder role"
    );

    // User prompt should have structured sections
    assert!(
        composed.user.contains("## Task"),
        "should have Task section"
    );
    assert!(
        composed.user.contains("Implement feature X"),
        "should contain instruction"
    );
    assert!(
        composed.user.contains("日本語"),
        "should contain i18n language instruction"
    );
    assert!(
        composed.user.contains("## Constraints"),
        "should have Constraints from policy"
    );
    assert!(
        composed.user.contains("unwrap"),
        "coding policy should mention unwrap prohibition"
    );
    assert!(
        composed.user.contains("## Output Format"),
        "should have Output Format section"
    );
    assert!(
        composed.user.contains("JSON"),
        "output contract should mention JSON"
    );

    // Verify section ordering: Task before Constraints before Output Format
    let task_pos = composed.user.find("## Task").unwrap();
    let constraints_pos = composed.user.find("## Constraints").unwrap();
    let output_pos = composed.user.find("## Output Format").unwrap();
    assert!(
        task_pos < constraints_pos,
        "Task should come before Constraints"
    );
    assert!(
        constraints_pos < output_pos,
        "Constraints should come before Output Format"
    );

    // Generate judge tag instructions for a sample movement's rules
    let rules = vec![
        MovementRule {
            condition: RuleCondition::Simple("success".to_string()),
            next: "deploy".to_string(),
            priority: 0,
        },
        MovementRule {
            condition: RuleCondition::Simple("needs_fix".to_string()),
            next: "fix".to_string(),
            priority: 1,
        },
    ];
    let tag_instructions = MovementJudge::generate_tag_instructions(&rules);
    assert!(tag_instructions.contains("[STEP:0]"));
    assert!(tag_instructions.contains("[STEP:1]"));
    assert!(tag_instructions.contains("success"));
    assert!(tag_instructions.contains("needs_fix"));

    // Judge should match based on step tags
    let judge = MovementJudge::default();
    let result = judge
        .evaluate("[STEP:0]\nAll good", &rules, None)
        .expect("judge failed");
    assert_eq!(result.matched_rule_index, Some(0));
    assert_eq!(result.match_method, MatchMethod::StepTag);
}
