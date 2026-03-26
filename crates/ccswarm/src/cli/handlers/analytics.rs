use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_search_cmd(&self, action: &SearchAction) -> Result<()> {
        match action {
            SearchAction::Docs { query, limit } => {
                let docs_dir = self.repo_path.join("docs");
                if !docs_dir.exists() {
                    anyhow::bail!("docs/ directory not found at {}", docs_dir.display());
                }

                let query_lower = query.to_lowercase();
                let mut results: Vec<(String, usize, String)> = Vec::new();

                Self::search_dir_recursive(&docs_dir, &query_lower, &mut results).await?;

                results.truncate(*limit);

                if self.json_output {
                    let data: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(file, line, text)| {
                            serde_json::json!({
                                "file": file,
                                "line": line,
                                "text": text.trim(),
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "query": query,
                            "data": data,
                        }))?
                    );
                } else if results.is_empty() {
                    println!("No matches for \"{}\" in docs/", query);
                } else {
                    println!(
                        "{} \"{}\" ({} matches)",
                        "Search results:".bright_cyan().bold(),
                        query.bright_yellow(),
                        results.len()
                    );
                    for (file, line, text) in &results {
                        println!("  {}:{} {}", file.bright_green(), line, text.trim());
                    }
                }
            }
            SearchAction::Code { query, glob, limit } => {
                let query_lower = query.to_lowercase();
                let mut results: Vec<(String, usize, String)> = Vec::new();

                Self::search_dir_recursive(&self.repo_path, &query_lower, &mut results).await?;

                // Apply glob filter if provided
                if let Some(glob_pattern) = glob {
                    let matcher = glob::Pattern::new(glob_pattern)
                        .map_err(|e| anyhow!("Invalid glob pattern: {}", e))?;
                    results.retain(|(file, _, _)| {
                        Path::new(file)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|name| matcher.matches(name))
                    });
                }

                results.truncate(*limit);

                if self.json_output {
                    let data: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(file, line, text)| {
                            serde_json::json!({
                                "file": file,
                                "line": line,
                                "text": text.trim(),
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "query": query,
                            "glob": glob,
                            "data": data,
                        }))?
                    );
                } else if results.is_empty() {
                    println!("No matches for \"{}\" in source code", query);
                } else {
                    println!(
                        "{} \"{}\" ({} matches)",
                        "Code search:".bright_cyan().bold(),
                        query.bright_yellow(),
                        results.len()
                    );
                    for (file, line, text) in &results {
                        println!("  {}:{} {}", file.bright_green(), line, text.trim());
                    }
                }
            }
        }

        Ok(())
    }

    /// Recursively search directory for query matches in text files
    async fn search_dir_recursive(
        dir: &Path,
        query: &str,
        results: &mut Vec<(String, usize, String)>,
    ) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip hidden dirs and common non-text dirs
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && (name.starts_with('.') || name == "target" || name == "node_modules")
            {
                continue;
            }

            if path.is_dir() {
                // Use Box::pin to handle recursive async
                Box::pin(Self::search_dir_recursive(&path, query, results)).await?;
            } else if path.is_file() {
                // Only search text-like files
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !matches!(
                    ext,
                    "rs" | "md"
                        | "toml"
                        | "yaml"
                        | "yml"
                        | "json"
                        | "ts"
                        | "js"
                        | "py"
                        | "go"
                        | "sh"
                        | "txt"
                        | "html"
                        | "css"
                ) {
                    continue;
                }

                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    for (i, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(query) {
                            results.push((path.display().to_string(), i + 1, line.to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_evolution(&self, action: &EvolutionAction) -> Result<()> {
        match action {
            EvolutionAction::Metrics { agent, format } => {
                let status_dir = PathBuf::from("coordination/agent-status");
                let mut metrics: Vec<serde_json::Value> = Vec::new();

                if status_dir.exists() {
                    let mut entries = tokio::fs::read_dir(&status_dir).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json")
                            && let Ok(content) = tokio::fs::read_to_string(&path).await
                            && let Ok(status) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            if let Some(filter) = agent {
                                if status
                                    .get("agent_id")
                                    .and_then(|s| s.as_str())
                                    .is_some_and(|s| s == filter)
                                {
                                    metrics.push(status);
                                }
                            } else {
                                metrics.push(status);
                            }
                        }
                    }
                }

                if format == "json" || self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": metrics,
                        }))?
                    );
                } else if metrics.is_empty() {
                    println!("No agent metrics found in coordination/agent-status/");
                } else {
                    println!("{}", "Agent Metrics".bright_cyan().bold());
                    println!("{}", "=============".bright_cyan());
                    for m in &metrics {
                        let agent_id = m.get("agent_id").and_then(|v| v.as_str()).unwrap_or("?");
                        let st = m.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let ts = m.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?");
                        println!(
                            "  {} [{}] last updated: {}",
                            agent_id.bright_yellow(),
                            st.bright_white(),
                            ts.bright_black()
                        );
                    }
                    println!("\nTotal: {} agents", metrics.len());
                }
            }
            EvolutionAction::Patterns { agent, limit } => {
                let task_dir = PathBuf::from("coordination/task-queue");
                let mut tasks: Vec<serde_json::Value> = Vec::new();

                if task_dir.exists() {
                    let mut entries = tokio::fs::read_dir(&task_dir).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json")
                            && let Ok(content) = tokio::fs::read_to_string(&path).await
                            && let Ok(task) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            if let Some(filter) = agent {
                                if task
                                    .get("assigned_agent")
                                    .and_then(|s| s.as_str())
                                    .is_some_and(|s| s == filter)
                                {
                                    tasks.push(task);
                                }
                            } else {
                                tasks.push(task);
                            }
                        }
                    }
                }

                tasks.truncate(*limit);

                // Count by status
                let mut status_counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                for task in &tasks {
                    let st = task
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    *status_counts.entry(st).or_default() += 1;
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": {
                                "tasks": tasks,
                                "status_counts": status_counts,
                            },
                        }))?
                    );
                } else {
                    println!("{}", "Task Patterns".bright_cyan().bold());
                    println!("{}", "=============".bright_cyan());
                    if status_counts.is_empty() {
                        println!("No tasks found in coordination/task-queue/");
                    } else {
                        println!("  Status breakdown:");
                        for (status, count) in &status_counts {
                            println!(
                                "    {}: {}",
                                status.bright_white(),
                                count.to_string().bright_yellow()
                            );
                        }
                        println!("\n  Total: {} tasks analyzed", tasks.len());
                    }
                }
            }
            EvolutionAction::Report { format } => {
                // Combine metrics + patterns
                let status_dir = PathBuf::from("coordination/agent-status");
                let task_dir = PathBuf::from("coordination/task-queue");

                let agent_count = if status_dir.exists() {
                    let mut count = 0usize;
                    let mut entries = tokio::fs::read_dir(&status_dir).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        if entry.path().extension().is_some_and(|e| e == "json") {
                            count += 1;
                        }
                    }
                    count
                } else {
                    0
                };

                let task_count = if task_dir.exists() {
                    let mut count = 0usize;
                    let mut entries = tokio::fs::read_dir(&task_dir).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        if entry.path().extension().is_some_and(|e| e == "json") {
                            count += 1;
                        }
                    }
                    count
                } else {
                    0
                };

                let report = serde_json::json!({
                    "agent_count": agent_count,
                    "task_count": task_count,
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                });

                match format.as_str() {
                    "json" => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "data": report,
                            }))?
                        );
                    }
                    "markdown" => {
                        println!("# Evolution Report\n");
                        println!("- **Agents tracked**: {}", agent_count);
                        println!("- **Tasks in queue**: {}", task_count);
                        println!(
                            "- **Generated**: {}",
                            report
                                .get("generated_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                        );
                    }
                    _ => {
                        println!("{}", "Evolution Report".bright_cyan().bold());
                        println!("{}", "================".bright_cyan());
                        println!("  Agents tracked: {}", agent_count);
                        println!("  Tasks in queue: {}", task_count);
                    }
                }
            }
        }

        Ok(())
    }
}
