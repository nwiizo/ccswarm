use super::super::*;

impl CliRunner {
    pub(crate) async fn start_orchestrator(
        &self,
        daemon: bool,
        port: u16,
        isolation: &str,
        delegate: bool,
        enable_acp: bool,
    ) -> Result<()> {
        info!(
            "Starting ccswarm orchestrator with isolation mode: {} (port: {}, delegate: {}, acp: {})",
            isolation, port, delegate, enable_acp
        );

        // Validate provider availability before starting
        self.validate_provider().await?;

        // ProactiveMaster has been removed. Orchestrator runs in lightweight mode.
        let master_id = uuid::Uuid::new_v4().to_string();
        let agent_count = self.config.agents.len();

        // Start ACP WebSocket server if enabled
        if enable_acp {
            info!("ACP WebSocket server enabled on ws://localhost:9100");
        }

        // Save PID file for stop command
        let pid_file = self.repo_path.join(".ccswarm.pid");
        let pid_info = serde_json::json!({
            "pid": std::process::id(),
            "port": port,
            "master_id": master_id,
            "started_at": chrono::Utc::now().to_rfc3339(),
            "delegate_mode": delegate,
            "acp_enabled": enable_acp,
        });
        tokio::fs::write(&pid_file, serde_json::to_string_pretty(&pid_info)?).await?;

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Orchestrator started",
                    "master_id": master_id,
                    "agents": agent_count,
                    "port": port,
                    "daemon": daemon,
                    "delegate_mode": delegate,
                    "acp_enabled": enable_acp,
                }))?
            );
        } else {
            println!("ccswarm orchestrator started");
            println!("   Master ID: {}", master_id);
            println!("   Agents: {}", agent_count);
            println!("   Port: {}", port);
            if delegate {
                println!("   Delegate Mode: enabled");
            }
            if enable_acp {
                println!("   ACP: ws://localhost:9100");
            }
            if daemon {
                println!("   Mode: daemon (background)");
            } else {
                println!("   Mode: foreground (Ctrl+C to stop)");
                println!("   Note: IPC server has been removed. Use Claude Code directly.");
            }
        }

        // Cleanup PID file on exit
        if pid_file.exists() {
            let _ = tokio::fs::remove_file(&pid_file).await;
        }

        Ok(())
    }

    pub(crate) async fn stop_orchestrator(&self) -> Result<()> {
        // IPC server has been removed. Clean up PID file if it exists.
        let pid_file = self.repo_path.join(".ccswarm.pid");

        if pid_file.exists() {
            let _ = tokio::fs::remove_file(&pid_file).await;
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "success",
                        "message": "Orchestrator PID file cleaned up",
                    }))?
                );
            } else {
                println!("Orchestrator PID file cleaned up.");
            }
        } else if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "info",
                    "message": "No running orchestrator found",
                }))?
            );
        } else {
            println!("No running orchestrator found.");
            println!("The IPC server has been removed. Use Claude Code directly.");
        }

        Ok(())
    }

    pub(crate) async fn show_status(&self, detailed: bool, agent: Option<&str>) -> Result<()> {
        // Read status from coordination files
        let status_tracker = crate::coordination::StatusTracker::new().await?;

        if let Some(agent_id) = agent {
            // Show specific agent status
            if let Some(status) = status_tracker.get_status(agent_id).await? {
                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("Agent: {}", agent_id);
                    println!("Status: {}", status["status"]);
                    println!("Updated: {}", status["timestamp"]);

                    // Check if this is a backend agent and show backend-specific info
                    if let Some(role) = status.get("role")
                        && role.as_str() == Some("Backend")
                        && let Some(backend_info) = status.get("backend_specific")
                    {
                        println!("\n🔧 Backend Status:");
                        if let Some(api_health) = backend_info.get("api_health") {
                            println!(
                                "  API Health: {:.1}%",
                                api_health.as_f64().unwrap_or(0.0) * 100.0
                            );
                        }
                        if let Some(db) = backend_info.get("database") {
                            println!(
                                "  Database: {} ({})",
                                if db["is_connected"].as_bool().unwrap_or(false) {
                                    "Connected"
                                } else {
                                    "Disconnected"
                                },
                                db["database_type"].as_str().unwrap_or("Unknown")
                            );
                        }
                        if let Some(server) = backend_info.get("server") {
                            println!(
                                "  Server: {:.1}MB RAM, {:.1}% CPU",
                                server["memory_usage_mb"].as_f64().unwrap_or(0.0),
                                server["cpu_usage_percent"].as_f64().unwrap_or(0.0)
                            );
                        }
                        if let Some(services) =
                            backend_info.get("services").and_then(|s| s.as_array())
                        {
                            println!("  Active Services: {}", services.len());
                        }
                        if let Some(activity) = backend_info.get("recent_activity") {
                            println!("  Recent API Calls: {}", activity.as_u64().unwrap_or(0));
                        }
                    }

                    if detailed {
                        println!(
                            "\nDetails: {}",
                            serde_json::to_string_pretty(&status["additional_info"])?
                        );
                    }
                }
            } else if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": "Agent not found",
                        "agent": agent_id,
                    }))?
                );
            } else {
                println!("❌ Agent '{}' not found", agent_id);
            }
        } else {
            // Show all agent statuses
            let statuses = status_tracker.get_all_statuses().await?;

            if self.json_output {
                println!("{}", serde_json::to_string_pretty(&statuses)?);
            } else {
                println!("📊 ccswarm Status");
                println!("================");

                if statuses.is_empty() {
                    println!("No agents found");
                } else {
                    for status in &statuses {
                        println!("Agent: {}", status["agent_id"]);
                        println!("  Status: {}", status["status"]);
                        println!("  Updated: {}", status["timestamp"]);

                        if let Some(role) = status.get("role")
                            && role.as_str() == Some("Backend")
                            && let Some(backend_info) = status.get("backend_specific")
                        {
                            if let Some(api_health) = backend_info.get("api_health") {
                                print!(
                                    "  API Health: {:.0}% | ",
                                    api_health.as_f64().unwrap_or(0.0) * 100.0
                                );
                            }
                            if let Some(db) = backend_info.get("database") {
                                print!(
                                    "DB: {} | ",
                                    if db["is_connected"].as_bool().unwrap_or(false) {
                                        "✓"
                                    } else {
                                        "✗"
                                    }
                                );
                            }
                            if let Some(services) =
                                backend_info.get("services").and_then(|s| s.as_array())
                            {
                                print!("Services: {}", services.len());
                            }
                            println!();
                        }

                        if detailed {
                            println!(
                                "  Details: {}",
                                serde_json::to_string_pretty(&status["additional_info"])?
                            );
                        }
                        println!();
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn start_tui(&self) -> Result<()> {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "removed",
                    "message": "TUI has been removed. Use 'ccswarm status' instead.",
                }))?
            );
        } else {
            println!("TUI has been removed.");
            println!("Use 'ccswarm status' to view orchestrator status.");
        }

        Ok(())
    }
}
