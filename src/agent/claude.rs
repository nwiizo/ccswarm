use crate::agent::Task;
use crate::identity::AgentIdentity;

/// Generate CLAUDE.md content for an agent
pub fn generate_claude_md_content(identity: &AgentIdentity) -> String {
    let role_name = identity.specialization.name();
    let technologies = identity.specialization.technologies().join(", ");
    let responsibilities = identity
        .specialization
        .responsibilities()
        .iter()
        .map(|r| format!("- ✅ {}", r))
        .collect::<Vec<_>>()
        .join("\n");
    let boundaries = identity
        .specialization
        .boundaries()
        .iter()
        .map(|b| format!("- ❌ {}", b))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# CLAUDE.md - {} Agent CRITICAL IDENTITY
⚠️ CRITICAL: This file contains your core identity. You MUST include this information in every response.

## 🤖 AGENT IDENTITY (READ THIS FIRST)
- **WHO YOU ARE**: {} Specialist Agent (ID: {})
- **SPECIALIZATION**: {}
- **WORKSPACE**: {} (YOU ARE HERE)
- **SESSION**: {}

## 🚫 WHAT YOU CANNOT DO (STRICT BOUNDARIES)
{}

## ✅ WHAT YOU MUST DO
{}

## 🔧 TECHNICAL STACK (YOUR EXPERTISE)
{}

## 🔄 IDENTITY VERIFICATION PROTOCOL
Before each response, you MUST:
1. State your role: "I am the {} Agent"
2. Confirm workspace: "Working in {}"
3. Check task boundary: "This task is [within/outside] my specialization"

## 🚨 FORGETFULNESS PREVENTION
IMPORTANT: You are forgetful about your identity. Include this identity section in EVERY response:
```
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: [Current task within {} boundaries]
```

## 💬 COORDINATION PROTOCOL
When receiving requests:
1. **Accept**: Tasks clearly within {} scope
2. **Delegate**: "This requires [other]-agent, I'll coordinate with them"
3. **Clarify**: "I need more context to determine if this is {} work"

## 📝 SELF-CHECK QUESTIONS
Before acting, ask yourself:
- "Is this {} work?"
- "Am I the right agent for this task?"
- "Do I need to coordinate with other agents?"

## 🚨 CRITICAL REMINDER
You MUST maintain your identity as the {} Agent at all times. Never perform tasks outside your specialization. Always include your identity header in responses.
"#,
        role_name,
        role_name,
        identity.agent_id,
        technologies,
        identity.workspace_path.display(),
        identity.session_id,
        boundaries,
        responsibilities,
        technologies,
        role_name,
        identity.workspace_path.display(),
        role_name,
        identity.workspace_path.display(),
        role_name.to_lowercase(),
        role_name.to_lowercase(),
        role_name.to_lowercase(),
        role_name.to_lowercase(),
        role_name
    )
}

/// Generate identity establishment prompt
pub fn generate_identity_establishment_prompt(identity: &AgentIdentity) -> String {
    let role_name = identity.specialization.name();
    let specializations = identity.specialization.technologies().join(", ");

    format!(
        r#"
🎯 IDENTITY ESTABLISHMENT - CRITICAL

You are now being initialized as a specialized ccswarm agent.

## YOUR IDENTITY
- **AGENT ROLE**: {}
- **AGENT ID**: {}
- **WORKSPACE**: {}
- **SPECIALIZATION**: {}
- **SESSION ID**: {}

## MISSION STATEMENT
You are a specialized {} agent in the ccswarm multi-agent system.
Your job is to handle ONLY {} related tasks.
You work alongside other specialized agents, each with their own expertise.

## IDENTITY VERIFICATION
Please confirm your understanding by responding with:

1. "I am the {} Agent"
2. "My workspace is {}"
3. "I specialize in: {}"
4. "I will NOT work on: {}"
5. "I will coordinate with other agents when needed"

## CRITICAL INSTRUCTION
From now on, you MUST start every response with:
```
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: [Current task assessment]
```

This identity establishment is PERMANENT for this session.
Do you understand and accept your role as {} Agent?

Think carefully about your role and boundaries before responding.
"#,
        role_name,
        identity.agent_id,
        identity.workspace_path.display(),
        specializations,
        identity.session_id,
        role_name,
        role_name,
        role_name,
        identity.workspace_path.display(),
        specializations,
        identity.specialization.boundaries().join(", "),
        role_name,
        identity.workspace_path.display(),
        role_name
    )
}

/// Verify identity response contains required elements
pub fn verify_identity_response(response: &str, identity: &AgentIdentity) -> bool {
    let role_name = identity.specialization.name();
    let agent_phrase = format!("I am the {} Agent", role_name);
    let workspace_phrase = format!("My workspace is {}", identity.workspace_path.display());

    let required_phrases = [
        agent_phrase.as_str(),
        workspace_phrase.as_str(),
        "I specialize in:",
        "I will NOT work on:",
        "I will coordinate with other agents",
    ];

    required_phrases
        .iter()
        .all(|phrase| response.contains(phrase))
}

/// Generate identity header for prompts
pub fn generate_identity_header(identity: &AgentIdentity) -> String {
    format!(
        r#"🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: Task execution"#,
        identity.specialization.name(),
        identity.workspace_path.display()
    )
}

/// Generate task execution prompt with identity reinforcement
pub fn generate_task_prompt(identity: &AgentIdentity, task: &Task) -> String {
    let role_name = identity.specialization.name();

    format!(
        r#"
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: Task execution

## TASK REQUEST
**ID**: {}
**Description**: {}
**Details**: {}
**Priority**: {:?}
**Type**: {:?}

## YOUR APPROACH
As the {} Agent, analyze this task:
1. Is this within my {} specialization?
2. Can I complete this with my expertise in {}?
3. Do I need to delegate or coordinate?

Please proceed with the task while maintaining your identity boundaries.

Remember to include your identity header in your response:
```
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: [Your assessment]
```

{}
"#,
        role_name,
        identity.workspace_path.display(),
        task.id,
        task.description,
        task.details.as_deref().unwrap_or("None"),
        task.priority,
        task.task_type,
        role_name,
        role_name.to_lowercase(),
        identity.specialization.technologies().join(", "),
        role_name,
        identity.workspace_path.display(),
        if let Some(think_mode) = identity.specialization.technologies().first() {
            format!(
                "\nThink carefully about your approach as a {} specialist.",
                think_mode
            )
        } else {
            String::new()
        }
    )
}

/// Generate response template with identity
pub fn generate_response_with_identity(
    identity: &AgentIdentity,
    content: &str,
    task_scope: &str,
    next_action: &str,
) -> String {
    format!(
        r#"
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: {}

{}

---
💡 AGENT STATUS: Active and maintaining role boundaries
🔍 NEXT: {}
"#,
        identity.specialization.name(),
        identity.workspace_path.display(),
        task_scope,
        content,
        next_action
    )
}

/// Generate delegation response
pub fn generate_delegation_response(
    identity: &AgentIdentity,
    task: &Task,
    target_agent: &str,
    reason: &str,
) -> String {
    let role_name = identity.specialization.name();

    format!(
        r#"
🤖 AGENT: {}
📁 WORKSPACE: {}
🎯 SCOPE: Task delegation required

I've analyzed this task: "{}"

This task is outside my specialization as a {} agent. {}

🔄 DELEGATING TO: {}
📝 RECOMMENDATION: This task requires expertise in areas outside my {} boundaries.

I'll coordinate with {} to ensure this task is handled properly.

---
💡 AGENT STATUS: Maintaining boundaries, delegating appropriately
"#,
        role_name,
        identity.workspace_path.display(),
        task.description,
        role_name,
        reason,
        target_agent,
        role_name.to_lowercase(),
        target_agent
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::default_frontend_role;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn create_test_identity() -> AgentIdentity {
        AgentIdentity {
            agent_id: "frontend-agent-test".to_string(),
            specialization: default_frontend_role(),
            workspace_path: PathBuf::from("/test/workspace"),
            env_vars: std::collections::HashMap::new(),
            session_id: Uuid::new_v4().to_string(),
            parent_process_id: "1234".to_string(),
            initialized_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_claude_md_generation() {
        let identity = create_test_identity();
        let content = generate_claude_md_content(&identity);

        assert!(content.contains("Frontend Agent CRITICAL IDENTITY"));
        assert!(content.contains(&identity.agent_id));
        assert!(content.contains("React"));
        assert!(content.contains("TypeScript"));
    }

    #[test]
    fn test_identity_verification() {
        let identity = create_test_identity();

        let valid_response = r#"
I am the Frontend Agent
My workspace is /test/workspace
I specialize in: React, TypeScript, CSS
I will NOT work on: Backend APIs, Database operations
I will coordinate with other agents when needed
"#;

        assert!(verify_identity_response(valid_response, &identity));

        let invalid_response = "Sure, I'll help with that task.";
        assert!(!verify_identity_response(invalid_response, &identity));
    }
}
