use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_config(&self, action: &ConfigAction) -> Result<()> {
        match action {
            ConfigAction::Generate { output, template } => {
                let config = match template.as_str() {
                    "minimal" => create_minimal_config(&self.repo_path)?,
                    "frontend-only" => create_frontend_only_config(&self.repo_path)?,
                    "full-stack" => create_default_config(&self.repo_path)?,
                    _ => create_default_config(&self.repo_path)?,
                };

                config.to_file(output.clone()).await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Configuration generated",
                            "file": output,
                            "template": template,
                        }))?
                    );
                } else {
                    println!("✅ Configuration generated: {}", output.display());
                    println!("   Template: {}", template);
                }
            }
            ConfigAction::Validate { file } => match CcswarmConfig::from_file(file.clone()).await {
                Ok(_) => {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "Configuration is valid",
                                "file": file,
                            }))?
                        );
                    } else {
                        println!("✅ Configuration is valid: {}", file.display());
                    }
                }
                Err(e) => {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Configuration is invalid",
                                "file": file,
                                "error": e.to_string(),
                            }))?
                        );
                    } else {
                        println!("❌ Configuration is invalid: {}", file.display());
                        println!("   Error: {}", e);
                    }
                    return Err(e);
                }
            },
            ConfigAction::Show { file, agent, raw } => {
                let raw_contents = match tokio::fs::read_to_string(file.clone()).await {
                    Ok(contents) => contents,
                    Err(e) => {
                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "error",
                                    "message": "Failed to read configuration file",
                                    "file": file,
                                    "error": e.to_string(),
                                }))?
                            );
                        } else {
                            println!("❌ Failed to read configuration {}: {}", file.display(), e);
                        }
                        return Err(e.into());
                    }
                };

                let config: CcswarmConfig = match serde_json::from_str(&raw_contents) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "error",
                                    "message": "Configuration file is not valid JSON",
                                    "file": file,
                                    "error": e.to_string(),
                                }))?
                            );
                        } else {
                            println!(
                                "❌ Configuration {} is not valid JSON: {}",
                                file.display(),
                                e
                            );
                        }
                        return Err(e.into());
                    }
                };

                if *raw && agent.is_none() {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "file": file,
                                "raw": raw_contents,
                            }))?
                        );
                    } else {
                        println!("{}", raw_contents);
                    }
                    return Ok(());
                }

                if let Some(agent_name) = agent {
                    match config.agents.get(agent_name) {
                        Some(agent_cfg) => {
                            if self.json_output {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "status": "success",
                                        "file": file,
                                        "agent": agent_name,
                                        "config": agent_cfg,
                                    }))?
                                );
                            } else if *raw {
                                println!("{}", serde_json::to_string_pretty(agent_cfg)?);
                            } else {
                                println!("📄 Agent configuration: {}", agent_name);
                                println!("  Specialization: {}", agent_cfg.specialization);
                                println!("  Worktree: {}", agent_cfg.worktree);
                                println!("  Branch: {}", agent_cfg.branch);
                                println!(
                                    "  Think mode: {}",
                                    agent_cfg
                                        .claude_config
                                        .think_mode
                                        .as_ref()
                                        .map(|m| m.to_string())
                                        .unwrap_or_else(|| "default".to_string())
                                );
                                println!(
                                    "  Custom commands: {}",
                                    if agent_cfg.claude_config.custom_commands.is_empty() {
                                        "none".to_string()
                                    } else {
                                        agent_cfg.claude_config.custom_commands.join(", ")
                                    }
                                );
                            }
                        }
                        None => {
                            let err_msg =
                                format!("Agent '{}' not found in configuration", agent_name);
                            if self.json_output {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "status": "error",
                                        "message": err_msg,
                                        "file": file,
                                    }))?
                                );
                            } else {
                                println!("❌ {}", err_msg);
                            }
                            return Err(anyhow::anyhow!(err_msg));
                        }
                    }
                } else if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "file": file,
                            "config": config,
                        }))?
                    );
                } else {
                    println!("📄 Configuration: {}", file.display());
                    println!("Project: {}", config.project.name);
                    println!(
                        "Repository: {} (branch: {})",
                        config.project.repository.url, config.project.repository.main_branch
                    );
                    println!(
                        "Master Claude role: {}, think mode: {}",
                        config.project.master_claude.role, config.project.master_claude.think_mode
                    );
                    println!("Agents ({}):", config.agents.len());
                    for (name, agent) in &config.agents {
                        println!(
                            "  - {} [{}] -> {}",
                            name, agent.specialization, agent.worktree
                        );
                    }
                    println!(
                        "Coordination: method={}, sync={}s, quality_gate={}, master_review={}",
                        config.coordination.communication_method,
                        config.coordination.sync_interval,
                        config.coordination.quality_gate_frequency,
                        config.coordination.master_review_trigger
                    );
                }
            }
        }

        Ok(())
    }

}
