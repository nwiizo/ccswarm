# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-06-14

### Added
- **Interleaved Thinking Pattern**: Agents can now evaluate and adjust their approach mid-execution
  - Decision types: Continue, Refine, RequestContext, Pivot, Complete, Abort
  - Confidence-based decision making with role-specific patterns
  - Thinking history tracking for debugging and transparency

- **LLM Quality Judge System**: Advanced code quality evaluation inspired by Anthropic's research
  - Multi-dimensional evaluation across 8 quality aspects
  - Role-specific scoring adjustments (Frontend, Backend, DevOps, QA)
  - Intelligent issue categorization by severity (Critical, High, Medium, Low)
  - Actionable remediation instructions with effort estimates
  - Performance-optimized with evaluation caching

- **Enhanced Quality Review Cycle**: Automated quality checks with remediation
  - Periodic quality reviews every 30 seconds
  - Automatic remediation task creation for quality issues
  - Review history tracking with iteration counting
  - Re-evaluation after remediation completion

### Changed
- Improved task delegation accuracy with enhanced pattern matching
- Updated agent boundary checking to evaluate forbidden patterns first
- Enhanced test infrastructure with temporary directories for isolation
- Optimized session management for better token efficiency

### Fixed
- Fixed task queue tests using persistent directories
- Corrected Backend delegation rules to use AND conditions
- Fixed DevOps boundary checker to properly delegate application tasks
- Resolved all clippy warnings for CI/CD compliance
- Fixed identity test environment variable population

### Developer Experience
- All tests now pass successfully
- `cargo fmt --check` compliant
- `cargo clippy` shows no warnings
- Improved error messages and logging

## [0.2.1] - 2025-06-01

### Changed
- Minor bug fixes and performance improvements

## [0.2.0] - 2025-05-15

### Added
- Session persistence with 93% token reduction
- Master delegation system with multiple strategies
- Auto-create system for generating complete applications
- Multi-provider support (Claude Code, Aider, OpenAI Codex)
- Enhanced TUI with real-time monitoring
- Git worktree-based agent isolation

### Changed
- Improved identity management system
- Enhanced safety features with risk assessment
- Better error recovery mechanisms

## [0.1.0] - 2025-04-01

### Added
- Initial release with basic multi-agent orchestration
- Simple task delegation
- Basic monitoring capabilities