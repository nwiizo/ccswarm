//! Tool lists for a stage's permission level.
//!
//! The result becomes the provider's allowed-tools list. Claude still runs
//! with `--dangerously-skip-permissions`, so this list is not a sandbox.

use super::flow::MovementPermission;

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
    fn explicit_tools_override_the_level_in_order() {
        let explicit = ["grep", "read", "grep"].map(String::from);
        assert_eq!(
            stage_tools(&MovementPermission::Full, &explicit),
            ["grep", "read"]
        );
    }
}
