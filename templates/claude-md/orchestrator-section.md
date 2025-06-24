# Agent Orchestrator Process

When handling complex tasks, follow this orchestration pattern to break down work into manageable steps with parallel execution.

## Orchestration Process

### Step 1: Initial Analysis (Single Task)
Always begin by analyzing the entire task to understand:
- Task scope and requirements
- Dependencies between subtasks
- Required resources and tools
- Expected outcomes

### Step 2: Task Planning
Break down the task into 2-4 sequential steps:
- Each step should have a clear objective
- Steps execute in order (dependencies respected)
- Each step can contain multiple parallel subtasks
- Define what context each step needs from previous steps

### Step 3: Parallel Execution Within Steps
For each step:
- Identify independent subtasks that can run simultaneously
- Execute all parallel tasks within the step
- Collect results from all parallel tasks
- Only proceed to next step when all current tasks complete

### Step 4: Review and Adapt
After each step:
- Review the results from all parallel tasks
- Determine if the remaining plan is still appropriate
- Adapt the plan based on discoveries:
  - Skip unnecessary steps
  - Add new steps if issues found
  - Modify upcoming steps based on results

### Step 5: Progressive Synthesis
- Build understanding incrementally across steps
- Pass only essential summaries between steps (100-200 words)
- Maintain a comprehensive view while avoiding context overflow
- Synthesize final results from all steps

## Example Orchestration

### Quality Check Workflow
```
Step 1: Analysis (1 task)
- Analyze project structure

Step 2: Quality Checks (3 parallel tasks)
- Run tests
- Run linting
- Check formatting

Step 3: Fix Issues (conditional, based on Step 2)
- Fix test failures (if any)
- Fix lint errors (if any)
- Fix formatting (if any)

Step 4: Validation (2 parallel tasks)
- Re-run all checks
- Generate quality report
```

### Implementation Workflow
```
Step 1: Requirements Analysis (2 parallel tasks)
- Analyze existing code
- Check dependencies

Step 2: Implementation (1 task)
- Implement the feature

Step 3: Testing (2 parallel tasks)
- Write unit tests
- Run integration tests

Step 4: Documentation (2 parallel tasks)
- Update API docs
- Update README
```

## Key Benefits

1. **Efficiency**: Parallel tasks within steps maximize throughput
2. **Adaptability**: Plan adjusts based on intermediate results
3. **Clarity**: Clear dependencies and step progression
4. **Memory Optimization**: Minimal context passed between steps
5. **Error Recovery**: Issues identified early can be fixed in subsequent steps

## Implementation Guidelines

- Start with analysis to understand the full scope
- Group related parallel tasks in the same step
- Keep step summaries concise (100-200 words)
- Explicitly state what context is needed for each step
- Always consider if remaining steps need adjustment
- Use clear success/failure criteria for each task