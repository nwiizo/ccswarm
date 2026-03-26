#![allow(dead_code)]

use super::super::*;
use anyhow::{Context, Result};
use colored::Colorize;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone, Deserialize, Default)]
struct HarnessAssert {
    #[serde(default)]
    expect_success: bool,
    #[serde(default)]
    expect_text: Vec<String>,
    #[serde(default)]
    forbid_text: Vec<String>,
    #[serde(default)]
    files_exist: Vec<String>,
    #[serde(default)]
    command_ok: Vec<String>,
    #[serde(default)]
    metrics: Option<AssertMetrics>,
}

#[derive(Debug, Clone, Deserialize)]
struct HarnessScenario {
    name: Option<String>,
    piece: String,
    task: String,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
    #[serde(default = "default_output_format")]
    output_format: String,
    #[serde(default)]
    assert: HarnessAssert,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    matrix: Option<serde_json::Map<String, serde_json::Value>>, // simple string arrays
    #[serde(default)]
    setup: Vec<String>,
    #[serde(default)]
    teardown: Vec<String>,
}

fn default_timeout() -> u64 {
    600
}
fn default_output_format() -> String {
    "text".to_string()
}

impl CliRunner {
    pub(crate) async fn handle_harness(&self, action: &HarnessAction) -> Result<()> {
        match action {
            HarnessAction::Run {
                scenario,
                dir,
                report,
                format,
                jobs,
            } => {
                let mut scenarios = Vec::new();

                if let Some(path) = scenario {
                    scenarios.push(load_scenario(path)?);
                }

                if let Some(dir) = dir {
                    scenarios.extend(load_scenarios_from_dir(dir)?);
                }

                if scenarios.is_empty() {
                    let default_dir = self.repo_path.join(".ccswarm").join("harness");
                    if default_dir.exists() {
                        scenarios.extend(load_scenarios_from_dir(&default_dir)?);
                    } else {
                        anyhow::bail!(
                            "No scenarios found. Use --scenario/--dir or create .ccswarm/harness/"
                        );
                    }
                }

                let mut results = Vec::new();
                let mut all_pass = true;

                // Build job list
                let mut jobs_vec: Vec<(
                    PathBuf,
                    HarnessScenario,
                    std::collections::HashMap<String, String>,
                )> = Vec::new();
                for (path, sc) in scenarios {
                    let varsets = match &sc.matrix {
                        Some(m) => expand_matrix(m)?,
                        None => vec![std::collections::HashMap::new()],
                    };
                    for vars in varsets {
                        jobs_vec.push((path.clone(), sc.clone(), vars));
                    }
                }

                // Determine concurrency
                let auto = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4);
                let concurrency = if *jobs == 0 { auto } else { *jobs };
                info!(
                    "Harness executing {} instances (concurrency: {})",
                    jobs_vec.len(),
                    concurrency
                );

                // Run with bounded parallelism
                let mut stream = futures::stream::iter(jobs_vec.into_iter().map(
                    |(path, sc, vars)| async move {
                        let res = self.run_single_scenario_instance(&sc, &vars).await;
                        (path, sc, vars, res)
                    },
                ))
                .buffer_unordered(concurrency);

                while let Some((path, sc, vars, res)) = stream.next().await {
                    match res {
                        Ok((ok, details)) => {
                            if !ok {
                                all_pass = false;
                            }
                            results.push(serde_json::json!({
                                "path": path.display().to_string(),
                                "name": sc.name.clone().unwrap_or_default(),
                                "piece": substitute_vars(&sc.piece, &vars),
                                "task": substitute_vars(&sc.task, &vars),
                                "vars": vars,
                                "ok": ok,
                                "details": details,
                            }));
                        }
                        Err(e) => {
                            all_pass = false;
                            results.push(serde_json::json!({
                                "path": path.display().to_string(),
                                "name": sc.name.clone().unwrap_or_default(),
                                "piece": substitute_vars(&sc.piece, &vars),
                                "task": substitute_vars(&sc.task, &vars),
                                "vars": vars,
                                "ok": false,
                                "details": {"error": e.to_string()},
                            }));
                        }
                    }
                }

                let report_obj = serde_json::json!({
                    "status": if all_pass { "success" } else { "error" },
                    "message": if all_pass { "All scenarios passed" } else { "Some scenarios failed" },
                    "data": { "results": results }
                });

                if let Some(out) = report {
                    if format == "json" {
                        fs::create_dir_all(Path::new(out).parent().unwrap_or(Path::new(".")))?;
                        fs::write(out, serde_json::to_string_pretty(&report_obj)?).with_context(
                            || format!("Failed writing report to {}", out.display()),
                        )?;
                        println!(
                            "{} Wrote report to {}",
                            "OK".bright_green().bold(),
                            out.display()
                        );
                    } else {
                        let mut buf = String::new();
                        buf.push_str("# Harness Report\n\n");
                        if let Some(arr) = report_obj["data"]["results"].as_array() {
                            for r in arr {
                                let ok = r["ok"].as_bool().unwrap_or(false);
                                let mark = if ok { "✅" } else { "❌" };
                                buf.push_str(&format!(
                                    "{} {} — {}\n",
                                    mark,
                                    r["path"].as_str().unwrap_or(""),
                                    r["task"].as_str().unwrap_or("")
                                ));
                            }
                        }
                        fs::create_dir_all(Path::new(out).parent().unwrap_or(Path::new(".")))?;
                        fs::write(out, buf).with_context(|| {
                            format!("Failed writing report to {}", out.display())
                        })?;
                        println!(
                            "{} Wrote report to {}",
                            "OK".bright_green().bold(),
                            out.display()
                        );
                    }
                } else if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&report_obj)?);
                } else {
                    let status = if all_pass {
                        "SUCCESS".bright_green().bold()
                    } else {
                        "FAIL".bright_red().bold()
                    };
                    println!("Harness: {}", status);
                    if let Some(arr) = report_obj["data"]["results"].as_array() {
                        for r in arr {
                            let ok = r["ok"].as_bool().unwrap_or(false);
                            let mark = if ok { "✅" } else { "❌" };
                            println!(
                                "{} {} — {}",
                                mark,
                                r["path"].as_str().unwrap_or(""),
                                r["task"].as_str().unwrap_or("")
                            );
                        }
                    }
                }

                if !all_pass {
                    std::process::exit(1);
                }
            }
            HarnessAction::List => {
                let default_dir = self.repo_path.join(".ccswarm").join("harness");
                if !default_dir.exists() {
                    println!("No scenarios found. Create {}", default_dir.display());
                    return Ok(());
                }
                let scenarios = load_scenarios_from_dir(&default_dir)?;
                for (path, sc) in scenarios {
                    println!(
                        "- {} (piece: {}, task: {})",
                        path.display(),
                        sc.piece,
                        sc.task
                    );
                }
            }
            HarnessAction::Init { output, name } => {
                let default_dir = self.repo_path.join(".ccswarm").join("harness");
                let out_path = output.clone().unwrap_or(default_dir.join("sample.yaml"));
                let parent = out_path.parent().unwrap_or(std::path::Path::new("."));
                fs::create_dir_all(parent)?;
                let content = format!(
                    "# ccswarm harness scenario\nname: \"{}\"\n\npiece: \"default\"\ntask: \"Create login form [feature]\"\n\n# optional: timeout_secs: 600\n# optional: output_format: text\n\nsetup:\n  - \"git checkout main\"\n\nassert:\n  expect_success: true\n  expect_text: [\"Plan\", \"Review\"]\n  forbid_text: [\"ABORT\"]\n  files_exist: [\"README.md\"]\n  command_ok: [\"cargo check -q\"]\n  metrics:\n    max_duration_secs: 300\n\n# teardown:\n#   - \"git reset --hard && git clean -fdx\"\n",
                    name
                );
                fs::write(&out_path, content)?;
                println!(
                    "{} Created {}",
                    "OK".bright_green().bold(),
                    out_path.display()
                );
            }
            HarnessAction::Plan { scenario, dir } => {
                let mut scenarios = Vec::new();
                if let Some(path) = scenario {
                    scenarios.push(load_scenario(path)?);
                }
                if let Some(dir) = dir {
                    scenarios.extend(load_scenarios_from_dir(dir)?);
                }
                if scenarios.is_empty() {
                    let default_dir = self.repo_path.join(".ccswarm").join("harness");
                    scenarios.extend(load_scenarios_from_dir(&default_dir)?);
                }
                let mut total = 0usize;
                for (path, sc) in scenarios {
                    let varsets = match &sc.matrix {
                        Some(m) => expand_matrix(m)?,
                        None => vec![std::collections::HashMap::new()],
                    };
                    println!("Scenario: {} ({} combos)", path.display(), varsets.len());
                    for vars in varsets {
                        total += 1;
                        println!(
                            "  - piece={} task={} vars={}",
                            substitute_vars(&sc.piece, &vars),
                            substitute_vars(&sc.task, &vars),
                            serde_json::to_string(&vars).unwrap_or_default()
                        );
                    }
                }
                println!("Total instances: {}", total);
            }
            HarnessAction::Diff {
                baseline,
                scenario,
                dir,
                format,
            } => {
                // Load baseline
                let base_str = std::fs::read_to_string(baseline)
                    .with_context(|| format!("Failed to read baseline {}", baseline.display()))?;
                let base_json: serde_json::Value =
                    serde_json::from_str(&base_str).context("Invalid baseline JSON")?;
                let base_map = index_results(&base_json);

                // Produce current results by running
                let mut tmp_results = Vec::new();
                let mut scenarios = Vec::new();
                if let Some(path) = scenario {
                    scenarios.push(load_scenario(path)?);
                }
                if let Some(d) = dir {
                    scenarios.extend(load_scenarios_from_dir(d)?);
                }
                if scenarios.is_empty() {
                    let default_dir = self.repo_path.join(".ccswarm").join("harness");
                    scenarios.extend(load_scenarios_from_dir(&default_dir)?);
                }
                for (_path, sc) in scenarios {
                    let varsets = match &sc.matrix {
                        Some(m) => expand_matrix(m)?,
                        None => vec![std::collections::HashMap::new()],
                    };
                    for vars in varsets {
                        let (ok, details) = self.run_single_scenario_instance(&sc, &vars).await?;
                        tmp_results.push(serde_json::json!({
                            "name": sc.name.clone().unwrap_or_default(),
                            "piece": substitute_vars(&sc.piece, &vars),
                            "task": substitute_vars(&sc.task, &vars),
                            "vars": vars,
                            "ok": ok,
                            "details": details,
                        }));
                    }
                }
                let curr = serde_json::json!({"data": {"results": tmp_results}});
                let curr_map = index_results(&curr);

                // Compute diff
                let mut regressions = Vec::new();
                let mut improvements = Vec::new();
                let mut added = Vec::new();
                let mut removed = Vec::new();
                for (k, v) in &curr_map {
                    match base_map.get(k) {
                        Some(b) => {
                            let b_ok = b["ok"].as_bool().unwrap_or(false);
                            let c_ok = v["ok"].as_bool().unwrap_or(false);
                            if b_ok && !c_ok {
                                regressions.push(k.clone());
                            }
                            if !b_ok && c_ok {
                                improvements.push(k.clone());
                            }
                        }
                        None => added.push(k.clone()),
                    }
                }
                for k in base_map.keys() {
                    if !curr_map.contains_key(k) {
                        removed.push(k.clone());
                    }
                }
                let summary = serde_json::json!({
                    "regressions": regressions,
                    "improvements": improvements,
                    "added": added,
                    "removed": removed,
                });
                if format == "json" || self.json_output {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else if format == "markdown" {
                    let md = render_diff_markdown(&summary, &curr);
                    println!("{}", md);
                } else {
                    println!(
                        "Regressions: {}",
                        summary["regressions"]
                            .as_array()
                            .map(|a| a.len())
                            .unwrap_or(0)
                    );
                    println!(
                        "Improvements: {}",
                        summary["improvements"]
                            .as_array()
                            .map(|a| a.len())
                            .unwrap_or(0)
                    );
                    println!(
                        "Added: {}",
                        summary["added"].as_array().map(|a| a.len()).unwrap_or(0)
                    );
                    println!(
                        "Removed: {}",
                        summary["removed"].as_array().map(|a| a.len()).unwrap_or(0)
                    );
                }
                // Non-zero exit on regressions
                if let Some(arr) = summary["regressions"].as_array()
                    && !arr.is_empty()
                {
                    std::process::exit(1);
                }
            }
            HarnessAction::Approve {
                report,
                baseline,
                force,
            } => {
                if baseline.exists() && !force {
                    anyhow::bail!("Baseline exists. Use --force to overwrite");
                }
                std::fs::create_dir_all(baseline.parent().unwrap_or(std::path::Path::new(".")))?;
                std::fs::copy(report, baseline).with_context(|| "Failed to write baseline")?;
                println!("{} Baseline updated", "OK".bright_green().bold());
            }
        }

        Ok(())
    }

    async fn run_single_scenario_instance(
        &self,
        sc: &HarnessScenario,
        vars: &std::collections::HashMap<String, String>,
    ) -> Result<(bool, serde_json::Value)> {
        use crate::workflow::pipeline::{PipelineConfig, PipelineRunner};

        // Run setup commands (in repo cwd)
        for cmd in &sc.setup {
            let cmd_line = substitute_vars(cmd, vars);
            run_shell_in_cwd(&cmd_line, &self.repo_path)?;
        }

        let task_text = substitute_vars(&sc.task, vars);
        let piece_name = substitute_vars(&sc.piece, vars);

        let started = std::time::Instant::now();
        let config = PipelineConfig::builder()
            .piece_name(&piece_name)
            .task_text(&task_text)
            .output_format(&sc.output_format)
            .timeout(Duration::from_secs(sc.timeout_secs))
            .verbose(false)
            .build()
            .context("Failed to build pipeline configuration")?;

        let runner = PipelineRunner::new();
        let result = runner.execute(config).await?;
        let duration_secs = started.elapsed().as_secs();

        let formatted = match sc.output_format.as_str() {
            "json" => result.format_json()?,
            "markdown" => result.format_markdown(),
            _ => result.format_text(),
        };

        let mut ok = true;
        let mut violations: Vec<String> = vec![];

        if sc.assert.expect_success && !result.is_success() {
            ok = false;
            violations.push("Pipeline did not succeed".to_string());
        }

        for needle in &sc.assert.expect_text {
            if !formatted.contains(needle) {
                ok = false;
                violations.push(format!("Missing text: '{}'", needle));
            }
        }

        for needle in &sc.assert.forbid_text {
            if formatted.contains(needle) {
                ok = false;
                violations.push(format!("Forbidden text present: '{}'", needle));
            }
        }

        // File existence checks
        for p in &sc.assert.files_exist {
            let pp = self.repo_path.join(substitute_vars(p, vars));
            if !pp.exists() {
                ok = false;
                violations.push(format!("File/dir not found: {}", pp.display()));
            }
        }

        // Command checks (best effort)
        for cmd in &sc.assert.command_ok {
            let cmd_line = substitute_vars(cmd, vars);
            if let Err(e) = run_shell_in_cwd(&cmd_line, &self.repo_path) {
                ok = false;
                violations.push(format!("Command failed: '{}': {}", cmd_line, e));
            }
        }

        // Metrics bounds
        if let Some(m) = &sc.assert.metrics
            && let Some(max_dur) = m.max_duration_secs
            && duration_secs > max_dur
        {
            ok = false;
            violations.push(format!(
                "Exceeded max_duration_secs: {}s > {}s",
                duration_secs, max_dur
            ));
        }

        // Always attempt teardown (best effort)
        for cmd in &sc.teardown {
            let cmd_line = substitute_vars(cmd, vars);
            if let Err(e) = run_shell_in_cwd(&cmd_line, &self.repo_path) {
                warn!("Teardown failed: {} => {}", cmd_line, e);
            }
        }

        // Try to summarize latest events.ndjson under .ccswarm/runs (best effort)
        let events_summary = find_and_summarize_recent_events(&self.repo_path, 180)?;

        let details = serde_json::json!({
            "success": result.is_success(),
            "exit_code": result.exit_code().as_code(),
            "duration_secs": duration_secs,
            "violations": violations,
            "events": events_summary,
        });

        Ok((ok, details))
    }
}

fn load_scenario(path: &Path) -> Result<(PathBuf, HarnessScenario)> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let sc: HarnessScenario = serde_yaml::from_str(&content)
        .with_context(|| format!("Invalid YAML in {}", path.display()))?;
    Ok((path.to_path_buf(), sc))
}

fn load_scenarios_from_dir(dir: &Path) -> Result<Vec<(PathBuf, HarnessScenario)>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map(|e| e == "yaml" || e == "yml")
            .unwrap_or(false)
        {
            if let Ok(sc) = load_scenario(&path) {
                out.push(sc);
            } else {
                warn!("Skipping invalid scenario: {}", path.display());
            }
        }
    }
    Ok(out)
}

fn index_results(root: &serde_json::Value) -> std::collections::HashMap<String, serde_json::Value> {
    let mut map = std::collections::HashMap::new();
    if let Some(arr) = root
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.as_array())
    {
        for item in arr {
            let piece = item.get("piece").and_then(|v| v.as_str()).unwrap_or("");
            let task = item.get("task").and_then(|v| v.as_str()).unwrap_or("");
            let vars = item.get("vars").cloned().unwrap_or(serde_json::json!({}));
            let key = format!(
                "{}|{}|{}",
                piece,
                task,
                serde_json::to_string(&vars).unwrap_or_default()
            );
            map.insert(key, item.clone());
        }
    }
    map
}

fn find_and_summarize_recent_events(
    repo: &Path,
    within_secs: u64,
) -> Result<Option<serde_json::Value>> {
    use std::time::{Duration, SystemTime};
    use walkdir::WalkDir;
    let runs_dir = repo.join(".ccswarm").join("runs");
    if !runs_dir.exists() {
        return Ok(None);
    }
    let now = SystemTime::now();
    let mut latest_path: Option<PathBuf> = None;
    let mut latest_mtime = SystemTime::UNIX_EPOCH;
    for entry in WalkDir::new(&runs_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "events.ndjson"
            && let Ok(meta) = entry.metadata()
            && let Ok(mtime) = meta.modified()
            && now
                .duration_since(mtime)
                .unwrap_or(Duration::from_secs(u64::MAX))
                .as_secs()
                <= within_secs
            && mtime > latest_mtime
        {
            latest_mtime = mtime;
            latest_path = Some(entry.path().to_path_buf());
        }
    }
    if let Some(path) = latest_path {
        Ok(Some(summarize_events_file(&path)?))
    } else {
        Ok(None)
    }
}

fn summarize_events_file(path: &Path) -> Result<serde_json::Value> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut total = 0u64;
    let mut by_level: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    for l in reader.lines().map_while(Result::ok) {
        if l.trim().is_empty() {
            continue;
        }
        total += 1;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&l) {
            if let Some(level) = v.get("level").and_then(|x| x.as_str()) {
                *by_level.entry(level.to_string()).or_insert(0) += 1;
            }
            if let Some(ts) = v.get("ts").and_then(|x| x.as_str()) {
                if first_ts.is_none() {
                    first_ts = Some(ts.to_string());
                }
                last_ts = Some(ts.to_string());
            }
        }
    }
    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "total": total,
        "levels": by_level,
        "first_ts": first_ts,
        "last_ts": last_ts,
    }))
}

fn render_diff_markdown(summary: &serde_json::Value, current: &serde_json::Value) -> String {
    fn list_section(title: &str, keys: Option<&Vec<serde_json::Value>>) -> String {
        let mut s = String::new();
        s.push_str(&format!("\n### {}\n", title));
        if let Some(arr) = keys {
            if arr.is_empty() {
                s.push_str("- (none)\n");
                return s;
            }
            for k in arr {
                s.push_str(&format!("- {}\n", k.as_str().unwrap_or("")));
            }
        } else {
            s.push_str("- (none)\n");
        }
        s
    }
    let mut out = String::new();
    out.push_str("## Harness Diff Report\n\n");
    let regs = summary.get("regressions").and_then(|v| v.as_array());
    let imps = summary.get("improvements").and_then(|v| v.as_array());
    let adds = summary.get("added").and_then(|v| v.as_array());
    let rems = summary.get("removed").and_then(|v| v.as_array());
    out.push_str(&format!(
        "Summary: regressions={} | improvements={} | added={} | removed={}\n\n",
        regs.map(|a| a.len()).unwrap_or(0),
        imps.map(|a| a.len()).unwrap_or(0),
        adds.map(|a| a.len()).unwrap_or(0),
        rems.map(|a| a.len()).unwrap_or(0)
    ));
    out.push_str(&list_section("Regressions", regs));
    out.push_str(&list_section("Improvements", imps));
    out.push_str(&list_section("Added", adds));
    out.push_str(&list_section("Removed", rems));

    if let Some(results) = current
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.as_array())
    {
        out.push_str("\n### Current Results (subset)\n\n");
        out.push_str("| piece | task | ok |\n|---|---|---|\n");
        for r in results.iter().take(30) {
            let piece = r.get("piece").and_then(|v| v.as_str()).unwrap_or("");
            let task = r.get("task").and_then(|v| v.as_str()).unwrap_or("");
            let ok = if r.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                "✅"
            } else {
                "❌"
            };
            out.push_str(&format!("| {} | {} | {} |\n", piece, task, ok));
        }
        if results.len() > 30 {
            out.push_str(&format!("\n(_{} more not shown_)\n", results.len() - 30));
        }
    }
    out
}

#[derive(Debug, Deserialize, Default, Clone)]
struct AssertMetrics {
    #[serde(default)]
    max_duration_secs: Option<u64>,
    #[serde(default)]
    max_tokens: Option<u64>,
}

fn expand_matrix(
    matrix: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<std::collections::HashMap<String, String>>> {
    let mut dims: Vec<(String, Vec<String>)> = Vec::new();
    for (k, v) in matrix {
        let arr = v
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("matrix entry for '{}' must be an array", k))?;
        let vals: Vec<String> = arr
            .iter()
            .map(|x| x.as_str().unwrap_or("").to_string())
            .collect();
        dims.push((k.clone(), vals));
    }
    // Cartesian product
    let mut out: Vec<std::collections::HashMap<String, String>> =
        vec![std::collections::HashMap::new()];
    for (k, vals) in dims {
        let mut next = Vec::new();
        for base in &out {
            for val in &vals {
                let mut m = base.clone();
                m.insert(k.clone(), val.clone());
                next.push(m);
            }
        }
        out = next;
    }
    Ok(out)
}

fn substitute_vars(input: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut s = input.to_string();
    for (k, v) in vars {
        let needle = format!("${{{}}}", k);
        s = s.replace(&needle, v);
    }
    s
}

fn run_shell(cmd_line: &str) -> Result<()> {
    use std::process::Command;
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd_line]).status()
    } else {
        Command::new("sh").args(["-c", cmd_line]).status()
    }?;
    if !status.success() {
        anyhow::bail!("Command failed with status: {:?}", status);
    }
    Ok(())
}

fn run_shell_in_cwd(cmd_line: &str, cwd: &Path) -> Result<()> {
    use std::process::Command;
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir(cwd)
            .args(["/C", cmd_line])
            .status()
    } else {
        Command::new("sh")
            .current_dir(cwd)
            .args(["-c", cmd_line])
            .status()
    }?;
    if !status.success() {
        anyhow::bail!("Command failed with status: {:?}", status);
    }
    Ok(())
}
