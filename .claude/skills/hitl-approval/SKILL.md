---
name: hitl-approval
description: Human-in-the-loop approval workflow for high-risk agent operations (file deletions, deployments, config changes).
user-invocable: true
argument-hint: "[gate-type] [--id run-id]"
---

Manage human-in-the-loop approval gates for ccswarm agent operations.

## Gates

| Gate | When Required | Risk Level |
|------|---------------|------------|
| plan | Before executing multi-step plans | Medium |
| risky-edit | File deletions, production configs | High |
| deploy | Deployment actions | Critical |
| merge | PR merge operations | High |

## Commands

```bash
# Request approval
ccswarm approve $ARGUMENTS

# List pending approvals
ccswarm approve list

# Approve with comment
ccswarm approve $ARGUMENTS --id <run-id>

# Reject with reason
ccswarm approve $ARGUMENTS --id <run-id> --reject --reason "explanation"
```

## Risk Levels

| Level | Action | HITL Required |
|-------|--------|---------------|
| Low | Read operations | No |
| Medium | Local file writes | Optional |
| High | External modifications | Yes |
| Critical | Production changes | Always |

All HITL decisions are logged to `.ccswarm/runs/` audit trail.
