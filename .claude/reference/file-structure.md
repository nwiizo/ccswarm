# Project File Structure

```
ccswarm/
├── Cargo.toml                   # Workspace configuration
├── CLAUDE.md                    # Project overview (minimal)
├── README.md                    # Main documentation
├── docs/
│   ├── ARCHITECTURE.md          # System architecture
│   ├── APPLICATION_SPEC.md      # Application specifications
│   ├── CLAUDE_ACP.md            # Claude ACP integration
│   └── commands/
│       └── workspace-commands.md
├── crates/
│   └── ccswarm/                 # Main application crate
│       ├── src/
│       │   ├── acp_claude/      # Claude ACP integration
│       │   │   ├── adapter.rs
│       │   │   ├── config.rs
│       │   │   └── error.rs
│       │   ├── cli/             # CLI with command registry
│       │   │   ├── command_registry.rs
│       │   │   ├── command_handler.rs
│       │   │   └── commands/
│       │   └── utils/
│       │       └── error_template.rs
│       ├── tests/               # Integration tests
│       └── Cargo.toml
├── sample/                      # Sample scripts and demos
│   ├── claude_acp_demo.sh
│   ├── task_management_demo.sh
│   ├── multi_agent_demo.sh
│   ├── setup.sh
│   └── ccswarm.yaml
└── .claude/
    ├── settings.json            # Claude Code settings
    ├── rules/                   # Path-specific rules
    ├── skills/                  # Multi-step workflows
    ├── reference/               # Reference documentation
    ├── commands/                # Slash commands
    └── agents/                  # Agent definitions
```

## Key Directories

| Directory | Purpose |
|-----------|---------|
| `crates/ccswarm/src/` | Main source code |
| `crates/ccswarm/tests/` | Integration tests |
| `docs/` | Architecture and specs |
| `.claude/` | Claude Code configuration |
| `sample/` | Demo scripts |
