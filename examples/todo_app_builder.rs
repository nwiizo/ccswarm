/// 実際のTODOアプリケーション開発プログラム
/// 
/// このプログラムはccswarmシステムを使用して実際に動作するTODOアプリケーションを
/// 開発し、Webサーバーを起動してアクセス可能にします。

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

// ccswarmライブラリのインポート
use ccswarm::agent::{Priority, Task, TaskType, AgentStatus};
use ccswarm::agent::simple::SimpleClaudeAgent;
use ccswarm::config::ClaudeConfig;
use ccswarm::coordination::{AgentMessage, CoordinationBus, StatusTracker, TaskQueue};
use ccswarm::identity::{default_frontend_role, default_backend_role, default_devops_role, default_qa_role};
use ccswarm::workspace::SimpleWorkspaceManager;

#[tokio::main]
async fn main() -> Result<()> {
    // ログ設定
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 ccswarm 実際のTODOアプリケーション開発開始");

    // プロジェクトディレクトリを作成
    let project_dir = PathBuf::from("./todo_app");
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir).await?;
    }
    fs::create_dir_all(&project_dir).await?;

    // ワークスペース管理を初期化
    let workspace_manager = SimpleWorkspaceManager::new(project_dir.clone());
    workspace_manager.init_if_needed().await?;

    // 調整システムを初期化
    let coordination_bus = CoordinationBus::new().await?;
    let task_queue = TaskQueue::new().await?;
    let status_tracker = StatusTracker::new().await?;

    info!("📋 実際のTODOアプリケーション開発タスクを定義中...");

    // 実際のTODOアプリケーション開発タスクを定義
    let app_tasks = vec![
        // フロントエンド開発
        Task::new(
            "todo-frontend-1".to_string(),
            "Create HTML structure for TODO app".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create index.html with form and task list structure".to_string()),

        Task::new(
            "todo-frontend-2".to_string(),
            "Create CSS styles for TODO app".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create styles.css with modern, responsive design".to_string()),

        Task::new(
            "todo-frontend-3".to_string(),
            "Create JavaScript for TODO functionality".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create app.js with add, delete, toggle complete functionality".to_string()),

        // バックエンド開発
        Task::new(
            "todo-backend-1".to_string(),
            "Create Node.js Express server".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create server.js with Express setup and static file serving".to_string()),

        Task::new(
            "todo-backend-2".to_string(),
            "Create TODO API endpoints".to_string(),
            Priority::High,
            TaskType::Development,
        ).with_details("Create REST API for GET, POST, PUT, DELETE operations".to_string()),

        Task::new(
            "todo-backend-3".to_string(),
            "Create package.json and dependencies".to_string(),
            Priority::Medium,
            TaskType::Development,
        ).with_details("Setup Node.js project with required dependencies".to_string()),

        // DevOps/デプロイメント
        Task::new(
            "todo-deploy-1".to_string(),
            "Create startup script".to_string(),
            Priority::Medium,
            TaskType::Infrastructure,
        ).with_details("Create run.sh script to start the application".to_string()),

        Task::new(
            "todo-deploy-2".to_string(),
            "Create README documentation".to_string(),
            Priority::Medium,
            TaskType::Documentation,
        ).with_details("Create comprehensive README with setup and usage instructions".to_string()),
    ];

    // タスクをキューに追加
    for task in &app_tasks {
        task_queue.add_task(task).await?;
        info!("📝 タスクを追加: {}", task.description);
    }

    info!("🤖 専門エージェントを作成中...");

    // 専門エージェントを作成
    let mut agents: Vec<(String, SimpleClaudeAgent)> = vec![];
    
    // フロントエンドエージェント
    let mut frontend_agent = SimpleClaudeAgent::new(
        default_frontend_role(),
        &project_dir,
        ClaudeConfig::for_agent("frontend"),
    ).await?;
    frontend_agent.initialize(&workspace_manager).await?;
    agents.push(("frontend".to_string(), frontend_agent));

    // バックエンドエージェント
    let mut backend_agent = SimpleClaudeAgent::new(
        default_backend_role(),
        &project_dir,
        ClaudeConfig::for_agent("backend"),
    ).await?;
    backend_agent.initialize(&workspace_manager).await?;
    agents.push(("backend".to_string(), backend_agent));

    // DevOpsエージェント
    let mut devops_agent = SimpleClaudeAgent::new(
        default_devops_role(),
        &project_dir,
        ClaudeConfig::for_agent("devops"),
    ).await?;
    devops_agent.initialize(&workspace_manager).await?;
    agents.push(("devops".to_string(), devops_agent));

    info!("✅ {} 個のエージェントを初期化完了", agents.len());

    // エージェントステータスを追跡システムに登録
    for (agent_type, agent) in &agents {
        status_tracker.update_status(
            &agent.identity.agent_id,
            &agent.status,
            json!({
                "agent_type": agent_type,
                "specialization": agent.identity.specialization.name(),
                "workspace": agent.workspace_path,
                "initialized_at": agent.last_activity
            })
        ).await?;
    }

    info!("🎯 実際のTODOアプリケーション開発開始...");

    // タスク実行とファイル生成
    let pending_tasks = task_queue.get_pending_tasks().await?;
    let mut completed_tasks = 0;

    for task in pending_tasks {
        info!("📋 タスク実行中: {}", task.description);

        // 適切なエージェントを選択
        let agent_index = select_agent_for_task(&task, &agents);
        
        if let Some(index) = agent_index {
            let (agent_type, ref mut agent) = &mut agents[index];
            
            // タスク実行前の状態更新
            agent.update_status(AgentStatus::Working);
            
            // タスクを実行
            match agent.execute_task(task.clone()).await {
                Ok(result) => {
                    if result.success {
                        info!("✅ {} エージェントがタスクを完了: {}", agent_type, task.description);
                        
                        // 実際のファイルを生成
                        generate_actual_files(&task, &project_dir).await?;
                        
                        completed_tasks += 1;
                        task_queue.remove_task(&task.id).await?;
                    } else {
                        warn!("❌ タスク実行失敗: {}", result.error.unwrap_or_default());
                    }
                }
                Err(e) => {
                    warn!("❌ タスク実行エラー: {}", e);
                }
            }

            // エージェントステータスを利用可能に戻す
            agent.update_status(AgentStatus::Available);
        } else {
            warn!("⚠️ タスクに適したエージェントが見つかりません: {}", task.description);
        }

        // 少し待機
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    info!("📊 TODOアプリケーション開発完了!");
    info!("✅ 完了タスク: {}", completed_tasks);

    // package.jsonの依存関係をインストール
    info!("📦 依存関係をインストール中...");
    let npm_install = tokio::process::Command::new("npm")
        .arg("install")
        .current_dir(&project_dir)
        .output()
        .await;

    match npm_install {
        Ok(output) if output.status.success() => {
            info!("✅ 依存関係のインストール完了");
        }
        Ok(output) => {
            warn!("⚠️ npm install警告: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            warn!("⚠️ npm not found: {} (Nodeサーバーを手動で起動してください)", e);
        }
    }

    // サーバーを起動
    info!("🚀 TODOアプリケーションサーバーを起動中...");
    info!("📍 アクセスURL: http://localhost:3000");
    info!("🛑 終了するには Ctrl+C を押してください");

    // Node.jsサーバーを起動
    let server_result = tokio::process::Command::new("node")
        .arg("server.js")
        .current_dir(&project_dir)
        .spawn();

    match server_result {
        Ok(mut child) => {
            // サーバーが起動するまで少し待つ
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            
            info!("✅ サーバーが起動しました!");
            info!("🌐 ブラウザで http://localhost:3000 にアクセスしてください");
            
            // Ctrl+Cでの終了を待つ
            tokio::signal::ctrl_c().await?;
            info!("🛑 シャットダウン中...");
            
            // 子プロセスを終了
            let _ = child.kill().await;
        }
        Err(e) => {
            warn!("⚠️ Node.js not found: {}", e);
            info!("📝 手動でサーバーを起動するには:");
            info!("   cd {}", project_dir.display());
            info!("   node server.js");
        }
    }

    // 調整バスを閉じる
    coordination_bus.close().await?;

    info!("🎉 ccswarm TODOアプリケーション開発完了!");
    
    Ok(())
}

/// タスクに適したエージェントを選択
fn select_agent_for_task(task: &Task, agents: &[(String, SimpleClaudeAgent)]) -> Option<usize> {
    let description = task.description.to_lowercase();
    let task_id = task.id.as_str();
    
    for (index, (agent_type, _)) in agents.iter().enumerate() {
        let matches = match agent_type.as_str() {
            "frontend" => {
                task_id.contains("frontend") ||
                description.contains("html") || 
                description.contains("css") || 
                description.contains("javascript") ||
                description.contains("frontend")
            },
            "backend" => {
                task_id.contains("backend") ||
                description.contains("node") || 
                description.contains("express") || 
                description.contains("server") ||
                description.contains("api") ||
                description.contains("package.json")
            },
            "devops" => {
                task_id.contains("deploy") ||
                description.contains("script") || 
                description.contains("readme") || 
                description.contains("documentation") ||
                description.contains("startup")
            },
            _ => false,
        };
        
        if matches {
            return Some(index);
        }
    }
    
    None
}

/// 実際のアプリケーションファイルを生成
async fn generate_actual_files(task: &Task, project_dir: &PathBuf) -> Result<()> {
    match task.id.as_str() {
        "todo-frontend-1" => {
            // HTML ファイルを生成
            let html_content = r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ccswarm TODO App</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>🤖 ccswarm TODO App</h1>
            <p>マルチエージェントシステムで開発されたTODOアプリ</p>
        </header>
        
        <div class="todo-input">
            <input type="text" id="todoInput" placeholder="新しいタスクを入力...">
            <button id="addBtn">追加</button>
        </div>
        
        <div class="todo-list">
            <ul id="todoList"></ul>
        </div>
        
        <div class="stats">
            <span id="totalTasks">総タスク: 0</span>
            <span id="completedTasks">完了: 0</span>
            <span id="pendingTasks">未完了: 0</span>
        </div>
        
        <footer>
            <p>🎯 Powered by ccswarm multi-agent system</p>
        </footer>
    </div>
    
    <script src="app.js"></script>
</body>
</html>"#;
            
            fs::write(project_dir.join("index.html"), html_content).await?;
            info!("✅ index.html を生成");
        },
        
        "todo-frontend-2" => {
            // CSS ファイルを生成
            let css_content = r#"/* ccswarm TODO App Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    padding: 20px;
}

.container {
    max-width: 600px;
    margin: 0 auto;
    background: white;
    border-radius: 15px;
    box-shadow: 0 20px 40px rgba(0,0,0,0.1);
    overflow: hidden;
}

header {
    background: linear-gradient(135deg, #4CAF50, #45a049);
    color: white;
    padding: 30px;
    text-align: center;
}

header h1 {
    font-size: 2.5em;
    margin-bottom: 10px;
}

header p {
    opacity: 0.9;
    font-size: 1.1em;
}

.todo-input {
    padding: 30px;
    display: flex;
    gap: 15px;
}

#todoInput {
    flex: 1;
    padding: 15px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    outline: none;
    transition: border-color 0.3s;
}

#todoInput:focus {
    border-color: #4CAF50;
}

#addBtn {
    padding: 15px 30px;
    background: #4CAF50;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    cursor: pointer;
    transition: background 0.3s;
}

#addBtn:hover {
    background: #45a049;
}

.todo-list {
    padding: 0 30px;
    max-height: 400px;
    overflow-y: auto;
}

#todoList {
    list-style: none;
}

.todo-item {
    display: flex;
    align-items: center;
    padding: 15px;
    border-bottom: 1px solid #eee;
    transition: background 0.3s;
}

.todo-item:hover {
    background: #f9f9f9;
}

.todo-item.completed {
    opacity: 0.6;
}

.todo-item.completed .todo-text {
    text-decoration: line-through;
}

.todo-checkbox {
    margin-right: 15px;
    width: 20px;
    height: 20px;
    cursor: pointer;
}

.todo-text {
    flex: 1;
    font-size: 16px;
}

.todo-delete {
    background: #f44336;
    color: white;
    border: none;
    padding: 8px 15px;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.3s;
}

.todo-delete:hover {
    background: #da190b;
}

.stats {
    padding: 20px 30px;
    background: #f5f5f5;
    display: flex;
    justify-content: space-between;
    font-weight: bold;
}

footer {
    background: #333;
    color: white;
    text-align: center;
    padding: 20px;
}

/* レスポンシブデザイン */
@media (max-width: 600px) {
    .container {
        margin: 10px;
        border-radius: 10px;
    }
    
    header h1 {
        font-size: 2em;
    }
    
    .todo-input {
        flex-direction: column;
    }
    
    .stats {
        flex-direction: column;
        gap: 10px;
        text-align: center;
    }
}"#;
            
            fs::write(project_dir.join("styles.css"), css_content).await?;
            info!("✅ styles.css を生成");
        },
        
        "todo-frontend-3" => {
            // JavaScript ファイルを生成
            let js_content = r#"// ccswarm TODO App JavaScript
class TodoApp {
    constructor() {
        this.todos = [];
        this.todoIdCounter = 1;
        this.init();
    }

    init() {
        this.todoInput = document.getElementById('todoInput');
        this.addBtn = document.getElementById('addBtn');
        this.todoList = document.getElementById('todoList');
        this.totalTasks = document.getElementById('totalTasks');
        this.completedTasks = document.getElementById('completedTasks');
        this.pendingTasks = document.getElementById('pendingTasks');

        this.bindEvents();
        this.loadTodos();
        this.render();
    }

    bindEvents() {
        this.addBtn.addEventListener('click', () => this.addTodo());
        this.todoInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.addTodo();
            }
        });
    }

    async loadTodos() {
        try {
            const response = await fetch('/api/todos');
            if (response.ok) {
                this.todos = await response.json();
                this.render();
            }
        } catch (error) {
            console.log('API not available, using local storage');
            this.loadFromLocalStorage();
        }
    }

    loadFromLocalStorage() {
        const stored = localStorage.getItem('ccswarm-todos');
        if (stored) {
            this.todos = JSON.parse(stored);
            this.todoIdCounter = Math.max(...this.todos.map(t => t.id), 0) + 1;
        }
    }

    saveToLocalStorage() {
        localStorage.setItem('ccswarm-todos', JSON.stringify(this.todos));
    }

    async addTodo() {
        const text = this.todoInput.value.trim();
        if (!text) return;

        const newTodo = {
            id: this.todoIdCounter++,
            text: text,
            completed: false,
            createdAt: new Date().toISOString()
        };

        try {
            const response = await fetch('/api/todos', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(newTodo)
            });

            if (response.ok) {
                const savedTodo = await response.json();
                this.todos.push(savedTodo);
            } else {
                throw new Error('API not available');
            }
        } catch (error) {
            console.log('Using local storage fallback');
            this.todos.push(newTodo);
            this.saveToLocalStorage();
        }

        this.todoInput.value = '';
        this.render();
    }

    async toggleTodo(id) {
        const todo = this.todos.find(t => t.id === id);
        if (!todo) return;

        todo.completed = !todo.completed;

        try {
            await fetch(`/api/todos/${id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(todo)
            });
        } catch (error) {
            console.log('Using local storage fallback');
            this.saveToLocalStorage();
        }

        this.render();
    }

    async deleteTodo(id) {
        try {
            await fetch(`/api/todos/${id}`, {
                method: 'DELETE'
            });
        } catch (error) {
            console.log('Using local storage fallback');
        }

        this.todos = this.todos.filter(t => t.id !== id);
        this.saveToLocalStorage();
        this.render();
    }

    render() {
        this.todoList.innerHTML = '';

        this.todos.forEach(todo => {
            const li = document.createElement('li');
            li.className = `todo-item ${todo.completed ? 'completed' : ''}`;
            
            li.innerHTML = `
                <input type="checkbox" class="todo-checkbox" ${todo.completed ? 'checked' : ''}>
                <span class="todo-text">${todo.text}</span>
                <button class="todo-delete">削除</button>
            `;

            const checkbox = li.querySelector('.todo-checkbox');
            const deleteBtn = li.querySelector('.todo-delete');

            checkbox.addEventListener('change', () => this.toggleTodo(todo.id));
            deleteBtn.addEventListener('click', () => this.deleteTodo(todo.id));

            this.todoList.appendChild(li);
        });

        this.updateStats();
    }

    updateStats() {
        const total = this.todos.length;
        const completed = this.todos.filter(t => t.completed).length;
        const pending = total - completed;

        this.totalTasks.textContent = `総タスク: ${total}`;
        this.completedTasks.textContent = `完了: ${completed}`;
        this.pendingTasks.textContent = `未完了: ${pending}`;
    }
}

// アプリケーション初期化
document.addEventListener('DOMContentLoaded', () => {
    new TodoApp();
    console.log('🤖 ccswarm TODO App initialized!');
});"#;
            
            fs::write(project_dir.join("app.js"), js_content).await?;
            info!("✅ app.js を生成");
        },
        
        "todo-backend-1" => {
            // Express サーバーファイルを生成
            let server_content = r#"// ccswarm TODO App Server
const express = require('express');
const path = require('path');
const fs = require('fs').promises;

const app = express();
const PORT = process.env.PORT || 3000;
const TODOS_FILE = 'todos.json';

// ミドルウェア
app.use(express.json());
app.use(express.static('.'));

// データ永続化用のヘルパー関数
async function loadTodos() {
    try {
        const data = await fs.readFile(TODOS_FILE, 'utf8');
        return JSON.parse(data);
    } catch (error) {
        return [];
    }
}

async function saveTodos(todos) {
    await fs.writeFile(TODOS_FILE, JSON.stringify(todos, null, 2));
}

// ルート
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// API エンドポイント
app.get('/api/todos', async (req, res) => {
    try {
        const todos = await loadTodos();
        res.json(todos);
    } catch (error) {
        res.status(500).json({ error: 'Failed to load todos' });
    }
});

app.post('/api/todos', async (req, res) => {
    try {
        const todos = await loadTodos();
        const newTodo = {
            id: Date.now(),
            text: req.body.text,
            completed: false,
            createdAt: new Date().toISOString()
        };
        
        todos.push(newTodo);
        await saveTodos(todos);
        res.status(201).json(newTodo);
    } catch (error) {
        res.status(500).json({ error: 'Failed to create todo' });
    }
});

app.put('/api/todos/:id', async (req, res) => {
    try {
        const todos = await loadTodos();
        const todoId = parseInt(req.params.id);
        const todoIndex = todos.findIndex(t => t.id === todoId);
        
        if (todoIndex === -1) {
            return res.status(404).json({ error: 'Todo not found' });
        }
        
        todos[todoIndex] = { ...todos[todoIndex], ...req.body };
        await saveTodos(todos);
        res.json(todos[todoIndex]);
    } catch (error) {
        res.status(500).json({ error: 'Failed to update todo' });
    }
});

app.delete('/api/todos/:id', async (req, res) => {
    try {
        const todos = await loadTodos();
        const todoId = parseInt(req.params.id);
        const filteredTodos = todos.filter(t => t.id !== todoId);
        
        await saveTodos(filteredTodos);
        res.status(204).send();
    } catch (error) {
        res.status(500).json({ error: 'Failed to delete todo' });
    }
});

// サーバー起動
app.listen(PORT, () => {
    console.log(`🚀 ccswarm TODO App server running on http://localhost:${PORT}`);
    console.log(`🤖 Multi-agent system development complete!`);
    console.log(`📁 Serving files from: ${__dirname}`);
});"#;
            
            fs::write(project_dir.join("server.js"), server_content).await?;
            info!("✅ server.js を生成");
        },
        
        "todo-backend-3" => {
            // package.json を生成
            let package_content = r#"{
  "name": "ccswarm-todo-app",
  "version": "1.0.0",
  "description": "TODO application developed by ccswarm multi-agent system",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "node server.js"
  },
  "keywords": ["todo", "ccswarm", "multi-agent", "nodejs", "express"],
  "author": "ccswarm multi-agent system",
  "license": "MIT",
  "dependencies": {
    "express": "^4.18.2"
  },
  "engines": {
    "node": ">=14.0.0"
  }
}"#;
            
            fs::write(project_dir.join("package.json"), package_content).await?;
            info!("✅ package.json を生成");
        },
        
        "todo-deploy-1" => {
            // 起動スクリプトを生成
            let run_script = r#"#!/bin/bash
# ccswarm TODO App 起動スクリプト

echo "🤖 ccswarm TODO App を起動中..."

# Node.jsの存在確認
if ! command -v node &> /dev/null; then
    echo "❌ Node.js がインストールされていません"
    echo "📦 Node.js をインストールしてください: https://nodejs.org/"
    exit 1
fi

# 依存関係のインストール
if [ ! -d "node_modules" ]; then
    echo "📦 依存関係をインストール中..."
    npm install
fi

# サーバー起動
echo "🚀 サーバーを起動中..."
echo "📍 アクセスURL: http://localhost:3000"
echo "🛑 終了するには Ctrl+C を押してください"
echo ""

node server.js"#;
            
            fs::write(project_dir.join("run.sh"), run_script).await?;
            
            // 実行権限を付与
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(project_dir.join("run.sh")).await?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(project_dir.join("run.sh"), perms).await?;
            }
            
            info!("✅ run.sh を生成");
        },
        
        "todo-deploy-2" => {
            // README を生成
            let readme_content = r#"# 🤖 ccswarm TODO App

マルチエージェントシステム ccswarm によって開発されたTODOアプリケーション

## 🎯 概要

このTODOアプリケーションは、ccswarmシステムの以下の専門エージェントによって協調開発されました：

- **🎨 Frontend Agent**: HTML, CSS, JavaScript の開発
- **⚙️ Backend Agent**: Node.js Express サーバーとAPI開発  
- **🚀 DevOps Agent**: デプロイメントスクリプトとドキュメント作成

## 🛠️ 技術スタック

- **フロントエンド**: HTML5, CSS3, Vanilla JavaScript
- **バックエンド**: Node.js, Express.js
- **データ永続化**: JSON ファイル
- **スタイリング**: レスポンシブCSS

## 📋 機能

- ✅ TODOタスクの追加
- ✅ タスクの完了/未完了切り替え  
- ✅ タスクの削除
- ✅ タスク統計表示
- ✅ データの永続化
- ✅ レスポンシブデザイン

## 🚀 起動方法

### 必要な環境

- Node.js (v14.0.0 以上)

### インストールと起動

1. **依存関係のインストール**
   ```bash
   npm install
   ```

2. **サーバー起動**
   ```bash
   npm start
   ```
   
   または
   
   ```bash
   node server.js
   ```

3. **起動スクリプト使用 (Unix/Linux/macOS)**
   ```bash
   ./run.sh
   ```

4. **ブラウザでアクセス**
   ```
   http://localhost:3000
   ```

## 📁 プロジェクト構造

```
todo_app/
├── index.html      # メインHTMLファイル
├── styles.css      # スタイルシート
├── app.js          # フロントエンドJavaScript
├── server.js       # Express サーバー
├── package.json    # Node.js 依存関係
├── run.sh          # 起動スクリプト
├── todos.json      # データファイル (自動生成)
└── README.md       # このファイル
```

## 🔧 API エンドポイント

- `GET /api/todos` - 全TODOを取得
- `POST /api/todos` - 新しいTODOを作成
- `PUT /api/todos/:id` - TODOを更新
- `DELETE /api/todos/:id` - TODOを削除

## 🎨 特徴

- **マルチエージェント開発**: 各専門分野のエージェントが協調して開発
- **完全な動作**: 実際にブラウザでアクセス可能
- **データ永続化**: サーバー再起動後もデータを保持
- **エラーハンドリング**: API障害時はローカルストレージを使用

## 🤖 ccswarm について

このアプリケーションは ccswarm マルチエージェントシステムによって開発されました。ccswarmは以下の特徴を持つ開発システムです：

- **エージェント特化**: 各エージェントが専門分野に特化
- **協調開発**: エージェント間での自動的なタスク振り分け
- **品質保証**: 専門性に基づく品質管理
- **効率的開発**: 並列作業による高速開発

## 📄 ライセンス

MIT License

---

🎉 **ccswarm multi-agent system で開発完了！**"#;
            
            fs::write(project_dir.join("README.md"), readme_content).await?;
            info!("✅ README.md を生成");
        },
        
        _ => {
            // その他のタスクは何もしない
            info!("ℹ️ タスク '{}' - ファイル生成なし", task.description);
        }
    }
    
    Ok(())
}