# ccswarm quality

Agent-managed quality checks with automated remediation.

## Usage
```bash
ccswarm quality <SUBCOMMAND>
```

## Subcommands
- `check` - Run all quality checks
- `lint` - Run linting checks
- `format` - Format code automatically
- `test` - Run test suite with coverage
- `security` - Security vulnerability scan
- `review` - Trigger LLM-based code review

## Description
Performs comprehensive quality checks using specialized agents and automated tools. Failed checks automatically create remediation tasks for agents to fix.

## Examples

### Run All Quality Checks
```bash
$ ccswarm quality check

🔍 Running Quality Checks
════════════════════════════════════════════

📋 Format Check:
  ⏳ Running cargo fmt --check...
  ✅ Code formatting: Passed

📋 Lint Check:
  ⏳ Running cargo clippy...
  ⚠️  Warning: unused variable `config` at src/main.rs:45
  ❌ Linting: Failed (1 warning, 0 errors)

📋 Test Coverage:
  ⏳ Running cargo test...
  ✅ Tests: 156 passed, 0 failed
  📊 Coverage: 87.3% (target: 85%)
  ✅ Test coverage: Passed

📋 Security Scan:
  ⏳ Checking dependencies...
  ❌ Found 1 vulnerability:
     - tokio 1.0.0 → CVE-2023-12345 (High)
  ❌ Security: Failed

Summary: 2 checks failed
🤖 Creating remediation tasks...
  ✅ Task created: Fix clippy warnings (assigned to backend agent)
  ✅ Task created: Update vulnerable dependencies (assigned to devops agent)
```

### Format Code
```bash
$ ccswarm quality format

🎨 Auto-Formatting Code
════════════════════════════════════════════

Language Detection:
  → Rust: 45 files
  → TypeScript: 23 files
  → Python: 12 files

Formatting:
  ⏳ Running cargo fmt...
  ✅ Formatted 3 Rust files
  
  ⏳ Running prettier...
  ✅ Formatted 7 TypeScript files
  
  ⏳ Running black...
  ✅ Formatted 2 Python files

✅ All code formatted successfully!
```

### Run Tests with Coverage
```bash
$ ccswarm quality test --coverage

🧪 Running Test Suite
════════════════════════════════════════════

⏳ Discovering tests...
Found: 234 tests across 45 files

Running tests:
  ✅ Unit tests: 189/189 passed
  ✅ Integration tests: 43/43 passed
  ❌ E2E tests: 1/2 passed

Failed test:
  × test_user_login_flow
    Expected: 200 OK
    Actual: 401 Unauthorized
    at tests/e2e/auth.test.ts:45

📊 Coverage Report:
  Overall: 78.5%
  src/controllers: 92.3% ✅
  src/models: 88.1% ✅
  src/utils: 65.2% ⚠️
  src/middleware: 45.8% ❌

❌ Coverage below threshold (85%)
🤖 Creating task: Improve test coverage for utils and middleware
```

### Security Vulnerability Scan
```bash
$ ccswarm quality security

🔒 Security Vulnerability Scan
════════════════════════════════════════════

Scanning dependencies...
  → npm packages: 1,245
  → cargo crates: 89
  → pip packages: 34

🚨 Vulnerabilities Found:

Critical (1):
  - lodash@4.17.15
    CVE-2021-23337: Prototype pollution
    Fix: Update to 4.17.21

High (2):
  - express@4.16.0
    CVE-2022-24999: ReDoS vulnerability
    Fix: Update to 4.18.2
    
  - jsonwebtoken@8.5.0
    CVE-2022-23529: Weak signature verification
    Fix: Update to 9.0.0

Medium (3):
  [Details...]

📋 Security Score: C (65/100)

🤖 Auto-remediation in progress...
  ⏳ Creating PR with dependency updates...
  ✅ PR #234 created: "fix: update vulnerable dependencies"
```

### LLM-Based Code Review
```bash
$ ccswarm quality review --file src/auth/login.rs

🤖 AI Code Review
════════════════════════════════════════════

Analyzing: src/auth/login.rs
Model: Claude 3.5 Sonnet

📊 Quality Scores:
  Correctness: 0.85 ✅
  Security: 0.65 ⚠️
  Performance: 0.90 ✅
  Maintainability: 0.75 ✅
  Documentation: 0.40 ❌

🔍 Issues Found:

[HIGH] Security - Line 45:
  Password is logged in plain text
  ```rust
  debug!("Login attempt: {}, {}", username, password);
  ```
  Suggestion: Remove password from logs or hash it

[MEDIUM] Documentation - Lines 12-35:
  Function lacks documentation
  Suggestion: Add doc comments explaining parameters and return values

[LOW] Performance - Line 78:
  Database query in loop could be optimized
  Suggestion: Use batch query or caching

💡 Recommendations:
1. Add rate limiting to prevent brute force
2. Implement proper error handling for DB failures
3. Add unit tests for edge cases

Overall Score: 71% (C+)
Status: Needs improvement

🤖 Creating remediation task for security issues...
```

## Features

### Multi-Language Support
- **Rust**: cargo fmt, clippy, test
- **JavaScript/TypeScript**: prettier, eslint, jest
- **Python**: black, flake8, pytest
- **Go**: gofmt, golint, go test

### Automated Remediation
- Failed checks create tasks automatically
- Tasks assigned to appropriate agents
- Agents fix issues and re-run checks
- Continuous improvement loop

### Quality Metrics
- Code coverage thresholds
- Complexity analysis
- Security scoring
- Performance benchmarks
- Documentation coverage

### Integration
- Git hooks for pre-commit checks
- CI/CD pipeline integration
- Pull request automation
- Slack/Discord notifications

## Configuration
Set quality thresholds in `ccswarm.json`:
```json
{
  "quality": {
    "coverage_threshold": 85,
    "max_complexity": 10,
    "security_level": "high",
    "auto_fix": true,
    "create_tasks": true
  }
}
```

## Related Commands
- `ccswarm review` - Manual quality review
- `ccswarm task list --type remediation` - View fix tasks
- `ccswarm agent list` - See which agents handle quality