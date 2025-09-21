# ccswarm auto-create

Generate complete applications from natural language descriptions.

## Usage
```bash
ccswarm auto-create <DESCRIPTION> [OPTIONS]
```

## Options
- `-o, --output <PATH>` - Output directory for generated application
- `--template <TEMPLATE>` - Use specific template (todo, blog, ecommerce, custom)
- `--agents <AGENTS>` - Specify which agents to use (comma-separated)

## Description
Creates complete, working applications based on natural language descriptions. The system analyzes your requirements and generates a full-stack application with appropriate technologies.

## Examples

### Create TODO Application
```bash
$ ccswarm auto-create "Create a modern TODO app with user authentication" --output ./todo-app

🚀 Auto-Creating Application
════════════════════════════════════════════

📝 Analyzing requirements...
  ✓ User authentication needed
  ✓ TODO CRUD operations
  ✓ Modern UI required

🤖 Delegating to agents:
  → Frontend: React + Tailwind CSS
  → Backend: Node.js + Express + JWT
  → Database: PostgreSQL

⏳ Generating application structure...
✅ Created: todo-app/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   └── App.jsx
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── routes/
│   │   ├── models/
│   │   └── server.js
│   └── package.json
├── docker-compose.yml
├── README.md
└── .gitignore

📋 Next steps:
1. cd todo-app
2. docker-compose up -d
3. cd frontend && npm install && npm start
4. cd backend && npm install && npm start

🎉 Application created successfully!
```

### Create Blog Platform
```bash
$ ccswarm auto-create "Blog platform with markdown support and comments" --output ./my-blog

🚀 Auto-Creating Application
════════════════════════════════════════════

📝 Analyzing requirements...
  ✓ Blog post management
  ✓ Markdown editor/renderer
  ✓ Comment system
  ✓ SEO optimization

🤖 Agent assignments:
  → Frontend: Next.js + MDX
  → Backend: Strapi CMS
  → DevOps: Docker + Nginx

⏳ Building blog platform...
[Progress indicators...]

✅ Blog platform created with:
- Server-side rendering
- Markdown/MDX support
- Nested comments
- RSS feed
- SEO meta tags
- Admin dashboard
```

### Create E-commerce Site
```bash
$ ccswarm auto-create "Online shop with cart and Stripe payments" --output ./shop

🚀 Auto-Creating Application
════════════════════════════════════════════

📝 Requirements detected:
  ✓ Product catalog
  ✓ Shopping cart
  ✓ Payment processing (Stripe)
  ✓ Order management

🤖 Technology stack selected:
  → Frontend: Vue.js + Vuetify
  → Backend: Python/FastAPI
  → Database: MongoDB
  → Payments: Stripe integration

⏳ Generating e-commerce platform...
[Progress indicators...]

✅ E-commerce site ready with:
- Product catalog with search
- Shopping cart persistence
- Stripe checkout integration
- Order tracking
- Admin panel
- Email notifications
```

### Custom Template
```bash
$ ccswarm auto-create "Real-time chat app" --template custom --output ./chat-app

🚀 Custom Application Creation
════════════════════════════════════════════

📝 Custom requirements analysis...
  ✓ Real-time messaging
  ✓ WebSocket needed
  ✓ User presence tracking

💡 AI recommends:
  → Frontend: React + Socket.io-client
  → Backend: Node.js + Socket.io
  → Database: Redis for sessions

Continue with these technologies? [Y/n]: y

⏳ Building custom application...
✅ Real-time chat created!
```

## Features

### Intelligent Analysis
- Natural language understanding
- Automatic requirement extraction
- Technology stack selection
- Architecture decisions

### Complete Applications
- Frontend with modern UI
- Backend API implementation
- Database schema and migrations
- Docker configuration
- Documentation

### Best Practices
- Security (auth, CORS, validation)
- Error handling
- Environment configuration
- Git setup with .gitignore
- README with setup instructions

## Supported Application Types
1. **TODO Apps** - Task management with auth
2. **Blogs** - CMS with markdown support
3. **E-commerce** - Full shopping experience
4. **Chat Apps** - Real-time messaging
5. **Dashboards** - Data visualization
6. **APIs** - RESTful or GraphQL
7. **Custom** - Any application type

## Technology Stacks
- **Frontend**: React, Vue, Angular, Next.js
- **Backend**: Node.js, Python, Go, Rust
- **Database**: PostgreSQL, MongoDB, MySQL
- **Styling**: Tailwind, Material-UI, Bootstrap
- **Deployment**: Docker, Kubernetes configs

## Related Commands
- `ccswarm task` - Create specific development tasks
- `ccswarm agent list` - View available agents
- `ccswarm status` - Monitor creation progress