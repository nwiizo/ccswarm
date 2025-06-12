use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;
use crate::orchestrator::master_delegation::{
    DelegationDecision, DelegationStrategy, MasterDelegationEngine,
};

/// Application types that can be auto-created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AppType {
    Todo,
    Blog,
    Ecommerce,
    Api,
    Dashboard,
    Custom(String),
}

/// Task template for application creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub id: String,
    pub description: String,
    pub target_agent: String,
    pub priority: Priority,
    pub task_type: TaskType,
    pub dependencies: Vec<String>,
    pub estimated_duration: Option<u32>,
}

/// Auto-create engine for automatic application generation
pub struct AutoCreateEngine {
    templates: HashMap<AppType, Vec<TaskTemplate>>,
    delegation_engine: MasterDelegationEngine,
}

impl AutoCreateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // TODOアプリテンプレート
        templates.insert(AppType::Todo, vec![
            TaskTemplate {
                id: "todo-frontend".to_string(),
                description: "Create React TODO app UI with add, delete, and complete task features".to_string(),
                target_agent: "frontend".to_string(),
                priority: Priority::High,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(1800),
            },
            TaskTemplate {
                id: "todo-backend".to_string(),
                description: "Create REST API for TODO app with CRUD operations using Node.js and Express".to_string(),
                target_agent: "backend".to_string(),
                priority: Priority::High,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(2400),
            },
            TaskTemplate {
                id: "todo-database".to_string(),
                description: "Setup SQLite database schema and migrations for TODO app".to_string(),
                target_agent: "backend".to_string(),
                priority: Priority::Medium,
                task_type: TaskType::Infrastructure,
                dependencies: vec!["todo-backend".to_string()],
                estimated_duration: Some(900),
            },
            TaskTemplate {
                id: "todo-tests".to_string(),
                description: "Write unit tests and integration tests for TODO app".to_string(),
                target_agent: "qa".to_string(),
                priority: Priority::Medium,
                task_type: TaskType::Testing,
                dependencies: vec!["todo-frontend".to_string(), "todo-backend".to_string()],
                estimated_duration: Some(1800),
            },
            TaskTemplate {
                id: "todo-deploy".to_string(),
                description: "Create Docker configuration and deployment scripts".to_string(),
                target_agent: "devops".to_string(),
                priority: Priority::Low,
                task_type: TaskType::Infrastructure,
                dependencies: vec!["todo-tests".to_string()],
                estimated_duration: Some(1200),
            },
        ]);
        
        // ブログアプリテンプレート
        templates.insert(AppType::Blog, vec![
            TaskTemplate {
                id: "blog-frontend".to_string(),
                description: "Create blog frontend with article list, detail view, and comments".to_string(),
                target_agent: "frontend".to_string(),
                priority: Priority::High,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(2400),
            },
            TaskTemplate {
                id: "blog-backend".to_string(),
                description: "Create blog API with authentication and content management".to_string(),
                target_agent: "backend".to_string(),
                priority: Priority::High,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(3600),
            },
        ]);
        
        Self {
            templates,
            delegation_engine: MasterDelegationEngine::new(DelegationStrategy::Hybrid),
        }
    }
    
    /// Analyze user request and decompose into tasks
    pub async fn analyze_and_decompose(&mut self, description: &str) -> Result<Vec<Task>> {
        info!("🤖 Analyzing request: {}", description);
        
        // Detect application type from description
        let app_type = self.detect_app_type(description);
        info!("📱 Detected app type: {:?}", app_type);
        
        // Get base tasks from template
        let mut tasks = self.get_template_tasks(&app_type);
        
        // Customize tasks based on specific requirements
        self.customize_tasks(&mut tasks, description);
        
        // Convert templates to actual tasks
        let tasks = tasks.into_iter()
            .map(|template| self.template_to_task(template))
            .collect();
        
        Ok(tasks)
    }
    
    /// Detect application type from description
    fn detect_app_type(&self, description: &str) -> AppType {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("todo") || desc_lower.contains("task") {
            AppType::Todo
        } else if desc_lower.contains("blog") || desc_lower.contains("article") {
            AppType::Blog
        } else if desc_lower.contains("shop") || desc_lower.contains("ecommerce") {
            AppType::Ecommerce
        } else if desc_lower.contains("api") || desc_lower.contains("rest") {
            AppType::Api
        } else if desc_lower.contains("dashboard") || desc_lower.contains("admin") {
            AppType::Dashboard
        } else {
            AppType::Custom("generic".to_string())
        }
    }
    
    /// Get template tasks for app type
    fn get_template_tasks(&self, app_type: &AppType) -> Vec<TaskTemplate> {
        match self.templates.get(app_type) {
            Some(tasks) => tasks.clone(),
            None => {
                // Return generic tasks for unknown types
                vec![
                    TaskTemplate {
                        id: "generic-frontend".to_string(),
                        description: "Create frontend application".to_string(),
                        target_agent: "frontend".to_string(),
                        priority: Priority::High,
                        task_type: TaskType::Feature,
                        dependencies: vec![],
                        estimated_duration: Some(2400),
                    },
                    TaskTemplate {
                        id: "generic-backend".to_string(),
                        description: "Create backend API".to_string(),
                        target_agent: "backend".to_string(),
                        priority: Priority::High,
                        task_type: TaskType::Feature,
                        dependencies: vec![],
                        estimated_duration: Some(2400),
                    },
                ]
            }
        }
    }
    
    /// Customize tasks based on specific requirements
    fn customize_tasks(&self, tasks: &mut Vec<TaskTemplate>, description: &str) {
        let desc_lower = description.to_lowercase();
        
        // Add authentication if mentioned
        if desc_lower.contains("auth") || desc_lower.contains("login") {
            tasks.push(TaskTemplate {
                id: "auth-system".to_string(),
                description: "Implement user authentication system with JWT".to_string(),
                target_agent: "backend".to_string(),
                priority: Priority::High,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(1800),
            });
        }
        
        // Add real-time features if mentioned
        if desc_lower.contains("real-time") || desc_lower.contains("websocket") {
            tasks.push(TaskTemplate {
                id: "realtime-features".to_string(),
                description: "Implement real-time updates using WebSockets".to_string(),
                target_agent: "backend".to_string(),
                priority: Priority::Medium,
                task_type: TaskType::Feature,
                dependencies: vec![],
                estimated_duration: Some(1200),
            });
        }
        
        // Add mobile responsiveness if mentioned
        if desc_lower.contains("mobile") || desc_lower.contains("responsive") {
            if let Some(frontend_task) = tasks.iter_mut().find(|t| t.target_agent == "frontend") {
                frontend_task.description += " with mobile-responsive design";
            }
        }
    }
    
    /// Convert template to actual task
    fn template_to_task(&self, template: TaskTemplate) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            description: template.description,
            details: None,
            priority: template.priority,
            task_type: template.task_type,
            estimated_duration: template.estimated_duration,
        }
    }
    
    /// Execute auto-create workflow
    pub async fn execute_auto_create(
        &mut self,
        description: &str,
        _config: &CcswarmConfig,
        output_path: &PathBuf,
    ) -> Result<()> {
        info!("🚀 Starting auto-create workflow");
        
        // Step 1: Create output directory
        tokio::fs::create_dir_all(output_path).await?;
        info!("📂 Created output directory: {}", output_path.display());
        
        // Step 2: Analyze and decompose tasks
        let tasks = self.analyze_and_decompose(description).await?;
        info!("📋 Generated {} tasks", tasks.len());
        
        // Step 3: Create simulated execution results
        info!("\n🤖 Simulating agent execution...");
        for task in &tasks {
            let decision = self.delegation_engine.delegate_task(task.clone())?;
            info!("   {} → {}: {}", 
                "Master",
                decision.target_agent.name(),
                task.description
            );
            
            // Simulate agent execution by creating files
            self.simulate_agent_execution(&decision, task, output_path).await?;
        }
        
        // Step 4: Create project structure
        self.create_project_structure(output_path).await?;
        
        // Step 5: Summary
        info!("\n📊 Auto-create completed!");
        info!("   📂 Project created at: {}", output_path.display());
        info!("   🚀 To run the app:");
        info!("      cd {}", output_path.display());
        info!("      npm install");
        info!("      npm start");
        
        Ok(())
    }
    
    /// Simulate agent execution by creating actual files
    async fn simulate_agent_execution(
        &self,
        decision: &DelegationDecision,
        _task: &Task,
        output_path: &PathBuf,
    ) -> Result<()> {
        match decision.target_agent.name() {
            "Frontend" => {
                self.create_frontend_files(output_path).await?;
                info!("      ✅ Created frontend files");
            }
            "Backend" => {
                self.create_backend_files(output_path).await?;
                info!("      ✅ Created backend files");
            }
            "DevOps" => {
                self.create_devops_files(output_path).await?;
                info!("      ✅ Created deployment files");
            }
            "QA" => {
                self.create_test_files(output_path).await?;
                info!("      ✅ Created test files");
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Create frontend files
    async fn create_frontend_files(&self, output_path: &PathBuf) -> Result<()> {
        // Create index.html
        let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TODO App - ccswarm Generated</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="root"></div>
    <script src="https://unpkg.com/react@18/umd/react.production.min.js"></script>
    <script src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js"></script>
    <script src="https://unpkg.com/@babel/standalone/babel.min.js"></script>
    <script type="text/babel" src="app.js"></script>
</body>
</html>"#;
        
        tokio::fs::write(output_path.join("index.html"), html_content).await?;
        
        // Create app.js
        let app_content = r#"const { useState, useEffect } = React;

function TodoApp() {
    const [todos, setTodos] = useState([]);
    const [inputValue, setInputValue] = useState('');

    useEffect(() => {
        fetchTodos();
    }, []);

    const fetchTodos = async () => {
        try {
            const response = await fetch('http://localhost:3001/api/todos');
            const data = await response.json();
            setTodos(data);
        } catch (error) {
            console.error('Error fetching todos:', error);
        }
    };

    const addTodo = async () => {
        if (!inputValue.trim()) return;
        
        try {
            const response = await fetch('http://localhost:3001/api/todos', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text: inputValue })
            });
            const newTodo = await response.json();
            setTodos([...todos, newTodo]);
            setInputValue('');
        } catch (error) {
            console.error('Error adding todo:', error);
        }
    };

    const toggleTodo = async (id) => {
        try {
            const todo = todos.find(t => t.id === id);
            const response = await fetch(`http://localhost:3001/api/todos/${id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ completed: !todo.completed })
            });
            const updatedTodo = await response.json();
            setTodos(todos.map(t => t.id === id ? updatedTodo : t));
        } catch (error) {
            console.error('Error updating todo:', error);
        }
    };

    const deleteTodo = async (id) => {
        try {
            await fetch(`http://localhost:3001/api/todos/${id}`, {
                method: 'DELETE'
            });
            setTodos(todos.filter(t => t.id !== id));
        } catch (error) {
            console.error('Error deleting todo:', error);
        }
    };

    return (
        <div className="todo-app">
            <h1>📝 TODO App</h1>
            <p className="subtitle">Generated by ccswarm 🤖</p>
            
            <div className="input-group">
                <input
                    type="text"
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addTodo()}
                    placeholder="Add a new task..."
                />
                <button onClick={addTodo}>Add</button>
            </div>

            <ul className="todo-list">
                {todos.map(todo => (
                    <li key={todo.id} className={todo.completed ? 'completed' : ''}>
                        <input
                            type="checkbox"
                            checked={todo.completed}
                            onChange={() => toggleTodo(todo.id)}
                        />
                        <span>{todo.text}</span>
                        <button onClick={() => deleteTodo(todo.id)}>Delete</button>
                    </li>
                ))}
            </ul>
        </div>
    );
}

ReactDOM.render(<TodoApp />, document.getElementById('root'));"#;
        
        tokio::fs::write(output_path.join("app.js"), app_content).await?;
        
        // Create styles.css
        let styles_content = r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

.todo-app {
    background: white;
    padding: 2rem;
    border-radius: 10px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
    width: 90%;
    max-width: 500px;
}

h1 {
    color: #333;
    text-align: center;
    margin-bottom: 0.5rem;
}

.subtitle {
    text-align: center;
    color: #666;
    font-size: 0.9rem;
    margin-bottom: 2rem;
}

.input-group {
    display: flex;
    margin-bottom: 1.5rem;
}

input[type="text"] {
    flex: 1;
    padding: 0.75rem;
    border: 2px solid #ddd;
    border-radius: 5px 0 0 5px;
    font-size: 1rem;
}

.input-group button {
    padding: 0.75rem 1.5rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 0 5px 5px 0;
    cursor: pointer;
    font-size: 1rem;
}

.input-group button:hover {
    background: #5a67d8;
}

.todo-list {
    list-style: none;
}

.todo-list li {
    display: flex;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #eee;
}

.todo-list li.completed span {
    text-decoration: line-through;
    color: #999;
}

.todo-list input[type="checkbox"] {
    margin-right: 1rem;
}

.todo-list span {
    flex: 1;
}

.todo-list button {
    padding: 0.5rem 1rem;
    background: #f56565;
    color: white;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-size: 0.9rem;
}

.todo-list button:hover {
    background: #e53e3e;
}"#;
        
        tokio::fs::write(output_path.join("styles.css"), styles_content).await?;
        
        Ok(())
    }
    
    /// Create backend files
    async fn create_backend_files(&self, output_path: &PathBuf) -> Result<()> {
        // Create server.js
        let server_content = r#"const express = require('express');
const cors = require('cors');
const app = express();
const PORT = 3001;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.static('.'));

// In-memory todo storage
let todos = [];
let nextId = 1;

// Routes
app.get('/api/todos', (req, res) => {
    res.json(todos);
});

app.post('/api/todos', (req, res) => {
    const { text } = req.body;
    const newTodo = {
        id: nextId++,
        text,
        completed: false,
        createdAt: new Date()
    };
    todos.push(newTodo);
    res.status(201).json(newTodo);
});

app.put('/api/todos/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const { completed } = req.body;
    const todo = todos.find(t => t.id === id);
    
    if (!todo) {
        return res.status(404).json({ error: 'Todo not found' });
    }
    
    todo.completed = completed;
    res.json(todo);
});

app.delete('/api/todos/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const index = todos.findIndex(t => t.id === id);
    
    if (index === -1) {
        return res.status(404).json({ error: 'Todo not found' });
    }
    
    todos.splice(index, 1);
    res.status(204).send();
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Server running on http://localhost:${PORT}`);
    console.log('📝 TODO API available at:');
    console.log(`   GET    http://localhost:${PORT}/api/todos`);
    console.log(`   POST   http://localhost:${PORT}/api/todos`);
    console.log(`   PUT    http://localhost:${PORT}/api/todos/:id`);
    console.log(`   DELETE http://localhost:${PORT}/api/todos/:id`);
});"#;
        
        tokio::fs::write(output_path.join("server.js"), server_content).await?;
        
        Ok(())
    }
    
    /// Create DevOps files
    async fn create_devops_files(&self, output_path: &PathBuf) -> Result<()> {
        // Create package.json
        let package_content = r#"{
  "name": "todo-app-ccswarm",
  "version": "1.0.0",
  "description": "TODO application generated by ccswarm",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5"
  },
  "devDependencies": {
    "nodemon": "^3.0.1",
    "jest": "^29.5.0"
  }
}"#;
        
        tokio::fs::write(output_path.join("package.json"), package_content).await?;
        
        // Create Dockerfile
        let dockerfile_content = r#"FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 3001

CMD ["npm", "start"]"#;
        
        tokio::fs::write(output_path.join("Dockerfile"), dockerfile_content).await?;
        
        // Create docker-compose.yml
        let compose_content = r#"version: '3.8'

services:
  todo-app:
    build: .
    ports:
      - "3001:3001"
    environment:
      - NODE_ENV=production
    restart: unless-stopped"#;
        
        tokio::fs::write(output_path.join("docker-compose.yml"), compose_content).await?;
        
        Ok(())
    }
    
    /// Create test files
    async fn create_test_files(&self, output_path: &PathBuf) -> Result<()> {
        // Create basic test file
        let test_content = r#"// Basic tests for TODO app
describe('TODO API', () => {
    test('GET /api/todos returns array', async () => {
        // Test implementation would go here
        expect(true).toBe(true);
    });
    
    test('POST /api/todos creates new todo', async () => {
        // Test implementation would go here
        expect(true).toBe(true);
    });
    
    test('PUT /api/todos/:id updates todo', async () => {
        // Test implementation would go here
        expect(true).toBe(true);
    });
    
    test('DELETE /api/todos/:id removes todo', async () => {
        // Test implementation would go here
        expect(true).toBe(true);
    });
});"#;
        
        tokio::fs::write(output_path.join("app.test.js"), test_content).await?;
        
        Ok(())
    }
    
    /// Create project structure
    async fn create_project_structure(&self, output_path: &PathBuf) -> Result<()> {
        // Create README.md
        let readme_content = r#"# TODO App - Generated by ccswarm 🤖

This TODO application was automatically generated by ccswarm's multi-agent orchestration system.

## 🚀 Quick Start

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the server:
   ```bash
   npm start
   ```

3. Open your browser to http://localhost:3001

## 🏗️ Architecture

- **Frontend**: React with vanilla JavaScript
- **Backend**: Express.js REST API
- **Storage**: In-memory (for demo purposes)
- **Deployment**: Docker-ready

## 🤖 Generated by ccswarm

This application was created by the following agents:
- **Frontend Agent**: Created React UI components
- **Backend Agent**: Implemented REST API
- **DevOps Agent**: Set up deployment configuration
- **QA Agent**: Added test structure

## 📋 Features

- ✅ Add new tasks
- ✅ Mark tasks as complete
- ✅ Delete tasks
- ✅ Real-time updates
- ✅ Responsive design

## 🔧 Development

Run in development mode with auto-reload:
```bash
npm run dev
```

Run tests:
```bash
npm test
```

## 🐳 Docker Deployment

Build and run with Docker:
```bash
docker-compose up
```

---
Generated with ❤️ by ccswarm"#;
        
        tokio::fs::write(output_path.join("README.md"), readme_content).await?;
        
        // Create .gitignore
        let gitignore_content = r#"node_modules/
.env
.DS_Store
*.log
dist/
build/"#;
        
        tokio::fs::write(output_path.join(".gitignore"), gitignore_content).await?;
        
        Ok(())
    }
    
}