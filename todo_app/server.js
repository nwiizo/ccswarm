// ccswarm TODO App Server
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
});