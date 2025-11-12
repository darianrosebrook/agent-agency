# Agent Agency V3 Startup Script

## Overview

`start_v3_agent.sh` is a comprehensive startup script that manages all V3 system services:
- PostgreSQL database (Docker container)
- V3 API Server (Rust)
- V3 Dashboard (Next.js)

## Port Configuration

The script uses **non-standard ports** to avoid conflicts with other local development projects:

| Service | Port | Standard Port | Reason |
|---------|------|---------------|--------|
| PostgreSQL | **5433** | 5432 | Avoids conflicts with default PostgreSQL |
| API Server | **18080** | 8080 | Avoids conflicts with common dev servers |
| Dashboard | **13000** | 3000 | Avoids conflicts with other Next.js apps |

## Usage

### Start All Services

```bash
./scripts/v3/start_v3_agent.sh
# or
./scripts/v3/start_v3_agent.sh start
```

The script will:
1. Check if services are already running (won't start duplicates)
2. Start PostgreSQL in Docker (if not running)
3. Start API Server (if not running)
4. Start Dashboard (if not running)
5. Wait for all services to be ready
6. Perform health checks

### Check Status

```bash
./scripts/v3/start_v3_agent.sh status
```

Shows the current status of all services.

### Stop All Services

```bash
./scripts/v3/start_v3_agent.sh stop
```

Stops all services gracefully.

### View Logs

```bash
# View all logs
./scripts/v3/start_v3_agent.sh logs

# View specific service logs
./scripts/v3/start_v3_agent.sh logs api-server
./scripts/v3/start_v3_agent.sh logs dashboard
./scripts/v3/start_v3_agent.sh logs postgres
```

## Features

### Smart Startup
- **No Duplicates**: Checks if services are already running before starting
- **Port Detection**: Warns if ports are in use by other processes
- **Health Checks**: Waits for services to be ready before proceeding
- **Graceful Handling**: Handles stale PID files and containers

### Service Management
- **PostgreSQL**: Docker container management with persistent data
- **API Server**: Rust binary with proper environment setup
- **Dashboard**: Next.js dev server with automatic dependency installation

### Logging
- All logs are written to `iterations/v3/logs/`
- Separate log files for each service
- PID files stored in `iterations/v3/pids/`

## Environment Variables

The script sets up the following environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `RUST_LOG`: Rust logging level (default: `info`)
- `RUST_BACKTRACE`: Rust backtrace (default: `1`)
- `PORT`: Dashboard port (default: `13000`)

## Configuration Files

### Dashboard `.env.local`

The script automatically creates/updates `.env.local` in the dashboard directory:

```env
NEXT_PUBLIC_API_URL=http://localhost:18080
API_ADMIN_USERNAME=admin
API_ADMIN_PASSWORD=
```

## Troubleshooting

### Port Already in Use

If a port is already in use:
1. Check what's using it: `lsof -i :PORT`
2. Stop the conflicting service
3. Or modify the port in the script

### Services Won't Start

1. Check logs: `./scripts/v3/start_v3_agent.sh logs`
2. Verify Docker is running: `docker ps`
3. Check Rust toolchain: `cargo --version`
4. Check Node.js: `node --version`

### PostgreSQL Container Issues

```bash
# Remove and recreate container
docker stop agent-agency-v3-postgres
docker rm agent-agency-v3-postgres
./scripts/v3/start_v3_agent.sh start
```

## Service URLs

Once started, services are available at:

- **Dashboard**: http://localhost:13000
- **API Server**: http://localhost:18080
- **PostgreSQL**: localhost:5433

## Notes

- PostgreSQL data persists in `iterations/v3/data/postgres/`
- The script uses `nohup` to run services in the background
- PID files allow the script to track running processes
- All services are started with proper environment configuration














