# Claude Code Agents

Sub-agents for the ccswarm project. Designed for both individual invocation and Agent Teams parallel execution.

## Agent Teams (Parallel Execution)

The 4 domain agents support Agent Teams for parallel multi-agent work:

```bash
claude --agent-team                          # Interactive team setup
claude --team "frontend-specialist" "backend-specialist" "qa-specialist"
```

Each domain agent has `isolation: worktree` — they work in independent git worktrees and communicate via `@agent-name` direct messaging.

### Domain Agents (Agent Teams compatible)

| Agent | Model | Isolation | Focus |
|-------|-------|-----------|-------|
| `frontend-specialist` | sonnet | worktree | React, Vue, UI/UX, CSS |
| `backend-specialist` | sonnet | worktree | APIs, databases, Rust server logic |
| `devops-specialist` | sonnet | worktree | Docker, CI/CD, infrastructure |
| `qa-specialist` | sonnet | worktree | Testing, quality, coverage |

### Review Agents (Subagent invocation)

| Agent | Model | Skill | Purpose |
|-------|-------|-------|---------|
| `all-reviewer` | sonnet | `/review-all` | Integrated review (design, quality, architecture) |
| `architecture-reviewer` | sonnet | `/review-architecture` | Architecture pattern compliance |
| `rust-fix-agent` | opus | Proactive | Build/clippy error fixing (YAGNI) |
| `code-refactor-agent` | opus | `/review-duplicates` | Duplicate detection and refactoring (DRY) |

## Usage

### As Agent Team
```bash
# Start a team of frontend + backend + QA working in parallel
claude --team "frontend-specialist" "backend-specialist" "qa-specialist"

# Each agent gets its own worktree and context window
# Communicate between agents: @backend-specialist status?
```

### As Individual Subagent
```
subagent_type: "rust-fix-agent"
prompt: "Fix all clippy warnings in crates/ccswarm/"
```

### Via Skill
```
/review-all              # Triggers all-reviewer agent
/review-architecture     # Triggers architecture-reviewer agent
/review-duplicates       # Triggers code-refactor-agent
/check-production-ready  # Triggers rust-fix-agent
```

## Agent Coordination Flow

```
                     ┌─────────────────┐
                     │  Orchestrator    │
                     │  (Main Claude)   │
                     └────────┬────────┘
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                  │
   ┌────────▼────────┐ ┌─────▼──────┐ ┌────────▼────────┐
   │ Domain Agents    │ │ Review     │ │ Fix Agents      │
   │ (Agent Teams)    │ │ Agents     │ │ (Proactive)     │
   │                  │ │            │ │                  │
   │ frontend ◄──────►│ │ all-review │ │ rust-fix-agent   │
   │ backend  ◄──────►│ │ arch-review│ │ code-refactor    │
   │ devops   ◄──────►│ └────────────┘ └─────────────────┘
   │ qa       ◄──────►│
   └──────────────────┘
     (worktree isolated,
      direct messaging)
```
