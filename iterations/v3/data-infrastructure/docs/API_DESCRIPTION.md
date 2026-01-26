# Agent Agency API

REST API for Agent Agency - AI agent orchestration and management platform.

## Overview

The Agent Agency API provides comprehensive endpoints for:

- **Task Management**: Submit, monitor, and control autonomous agent tasks
- **Chat Sessions**: Real-time chat with AI agents
- **Authentication**: User authentication and session management
- **Provenance**: Code provenance tracking and audit trails
- **System Monitoring**: Health checks, metrics, and observability
- **Agent Management**: Agent lifecycle and performance monitoring
- **Project Management**: Project organization and task grouping
- **Analytics**: Performance analytics and reporting

## Base URL

- **Development**: `http://localhost:8889`
- **Production**: `https://api.agent-agency.dev`

## Authentication

The API supports two authentication methods:

1. **API Key**: Include `X-API-Key` header with your API key
2. **Bearer Token**: Include `Authorization: Bearer <token>` header

## Rate Limiting

API requests are rate-limited to prevent abuse. Rate limits are configurable per endpoint.

## Error Handling

All errors follow a consistent format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "status": 400
}
```

## Versioning

The API is versioned using URL paths: `/api/v1/...`

