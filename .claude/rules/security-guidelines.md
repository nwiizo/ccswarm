# Security Guidelines

## Security Requirements

- Never hardcode API keys or secrets
- Validate all user inputs
- Respect protected file patterns: `.env`, `*.key`, `.git/`
- Use environment variables for sensitive data

## Agent Role Enforcement

Each agent has strict boundaries:

| Agent | Allowed Scope |
|-------|---------------|
| Frontend | React, Vue, UI/UX, CSS only |
| Backend | APIs, databases, server logic only |
| DevOps | Docker, CI/CD, infrastructure only |
| QA | Testing and quality assurance only |

## Environment Variables

### Required
- `ANTHROPIC_API_KEY`: For Claude-based providers

### Optional
- `OPENAI_API_KEY`: For OpenAI-based providers
- `RUST_LOG`: Control logging verbosity
- `CCSWARM_HOME`: Configuration directory (default: `~/.ccswarm`)
