# ccswarm Project-Specific Rules

This document contains the critical rules and patterns that must be followed when working on the ccswarm codebase.

## Core Development Rules

### 1. Agent Role Boundaries
**CRITICAL**: Agents must NEVER work outside their designated role.

```rust
// CORRECT: Frontend agent working on UI
// Location: agents/frontend/src/components/Dashboard.tsx
const Dashboard: React.FC = () => { /* UI code */ }

// INCORRECT: Frontend agent working on API
// This would trigger an identity violation
app.post('/api/users', ...) // ❌ Backend work in frontend agent
```

### 2. Session Management Requirements
All agent interactions MUST use the ai-session system.

```rust
// CORRECT: Using ai-session adapter
let session = AISessionAdapter::create_session(&agent_name).await?;
session.execute_command("cargo test").await?;

// INCORRECT: Direct process spawning
std::process::Command::new("cargo").arg("test").spawn()?; // ❌
```

### 3. Error Handling Standards
Production code must NEVER use `.unwrap()`.

```rust
// CORRECT: Proper error handling
let file = File::open(path)
    .map_err(|e| MyError::FileAccess(e))?;

// INCORRECT: Unwrap in production
let file = File::open(path).unwrap(); // ❌
```

### 4. Async Best Practices
Always use tokio for async operations.

```rust
// CORRECT: Tokio async
#[tokio::test]
async fn test_async_operation() {
    let result = my_async_fn().await?;
}

// INCORRECT: Blocking in async context
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1)); // ❌ Blocks runtime
}
```

### 5. Testing Requirements
All new features must include tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_feature() {
        // Test coverage required for:
        // - Happy path
        // - Error cases
        // - Edge cases
    }
}
```

## Security Rules

### 1. No Hardcoded Secrets
```rust
// CORRECT: Environment variable
let api_key = env::var("ANTHROPIC_API_KEY")?;

// INCORRECT: Hardcoded secret
let api_key = "sk-ant-12345"; // ❌ NEVER DO THIS
```

### 2. Input Validation
```rust
// CORRECT: Validate user input
fn validate_task(description: &str) -> Result<()> {
    if description.len() > 1000 {
        return Err(MyError::InvalidInput("Task too long"));
    }
    // Additional validation...
}
```

### 3. File Access Controls
```rust
// CORRECT: Check protected patterns
if is_protected_file(path) {
    return Err(MyError::AccessDenied);
}

// Protected patterns include:
// - .env files
// - *.key files
// - .git/ directory
// - System files
```

## Performance Rules

### 1. Session Reuse
```rust
// CORRECT: Reuse existing session
let session = session_pool.get_or_create(agent_name).await?;

// INCORRECT: Always creating new sessions
let session = create_new_session().await?; // ❌ Wasteful
```

### 2. Concurrent Operations
```rust
// CORRECT: Parallel execution
let futures = vec![
    task1.boxed(),
    task2.boxed(),
    task3.boxed(),
];
let results = futures::future::join_all(futures).await;

// INCORRECT: Sequential when could be parallel
let r1 = task1.await?;
let r2 = task2.await?; // ❌ Could run concurrently
let r3 = task3.await?;
```

### 3. Memory Management
```rust
// CORRECT: Clean up resources
impl Drop for SessionManager {
    fn drop(&mut self) {
        // Clean up sessions
        self.sessions.clear();
    }
}
```

## Quality Standards

### 1. Documentation Requirements
All public APIs must be documented:

```rust
/// Executes a task on the specified agent.
/// 
/// # Arguments
/// * `agent_name` - The name of the agent to execute on
/// * `task` - The task to execute
/// 
/// # Returns
/// The result of the task execution
/// 
/// # Errors
/// Returns error if agent not found or task fails
pub async fn execute_task(
    agent_name: &str,
    task: &Task,
) -> Result<TaskResult> {
    // Implementation
}
```

### 2. Complexity Limits
Functions should have cyclomatic complexity < 10:

```rust
// CORRECT: Simple, focused functions
fn calculate_risk(operation: &Operation) -> u8 {
    match operation.kind {
        OpKind::Read => 1,
        OpKind::Write => 5,
        OpKind::Delete => 10,
    }
}

// INCORRECT: Overly complex function
fn do_everything() {
    // 100+ lines of nested logic ❌
}
```

### 3. Test Coverage
Maintain >85% test coverage:

```bash
# Check coverage
cargo tarpaulin --out Html

# Required coverage areas:
# - Business logic: 90%+
# - Error handling: 85%+
# - Edge cases: 80%+
```

## Git Workflow Rules

### 1. Branch Protection
- Never push directly to master/main
- All changes via pull requests
- Require CI passing before merge

### 2. Commit Standards
```bash
# CORRECT: Conventional commit
git commit -m "feat(session): add compression support"
git commit -m "fix(agent): resolve identity violation"

# INCORRECT: Poor commit messages
git commit -m "fixed stuff" # ❌
git commit -m "WIP" # ❌
```

### 3. Code Review Requirements
- All PRs require review
- Address all feedback
- Ensure tests pass
- Update documentation

## Common Violations to Avoid

### 1. Cross-Agent Contamination
```rust
// ❌ NEVER: Frontend agent accessing backend worktree
let backend_file = "../backend-agent/src/api.rs";
```

### 2. Synchronous Blocking
```rust
// ❌ NEVER: Block async runtime
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));
}
```

### 3. Resource Leaks
```rust
// ❌ NEVER: Forget to clean up
let session = create_session().await?;
// Missing: session.cleanup()
```

### 4. Unsafe Operations
```rust
// ❌ NEVER: Use unsafe without justification
unsafe {
    // Undefined behavior risk
}
```

## Debugging Protocols

### 1. Logging Standards
```rust
// CORRECT: Structured logging
log::debug!("Task assigned: agent={}, task_id={}", agent_name, task.id);
log::error!("Task failed: error={:?}", error);

// INCORRECT: println! in production
println!("Debug: {}", value); // ❌
```

### 2. Error Context
```rust
// CORRECT: Rich error context
operation
    .await
    .map_err(|e| e.context("Failed to execute task"))
    .map_err(|e| e.with_metadata("agent", agent_name))?;
```

### 3. Performance Monitoring
```rust
// CORRECT: Measure critical paths
let start = Instant::now();
let result = expensive_operation().await?;
let duration = start.elapsed();
metrics::histogram!("operation.duration", duration);
```

Remember: These rules exist to maintain system integrity, performance, and security. Violations can cause system-wide failures or security breaches.