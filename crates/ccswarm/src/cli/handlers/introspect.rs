//! Handlers for introspection-style commands: `facets`, `tail`, `cost`.

use super::super::*;
use super::run_utils::resolve_run_path;
use std::io::BufRead;

impl CliRunner {
    pub(crate) async fn handle_facets(&self, kind: &str, detailed: bool) -> Result<()> {
        use crate::workflow::facets::FacetRegistry;

        let mut registry = FacetRegistry::new_with_builtins();
        let project_facets = self.repo_path.join(".ccswarm").join("facets");
        if project_facets.exists() {
            let _ = registry.load_from_dir(&project_facets).await;
        }

        let k = kind.to_lowercase();
        let show_all = matches!(k.as_str(), "all" | "");
        let show_personas = show_all || k == "personas" || k == "persona";
        let show_policies = show_all || k == "policies" || k == "policy";
        let show_knowledge = show_all || k == "knowledge";

        if show_personas {
            let mut names: Vec<&String> = registry.personas.keys().collect();
            names.sort();
            println!("{}", "Personas".bright_cyan().bold());
            println!("{}", "========".bright_cyan());
            if names.is_empty() {
                println!("  (none)");
            }
            for name in names {
                if detailed && let Some(p) = registry.personas.get(name) {
                    let first_line = p.role.lines().next().unwrap_or("").trim();
                    println!(
                        "  {} — {}",
                        name.bright_green().bold(),
                        if first_line.is_empty() {
                            "(role unspecified)"
                        } else {
                            first_line
                        }
                    );
                } else {
                    println!("  {}", name.bright_green());
                }
            }
            println!();
        }

        if show_policies {
            let mut names: Vec<&String> = registry.policies.keys().collect();
            names.sort();
            println!("{}", "Policies".bright_cyan().bold());
            println!("{}", "========".bright_cyan());
            if names.is_empty() {
                println!("  (none)");
            }
            for name in names {
                if detailed && let Some(p) = registry.policies.get(name) {
                    let desc = if p.description.is_empty() {
                        "(no description)"
                    } else {
                        p.description.as_str()
                    };
                    println!("  {} — {}", name.bright_yellow().bold(), desc);
                } else {
                    println!("  {}", name.bright_yellow());
                }
            }
            println!();
        }

        if show_knowledge {
            let mut names: Vec<&String> = registry.knowledge.keys().collect();
            names.sort();
            println!("{}", "Knowledge".bright_cyan().bold());
            println!("{}", "=========".bright_cyan());
            if names.is_empty() {
                println!("  (none)");
            }
            for name in names {
                println!("  {}", name.bright_magenta());
            }
            println!();
        }
        Ok(())
    }

    pub(crate) async fn handle_tail(&self, run_id: Option<&str>, no_follow: bool) -> Result<()> {
        let resolved = resolve_run_path(&self.repo_path, run_id).await?;

        let events_file = resolved.join("events.ndjson");
        if !events_file.exists() {
            return Err(anyhow!("events.ndjson not found in {}", resolved.display()));
        }

        println!(
            "{} Tailing run {}",
            "→".bright_cyan(),
            resolved
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
                .bright_white()
        );
        println!("{} {}", "File:".bright_black(), events_file.display());
        println!();

        // Print existing events
        let file = std::fs::File::open(&events_file)?;
        let reader = std::io::BufReader::new(file);
        let mut last_pos: u64 = 0;
        for line in reader.lines().map_while(Result::ok) {
            last_pos += line.len() as u64 + 1;
            print_event_line(&line);
        }

        if no_follow {
            return Ok(());
        }

        // Follow: poll for new bytes, a summary.json appearing ends the tail.
        let summary_path = resolved.join("summary.json");
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let metadata = std::fs::metadata(&events_file)?;
            let size = metadata.len();
            if size > last_pos {
                use std::io::{Read, Seek, SeekFrom};
                let mut file = std::fs::File::open(&events_file)?;
                file.seek(SeekFrom::Start(last_pos))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                for line in buf.lines() {
                    print_event_line(line);
                }
                last_pos = size;
            }
            if summary_path.exists() {
                println!("{} run finished", "✓".bright_green().bold());
                break;
            }
        }
        Ok(())
    }
}

impl CliRunner {
    pub(crate) async fn handle_cost(&self, run_id: Option<&str>) -> Result<()> {
        let run_path = resolve_run_path(&self.repo_path, run_id).await?;
        let summary_path = run_path.join("summary.json");
        let events_path = run_path.join("events.ndjson");

        println!(
            "{} {}",
            "Run:".bright_black(),
            run_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
                .bright_white()
        );
        println!();

        // Per-stage durations from events
        let mut movement_duration_ms: std::collections::BTreeMap<String, u64> =
            std::collections::BTreeMap::new();
        let mut agent_durations: std::collections::BTreeMap<String, u64> =
            std::collections::BTreeMap::new();
        let mut total_tokens_in: u64 = 0;
        let mut total_tokens_out: u64 = 0;
        let mut event_count: u64 = 0;

        if events_path.exists() {
            let file = std::fs::File::open(&events_path)?;
            let reader = std::io::BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                event_count += 1;
                if let Some(m) = v.get("stage").and_then(|s| s.as_str())
                    && let Some(dur) = v
                        .get("metadata")
                        .and_then(|md| md.get("duration_ms"))
                        .and_then(|d| d.as_u64())
                {
                    *movement_duration_ms.entry(m.to_string()).or_insert(0) += dur;
                }
                if let Some(a) = v.get("agent").and_then(|s| s.as_str())
                    && let Some(dur) = v
                        .get("metadata")
                        .and_then(|md| md.get("duration_ms"))
                        .and_then(|d| d.as_u64())
                {
                    *agent_durations.entry(a.to_string()).or_insert(0) += dur;
                }
                let md = v.get("metadata");
                if let Some(md) = md {
                    total_tokens_in += md.get("tokens_in").and_then(|n| n.as_u64()).unwrap_or(0);
                    total_tokens_out += md.get("tokens_out").and_then(|n| n.as_u64()).unwrap_or(0);
                }
            }
        } else {
            println!(
                "{}",
                "(events.ndjson not found — cannot aggregate durations)".bright_yellow()
            );
        }

        if !movement_duration_ms.is_empty() {
            println!("{}", "By stage".bright_cyan().bold());
            println!("{}", "-----------".bright_cyan());
            let total: u64 = movement_duration_ms.values().sum();
            for (name, ms) in &movement_duration_ms {
                let pct = if total > 0 {
                    (*ms as f64) * 100.0 / (total as f64)
                } else {
                    0.0
                };
                println!(
                    "  {:<20} {:>8} ms  ({:>5.1}%)",
                    name.bright_green(),
                    ms,
                    pct
                );
            }
            println!();
        }

        if !agent_durations.is_empty() {
            println!("{}", "By agent".bright_cyan().bold());
            println!("{}", "--------".bright_cyan());
            for (name, ms) in &agent_durations {
                println!("  {:<20} {:>8} ms", name.bright_yellow(), ms);
            }
            println!();
        }

        println!("{}", "Totals".bright_cyan().bold());
        println!("{}", "------".bright_cyan());
        println!("  events:       {}", event_count);
        if total_tokens_in > 0 || total_tokens_out > 0 {
            println!(
                "  tokens in:    {}  |  tokens out: {}",
                total_tokens_in, total_tokens_out
            );
        } else {
            println!(
                "  {}",
                "(no token metadata recorded yet — track via ai-session context metrics)"
                    .bright_black()
            );
        }

        if summary_path.exists() {
            let raw = std::fs::read_to_string(&summary_path)?;
            if let Ok(s) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(d) = s.get("total_duration_ms").and_then(|x| x.as_u64()) {
                    println!("  total:        {} ms", d);
                }
                if let Some(status) = s.get("status").and_then(|x| x.as_str()) {
                    println!("  status:       {}", status);
                }
            }
        }
        Ok(())
    }
}

fn print_event_line(line: &str) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(v) => {
            let ts = v
                .get("ts")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .split('T')
                .nth(1)
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("");
            let level = v.get("level").and_then(|l| l.as_str()).unwrap_or("info");
            let ev = v.get("event_type").and_then(|s| s.as_str()).unwrap_or("?");
            let stage = v.get("stage").and_then(|m| m.as_str()).unwrap_or("");
            let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("");
            let attention = v
                .get("metadata")
                .and_then(|md| md.get("attention"))
                .and_then(|a| a.as_str())
                .unwrap_or("");

            let lvl = match level {
                "error" => level.bright_red().bold(),
                "warn" => level.bright_yellow(),
                "debug" => level.bright_black(),
                _ => level.bright_cyan(),
            };
            let att = format_attention(attention);
            println!(
                "{}  {:>5}  {:<20} {:<15}  {:<10}  {}",
                ts.bright_black(),
                lvl,
                ev.bright_white(),
                stage.bright_magenta(),
                att,
                message
            );
        }
        Err(_) => {
            println!("{}", line);
        }
    }
}

/// Render an attention tag with state-appropriate color. Empty input returns
/// padded blank space so the column stays aligned for events that don't carry
/// an attention signal (e.g. RunStart, MovementStart).
fn format_attention(attention: &str) -> colored::ColoredString {
    use colored::Colorize;
    match attention {
        "done" => "done".bright_green().bold(),
        "error" => "error".bright_red().bold(),
        "waiting" => "waiting".bright_yellow().bold(),
        "running" => "running".bright_cyan(),
        "idle" => "idle".bright_black(),
        _ => "".normal(),
    }
}

#[cfg(test)]
mod tests {
    use super::format_attention;

    #[test]
    fn known_states_render_non_empty() {
        for state in ["done", "error", "waiting", "running", "idle"] {
            let rendered = format_attention(state);
            // Strip ANSI codes by checking the underlying string slice.
            assert!(
                rendered.to_string().contains(state),
                "expected rendered '{state}' to contain its label"
            );
        }
    }

    #[test]
    fn unknown_state_renders_empty() {
        // Caller pads with a fixed-width column, so an unknown state should
        // emit an empty label rather than a fallback word.
        assert_eq!(format_attention("").to_string(), "");
        assert_eq!(format_attention("garbage").to_string(), "");
    }
}
