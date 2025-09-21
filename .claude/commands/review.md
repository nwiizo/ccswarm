# ccswarm review

Run quality reviews on completed tasks and manage the review system.

## Description

The `review` command manages the automated quality review system where Master Claude analyzes completed tasks for code quality, test coverage, security issues, and documentation completeness. It can trigger manual reviews, view review history, and configure review settings.

## Usage

```bash
ccswarm review [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `status` - Show current review system status (default)
- `trigger` - Manually trigger a quality review
- `history` - View review history
- `task <TASK-ID>` - Review specific task
- `config` - Configure review settings
- `stats` - Show review statistics
- `enable` - Enable automatic reviews
- `disable` - Disable automatic reviews

## Options

### For `status`
- `--detailed` - Show detailed status information
- `--json` - Output in JSON format

### For `trigger`
- `--all` - Review all completed tasks
- `--agent <NAME>` - Review tasks from specific agent
- `--since <TIME>` - Review tasks completed since time
- `--force` - Force review even if recently reviewed

### For `history`
- `--limit <NUM>` - Number of reviews to show (default: 10)
- `--agent <NAME>` - Filter by agent
- `--failed` - Show only failed reviews
- `--task <ID>` - Show reviews for specific task

### For `config`
- `--interval <SECONDS>` - Set review interval (default: 30)
- `--min-coverage <PERCENT>` - Minimum test coverage (default: 85)
- `--max-complexity <NUM>` - Maximum cyclomatic complexity (default: 10)
- `--standards <FILE>` - Custom quality standards file

## Examples

### Check review status
```bash
ccswarm review
```

Output:
```
Quality Review System: Enabled
Review Interval: 30 seconds
Last Review: 2 minutes ago
Pending Reviews: 3 tasks

Recent Issues Found:
- Low test coverage in auth module (task_123)
- High complexity in payment service (task_124)
- Missing documentation in API endpoints (task_125)
```

### Trigger manual review
```bash
ccswarm review trigger
```

### Review specific task
```bash
ccswarm review task task_123
```

### View review history
```bash
ccswarm review history --limit 20
```

### Configure review settings
```bash
ccswarm review config --interval 60 --min-coverage 90
```

## Quality Standards

### Default Standards
- **Test Coverage**: Minimum 85%
- **Cyclomatic Complexity**: Maximum 10
- **Documentation**: All public APIs documented
- **Security**: No hardcoded secrets or vulnerabilities
- **Performance**: No obvious bottlenecks

### Custom Standards File
```json
{
  "test_coverage": {
    "minimum": 0.90,
    "exclude_patterns": ["test/*", "mock/*"]
  },
  "complexity": {
    "max_cyclomatic": 8,
    "max_cognitive": 15
  },
  "documentation": {
    "require_examples": true,
    "require_params": true,
    "require_returns": true
  },
  "security": {
    "scan_dependencies": true,
    "check_auth": true
  }
}
```

### Apply custom standards
```bash
ccswarm review config --standards ./quality-standards.json
```

## Review Process

### Automated Review Flow
1. **Detection** - Completed tasks identified
2. **Analysis** - Code quality metrics calculated
3. **Evaluation** - Standards compliance checked
4. **Issue Identification** - Problems catalogued
5. **Remediation** - Fix tasks created
6. **Assignment** - Tasks sent to original agent

### Manual Review Flow
```bash
# 1. Trigger review
ccswarm review trigger --agent backend-specialist

# 2. View results
ccswarm review history --limit 1

# 3. Check created remediation tasks
ccswarm task list --type remediation
```

## Review Metrics

### View statistics
```bash
ccswarm review stats --period 7d
```

Output:
```
Review Statistics (Last 7 days)
==============================
Total Reviews: 156
Passed: 118 (76%)
Failed: 38 (24%)

Common Issues:
1. Low test coverage: 15 (39%)
2. High complexity: 10 (26%)
3. Missing docs: 8 (21%)
4. Security issues: 5 (13%)

Average Fix Time: 45 minutes
Review Efficiency: 94%
```

## Issue Types and Fixes

### Test Coverage Issues
```
Issue: "Test coverage below 85% (current: 72%)"
Fix: "Add unit tests for uncovered functions: getUserById, updateProfile, deleteAccount"
```

### Complexity Issues
```
Issue: "Cyclomatic complexity 15 exceeds limit of 10"
Fix: "Refactor processPayment() into smaller functions: validatePayment(), calculateFees(), executeTransaction()"
```

### Documentation Issues
```
Issue: "Missing documentation for public API"
Fix: "Add JSDoc comments for: POST /api/users, PUT /api/users/:id, DELETE /api/users/:id"
```

### Security Issues
```
Issue: "Potential SQL injection vulnerability"
Fix: "Use parameterized queries in searchUsers() function, validate input in lines 45-52"
```

## Integration with CI/CD

### GitHub Actions
```yaml
- name: Run ccswarm quality review
  run: |
    ccswarm review trigger --all
    if ! ccswarm review status --json | jq -e '.pending_issues == 0'; then
      echo "Quality issues found"
      exit 1
    fi
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
ccswarm review trigger --since "1 hour ago"
```

## Troubleshooting

### Reviews not running
```bash
# Check if enabled
ccswarm review status

# Enable if disabled
ccswarm review enable

# Check logs
ccswarm logs --filter "quality_review"
```

### False positives
```bash
# Adjust thresholds
ccswarm review config --min-coverage 80

# Exclude patterns
ccswarm review config --standards custom-standards.json
```

## Related Commands

- [`task`](task.md) - View remediation tasks
- [`status`](status.md) - System status including reviews
- [`agents`](agents.md) - Agent performance metrics
- [`tui`](tui.md) - Interactive review monitoring

## Notes

- Reviews run automatically every 30 seconds by default
- Only completed tasks are reviewed
- Remediation tasks are high priority
- Review history is maintained for 30 days
- Custom standards override defaults