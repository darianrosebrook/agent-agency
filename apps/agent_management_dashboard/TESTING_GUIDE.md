# API Testing Guide

## Servers Required

### 1. PostgreSQL Database
- **Port**: 5432 (or 5433 if configured)
- **Status**: ✅ Running
- **Check**: `pg_isready -h localhost -p 5432`
- **Connection**: The API server needs `DATABASE_URL` environment variable

### 2. Rust API Server
- **Port**: 8080
- **Status**: ✅ Running (but database connection timeout)
- **Health Check**: `curl http://localhost:8080/health`
- **Start Command**: 
  ```bash
  cd iterations/v3/data-interfaces-adapters
  DATABASE_URL="postgresql://postgres:password@localhost:5432/agent_agency" \
    cargo run --bin api-server -- --port 8080
  ```

### 3. Next.js Dashboard
- **Port**: 3001 (or 3000)
- **Status**: ✅ Running
- **Start Command**: 
  ```bash
  cd apps/agent_management_dashboard
  npm run dev
  ```

## API Proxy Route

The dashboard uses a Next.js API proxy route at `/api/proxy/[...path]` that forwards requests to the Rust API server.

- **Frontend Request**: `/api/proxy/api/v1/agents/stats`
- **Proxied To**: `http://localhost:8080/api/v1/agents/stats`

## Testing Endpoints

### Health Check
```bash
# Direct API
curl http://localhost:8080/health

# Through proxy
curl http://localhost:3001/api/proxy/api/v1/health
```

### Agent Endpoints
```bash
# Get all agents
curl http://localhost:3001/api/proxy/api/v1/agents

# Get agent stats
curl http://localhost:3001/api/proxy/api/v1/agents/stats

# Get specific agent stats
curl http://localhost:3001/api/proxy/api/v1/agents/{agentId}/stats
```

### Telemetry Endpoints
```bash
# Get contributions
curl 'http://localhost:3001/api/proxy/api/v1/telemetry/contributions?start_date=2024-01-01'

# Get model contributions
curl http://localhost:3001/api/proxy/api/v1/telemetry/model-contributions

# Get agent activity
curl 'http://localhost:3001/api/proxy/api/v1/telemetry/agent-activity?start_date=2024-01-01'
```

### Observability Endpoints
```bash
# Get efficiency metrics
curl http://localhost:3001/api/proxy/api/v1/observability/efficiency

# Get system metrics
curl http://localhost:3001/api/proxy/api/v1/observability/system-metrics
```

### Project Endpoints
```bash
# Get projects
curl http://localhost:3001/api/proxy/api/v1/projects

# Get project details
curl http://localhost:3001/api/proxy/api/v1/projects/{projectId}

# Get project settings
curl http://localhost:3001/api/proxy/api/v1/projects/{projectId}/settings
```

## Common Issues

### Database Connection Timeout
**Symptom**: Health check shows `"database":{"error":"pool timed out while waiting for an open connection"}`

**Solution**:
1. Ensure PostgreSQL is running: `pg_isready`
2. Check DATABASE_URL matches your PostgreSQL configuration
3. Restart API server with correct DATABASE_URL

### Empty Responses
**Symptom**: Endpoints return empty responses or 200 OK with no data

**Possible Causes**:
1. Database has no data (expected for new installations)
2. Authentication required (check if endpoints need auth tokens)
3. Database connection issues

### CORS Errors
**Symptom**: Browser console shows CORS errors

**Solution**: The API proxy route handles CORS by proxying through Next.js server-side

## Testing Checklist

- [ ] PostgreSQL is running and accessible
- [ ] API server is running on port 8080
- [ ] API server can connect to database
- [ ] Next.js dashboard is running
- [ ] API proxy route responds correctly
- [ ] Health endpoint works through proxy
- [ ] Agent endpoints return data (or empty arrays if no data)
- [ ] Telemetry endpoints work
- [ ] Observability endpoints work
- [ ] Project endpoints work

## Next Steps

1. **Fix Database Connection**: Ensure API server has correct DATABASE_URL
2. **Seed Test Data**: Add some test data to database for testing
3. **Test Dashboard Components**: Open dashboard and verify components load data
4. **Check Browser Console**: Look for API errors in browser dev tools




