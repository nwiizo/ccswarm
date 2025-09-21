# ccswarm delegate

Control task delegation and Master Claude's orchestration decisions.

## Description

The `delegate` command manages how Master Claude analyzes and assigns tasks to agents. It provides tools for manual delegation, strategy configuration, and delegation analytics.

## Usage

```bash
ccswarm delegate [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `analyze` - Analyze a task without executing
- `task` - Manually delegate a task to an agent
- `strategy` - Configure delegation strategy
- `stats` - View delegation statistics
- `history` - View delegation history
- `rebalance` - Rebalance task assignments

## Options

### For `analyze`
- `--verbose` - Show detailed analysis
- `--suggest-agent` - Suggest best agent
- `--show-confidence` - Show confidence scores

### For `task`
- `--agent <NAME>` - Target agent (required)
- `--priority <LEVEL>` - Task priority
- `--override` - Override Master Claude's recommendation

### For `strategy`
- `--set <STRATEGY>` - Set delegation strategy
- `--weights <JSON>` - Configure strategy weights
- `--show` - Show current strategy

### For `stats`
- `--period <TIME>` - Time period (24h, 7d, 30d)
- `--by-agent` - Group by agent
- `--by-type` - Group by task type

## Examples

### Analyze task delegation
```bash
ccswarm delegate analyze "Create user authentication system" --verbose
```

Output:
```
Task Analysis: Create user authentication system
==============================================
Complexity: High
Domain: Backend, Security
Technologies: Auth, Database, API

Agent Recommendations:
1. backend-api (95% confidence)
   - Expertise: Authentication, API development
   - Current load: 2 tasks
   - Availability: High

2. backend-specialist (78% confidence)
   - Expertise: General backend
   - Current load: 4 tasks
   - Availability: Medium

Suggested Assignment: backend-api
```

### Manual delegation
```bash
ccswarm delegate task "Fix CSS layout issues" --agent frontend-specialist
```

### Override recommendation
```bash
ccswarm delegate task "Complex API endpoint" \
  --agent backend-specialist \
  --override \
  --priority high
```

### View delegation statistics
```bash
ccswarm delegate stats --period 7d
```

## Delegation Strategies

### Available Strategies

#### ContentBased (Default)
Matches task content with agent expertise:
```bash
ccswarm delegate strategy --set ContentBased
```

#### LoadBalanced
Distributes tasks evenly across agents:
```bash
ccswarm delegate strategy --set LoadBalanced
```

#### ExpertiseBased
Uses historical performance data:
```bash
ccswarm delegate strategy --set ExpertiseBased
```

#### WorkflowBased
Considers task dependencies:
```bash
ccswarm delegate strategy --set WorkflowBased
```

#### Hybrid
Combines all strategies with weights:
```bash
ccswarm delegate strategy --set Hybrid --weights '{
  "content": 0.4,
  "load": 0.2,
  "expertise": 0.3,
  "workflow": 0.1
}'
```

### Configure Strategy
```bash
# Show current strategy
ccswarm delegate strategy --show

# Set with custom weights
ccswarm delegate strategy --set Hybrid --weights '{
  "content": 0.5,
  "load": 0.3,
  "expertise": 0.2
}'
```

## Delegation Analytics

### View statistics
```bash
ccswarm delegate stats --period 30d --by-agent
```

Output:
```
Delegation Statistics (Last 30 days)
===================================

By Agent:
frontend-specialist:
  Tasks: 145
  Success Rate: 92%
  Avg Time: 25m
  Specialties: UI (65%), Styling (35%)

backend-api:
  Tasks: 203
  Success Rate: 89%
  Avg Time: 35m
  Specialties: API (45%), Database (30%), Auth (25%)

Overall:
  Total Delegations: 487
  Correct Assignments: 94%
  Reassignments: 29 (6%)
  Average Decision Time: 2.3s
```

### Historical analysis
```bash
ccswarm delegate history --limit 20
```

### Task type distribution
```bash
ccswarm delegate stats --by-type --period 7d
```

## Rebalancing

### Automatic rebalancing
```bash
ccswarm delegate rebalance
```

Redistributes queued tasks based on:
- Current agent workload
- Task priorities
- Agent expertise
- Historical performance

### Dry run
```bash
ccswarm delegate rebalance --dry-run
```

## Advanced Analysis

### Task complexity scoring
```bash
ccswarm delegate analyze "Implement microservices architecture" \
  --show-complexity
```

Output:
```
Complexity Analysis:
- Lines of Code Estimate: 5000-10000
- Technologies Required: 8
- Integration Points: 12
- Testing Complexity: High
- Overall Score: 8.5/10
```

### Multi-agent tasks
```bash
ccswarm delegate analyze "Full-stack feature implementation" \
  --suggest-multiple
```

Output:
```
Multi-Agent Recommendation:
1. Frontend: frontend-specialist (navbar, forms)
2. Backend: backend-api (API endpoints)
3. Testing: qa-tester (integration tests)

Suggested Breakdown:
- Task 1: Create UI components → frontend-specialist
- Task 2: Implement API → backend-api
- Task 3: Write tests → qa-tester
```

## Delegation Rules

### Custom rules
```json
{
  "delegation_rules": [
    {
      "pattern": "security|auth|crypto",
      "preferred_agent": "security-specialist",
      "priority_boost": 1
    },
    {
      "pattern": "ui|frontend|react",
      "preferred_agent": "frontend-specialist"
    }
  ]
}
```

### Apply rules
```bash
ccswarm config update --set delegation_rules=@rules.json
```

## Integration with Master Claude

### Master Claude Instructions
Configure how Master Claude makes decisions:
```bash
ccswarm config update --set project.master_claude_instructions="
Prioritize security-related tasks.
Prefer specialists over generalists.
Consider agent workload before assigning.
Group related tasks for the same agent.
"
```

### Delegation feedback
```bash
# Mark delegation as successful
ccswarm delegate feedback task_123 --success

# Mark as needing reassignment
ccswarm delegate feedback task_124 --reassign --reason "Wrong expertise"
```

## Troubleshooting

### Poor delegation decisions
```bash
# Review recent delegations
ccswarm delegate history --failed

# Adjust strategy weights
ccswarm delegate strategy --set Hybrid --weights '{
  "expertise": 0.6,
  "content": 0.3,
  "load": 0.1
}'
```

### Agent overload
```bash
# Check load distribution
ccswarm delegate stats --by-agent --show-load

# Rebalance tasks
ccswarm delegate rebalance
```

## Related Commands

- [`task`](task.md) - Create tasks for delegation
- [`agents`](agents.md) - View agent capabilities
- [`status`](status.md) - Check delegation queue
- [`review`](review.md) - Review delegation effectiveness

## Notes

- Master Claude learns from delegation feedback
- Strategies can be changed without restart
- Manual overrides are logged for analysis
- Delegation decisions are cached for 5 minutes
- Historical data improves future decisions