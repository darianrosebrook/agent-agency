# API Integration TODOs

**Author:** @darianrosebrook  
**Date:** 2025-01-28  
**Status:** In Progress

## Overview

This document tracks the implementation of API endpoints in the agent_management_dashboard using a centralized API provider with TypeScript types and Zod validation.

## Implementation Strategy

1. **Centralized API Provider** - React Context provider for API state management
2. **TypeScript Types** - Generated from OpenAPI spec or manually created
3. **Zod Schemas** - Runtime validation for all API requests/responses
4. **No Hard-coded Endpoints** - All API calls go through the provider
5. **No Mock APIs** - Remove all placeholder/mock implementations

## TODO Categories

### ✅ Completed
- [x] Chat API (`src/lib/api/chat.ts`)
- [x] Basic API utilities (`src/lib/utils/api.ts`)
- [x] Proxy route setup (`src/app/api/proxy/[...path]/route.ts`)

### 🔄 In Progress
- [ ] API Provider setup and context
- [ ] TypeScript type generation from OpenAPI
- [ ] Zod schema creation

### 📋 Pending by Category

#### Health Endpoints (2 endpoints)
- [ ] `GET /health` - Basic health check
- [ ] `GET /api/v1/health` - Detailed system health check
- **File:** `src/lib/api/health.ts`
- **Provider Method:** `health.getHealth()`, `health.getSystemHealth()`

#### Task Management (11 endpoints)
- [ ] `POST /api/v1/tasks` - Submit a new task
- [ ] `GET /api/v1/tasks` - List all tasks
- [ ] `GET /api/v1/tasks/{task_id}` - Get task status
- [ ] `GET /api/v1/tasks/{task_id}/result` - Get task result
- [ ] `POST /api/v1/tasks/{task_id}/cancel` - Cancel a task
- [ ] `POST /api/v1/tasks/{task_id}/pause` - Pause a task
- [ ] `POST /api/v1/tasks/{task_id}/resume` - Resume a task
- [ ] `GET /api/v1/tasks/{task_id}/chain-of-thought` - Get reasoning chain
- [ ] `GET /api/v1/tasks/{task_id}/council-decisions` - Get council decisions
- [ ] `GET /api/v1/tasks/{task_id}/worker-actions` - Get worker actions
- **File:** `src/lib/api/tasks.ts` (expand existing)
- **Provider Method:** `tasks.*`

#### Task Observation (8 endpoints)
- [ ] `GET /api/v1/tasks/stats` - Task statistics
- [ ] `GET /api/v1/tasks/stats/history` - Task statistics history
- [ ] `GET /api/v1/tasks/{task_id}/logs` - Task execution logs
- [ ] `GET /api/v1/tasks/{task_id}/progress` - Task progress details
- [ ] `GET /api/v1/tasks/{task_id}/events` - Task execution events
- [ ] `GET /api/v1/tasks/{task_id}/comments` - Task comments
- [ ] `POST /api/v1/tasks/{task_id}/comments` - Create task comment
- [ ] `GET /api/v1/tasks/{task_id}/provenance` - Task provenance
- **File:** `src/lib/api/tasks.ts` (expand existing)
- **Provider Method:** `tasks.*`

#### Chat Endpoints (4 endpoints)
- [x] `GET /api/v1/chat/sessions` - List chat sessions ✅
- [x] `POST /api/v1/chat/sessions` - Create chat session ✅
- [x] `GET /api/v1/chat/sessions/{session_id}/messages` - Get chat messages ✅
- [x] `POST /api/v1/chat/sessions/{session_id}/messages` - Send chat message ✅
- [ ] `GET /api/v1/chat/sessions/{session_id}` - Get chat session
- **File:** `src/lib/api/chat.ts` (expand existing)
- **Provider Method:** `chat.*`

#### Authentication (4 endpoints)
- [ ] `POST /api/v1/auth/login` - User login
- [ ] `POST /api/v1/auth/logout` - User logout
- [ ] `POST /api/v1/auth/refresh` - Refresh token
- [ ] `GET /api/v1/users/me` - Get current user
- **File:** `src/lib/api/auth.ts` (new)
- **Provider Method:** `auth.*`

#### Agent Management (3 endpoints)
- [x] `GET /api/v1/agents` - List all agents ✅
- [x] `GET /api/v1/agents/{id}` - Get agent details ✅
- [x] `GET /api/v1/agents/{id}/stats` - Get agent statistics ✅
- **File:** `src/lib/api/agents.ts` (expand existing)
- **Provider Method:** `agents.*`

#### Agent Observation (8 endpoints)
- [x] `GET /api/v1/agents/stats` - Overall agent statistics ✅
- [ ] `GET /api/v1/agents/tasks/completion` - Agent task completion metrics
- [ ] `GET /api/v1/agents/efficiency` - Agent efficiency metrics
- [ ] `GET /api/v1/agents/{id}/health` - Agent health status
- [ ] `GET /api/v1/agents/{id}/metrics` - Agent performance metrics
- [ ] `GET /api/v1/agents/{id}/logs` - Agent execution logs
- **File:** `src/lib/api/agents.ts` (expand existing)
- **Provider Method:** `agents.*`

#### Judge Management (5 endpoints)
- [ ] `GET /api/v1/judges` - List all judges
- [ ] `GET /api/v1/judges/stats` - Judge statistics
- [ ] `GET /api/v1/judges/{id}` - Get judge details
- [ ] `GET /api/v1/judges/{id}/stats` - Judge-specific statistics
- [ ] `GET /api/v1/judges/{id}/evaluations` - Judge evaluation history
- **File:** `src/lib/api/judges.ts` (new)
- **Provider Method:** `judges.*`

#### Telemetry & Observability (6 endpoints)
- [x] `GET /api/v1/telemetry/contributions` - Contributions telemetry ✅
- [x] `GET /api/v1/telemetry/model-contributions` - Model contributions ✅
- [x] `GET /api/v1/telemetry/agent-activity` - Agent activity tracking ✅
- [x] `GET /api/v1/observability/efficiency` - System efficiency metrics ✅
- [x] `GET /api/v1/observability/system-metrics` - System observability metrics ✅
- [x] `GET /api/v1/observability/alerts` - System alerts ✅
- **File:** `src/lib/api/observability.ts` (expand existing)
- **Provider Method:** `observability.*`

#### System Monitoring (3 endpoints)
- [ ] `GET /api/v1/system/health` - Detailed system health
- [ ] `GET /api/v1/system/resources` - System resource usage
- [ ] `GET /api/v1/system/metrics` - System metrics
- **File:** `src/lib/api/system.ts` (new)
- **Provider Method:** `system.*`

#### Session Control (4 endpoints)
- [ ] `GET /api/v1/sessions/{session_id}` - Get session status
- [ ] `POST /api/v1/sessions/{session_id}/pause` - Pause session
- [ ] `POST /api/v1/sessions/{session_id}/resume` - Resume session
- [ ] `POST /api/v1/sessions/{session_id}/cancel` - Cancel session
- **File:** `src/lib/api/sessions.ts` (new)
- **Provider Method:** `sessions.*`

#### Search & Queries (5 endpoints)
- [x] `GET /api/v1/search` - Global search ✅
- [ ] `GET /api/v1/queries` - List saved queries
- [ ] `POST /api/v1/queries` - Save query
- [ ] `DELETE /api/v1/queries/{query_id}` - Delete query
- **File:** `src/lib/api/search.ts` (expand existing), `src/lib/api/queries.ts` (new)
- **Provider Method:** `search.*`, `queries.*`

#### Query Performance (4 endpoints)
- [ ] `GET /api/v1/query-performance/summary` - Query performance summary
- [ ] `GET /api/v1/query-performance/metrics` - Query performance metrics
- [ ] `GET /api/v1/query-performance/slow` - Slow queries
- [ ] `GET /api/v1/query-performance/top-slow` - Top slow queries
- **File:** `src/lib/api/queryPerformance.ts` (new)
- **Provider Method:** `queryPerformance.*`

#### Provenance (4 endpoints)
- [ ] `GET /api/v1/provenance` - List provenance records
- [ ] `POST /api/v1/provenance/link` - Link provenance to task
- [ ] `GET /api/v1/provenance/verify/{commit_hash}` - Verify provenance
- [ ] `GET /api/v1/provenance/commit/{commit_hash}` - Get provenance by commit
- **File:** `src/lib/api/provenance.ts` (new)
- **Provider Method:** `provenance.*`

#### Waivers (3 endpoints)
- [ ] `GET /api/v1/waivers` - List waivers
- [ ] `POST /api/v1/waivers` - Create waiver
- [ ] `POST /api/v1/waivers/{waiver_id}/approve` - Approve waiver
- **File:** `src/lib/api/waivers.ts` (new)
- **Provider Method:** `waivers.*`

#### SLOs (4 endpoints)
- [ ] `GET /api/v1/slos` - List SLOs
- [ ] `GET /api/v1/slos/{slo_name}/status` - Get SLO status
- [ ] `GET /api/v1/slos/{slo_name}/measurements` - Get SLO measurements
- [ ] `GET /api/v1/slo-alerts` - List SLO alerts
- **File:** `src/lib/api/slos.ts` (new)
- **Provider Method:** `slos.*`

#### Projects (3 endpoints)
- [x] `GET /api/v1/projects` - List all projects ✅
- [x] `GET /api/v1/projects/{project_id}` - Get project details ✅
- [x] `GET /api/v1/projects/{project_id}/tasks` - Get project tasks ✅
- **File:** `src/lib/api/projects.ts` (expand existing)
- **Provider Method:** `projects.*`

#### Database (3 endpoints)
- [ ] `GET /api/v1/database/tables` - List database tables
- [ ] `GET /api/v1/database/tables/{table_name}` - Get table schema
- [ ] `POST /api/v1/database/query` - Execute database query
- **File:** `src/lib/api/database.ts` (new)
- **Provider Method:** `database.*`

#### Analytics (3 endpoints)
- [x] `GET /api/v1/analytics/tasks` - Get task analytics ✅
- [x] `GET /api/v1/analytics/performance` - Get performance analytics ✅
- [x] `GET /api/v1/analytics/success-rates` - Get success rates ✅
- **File:** `src/lib/api/analytics.ts` (expand existing)
- **Provider Method:** `analytics.*`

## Implementation Checklist

### Phase 1: Foundation
- [ ] Create API Provider Context (`src/lib/providers/ApiProvider.tsx`)
- [ ] Create base API client with error handling
- [ ] Set up Zod schema directory structure
- [ ] Create TypeScript type definitions from OpenAPI
- [ ] Remove hard-coded API URLs from components

### Phase 2: Core APIs
- [ ] Health endpoints
- [ ] Authentication endpoints
- [ ] Task management (expand existing)
- [ ] Task observation (expand existing)
- [ ] Agent management (expand existing)
- [ ] Agent observation (expand existing)

### Phase 3: Advanced APIs
- [ ] Judge management
- [ ] Session control
- [ ] Query management
- [ ] Query performance
- [ ] Provenance
- [ ] Waivers
- [ ] SLOs
- [ ] Database inspection

### Phase 4: Cleanup
- [ ] Remove all mock/placeholder APIs
- [ ] Remove all hard-coded endpoints
- [ ] Ensure all API calls go through provider
- [ ] Add Zod validation to all endpoints
- [ ] Add error handling and retry logic
- [ ] Add loading states and caching

## Hard-coded Endpoints to Replace

1. **`src/lib/stores/projectStore.ts`** - Lines 903, 952, 1023
   - Replace with `api.projects.*` methods

2. **`src/lib/services/kokoroTTS.ts`** - Lines 94, 187
   - Keep as-is (external service, not our API)

3. **`src/components/Chat.tsx`** - Line 190
   - Replace with `api.chat.stream()` method

## Mock/Placeholder APIs to Remove

1. **`src/components/ChatAIHelper.ts`** - `simulateAIResponse`
   - Remove fallback simulation, use real API only

2. Check for any other `TODO: Replace with actual API` comments

## Notes

- All API calls should use the centralized provider
- All requests/responses should be validated with Zod
- All types should be generated from OpenAPI or manually maintained
- Error handling should be consistent across all endpoints
- Loading states should be managed by the provider
- Caching should be implemented where appropriate

