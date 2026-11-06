//! Entry points for the ccswarm Herdr plugin (`herdr-plugin/`).

use super::super::*;

/// Where a plugin action should open its pane, from
/// `HERDR_PLUGIN_CONTEXT_JSON`.
#[derive(Debug, Default, PartialEq, Eq)]
struct PaneTarget {
    cwd: Option<String>,
    workspace_id: Option<String>,
    focused_pane_id: Option<String>,
}

impl PaneTarget {
    fn from_context(context_json: Option<&str>) -> Self {
        let Some(value) =
            context_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        else {
            return Self::default();
        };
        let field = |name: &str| {
            value
                .get(name)
                .and_then(serde_json::Value::as_str)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
        };
        Self {
            cwd: field("focused_pane_cwd").or_else(|| field("workspace_cwd")),
            workspace_id: field("workspace_id"),
            focused_pane_id: field("focused_pane_id"),
        }
    }

    fn pane_open_args(&self, view: HerdrView) -> Vec<String> {
        let mut args = vec![
            "plugin".to_string(),
            "pane".to_string(),
            "open".to_string(),
            "--plugin".to_string(),
            "ccswarm".to_string(),
            "--entrypoint".to_string(),
            view.entrypoint().to_string(),
        ];
        // Plugin pane commands start in the plugin directory, so the
        // repository travels in the environment (see herdr-plugin.toml).
        let repo = self
            .cwd
            .as_ref()
            .map(|cwd| format!("CCSWARM_HERDR_REPO={cwd}"));
        // Herdr 0.9.5 rules: popups always open over the active pane and
        // reject a target; split panes need `--target-pane` and reject it
        // combined with `--workspace`, which is only a fallback.
        let (target_pane, workspace) = match (view.is_popup(), &self.focused_pane_id) {
            (true, _) => (None, None),
            (false, Some(pane)) => (Some(pane.clone()), None),
            (false, None) => (None, self.workspace_id.clone()),
        };
        let options = [
            ("--env", &repo),
            ("--target-pane", &target_pane),
            ("--workspace", &workspace),
        ];
        for (flag, value) in options {
            if let Some(value) = value {
                args.push(flag.to_string());
                args.push(value.clone());
            }
        }
        args
    }
}

impl CliRunner {
    pub(crate) async fn handle_herdr(&self, action: &HerdrAction) -> Result<()> {
        match action {
            HerdrAction::Action { view } => {
                let context = std::env::var("HERDR_PLUGIN_CONTEXT_JSON").ok();
                let args = PaneTarget::from_context(context.as_deref()).pane_open_args(*view);
                let program = crate::session::herdr::herdr_program();
                let status = tokio::process::Command::new(&program)
                    .args(&args)
                    .status()
                    .await
                    .with_context(|| format!("Failed to execute {}", program.display()))?;
                if !status.success() {
                    anyhow::bail!("herdr plugin pane open exited with {status}");
                }
                Ok(())
            }
            HerdrAction::Show { view } => {
                let result = match view {
                    HerdrView::RunBoard => self.handle_tail(None, false).await,
                    HerdrView::Runs => self.handle_run(&RunAction::List).await,
                    HerdrView::Doctor => self.handle_doctor(false, false).await,
                };
                if let Err(error) = &result {
                    eprintln!("\n{error:#}");
                }
                // Herdr closes a plugin pane when its command exits, including
                // a run board whose run already finished; keep it readable.
                print!("\nPress Enter to close.");
                std::io::stdout().flush()?;
                let mut line = String::new();
                std::io::stdin().read_line(&mut line)?;
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_target_prefers_the_focused_pane_directory() {
        let context = r#"{"focused_pane_cwd":"/repo","workspace_cwd":"/ws","workspace_id":"w1","focused_pane_id":"w1:p2","selected_text":null}"#;
        let target = PaneTarget::from_context(Some(context));
        assert_eq!(
            target.pane_open_args(HerdrView::RunBoard),
            [
                "plugin",
                "pane",
                "open",
                "--plugin",
                "ccswarm",
                "--entrypoint",
                "run-board",
                "--env",
                "CCSWARM_HERDR_REPO=/repo",
                "--target-pane",
                "w1:p2",
            ]
        );

        let fallback = PaneTarget::from_context(Some(
            r#"{"workspace_cwd":"/ws","focused_pane_cwd":"","workspace_id":"w1"}"#,
        ));
        assert_eq!(fallback.cwd.as_deref(), Some("/ws"));
        assert_eq!(
            &fallback.pane_open_args(HerdrView::RunBoard)[7..],
            ["--env", "CCSWARM_HERDR_REPO=/ws", "--workspace", "w1"]
        );
        // Popups open over the active pane and accept no target.
        assert_eq!(
            &target.pane_open_args(HerdrView::Doctor)[7..],
            ["--env", "CCSWARM_HERDR_REPO=/repo"]
        );
        assert_eq!(
            PaneTarget::from_context(Some("not json")),
            PaneTarget::default()
        );
        assert_eq!(PaneTarget::from_context(None), PaneTarget::default());
    }
}
