# ccswarm task

Create and manage development tasks for AI agents.

## Usage
```bash
ccswarm task <DESCRIPTION> [OPTIONS]
```

## Options
- `--priority <LEVEL>` - Set priority (low, medium, high, critical)
- `--type <TYPE>` - Task type (feature, bug, test, docs, refactor)
- `--agent <NAME>` - Assign to specific agent
- `--auto` - Enable auto-accept for this task
- `--depends-on <TASK_ID>` - Set task dependencies

## Description
Creates tasks that are automatically analyzed and delegated to the most appropriate AI agent based on content and expertise.

## Examples

### Basic Task Creation
```bash
$ ccswarm task "Create a login form with email and password fields"

✅ Task created successfully!

   Task ID: task-a1b2c3
   Description: Create a login form with email and password fields
   Priority: 🟢 Medium
   Type: Feature
   
💡 Quick tips:
  • View task progress: ccswarm task status task-a1b2c3
  • List all tasks: ccswarm task list

🤖 Analyzing task...
   → Frontend expertise detected (React, forms, UI)
   → Delegating to frontend-specialist
   
⏳ Frontend agent starting work...
```

### Task with Modifiers
```bash
$ ccswarm task "Fix memory leak in user service [high] [bug]"

✅ Task created successfully!

   Task ID: task-d4e5f6
   Description: Fix memory leak in user service
   Priority: 🟡 High
   Type: Bug
   
🤖 Master Claude analysis:
   → Backend issue detected
   → Memory profiling required
   → Assigning to backend-specialist with debugger tools
```

### Priority Levels
```bash
# Critical - Immediate attention
$ ccswarm task "Production server down" --priority critical

# High - Next in queue
$ ccswarm task "Security vulnerability patch" --priority high

# Medium - Normal workflow (default)
$ ccswarm task "Add user profile page" --priority medium

# Low - When time permits
$ ccswarm task "Refactor old utils" --priority low
```

### Task Types
```bash
# Feature - New functionality
$ ccswarm task "Add dark mode toggle" --type feature

# Bug - Fix issues
$ ccswarm task "Users can't reset password" --type bug

# Test - Testing tasks
$ ccswarm task "Write E2E tests for checkout" --type test

# Docs - Documentation
$ ccswarm task "Document API endpoints" --type docs

# Refactor - Code improvement
$ ccswarm task "Extract payment logic to service" --type refactor
```

### Direct Agent Assignment
```bash
$ ccswarm task "Setup CI/CD pipeline" --agent devops

🎯 Direct Assignment
════════════════════════════════════════════

Task: Setup CI/CD pipeline
Assigned to: devops-specialist
Reason: Manual assignment

✅ Task queued for DevOps agent
```

### Task Dependencies
```bash
$ ccswarm task "Deploy to production" --depends-on task-a1b2c3,task-d4e5f6

📋 Task Dependencies Set
════════════════════════════════════════════

Task: Deploy to production
Depends on:
  - task-a1b2c3 (Create login form) - In Progress
  - task-d4e5f6 (Fix memory leak) - Completed ✅

Status: Blocked (waiting for dependencies)
Will start automatically when ready
```

### Batch Task Creation
```bash
$ ccswarm task import tasks.txt

📥 Importing Tasks
════════════════════════════════════════════

Reading tasks.txt...
Found 5 tasks:

1. ✅ Create user dashboard
2. ✅ Add email notifications
3. ✅ Implement search functionality
4. ✅ Fix responsive design issues
5. ✅ Add unit tests for auth module

All tasks created and delegated!
```

## Task Management Commands

### List Tasks
```bash
$ ccswarm task list

📋 Active Tasks
════════════════════════════════════════════

High Priority (2):
  • task-g7h8i9 - Fix payment processing bug
    Status: In Progress (backend) ⏳
    
  • task-j0k1l2 - Security audit findings
    Status: Pending

Medium Priority (3):
  • task-m3n4o5 - User profile page
    Status: In Progress (frontend) ⏳
    
  • task-p6q7r8 - API documentation
    Status: Completed ✅
    
  • task-s9t0u1 - Database optimization
    Status: Queued

Low Priority (1):
  • task-v2w3x4 - Code cleanup
    Status: Pending
```

### Task Status
```bash
$ ccswarm task status task-m3n4o5

📊 Task Details
════════════════════════════════════════════

ID: task-m3n4o5
Description: User profile page
Priority: Medium
Type: Feature
Created: 2024-06-24 10:30:00
Agent: frontend-specialist

Progress Timeline:
  10:30 - Task created
  10:31 - Assigned to frontend agent
  10:32 - Work started
  10:45 - Created ProfilePage component
  10:52 - Added form validation
  11:05 - Styling with Tailwind CSS
  11:15 - Current: Writing tests

Estimated Completion: ~15 minutes

Files Modified:
  • src/pages/ProfilePage.tsx (created)
  • src/components/ProfileForm.tsx (created)
  • src/routes/index.ts (modified)
  • tests/profile.test.tsx (created)
```

### Update Task
```bash
$ ccswarm task update task-v2w3x4 --priority high

📝 Task Updated
════════════════════════════════════════════

Task: task-v2w3x4
Changed: Priority low → high
Status: Re-queued with higher priority
```

### Cancel Task
```bash
$ ccswarm task cancel task-s9t0u1 --reason "No longer needed"

❌ Task Cancelled
════════════════════════════════════════════

Task: task-s9t0u1 (Database optimization)
Reason: No longer needed
Status: Removed from queue
```

## Task Modifiers in Description

You can include modifiers directly in the task description:

```bash
# Priority modifiers
ccswarm task "Fix login bug [high]"
ccswarm task "Update logo [low]"

# Type modifiers
ccswarm task "Payment not working [bug]"
ccswarm task "Add cart functionality [feature]"

# Combined modifiers
ccswarm task "Site is slow [high] [bug]"
ccswarm task "Write auth tests [test] [medium]"

# Auto-accept modifier
ccswarm task "Format code [auto]"
```

## Integration Features

### Proactive Task Suggestions
When proactive mode is enabled, Master Claude suggests tasks:
```
💡 Suggested Tasks (based on codebase analysis):
1. Add error handling to payment service
2. Increase test coverage for auth module (currently 67%)
3. Update deprecated dependencies
```

### Quality Review Integration
Failed quality checks automatically create remediation tasks:
```
❌ Quality Check Failed
Creating remediation task: "Fix ESLint errors in components/"
Assigned to: frontend-specialist
Priority: High
```

## Related Commands
- `ccswarm delegate` - Manual task delegation
- `ccswarm agent list` - View available agents
- `ccswarm session stats` - Task completion metrics and AI-Session token savings
- `ccswarm tui` - Monitor tasks in real-time
- `ccswarm session` - Manage AI sessions for task execution

## Task Execution & Sessions
Tasks are executed within AI-Session contexts that provide:
- **93% token savings** through intelligent conversation compression
- **Context persistence** across task executions
- **Session reuse** for related tasks on the same agent
- **Automatic recovery** from crashes or restarts

See [session.md](session.md) for detailed session management.

## Related Documentation
- **[Session Management](session.md)** - How tasks use AI sessions for execution
- **[AI-Session Documentation](../crates/ai-session/docs/README.md)** - Complete session management guide
- **[Agent Management](agents.md)** - Understanding specialized agents
- **[Quality Review](quality.md)** - Automatic code quality checks
- **[Architecture Overview](../docs/ARCHITECTURE.md)** - How tasks flow through the system