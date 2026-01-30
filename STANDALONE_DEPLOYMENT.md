# Standalone Deployment

Run ccswarm without Claude Code or external AI providers.

## Simulation Mode

Run with simulated agents for testing and learning:

```bash
# Start in simulation mode
CCSWARM_SIMULATION=true ccswarm start

# Or via config
ccswarm init --name "MyProject"  # simulation is default without API key
```

Agents return mock responses, allowing you to test orchestration logic without API costs.

## Built-in Templates

Generate complete applications without AI providers:

```bash
# Generate TODO app using built-in template
ccswarm auto-create "Create TODO app" --output ./my-app

# Run the generated app
cd my-app && npm install && npm start
```

Available templates: `todo`, `blog`, `generic`

## Docker Deployment

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p ccswarm

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ccswarm /usr/local/bin/
ENV CCSWARM_SIMULATION=true
ENTRYPOINT ["ccswarm"]
```

```bash
docker build -t ccswarm:standalone .
docker run -it -v $(pwd):/workspace ccswarm:standalone init --name "DockerProject"
```

## Custom Providers

Create your own agent implementations:

```json
{
  "provider": "custom",
  "config": {
    "command": "/path/to/custom-tool",
    "args": ["--mode", "agent"]
  }
}
```

## Offline Operation

Full functionality without internet:

1. Build ccswarm locally: `cargo build --release`
2. Use simulation mode or custom providers
3. Generate apps with built-in templates

## See Also

- [Configuration](docs/CONFIGURATION.md) - Provider setup
- [Commands Reference](docs/COMMANDS.md) - CLI usage
