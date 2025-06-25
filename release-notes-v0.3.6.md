# v0.3.6 - Enhanced Error Handling & Developer Experience

## 🎉 Highlights

This release significantly improves the developer experience with enhanced error handling, resource monitoring, and streamlined project initialization. We've also resolved all code quality issues identified by clippy and improved overall system performance.

## ✨ Major Features

### 🔍 Enhanced Error Visualization
- **Visual Error Diagrams**: Rich ASCII art diagrams for common errors
- **Context-Aware Help**: Detailed error explanations with resolution steps
- **Error Pattern Recognition**: Intelligent suggestions based on error history
- **Interactive Error Recovery**: Step-by-step guides for fixing issues

### 📊 Resource Monitoring System
- **Real-time Tracking**: Monitor CPU, memory, and disk usage
- **Configurable Limits**: Set resource thresholds for agents
- **Alert System**: Warnings when approaching resource limits
- **Historical Data**: Track resource usage over time

### 📋 Template Management
- **Project Templates**: Pre-configured setups for common project types
- **Agent Templates**: Role-specific configurations and personalities
- **Version Control**: Template versioning with metadata
- **Custom Templates**: Create and share your own templates

### 🔄 Message Conversion Framework
- **Unified Format**: Seamless conversion between ccswarm and ai-session messages
- **Type Safety**: Strongly typed message conversions
- **Extensible**: Easy to add new message types
- **Backward Compatible**: Works with existing message formats

### 🚀 Quickstart Command
- **Interactive Setup**: Guided project initialization
- **Smart Defaults**: Intelligent configuration suggestions
- **Template Selection**: Choose from predefined templates
- **Immediate Productivity**: Get started in seconds

## 🐛 Bug Fixes & Improvements

### Code Quality
- Fixed all clippy warnings including:
  - Collapsible if statements
  - Unused variables and functions
  - Needless borrows
  - Complex type definitions
  - Async/await best practices

### Performance
- Optimized memory usage in resource monitoring
- Improved message bus efficiency
- Reduced context switching overhead
- Better iterator usage patterns

### Developer Experience
- Cleaner error messages
- Better documentation
- Improved test coverage
- Simplified API interfaces

## 📦 Installation

```bash
# Install from source
cargo install --path crates/ccswarm --version 0.3.6

# Or clone and build
git clone https://github.com/nwiizo/ccswarm
cd ccswarm
cargo build --release
```

## 🚀 Quick Start

```bash
# Initialize a new project with quickstart
ccswarm quickstart

# Monitor system resources
ccswarm resource status

# View available templates
ccswarm template list

# Get help for any error
ccswarm help error E001
```

## 🔧 Technical Details

### Breaking Changes
- None in this release

### Dependencies
- Updated ai-session to v0.3.6
- All dependencies remain compatible

### Compatibility
- Linux: ✅ Fully supported
- macOS: ✅ Fully supported  
- Windows: ❌ Not supported (Unix-specific dependencies)

## 📈 What's Next

We're already working on v0.4.0 which will include:
- Advanced agent collaboration patterns
- Plugin system for custom extensions
- Enhanced security features
- Performance optimizations for large-scale deployments

## 🙏 Acknowledgments

Thanks to all contributors who helped identify and fix issues in this release. Special thanks for the feedback on error handling and resource monitoring features.

## 📝 Full Changelog

See [CHANGELOG.md](https://github.com/nwiizo/ccswarm/blob/v0.3.6/CHANGELOG.md) for complete details.