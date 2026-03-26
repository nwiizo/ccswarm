use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_approve(&self, action: &ApproveAction) -> Result<()> {
        let approvals_dir = PathBuf::from(".ccswarm/approvals");
        tokio::fs::create_dir_all(&approvals_dir).await?;

        match action {
            ApproveAction::Plan { id, reject, reason } => {
                self.process_approval("plan", id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::RiskyEdit { id, reject, reason } => {
                self.process_approval("risky-edit", id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::Deploy { id, reject, reason } => {
                self.process_approval("deploy", id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::Merge { id, reject, reason } => {
                self.process_approval("merge", id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::List { status } => {
                self.list_approvals(status.as_deref()).await?;
            }
        }

        Ok(())
    }

    async fn process_approval(
        &self,
        gate: &str,
        id: &str,
        reject: bool,
        reason: Option<&str>,
    ) -> Result<()> {
        let approvals_dir = PathBuf::from(".ccswarm/approvals");
        tokio::fs::create_dir_all(&approvals_dir).await?;

        let status = if reject { "rejected" } else { "approved" };
        let record = serde_json::json!({
            "id": id,
            "gate": gate,
            "status": status,
            "reason": reason,
            "decided_at": chrono::Utc::now().to_rfc3339(),
            "decided_by": "cli",
        });

        let filepath = approvals_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&record)?;
        tokio::fs::write(&filepath, content).await?;

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": format!("Decision recorded: {}", status),
                    "data": record,
                }))?
            );
        } else {
            let status_colored = if reject {
                status.bright_red().bold()
            } else {
                status.bright_green().bold()
            };
            println!(
                "{} [{}] {} {}",
                "OK".bright_green().bold(),
                gate.bright_cyan(),
                id.bright_yellow(),
                status_colored
            );
            if let Some(r) = reason {
                println!("  Reason: {}", r);
            }
        }

        Ok(())
    }

    async fn list_approvals(&self, status_filter: Option<&str>) -> Result<()> {
        let approvals_dir = PathBuf::from(".ccswarm/approvals");
        tokio::fs::create_dir_all(&approvals_dir).await?;

        let mut entries = tokio::fs::read_dir(&approvals_dir).await?;
        let mut approvals: Vec<serde_json::Value> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(content) = tokio::fs::read_to_string(&path).await
                && let Ok(record) = serde_json::from_str::<serde_json::Value>(&content)
            {
                if let Some(filter) = status_filter {
                    if record
                        .get("status")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| s == filter)
                    {
                        approvals.push(record);
                    }
                } else {
                    approvals.push(record);
                }
            }
        }

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "data": approvals,
                }))?
            );
        } else if approvals.is_empty() {
            println!("No approval records found.");
        } else {
            println!("{}", "Approvals".bright_cyan().bold());
            println!("{}", "=========".bright_cyan());
            for record in &approvals {
                let id = record.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let gate = record.get("gate").and_then(|v| v.as_str()).unwrap_or("?");
                let st = record.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                let decided_at = record
                    .get("decided_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let status_colored = match st {
                    "approved" => st.bright_green(),
                    "rejected" => st.bright_red(),
                    _ => st.bright_white(),
                };
                println!(
                    "  {} [{}] {} ({})",
                    id.bright_yellow(),
                    gate.bright_cyan(),
                    status_colored,
                    decided_at.bright_black()
                );
            }
            println!("\nTotal: {} records", approvals.len());
        }

        Ok(())
    }
}
