use super::super::*;

impl CliRunner {
    pub(crate) async fn init_project(
        &self,
        name: &str,
        repo_url: Option<&str>,
        agents: &[String],
    ) -> Result<()> {
        use crate::utils::user_error::CommonErrors;

        if !self.json_output {
            info!("Initializing ccswarm project: {}", name);
        }

        if !self.json_output {
            println!(
                "{}",
                format!("🚀 Initializing ccswarm project: {}", name)
                    .bright_cyan()
                    .bold()
            );
            println!();
        }

        // Check if git is available
        if !crate::git::shell::ShellWorktreeManager::is_git_available() {
            CommonErrors::git_not_initialized().display();
            return Err(anyhow!("Git is required for ccswarm"));
        }

        // Initialize Git repository if needed
        if !self.json_output {
            crate::utils::user_error::show_progress("Setting up git repository...");
        }
        crate::git::shell::ShellWorktreeManager::init_if_needed(&self.repo_path)
            .await
            .inspect_err(|e| {
                eprintln!();
                CommonErrors::git_not_initialized()
                    .with_details(e.to_string())
                    .display();
            })?;
        if !self.json_output {
            println!("✅ Git repository ready");
        }

        // Create configuration
        if !self.json_output {
            crate::utils::user_error::show_progress("Creating project configuration...");
        }
        let mut config = create_default_config(&self.repo_path)?;
        config.project.name = name.to_string();

        if let Some(url) = repo_url {
            config.project.repository.url = url.to_string();
        }

        // Add requested agents
        for agent_type in agents {
            let agent_config = crate::config::AgentConfig {
                specialization: agent_type.clone(),
                worktree: format!("../worktrees/{}-agent", agent_type),
                branch: format!("feature/{}", agent_type),
                claude_config: crate::config::ClaudeConfig::for_agent(agent_type),
                claude_md_template: format!("{}_specialist", agent_type),
            };
            config.agents.insert(agent_type.clone(), agent_config);
        }

        // Save configuration
        let config_file = self.repo_path.join("ccswarm.json");
        config.to_file(config_file).await?;

        let mut configured_agents = config.agents.keys().cloned().collect::<Vec<_>>();
        configured_agents.sort();

        let data = serde_json::json!({
            "project": name,
            "agents": configured_agents,
        });

        println!(
            "{}",
            self.formatter.format_success(
                &format!("ccswarm project '{}' initialized", name),
                Some(data)
            )
        );

        Ok(())
    }

    pub(crate) async fn handle_quickstart(
        &self,
        name: Option<&str>,
        no_prompt: bool,
        all_agents: bool,
        with_tests: bool,
    ) -> Result<()> {
        // Delegate to simplified implementation
        quickstart_simple::handle_quickstart_simple(
            &self.repo_path,
            name,
            no_prompt,
            all_agents,
            with_tests,
        )
        .await
    }
}
