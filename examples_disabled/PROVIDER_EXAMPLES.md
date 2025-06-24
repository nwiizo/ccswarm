# ccswarm Provider Configuration Examples

This directory contains example configurations demonstrating how to use different AI providers with ccswarm multi-agent orchestration system.

## Available Providers

### 🤖 Claude Code (Default)
- **Icon**: 🤖
- **Color**: Blue
- **Best for**: General development, complex reasoning, file operations
- **Features**: Direct file access, Git operations, think modes, JSON output

### 🔧 Aider
- **Icon**: 🔧  
- **Color**: Green
- **Best for**: Collaborative coding, automatic commits, iterative development
- **Features**: Auto-commit, Git integration, multiple model support

### 🧠 OpenAI Codex
- **Icon**: 🧠
- **Color**: Purple  
- **Best for**: Code generation, API-based interactions, scalable processing
- **Features**: Multiple model options, temperature control, organization support

### ⚙️ Custom Commands
- **Icon**: ⚙️
- **Color**: Gray
- **Best for**: Integration with existing tools, specialized workflows
- **Features**: Flexible command execution, environment variables, custom timeouts

## Configuration Files

### `ccswarm-full-stack.json`
**Default Claude Code Setup**
- Uses Claude Code for all agents
- Traditional ccswarm configuration
- Best for teams familiar with Claude Code CLI

### `ccswarm-aider-focused.json`
**Aider-Powered Development**
- Frontend: Claude 3.5 Sonnet via Aider
- Backend: GPT-4 via Aider  
- QA: GPT-3.5 Turbo via Aider
- Master: Claude Code for orchestration
- Features auto-commit and Git integration

### `ccswarm-mixed-providers.json`
**Multi-Provider Team**
- Frontend: Claude Code (native)
- Backend: Aider with GPT-4
- DevOps: Custom Terraform tool
- AI Research: OpenAI Codex
- QA: Aider with Claude 3.5 Sonnet
- Demonstrates provider diversity

### `ccswarm-openai-codex.json`
**OpenAI Codex Team**
- All agents use OpenAI Codex
- Different models for different roles
- API-based development workflow
- Master orchestrator uses Claude Code

### `ccswarm-custom-tools.json`
**Custom Tool Integration**
- Rust Developer: Custom rust-assistant tool
- Data Scientist: Jupyter-AI integration  
- Security Auditor: Security scanner tool
- Documentation: Custom doc generator
- Performance: Performance analysis tool
- Demonstrates extensive customization

## Environment Variables

### For Aider Configurations
```bash
export ANTHROPIC_API_KEY="your_anthropic_key_here"
export OPENAI_API_KEY="your_openai_key_here"
```

### For OpenAI Codex Configurations
```bash
export OPENAI_API_KEY="your_openai_key_here"
export OPENAI_ORG_ID="your_organization_id"  # Optional
```

### For Custom Tool Configurations
```bash
# Set environment variables as needed by your custom tools
export RUST_LOG="debug"
export PYTHONPATH="/opt/ml/code"
# etc.
```

## Provider Selection Guidelines

### Choose Claude Code When:
- ✅ You need direct file system access
- ✅ Complex reasoning and planning required
- ✅ Working with large codebases
- ✅ Need Git operations integration
- ✅ Want built-in safety features

### Choose Aider When:
- ✅ Collaborative iterative development
- ✅ Automatic commit management desired
- ✅ Working with multiple AI models
- ✅ Need diff-based editing
- ✅ Want Git-native workflow

### Choose OpenAI Codex When:
- ✅ API-based architecture preferred
- ✅ Need scalable processing
- ✅ Want fine-grained model control
- ✅ Integration with OpenAI ecosystem
- ✅ Custom temperature/token settings

### Choose Custom Commands When:
- ✅ Existing specialized tools
- ✅ Domain-specific requirements
- ✅ Legacy system integration
- ✅ Unique workflow needs
- ✅ Maximum flexibility required

## Configuration Migration

### From Legacy Claude Code
```json
{
  "claude_config": {
    "model": "claude-3.5-sonnet",
    "dangerous_skip": true
  }
}
```

### To Multi-Provider
```json
{
  "provider": {
    "provider_type": "claude_code",
    "claude_code": {
      "model": "claude-3.5-sonnet", 
      "dangerous_skip": true
    }
  }
}
```

The system automatically migrates legacy configurations when loading.

## Advanced Provider Features

### Provider Capabilities
Each provider exposes different capabilities:

| Provider | JSON Output | File Ops | Git Ops | Code Exec | Streaming |
|----------|-------------|----------|---------|-----------|-----------|
| Claude Code | ✅ | ✅ | ✅ | ✅ | ❌ |
| Aider | ❌ | ✅ | ✅ | ❌ | ❌ |
| Codex | ✅ | ❌ | ❌ | ❌ | ❌ |
| Custom | ⚙️ | ⚙️ | ⚙️ | ⚙️ | ❌ |

### Health Monitoring
All providers support health checks:
- Version detection
- Response time monitoring  
- Error rate tracking
- Availability status

### Provider Statistics
The TUI displays provider distribution:
- Agent count per provider
- Performance metrics
- Usage patterns
- Health status

## Troubleshooting

### Common Issues

**Aider "not found in PATH"**
```bash
pip install aider-chat
# or
pipx install aider-chat
```

**OpenAI API key issues**
```bash
# Check environment variable
echo $OPENAI_API_KEY

# Test API access
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models
```

**Custom command not working**
- Verify command exists in PATH
- Check environment variables
- Validate argument placeholders
- Review timeout settings

### Provider Validation
```bash
# Test configuration validity
cargo run -- config validate examples/ccswarm-mixed-providers.json

# Check provider availability  
cargo run -- providers check

# Validate specific provider
cargo run -- providers test aider
```

## Best Practices

### Security
- Use environment variables for API keys
- Never commit API keys to version control
- Rotate keys regularly
- Use least-privilege access

### Performance  
- Match model complexity to task complexity
- Use appropriate timeout values
- Monitor provider response times
- Balance cost vs. capability

### Reliability
- Have fallback providers configured
- Monitor provider health
- Implement retry logic
- Track error rates

### Cost Management
- Use cheaper models for simple tasks
- Implement usage monitoring
- Set reasonable token limits
- Review provider costs regularly

## Contributing

When adding new provider examples:

1. Create descriptive configuration file
2. Update this README
3. Add example environment variables
4. Include troubleshooting notes
5. Test configuration validity

For custom provider integrations:
1. Implement `ProviderExecutor` trait
2. Add configuration struct
3. Include health check logic
4. Add comprehensive tests
5. Document capabilities and limitations