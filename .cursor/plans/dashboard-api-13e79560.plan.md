<!-- 13e79560-9404-4e79-91c4-a93e8f30bf12 20f75cf1-52a4-4b0a-962a-93e9736788bb -->
# Dashboard API Integration Plan

## Current State Summary

**Working Endpoints (12):** Tasks, task stats, agents, agent stats, projects, milestones, system health, analytics

**Partial/Mock Data (7):** Telemetry endpoints returning empty data, efficiency metrics incomplete

**Not Implemented (5):** Agent health/metrics/logs, system metrics (partially), project task stats

## Bill of Materials

### Milestone 1: Quick Wins (Low Effort, High Impact)

#### 1.1 Fix Project Task Stats Endpoint

- **File:** [api-server.rs](iterations/v3/data-interfaces-adapters/src/bin/api-server.rs)
- **Issue:** `GET /api/v1/projects/:id/tasks/stats` returns 0 for all counts
- **Fix:** Query tasks table with `WHERE project_id = $1` filter
- **Data Source:** Existing `tasks` table with `project_id` column

#### 1.2 System Metrics Already Working

- **Status:** `get_resource_usage_handler` at line 7066 already uses `sysinfo` crate
- **Returns:** CPU, memory, disk, network metrics
- **No changes needed** - verify frontend is calling correct endpoint

### Milestone 2: Database Schema for Telemetry (Migration Required)

#### 2.1 New Migration: `030_create_telemetry_tracking_tables.sql`

Location: [migrations/](iterations/v3/data-infrastructure/migrations/)

```sql
-- Model Contributions (LLM usage tracking)
CREATE TABLE telemetry_model_contributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent Activity (execution events)
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
```

### Milestone 3: API Handlers for Telemetry Data

#### 3.1 Model Contributions Endpoint

- **Endpoint:** `GET /api/v1/telemetry/model-contributions`
- **Handler:** New `get_model_contributions_handler`
- **Query:** Aggregate from `telemetry_model_contributions` table
- **Response Format:**
```json
{
  "contributions": [
    {"model_name": "gpt-4", "request_count": 150, "total_tokens": 50000, "success_rate": 0.98}
  ]
}
```


#### 3.2 Agent Activity Endpoint

- **Endpoint:** `GET /api/v1/telemetry/agent-activity`
- **Handler:** New `get_agent_activity_handler`
- **Query:** Aggregate from `telemetry_agent_activity` table
- **Response Format:**
```json
{
  "activity": [
    {"agent_id": "uuid", "activity_type": "task_execution", "count": 25, "timestamp": "..."}
  ]
}
```


#### 3.3 Task Stats History Endpoint

- **Endpoint:** `GET /api/v1/tasks/stats/history`
- **Handler:** New `get_task_stats_history_handler`
- **Query:** Select from `task_stats_history` ORDER BY `snapshot_date DESC`

### Milestone 4: Data Collection Integration Points

#### 4.1 LLM Request Logging

- **Location:** Agent workers when making LLM API calls
- **Action:** Insert into `telemetry_model_contributions` after each LLM call
- **Fields:** model_name, tokens_used, response_time_ms, success/failure

#### 4.2 Agent Activity Logging

- **Location:** [UnifiedOrchestratorAdapter](iterations/v3/data-interfaces-adapters/src/unified_orchestrator_adapter.rs)
- **Action:** Insert into `telemetry_agent_activity` on task start/complete/fail
- **Events:** task_started, task_completed, task_failed, task_cancelled

#### 4.3 Daily Stats Snapshot Job

- **Option A:** Background task in API server (using tokio::spawn)
- **Option B:** Database trigger/function that runs daily
- **Action:** Insert daily aggregate from tasks table into `task_stats_history`

### Milestone 5: Agent Health Monitoring (Future)

#### 5.1 Agent Health Endpoint (Not in initial scope)

- **Endpoint:** `GET /api/v1/agents/:id/health`
- **Requires:** Periodic health check service
- **Complexity:** High - requires agent ping/response infrastructure

#### 5.2 Agent Metrics Endpoint (Not in initial scope)

- **Endpoint:** `GET /api/v1/agents/:id/metrics`
- **Requires:** Resource monitoring of agent processes
- **Complexity:** High - requires process-level monitoring

## Implementation Order

| Priority | Item | Effort | Impact |

|----------|------|--------|--------|

| 1 | Fix project task stats | Low | High |

| 2 | Create telemetry migration | Low | Medium |

| 3 | Model contributions endpoint | Medium | High |

| 4 | Agent activity endpoint | Medium | High |

| 5 | Task stats history endpoint | Medium | Medium |

| 6 | LLM request logging hook | Medium | High |

| 7 | Agent activity logging hook | Medium | High |

| 8 | Daily stats snapshot job | Medium | Medium |

## Files to Modify

1. [api-server.rs](iterations/v3/data-interfaces-adapters/src/bin/api-server.rs) - Add new handlers
2. [orchestrator.rs](iterations/v3/data-infrastructure/src/client/orchestrator.rs) - Add telemetry queries
3. New migration file in [migrations/](iterations/v3/data-infrastructure/migrations/)
4. [unified_orchestrator_adapter.rs](iterations/v3/data-interfaces-adapters/src/unified_orchestrator_adapter.rs) - Add activity logging

## Questions Before Proceeding

1. Should we implement the daily stats snapshot as a background task in the API server, or as a separate cron job/database function?

- daily snapshots as the server is active. Basically a check (have we ran a snapshot recently? no, then run snapshot of data)

2. For LLM request logging, where are the agent workers making LLM calls? Is there a central location to hook into?

- I honestly am not sure, but we have the orchestrator, the council judges, the worker pleading, the way they reason. These should be exposed at some level for research purposes so we can look at the traces and evaluate what's working well and what can be improved.

3. Do we want to start with just the quick wins (project task stats fix) or proceed with the full telemetry infrastructure?

- full telemetry structure. We don't want to have to come back and realize "oh this would have been great if we had this"

### To-dos

- [ ] Fix GET /api/v1/projects/:id/tasks/stats to filter by project_id
- [ ] Create migration 030 for telemetry tracking tables
- [ ] Implement GET /api/v1/telemetry/model-contributions handler
- [ ] Implement GET /api/v1/telemetry/agent-activity handler
- [ ] Implement GET /api/v1/tasks/stats/history handler
- [ ] Add LLM request logging to agent workers
- [ ] Add agent activity logging to orchestrator
- [ ] Implement daily task stats snapshot job