# Agent Agency V3 - Complete System Demo

🎉 **Welcome to Agent Agency V3!** This guide will get your complete AI orchestration platform running with a beautiful web interface.

## ⚠️ CRITICAL: Observational API Design

**The API is designed for OBSERVATION, not manipulation.**

The API acts as a "doctor's MRI machine" - it observes what's happening inside
the orchestrator without directly controlling execution. This preserves research
integrity by ensuring the orchestrator maintains full autonomy over its execution
lifecycle.

### Design Principles

1. **Observation Only**: All endpoints observe orchestrator state, never manipulate it directly
2. **Request-Based Control**: Control operations (pause/resume/cancel) are requests that
   are logged in chain-of-thought, but the orchestrator decides whether to honor them
3. **Research Integrity**: No direct manipulation of execution state - orchestrator maintains
   full control over its own execution lifecycle
4. **Agent Autonomy**: Agents use their own connections to task execution, not through the API

### What This Means

- **Task Submission**: Requests orchestrator to start a task (orchestrator handles execution)
- **State Observation**: Query task status, chain of thought, council decisions, worker actions
- **Control Requests**: Request pause/resume/cancel (orchestrator decides if safe)
- **Never Manipulate**: Never directly change execution state - only observe and request

### Why This Matters

Direct manipulation of orchestrator execution state would compromise research integrity.
By maintaining strict observation boundaries, we ensure that:
- Orchestrator decisions are autonomous and reproducible
- Research results are not contaminated by external manipulation
- The orchestrator's chain of thought accurately reflects its own reasoning
- Agents maintain their own execution connections independently

## 🚀 Quick Start (5 minutes)

### Prerequisites

1. **Rust** (latest stable)
2. **Node.js** (v16+)
3. **PostgreSQL** (v12+)
4. **Python 3** (for web server)

### One-Command Setup

```bash
# Clone and navigate to the project
cd iterations/v3

# Run the complete demo (starts everything automatically)
./start-demo.sh
```

That's it! Your system will be running at:
- 🌐 **Web Dashboard**: http://localhost:3000
- 🔌 **API Server**: http://localhost:8080

---

## 🏗️ System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Interface │◄──►│   API Server    │◄──►│   PostgreSQL    │
│   (Port 3000)   │    │   (Port 8080)   │    │   Database      │
│                 │    │                 │    │                 │
│ • Task Dashboard│    │ • CQRS Commands │    │ • Task History  │
│ • System Health │    │ • REST API      │    │ • Artifacts     │
│ • Worker Status │    │ • Real-time     │    │ • Metrics       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                   ┌─────────────────┐
                   │ Orchestration   │
                   │ Engine (Rust)   │
                   │                 │
                   │ • CQRS Bus      │
                   │ • Task Execution│
                   │ • Worker Mgmt   │
                   └─────────────────┘
```

---

## 📊 What You'll See

### Web Dashboard Features

1. **System Health Dashboard**
   - Total workers, active tasks, completed tasks
   - Real-time system metrics
   - Connection status indicator

2. **Task Management**
   - Create new AI tasks with descriptions
   - Configure risk tiers (1-3) and scope
   - Submit tasks for execution
   - Monitor active task status

3. **Worker Pool Monitoring**
   - View registered AI workers
   - See worker capabilities and status
   - Real-time health updates

4. **Live System Logs**
   - Real-time activity feed
   - Color-coded log levels (info, success, warning, error)
   - Task execution events

### API Endpoints Available

```bash
# System Health
GET  /api/health

# Task Management
GET  /api/tasks/active
POST /api/tasks/{id}/execute
POST /api/tasks/{id}/cancel
GET  /api/tasks/{id}/status

# Worker Management
POST /api/workers/register
POST /api/workers/{id}/health

# Legacy Endpoints
GET  /api/tasks
GET  /api/tasks/{id}
GET  /api/tasks/{id}/events
```

---

## 🔧 Manual Setup (Alternative)

If you prefer to start services individually:

### 1. Start Database

```bash
# macOS
brew services start postgresql

# Linux
sudo systemctl start postgresql

# Create database
createdb agent_agency
```

### 2. Setup Database Schema

```bash
# Run migrations
export DATABASE_URL="postgresql://postgres:password@localhost/agent_agency"
node scripts/setup/setup-database-v3.cjs
```

### 3. Start API Server

```bash
# In terminal 1
export DATABASE_URL="postgresql://postgres:password@localhost/agent_agency"
cargo run --features api-server -- serve --port 8080 --database-url "$DATABASE_URL"
```

### 4. Start Web Interface

```bash
# In terminal 2
cd web-app
npm run dev
# or
python3 server.py 3000
```

---

## 🎮 Using the System

### Creating Your First AI Task

1. **Open the Web Dashboard** at http://localhost:3000
2. **Fill out the task form**:
   - **Description**: "Analyze the code quality in src/ directory"
   - **Risk Tier**: 2 (Standard)
   - **Scope**: "src/, tests/"
3. **Click "Execute Task"**
4. **Watch the live logs** as the system processes your task

### Monitoring System Health

- **Dashboard Cards** update every 5 seconds
- **System Status** shows connection health
- **Active Tasks** displays current work
- **Worker Pool** shows available AI agents

### Understanding Risk Tiers

| Tier | Use Case | Coverage | Description |
|------|----------|----------|-------------|
| **1** | Critical | 90%+ | Auth, billing, data integrity |
| **2** | Standard | 80%+ | API changes, features |
| **3** | Low Risk | 70%+ | UI, internal tools |

---

## 🔍 Testing the API

### Quick API Test

```bash
cd web-app
npm run test:api
```

### Manual API Testing

```bash
# Check system health
curl http://localhost:8080/api/health

# Get active tasks
curl http://localhost:8080/api/tasks/active

# Create a task (replace with actual task data)
curl -X POST http://localhost:8080/api/tasks/123e4567-e89b-12d3-a456-426614174000/execute \
  -H "Content-Type: application/json" \
  -d '{
    "task_descriptor": {
      "task_id": "123e4567-e89b-12d3-a456-426614174000",
      "description": "Test task",
      "risk_tier": 2,
      "scope_in": ["src/"],
      "scope_out": ["target/"],
      "acceptance": ["Task completed"],
      "metadata": {}
    },
    "worker_id": "worker-1",
    "requested_at": "2024-01-01T00:00:00Z"
  }'
```

---

## 🛠️ Development & Customization

### Web Interface Structure

```
web-app/
├── index.html      # Main application (single-page)
├── package.json    # NPM scripts and config
├── server.py       # Python HTTP server with CORS
├── test-api.js     # API connectivity tests
└── README.md       # Web-specific documentation
```

### Adding New Features

1. **API Endpoints**: Add to `orchestration/src/api.rs` and `cqrs_router.rs`
2. **UI Components**: Modify `index.html` JavaScript functions
3. **Styling**: Uses Tailwind CSS classes (CDN loaded)

### Customization Examples

```javascript
// Add new API endpoint
async function customApiCall() {
    const response = await fetch(`${API_BASE}/custom/endpoint`);
    return await response.json();
}

// Add new UI component
function addCustomWidget(data) {
    const widget = document.createElement('div');
    widget.className = 'bg-gray-800 rounded-lg p-4';
    widget.innerHTML = `<h3>${data.title}</h3><p>${data.value}</p>`;
    document.getElementById('dashboard').appendChild(widget);
}
```

---

## 📈 System Capabilities

### AI Task Types Supported

- **Code Analysis**: Quality assessment, security audits
- **Refactoring**: Automated code improvements
- **Testing**: Generate and run test suites
- **Documentation**: API docs, code comments
- **Optimization**: Performance improvements
- **Security**: Vulnerability scanning
- **Integration**: API and system integration

### Quality Gates

- **Tier 1**: 90%+ test coverage, mutation score 70%+
- **Tier 2**: 80%+ test coverage, mutation score 50%+
- **Tier 3**: 70%+ test coverage, mutation score 30%+

### Real-time Monitoring

- **Performance Metrics**: Response times, throughput
- **System Health**: CPU, memory, disk usage
- **Task Progress**: Execution status, completion %
- **Error Tracking**: Failed tasks, retry attempts

---

## 🚨 Troubleshooting

### Common Issues

**"Connection Failed" in Web Interface**
```bash
# Check if API server is running
curl http://localhost:8080/api/health

# Restart API server
cargo run --features api-server -- serve --port 8080 --database-url "$DATABASE_URL"
```

**"Database connection failed"**
```bash
# Check PostgreSQL status
pg_isready -h localhost -p 5432

# Verify DATABASE_URL
echo $DATABASE_URL

# Restart database
brew services restart postgresql  # macOS
sudo systemctl restart postgresql # Linux
```

**Tasks not executing**
```bash
# Check worker registration
curl http://localhost:8080/api/workers

# Verify task submission
curl http://localhost:8080/api/tasks/active
```

**Web interface not loading**
```bash
# Check web server
curl http://localhost:3000

# Restart web server
cd web-app && python3 server.py 3000
```

---

## 📚 Architecture Deep Dive

### CQRS Pattern Implementation

```
┌─────────────┐    ┌─────────────┐
│  Commands   │    │   Queries   │
│             │    │             │
│ • ExecuteTask│    │ • GetHealth │
│ • CancelTask │    │ • GetTasks  │
│ • UpdateTask │    │ • GetStatus │
└──────┬──────┘    └──────┬──────┘
       │                  │
       └────────┬─────────┘
                │
        ┌───────▼───────┐
        │   CQRS Bus    │
        │               │
        │ • Command Bus │
        │ • Query Bus   │
        │ • Handlers    │
        └───────┬───────┘
                │
        ┌───────▼───────┐
        │   Handlers    │
        │               │
        │ • Validation  │
        │ • Business    │
        │ • Persistence │
        └───────────────┘
```

### Database Schema

```sql
-- Core tables
CREATE TABLE tasks (...);
CREATE TABLE workers (...);
CREATE TABLE task_executions (...);
CREATE TABLE execution_artifacts (...);
CREATE TABLE artifact_metadata (...);
CREATE TABLE task_audit_logs (...);
```

### API Response Formats

```typescript
// System Health
interface SystemHealth {
  total_workers: number;
  active_workers: number;
  healthy_workers: number;
  total_tasks: number;
  active_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  average_task_duration_ms: number;
  uptime_seconds: number;
}

// Task Status
interface TaskStatus {
  task_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  progress_percentage: number;
  started_at?: string;
  completed_at?: string;
  error_message?: string;
}
```

---

## 🎯 Next Steps

### Immediate Possibilities

1. **Add Real Workers**: Connect actual AI agents to execute tasks
2. **Enhanced UI**: Add charts, graphs, and advanced monitoring
3. **User Authentication**: Add login and user management
4. **Task Templates**: Pre-built task configurations
5. **WebSocket Updates**: Real-time task progress updates

### Advanced Features

1. **Multi-tenant Support**: Multiple organizations/users
2. **Task Scheduling**: Cron-like task scheduling
3. **Workflow Orchestration**: Complex task dependencies
4. **Performance Analytics**: Detailed execution metrics
5. **Integration APIs**: Third-party service integrations

---

## 🤝 Contributing

### Development Workflow

1. **Fork** the repository
2. **Create** a feature branch
3. **Test** your changes thoroughly
4. **Submit** a pull request
5. **Review** and iterate

### Code Quality

- **Rust**: `cargo clippy` and `cargo fmt`
- **JavaScript**: ESLint and Prettier
- **Tests**: 80%+ coverage requirement
- **Documentation**: Update READMEs for changes

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🆘 Support

- **Issues**: GitHub Issues for bugs and feature requests
- **Discussions**: GitHub Discussions for questions
- **Documentation**: See `docs/` directory for detailed guides

---

**🎉 Happy orchestrating! Your AI development platform is now ready to revolutionize how you build software.**
