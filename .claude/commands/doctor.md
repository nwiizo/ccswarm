# ccswarm doctor

System health check and diagnostic tool.

## Usage
```bash
ccswarm doctor [OPTIONS]
```

## Options
- `-f, --fix` - Automatically fix common issues

## Description
Performs comprehensive system health checks and can automatically fix common problems. Checks include:
- Git installation and configuration
- API key configuration
- Port availability
- Session health
- Configuration validity
- Disk space

## Examples

### Basic Health Check
```bash
$ ccswarm doctor
🏥 ccswarm System Diagnostics
════════════════════════════════

Checking git... ✅ Installed (2.43.0)
Checking API keys... ✅ ANTHROPIC_API_KEY set
Checking configuration... ✅ Valid
Checking disk space... ✅ 125GB available
Checking ports... ❌ Port 8080 in use
Checking sessions... ✅ No active sessions

Issues found: 1

💡 Suggestions:
1. Port 8080 is in use. Run with --port 8081 or kill the process using:
   lsof -ti:8080 | xargs kill -9
```

### Auto-Fix Mode
```bash
$ ccswarm doctor --fix
🏥 ccswarm System Diagnostics
════════════════════════════════

Checking git... ✅ Installed
Checking API keys... ⚠️ ANTHROPIC_API_KEY not set
   🔧 Fixing: Creating .env file template...
   ✅ Fixed! Please edit .env and add your API key

Checking configuration... ❌ Not found
   🔧 Fixing: Running setup wizard...
   ✅ Fixed! Configuration created

All issues resolved! ✅
```

## Checks Performed
1. **Git availability** - Required for worktree management
2. **API keys** - Checks for required environment variables
3. **Configuration** - Validates ccswarm.json
4. **Port availability** - Ensures required ports are free
5. **Session health** - Checks for stuck sessions
6. **Disk space** - Ensures sufficient space for operations
7. **Dependencies** - Verifies required tools

## Fix Actions
- Creates `.env` template if missing
- Runs setup wizard for missing config
- Cleans up stuck sessions
- Suggests alternative ports
- Creates required directories

## Related Commands
- `ccswarm setup` - Interactive configuration
- `ccswarm status` - View system status
- `ccswarm session cleanup` - Clean stuck sessions