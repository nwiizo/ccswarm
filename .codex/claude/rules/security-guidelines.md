# Security Guidelines

## Security Requirements

- Never hardcode API keys or secrets
- Validate all user inputs
- Respect protected file patterns: `.env`, `*.key`, `.git/`
- Use environment variables for sensitive data
- **Use `SensitiveString` for API keys** (prevents accidental logging)

## SensitiveString Pattern

API keys and secrets must use `SensitiveString` wrapper:

```rust
use ccswarm::providers::SensitiveString;

// Create
let api_key = SensitiveString::new("sk-secret-key");

// Safe: Debug/Display masks the value
println!("{:?}", api_key);  // Output: SensitiveString(****)
println!("{}", api_key);    // Output: ****

// When you need the actual value (use sparingly)
let actual = api_key.expose();

// Serialization outputs "[REDACTED]", never the actual secret
let json = serde_json::to_string(&api_key)?; // "[REDACTED]"
```

Benefits:
- Prevents accidental logging of secrets
- Memory zeroed on drop (secrecy crate)
- Clone/Serialize/Deserialize compatible

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
