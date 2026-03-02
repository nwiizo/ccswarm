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
        use crate::ipc::{DEFAULT_IPC_HOST, start_ipc_server};
        use tokio::sync::RwLock;

        info!(
            "Starting ccswarm orchestrator with isolation mode: {} (port: {}, delegate: {}, acp: {})",
            isolation, port, delegate, enable_acp
        );

        // Validate provider availability before starting
        self.validate_provider().await?;

        // Parse isolation mode
        let isolation_mode = match isolation {
            "container" => crate::agent::IsolationMode::Container,
            "hybrid" => crate::agent::IsolationMode::Hybrid,
            _ => crate::agent::IsolationMode::GitWorktree,
        };

        let mut master =
            ProactiveMaster::new_with_config(self.config.clone(), self.repo_path.clone()).await?;

        // Set isolation mode for all agents
        master.set_isolation_mode(isolation_mode);

        // Configure delegate mode if enabled
        if delegate {
            info!("Delegate mode enabled: lead orchestrates only, no direct code execution");
            master.set_delegate_mode(true);
        }

        // Initialize agents
        master.initialize().await?;

        let master_id = master.id.clone();
        let agent_count = master.agents.len();

        // Wrap master in Arc<RwLock> for shared access
        let master = Arc::new(RwLock::new(master));

        // Start IPC server
        let _ipc_shutdown = start_ipc_server(DEFAULT_IPC_HOST, port, Arc::clone(&master)).await?;

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
            println!("🚀 ccswarm orchestrator started");
            println!("   Master ID: {}", master_id);
            println!("   Agents: {}", agent_count);
            println!("   IPC Port: {}", port);
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
            }
        }

        // Start coordination loop (blocks until shutdown)
        {
            let mut master_guard = master.write().await;
            master_guard.start_coordination().await?;
        }

        // Cleanup PID file on exit
        if pid_file.exists() {
            let _ = tokio::fs::remove_file(&pid_file).await;
        }

        Ok(())
    }

    pub(crate) async fn stop_orchestrator(&self) -> Result<()> {
        use crate::ipc::{DEFAULT_IPC_HOST, DEFAULT_IPC_PORT, ShutdownRequest, ShutdownResponse};

        // Try to read PID file to get port
        let pid_file = self.repo_path.join(".ccswarm.pid");
        let port = if pid_file.exists() {
            match tokio::fs::read_to_string(&pid_file).await {
                Ok(content) => {
                    if let Ok(info) = serde_json::from_str::<serde_json::Value>(&content) {
                        info["port"].as_u64().unwrap_or(DEFAULT_IPC_PORT as u64) as u16
                    } else {
                        DEFAULT_IPC_PORT
                    }
                }
                Err(_) => DEFAULT_IPC_PORT,
            }
        } else {
            DEFAULT_IPC_PORT
        };

        // Send shutdown request via HTTP
        let url = format!("http://{}:{}/shutdown", DEFAULT_IPC_HOST, port);
        let client = reqwest::Client::new();

        let shutdown_request = ShutdownRequest {
            reason: Some("CLI stop command".to_string()),
            force: false,
        };

        match client.post(&url).json(&shutdown_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ShutdownResponse>().await {
                        Ok(shutdown_response) => {
                            if self.json_output {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "status": "success",
                                        "message": shutdown_response.message,
                                    }))?
                                );
                            } else {
                                println!("🛑 {}", shutdown_response.message);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse shutdown response: {}", e);
                            if !self.json_output {
                                println!("🛑 Shutdown signal sent (response parse error)");
                            }
                        }
                    }
                } else {
                    let status = response.status();
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": format!("Shutdown request failed: {}", status),
                            }))?
                        );
                    } else {
                        println!("❌ Shutdown request failed: {}", status);
                    }
                }
            }
            Err(e) => {
                // Connection error - orchestrator might not be running
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "error",
                            "message": format!("Could not connect to orchestrator: {}", e),
                            "hint": "The orchestrator may not be running",
                        }))?
                    );
                } else {
                    println!("❌ Could not connect to orchestrator on port {}", port);
                    println!("   The orchestrator may not be running.");
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn show_status(&self, detailed: bool, agent: Option<&str>) -> Result<()> {
        use crate::ipc::{DEFAULT_IPC_HOST, DEFAULT_IPC_PORT, StatusResponse};

        // Try to get live status from IPC server first
        let pid_file = self.repo_path.join(".ccswarm.pid");
        let port = if pid_file.exists() {
            match tokio::fs::read_to_string(&pid_file).await {
                Ok(content) => {
                    if let Ok(info) = serde_json::from_str::<serde_json::Value>(&content) {
                        info["port"].as_u64().unwrap_or(DEFAULT_IPC_PORT as u64) as u16
                    } else {
                        DEFAULT_IPC_PORT
                    }
                }
                Err(_) => DEFAULT_IPC_PORT,
            }
        } else {
            DEFAULT_IPC_PORT
        };

        // Try IPC server for live status
        let url = format!("http://{}:{}/status", DEFAULT_IPC_HOST, port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?;

        if let Ok(response) = client.get(&url).send().await
            && response.status().is_success()
            && let Ok(status) = response.json::<StatusResponse>().await
        {
            if !self.json_output {
                println!("📊 ccswarm Live Status (IPC)");
                println!("============================");
                println!("Master ID: {}", status.master_id);
                println!("Active Agents: {}", status.active_agents);
                println!("Pending Tasks: {}", status.pending_tasks);
                println!("Running Tasks: {}", status.running_tasks);
                println!("Completed Tasks: {}", status.completed_tasks);
                println!("Phase: {}", status.phase);
                println!("Uptime: {}s", status.uptime_secs);
                println!();
            } else {
                println!("{}", serde_json::to_string_pretty(&status)?);
                return Ok(());
            }
        }

        // Fall back to reading status from coordination files
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
        info!("Starting ccswarm TUI");

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Starting TUI mode",
                }))?
            );
        } else {
            println!("🖥️  Starting ccswarm TUI...");
            println!("   Press 'q' to quit");
        }

        // Start TUI with execution engine if available
        if let Some(ref execution_engine) = self.execution_engine {
            crate::tui::run_tui_with_engine(Arc::new(execution_engine.clone())).await?;
        } else {
            crate::tui::run_tui().await?;
        }

        Ok(())
    }

}
