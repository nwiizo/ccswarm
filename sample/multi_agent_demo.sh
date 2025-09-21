#!/bin/bash

# Multi-Agent Collaboration Demo
# 複数エージェントの協調動作を示すデモ

set -e

echo "🤖 CCSwarm Multi-Agent Collaboration Demo"
echo "=========================================="
echo ""

# カラー設定
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# クリーンアップ関数
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    ccswarm stop || true
}
trap cleanup EXIT

echo -e "${YELLOW}📋 Step 1: オーケストレーターの起動${NC}"
echo "Starting the orchestrator with multiple agents..."

# バックグラウンドでオーケストレーターを起動
ccswarm start --daemon &
ORCHESTRATOR_PID=$!

# 起動を待つ
sleep 3

echo -e "${GREEN}✅ Orchestrator started (PID: $ORCHESTRATOR_PID)${NC}"

echo ""
echo -e "${YELLOW}📋 Step 2: プロジェクトシミュレーション${NC}"
echo "Simulating a web application development project..."

# Webアプリケーション開発プロジェクトをシミュレート
PROJECT_NAME="webapp-$(date +%s)"

echo ""
echo -e "${CYAN}🎯 Creating project: $PROJECT_NAME${NC}"
ccswarm init --name "$PROJECT_NAME" --agents frontend,backend,devops,qa

echo ""
echo -e "${YELLOW}📋 Step 3: タスクの分散${NC}"
echo "Distributing tasks across agents..."

# フロントエンドタスク
echo -e "${MAGENTA}[Frontend Agent]${NC} Creating UI components..."
ccswarm task create \
  --description "Design and implement responsive navbar component" \
  --agent frontend \
  --priority high

ccswarm task create \
  --description "Create user dashboard with data visualization" \
  --agent frontend \
  --priority medium

# バックエンドタスク
echo -e "${BLUE}[Backend Agent]${NC} Building API endpoints..."
ccswarm task create \
  --description "Design RESTful API for user management" \
  --agent backend \
  --priority critical

ccswarm task create \
  --description "Implement database schema and migrations" \
  --agent backend \
  --priority high

# DevOpsタスク
echo -e "${CYAN}[DevOps Agent]${NC} Setting up infrastructure..."
ccswarm task create \
  --description "Configure CI/CD pipeline with GitHub Actions" \
  --agent devops \
  --priority high

ccswarm task create \
  --description "Set up Docker containers for microservices" \
  --agent devops \
  --priority medium

# QAタスク
echo -e "${GREEN}[QA Agent]${NC} Preparing test suites..."
ccswarm task create \
  --description "Write unit tests for critical components" \
  --agent qa \
  --priority high

ccswarm task create \
  --description "Create E2E test scenarios for user workflows" \
  --agent qa \
  --priority medium

echo ""
echo -e "${YELLOW}📋 Step 4: エージェント間の協調${NC}"
echo "Demonstrating inter-agent communication..."

# Sanghaモードで民主的な決定
echo ""
echo -e "${CYAN}🗳️ Running Sangha consensus for architecture decision...${NC}"
ccswarm sangha propose \
  --title "Choose database technology" \
  --options "PostgreSQL,MongoDB,DynamoDB" \
  --timeout 10

echo ""
echo -e "${YELLOW}📋 Step 5: 進捗モニタリング${NC}"
ccswarm status --detailed

echo ""
echo -e "${YELLOW}📋 Step 6: 品質レビュー${NC}"
echo "Running quality review across all agents..."
ccswarm review --all --strict

echo ""
echo -e "${YELLOW}📋 Step 7: 統合結果${NC}"
echo "Showing integrated results from all agents..."

# タスクの完了状況を表示
ccswarm task list --all --format table

# エージェントのメトリクスを表示
echo ""
echo -e "${CYAN}📊 Agent Metrics:${NC}"
ccswarm agents --metrics

echo ""
echo -e "${BLUE}💡 Advanced Features:${NC}"
echo "- Use 'ccswarm tui' for real-time monitoring interface"
echo "- Use 'ccswarm delegate analyze' for task delegation analysis"
echo "- Use 'ccswarm session list' to view all active sessions"
echo "- Use 'ccswarm resource status' to check resource usage"

echo ""
echo -e "${GREEN}✅ Multi-agent collaboration demo completed!${NC}"
echo ""
echo "Summary:"
echo "- Created a project with 4 specialized agents"
echo "- Distributed 8 tasks across different agents"
echo "- Demonstrated inter-agent communication"
echo "- Showed quality review and monitoring capabilities"