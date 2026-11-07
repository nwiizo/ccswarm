//! Tool lists and workspace access for a stage's permission level.
//!
//! The tool list becomes the provider's allowed-tools list. Claude still runs
//! with `--dangerously-skip-permissions`, so this list is not a sandbox. The
//! workspace access selects the Codex sandbox.

use super::flow::MovementPermission;
use crate::providers::WorkspaceAccess;

const READONLY_TOOLS: &[&str] = &["read", "grep", "glob", "search", "list", "cat", "find"];
const EDIT_TOOLS: &[&str] = &[
    "read", "grep", "glob", "search", "list", "cat", "find", "edit", "write", "replace",
];
const FULL_TOOLS: &[&str] = &[
    "read", "grep", "glob", "search", "list", "cat", "find", "edit", "write", "replace", "bash",
    "shell", "exec", "delete", "create", "mkdir", "rm",
];

/// Tools for a stage: its explicit `tools:` list in order (duplicates
/// removed), or the defaults for its permission level.
pub fn stage_tools(permission: &MovementPermission, explicit: &[String]) -> Vec<String> {
    if !explicit.is_empty() {
        let mut tools: Vec<String> = Vec::with_capacity(explicit.len());
        for tool in explicit {
            if !tools.contains(tool) {
                tools.push(tool.clone());
            }
        }
        return tools;
    }

    let level_tools = match permission {
        MovementPermission::Readonly => READONLY_TOOLS,
        MovementPermission::Edit => EDIT_TOOLS,
        MovementPermission::Full => FULL_TOOLS,
    };
    level_tools.iter().map(|tool| tool.to_string()).collect()
}

/// Workspace access for a stage's permission level. Explicit `tools:` do not
/// widen it: a `readonly` stage stays read-only even if it lists `bash`.
pub(crate) fn workspace_access(permission: &MovementPermission) -> WorkspaceAccess {
    match permission {
        MovementPermission::Readonly => WorkspaceAccess::ReadOnly,
        MovementPermission::Edit | MovementPermission::Full => WorkspaceAccess::Write,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_levels_widen_the_default_tools() {
        let readonly = stage_tools(&MovementPermission::Readonly, &[]);
        let edit = stage_tools(&MovementPermission::Edit, &[]);
        let full = stage_tools(&MovementPermission::Full, &[]);

        assert!(readonly.contains(&"read".to_string()));
        assert!(!readonly.contains(&"edit".to_string()));
        assert!(edit.contains(&"edit".to_string()));
        assert!(!edit.contains(&"bash".to_string()));
        assert!(full.contains(&"bash".to_string()));
    }

    #[test]
    fn only_readonly_requests_a_read_only_workspace() {
        assert_eq!(
            workspace_access(&MovementPermission::Readonly),
            WorkspaceAccess::ReadOnly
        );
        assert_eq!(
            workspace_access(&MovementPermission::Edit),
            WorkspaceAccess::Write
        );
        assert_eq!(
            workspace_access(&MovementPermission::Full),
            WorkspaceAccess::Write
        );
    }

    /// Builtin flows keep `permission` and `tools:` consistent: readonly
    /// stages list no file-writing tools, and local summary stages, which
    /// never reach a provider, are readonly.
    #[test]
    fn builtin_readonly_stages_stay_read_only() {
        const WRITE_TOOLS: &[&str] = &["write", "edit", "replace", "delete", "create", "rm"];
        for flow in super::super::flow::builtin_flows() {
            for stage in &flow.stages {
                let local = stage.instruction.is_empty() || stage.instruction.starts_with("_local");
                assert!(
                    !local || stage.permission == MovementPermission::Readonly,
                    "{}/{}: local summary stage must be readonly",
                    flow.name,
                    stage.id
                );
                if stage.permission == MovementPermission::Readonly {
                    let writes: Vec<_> = stage
                        .tools
                        .iter()
                        .filter(|tool| WRITE_TOOLS.contains(&tool.to_ascii_lowercase().as_str()))
                        .collect();
                    assert!(
                        writes.is_empty(),
                        "{}/{}: readonly stage lists write tools {writes:?}",
                        flow.name,
                        stage.id
                    );
                }
            }
        }
    }

    #[test]
    fn explicit_tools_override_the_level_in_order() {
        let explicit = ["grep", "read", "grep"].map(String::from);
        assert_eq!(
            stage_tools(&MovementPermission::Full, &explicit),
            ["grep", "read"]
        );
    }
}
