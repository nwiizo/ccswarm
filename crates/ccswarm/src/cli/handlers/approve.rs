use super::super::*;
use crate::hitl::{ApprovalRecord, ApprovalStatus, ApprovalStore, Gate};

impl CliRunner {
    pub(crate) async fn handle_approve(&self, action: &ApproveAction) -> Result<()> {
        match action {
            ApproveAction::Plan { id, reject, reason } => {
                self.process_approval(Gate::Plan, id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::RiskyEdit { id, reject, reason } => {
                self.process_approval(Gate::RiskyEdit, id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::Deploy { id, reject, reason } => {
                self.process_approval(Gate::Deploy, id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::Merge { id, reject, reason } => {
                self.process_approval(Gate::Merge, id, *reject, reason.as_deref())
                    .await?;
            }
            ApproveAction::Commit { id, reject, reason } => {
                self.process_approval(Gate::Commit, id, *reject, reason.as_deref())
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
        gate: Gate,
        id: &str,
        reject: bool,
        reason: Option<&str>,
    ) -> Result<()> {
        let store = ApprovalStore::new(&self.repo_path);
        let record = store.decide(id, gate, !reject, reason).await?;
        let status = if reject { "rejected" } else { "approved" };

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
                gate.to_string().bright_cyan(),
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
        let store = ApprovalStore::new(&self.repo_path);
        let approvals: Vec<ApprovalRecord> = store
            .list()
            .await?
            .into_iter()
            .filter(|r| {
                status_filter.is_none_or(|filter| {
                    let st = match r.status {
                        ApprovalStatus::Pending => "pending",
                        ApprovalStatus::Approved => "approved",
                        ApprovalStatus::Rejected => "rejected",
                    };
                    st == filter
                })
            })
            .collect();

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
                let status_colored = match record.status {
                    ApprovalStatus::Approved => "approved".bright_green(),
                    ApprovalStatus::Rejected => "rejected".bright_red(),
                    ApprovalStatus::Pending => "pending".bright_yellow(),
                };
                let ts = record
                    .decided_at
                    .as_deref()
                    .or(record.requested_at.as_deref())
                    .unwrap_or("?");
                println!(
                    "  {} [{}] {} ({})",
                    record.id.bright_yellow(),
                    record.gate.to_string().bright_cyan(),
                    status_colored,
                    ts.bright_black()
                );
            }
            println!("\nTotal: {} records", approvals.len());
        }

        Ok(())
    }
}
