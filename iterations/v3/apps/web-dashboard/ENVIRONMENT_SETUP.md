# Environment Setup for Agent Agency Web Dashboard

This document explains how to configure the environment for the web dashboard component of the Agent Agency system.

## Required Environment Variables

### V3 Backend Configuration

The web dashboard requires connection to the V3 backend service for most functionality.

```bash
# V3 Backend Host - The main backend service for agent orchestration
V3_BACKEND_HOST=http://localhost:8080

# V3 Backend API Key (if authentication is required)
V3_BACKEND_API_KEY=your-api-key-here
```

### Development Configuration

```bash
# Enable development features
NODE_ENV=development

# Next.js specific configuration
NEXT_PUBLIC_API_URL=http://localhost:3000/api

# Enable React strict mode in development
NEXT_PUBLIC_STRICT_MODE=true
```

### Feature Flags

```bash
# Enable experimental features
NEXT_PUBLIC_ENABLE_EXPERIMENTAL_FEATURES=false

# Enable debug logging
NEXT_PUBLIC_DEBUG_MODE=true

# Enable mock data for development (when V3 backend is not available)
NEXT_PUBLIC_USE_MOCK_DATA=true
```

## Setup Instructions

### 1. Create Environment File

Create a `.env.local` file in the web dashboard root directory:

```bash
cd iterations/v3/apps/web-dashboard
cp .env.example .env.local  # If .env.example exists, otherwise create manually
```

### 2. Configure V3 Backend Connection

The most critical configuration is the V3 backend host:

```bash
# For local development
V3_BACKEND_HOST=http://localhost:8080

# For production deployment
V3_BACKEND_HOST=https://your-v3-backend.example.com
```

### 3. Verify Configuration

The health check endpoint (`/api/health`) will validate the backend connection:

```bash
curl http://localhost:3000/api/health
```

Expected response when V3 backend is available:
```json
{
  "status": "healthy",
  "timestamp": "2025-10-20T...",
  "dashboard": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": 3600,
    "node_version": "v18.17.0"
  },
  "backend": {
    "status": "healthy",
    "url": "http://localhost:8080",
    "response_time_ms": 45
  }
}
```

## Development Mode

When `NEXT_PUBLIC_USE_MOCK_DATA=true`, the dashboard will use mock implementations for:

- Chat sessions and messaging
- Task management
- Database operations
- Metrics and monitoring

This allows development of the UI without requiring a running V3 backend.

## Production Deployment

For production deployment:

1. Set `NODE_ENV=production`
2. Configure the actual V3 backend URL
3. Set `NEXT_PUBLIC_USE_MOCK_DATA=false`
4. Configure any required API keys
5. Set up proper CORS origins if needed

## Troubleshooting

### Backend Connection Issues

If the health check shows `"backend.status": "unreachable"`:

1. Verify V3 backend is running
2. Check `V3_BACKEND_HOST` is correct
3. Ensure no firewall blocking the connection
4. Check backend logs for errors

### Mock Data Not Working

If mock data isn't loading in development:

1. Verify `NEXT_PUBLIC_USE_MOCK_DATA=true`
2. Check browser console for errors
3. Ensure API client is using mock implementations

## Security Notes

- Never commit `.env.local` files to version control
- Use strong, unique API keys for production
- Regularly rotate authentication tokens
- Monitor backend response times for performance issues
