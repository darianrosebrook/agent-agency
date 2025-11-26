# Dashboard Data Inventory & API Integration Analysis

**Date**: November 26, 2025  
**Status**: Completed (Milestone 1-4)  
**Last Updated**: November 26, 2025

## Overview

This document provides a comprehensive inventory of all dashboard pages, the data they display, the API endpoints they consume, and gaps in data collection that need to be addressed.

---

## 1. Pages Inventory

### 1.1 Main Dashboard (`/`)
**Component**: `src/components/dashboard/Dashboard.tsx`

| Chart/Widget | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| TaskProgressChart | `completed`, `total` tasks | `GET /api/v1/tasks/stats` | ✅ Working |
| RadialTaskProgress | `completed`, `total`, `in_progress`, `pending` | `GET /api/v1/tasks/stats` | ✅ Working |
| MultiRingProgress | Project milestones, task stats per milestone | `GET /api/v1/projects/:id/milestones`, `GET /api/v1/projects/:id/tasks/stats` | ✅ Working |
| CodeContributionChart | Daily contribution counts | `GET /api/v1/telemetry/contributions?group_by=day` | ⚠️ Mock Data |
| HexagonHeatmap | Task activity by day/hour | `GET /api/v1/tasks/stats/history` | ✅ Working |
| ModelContributionStream | Model usage stats | `GET /api/v1/telemetry/model-contributions` | ✅ Working |
| TaskCompletionGauge | Task creation vs completion rate | `GET /api/v1/tasks/stats` | ✅ Working |
| ServerEfficiencyChart | Server efficiency metrics | `GET /api/v1/agents/efficiency` | ⚠️ Partial |
| AgentActivityChart | Agent activity time series | `GET /api/v1/telemetry/agent-activity` | ✅ Working |
| AnalyticsMetrics | Task analytics summary | `GET /api/v1/analytics/tasks` | ✅ Working |

### 1.2 Agent Stats Page (`/agent-stats`)
**Component**: `src/app/agent-stats/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Overview Metrics | total, active, inactive agents | `GET /api/v1/agents/stats` | ✅ Working |
| Agent Type Breakdown | by_type counts | `GET /api/v1/agents/stats` | ✅ Working |
| Model Usage | model name, task count, success rate, avg completion time | `GET /api/v1/telemetry/model-contributions` | ✅ Working |
| Code Contributions | lines added/modified/deleted, files changed, commits | `GET /api/v1/telemetry/contributions` | ⚠️ Mock Data |
| Task Completion Metrics | completion rate, success rate, execution time | `GET /api/v1/agents/tasks/completion` | ⚠️ Mock Data |
| Efficiency Metrics | tasks/hour, efficiency score, token usage | `GET /api/v1/agents/efficiency` | ⚠️ Partial |
| Agent Activity Chart | activity time series | `GET /api/v1/telemetry/agent-activity` | ✅ Working |

### 1.3 Agent Health Page (`/agent-health`)
**Component**: `src/app/agent-health/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Infrastructure Services | API server status, database status | `GET /api/v1/system/health` | ✅ Working |
| System Metrics | CPU, memory, disk, network | `GET /api/v1/observability/system-metrics` | ❌ Not Implemented |
| Active Alerts | severity, title, message, source | `GET /api/v1/observability/alerts` | ⚠️ Returns Empty |
| Agent List | name, type, specialty, is_active | `GET /api/v1/agents` | ✅ Working |
| Agent Health | status, uptime, health score, error count | `GET /api/v1/agents/:id/health` | ❌ Not Implemented |
| Agent Metrics | CPU, memory, response times, requests/sec | `GET /api/v1/agents/:id/metrics` | ❌ Not Implemented |
| Agent Logs | level, message, timestamp | `GET /api/v1/agents/:id/logs` | ❌ Not Implemented |

### 1.4 Projects Page (`/projects`)
**Component**: `src/components/projects/Projects.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Project List | id, name, overview, state, created_at | `GET /api/v1/projects` | ✅ Working |
| Project Details | full project data | `GET /api/v1/projects/:id` | ✅ Working |
| Project Milestones | title, description, due_date, completed | `GET /api/v1/projects/:id/milestones` | ✅ Working |
| Project Tasks | title, status, priority, assignee | `GET /api/v1/projects/:id/tasks` | ✅ Working |
| Project Task Stats | total, completed, in_progress, pending | `GET /api/v1/projects/:id/tasks/stats` | ✅ Working |

### 1.5 Chat Page (`/chat`)
**Component**: `src/app/chat/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Chat Sessions | session_id, title, created_at | `GET /api/v1/chat/sessions` | ✅ Working |
| Chat Messages | role, content, timestamp | `GET /api/v1/chat/sessions/:id/messages` | ✅ Working |
| Send Message | message content | `POST /api/v1/chat/sessions/:id/messages` | ✅ Working |
| Stream Response | streaming SSE | `POST /api/v1/chat/stream` | ✅ Working |

### 1.6 Settings Page (`/settings`)
**Component**: `src/app/settings/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| User Settings | key-value pairs | `GET /api/v1/settings/user` | ✅ Working |
| App Settings | key-value pairs | `GET /api/v1/settings/app` | ✅ Working |
| API Keys | id, name, created_at, last_used | `GET /api/v1/settings/api-keys` | ✅ Working |
| Password Change | old/new password | `POST /api/v1/settings/password` | ✅ Working |

### 1.7 Rules & Governance Page (`/rules-governance`)
**Component**: `src/app/rules-governance/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Rules List | id, name, type, enabled | `GET /api/v1/rules` | ✅ Working |
| Rule Templates | id, name, description | `GET /api/v1/rules/templates` | ✅ Working |
| Violations | id, rule_id, severity, resolved | `GET /api/v1/violations` | ✅ Working |
| Specifications | id, name, type | `GET /api/v1/specifications` | ✅ Working |

### 1.8 Search Page (`/search`)
**Component**: `src/app/search/page.tsx`

| Data Section | Data Required | API Endpoint | Status |
|--------------|---------------|--------------|--------|
| Global Search | query results | `GET /api/v1/search?q=...` | ✅ Working |
| Saved Queries | query_id, name, query | `GET /api/v1/queries` | ✅ Working |

---

## 2. API Endpoints Status Matrix

### 2.1 Fully Implemented (Data Flows End-to-End)

| Endpoint | Handler | Data Source |
|----------|---------|-------------|
| `GET /api/v1/tasks` | `list_tasks_handler` | PostgreSQL `tasks` table |
| `GET /api/v1/tasks/stats` | `get_tasks_stats_handler` | PostgreSQL `tasks` table |
| `GET /api/v1/tasks/:id` | `get_task_status_handler` | PostgreSQL `tasks` table |
| `POST /api/v1/tasks` | `submit_task_handler` | PostgreSQL + Orchestrator |
| `GET /api/v1/agents` | `list_agents_handler` | PostgreSQL `workers` table |
| `GET /api/v1/agents/stats` | `get_agents_stats_handler` | PostgreSQL `workers` table |
| `GET /api/v1/projects` | `list_projects_handler` | PostgreSQL `projects` table |
| `GET /api/v1/projects/:id` | `get_project_handler` | PostgreSQL `projects` table |
| `GET /api/v1/projects/:id/milestones` | `get_project_milestones_handler` | PostgreSQL `project_milestones` table |
| `GET /api/v1/projects/:id/tasks` | `get_project_tasks_handler` | PostgreSQL `tasks` table |
| `GET /api/v1/system/health` | `get_system_health_handler` | Database connection check |
| `GET /api/v1/analytics/tasks` | `get_task_analytics_handler` | PostgreSQL `tasks` table |

### 2.2 Implemented but Returning Mock/Placeholder Data

| Endpoint | Issue | Required Data Source |
|----------|-------|---------------------|
| `GET /api/v1/telemetry/contributions` | Returns empty/mock | Needs git integration or provenance tracking |
| `GET /api/v1/agents/efficiency` | Partial data | Needs task execution metrics |
| `GET /api/v1/agents/tasks/completion` | Partial data | Needs task execution metrics |
| `GET /api/v1/observability/alerts` | Returns empty | Needs alerting system |

### 2.3 Recently Implemented (Now Working)

| Endpoint | Implementation | Data Source |
|----------|---------------|-------------|
| `GET /api/v1/telemetry/model-contributions` | Uses `telemetry_model_contributions` table | LLM request logging via TelemetryService |
| `GET /api/v1/telemetry/agent-activity` | Uses `telemetry_agent_activity` table | Agent activity logging via TelemetryService |
| `GET /api/v1/tasks/stats/history` | Uses `task_stats_history` table + fallback | Daily snapshots + computed from tasks |
| `GET /api/v1/projects/:id/tasks/stats` | Uses `get_project_task_stats()` | PostgreSQL with project_id filter |

### 2.4 Not Implemented (Returns 501 or Empty)

| Endpoint | Frontend Expectation | Required Implementation |
|----------|---------------------|------------------------|
| `GET /api/v1/agents/:id/health` | AgentHealth object | Agent health monitoring service |
| `GET /api/v1/agents/:id/metrics` | AgentMetrics object | Agent metrics collection |
| `GET /api/v1/agents/:id/logs` | AgentLog[] | Agent log aggregation |
| `GET /api/v1/observability/system-metrics` | SystemMetrics | System metrics collection |

---

## 3. Data Collection Gaps

### 3.1 Missing: Telemetry Data

**Problem**: Dashboard charts for contributions, model usage, and agent activity show mock data.

**Required Data Collection**:

1. **Code Contributions** (`telemetry_contributions` table)
   - `agent_id`, `lines_added`, `lines_modified`, `lines_deleted`, `files_changed`, `commits`, `timestamp`
   - **Source**: Git commit analysis or provenance tracking
   
2. **Model Contributions** (`telemetry_model_contributions` table)
   - `model_name`, `request_count`, `total_tokens`, `success_count`, `avg_response_time_ms`, `timestamp`
   - **Source**: LLM API request logging in agent workers

3. **Agent Activity** (`telemetry_agent_activity` table)
   - `agent_id`, `activity_type`, `count`, `timestamp`
   - **Source**: Agent execution logging

### 3.2 Missing: Agent Health Metrics

**Problem**: Agent health page cannot show real health, metrics, or logs.

**Required Data Collection**:

1. **Agent Health** (`agent_health_snapshots` table)
   - `agent_id`, `status`, `uptime_seconds`, `error_count`, `response_time_ms`, `health_score`, `timestamp`
   - **Source**: Periodic health checks of agent workers

2. **Agent Metrics** (`agent_metrics` table)
   - `agent_id`, `cpu_usage_percent`, `memory_usage_mb`, `requests_per_second`, `error_rate`, `timestamp`
   - **Source**: Resource monitoring of agent processes

3. **Agent Logs** (`agent_logs` table)
   - `agent_id`, `level`, `message`, `metadata`, `timestamp`
   - **Source**: Agent log aggregation

### 3.3 Missing: Historical Stats

**Problem**: Task stats history returns empty, preventing trend charts.

**Required Data Collection**:

1. **Task Stats History** (`task_stats_history` table)
   - `date`, `total`, `completed`, `in_progress`, `pending`, `failed`, `cancelled`, `completion_rate`
   - **Source**: Daily cron job to snapshot task stats

### 3.4 Missing: System Metrics

**Problem**: System metrics endpoint returns null.

**Required Data Collection**:

1. **System Metrics** (in-memory or `system_metrics` table)
   - `cpu_usage_percent`, `memory_usage_mb`, `disk_usage_percent`, `network_io_mbps`, `timestamp`
   - **Source**: `sysinfo` crate already imported, needs periodic collection

---

## 4. Bill of Materials: API Integration Requirements

### 4.1 Priority 1: Fix Existing Endpoints (Data Already Available)

| Endpoint | Fix Required | Effort |
|----------|--------------|--------|
| `GET /api/v1/projects/:id/tasks/stats` | Add project_id filter to query | Low |
| `GET /api/v1/observability/system-metrics` | Use `sysinfo` crate to collect | Medium |
| `GET /api/v1/tasks/stats/history` | Create daily snapshot job | Medium |

### 4.2 Priority 2: Add Data Collection (New Infrastructure)

| Data Type | Collection Method | Storage | Effort |
|-----------|------------------|---------|--------|
| LLM Request Metrics | Middleware in agent workers | `telemetry_model_contributions` | Medium |
| Agent Activity | Event logging in orchestrator | `telemetry_agent_activity` | Medium |
| Agent Health | Periodic health check service | `agent_health_snapshots` | High |
| Agent Metrics | Resource monitoring service | `agent_metrics` | High |
| Agent Logs | Log aggregation service | `agent_logs` | High |

### 4.3 Priority 3: Git/Provenance Integration

| Data Type | Collection Method | Storage | Effort |
|-----------|------------------|---------|--------|
| Code Contributions | Git commit analysis | `telemetry_contributions` | High |
| Provenance Tracking | Commit-to-task linking | `provenance` table | High |

---

## 5. Recommended Implementation Order

### Phase 1: Quick Wins (1-2 days)
1. Fix `GET /api/v1/projects/:id/tasks/stats` to filter by project_id
2. Implement `GET /api/v1/observability/system-metrics` using `sysinfo`
3. Add task stats history table and daily snapshot job

### Phase 2: Telemetry Infrastructure (3-5 days)
1. Add LLM request logging in agent workers
2. Create `telemetry_model_contributions` table and endpoint
3. Add agent activity logging in orchestrator
4. Create `telemetry_agent_activity` table and endpoint

### Phase 3: Agent Monitoring (5-7 days)
1. Design agent health monitoring architecture
2. Implement periodic health checks
3. Create agent metrics collection service
4. Implement agent log aggregation

### Phase 4: Git Integration (Optional, 3-5 days)
1. Analyze git commits for code contribution stats
2. Link commits to tasks via provenance tracking
3. Implement contribution calculation

---

## 6. Database Schema Requirements

### New Tables Needed

```sql
-- Telemetry: Model Contributions
CREATE TABLE telemetry_model_contributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Telemetry: Agent Activity
CREATE TABLE telemetry_agent_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES workers(id),
    activity_type VARCHAR(100) NOT NULL,
    activity_count INTEGER NOT NULL DEFAULT 1,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Task Stats History (daily snapshots)
CREATE TABLE task_stats_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date DATE NOT NULL UNIQUE,
    total INTEGER NOT NULL DEFAULT 0,
    completed INTEGER NOT NULL DEFAULT 0,
    in_progress INTEGER NOT NULL DEFAULT 0,
    pending INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    cancelled INTEGER NOT NULL DEFAULT 0,
    completion_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent Health Snapshots
CREATE TABLE agent_health_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES workers(id),
    status VARCHAR(20) NOT NULL, -- healthy, warning, critical, offline
    uptime_seconds BIGINT NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    health_score DOUBLE PRECISION NOT NULL DEFAULT 100.0,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent Metrics
CREATE TABLE agent_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES workers(id),
    cpu_usage_percent DOUBLE PRECISION,
    memory_usage_mb DOUBLE PRECISION,
    response_time_p50_ms INTEGER,
    response_time_p95_ms INTEGER,
    response_time_p99_ms INTEGER,
    requests_per_second DOUBLE PRECISION,
    error_rate DOUBLE PRECISION,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent Logs
CREATE TABLE agent_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES workers(id),
    level VARCHAR(10) NOT NULL, -- error, warn, info, debug
    message TEXT NOT NULL,
    metadata JSONB,
    logged_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_telemetry_model_contributions_recorded_at ON telemetry_model_contributions(recorded_at);
CREATE INDEX idx_telemetry_agent_activity_agent_id ON telemetry_agent_activity(agent_id);
CREATE INDEX idx_telemetry_agent_activity_recorded_at ON telemetry_agent_activity(recorded_at);
CREATE INDEX idx_task_stats_history_date ON task_stats_history(snapshot_date);
CREATE INDEX idx_agent_health_snapshots_agent_id ON agent_health_snapshots(agent_id);
CREATE INDEX idx_agent_health_snapshots_recorded_at ON agent_health_snapshots(recorded_at);
CREATE INDEX idx_agent_metrics_agent_id ON agent_metrics(agent_id);
CREATE INDEX idx_agent_metrics_recorded_at ON agent_metrics(recorded_at);
CREATE INDEX idx_agent_logs_agent_id ON agent_logs(agent_id);
CREATE INDEX idx_agent_logs_level ON agent_logs(level);
CREATE INDEX idx_agent_logs_logged_at ON agent_logs(logged_at);
```

---

## 7. Summary

### Current State (Updated November 26, 2025)
- **Fully Working**: 16 endpoints with real data (up from 12)
- **Partial/Mock Data**: 4 endpoints returning empty or placeholder data (down from 7)
- **Not Implemented**: 4 endpoints returning 501 (down from 5)

### Completed Work
- Fixed `GET /api/v1/projects/:id/tasks/stats` with efficient database query
- Created telemetry infrastructure (migration 030)
- Implemented `TelemetryService` for centralized logging
- Added agent activity logging on task start/complete/fail
- Implemented daily task stats snapshot job
- Updated model contributions and agent activity endpoints

### Remaining Data Gaps
- No code contribution tracking (git integration)
- No agent health/metrics monitoring
- No system metrics collection
- No alerting system

### Recommended Next Steps
1. **Medium Impact, Medium Effort**: Add system metrics collection using `sysinfo`
2. **Medium Impact, High Effort**: Agent health monitoring infrastructure
3. **Low Impact, High Effort**: Git integration for code contribution tracking

