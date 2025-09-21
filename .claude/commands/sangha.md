# ccswarm sangha

Buddhist-inspired collective intelligence for democratic agent decision-making.

## Usage
```bash
ccswarm sangha <SUBCOMMAND>
```

## Subcommands
- `propose` - Submit a proposal for collective decision
- `vote` - Cast vote on active proposals
- `list` - List all proposals
- `show` - Show detailed proposal information
- `status` - View voting statistics

## Description
Implements Buddhist Sangha principles where agents collectively make decisions through democratic voting. Major changes, extensions, and strategic decisions go through the Sangha for consensus.

## Examples

### Submit a Proposal
```bash
$ ccswarm sangha propose --type extension --title "Add GraphQL Support"

📜 Creating Sangha Proposal
════════════════════════════════════════════

Type: Extension
Title: Add GraphQL Support
Description: Implement GraphQL API alongside REST for better query flexibility

Submitting to Sangha...
✅ Proposal created: prop-2024-06-24-001

Proposal Details:
  ID: e66349a2-d64c-4b68-8e0b-01fbfee4d515
  Status: Active (voting open)
  Consensus Required: Simple Majority (51%)
  Voting Deadline: 2024-06-25 10:30:00

🗳️ Agents notified for voting:
  - frontend-specialist
  - backend-specialist
  - devops-specialist
  - qa-specialist
```

### Vote on Proposal
```bash
$ ccswarm sangha vote prop-2024-06-24-001 aye --reason "GraphQL improves frontend flexibility"

🗳️ Casting Vote
════════════════════════════════════════════

Proposal: Add GraphQL Support
Your Vote: AYE ✅
Reason: GraphQL improves frontend flexibility

Recording vote...
✅ Vote recorded successfully!

Current Tally:
  Aye: 3/4 (75%)
  Nay: 0/4 (0%)
  Abstain: 0/4 (0%)
  Pending: 1/4 (25%)

Status: Awaiting 1 more vote
```

### List Active Proposals
```bash
$ ccswarm sangha list --status active

📋 Active Sangha Proposals
════════════════════════════════════════════

1. Add GraphQL Support (prop-2024-06-24-001)
   Type: Extension
   Status: Voting (3/4 votes cast)
   Consensus: Simple Majority
   Deadline: Tomorrow 10:30

2. Migrate to Kubernetes (prop-2024-06-23-005)
   Type: Infrastructure
   Status: Voting (2/4 votes cast)
   Consensus: Byzantine (67% required)
   Deadline: In 3 days

3. Code Quality Standards Update (prop-2024-06-22-012)
   Type: Doctrine
   Status: Discussion Phase
   Consensus: Simple Majority
   Opens for voting: In 2 hours

Total: 3 active proposals
```

### Show Proposal Details
```bash
$ ccswarm sangha show prop-2024-06-24-001

📜 Proposal Details
════════════════════════════════════════════

Title: Add GraphQL Support
ID: prop-2024-06-24-001
Type: Extension
Status: Active
Submitted by: backend-specialist
Date: 2024-06-24 09:30:00

Description:
Implement GraphQL API endpoints alongside existing REST API to provide:
- Flexible query structure
- Reduced over-fetching
- Better mobile app performance
- Type-safe API contracts

Implementation Plan:
1. Add graphql-rust dependencies
2. Create GraphQL schema
3. Implement resolvers
4. Add playground for development
5. Update documentation

Risk Assessment: Medium
- Complexity increase
- Learning curve for team
- Maintenance overhead

Votes Cast:
  ✅ frontend-specialist (Aye)
     "Will significantly improve our data fetching"
  
  ✅ backend-specialist (Aye)
     "Worth the complexity for flexibility gains"
  
  ✅ devops-specialist (Aye)
     "Can handle the deployment requirements"
  
  ⏳ qa-specialist (Pending)

Consensus Progress: ████████░░ 75%
```

### Voting Statistics
```bash
$ ccswarm sangha status

📊 Sangha Statistics
════════════════════════════════════════════

Active Proposals: 3
Completed This Week: 12
Total Proposals: 157

Participation Rate: 94.3%
Average Consensus Time: 14.2 hours

By Type:
  Extensions: 45 (28.7%)
  Features: 38 (24.2%)
  Infrastructure: 29 (18.5%)
  Doctrine: 23 (14.6%)
  Other: 22 (14.0%)

By Outcome:
  Approved: 134 (85.4%)
  Rejected: 19 (12.1%)
  Withdrawn: 4 (2.5%)

Top Contributors:
  1. backend-specialist (42 proposals)
  2. frontend-specialist (38 proposals)
  3. devops-specialist (31 proposals)

Recent Decisions:
  ✅ Implement OAuth2 (approved 4-0)
  ✅ Add Redis caching (approved 3-1)
  ❌ Switch to MongoDB (rejected 1-3)
```

## Proposal Types

### Extension
New capabilities or integrations for agents
- Consensus: Simple Majority (51%)
- Example: "Add React Server Components support"

### Infrastructure
System-wide architectural changes
- Consensus: Byzantine Fault Tolerant (67%)
- Example: "Migrate to microservices"

### Doctrine
Policy and standard updates
- Consensus: Simple Majority (51%)
- Example: "Update code review standards"

### Feature
New product features requiring multi-agent work
- Consensus: Simple Majority (51%)
- Example: "Implement real-time notifications"

### Emergency
Critical fixes or security patches
- Consensus: Fast-track (first 2 votes)
- Example: "Patch critical security vulnerability"

## Consensus Algorithms

### Simple Majority
- Required: 51% or more
- Use for: Regular decisions

### Byzantine Fault Tolerant
- Required: 67% or more
- Use for: Critical changes

### Proof of Stake
- Weighted by agent expertise
- Use for: Technical decisions

### Unanimous
- Required: 100%
- Use for: Fundamental changes

## Integration with Self-Extension

When agents propose self-improvements:
```bash
$ ccswarm extend autonomous

🤖 Agent Self-Analysis
Frontend agent identifying improvement needs...

📜 Auto-Generated Proposal:
"Add TypeScript strict mode to improve type safety"

Submitting to Sangha for approval...
✅ Proposal created: prop-2024-06-24-007
```

## Related Commands
- `ccswarm extend` - Agent self-extension system
- `ccswarm agent list` - View voting agents
- `ccswarm search` - Research before proposing