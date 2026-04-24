use super::super::*;

/// Ask a yes/no question on stderr, return true for 'y'
fn ask_yn(question: &str) -> bool {
    eprint!("  {} [y/N] ", question);
    let _ = std::io::Write::flush(&mut std::io::stderr());
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        input.trim().eq_ignore_ascii_case("y")
    } else {
        false
    }
}

impl CliRunner {
    pub(crate) async fn handle_interactive(&self, mode: &str, flow: Option<&str>) -> Result<()> {
        use crate::workflow::flow::FlowEngine;
        use crate::workflow::interactive::{
            InteractiveAction, InteractiveMode, InteractiveSession,
        };

        // Parse interaction mode
        let interactive_mode = match mode {
            "persona" => InteractiveMode::Persona,
            "quiet" => InteractiveMode::Quiet,
            "passthrough" => InteractiveMode::Passthrough,
            _ => InteractiveMode::Assistant,
        };

        println!("{}", "🎯 ccswarm interactive mode".bright_cyan().bold());
        println!("   Mode: {:?}", interactive_mode);
        if let Some(p) = flow {
            println!("   Flow: {}", p);
        }
        println!(
            "   Commands: {} {} {} {}",
            "/go".bright_green(),
            "/play <task>".bright_green(),
            "/mode <mode>".bright_green(),
            "/quit".bright_red()
        );
        println!();

        // Load flow engine and optional flow
        let mut engine = FlowEngine::new();
        let loaded_flow = if let Some(flow_name) = flow {
            // Try to load from builtin flows
            engine.load_builtin_flows();
            engine.get_flow(flow_name).cloned()
        } else {
            engine.load_builtin_flows();
            None
        };

        let mut session = InteractiveSession::new(interactive_mode);
        if let Some(p) = flow {
            session.select_flow(p);
        }

        // REPL loop
        loop {
            // Show prompt
            let prompt_str = match session.mode {
                InteractiveMode::Assistant => "ccswarm(assistant)> ",
                InteractiveMode::Persona => "ccswarm(persona)> ",
                InteractiveMode::Quiet => "ccswarm(quiet)> ",
                InteractiveMode::Passthrough => "ccswarm(pass)> ",
            };
            print!("{}", prompt_str.bright_yellow());
            std::io::stdout().flush()?;

            // Read input
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input)? == 0 {
                break; // EOF
            }
            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            // Process input through interactive session
            let action = session.process_input(input, loaded_flow.as_ref())?;

            match action {
                InteractiveAction::AskQuestion(clarification) => {
                    println!("\n{} {}", "?".bright_blue().bold(), clarification.question);
                    if !clarification.options.is_empty() {
                        for (i, opt) in clarification.options.iter().enumerate() {
                            println!("  {}. {}", i + 1, opt);
                        }
                    }
                    println!();
                }
                InteractiveAction::ShowMessage(msg) => {
                    println!("{}", msg);
                }
                InteractiveAction::Execute(task_text) => {
                    println!(
                        "\n{} Executing task: {}",
                        ">>>".bright_green().bold(),
                        task_text.bright_white()
                    );

                    // Execute via AISessionBridge if available, otherwise show the task
                    println!(
                        "{}",
                        "Task queued for execution. Use 'ccswarm pipeline' for full execution."
                            .bright_yellow()
                    );
                    println!();

                    // Reset session for next task
                    session = InteractiveSession::new(session.mode);
                    if let Some(p) = flow {
                        session.select_flow(p);
                    }
                }
                InteractiveAction::Exit => {
                    println!("{}", "Goodbye!".bright_cyan());
                    break;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn handle_pipeline(
        &self,
        task: &str,
        flow: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
        _isolate: bool, // TODO: pass to MovementExecOptions.worktree_name
        budget: Option<f64>,
        run_budget_tokens: Option<u64>,
        _model_override: Option<&str>, // TODO: pass to MovementExecOptions.model
        auto_commit: bool,
        create_pr: bool,
    ) -> Result<()> {
        self.handle_pipeline_returning_id(
            task,
            flow,
            output_format,
            timeout,
            verbose,
            output_file,
            _isolate,
            budget,
            run_budget_tokens,
            _model_override,
            auto_commit,
            create_pr,
        )
        .await
        .map(|_run_id| ())
    }

    /// Dispatcher shim: routes to the dry-run preview or the real pipeline.
    /// Issue #48: `--dry-run` prints the composed prompt per stage and exits.
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_pipeline_with_dry_run(
        &self,
        task: &str,
        flow: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
        isolate: bool,
        budget: Option<f64>,
        run_budget_tokens: Option<u64>,
        model_override: Option<&str>,
        auto_commit: bool,
        create_pr: bool,
        dry_run: bool,
    ) -> Result<()> {
        if dry_run {
            return self.handle_pipeline_dry_run(task, flow).await;
        }
        self.handle_pipeline(
            task,
            flow,
            output_format,
            timeout,
            verbose,
            output_file,
            isolate,
            budget,
            run_budget_tokens,
            model_override,
            auto_commit,
            create_pr,
        )
        .await
    }

    /// Dry-run: compose per-stage prompts and print them, no provider CLI spawn.
    /// Builds on the same facet registry `flow render` uses, but also substitutes the
    /// user's `{task}` template variable so what you see is actually what would be sent.
    async fn handle_pipeline_dry_run(&self, task: &str, flow_name: &str) -> Result<()> {
        use crate::workflow::facets::FacetRegistry;
        use crate::workflow::flow::{Flow, builtin_flows};

        // Resolve flow: builtin → .ccswarm/flows/<name>.yaml
        let flow_obj: Flow = builtin_flows()
            .into_iter()
            .find(|f| f.name == flow_name)
            .or_else(|| {
                let path = self
                    .repo_path
                    .join(".ccswarm")
                    .join("flows")
                    .join(format!("{}.yaml", flow_name));
                if path.is_file() {
                    let yaml = std::fs::read_to_string(&path).ok()?;
                    Flow::from_yaml(&yaml).ok()
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("flow '{}' not found", flow_name))?;

        let mut registry = FacetRegistry::new_with_builtins();
        let project_facets = self.repo_path.join(".ccswarm").join("facets");
        if project_facets.exists() {
            let _ = registry.load_from_dir(&project_facets).await;
        }

        println!(
            "{} dry-run — flow={} task={}",
            "▶".bright_cyan().bold(),
            flow_name.bright_white(),
            task.chars().take(60).collect::<String>().bright_black()
        );
        println!();

        for stage in &flow_obj.stages {
            let contract_text = stage
                .output_contract
                .as_ref()
                .map(|c| format!("format: {}", c.format));
            // Substitute {task} in the instruction so the preview is realistic.
            let instruction = stage.instruction.replace("{task}", task);
            let composed = registry.compose(
                stage.persona.as_deref(),
                stage.policy.as_deref(),
                stage.knowledge.as_deref(),
                &instruction,
                contract_text.as_deref(),
            );
            println!(
                "{}",
                format!("=== stage: {} ===", stage.id).bright_cyan().bold()
            );
            if !composed.system.is_empty() {
                println!("{}", "--- system ---".bright_black());
                println!("{}", composed.system);
            }
            println!("{}", "--- user ---".bright_black());
            println!("{}", composed.user);
            println!();
        }
        Ok(())
    }

    /// Same as [`handle_pipeline`] but surfaces the `run_id` so autonomous callers
    /// (`ccswarm auto`) can cross-reference their own `auto.ndjson` with the run's
    /// `events.ndjson`. Addresses codex #6.
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_pipeline_returning_id(
        &self,
        task: &str,
        flow: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
        _isolate: bool,
        budget: Option<f64>,
        run_budget_tokens: Option<u64>,
        _model_override: Option<&str>,
        auto_commit: bool,
        create_pr: bool,
    ) -> Result<String> {
        let (run_id, result) = self
            .execute_pipeline_core(
                task,
                flow,
                output_format,
                timeout,
                verbose,
                output_file,
                budget,
                run_budget_tokens,
            )
            .await?;

        // Post-pipeline assisted flow: auto-detect → execute → ask OK/NG
        self.run_post_pipeline_flow(task, flow, &run_id, &result, auto_commit, create_pr)
            .await;

        Ok(run_id)
    }

    /// Core pipeline execution logic without post-pipeline flow.
    /// Returns (run_id, PipelineOutput) on success.
    /// Separated from `handle_pipeline` to allow `run_post_pipeline_flow`
    /// to run fix pipelines without triggering recursive post-pipeline flows.
    #[allow(clippy::too_many_arguments)]
    async fn execute_pipeline_core(
        &self,
        task: &str,
        flow: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
        budget: Option<f64>,
        run_budget_tokens: Option<u64>,
    ) -> Result<(String, crate::workflow::pipeline::PipelineOutput)> {
        use crate::workflow::pipeline::{PipelineConfig, PipelineRunner};
        use std::time::Duration;

        info!("Starting pipeline: task='{}', flow='{}'", task, flow);

        let config = PipelineConfig::builder()
            .flow_name(flow)
            .task_text(task)
            .output_format(output_format)
            .timeout(Duration::from_secs(timeout))
            .verbose(verbose)
            .build()
            .context("Failed to build pipeline configuration")?;

        // Configure bridge for real Claude Code CLI execution
        let mut engine = crate::workflow::flow::FlowEngine::new();
        let bridge = crate::session::bridge::AISessionBridge::new(
            self.repo_path.join(".ccswarm").join("sessions"),
        );
        engine.set_bridge(std::sync::Arc::new(bridge));
        engine.set_working_dir(self.repo_path.clone());
        if let Some(b) = budget {
            engine.set_budget(b);
        }
        if let Some(cap) = run_budget_tokens {
            engine.set_run_token_cap(cap);
        }

        // Load custom flows from .ccswarm/flows/
        let custom_pieces_dir = self.repo_path.join(".ccswarm").join("flows");
        if let Ok(mut entries) = tokio::fs::read_dir(&custom_pieces_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let is_yaml = path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml");
                if is_yaml && let Err(e) = engine.load_flow(&path).await {
                    warn!("Failed to load custom flow {:?}: {}", path, e);
                }
            }
        }

        // Configure event recorder for observability
        let run_id = uuid::Uuid::new_v4().to_string();
        if let Ok(recorder) = crate::events::EventRecorder::new(&run_id).await {
            engine.set_event_recorder(recorder);
        }

        // Set up real-time progress display
        let (progress_tx, mut progress_rx) =
            tokio::sync::mpsc::unbounded_channel::<crate::workflow::flow::MovementProgress>();
        engine.set_progress_channel(progress_tx);

        // Spawn progress display task
        let flow_name = flow.to_string();
        let progress_handle = tokio::spawn(async move {
            let mut total_ms: u64 = 0;
            eprintln!(
                "{}",
                format!("Pipeline: {}", flow_name).bright_cyan().bold()
            );
            while let Some(progress) = progress_rx.recv().await {
                total_ms += progress.duration_ms;
                let icon = if progress.success {
                    "\u{2713}"
                } else {
                    "\u{2717}"
                };
                let secs = progress.duration_ms as f64 / 1000.0;
                let total_secs = total_ms as f64 / 1000.0;
                let line = format!(
                    "  {} {} ({:.0}s, total {:.0}s)",
                    icon, progress.movement_id, secs, total_secs
                );
                if progress.success {
                    eprintln!("{}", line.bright_green());
                } else {
                    eprintln!("{}", line.bright_red());
                }
            }
        });

        let runner = PipelineRunner::with_engine(engine);
        let result = runner.execute(config).await?;

        // Clean up progress display
        drop(progress_handle);

        // Format output based on requested format
        let formatted = match output_format {
            "json" => result.format_json()?,
            "markdown" => result.format_markdown(),
            _ => result.format_text(),
        };

        // Write to file or stdout
        if let Some(path) = output_file {
            std::fs::write(path, &formatted)
                .with_context(|| format!("Failed to write output to {}", path.display()))?;
            println!(
                "{} Output written to {}",
                "OK".bright_green().bold(),
                path.display()
            );
        } else {
            println!("{}", formatted);
        }

        if !result.is_success() {
            std::process::exit(result.exit_code().as_code());
        }

        Ok((run_id, result))
    }

    /// Post-pipeline assisted flow: auto-detect tests, run them, then guide
    /// the user through OK/NG decisions for commit and PR.
    #[allow(clippy::too_many_arguments)]
    async fn run_post_pipeline_flow(
        &self,
        task: &str,
        flow: &str,
        run_id: &str,
        result: &crate::workflow::pipeline::PipelineOutput,
        auto_commit: bool,
        create_pr: bool,
    ) {
        let repo = &self.repo_path;
        eprintln!();

        // Step 1: Auto-detect and run tests, with auto-fix loop (max 3 retries)
        let mut test_passed = self.auto_run_tests(repo).await;
        let mut fix_attempts = 0;
        const MAX_FIX_ATTEMPTS: u32 = 3;

        while !test_passed && fix_attempts < MAX_FIX_ATTEMPTS {
            fix_attempts += 1;
            eprintln!(
                "\n  \u{1f527} Auto-fix attempt {}/{}...",
                fix_attempts, MAX_FIX_ATTEMPTS
            );

            // codex #5 fix: route the fix call through crate::providers instead of
            // hardcoding `claude`. Otherwise a Codex/Copilot-configured project silently
            // reverts to Claude on every test failure (and bills the wrong account).
            let provider_kind = std::env::var("CCSWARM_PROVIDER")
                .ok()
                .as_deref()
                .and_then(crate::providers::ProviderKind::parse)
                .unwrap_or(crate::providers::ProviderKind::Claude);
            let provider = crate::providers::resolve(provider_kind);
            let options = crate::providers::ProviderOptions::default();
            let mut fix_cmd = provider.build_command(
                "Fix the failing tests. Read the test output, identify the issue, and fix the code.",
                repo,
                &options,
            );
            // The centrally enforced cwd in bridge.rs doesn't apply here because we bypass
            // the bridge; do it ourselves.
            fix_cmd.current_dir(repo);
            let fix_output = fix_cmd.output().await;

            if fix_output.is_err() || !fix_output.as_ref().is_ok_and(|o| o.status.success()) {
                eprintln!("  {} Fix attempt failed", "\u{2717}".bright_red());
                break;
            }
            eprintln!("  {} Fix applied", "\u{2713}".bright_green());

            test_passed = self.auto_run_tests(repo).await;
        }

        if !test_passed && fix_attempts >= MAX_FIX_ATTEMPTS {
            eprintln!(
                "  {} Tests still failing after {} fix attempts",
                "\u{2717}".bright_red(),
                MAX_FIX_ATTEMPTS
            );
        }

        // Step 2: Commit (auto or ask)
        let committed = if auto_commit && test_passed {
            self.do_commit(repo, task).await
        } else if test_passed {
            if ask_yn("Commit changes?") {
                self.do_commit(repo, task).await
            } else {
                false
            }
        } else {
            false
        };

        // Step 3: PR (auto or ask)
        if committed && (create_pr || ask_yn("Create pull request?")) {
            self.do_create_pr(repo, task, flow, result, run_id).await;
        }

        let short_id = &run_id[..8.min(run_id.len())];
        eprintln!(
            "\n  {} ccswarm run view {short_id}",
            "View details:".bright_cyan()
        );
    }

    /// Detect and run tests automatically. Returns true if tests pass.
    async fn auto_run_tests(&self, repo: &std::path::Path) -> bool {
        // Prefer a project-local `.claude/skills/check-impl/SKILL.md` over the
        // per-language heuristic below. The skill pins the exact fmt/lint/test/build
        // sequence the project cares about; the heuristic is only a fallback for
        // projects that haven't defined one.
        if let Some(result) = self.run_check_impl_skill(repo).await {
            return result;
        }

        // Detect test runner
        let (cmd, args): (&str, &[&str]) = if repo.join("playwright.config.ts").exists()
            || repo.join("playwright.config.js").exists()
            || repo.join("playwright.config.mjs").exists()
        {
            ("npx", &["playwright", "test"])
        } else if repo.join("Cargo.toml").exists() {
            ("cargo", &["test"])
        } else if repo.join("package.json").exists() {
            ("npm", &["test"])
        } else if repo.join("go.mod").exists() {
            ("go", &["test", "./..."])
        } else if repo.join("pyproject.toml").exists() {
            ("uv", &["run", "--frozen", "pytest"])
        } else if let Some(spec) = find_loose_test_spec(repo) {
            // #42 fix: even without a package.json, if the AI generated a *.spec.mjs /
            // *.test.js alongside the app, run just that file through Playwright. This
            // makes the Tetris-style "HTML + generated spec" flow self-verifying.
            eprintln!(
                "  {} Auto-detected spec file: {}",
                "\u{25b6}".bright_blue(),
                spec.display()
            );
            let output = tokio::process::Command::new("npx")
                .args(["playwright", "test", &spec.to_string_lossy()])
                .current_dir(repo)
                .output()
                .await;
            return match output {
                Ok(o) if o.status.success() => {
                    eprintln!("  {} Spec passed", "\u{2713}".bright_green());
                    true
                }
                Ok(_) => {
                    eprintln!("  {} Spec failed", "\u{2717}".bright_red());
                    false
                }
                Err(_) => {
                    eprintln!("  {} npx playwright unavailable", "-".bright_yellow());
                    true
                }
            };
        } else {
            eprintln!("  {} No test runner detected", "-".bright_yellow());
            return true; // No tests = pass
        };

        eprintln!(
            "  {} Running: {} {}",
            "\u{25b6}".bright_blue(),
            cmd,
            args.join(" ")
        );

        let output = tokio::process::Command::new(cmd)
            .args(args)
            .current_dir(repo)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                eprintln!("  {} Tests passed", "\u{2713}".bright_green());
                true
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                eprintln!("  {} Tests failed", "\u{2717}".bright_red());
                // Show last few lines of output
                let combined = format!("{}{}", stdout, stderr);
                for line in combined
                    .lines()
                    .rev()
                    .take(5)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    eprintln!("    {}", line);
                }
                false
            }
            Err(e) => {
                eprintln!("  {} Could not run tests: {}", "\u{2717}".bright_red(), e);
                true // Can't run = skip
            }
        }
    }

    /// Run `.claude/skills/check-impl/SKILL.md`'s documented command sequence, if present.
    /// Returns `Some(true)` if every command succeeded, `Some(false)` if any failed,
    /// `None` if no usable skill was found (the caller then falls back to the heuristic).
    ///
    /// We execute the first fenced `bash`/`sh`/`shell` block verbatim, line-by-line. This
    /// matches how Claude Code itself invokes the skill — ccswarm is just skipping the
    /// LLM round-trip when the sequence is purely mechanical.
    async fn run_check_impl_skill(&self, repo: &std::path::Path) -> Option<bool> {
        let skill_path = repo.join(".claude/skills/check-impl/SKILL.md");
        if !skill_path.is_file() {
            return None;
        }
        let content = tokio::fs::read_to_string(&skill_path).await.ok()?;
        let commands = extract_first_shell_block(&content);
        if commands.is_empty() {
            return None;
        }

        eprintln!(
            "  {} .claude/skills/check-impl ({} command{})",
            "\u{25b6}".bright_blue(),
            commands.len(),
            if commands.len() == 1 { "" } else { "s" }
        );

        for cmd in &commands {
            eprintln!("  {} {}", "$".bright_black(), cmd.bright_black());
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(repo)
                .output()
                .await;
            match output {
                Ok(o) if o.status.success() => continue,
                Ok(o) => {
                    eprintln!("  {} check-impl step failed", "\u{2717}".bright_red());
                    let combined = format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    );
                    for line in combined
                        .lines()
                        .rev()
                        .take(5)
                        .collect::<Vec<_>>()
                        .iter()
                        .rev()
                    {
                        eprintln!("    {}", line);
                    }
                    return Some(false);
                }
                Err(e) => {
                    eprintln!(
                        "  {} check-impl step could not run: {}",
                        "\u{2717}".bright_red(),
                        e
                    );
                    return Some(false);
                }
            }
        }

        eprintln!("  {} check-impl passed", "\u{2713}".bright_green());
        Some(true)
    }

    /// Commit changes. In auto mode (`ccswarm auto`) this runs without user confirmation,
    /// so we deliberately stage only files that are *already tracked* (`git add -u`).
    /// Untracked secret-like files (.env, *.key, credentials*) or anything the user has
    /// gitignored are left untouched. If the task genuinely needed to add new files, the
    /// user can do it manually in interactive mode.
    async fn do_commit(&self, repo: &std::path::Path, task: &str) -> bool {
        // Stage modifications + deletions of tracked files only.
        let _ = tokio::process::Command::new("git")
            .args(["add", "-u"])
            .current_dir(repo)
            .output()
            .await;
        // Additionally add any *new* files that are NOT matched by a deny-list. This keeps
        // common workflows (e.g. `touch README.md`) functional while refusing to silently
        // commit secrets.
        if let Ok(o) = tokio::process::Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(repo)
            .output()
            .await
            && o.status.success()
        {
            for untracked in String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|p| !is_sensitive_path(p))
            {
                let _ = tokio::process::Command::new("git")
                    .args(["add", "--", untracked])
                    .current_dir(repo)
                    .output()
                    .await;
            }
        }
        let msg = format!("ccswarm: {}", task.chars().take(72).collect::<String>());
        let result = tokio::process::Command::new("git")
            .args(["commit", "-m", &msg])
            .current_dir(repo)
            .output()
            .await;
        match result {
            Ok(o) if o.status.success() => {
                eprintln!("  {} Committed", "\u{2713}".bright_green());
                true
            }
            _ => {
                eprintln!("  {} Nothing to commit", "-".bright_yellow());
                false
            }
        }
    }

    /// Create a pull request
    async fn do_create_pr(
        &self,
        repo: &std::path::Path,
        task: &str,
        flow: &str,
        result: &crate::workflow::pipeline::PipelineOutput,
        run_id: &str,
    ) {
        let short_id = &run_id[..8.min(run_id.len())];
        let pr_output = tokio::process::Command::new("gh")
            .args([
                "pr",
                "create",
                "--title",
                &format!("ccswarm: {}", task.chars().take(72).collect::<String>()),
                "--body",
                &format!(
                    "## Generated by ccswarm\n\nPiece: {}\nMovements: {}\nDuration: {:.0}s\nRun: {}",
                    flow,
                    result.movement_count,
                    result.duration.as_secs_f64(),
                    short_id
                ),
            ])
            .current_dir(repo)
            .output()
            .await;
        match pr_output {
            Ok(o) if o.status.success() => {
                let url = String::from_utf8_lossy(&o.stdout);
                eprintln!("  {} PR: {}", "\u{2713}".bright_green(), url.trim());
            }
            _ => {
                eprintln!(
                    "  {} PR creation failed (is gh cli installed?)",
                    "\u{2717}".bright_red()
                );
            }
        }
    }

    pub(crate) async fn handle_piece(&self, action: &FlowAction) -> Result<()> {
        use crate::workflow::flow::builtin_flows;

        match action {
            FlowAction::List => {
                let flows = builtin_flows();

                // Also check for custom flows in .ccswarm/flows/
                let custom_dir = self.repo_path.join(".ccswarm").join("flows");
                let mut custom_count = 0;

                println!("{}", "Available Flows".bright_cyan().bold());
                println!("{}", "================".bright_cyan());
                println!();
                println!("{}", "Builtin:".bright_white().bold());
                for flow in &flows {
                    println!(
                        "  {} - {} ({} stages)",
                        flow.name.bright_green(),
                        flow.description,
                        flow.stages.len()
                    );
                }

                if custom_dir.exists()
                    && let Ok(entries) = std::fs::read_dir(&custom_dir)
                {
                    let yaml_files: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "yaml" || ext == "yml")
                                .unwrap_or(false)
                        })
                        .collect();

                    if !yaml_files.is_empty() {
                        println!();
                        println!("{}", "Custom:".bright_white().bold());
                        for entry in &yaml_files {
                            let name = entry
                                .path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            println!("  {} ({})", name.bright_yellow(), entry.path().display());
                            custom_count += 1;
                        }
                    }
                }

                println!();
                println!("Total: {} builtin, {} custom", flows.len(), custom_count);
            }
            FlowAction::Eject { name, output } => {
                let flows = builtin_flows();
                let flow = flows.iter().find(|p| p.name == *name).ok_or_else(|| {
                    anyhow!(
                        "Flow '{}' not found. Use 'ccswarm flow list' to see available flows.",
                        name
                    )
                })?;

                let output_dir = output
                    .clone()
                    .unwrap_or_else(|| self.repo_path.join(".ccswarm").join("flows"));

                // Create output directory
                std::fs::create_dir_all(&output_dir).with_context(|| {
                    format!("Failed to create directory: {}", output_dir.display())
                })?;

                let output_path = output_dir.join(format!("{}.yaml", name));

                let yaml =
                    serde_yml::to_string(flow).context("Failed to serialize flow to YAML")?;

                std::fs::write(&output_path, &yaml)
                    .with_context(|| format!("Failed to write to {}", output_path.display()))?;

                println!(
                    "{} Ejected flow '{}' to {}",
                    "OK".bright_green().bold(),
                    name.bright_cyan(),
                    output_path.display()
                );
                println!(
                    "{}",
                    "Edit this file to customize the workflow, then use it with: ccswarm pipeline --flow <name>"
                        .bright_black()
                );
            }
            FlowAction::Inspect { name } => {
                let flows = builtin_flows();
                let flow = flows
                    .iter()
                    .find(|p| p.name == *name)
                    .ok_or_else(|| anyhow!("Flow '{}' not found", name))?;

                println!("{}", flow.name.bright_cyan().bold());
                println!("{}", "=".repeat(flow.name.len()).bright_cyan());
                println!();
                println!("{}", flow.description);
                println!();
                println!("Initial stage: {}", flow.initial_movement.bright_green());
                println!("Max stages: {}", flow.max_stages);
                println!();
                println!("{}", "Stages:".bright_white().bold());
                for stage in &flow.stages {
                    let persona = stage.persona.as_deref().unwrap_or("default");
                    println!(
                        "  {} [{}] - {}",
                        stage.id.bright_green(),
                        persona.bright_yellow(),
                        stage.instruction
                    );
                    if !stage.rules.is_empty() {
                        for rule in &stage.rules {
                            println!(
                                "    -> {} (on {:?})",
                                rule.next.bright_cyan(),
                                rule.condition
                            );
                        }
                    }
                }
            }
            FlowAction::Check { target } => {
                self.handle_piece_check(target).await?;
            }
            FlowAction::New {
                name,
                template,
                output,
            } => {
                self.handle_piece_new(name, template, output.as_deref())
                    .await?;
            }
            FlowAction::Render { target, stage } => {
                self.handle_piece_render(target, stage.as_deref()).await?;
            }
            FlowAction::Suggest { task } => {
                let (flow, reason) = suggest_flow_for_task(task);
                println!(
                    "{} {} — {}",
                    "→".bright_cyan().bold(),
                    flow.bright_green().bold(),
                    reason.bright_black()
                );
                println!();
                println!(
                    "Run it with: ccswarm pipeline --task \"...\" --flow {}",
                    flow
                );
            }
        }

        Ok(())
    }

    /// Locate a flow by name (builtin or custom) or path, return its YAML source.
    async fn resolve_piece_source(&self, target: &str) -> Result<(String, String)> {
        let as_path = std::path::PathBuf::from(target);
        if as_path.is_file() {
            let content = tokio::fs::read_to_string(&as_path)
                .await
                .with_context(|| format!("Failed to read {}", as_path.display()))?;
            return Ok((target.to_string(), content));
        }

        // Custom flow: .ccswarm/flows/<name>.yaml
        for ext in ["yaml", "yml"] {
            let custom = self
                .repo_path
                .join(".ccswarm")
                .join("flows")
                .join(format!("{}.{}", target, ext));
            if custom.is_file() {
                let content = tokio::fs::read_to_string(&custom)
                    .await
                    .with_context(|| format!("Failed to read {}", custom.display()))?;
                return Ok((target.to_string(), content));
            }
        }

        // Builtin flow: re-serialize to YAML
        use crate::workflow::flow::builtin_flows;
        let flows = builtin_flows();
        let flow = flows
            .iter()
            .find(|p| p.name == target)
            .ok_or_else(|| anyhow!("Flow '{}' not found", target))?;
        let yaml = serde_yml::to_string(flow)?;
        Ok((target.to_string(), yaml))
    }

    async fn handle_piece_check(&self, target: &str) -> Result<()> {
        use crate::workflow::flow::Flow;

        let (name, yaml) = self.resolve_piece_source(target).await?;
        let mut issues: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        match Flow::from_yaml(&yaml) {
            Ok(flow) => {
                if flow.description.trim().is_empty() {
                    warnings.push("flow has empty description".into());
                }
                for m in &flow.stages {
                    if m.instruction.trim().is_empty() && m.persona.is_none() {
                        warnings.push(format!(
                            "stage '{}' has empty instruction and no persona",
                            m.id
                        ));
                    }
                    if m.rules.is_empty() && m.id != "complete" {
                        warnings.push(format!("stage '{}' has no routing rules (terminal?)", m.id));
                    }
                }
            }
            Err(e) => issues.push(e.to_string()),
        }

        if issues.is_empty() && warnings.is_empty() {
            println!(
                "{} flow '{}' is valid",
                "OK".bright_green().bold(),
                name.bright_cyan()
            );
        } else {
            if !issues.is_empty() {
                println!(
                    "{} flow '{}' has {} issue(s):",
                    "NG".bright_red().bold(),
                    name.bright_cyan(),
                    issues.len()
                );
                for msg in &issues {
                    println!("  - {}", msg.bright_red());
                }
            }
            if !warnings.is_empty() {
                println!(
                    "{} flow '{}' has {} warning(s):",
                    "WARN".bright_yellow().bold(),
                    name.bright_cyan(),
                    warnings.len()
                );
                for msg in &warnings {
                    println!("  - {}", msg.bright_yellow());
                }
            }
            if !issues.is_empty() {
                return Err(anyhow!("Flow validation failed"));
            }
        }
        Ok(())
    }

    async fn handle_piece_new(
        &self,
        name: &str,
        template: &str,
        output: Option<&std::path::Path>,
    ) -> Result<()> {
        let output_dir = output
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.repo_path.join(".ccswarm").join("flows"));
        tokio::fs::create_dir_all(&output_dir).await?;
        let output_path = output_dir.join(format!("{}.yaml", name));

        if output_path.exists() {
            return Err(anyhow!(
                "File already exists: {}. Remove it first or pick another name.",
                output_path.display()
            ));
        }

        let body = match template {
            "minimal" => minimal_piece_template(name),
            "faceted" => faceted_piece_template(name),
            other => {
                return Err(anyhow!(
                    "Unknown template '{}'. Use 'minimal' or 'faceted'.",
                    other
                ));
            }
        };

        tokio::fs::write(&output_path, body).await?;
        println!(
            "{} Created flow '{}' at {}",
            "OK".bright_green().bold(),
            name.bright_cyan(),
            output_path.display()
        );
        println!(
            "{}",
            "  Edit it, then run: ccswarm flow check <name> && ccswarm pipeline --flow <name>"
                .bright_black()
        );
        Ok(())
    }

    async fn handle_piece_render(&self, target: &str, stage: Option<&str>) -> Result<()> {
        use crate::workflow::facets::FacetRegistry;
        use crate::workflow::flow::Flow;

        let (name, yaml) = self.resolve_piece_source(target).await?;
        let flow =
            Flow::from_yaml(&yaml).with_context(|| format!("Failed to parse flow '{}'", name))?;

        // Project-local facets under .ccswarm/facets override builtins
        let mut registry = FacetRegistry::new_with_builtins();
        let project_facets = self.repo_path.join(".ccswarm").join("facets");
        if project_facets.exists() {
            let _ = registry.load_from_dir(&project_facets).await;
        }

        for m in &flow.stages {
            if let Some(only) = stage
                && m.id != only
            {
                continue;
            }
            let output_contract_text = m
                .output_contract
                .as_ref()
                .map(|c| format!("format: {}", c.format));
            let composed = registry.compose(
                m.persona.as_deref(),
                m.policy.as_deref(),
                m.knowledge.as_deref(),
                &m.instruction,
                output_contract_text.as_deref(),
            );

            let header = format!("=== stage: {} ===", m.id);
            println!("{}", header.bright_cyan().bold());
            if !composed.system.is_empty() {
                println!("{}", "--- system ---".bright_black());
                println!("{}", composed.system);
            }
            println!("{}", "--- user ---".bright_black());
            println!("{}", composed.user);
            println!();
        }
        Ok(())
    }

    pub(crate) async fn handle_repertoire(&self, action: &RepertoireAction) -> Result<()> {
        use crate::workflow::repertoire::RepertoireManager;

        let manager = RepertoireManager::new()?;

        match action {
            RepertoireAction::Add { url } => {
                println!("Installing flow package from {}...", url.bright_cyan());
                let package = manager.add(url).await?;
                println!(
                    "{} Installed '{}' with {} flows",
                    "OK".bright_green().bold(),
                    package.name.bright_cyan(),
                    package.flows.len()
                );
                if !package.flows.is_empty() {
                    println!("Flows:");
                    for flow_name in &package.flows {
                        println!("  - {}", flow_name.bright_green());
                    }
                }
            }
            RepertoireAction::List => {
                let packages = manager.list().await?;
                if packages.is_empty() {
                    println!("No repertoire packages installed.");
                    println!();
                    println!("Install a package with: ccswarm repertoire add <git-url>");
                } else {
                    println!("{}", "Installed Packages".bright_cyan().bold());
                    println!("{}", "==================".bright_cyan());
                    for package in &packages {
                        println!();
                        println!(
                            "  {} ({})",
                            package.name.bright_green().bold(),
                            package.source_url.bright_black()
                        );
                        println!("    Path: {}", package.install_path.display());
                        println!("    Flows: {}", package.flows.join(", "));
                    }
                    println!();
                    println!("Total: {} packages", packages.len());
                }
            }
            RepertoireAction::Remove { name } => {
                manager.remove(name).await?;
                println!(
                    "{} Removed package '{}'",
                    "OK".bright_green().bold(),
                    name.bright_cyan()
                );
            }
        }

        Ok(())
    }
}

/// Paths that should never be auto-staged into a commit even if the task appears to have
/// created them. This is a denylist rather than an allowlist because the task space is
/// open-ended — we only want to stop obvious secret-leak patterns.
fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        basename,
        ".env"
            | ".envrc"
            | "credentials"
            | "credentials.json"
            | "id_rsa"
            | "id_ed25519"
            | "secrets.yml"
            | "secrets.yaml"
    ) || basename.ends_with(".pem")
        || basename.ends_with(".key")
        || basename.starts_with(".env.")
}

fn minimal_piece_template(name: &str) -> String {
    format!(
        r#"name: {name}
description: Minimal one-shot workflow
max_stages: 3
initial_movement: main
stages:
  - id: main
    persona: coder
    instruction: |
      Complete the task as described.
    tools: [read, write, edit, bash]
    permission: edit
    rules:
      - condition: success
        next: complete
  - id: complete
    persona: default
    instruction: ""
    rules: []
"#
    )
}

fn faceted_piece_template(name: &str) -> String {
    format!(
        r#"name: {name}
description: Plan -> implement -> review workflow
max_stages: 10
initial_movement: plan
stages:
  - id: plan
    persona: planner
    knowledge: architecture
    instruction: |
      Analyze the task and create an implementation plan.
    tools: [read, grep, glob]
    permission: readonly
    rules:
      - condition: success
        next: implement
  - id: implement
    persona: coder
    policy: coding
    instruction: |
      Implement the plan produced in the previous step.
    tools: [read, write, edit, bash]
    permission: edit
    rules:
      - condition: success
        next: review
      - condition: error
        next: implement
  - id: review
    persona: reviewer
    policy: security
    instruction: |
      Review the implementation against the plan and quality criteria.
    tools: [read, grep, glob]
    permission: readonly
    rules:
      - condition: success
        next: complete
      - condition: fixes_needed
        next: implement
  - id: complete
    persona: default
    instruction: ""
    rules: []
"#
    )
}

/// Pick a builtin flow for the given task description.
///
/// This is a tiny keyword heuristic — not an LLM — because decision fatigue (issue #45)
/// is about "just give me something reasonable", not "find the optimal workflow". The
/// user can always override with `--flow <name>`. Keywords are checked in priority
/// order; first match wins.
pub fn suggest_flow_for_task(task: &str) -> (&'static str, &'static str) {
    let lower = task.to_lowercase();
    // Parallel / multi-agent signals → `team` (plan + parallel frontend/backend + review).
    if lower.contains("frontend") && lower.contains("backend")
        || lower.contains("full-stack")
        || lower.contains("fullstack")
        || lower.contains(" parallel ")
    {
        return (
            "team",
            "frontend + backend keywords → parallel multi-agent flow",
        );
    }
    // Research / investigate signals → `research`.
    if lower.contains("investigate")
        || lower.contains("research ")
        || lower.contains("analyze ")
        || lower.contains("survey ")
    {
        return ("research", "investigative task → research flow");
    }
    // Fix / bug signals → `review-fix`.
    if lower.contains("fix ") || lower.contains("bug") || lower.contains("regression") {
        return ("review-fix", "fix-oriented task → review-fix loop");
    }
    // Trivial / one-shot signals → `quick`.
    if lower.contains("rename ")
        || lower.contains("trivial")
        || lower.contains("one-liner")
        || lower.contains("typo")
        || lower.contains("constant")
        || task.len() < 80
    {
        return ("quick", "short / trivial task → single-shot flow");
    }
    // Default: plan → implement → review → fix.
    ("default", "general task → plan → implement → review → fix")
}

#[cfg(test)]
mod suggest_tests {
    use super::suggest_flow_for_task;

    #[test]
    fn short_task_picks_quick() {
        let (flow, _) = suggest_flow_for_task("Rename foo to bar");
        assert_eq!(flow, "quick");
    }

    #[test]
    fn fix_task_picks_review_fix() {
        let (flow, _) = suggest_flow_for_task(
            "Fix the regression that breaks the login flow on Safari when cookies are disabled",
        );
        assert_eq!(flow, "review-fix");
    }

    #[test]
    fn fullstack_picks_team() {
        let (flow, _) = suggest_flow_for_task(
            "Add a full-stack endpoint: frontend dashboard page, backend POST /api/alerts, plus migration",
        );
        assert_eq!(flow, "team");
    }

    #[test]
    fn long_generic_picks_default() {
        let (flow, _) = suggest_flow_for_task(
            "Build a configuration editor that lets operators change limits across environments with audit trail and rollback",
        );
        assert_eq!(flow, "default");
    }
}

/// If the repo has no conventional test config but does have a loose `*.spec.mjs` /
/// `*.spec.js` / `*.test.mjs` / `*.test.js` file at the top level, return it so the
/// post-pipeline flow can try `npx playwright test <that file>` (issue #42). We only
/// look one level deep to keep this predictable and fast.
fn find_loose_test_spec(repo: &std::path::Path) -> Option<std::path::PathBuf> {
    let read = std::fs::read_dir(repo).ok()?;
    for entry in read.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if (name.ends_with(".spec.mjs")
            || name.ends_with(".spec.js")
            || name.ends_with(".test.mjs")
            || name.ends_with(".test.js"))
            && path.is_file()
        {
            return Some(path);
        }
    }
    None
}

/// Extract commands from the first fenced `bash`/`sh`/`shell` block in a markdown
/// document. Blank lines and comment-only lines are skipped. Line continuations
/// (trailing `\`) are joined so multi-line commands run as one.
///
/// Kept minimal on purpose: we execute these commands via `sh -c`, so full shell
/// syntax (pipes, `&&`, env overrides) inside a single line works. We don't try to
/// interpret bash constructs ourselves.
fn extract_first_shell_block(md: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_block = false;
    let mut current = String::new();
    for raw in md.lines() {
        let trimmed = raw.trim_start();
        if !in_block {
            if trimmed.starts_with("```bash")
                || trimmed.starts_with("```sh")
                || trimmed.starts_with("```shell")
            {
                in_block = true;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            break;
        }
        let line = trimmed.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_suffix('\\') {
            current.push_str(stripped.trim_end());
            current.push(' ');
            continue;
        }
        current.push_str(line);
        commands.push(std::mem::take(&mut current));
    }
    if !current.trim().is_empty() {
        commands.push(current);
    }
    commands
}

#[cfg(test)]
mod check_impl_tests {
    use super::extract_first_shell_block;

    #[test]
    fn extracts_commands_skipping_blanks_and_comments() {
        let md = "preamble\n\n```bash\n# 1. format\ncargo fmt --all --check\n\n# 2. lint\ncargo clippy --workspace -- -D warnings\n```\n\nmore text\n```bash\nshould_be_ignored\n```\n";
        let cmds = extract_first_shell_block(md);
        assert_eq!(
            cmds,
            vec![
                "cargo fmt --all --check".to_string(),
                "cargo clippy --workspace -- -D warnings".to_string(),
            ]
        );
    }

    #[test]
    fn no_block_returns_empty() {
        assert!(extract_first_shell_block("no fenced bash here").is_empty());
    }

    #[test]
    fn accepts_sh_and_shell_fences() {
        assert_eq!(
            extract_first_shell_block("```sh\necho hi\n```"),
            vec!["echo hi".to_string()]
        );
        assert_eq!(
            extract_first_shell_block("```shell\nls -la\n```"),
            vec!["ls -la".to_string()]
        );
    }

    #[test]
    fn joins_line_continuations() {
        let md = "```bash\ncargo test \\\n    --workspace \\\n    --no-fail-fast\n```";
        let cmds = extract_first_shell_block(md);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("--workspace"));
        assert!(cmds[0].contains("--no-fail-fast"));
    }
}
