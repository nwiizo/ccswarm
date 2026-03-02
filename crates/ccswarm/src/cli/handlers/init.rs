use super::super::*;

impl CliRunner {
    pub(crate) async fn init_project(
        &self,
        name: &str,
        repo_url: Option<&str>,
        agents: &[String],
    ) -> Result<()> {
        use crate::utils::user_error::CommonErrors;

        info!("Initializing ccswarm project: {}", name);

        // Show progress to user
        println!(
            "{}",
            format!("🚀 Initializing ccswarm project: {}", name)
                .bright_cyan()
                .bold()
        );
        println!();

        // Check if git is available
        if !crate::git::shell::ShellWorktreeManager::is_git_available() {
            CommonErrors::git_not_initialized().display();
            return Err(anyhow!("Git is required for ccswarm"));
        }

        // Initialize Git repository if needed
        crate::utils::user_error::show_progress("Setting up git repository...");
        crate::git::shell::ShellWorktreeManager::init_if_needed(&self.repo_path)
            .await
            .inspect_err(|e| {
                eprintln!();
                CommonErrors::git_not_initialized()
                    .with_details(e.to_string())
                    .display();
            })?;
        println!("✅ Git repository ready");

        // Create configuration
        crate::utils::user_error::show_progress("Creating project configuration...");
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

        let data = serde_json::json!({
            "project": name,
            "agents": agents,
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

    pub(crate) async fn handle_setup(&self) -> Result<()> {
        // Check if config already exists
        let config_path = self.repo_path.join("ccswarm.json");
        if config_path.exists() {
            println!("{}", "⚠️  Configuration already exists!".bright_yellow());
            println!();
            print!("Overwrite existing configuration? [y/N] ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Setup cancelled.");
                return Ok(());
            }
        }

        // Run setup wizard
        let _config = SetupWizard::run().await?;

        // Initialize project
        crate::utils::user_error::show_progress("Initializing project structure...");
        crate::git::shell::ShellWorktreeManager::init_if_needed(&self.repo_path).await?;

        Ok(())
    }

}
