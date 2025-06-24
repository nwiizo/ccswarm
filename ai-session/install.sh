#!/bin/bash
# AI-Session Installation Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="${HOME}/.local/bin"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🚀 AI-Session Installation"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: Rust/Cargo not found${NC}"
    echo "Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Build the project
echo "📦 Building ai-session..."
cd "$PROJECT_ROOT"
cargo build --package ai-session --bin ai-session --release

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Copy binaries
echo "📋 Installing binaries..."
cp "$PROJECT_ROOT/target/release/ai-session" "$INSTALL_DIR/"

# Create claude-chat wrapper
cat > "$INSTALL_DIR/claude-chat" << 'EOF'
#!/bin/bash
# Claude Chat - Quick access to Claude Code

exec ai-session claude-chat "$@"
EOF

chmod +x "$INSTALL_DIR/claude-chat"

# Create ai-session-server wrapper
cat > "$INSTALL_DIR/ai-session-server" << 'EOF'
#!/bin/bash
# AI-Session HTTP Server

exec ai-session-server --features server "$@"
EOF

chmod +x "$INSTALL_DIR/ai-session-server"

# Check if PATH includes install directory
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo -e "${YELLOW}⚠️  Note: $INSTALL_DIR is not in your PATH${NC}"
    echo ""
    echo "Add this to your shell profile (~/.bashrc or ~/.zshrc):"
    echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    echo ""
fi

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "Available commands:"
echo "  ai-session       - Main CLI tool"
echo "  claude-chat      - Quick chat with Claude Code"
echo "  ai-session-server - HTTP API server"
echo ""
echo "Quick start:"
echo "  1. Start server: ai-session-server --port 4000 &"
echo "  2. Chat with Claude: claude-chat"
echo ""