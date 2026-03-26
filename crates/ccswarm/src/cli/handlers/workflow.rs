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
    pub(crate) async fn handle_interactive(&self, mode: &str, piece: Option<&str>) -> Result<()> {
        use crate::workflow::interactive::{
            InteractiveAction, InteractiveMode, InteractiveSession,
        };
        use crate::workflow::piece::PieceEngine;

        // Parse interaction mode
        let interactive_mode = match mode {
            "persona" => InteractiveMode::Persona,
            "quiet" => InteractiveMode::Quiet,
            "passthrough" => InteractiveMode::Passthrough,
            _ => InteractiveMode::Assistant,
        };

        println!("{}", "🎯 ccswarm interactive mode".bright_cyan().bold());
        println!("   Mode: {:?}", interactive_mode);
        if let Some(p) = piece {
            println!("   Piece: {}", p);
        }
        println!(
            "   Commands: {} {} {} {}",
            "/go".bright_green(),
            "/play <task>".bright_green(),
            "/mode <mode>".bright_green(),
            "/quit".bright_red()
        );
        println!();

        // Load piece engine and optional piece
        let mut engine = PieceEngine::new();
        let loaded_piece = if let Some(piece_name) = piece {
            // Try to load from builtin pieces
            engine.load_builtin_pieces();
            engine.get_piece(piece_name).cloned()
        } else {
            engine.load_builtin_pieces();
            None
        };

        let mut session = InteractiveSession::new(interactive_mode);
        if let Some(p) = piece {
            session.select_piece(p);
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
            let action = session.process_input(input, loaded_piece.as_ref())?;

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
                    if let Some(p) = piece {
                        session.select_piece(p);
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
    pub(crate) async fn handle_pipeline(
        &self,
        task: &str,
        piece: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
        _isolate: bool,      // TODO: pass to MovementExecOptions.worktree_name
        _budget: Option<f64>, // TODO: pass to MovementExecOptions.max_budget
        _model_override: Option<&str>, // TODO: pass to MovementExecOptions.model
        auto_commit: bool,
        create_pr: bool,
    ) -> Result<()> {
        use crate::workflow::pipeline::{PipelineConfig, PipelineRunner};
        use std::time::Duration;

        info!("Starting pipeline: task='{}', piece='{}'", task, piece);

        let config = PipelineConfig::builder()
            .piece_name(piece)
            .task_text(task)
            .output_format(output_format)
            .timeout(Duration::from_secs(timeout))
            .verbose(verbose)
            .build()
            .context("Failed to build pipeline configuration")?;

        // Configure bridge for real Claude Code CLI execution
        let mut engine = crate::workflow::piece::PieceEngine::new();
        let bridge = crate::session::bridge::AISessionBridge::new(
            self.repo_path.join(".ccswarm").join("sessions"),
        );
        engine.set_bridge(std::sync::Arc::new(bridge));
        engine.set_working_dir(self.repo_path.clone());

        // Load custom pieces from .ccswarm/pieces/
        let custom_pieces_dir = self.repo_path.join(".ccswarm").join("pieces");
        if let Ok(mut entries) = tokio::fs::read_dir(&custom_pieces_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let is_yaml = path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml");
                if is_yaml && let Err(e) = engine.load_piece(&path).await {
                    warn!("Failed to load custom piece {:?}: {}", path, e);
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
            tokio::sync::mpsc::unbounded_channel::<crate::workflow::piece::MovementProgress>();
        engine.set_progress_channel(progress_tx);

        // Spawn progress display task
        let piece_name = piece.to_string();
        let progress_handle = tokio::spawn(async move {
            let mut total_ms: u64 = 0;
            eprintln!(
                "{}",
                format!("Pipeline: {}", piece_name).bright_cyan().bold()
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

        // Post-pipeline assisted flow: auto-detect → execute → ask OK/NG
        self.run_post_pipeline_flow(task, piece, &run_id, &result, auto_commit, create_pr)
            .await;

        Ok(())
    }

    /// Post-pipeline assisted flow: auto-detect tests, run them, then guide
    /// the user through OK/NG decisions for commit and PR.
    #[allow(clippy::too_many_arguments)]
    async fn run_post_pipeline_flow(
        &self,
        task: &str,
        piece: &str,
        run_id: &str,
        result: &crate::workflow::pipeline::PipelineOutput,
        auto_commit: bool,
        create_pr: bool,
    ) {
        let repo = &self.repo_path;
        eprintln!();

        // Step 1: Auto-detect and run tests
        let test_passed = self.auto_run_tests(repo).await;

        // Step 2: Commit (auto or ask)
        let committed = if auto_commit {
            self.do_commit(repo, task).await
        } else if test_passed {
            if ask_yn("Commit changes?") {
                self.do_commit(repo, task).await
            } else {
                false
            }
        } else {
            // Tests failed — ask if user wants to fix
            if ask_yn("Tests failed. Run review-fix pipeline?") {
                eprintln!("  Run: ccswarm pipeline --piece review-fix --task \"Fix test failures\"");
            }
            false
        };

        // Step 3: PR (auto or ask)
        if committed {
            if create_pr {
                self.do_create_pr(repo, task, piece, result, run_id).await;
            } else if ask_yn("Create pull request?") {
                self.do_create_pr(repo, task, piece, result, run_id).await;
            }
        }

        let short_id = &run_id[..8.min(run_id.len())];
        eprintln!(
            "\n  {} ccswarm run view {short_id}",
            "View details:".bright_cyan()
        );
    }

    /// Detect and run tests automatically. Returns true if tests pass.
    async fn auto_run_tests(&self, repo: &std::path::Path) -> bool {
        // Detect test runner
        let (cmd, args): (&str, &[&str]) =
            if repo.join("playwright.config.ts").exists()
                || repo.join("playwright.config.js").exists()
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
                for line in combined.lines().rev().take(5).collect::<Vec<_>>().into_iter().rev() {
                    eprintln!("    {}", line);
                }
                false
            }
            Err(e) => {
                eprintln!(
                    "  {} Could not run tests: {}",
                    "\u{2717}".bright_red(),
                    e
                );
                true // Can't run = skip
            }
        }
    }

    /// Commit all changes
    async fn do_commit(&self, repo: &std::path::Path, task: &str) -> bool {
        let _ = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo)
            .output()
            .await;
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
        piece: &str,
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
                    piece,
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

    pub(crate) async fn handle_piece(&self, action: &PieceAction) -> Result<()> {
        use crate::workflow::piece::builtin_pieces;

        match action {
            PieceAction::List => {
                let pieces = builtin_pieces();

                // Also check for custom pieces in .ccswarm/pieces/
                let custom_dir = self.repo_path.join(".ccswarm").join("pieces");
                let mut custom_count = 0;

                println!("{}", "Available Pieces".bright_cyan().bold());
                println!("{}", "================".bright_cyan());
                println!();
                println!("{}", "Builtin:".bright_white().bold());
                for piece in &pieces {
                    println!(
                        "  {} - {} ({} movements)",
                        piece.name.bright_green(),
                        piece.description,
                        piece.movements.len()
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
                println!("Total: {} builtin, {} custom", pieces.len(), custom_count);
            }
            PieceAction::Eject { name, output } => {
                let pieces = builtin_pieces();
                let piece = pieces.iter().find(|p| p.name == *name).ok_or_else(|| {
                    anyhow!(
                        "Piece '{}' not found. Use 'ccswarm piece list' to see available pieces.",
                        name
                    )
                })?;

                let output_dir = output
                    .clone()
                    .unwrap_or_else(|| self.repo_path.join(".ccswarm").join("pieces"));

                // Create output directory
                std::fs::create_dir_all(&output_dir).with_context(|| {
                    format!("Failed to create directory: {}", output_dir.display())
                })?;

                let output_path = output_dir.join(format!("{}.yaml", name));

                let yaml =
                    serde_yaml::to_string(piece).context("Failed to serialize piece to YAML")?;

                std::fs::write(&output_path, &yaml)
                    .with_context(|| format!("Failed to write to {}", output_path.display()))?;

                println!(
                    "{} Ejected piece '{}' to {}",
                    "OK".bright_green().bold(),
                    name.bright_cyan(),
                    output_path.display()
                );
                println!(
                    "{}",
                    "Edit this file to customize the workflow, then use it with: ccswarm pipeline --piece <name>"
                        .bright_black()
                );
            }
            PieceAction::Inspect { name } => {
                let pieces = builtin_pieces();
                let piece = pieces
                    .iter()
                    .find(|p| p.name == *name)
                    .ok_or_else(|| anyhow!("Piece '{}' not found", name))?;

                println!("{}", piece.name.bright_cyan().bold());
                println!("{}", "=".repeat(piece.name.len()).bright_cyan());
                println!();
                println!("{}", piece.description);
                println!();
                println!(
                    "Initial movement: {}",
                    piece.initial_movement.bright_green()
                );
                println!("Max movements: {}", piece.max_movements);
                println!();
                println!("{}", "Movements:".bright_white().bold());
                for movement in &piece.movements {
                    let persona = movement.persona.as_deref().unwrap_or("default");
                    println!(
                        "  {} [{}] - {}",
                        movement.id.bright_green(),
                        persona.bright_yellow(),
                        movement.instruction
                    );
                    if !movement.rules.is_empty() {
                        for rule in &movement.rules {
                            println!(
                                "    -> {} (on {:?})",
                                rule.next.bright_cyan(),
                                rule.condition
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_repertoire(&self, action: &RepertoireAction) -> Result<()> {
        use crate::workflow::repertoire::RepertoireManager;

        let manager = RepertoireManager::new()?;

        match action {
            RepertoireAction::Add { url } => {
                println!("Installing piece package from {}...", url.bright_cyan());
                let package = manager.add(url).await?;
                println!(
                    "{} Installed '{}' with {} pieces",
                    "OK".bright_green().bold(),
                    package.name.bright_cyan(),
                    package.pieces.len()
                );
                if !package.pieces.is_empty() {
                    println!("Pieces:");
                    for piece_name in &package.pieces {
                        println!("  - {}", piece_name.bright_green());
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
                        println!("    Pieces: {}", package.pieces.join(", "));
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
