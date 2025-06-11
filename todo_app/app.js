// ccswarm TODO App JavaScript
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
});