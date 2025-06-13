# ccswarm auto-create

Generate complete applications from natural language descriptions using AI agents.

## Description

The `auto-create` command leverages multiple AI agents to generate full-stack applications from simple descriptions. It analyzes requirements, creates project structure, implements features, and delivers a working application with documentation.

## Usage

```bash
ccswarm auto-create <DESCRIPTION> [OPTIONS]
```

## Options

- `--output <PATH>` - Output directory for the application (required)
- `--template <TYPE>` - Use specific template (todo, blog, ecommerce, chat, custom)
- `--stack <STACK>` - Technology stack (react-node, vue-django, angular-rails)
- `--features <LIST>` - Comma-separated list of features to include
- `--no-docker` - Skip Docker configuration
- `--no-tests` - Skip test generation
- `--no-docs` - Skip documentation generation
- `--agents <LIST>` - Specific agents to use
- `--review` - Enable quality review after generation
- `--deploy-ready` - Include deployment configurations

## Examples

### Basic TODO application
```bash
ccswarm auto-create "Create a TODO app with user authentication" --output ./todo-app
```

### Blog with specific features
```bash
ccswarm auto-create "Blog platform with markdown support, comments, and RSS feed" \
  --output ./my-blog \
  --features "auth,comments,rss,search" \
  --template blog
```

### E-commerce platform
```bash
ccswarm auto-create "E-commerce site with shopping cart and payment integration" \
  --output ./shop \
  --template ecommerce \
  --deploy-ready
```

### Custom application
```bash
ccswarm auto-create "Real-time collaborative drawing app with rooms" \
  --output ./draw-together \
  --stack react-node \
  --features "websockets,canvas,rooms,chat"
```

## Available Templates

### TODO Template
```bash
ccswarm auto-create "Task management app" --template todo --output ./tasks
```
Includes:
- CRUD operations
- User authentication
- Task categories
- Due dates
- Priority levels

### Blog Template
```bash
ccswarm auto-create "Personal blog" --template blog --output ./blog
```
Includes:
- Post management
- Markdown editor
- Comments system
- Categories/tags
- RSS feed

### E-commerce Template
```bash
ccswarm auto-create "Online store" --template ecommerce --output ./store
```
Includes:
- Product catalog
- Shopping cart
- User accounts
- Order management
- Payment integration

### Chat Template
```bash
ccswarm auto-create "Chat application" --template chat --output ./chat
```
Includes:
- Real-time messaging
- Multiple rooms
- User presence
- Message history
- File sharing

## Generated Structure

### Standard Output
```
my-app/
├── frontend/               # Frontend application
│   ├── src/               # Source code
│   ├── public/            # Static assets
│   ├── package.json       # Dependencies
│   └── README.md          # Frontend docs
├── backend/               # Backend API
│   ├── src/               # Source code
│   ├── tests/             # Test files
│   ├── package.json       # Dependencies
│   └── README.md          # Backend docs
├── docker-compose.yml     # Multi-container setup
├── Dockerfile.frontend    # Frontend container
├── Dockerfile.backend     # Backend container
├── .env.example          # Environment template
├── README.md             # Project documentation
└── docs/                 # Additional documentation
    ├── API.md           # API documentation
    ├── SETUP.md         # Setup instructions
    └── DEPLOYMENT.md    # Deployment guide
```

## Technology Stacks

### React + Node.js (Default)
```bash
ccswarm auto-create "App description" --stack react-node
```
- Frontend: React, TypeScript, Tailwind CSS
- Backend: Express.js, TypeScript, PostgreSQL
- Testing: Jest, React Testing Library

### Vue + Django
```bash
ccswarm auto-create "App description" --stack vue-django
```
- Frontend: Vue 3, TypeScript, Vuetify
- Backend: Django REST Framework, Python
- Testing: Pytest, Vue Test Utils

### Angular + Rails
```bash
ccswarm auto-create "App description" --stack angular-rails
```
- Frontend: Angular, TypeScript, Angular Material
- Backend: Ruby on Rails API
- Testing: RSpec, Karma

### Custom Stack
```bash
ccswarm auto-create "App description" \
  --stack custom \
  --features "nextjs,fastapi,mongodb"
```

## Feature Flags

### Authentication
```bash
--features "auth"
```
Adds:
- User registration/login
- JWT tokens
- Password reset
- Session management

### Real-time Features
```bash
--features "realtime,websockets"
```
Adds:
- WebSocket server
- Live updates
- Presence indicators
- Real-time notifications

### Search
```bash
--features "search,elasticsearch"
```
Adds:
- Full-text search
- Search filters
- Search suggestions
- Indexing system

### File Upload
```bash
--features "upload,s3"
```
Adds:
- File upload endpoints
- S3 integration
- Image processing
- File management

## Deployment Options

### Include deployment configs
```bash
ccswarm auto-create "App" --output ./app --deploy-ready
```

Generates:
- Kubernetes manifests
- GitHub Actions workflows
- Terraform configurations
- Environment setup scripts

### Platform-specific
```bash
# For Vercel/Netlify
ccswarm auto-create "App" --deploy-target vercel

# For AWS
ccswarm auto-create "App" --deploy-target aws

# For Google Cloud
ccswarm auto-create "App" --deploy-target gcp
```

## Agent Orchestration

### How it works
1. **Master Claude** analyzes requirements
2. **Frontend Agent** creates UI components
3. **Backend Agent** implements API and database
4. **DevOps Agent** sets up infrastructure
5. **QA Agent** generates tests

### Agent assignment
```bash
# Use specific agents
ccswarm auto-create "App" \
  --agents "frontend-expert,backend-specialist,devops-pro"

# Let Master Claude decide (default)
ccswarm auto-create "App" --output ./app
```

## Quality Assurance

### Enable review
```bash
ccswarm auto-create "App" --output ./app --review
```

Reviews:
- Code quality
- Test coverage
- Security vulnerabilities
- Performance issues
- Documentation completeness

### Quality standards
```bash
ccswarm auto-create "App" \
  --output ./app \
  --review \
  --min-coverage 85 \
  --max-complexity 10
```

## Advanced Usage

### Multi-stage generation
```bash
# Stage 1: Core functionality
ccswarm auto-create "Basic blog" --output ./blog --stage core

# Stage 2: Add features
ccswarm auto-create "Add comments and search" \
  --output ./blog \
  --stage enhance \
  --continue

# Stage 3: Optimization
ccswarm auto-create "Optimize performance and add caching" \
  --output ./blog \
  --stage optimize \
  --continue
```

### Integration with existing code
```bash
ccswarm auto-create "Add authentication to existing app" \
  --output ./my-app \
  --integrate \
  --preserve "src/core,src/utils"
```

## Post-Generation

### Start the application
```bash
cd ./my-app
docker-compose up

# Or without Docker
cd frontend && npm install && npm start
cd backend && npm install && npm start
```

### Run tests
```bash
cd ./my-app
npm test          # Frontend tests
cd backend
npm test          # Backend tests
```

### Deploy
```bash
cd ./my-app
./scripts/deploy.sh production
```

## Troubleshooting

### Generation fails
```bash
# Check agent status
ccswarm status --agents

# Retry with verbose output
ccswarm auto-create "App" --output ./app --verbose

# Use specific template
ccswarm auto-create "App" --output ./app --template todo
```

### Incomplete generation
```bash
# Continue from where it stopped
ccswarm auto-create --continue --output ./app

# Regenerate specific parts
ccswarm auto-create --regenerate frontend --output ./app
```

## Related Commands

- [`init`](init.md) - Initialize ccswarm project
- [`task`](task.md) - Add specific development tasks
- [`agents`](agents.md) - Manage agents for generation
- [`review`](review.md) - Review generated code

## Notes

- Generation time varies by complexity (5-30 minutes)
- All generated code includes comments and documentation
- Applications are production-ready with proper error handling
- Includes security best practices by default
- Can be customized after generation