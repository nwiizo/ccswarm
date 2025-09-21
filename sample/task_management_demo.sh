#!/bin/bash

# Task Management Demo Script
# ccswarmのタスク管理機能を示すデモ

set -e

echo "🎯 CCSwarm Task Management Demo"
echo "================================"
echo ""

# カラー設定
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# プロジェクトの初期化
echo -e "${YELLOW}📋 Step 1: プロジェクトの初期化${NC}"
echo "Initializing a new project..."

PROJECT_NAME="demo-project-$(date +%s)"
ccswarm init --name "$PROJECT_NAME" --agents frontend,backend,qa

echo -e "${GREEN}✅ Project initialized: $PROJECT_NAME${NC}"

echo ""
echo -e "${YELLOW}📋 Step 2: タスクの作成${NC}"
echo "Creating various tasks..."

# フロントエンドタスク
ccswarm task create \
  --description "Create React login component" \
  --agent frontend \
  --priority high

# バックエンドタスク
ccswarm task create \
  --description "Implement user authentication API" \
  --agent backend \
  --priority critical

# QAタスク
ccswarm task create \
  --description "Write E2E tests for login flow" \
  --agent qa \
  --priority medium

echo -e "${GREEN}✅ Tasks created successfully${NC}"

echo ""
echo -e "${YELLOW}📋 Step 3: タスクリストの確認${NC}"
ccswarm task list --all

echo ""
echo -e "${YELLOW}📋 Step 4: タスクの実行状況確認${NC}"
ccswarm status --detailed

echo ""
echo -e "${YELLOW}📋 Step 5: エージェント状態の確認${NC}"
ccswarm agents --all

echo ""
echo -e "${BLUE}💡 Demo Tips:${NC}"
echo "- Use 'ccswarm task update <id> --status completed' to mark tasks as done"
echo "- Use 'ccswarm task assign <id> --agent <name>' to reassign tasks"
echo "- Use 'ccswarm logs --agent <name>' to view agent-specific logs"
echo ""
echo -e "${GREEN}✅ Task management demo completed!${NC}"