use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_sangha(&self, action: &SanghaAction) -> Result<()> {
        let proposals_dir = self.repo_path.join("coordination/proposals");
        tokio::fs::create_dir_all(&proposals_dir).await?;

        match action {
            SanghaAction::Propose {
                title,
                description,
                proposal_type,
            } => {
                let proposal =
                    create_sangha_proposal(&proposals_dir, title, description, proposal_type, None)
                        .await?;
                let id = proposal
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("?");

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Proposal created",
                            "data": proposal,
                        }))?
                    );
                } else {
                    println!(
                        "{} Proposal created: {}",
                        "OK".bright_green().bold(),
                        id.bright_cyan()
                    );
                    println!("  Title: {}", title);
                    println!("  Type:  {}", proposal_type);
                    println!(
                        "  Vote:  ccswarm sangha vote {} --approve",
                        id.bright_yellow()
                    );
                }
            }
            SanghaAction::Vote {
                id,
                approve,
                reason,
            } => {
                let filepath = coordination_json_path(&proposals_dir, id, "Proposal")?;
                if !filepath.exists() {
                    anyhow::bail!("Proposal '{}' not found", id);
                }

                let content = tokio::fs::read_to_string(&filepath).await?;
                let mut proposal: serde_json::Value = serde_json::from_str(&content)?;

                let vote = serde_json::json!({
                    "approve": approve,
                    "reason": reason,
                    "voted_at": chrono::Utc::now().to_rfc3339(),
                });

                if let Some(votes) = proposal.get_mut("votes").and_then(|v| v.as_array_mut()) {
                    votes.push(vote);
                }

                let updated = serde_json::to_string_pretty(&proposal)?;
                tokio::fs::write(&filepath, updated).await?;

                let action_str = if *approve { "approved" } else { "rejected" };
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": format!("Vote recorded: {}", action_str),
                            "data": { "id": id, "vote": action_str },
                        }))?
                    );
                } else {
                    println!(
                        "{} Vote recorded on {}: {}",
                        "OK".bright_green().bold(),
                        id.bright_cyan(),
                        action_str.bright_yellow()
                    );
                }
            }
            SanghaAction::List { status } => {
                let mut entries = tokio::fs::read_dir(&proposals_dir).await?;
                let mut proposals: Vec<serde_json::Value> = Vec::new();

                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json")
                        && let Ok(content) = tokio::fs::read_to_string(&path).await
                        && let Ok(p) = serde_json::from_str::<serde_json::Value>(&content)
                    {
                        if let Some(filter) = status {
                            if p.get("status")
                                .and_then(|s| s.as_str())
                                .is_some_and(|s| s == filter)
                            {
                                proposals.push(p);
                            }
                        } else {
                            proposals.push(p);
                        }
                    }
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": proposals,
                        }))?
                    );
                } else if proposals.is_empty() {
                    println!("No proposals found.");
                } else {
                    println!("{}", "Proposals".bright_cyan().bold());
                    println!("{}", "=========".bright_cyan());
                    for p in &proposals {
                        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let st = p.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let votes = p
                            .get("votes")
                            .and_then(|v| v.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        println!(
                            "  {} [{}] {} ({} votes)",
                            id.bright_yellow(),
                            st.bright_white(),
                            title,
                            votes
                        );
                    }
                    println!("\nTotal: {} proposals", proposals.len());
                }
            }
            SanghaAction::Status { id } => {
                let filepath = coordination_json_path(&proposals_dir, id, "Proposal")?;
                if !filepath.exists() {
                    anyhow::bail!("Proposal '{}' not found", id);
                }

                let content = tokio::fs::read_to_string(&filepath).await?;
                let proposal: serde_json::Value = serde_json::from_str(&content)?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": proposal,
                        }))?
                    );
                } else {
                    let title = proposal
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let desc = proposal
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let st = proposal
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let ptype = proposal
                        .get("proposal_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");

                    println!(
                        "{} {}",
                        "Proposal:".bright_cyan().bold(),
                        id.bright_yellow()
                    );
                    println!("  Title:       {}", title);
                    println!("  Description: {}", desc);
                    println!("  Type:        {}", ptype);
                    println!("  Status:      {}", st);

                    if let Some(votes) = proposal.get("votes").and_then(|v| v.as_array()) {
                        let approve_count = votes
                            .iter()
                            .filter(|v| v.get("approve").and_then(|a| a.as_bool()).unwrap_or(false))
                            .count();
                        let reject_count = votes.len() - approve_count;
                        println!(
                            "  Votes:       {} approve, {} reject",
                            approve_count.to_string().bright_green(),
                            reject_count.to_string().bright_red()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_extend(&self, action: &ExtendAction) -> Result<()> {
        let extensions_dir = self.repo_path.join("coordination/extensions");
        let proposals_dir = self.repo_path.join("coordination/proposals");
        tokio::fs::create_dir_all(&extensions_dir).await?;
        tokio::fs::create_dir_all(&proposals_dir).await?;

        match action {
            ExtendAction::Propose {
                title,
                description,
                agent,
                auto_sangha,
            } => {
                let id = format!("ext-{}", &uuid::Uuid::new_v4().to_string()[..8]);
                let mut extension = serde_json::json!({
                    "id": id,
                    "title": title,
                    "description": description,
                    "agent": agent,
                    "status": "proposed",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                });

                let sangha_proposal_id = if *auto_sangha {
                    let proposal = create_sangha_proposal(
                        &proposals_dir,
                        title,
                        description,
                        "extension",
                        Some(&id),
                    )
                    .await?;
                    proposal
                        .get("id")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                } else {
                    None
                };
                if let Some(proposal_id) = &sangha_proposal_id {
                    extension["sangha_proposal_id"] = serde_json::json!(proposal_id);
                    extension["status"] = serde_json::json!("pending_consensus");
                }

                let filepath = extensions_dir.join(format!("{}.json", id));
                let content = serde_json::to_string_pretty(&extension)?;
                tokio::fs::write(&filepath, content).await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension proposed",
                            "data": extension,
                        }))?
                    );
                } else {
                    println!(
                        "{} Extension proposed: {}",
                        "OK".bright_green().bold(),
                        id.bright_cyan()
                    );
                    println!("  Title: {}", title);
                    println!("  Agent: {}", agent);
                    if let Some(proposal_id) = sangha_proposal_id {
                        println!(
                            "  Sangha: ccswarm lab sangha vote {} --approve",
                            proposal_id.bright_yellow()
                        );
                    }
                }
            }
            ExtendAction::AutoPropose {
                agent,
                reason,
                auto_sangha,
            } => {
                let title = format!("Extend {} agent capabilities", agent);
                let description = reason.clone().unwrap_or_else(|| {
                    "Automatically proposed from local workflow context: add or update agent facets, validation commands, and operating guidance where repeated work would benefit from specialized capability.".to_string()
                });
                let id = format!("ext-{}", &uuid::Uuid::new_v4().to_string()[..8]);
                let mut extension = serde_json::json!({
                    "id": id,
                    "title": title,
                    "description": description,
                    "agent": agent,
                    "status": "proposed",
                    "source": "auto",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                });

                let sangha_proposal_id = if *auto_sangha {
                    let proposal = create_sangha_proposal(
                        &proposals_dir,
                        &title,
                        &description,
                        "extension",
                        Some(&id),
                    )
                    .await?;
                    proposal
                        .get("id")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                } else {
                    None
                };
                if let Some(proposal_id) = &sangha_proposal_id {
                    extension["sangha_proposal_id"] = serde_json::json!(proposal_id);
                    extension["status"] = serde_json::json!("pending_consensus");
                }

                let filepath = extensions_dir.join(format!("{}.json", id));
                let content = serde_json::to_string_pretty(&extension)?;
                tokio::fs::write(&filepath, content).await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension auto-proposed",
                            "data": extension,
                        }))?
                    );
                } else {
                    println!(
                        "{} Extension auto-proposed: {}",
                        "OK".bright_green().bold(),
                        id.bright_cyan()
                    );
                    println!("  Title: {}", title);
                    println!("  Agent: {}", agent);
                    if let Some(proposal_id) = sangha_proposal_id {
                        println!(
                            "  Sangha: ccswarm lab sangha vote {} --approve",
                            proposal_id.bright_yellow()
                        );
                    }
                }
            }
            ExtendAction::List { status } => {
                let mut entries = tokio::fs::read_dir(&extensions_dir).await?;
                let mut extensions: Vec<serde_json::Value> = Vec::new();

                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json")
                        && let Ok(content) = tokio::fs::read_to_string(&path).await
                        && let Ok(ext) = serde_json::from_str::<serde_json::Value>(&content)
                    {
                        if let Some(filter) = status {
                            if ext
                                .get("status")
                                .and_then(|s| s.as_str())
                                .is_some_and(|s| s == filter)
                            {
                                extensions.push(ext);
                            }
                        } else {
                            extensions.push(ext);
                        }
                    }
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": extensions,
                        }))?
                    );
                } else if extensions.is_empty() {
                    println!("No extensions found.");
                } else {
                    println!("{}", "Extensions".bright_cyan().bold());
                    println!("{}", "==========".bright_cyan());
                    for ext in &extensions {
                        let id = ext.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = ext.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let st = ext.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let agent = ext.get("agent").and_then(|v| v.as_str()).unwrap_or("?");
                        println!(
                            "  {} [{}] {} (agent: {})",
                            id.bright_yellow(),
                            st.bright_white(),
                            title,
                            agent.bright_cyan()
                        );
                    }
                    println!("\nTotal: {} extensions", extensions.len());
                }
            }
            ExtendAction::Status { id } => {
                let filepath = coordination_json_path(&extensions_dir, id, "Extension")?;
                if !filepath.exists() {
                    anyhow::bail!("Extension '{}' not found", id);
                }

                let content = tokio::fs::read_to_string(&filepath).await?;
                let extension: serde_json::Value = serde_json::from_str(&content)?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": extension,
                        }))?
                    );
                } else {
                    let title = extension
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let desc = extension
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let st = extension
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let agent = extension
                        .get("agent")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");

                    println!(
                        "{} {}",
                        "Extension:".bright_cyan().bold(),
                        id.bright_yellow()
                    );
                    println!("  Title:       {}", title);
                    println!("  Description: {}", desc);
                    println!("  Agent:       {}", agent);
                    println!("  Status:      {}", st);
                }
            }
            ExtendAction::History { limit } => {
                let mut entries = tokio::fs::read_dir(&extensions_dir).await?;
                let mut extensions: Vec<serde_json::Value> = Vec::new();

                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json")
                        && let Ok(content) = tokio::fs::read_to_string(&path).await
                        && let Ok(ext) = serde_json::from_str::<serde_json::Value>(&content)
                    {
                        extensions.push(ext);
                    }
                }

                // Sort by created_at descending
                extensions.sort_by(|a, b| {
                    let a_time = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    let b_time = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    b_time.cmp(a_time)
                });

                extensions.truncate(*limit);

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "data": extensions,
                        }))?
                    );
                } else if extensions.is_empty() {
                    println!("No extension history.");
                } else {
                    println!(
                        "{} (last {})",
                        "Extension History".bright_cyan().bold(),
                        limit
                    );
                    println!("{}", "=================".bright_cyan());
                    for ext in &extensions {
                        let id = ext.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = ext.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let st = ext.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let created = ext
                            .get("created_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        println!(
                            "  {} [{}] {} ({})",
                            id.bright_yellow(),
                            st.bright_white(),
                            title,
                            created.bright_black()
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

async fn create_sangha_proposal(
    proposals_dir: &std::path::Path,
    title: &str,
    description: &str,
    proposal_type: &str,
    related_extension_id: Option<&str>,
) -> Result<serde_json::Value> {
    let id = format!("prop-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let mut proposal = serde_json::json!({
        "id": id,
        "title": title,
        "description": description,
        "proposal_type": proposal_type,
        "status": "open",
        "votes": [],
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    if let Some(extension_id) = related_extension_id {
        proposal["related_extension_id"] = serde_json::json!(extension_id);
    }

    let proposal_id = proposal
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow::anyhow!("generated proposal is missing id"))?;
    let filepath = proposals_dir.join(format!("{}.json", proposal_id));
    let content = serde_json::to_string_pretty(&proposal)?;
    tokio::fs::write(&filepath, content).await?;

    Ok(proposal)
}

fn coordination_json_path(
    base_dir: &std::path::Path,
    id: &str,
    label: &str,
) -> Result<std::path::PathBuf> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        anyhow::bail!("{} ID '{}' is not a safe coordination ID", label, id);
    }

    Ok(base_dir.join(format!("{}.json", id)))
}
