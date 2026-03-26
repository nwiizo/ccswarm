use super::super::*;

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

    pub(crate) async fn handle_pipeline(
        &self,
        task: &str,
        piece: &str,
        output_format: &str,
        timeout: u64,
        verbose: bool,
        output_file: Option<&Path>,
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

        let runner = PipelineRunner::new();
        let result = runner.execute(config).await?;

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

        Ok(())
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
