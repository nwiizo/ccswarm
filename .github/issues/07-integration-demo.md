# Issue #7: Day 10 - Integration Tests and Demo

## Overview
Run integration tests for Claude Code ACP integration and prepare demo.

## Task List

### 1. Integration Tests
- [ ] Create end-to-end tests:
```rust
#[tokio::test]
async fn test_full_workflow() {
    // 1. Connect
    // 2. Send task
    // 3. Verify result
    // 4. Disconnect
}
```

### 2. Performance Tests
- [ ] Measure connection time
- [ ] Measure task execution time
- [ ] Check memory usage

### 3. Demo Scenarios
- [ ] Basic task execution demo
- [ ] Error recovery demo
- [ ] Multiple consecutive task execution demo

### 4. Demo Script
```bash
#!/bin/bash
# demo.sh
echo "🚀 Claude Code ACP Demo"

# 1. Connection test
ccswarm claude-acp test

# 2. Simple task
ccswarm claude-acp send "Write hello world"

# 3. Complex task
ccswarm task "Create TODO app" --via-acp
```

### 5. Video/Screenshots
- [ ] Demo execution screenshots
- [ ] GIF animations of main features
- [ ] Setup procedure images

## Acceptance Criteria
- [ ] All integration tests succeed
- [ ] Demo script runs without issues
- [ ] Performance meets requirements (connection under 5 seconds)
- [ ] Demo materials complete

## Deliverables
- [ ] Test report
- [ ] Demo script
- [ ] Screenshots/videos
- [ ] Presentation materials

## Estimated Time
6-8 hours

## Labels
- `task`
- `day-10`
- `integration-test`
- `demo`
- `claude-acp`
