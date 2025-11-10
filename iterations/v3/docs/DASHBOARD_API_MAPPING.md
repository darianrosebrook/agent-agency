# Dashboard to API Endpoint Mapping

**Author:** @darianrosebrook  
**Date:** 2025-01-28  
**Purpose:** Complete mapping of dashboard pages to API endpoints and identification of missing integrations

---

## Dashboard Pages Analysis

### ✅ Implemented Dashboard Pages

| Page | Route | API Endpoints Used | Status |
|------|-------|-------------------|--------|
| **Dashboard Home** | `/` | `GET /api/v1/system/health`<br>`GET /api/v1/tasks`<br>`GET /api/v1/projects` | ✅ Working |
| **Tasks List** | `/tasks` | `GET /api/v1/tasks` | ✅ Working |
| **Task Detail** | `/tasks/[id]` | `GET /api/v1/tasks/:id`<br>`GET /api/v1/tasks/:id/chain-of-thought`<br>`GET /api/v1/tasks/:id/council-decisions`<br>`GET /api/v1/tasks/:id/worker-actions` | ✅ Working |
| **Projects List** | `/projects` | `GET /api/v1/projects` | ✅ Working |
| **Project Detail** | `/projects/[id]` | `GET /api/v1/projects/:id`<br>`GET /api/v1/projects/:id/tasks` | ✅ Working |
| **Database** | `/database` | `GET /api/v1/database/tables`<br>`GET /api/v1/database/stats` | ✅ Working |
| **Database Table** | `/database/tables/[name]` | `GET /api/v1/database/tables/:name` | ✅ Working |
| **Database Query** | `/database/query` | `POST /api/v1/database/query` | ✅ Working |
| **Provenance** | `/provenance` | `GET /api/v1/provenance`<br>`GET /api/v1/provenance/verify/:hash`<br>`GET /api/v1/provenance/commit/:hash` | ✅ Working |
| **System Health** | `/system/health` | `GET /api/v1/system/health`<br>`GET /api/v1/system/resources` | ✅ Working |
| **System Metrics** | `/system/metrics` | `GET /api/v1/system/metrics` | ✅ Working |
| **System Analytics** | `/system/analytics` | `GET /api/v1/analytics/tasks`<br>`GET /api/v1/analytics/performance`<br>`GET /api/v1/analytics/success-rates` | ✅ Working |

### ❌ Missing Dashboard Pages (No Implementation Found)

| Page | Expected Route | Required API Endpoints | Status |
|------|---------------|----------------------|--------|
| **Agent Stats** | `/agent-stats` | `GET /api/v1/agents`<br>`GET /api/v1/agents/stats`<br>`GET /api/v1/telemetry/contributions`<br>`GET /api/v1/telemetry/model-contributions`<br>`GET /api/v1/telemetry/agent-activity` | ⚠️ Endpoints exist, page missing |
| **Agent Health** | `/agent-health` | `GET /api/v1/agents/:id/health`<br>`GET /api/v1/agents/:id/metrics`<br>`GET /api/v1/observability/alerts` | ⚠️ Endpoints exist, page missing |
| **Settings** | `/settings` | `GET /api/v1/settings/user`<br>`GET /api/v1/settings/app`<br>`GET /api/v1/settings/integrations`<br>`GET /api/v1/settings/api-keys` | ❌ Endpoints missing, page missing |
| **Rules & Governance** | `/rules-governance` | `GET /api/v1/rules`<br>`GET /api/v1/rules/compliance`<br>`GET /api/v1/rules/violations` | ❌ Endpoints missing, page missing |

---

## API Client Files Status

### ✅ Implemented API Clients

| Client File | Endpoints Covered | Status |
|-------------|------------------|--------|
| `src/lib/api/tasks.ts` | 13 endpoints | ✅ Complete |
| `src/lib/api/projects.ts` | 4 endpoints | ✅ Complete |
| `src/lib/api/provenance.ts` | 5 endpoints | ✅ Complete |
| `src/lib/api/analytics.ts` | 3 endpoints | ✅ Complete |
| `src/lib/api/database.ts` | 4 endpoints | ✅ Complete |
| `src/lib/api/system.ts` | 3 endpoints | ✅ Complete |

### ❌ Missing API Clients

| Client File | Required Endpoints | Status |
|-------------|-------------------|--------|
| `src/lib/api/agents.ts` | 13 endpoints | ⚠️ Endpoints exist, client missing |
| `src/lib/api/telemetry.ts` | 6 endpoints | ⚠️ Endpoints exist, client missing |
| `src/lib/api/settings.ts` | 12 endpoints | ❌ Endpoints missing |
| `src/lib/api/rules.ts` | 9 endpoints | ❌ Endpoints missing |
| `src/lib/api/judges.ts` | 7 endpoints | ⚠️ Endpoints exist, client missing |

---

## Endpoint Implementation Verification

### ✅ Fully Implemented Handlers (Verified)

**Task Management (14 handlers):**
- `submit_task_handler` - ✅ Implemented
- `list_tasks_handler` - ✅ Implemented
- `get_task_status_handler` - ✅ Implemented
- `get_task_result_handler` - ✅ Implemented
- `cancel_task_handler` - ✅ Implemented
- `pause_task_handler` - ✅ Implemented
- `resume_task_handler` - ✅ Implemented
- `get_chain_of_thought_handler` - ✅ Implemented (lines 2145-2183)
- `get_council_decisions_handler` - ✅ Implemented (lines 2185-2223)
- `get_worker_actions_handler` - ✅ Implemented (lines 2225-2263)
- `get_task_logs_handler` - ✅ Implemented
- `get_task_progress_handler` - ✅ Implemented
- `get_task_events_handler` - ✅ Implemented
- `update_task_handler` - ✅ Implemented
- `delete_task_handler` - ✅ Implemented

**Agent Management (11 handlers):**
- `list_agents_handler` - ✅ Implemented (line 1215)
- `get_agents_stats_handler` - ✅ Implemented (line 1245)
- `get_agent_handler` - ✅ Implemented (line 1276)
- `update_agent_handler` - ✅ Implemented (line 1309)
- `delete_agent_handler` - ✅ Implemented (line 1347)
- `get_agent_stats_handler` - ✅ Implemented (line 1370)
- `get_agent_health_handler` - ✅ Implemented (line 1422)
- `get_agent_metrics_handler` - ✅ Implemented (line 1448)
- `get_agent_logs_handler` - ✅ Implemented (line 1473)
- `restart_agent_handler` - ✅ Implemented (line 1486)
- `stop_agent_handler` - ✅ Implemented (line 1499)

**Telemetry & Observability (6 handlers):**
- `get_contributions_handler` - ✅ Implemented (line 1841)
- `get_model_contributions_handler` - ✅ Implemented (line 1911)
- `get_agent_activity_handler` - ✅ Implemented (line 1953)
- `get_efficiency_handler` - ✅ Implemented (line 2013)
- `get_system_metrics_handler` - ✅ Implemented (line 2057)
- `get_alerts_handler` - ✅ Implemented (line 2099)

**Authentication (6 handlers):**
- `login_handler` - ✅ Implemented (line 539)
- `logout_handler` - ✅ Implemented (line 540)
- `refresh_token_handler` - ✅ Implemented (line 541)
- `get_current_user_handler` - ✅ Implemented (line 542)
- `request_password_reset_handler` - ✅ Implemented (line 543)
- `confirm_password_reset_handler` - ✅ Implemented (line 544)

---

## Dashboard Integration Completeness

### Core Features: ✅ 100% Connected

- ✅ **Task Management:** Fully integrated (13/13 endpoints)
- ✅ **Project Management:** Fully integrated (4/4 basic endpoints)
- ✅ **System Monitoring:** Fully integrated (6/6 endpoints)
- ✅ **Database Inspection:** Fully integrated (4/4 endpoints)
- ✅ **Provenance Tracking:** Fully integrated (5/5 endpoints)
- ✅ **Analytics:** Fully integrated (3/3 endpoints)
- ✅ **Chain-of-Thought:** Fully integrated (endpoints + components)
- ✅ **Council Decisions:** Fully integrated (endpoints + components)
- ✅ **Worker Actions:** Fully integrated (endpoints + components)

### Advanced Features: ⚠️ Partially Connected

- ⚠️ **Agent Management:** Endpoints exist (11/13), but no dashboard pages
- ⚠️ **Telemetry:** Endpoints exist (6/6), but no dashboard pages
- ⚠️ **Judge Management:** Endpoints exist (7/7), but no dashboard pages

### Missing Features: ❌ Not Connected

- ❌ **Settings Management:** No endpoints (0/12), no dashboard pages
- ❌ **Rules & Governance:** No endpoints (0/9), no dashboard pages
- ❌ **File Operations:** No endpoints (0/5), no dashboard pages
- ❌ **Search:** No endpoint (0/1), no dashboard integration

---

## Critical Finding: API Gap Analysis is Outdated

The `API_GAP_ANALYSIS.md` document incorrectly states:

**Claimed Missing (but actually implemented):**
- ❌ Authentication endpoints - **ACTUALLY EXISTS** (6/6 implemented)
- ❌ Agent management endpoints - **ACTUALLY EXISTS** (11/13 implemented)
- ❌ Telemetry endpoints - **ACTUALLY EXISTS** (6/6 implemented)

**Actual Status:**
- **Total Implemented:** ~65 endpoints (not 45)
- **Total Required:** ~85 endpoints
- **Actual Coverage:** ~76% (not 53%)

**Missing Categories (Accurate):**
- Settings Management: 0/12 endpoints
- Rules & Governance: 0/9 endpoints
- File Operations: 0/5 endpoints
- Search: 0/1 endpoint

---

## Dashboard Component Usage

### ✅ Components Using APIs

**Task Components:**
- `ChainOfThoughtViewer` - Uses `tasksApi.getChainOfThought()`
- `CouncilDecisionsViewer` - Uses `tasksApi.getCouncilDecisions()`
- Task detail page - Uses all task endpoints

**System Components:**
- System health page - Uses `systemApi.getSystemHealth()`
- System metrics page - Uses `systemApi.getSystemMetrics()`
- Analytics page - Uses `analyticsApi.*`

**Database Components:**
- Database page - Uses `databaseApi.listTables()`
- Table schema viewer - Uses `databaseApi.getTableSchema()`
- Query executor - Uses `databaseApi.executeQuery()`

### ❌ Missing Components

- No agent stats components
- No agent health components
- No telemetry visualization components
- No settings components
- No rules & governance components

---

## Integration Roadmap

### Phase 1: Connect Existing Endpoints (1 week)

**Priority:** High - Endpoints exist, just need dashboard integration

1. **Create Agent API Client**
   - File: `src/lib/api/agents.ts`
   - Endpoints: All 11 agent management endpoints
   - Effort: 2-3 hours

2. **Create Telemetry API Client**
   - File: `src/lib/api/telemetry.ts`
   - Endpoints: All 6 telemetry endpoints
   - Effort: 1-2 hours

3. **Create Agent Stats Page**
   - Route: `/agent-stats`
   - Components: Agent list, stats cards, contribution charts
   - Effort: 1-2 days

4. **Create Agent Health Page**
   - Route: `/agent-health`
   - Components: Health dashboard, metrics visualization
   - Effort: 1-2 days

### Phase 2: Implement Missing Endpoints (2-3 weeks)

**Priority:** Medium - Required for full dashboard functionality

1. **Settings Management Endpoints**
   - User settings CRUD
   - App settings CRUD
   - Integration management
   - API key management
   - 2FA endpoints
   - Effort: 1-2 weeks

2. **Rules & Governance Endpoints**
   - Rules CRUD
   - Compliance checking
   - Violation tracking
   - Effort: 1-2 weeks

### Phase 3: Advanced Features (1-2 weeks)

**Priority:** Low - Nice-to-have features

1. **File Operations Endpoints**
2. **Search Endpoint**
3. **Chat Extensions**

---

## Summary

### Current State

**Dashboard Integration:** 76% complete
- ✅ Core features: 100% connected
- ⚠️ Advanced features: Endpoints exist, pages missing
- ❌ Missing features: No endpoints, no pages

**API Coverage:** 76% complete
- ✅ 65 endpoints implemented
- ❌ 20 endpoints missing (primarily settings, rules, files)

**Usability:** Operational for core features
- ✅ Task management fully functional
- ✅ System monitoring fully functional
- ✅ Chain-of-thought visualization working
- ⚠️ Agent management endpoints exist but no UI
- ❌ Settings/rules features blocked

### Next Steps

1. **Immediate:** Create agent and telemetry API clients + dashboard pages
2. **Short-term:** Implement settings management endpoints
3. **Long-term:** Implement rules & governance system

