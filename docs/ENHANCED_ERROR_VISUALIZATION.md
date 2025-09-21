# Enhanced Error Visualization in ccswarm

## Overview

The enhanced error visualization system in ccswarm provides:

1. **Visual Error Diagrams** - ASCII art diagrams that explain error contexts
2. **Recovery Suggestions** - Step-by-step recovery procedures
3. **Auto-fix Capabilities** - Automatic resolution for common errors
4. **Interactive Error Resolution** - Guided troubleshooting

## Features

### 1. Visual Error Diagrams

Each error type includes a visual diagram to help users understand the problem:

- **Network Errors**: Connection flow diagram
- **Session Errors**: Session lifecycle visualization
- **Git Worktree Errors**: Worktree structure diagram
- **Permission Errors**: File permission hierarchy
- **Configuration Errors**: Config file structure
- **Task Format Errors**: Task processing flow
- **API Key Errors**: Setup flow diagram
- **Agent Communication Errors**: Agent interaction diagram

### 2. Error Codes

All errors have standardized codes for quick reference:

| Code | Category | Description |
|------|----------|-------------|
| ENV001 | Environment | API Key Missing |
| SES001 | Session Management | Session Not Found |
| CFG001 | Configuration | Configuration Not Found |
| GIT001 | Version Control | Git Not Initialized |
| PRM001 | File System | Permission Denied |
| NET001 | Network | Network Connection Failed |
| WRK001 | Version Control | Git Worktree Conflict |
| AGT001 | Agent Management | Agent Busy |
| TSK001 | Task Management | Invalid Task Format |
| AI001 | AI Provider | AI Response Error |

### 3. Auto-fix Capabilities

Errors that can be automatically fixed:
- Session creation (SES001)
- Configuration generation (CFG001)
- Git initialization (GIT001)
- Permission fixes (PRM001)
- Worktree cleanup (WRK001)

### 4. Interactive Commands

#### Diagnose specific error:
```bash
ccswarm doctor --error <CODE>
```

#### Auto-fix error:
```bash
ccswarm doctor --error <CODE> --fix
```

#### List all error codes:
```bash
ccswarm help errors
```

#### Run commands with auto-fix:
```bash
ccswarm <command> --fix
```

#### Check API connectivity:
```bash
ccswarm doctor --check-api
```

## Implementation Details

### Module Structure

```
src/utils/
├── error.rs              # Base error handling utilities
├── user_error.rs         # User-friendly error wrapper
├── error_diagrams.rs     # Visual ASCII diagrams
├── error_recovery.rs     # Recovery actions and auto-fix
└── error_tests.rs        # Comprehensive tests
```

### Key Components

#### UserError
Enhanced error type with:
- Title and details
- Multiple suggestions
- Error code
- Visual diagram
- Auto-fix capability flag

#### ErrorRecoveryDB
Database of recovery actions:
- Recovery steps (commands, file creation, env vars)
- Risk levels (Safe, Low, Medium, High)
- Auto-fix capability

#### ErrorDiagrams
ASCII art generators for each error type

#### ErrorResolver
Interactive resolution system

### Usage Example

```rust
// Creating an enhanced error
CommonErrors::api_key_missing("Anthropic")
    .display();

// With auto-fix in CLI
if cli.fix {
    error.display_and_fix(true).await?;
}
```

## Benefits

1. **Reduced Support Burden** - Users can self-diagnose and fix common issues
2. **Better User Experience** - Visual diagrams make errors less intimidating
3. **Faster Resolution** - Auto-fix capabilities resolve issues instantly
4. **Standardization** - Consistent error codes across the system
5. **Discoverability** - Help system integration makes solutions easy to find

## Future Enhancements

1. **Machine Learning** - Learn from user fixes to improve suggestions
2. **Community Fixes** - Share successful fixes between users
3. **Error Metrics** - Track which errors occur most frequently
4. **Custom Diagrams** - Allow plugins to add their own error diagrams
5. **Localization** - Translate error messages and diagrams