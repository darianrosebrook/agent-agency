# Agent Agency V3 System

A unified, production-ready system for AI agent orchestration, task management, and real-time monitoring.

## 🚀 Quick Start

### Prerequisites

- **PostgreSQL** (running on localhost:5432)
- **Redis** (running on localhost:6379)
- **Rust** (latest stable)
- **Node.js** (v20+)
- **Environment Variables**:
  ```bash
  export DATABASE_PASSWORD="agent_agency_secure_password_123"
  ```

### Single Command Startup

```bash
# Start all V3 services
./scripts/v3/start-v3-system.sh start

# Check status
./scripts/v3/start-v3-system.sh status

# View logs
./scripts/v3/start-v3-system.sh logs

# Stop all services
./scripts/v3/start-v3-system.sh stop
```

## 🏗️ System Architecture

### Core Services

| Service | Port | Description | Binary |
|---------|------|-------------|---------|
| **API Server** | 8080 | Main REST API, WebSocket, Health monitoring | `data-infrastructure` |
| **Worker System** | 8081 | Task execution, MCP workers | `agent-workers` |
| **Web Dashboard** | 3000 | NextJS React interface | `npm run dev` |

### External Dependencies

| Service | Port | Status | Management |
|---------|------|--------|------------|
| **PostgreSQL** | 5432 | ✅ Running | `brew services start postgresql@14` |
| **Redis** | 6379 | ✅ Running | `brew services start redis` |

## 📁 Project Structure

```
iterations/v3/
├── data-infrastructure/          # Main API server
│   ├── src/main.rs              # HTTP server, WebSocket, task management
│   └── migrations/              # Database schema
├── agent-workers/               # Worker system
│   └── src/main.rs              # Task execution engine
├── apps/web-dashboard/          # NextJS React interface
│   ├── package.json             # V3-specific scripts
│   └── src/lib/v3-api-client.ts # API client
├── system-*/                    # Supporting crates
└── scripts/v3/                  # Management scripts
    ├── start-v3-system.sh       # Unified startup/shutdown
    ├── update-nextjs-v3.sh      # Dashboard updates
    └── setup/setup-database-v3.js # Database initialization
```

## 🔧 Service Details

### API Server (`data-infrastructure`)

**Features:**
- REST API for task management
- WebSocket chat system
- Real-time metrics streaming
- Health monitoring
- Alert management
- RTO/RPO monitoring
- Database persistence
- Redis caching

**Endpoints:**
- `GET /health` - Health check
- `POST /api/v1/tasks` - Submit tasks
- `GET /api/v1/tasks` - List tasks
- `GET /api/v1/tasks/:id` - Get task details
- `POST /api/v1/tasks/:id/pause` - Pause task
- `POST /api/v1/tasks/:id/resume` - Resume task
- `POST /api/v1/tasks/:id/cancel` - Cancel task
- `GET /api/v1/metrics` - Get metrics
- `GET /api/v1/metrics/stream` - Stream metrics (SSE)
- `POST /api/v1/chat/session` - Create chat session
- `WS /api/v1/chat/ws/:session_id` - WebSocket chat
- `GET /api/v1/alerts` - Get active alerts

**Configuration:**
```bash
cargo run --package data-infrastructure --bin data-infrastructure \
  -- --host 127.0.0.1 --port 8080 --enable-cors \
  --db-host localhost --db-port 5432 --db-name agent_agency \
  --db-user postgres --db-password "$DATABASE_PASSWORD" \
  --enable-redis --redis-url "redis://localhost:6379"
```

### Worker System (`agent-workers`)

**Features:**
- Task execution engine
- MCP-based worker architecture
- Parallel task processing
- Resource management

**Configuration:**
```bash
cargo run --package agent-workers --bin agent-workers
```

### Web Dashboard (`apps/web-dashboard`)

**Features:**
- React/NextJS interface
- Real-time task monitoring
- Chat interface
- Metrics visualization
- Alert management
- Responsive design

**V3-Specific Scripts:**
```bash
npm run v3:dev     # Development server
npm run v3:build   # Production build
npm run v3:start   # Production server
```

## 🛠️ Management Commands

### Unified System Control

```bash
# Start all services
./scripts/v3/start-v3-system.sh start

# Stop all services
./scripts/v3/start-v3-system.sh stop

# Restart all services
./scripts/v3/start-v3-system.sh restart

# Check service status
./scripts/v3/start-v3-system.sh status

# View logs
./scripts/v3/start-v3-system.sh logs [service]

# Health check
./scripts/v3/start-v3-system.sh health
```

### Individual Service Control

```bash
# API Server
cd iterations/v3
cargo run --package data-infrastructure --bin data-infrastructure

# Worker System
cargo run --package agent-workers --bin agent-workers

# Web Dashboard
cd apps/web-dashboard
npm run v3:dev
```

### Database Management

```bash
# Setup database
node scripts/v3/setup/setup-database-v3.js init

# Check database status
node scripts/v3/setup/setup-database-v3.js status
```

## 🔍 Monitoring & Debugging

### Health Checks

```bash
# API Server
curl http://localhost:8080/health

# Web Dashboard
curl http://localhost:3000

# Worker System (if health endpoint exists)
curl http://localhost:8081/health
```

### Logs

```bash
# All services
./scripts/v3/start-v3-system.sh logs

# Specific service
./scripts/v3/start-v3-system.sh logs api-server
./scripts/v3/start-v3-system.sh logs worker-system
./scripts/v3/start-v3-system.sh logs web-dashboard

# Direct log access
tail -f iterations/v3/logs/api-server.log
tail -f iterations/v3/logs/worker-system.log
tail -f iterations/v3/logs/web-dashboard.log
```

### Metrics

```bash
# Get current metrics
curl http://localhost:8080/api/v1/metrics

# Stream metrics (Server-Sent Events)
curl http://localhost:8080/api/v1/metrics/stream
```

## 🔧 Configuration

### Environment Variables

```bash
# Required
export DATABASE_PASSWORD="agent_agency_secure_password_123"

# Optional
export DATABASE_URL="postgresql://postgres:${DATABASE_PASSWORD}@localhost:5432/agent_agency"
export REDIS_URL="redis://localhost:6379"
export RUST_LOG="info"
export RUST_BACKTRACE="1"
```

### Database Configuration

The system uses PostgreSQL with the following setup:
- **Database**: `agent_agency`
- **User**: `postgres`
- **Password**: From `DATABASE_PASSWORD` environment variable
- **Extensions**: `uuid-ossp`, `pgcrypto`, `vector` (if available)

### Redis Configuration

Redis is used for:
- Metrics caching
- Session storage
- Real-time data

## 🚨 Troubleshooting

### Common Issues

**1. Database Connection Failed**
```bash
# Check PostgreSQL status
brew services list | grep postgresql

# Start PostgreSQL
brew services start postgresql@14

# Verify connection
pg_isready -h localhost -p 5432
```

**2. Redis Connection Failed**
```bash
# Check Redis status
brew services list | grep redis

# Start Redis
brew services start redis

# Verify connection
redis-cli ping
```

**3. Port Already in Use**
```bash
# Find process using port
lsof -i :8080
lsof -i :3000

# Kill process
kill -9 <PID>
```

**4. Build Failures**
```bash
# Clean build cache
cargo clean

# Rebuild
cargo build --package data-infrastructure
```

### Service-Specific Issues

**API Server Issues:**
- Check database connection
- Verify Redis connectivity
- Check port 8080 availability
- Review logs: `tail -f logs/api-server.log`

**Worker System Issues:**
- Check port 8081 availability
- Verify MCP configuration
- Review logs: `tail -f logs/worker-system.log`

**Web Dashboard Issues:**
- Check Node.js version (v20+)
- Run `npm install`
- Check port 3000 availability
- Review logs: `tail -f logs/web-dashboard.log`

## 🔄 Development Workflow

### Making Changes

1. **Stop services**: `./scripts/v3/start-v3-system.sh stop`
2. **Make changes** to code
3. **Rebuild**: `cargo build` (for Rust changes)
4. **Restart**: `./scripts/v3/start-v3-system.sh start`

### Adding New Services

1. Add service to `SERVICES` array in `start-v3-system.sh`
2. Add port to `PORTS` array
3. Create startup function
4. Add to `start_all()` function

### Database Changes

1. Create migration file in `data-infrastructure/migrations/`
2. Run: `node scripts/v3/setup/setup-database-v3.js init`

## 📊 Performance

### Resource Usage

- **API Server**: ~50MB RAM, 1-2% CPU
- **Worker System**: ~30MB RAM, 1% CPU
- **Web Dashboard**: ~100MB RAM, 2-3% CPU
- **PostgreSQL**: ~200MB RAM, 1-2% CPU
- **Redis**: ~10MB RAM, <1% CPU

### Optimization Tips

- Use `sccache` for faster Rust builds
- Enable Redis caching for metrics
- Use connection pooling for database
- Enable gzip compression for API responses

## 🔒 Security

### Current Security Features

- Database password validation
- Rate limiting
- Input validation
- Audit logging
- Secure environment variable handling

### Security Checklist

- [ ] Change default database password
- [ ] Enable HTTPS in production
- [ ] Configure firewall rules
- [ ] Regular security updates
- [ ] Monitor audit logs

## 📈 Scaling

### Horizontal Scaling

- Multiple API server instances behind load balancer
- Worker system can scale independently
- Database read replicas
- Redis cluster for caching

### Vertical Scaling

- Increase database connection pool size
- Add more worker processes
- Increase Redis memory allocation
- Optimize database queries

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Make changes
4. Test with `./scripts/v3/start-v3-system.sh start`
5. Submit pull request

## 📝 License

MIT License - see LICENSE file for details.

---

**Need Help?** Check the logs, run health checks, or review the troubleshooting section above.
