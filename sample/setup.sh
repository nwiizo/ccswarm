#!/bin/bash

# CCSwarm Sample Setup Script
# サンプルプロジェクトの初期セットアップ

set -e

echo "🔧 CCSwarm Sample Setup"
echo "======================="
echo ""

# カラー設定
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# 現在のディレクトリを確認
SAMPLE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CCSWARM_DIR="$(dirname "$SAMPLE_DIR")"

echo -e "${BLUE}📁 Working directory: $SAMPLE_DIR${NC}"
echo -e "${BLUE}📁 CCSwarm directory: $CCSWARM_DIR${NC}"
echo ""

# 1. ccswarmのビルドとインストール
echo -e "${YELLOW}Step 1: Building ccswarm...${NC}"
cd "$CCSWARM_DIR"
if cargo build --release --features claude-acp; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

# 2. ccswarmをPATHに追加（ローカル実行用）
export PATH="$CCSWARM_DIR/target/release:$PATH"
echo -e "${GREEN}✅ ccswarm added to PATH${NC}"

# 3. 設定ファイルのコピー
echo ""
echo -e "${YELLOW}Step 2: Setting up configuration...${NC}"
cd "$SAMPLE_DIR"
if [ ! -f ~/.ccswarm/config.yaml ]; then
    mkdir -p ~/.ccswarm
    cp ccswarm.yaml ~/.ccswarm/config.yaml
    echo -e "${GREEN}✅ Configuration file copied to ~/.ccswarm/config.yaml${NC}"
else
    echo -e "${BLUE}ℹ️ Configuration file already exists${NC}"
fi

# 4. サンプルプロジェクトディレクトリの作成
echo ""
echo -e "${YELLOW}Step 3: Creating sample project structure...${NC}"
mkdir -p sample-webapp/{frontend,backend,infrastructure,tests,docs}

# package.json for frontend
cat > sample-webapp/frontend/package.json << 'EOF'
{
  "name": "sample-frontend",
  "version": "1.0.0",
  "description": "Sample frontend for ccswarm demo",
  "scripts": {
    "dev": "echo 'Starting development server...'",
    "build": "echo 'Building production bundle...'",
    "test": "echo 'Running tests...'"
  }
}
EOF

# requirements.txt for backend
cat > sample-webapp/backend/requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
pydantic==2.5.0
sqlalchemy==2.0.23
EOF

# Docker compose for infrastructure
cat > sample-webapp/infrastructure/docker-compose.yml << 'EOF'
version: '3.8'
services:
  frontend:
    build: ../frontend
    ports:
      - "3000:3000"
  backend:
    build: ../backend
    ports:
      - "8000:8000"
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: sampledb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
EOF

echo -e "${GREEN}✅ Sample project structure created${NC}"

# 5. 環境変数の設定
echo ""
echo -e "${YELLOW}Step 4: Setting up environment variables...${NC}"
cat > .env << 'EOF'
# Claude ACP Configuration
CCSWARM_CLAUDE_ACP_URL=ws://localhost:9100
CCSWARM_CLAUDE_ACP_AUTO_CONNECT=true
CCSWARM_CLAUDE_ACP_TIMEOUT=30
CCSWARM_CLAUDE_ACP_MAX_RETRIES=3

# Logging
RUST_LOG=ccswarm=info

# Project
PROJECT_NAME=sample-webapp
EOF

echo -e "${GREEN}✅ Environment variables configured${NC}"

# 6. 初期化完了メッセージ
echo ""
echo -e "${GREEN}🎉 Setup completed successfully!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Start Claude Code (if not already running)"
echo "2. Run the demos:"
echo "   ./claude_acp_demo.sh    - Test Claude Code integration"
echo "   ./task_management_demo.sh - Try task management"
echo "   ./multi_agent_demo.sh   - See multi-agent collaboration"
echo ""
echo "3. Or start ccswarm manually:"
echo "   ccswarm init --name my-project"
echo "   ccswarm start"
echo "   ccswarm claude-acp test"
echo ""
echo -e "${YELLOW}📝 Configuration file: ~/.ccswarm/config.yaml${NC}"
echo -e "${YELLOW}📁 Sample project: ./sample-webapp/${NC}"
echo ""
echo "Happy coding with ccswarm! 🚀"