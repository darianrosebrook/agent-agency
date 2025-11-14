# API Documentation

**Author:** @darianrosebrook  
**Date:** 2025-01-28  
**Status:** OpenAPI 3.0 specification implemented

---

## Overview

The Agent Agency API now has comprehensive OpenAPI 3.0 documentation with interactive Swagger UI.

## Access Points

### Interactive Documentation (Swagger UI)
- **URL:** `http://localhost:8080/swagger-ui/`
- **Description:** Interactive API explorer with try-it-out functionality

### OpenAPI JSON Specification
- **URL:** `http://localhost:8080/api-docs/openapi.json`
- **Description:** Machine-readable OpenAPI 3.0 specification in JSON format

## Documented Endpoints

### Health Endpoints (2)
- `GET /health` - Basic health check
- `GET /api/v1/health` - Detailed system health check

### Task Management (11)
- `POST /api/v1/tasks` - Submit a new task
- `GET /api/v1/tasks` - List all tasks
- `GET /api/v1/tasks/{task_id}` - Get task status
- `GET /api/v1/tasks/{task_id}/result` - Get task result
- `POST /api/v1/tasks/{task_id}/cancel` - Cancel a task
- `POST /api/v1/tasks/{task_id}/pause` - Pause a task
- `POST /api/v1/tasks/{task_id}/resume` - Resume a task
- `GET /api/v1/tasks/{task_id}/chain-of-thought` - Get reasoning chain
- `GET /api/v1/tasks/{task_id}/council-decisions` - Get council decisions
- `GET /api/v1/tasks/{task_id}/worker-actions` - Get worker actions

### Chat Endpoints (4)
- `GET /api/v1/chat/sessions` - List chat sessions
- `POST /api/v1/chat/sessions` - Create chat session
- `GET /api/v1/chat/sessions/{session_id}/messages` - Get chat messages
- `POST /api/v1/chat/sessions/{session_id}/messages` - Send chat message

### Authentication (4)
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/logout` - User logout
- `POST /api/v1/auth/refresh` - Refresh token
- `GET /api/v1/users/me` - Get current user

### Provenance (2)
- `GET /api/v1/provenance` - List provenance records
- `GET /api/v1/provenance/commit/{commit_hash}` - Get provenance by commit

### System Monitoring (1)
- `GET /api/v1/system/metrics` - Get system metrics

### Projects (3)
- `GET /api/v1/projects` - List all projects
- `GET /api/v1/projects/{project_id}` - Get project details
- `GET /api/v1/projects/{project_id}/tasks` - Get project tasks

### Database (3)
- `GET /api/v1/database/tables` - List database tables
- `GET /api/v1/database/tables/{table_name}` - Get table schema
- `POST /api/v1/database/query` - Execute database query

### Analytics (3)
- `GET /api/v1/analytics/tasks` - Get task analytics
- `GET /api/v1/analytics/performance` - Get performance analytics
- `GET /api/v1/analytics/success-rates` - Get success rates

### Agents (3)
- `GET /api/v1/agents` - List all agents
- `GET /api/v1/agents/{id}` - Get agent details
- `GET /api/v1/agents/{id}/stats` - Get agent statistics

**Total Documented Endpoints: 36**

## Documented Schemas

### Request/Response Types
- `TaskSubmissionRequest` - Task submission payload
- `TaskSubmissionResponse` - Task submission result
- `TaskStatusResponse` - Task status information
- `ChatSession` - Chat session data
- `ChatMessage` - Chat message data
- `LoginRequest` - Authentication credentials
- `LoginResponse` - Authentication result
- `UserResponse` - User information
- `ErrorResponse` - Standardized error format

## Authentication

The API supports two authentication methods:

1. **API Key**: Include `X-API-Key` header
2. **Bearer Token**: Include `Authorization: Bearer <token>` header

Both methods are documented in the OpenAPI spec under security schemes.

## Implementation Details

### OpenAPI Generation
- **Library:** `utoipa` 5.4
- **UI:** `utoipa-swagger-ui` 9.0.2
- **Location:** `iterations/v3/data-infrastructure/src/api/openapi.rs`
- **Path Documentation:** `iterations/v3/data-infrastructure/src/api/openapi_paths.rs`

### Schema Generation
- Types use `#[derive(ToSchema)]` from `utoipa`
- Compatible with existing `JsonSchema` derives from `schemars`
- Complex types from `agent_agency_contracts` are represented as `serde_json::Value` in schemas

## Next Steps

### Expand Documentation
1. **Add More Endpoints**: Document remaining endpoints (agents, projects, analytics, etc.)
2. **Add Request Examples**: Include example request/response bodies
3. **Add Parameter Documentation**: Document query parameters, path parameters, and headers
4. **Add Response Examples**: Include example responses for each endpoint

### Schema Improvements
1. **Add WorkingSpec Schema**: If `agent_agency_contracts` types can implement `ToSchema`
2. **Add ExecutionArtifacts Schema**: Document artifact structure
3. **Add QualityReport Schema**: Document quality report structure

### Integration
1. **Dashboard Integration**: Use OpenAPI spec to generate TypeScript types for dashboard
2. **Client Generation**: Generate API clients from OpenAPI spec
3. **Testing**: Use OpenAPI spec for contract testing

## Usage

### Viewing Documentation

1. Start the API server:
   ```bash
   cd iterations/v3/data-interfaces-adapters
   cargo run --bin api-server
   ```

2. Open Swagger UI:
   ```
   http://localhost:8080/swagger-ui/
   ```

3. Or fetch the OpenAPI JSON:
   ```bash
   curl http://localhost:8080/api-docs/openapi.json | jq
   ```

### Generating TypeScript Types

You can use tools like `openapi-typescript` to generate TypeScript types:

```bash
npx openapi-typescript http://localhost:8080/api-docs/openapi.json -o src/types/api.ts
```

## Files Modified

- `iterations/v3/data-infrastructure/Cargo.toml` - Added utoipa dependencies
- `iterations/v3/data-infrastructure/src/api/openapi.rs` - Main OpenAPI spec
- `iterations/v3/data-infrastructure/src/api/openapi_paths.rs` - Path documentation
- `iterations/v3/data-infrastructure/src/api/types.rs` - Added ToSchema derives
- `iterations/v3/data-infrastructure/src/api/api_errors.rs` - Added ToSchema derives
- `iterations/v3/data-infrastructure/src/api/handlers/auth_handlers.rs` - Added ToSchema derives
- `iterations/v3/data-infrastructure/src/chat_service.rs` - Added ToSchema derives
- `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs` - Added OpenAPI routes
- `iterations/v3/data-infrastructure/docs/API_DESCRIPTION.md` - API overview
- `iterations/v3/data-infrastructure/docs/API_DOCUMENTATION.md` - This file

