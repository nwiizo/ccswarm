# Semantic Features Quick Reference

## Quick Start

```bash
# Initialize semantic features
ccswarm semantic analyze --path .

# Find and apply refactorings
ccswarm semantic refactor --auto-apply --priority high

# Generate specialized agents
ccswarm semantic generate-agents --deploy

# Launch dashboard
ccswarm semantic dashboard --port 3000
```

## Common Tasks

### Analyze Code Quality

```bash
# Basic analysis
ccswarm semantic analyze

# Detailed with symbols
ccswarm semantic analyze --symbols --format markdown

# Output to file
ccswarm semantic analyze --format json > analysis.json
```

### Find Code Issues

```bash
# Find all issues
ccswarm semantic refactor

# High priority only
ccswarm semantic refactor --priority high

# Auto-fix safe issues
ccswarm semantic refactor --auto-apply --priority low
```

### Cross-Repository Analysis

```bash
# Analyze multiple repos
ccswarm semantic optimize \
  --repos "frontend:./frontend:typescript" \
  --repos "backend:./backend:rust" \
  --detailed

# Generate report
ccswarm semantic optimize \
  --repos "main:.:auto" \
  --output optimization-report.md
```

### Democratic Voting

```bash
# Create proposal
ccswarm semantic vote --create "Migrate to microservices"

# Vote on proposal
ccswarm semantic vote --submit "prop-123:approve:better scalability"

# Check results
ccswarm semantic vote --history
```

## Programmatic Usage

### Basic Setup

```rust
use ccswarm::semantic::{SemanticManager, SemanticConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let config = SemanticConfig::default();
    let manager = SemanticManager::new(config).await?;
    manager.initialize().await?;
    
    // Use features
    let symbols = manager.symbol_index()
        .find_symbol("UserService", None).await?;
    
    Ok(())
}
```

### Find and Fix Issues

```rust
use ccswarm::semantic::refactoring_system::AutomaticRefactoringSystem;

// Create refactoring system
let mut refactoring = AutomaticRefactoringSystem::new(
    manager.analyzer(),
    manager.symbol_index(),
    manager.memory(),
);

// Scan for issues
let proposals = refactoring.scan_codebase().await?;

// Apply high-priority fixes
for proposal in proposals {
    if proposal.priority == RefactoringPriority::High && proposal.automated {
        refactoring.apply_proposal(&proposal.id).await?;
    }
}
```

### Generate Agents Dynamically

```rust
use ccswarm::semantic::dynamic_agent_generation::DynamicAgentGenerator;

// Create generator
let generator = DynamicAgentGenerator::new(
    manager.analyzer(),
    manager.symbol_index(),
    manager.memory(),
);

// Analyze needs
let needs = generator.analyze_agent_needs().await?;

// Generate and deploy
for need in needs {
    let agent = generator.generate_agent(&need).await?;
    generator.deploy_agent(&agent).await?;
}
```

### Use Project Memory

```rust
use ccswarm::semantic::memory::{Memory, MemoryType};

// Store knowledge
manager.memory().store_memory(Memory {
    name: "api_pattern".to_string(),
    content: "Use RESTful conventions".to_string(),
    memory_type: MemoryType::CodingConvention,
    // ...
}).await?;

// Recall knowledge
let conventions = manager.memory()
    .recall_by_type(MemoryType::CodingConvention).await?;
```

## Web Dashboard API

### REST Endpoints

```bash
# Get metrics
curl http://localhost:3000/api/metrics

# Get symbols
curl http://localhost:3000/api/symbols

# Get refactoring proposals
curl http://localhost:3000/api/refactoring

# Trigger analysis
curl -X POST http://localhost:3000/api/analyze \
  -H 'Content-Type: application/json' \
  -d '{"path": ".", "deep": true}'
```

### WebSocket Connection

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:3000/ws');

// Handle messages
ws.onmessage = (event) => {
    const metrics = JSON.parse(event.data);
    console.log('Metrics update:', metrics);
};
```

## Configuration Options

### Minimal Config

```toml
[semantic]
enabled = true
```

### Full Config

```toml
[semantic]
enabled = true
cache_size = "2GB"
index_on_startup = true
mcp_enabled = true
mcp_port = 8080
memory_enabled = true
max_memories = 200

[semantic.refactoring]
auto_apply_threshold = "medium"
max_proposals = 100
scan_interval = "1h"

[semantic.agents]
auto_generate = true
complexity_threshold = "moderate"
deploy_on_creation = true

[semantic.voting]
consensus_algorithm = "simple_majority"
voting_period = "24h"
quorum_percentage = 0.51
```

## Performance Tips

### 1. Use Symbol Cache

```rust
// Cache symbols for repeated access
let symbols = manager.symbol_index().get_all_symbols().await?;
let symbol_map: HashMap<String, Symbol> = symbols
    .into_iter()
    .map(|s| (s.name.clone(), s))
    .collect();
```

### 2. Batch Operations

```rust
// Batch symbol lookups
let names = vec!["UserService", "AuthController", "Database"];
let symbols = futures::future::join_all(
    names.iter().map(|name| 
        manager.symbol_index().find_symbol(name, None)
    )
).await;
```

### 3. Selective Indexing

```bash
# Index only specific directories
ccswarm semantic analyze --path src/core
ccswarm semantic analyze --path src/api
```

### 4. Memory Optimization

```rust
// Use memory pruning
manager.memory().prune_old_memories(30).await?; // Remove >30 days old
manager.memory().compact().await?; // Optimize storage
```

## Troubleshooting Commands

```bash
# Clear semantic cache
rm -rf .ccswarm/cache/semantic

# Reset index
ccswarm semantic analyze --path . --force-rebuild

# Check system health
ccswarm semantic monitor --filter health

# Enable debug logging
RUST_LOG=ccswarm::semantic=debug ccswarm semantic analyze

# Verify MCP connection
curl http://localhost:8080/health
```

## Common Patterns

### Pattern 1: Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run semantic analysis
ccswarm semantic refactor --priority critical --max 1

# Fail if critical issues found
if [ $? -ne 0 ]; then
    echo "Critical refactoring needed. Please fix before committing."
    exit 1
fi
```

### Pattern 2: CI/CD Integration

```yaml
# .github/workflows/semantic.yml
name: Semantic Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Semantic Analysis
        run: |
          ccswarm semantic analyze --format json > analysis.json
          ccswarm semantic refactor --priority high --max 10
```

### Pattern 3: Scheduled Optimization

```bash
# Cron job for weekly analysis
0 0 * * 0 ccswarm semantic optimize \
  --repos "all:$HOME/projects:auto" \
  --output "$HOME/reports/weekly-$(date +%Y%m%d).md"
```

## Key Shortcuts

| Command | Alias | Description |
|---------|-------|-------------|
| `ccswarm semantic analyze` | `ccs a` | Quick analysis |
| `ccswarm semantic refactor` | `ccs r` | Find refactorings |
| `ccswarm semantic dashboard` | `ccs d` | Launch dashboard |
| `ccswarm semantic monitor` | `ccs m` | Monitor operations |

## Environment Variables

```bash
# Essential
export CCSWARM_SEMANTIC_ENABLED=true
export CCSWARM_SEMANTIC_CACHE_SIZE="2GB"

# Optional
export CCSWARM_SEMANTIC_MCP_PORT=8080
export CCSWARM_SEMANTIC_MAX_MEMORIES=200
export CCSWARM_SEMANTIC_AUTO_INDEX=true
export CCSWARM_SEMANTIC_DEBUG=true
```

## Help & Support

- Documentation: `ccswarm semantic --help`
- Detailed help: `ccswarm semantic <command> --help`
- GitHub Issues: https://github.com/nwiizo/ccswarm/issues
- Community: https://discord.gg/ccswarm