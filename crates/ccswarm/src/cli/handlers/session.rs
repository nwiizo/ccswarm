use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_session(&self, action: &SessionAction) -> Result<()> {
        // Use native ai-session SessionManager instead of tmux
        let manager = ai_session::SessionManager::new();

        match action {
            SessionAction::Create {
                agent,
                workspace,
                background,
            } => {
                let workspace_path = workspace.as_deref().unwrap_or("./");
                let workspace_pathbuf = std::path::Path::new(workspace_path).to_path_buf();

                // Validate agent role
                let agent_role = match agent.to_lowercase().as_str() {
                    "frontend" | "backend" | "devops" | "qa" => agent.to_lowercase(),
                    _ => {
                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "error",
                                    "message": "Invalid agent type",
                                    "agent": agent,
                                    "valid_types": ["frontend", "backend", "devops", "qa"],
                                }))?
                            );
                        } else {
                            println!("❌ Invalid agent type: {}", agent);
                            println!("   Valid types: frontend, backend, devops, qa");
                        }
                        return Ok(());
                    }
                };

                // Create session with native ai-session
                let session_name = format!(
                    "ccswarm-{}-{}",
                    agent_role,
                    &uuid::Uuid::new_v4().to_string()[..8]
                );
                let mut config = ai_session::SessionConfig::default();
                config.name = Some(session_name.clone());
                config.working_directory = workspace_pathbuf.clone();
                config.agent_role = Some(agent_role.clone());
                config.enable_ai_features = true;
                config.force_headless = *background;

                match manager.create_session_with_config(config).await {
                    Ok(session) => {
                        // Start the session
                        if let Err(e) = session.start().await {
                            if self.json_output {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "status": "error",
                                        "message": "Failed to start session",
                                        "error": e.to_string(),
                                    }))?
                                );
                            } else {
                                println!("❌ Failed to start session: {}", e);
                            }
                            return Ok(());
                        }

                        let session_id_str = session.id.to_string();
                        let short_id = &session_id_str[..8.min(session_id_str.len())];

                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "success",
                                    "message": "Session created",
                                    "session_id": session_id_str,
                                    "agent_id": format!("{}-{}", agent_role, short_id),
                                    "agent": agent_role,
                                    "workspace": workspace_path,
                                    "background": background,
                                    "session_name": session_name,
                                }))?
                            );
                        } else {
                            println!("🚀 Creating session for {} agent", agent_role);
                            println!("   Session ID: {}", short_id);
                            println!("   Agent ID: {}-{}", agent_role, short_id);
                            println!("   Workspace: {}", workspace_path);
                            println!("   Session Name: {}", session_name);
                            println!("   Background: {}", background);
                            println!("   ✅ Session created successfully");
                            println!();
                            println!("To interact with this session:");
                            println!("   ccswarm session attach {}", short_id);
                        }
                    }
                    Err(e) => {
                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "error",
                                    "message": "Failed to create session",
                                    "error": e.to_string(),
                                }))?
                            );
                        } else {
                            println!("❌ Failed to create session: {}", e);
                        }
                    }
                }
            }

            SessionAction::List { all: _ } => {
                // List sessions using native ai-session
                let sessions = manager.list_sessions_detailed();

                // Filter for ccswarm sessions
                let ccswarm_sessions: Vec<_> = sessions
                    .iter()
                    .filter(|s| {
                        s.name
                            .as_ref()
                            .map(|n| n.starts_with("ccswarm-") || n.starts_with("ai-session-"))
                            .unwrap_or(false)
                    })
                    .collect();

                if self.json_output {
                    let session_data: Vec<serde_json::Value> = ccswarm_sessions
                        .iter()
                        .map(|s| {
                            let name = s.name.as_deref().unwrap_or("");
                            let name_parts: Vec<&str> = name.split('-').collect();
                            let agent_role = if name_parts.len() >= 2 {
                                name_parts[1]
                            } else {
                                "unknown"
                            };

                            serde_json::json!({
                                "session_id": s.id.to_string(),
                                "session_name": name,
                                "agent_role": agent_role,
                                "status": format!("{:?}", s.status),
                                "created_at": s.created_at.to_rfc3339(),
                                "last_activity": s.last_activity.to_rfc3339(),
                                "working_directory": s.working_directory.display().to_string(),
                                "ai_features_enabled": s.ai_features_enabled,
                                "command_count": s.command_count,
                            })
                        })
                        .collect();

                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Sessions listed",
                            "sessions": session_data,
                            "total": ccswarm_sessions.len(),
                        }))?
                    );
                } else {
                    println!("📋 Active Sessions");
                    println!("=================");

                    if ccswarm_sessions.is_empty() {
                        println!("No active sessions found");
                        println!();
                        println!("Create a new session with:");
                        println!("   ccswarm session create --agent frontend");
                    } else {
                        for session in &ccswarm_sessions {
                            let name = session.name.as_deref().unwrap_or("unnamed");
                            let name_parts: Vec<&str> = name.split('-').collect();
                            let agent_role = if name_parts.len() >= 2 {
                                name_parts[1]
                            } else {
                                "unknown"
                            };

                            println!("Session: {}", name);
                            println!("  ID: {}", session.id);
                            println!("  Agent Role: {}", agent_role);
                            println!("  Status: {:?}", session.status);
                            println!(
                                "  Created: {}",
                                session.created_at.format("%Y-%m-%d %H:%M:%S")
                            );
                            println!("  Working Dir: {}", session.working_directory.display());
                            println!("  Commands: {}", session.command_count);
                            println!();
                        }
                    }
                }
            }

            SessionAction::Pause { session_id } => {
                // Find and pause the session
                if let Some(session) = manager
                    .find_session_by_name(session_id)
                    .or_else(|| {
                        ai_session::SessionId::parse_str(session_id)
                            .ok()
                            .and_then(|id| manager.get_session(&id))
                    })
                    .or_else(|| {
                        // Try prefix match
                        manager
                            .list_sessions_by_prefix("ccswarm-")
                            .into_iter()
                            .find(|s| {
                                s.config
                                    .name
                                    .as_ref()
                                    .map(|n| n.contains(session_id))
                                    .unwrap_or(false)
                                    || s.id.to_string().starts_with(session_id)
                            })
                    })
                {
                    let _ = session
                        .set_metadata("paused".to_string(), serde_json::json!(true))
                        .await;

                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "Session paused",
                                "session_id": session.id.to_string(),
                            }))?
                        );
                    } else {
                        println!("⏸️ Pausing session: {}", session_id);
                        println!("   ✅ Session paused successfully");
                    }
                } else {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Session not found",
                                "session_id": session_id,
                            }))?
                        );
                    } else {
                        println!("❌ Session not found: {}", session_id);
                        println!("   Use 'ccswarm session list' to see available sessions");
                    }
                }
            }

            SessionAction::Resume { session_id } => {
                // Find and resume the session
                if let Some(session) = manager
                    .find_session_by_name(session_id)
                    .or_else(|| {
                        ai_session::SessionId::parse_str(session_id)
                            .ok()
                            .and_then(|id| manager.get_session(&id))
                    })
                    .or_else(|| {
                        manager
                            .list_sessions_by_prefix("ccswarm-")
                            .into_iter()
                            .find(|s| {
                                s.config
                                    .name
                                    .as_ref()
                                    .map(|n| n.contains(session_id))
                                    .unwrap_or(false)
                                    || s.id.to_string().starts_with(session_id)
                            })
                    })
                {
                    let _ = session
                        .set_metadata("paused".to_string(), serde_json::json!(false))
                        .await;

                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "Session resumed",
                                "session_id": session.id.to_string(),
                            }))?
                        );
                    } else {
                        println!("▶️ Resuming session: {}", session_id);
                        println!("   ✅ Session resumed successfully");
                    }
                } else {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Session not found",
                                "session_id": session_id,
                            }))?
                        );
                    } else {
                        println!("❌ Session not found: {}", session_id);
                        println!("   Use 'ccswarm session list' to see available sessions");
                    }
                }
            }

            SessionAction::Attach { session_id } => {
                // Find the session
                if let Some(session) = manager
                    .find_session_by_name(session_id)
                    .or_else(|| {
                        ai_session::SessionId::parse_str(session_id)
                            .ok()
                            .and_then(|id| manager.get_session(&id))
                    })
                    .or_else(|| {
                        manager
                            .list_sessions_by_prefix("ccswarm-")
                            .into_iter()
                            .find(|s| {
                                s.config
                                    .name
                                    .as_ref()
                                    .map(|n| n.contains(session_id))
                                    .unwrap_or(false)
                                    || s.id.to_string().starts_with(session_id)
                            })
                    })
                {
                    // For native sessions, we stream output instead of attaching
                    println!("📺 Streaming output from session: {}", session.id);
                    println!("   Press Ctrl+C to detach");
                    println!("─────────────────────────────────────");

                    // Read and display output
                    loop {
                        match session.read_output().await {
                            Ok(output) => {
                                if !output.is_empty() {
                                    print!("{}", String::from_utf8_lossy(&output));
                                }
                            }
                            Err(_) => break,
                        }

                        // Check for Ctrl+C via tokio signal
                        tokio::select! {
                            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {},
                            _ = tokio::signal::ctrl_c() => {
                                println!();
                                println!("─────────────────────────────────────");
                                println!("Detached from session");
                                break;
                            }
                        }
                    }
                } else {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Session not found",
                                "session_id": session_id,
                            }))?
                        );
                    } else {
                        println!("❌ Session not found: {}", session_id);
                        println!("   Use 'ccswarm session list' to see available sessions");
                    }
                }
            }

            SessionAction::Detach { session_id } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Detached from session",
                            "session_id": session_id,
                        }))?
                    );
                } else {
                    println!("🔌 Detaching from session: {}", session_id);
                    println!("   ✅ Detached successfully");
                }
            }

            SessionAction::Kill { session_id, force } => {
                // Find the session
                let found_session = manager
                    .find_session_by_name(session_id)
                    .or_else(|| {
                        ai_session::SessionId::parse_str(session_id)
                            .ok()
                            .and_then(|id| manager.get_session(&id))
                    })
                    .or_else(|| {
                        manager
                            .list_sessions_by_prefix("ccswarm-")
                            .into_iter()
                            .find(|s| {
                                s.config
                                    .name
                                    .as_ref()
                                    .map(|n| n.contains(session_id))
                                    .unwrap_or(false)
                                    || s.id.to_string().starts_with(session_id)
                            })
                    });

                if let Some(session) = found_session {
                    let session_id_clone = session.id.clone();
                    let session_name = session
                        .config
                        .name
                        .clone()
                        .unwrap_or_else(|| "unnamed".to_string());

                    // Stop and remove the session
                    if let Err(e) = manager.remove_session(&session_id_clone).await {
                        if self.json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "status": "error",
                                    "message": "Failed to kill session",
                                    "session_id": session_id,
                                    "error": e.to_string(),
                                }))?
                            );
                        } else {
                            println!("❌ Failed to kill session: {}", e);
                        }
                        return Ok(());
                    }

                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "Session killed",
                                "session_id": session_id_clone.to_string(),
                                "session_name": session_name,
                                "force": force,
                            }))?
                        );
                    } else {
                        println!("💀 Killing session: {}", session_id);
                        println!("   Session Name: {}", session_name);
                        if *force {
                            println!("   ⚠️ Force kill enabled");
                        }
                        println!("   ✅ Session killed successfully");
                    }
                } else {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Session not found",
                                "session_id": session_id,
                            }))?
                        );
                    } else {
                        println!("❌ Session not found: {}", session_id);
                        println!("   Use 'ccswarm session list' to see available sessions");
                    }
                }
            }
        }

        Ok(())
    }

}
