# Release Notes

## v0.2.0 (2025-01-06)

### 🎉 New Features

#### Quality Review System
- **Automated Code Review**: Master Claude now performs quality reviews on completed tasks every 30 seconds
- **Remediation Tasks**: Automatically creates fix tasks when quality issues are detected
- **Review History Tracking**: Maintains history of all reviews and remediations for each task
- **Smart Fix Instructions**: Generates specific instructions based on detected issues:
  - Low test coverage → Add unit tests to achieve 85% coverage
  - High complexity → Refactor to reduce cyclomatic complexity  
  - Security issues → Fix vulnerabilities and validate inputs
  - Missing docs → Add comprehensive documentation

### 🔧 Improvements

- **Default Execution Mode**: Changed `dangerous_skip` default from `false` to `true` for automated execution
- **Enhanced Task Types**: Added `TaskType::Remediation` for quality fix tasks
- **Message Handling**: Implemented `AgentMessage::QualityIssue` handler in orchestrator
- **Task Relationships**: Added parent task tracking and quality issue fields to Task struct

### 📚 Documentation

- Updated README.md with comprehensive quality review system documentation
- Enhanced CLAUDE.md with implementation details and usage examples
- Added review workflow diagrams and fix instruction examples

### 🐛 Bug Fixes

- Fixed all compilation warnings in task initialization
- Resolved unused variable warnings in test files
- Fixed type mismatches in provider implementations
- Corrected pattern matching for new TaskType variants

### 🏗️ Architecture Changes

- Added review module to orchestrator for quality management
- Enhanced Task struct with quality-related fields:
  - `assigned_to`: Direct agent assignment
  - `parent_task_id`: Link to original task
  - `quality_issues`: List of detected problems
- Implemented review history tracking with `ReviewHistoryEntry` struct

### 🧪 Testing

- Added comprehensive tests for quality review workflow
- Added tests for fix instruction generation
- Added tests for review history tracking
- All existing tests pass with new functionality

### 💥 Breaking Changes

- None - All changes are backward compatible

### 🚀 Migration Guide

No migration required. The quality review system is automatically enabled and will start reviewing completed tasks after updating.

To customize review behavior, update your `ccswarm.json`:

```json
{
  "coordination": {
    "quality_gate_frequency": "on_commit",  // or "periodic", "on_milestone"
    "master_review_trigger": "all_tasks_complete"  // or "after_each_task"
  }
}
```

### 🙏 Acknowledgments

Thanks to all contributors and users who provided feedback for this release!