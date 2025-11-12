# Dashboard API Gap Analysis

Prioritized list of missing endpoints, mismatched paths, and type mismatches with implementation recommendations.

## Priority Levels

- **P0**: Critical for core functionality (dashboard, agent stats, projects)
- **P1**: Important for feature completeness (health, analytics)
- **P2**: Nice-to-have (testing, rules, phase planner)

## Critical Issues (P0)

### 1. Settings API Path Mismatch

**Issue**: All settings API functions use `/api/v1` instead of `/api/proxy/api/v1`, which may break if the proxy doesn't handle both paths.

**Affected Files**:
- `apps/agent_management_dashboard/src/lib/api/settings.ts`

**Impact**: Settings page may not work correctly if proxy doesn't forward `/api/v1` paths.

**Recommendation**: 
- Update `API_BASE` constant in `settings.ts` from `/api/v1` to `/api/proxy/api/v1`
- Or verify that the proxy route handles both `/api/v1` and `/api/proxy/api/v1` paths

**Files to Update**:
```typescript
// apps/agent_management_dashboard/src/lib/api/settings.ts
const API_BASE = '/api/proxy/api/v1'; // Change from '/api/v1'
```

**Estimated Effort**: 5 minutes

---

### 2. Observability Efficiency Endpoint Mismatch

**Issue**: `ServerEfficiencyChart` component calls `getEfficiencyMetrics()` which uses `/api/v1/observability/efficiency`, but this endpoint returns a different format than expected.

**Current Endpoint Response**:
```json
{
  "metrics": [
    {
      "metric": "efficiency_score",
      "average": 75.5,
      "min": 60.0,
      "max": 90.0
    }
  ],
  "period": "24 hours"
}
```

**Expected Format** (from `observability.ts`):
```typescript
interface EfficiencyMetrics {
  agent_id?: string;
  efficiency_score: number;
  resource_utilization: number;
  throughput: number;
  timestamp: string;
}
```

**Correct Endpoint**: `/api/v1/agents/efficiency` returns the correct format:
```json
{
  "agents": [
    {
      "agent_id": "...",
      "agent_name": "...",
      "efficiency_score": 75.5,
      "tasks_per_hour": 10.5,
      ...
    }
  ],
  "summary": {...},
  "period_hours": 24
}
```

**Affected Files**:
- `apps/agent_management_dashboard/src/lib/api/observability.ts`
- `apps/agent_management_dashboard/src/components/ServerEfficiencyChart.tsx`

**Impact**: ServerEfficiencyChart displays incorrect or no data.

**Recommendation**:
- Update `getEfficiencyMetrics()` in `observability.ts` to use `/api/v1/agents/efficiency` instead
- Update the function to return `EfficiencyMetrics[]` by extracting from the `agents` array
- Or create a new function `getAgentsEfficiency()` that uses the correct endpoint

**Implementation**:
```typescript
// Option 1: Update existing function
export async function getEfficiencyMetrics(agentId?: string): Promise<EfficiencyMetrics[]> {
  const queryParams = new URLSearchParams();
  if (agentId) queryParams.append('agent_id', agentId);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/agents/efficiency${queryString ? `?${queryString}` : ''}`;
  const response = await apiGet<EfficiencyResponse>(url);
  
  // Transform to expected format
  return response.agents.map(agent => ({
    agent_id: agent.agent_id,
    efficiency_score: agent.efficiency_score,
    resource_utilization: 0, // Not available in response
    throughput: agent.tasks_per_hour,
    timestamp: response.period_end,
  }));
}

// Option 2: Use agents API directly
// ServerEfficiencyChart should use getEfficiencyMetrics from agents.ts instead
```

**Estimated Effort**: 15 minutes

---

## Important Issues (P1)

### 3. Project Overview Versions Missing

**Issue**: `getProjectOverviewVersions()` and `restoreProjectOverviewVersion()` have no backend implementation.

**Affected Files**:
- `apps/agent_management_dashboard/src/lib/api/projects.ts`

**Impact**: Project overview version history feature doesn't work.

**Recommendation**:
- Implement backend endpoints:
  - `GET /api/v1/projects/:project_id/overview-versions` - List overview versions
  - `POST /api/v1/projects/:project_id/overview-versions` - Create new version
  - `POST /api/v1/projects/:project_id/overview-versions/:version_id/restore` - Restore version

**Backend Implementation Needed**:
```rust
// Add to api-server.rs routes
.route("/api/v1/projects/:project_id/overview-versions", get(get_project_overview_versions_handler))
.route("/api/v1/projects/:project_id/overview-versions", post(create_project_overview_version_handler))
.route("/api/v1/projects/:project_id/overview-versions/:version_id/restore", post(restore_project_overview_version_handler))

// Database table needed:
// CREATE TABLE project_overview_versions (
//   version_id UUID PRIMARY KEY,
//   project_id UUID REFERENCES projects(project_id),
//   overview TEXT NOT NULL,
//   created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//   created_by TEXT,
//   change_summary TEXT
// );
```

**Estimated Effort**: 2-3 hours

---

### 4. Project Task Settings Endpoint Verification

**Issue**: `getProjectTaskSettings()` calls `/api/v1/projects/:project_id/task-settings` which may not exist.

**Affected Files**:
- `apps/agent_management_dashboard/src/lib/api/projects.ts`

**Impact**: Project task settings feature may not work.

**Recommendation**:
- Verify if endpoint exists in backend
- If missing, either:
  - Implement the endpoint
  - Or use `/api/v1/projects/:project_id/settings` with a `type=task` query parameter

**Estimated Effort**: 30 minutes (verification) + 1 hour (implementation if needed)

---

### 5. Compliance Stats Client-Side Calculation

**Issue**: `getComplianceStats()` calculates stats client-side by fetching all rules and violations.

**Affected Files**:
- `apps/agent_management_dashboard/src/lib/api/rules.ts`

**Impact**: Performance issues with large datasets, unnecessary data transfer.

**Recommendation**:
- Add backend endpoint: `GET /api/v1/rules/compliance-stats`
- Return aggregated statistics without fetching all data

**Backend Implementation**:
```rust
// Add to api-server.rs routes
.route("/api/v1/rules/compliance-stats", get(get_compliance_stats_handler))

// Handler should return:
// {
//   "total_rules": 50,
//   "active_rules": 45,
//   "total_violations": 120,
//   "open_violations": 15,
//   "resolved_violations": 105,
//   "compliance_score": 85.5,
//   "violations_by_severity": {...},
//   "violations_by_rule": [...]
// }
```

**Estimated Effort**: 1-2 hours

---

## Nice-to-Have Issues (P2)

### 6. Code Contribution Daily Breakdown

**Issue**: `CodeContributionChart` uses client-side aggregation to create daily breakdown from aggregate contribution data.

**Impact**: Less accurate daily breakdown, potential performance issues.

**Recommendation**:
- Add optional `group_by=day` query parameter to `/api/v1/telemetry/contributions`
- Return daily breakdown from backend for better accuracy

**Estimated Effort**: 2-3 hours

---

### 7. Model Contribution Monthly Breakdown

**Issue**: `ModelContributionStream` uses client-side aggregation to create monthly breakdown from aggregate model contribution data.

**Impact**: Less accurate monthly breakdown, potential performance issues.

**Recommendation**:
- Add optional `group_by=month` query parameter to `/api/v1/telemetry/model-contributions`
- Return monthly breakdown from backend for better accuracy

**Estimated Effort**: 2-3 hours

---

### 8. Task Completion Historical Trends

**Issue**: `TaskCompletionGauge` mentions TODO for month-over-month comparison but no endpoint exists.

**Impact**: Cannot show trend improvements over time.

**Recommendation**:
- Add endpoint: `GET /api/v1/tasks/stats/history?period=30d`
- Return time-series data for completion rates

**Estimated Effort**: 2-3 hours

---

## Type Mismatches

### 1. Observability Efficiency Response

**File**: `apps/agent_management_dashboard/src/lib/api/observability.ts`

**Issue**: `EfficiencyMetrics` interface doesn't match actual endpoint response.

**Current Interface**:
```typescript
export interface EfficiencyMetrics {
  agent_id?: string;
  efficiency_score: number;
  resource_utilization: number;
  throughput: number;
  timestamp: string;
}
```

**Actual Response** (from `/api/v1/observability/efficiency`):
```json
{
  "metrics": [
    {
      "metric": "efficiency_score",
      "average": 75.5,
      "min": 60.0,
      "max": 90.0
    }
  ],
  "period": "24 hours"
}
```

**Recommendation**: Use `/api/v1/agents/efficiency` endpoint which matches the expected format better, or update the interface to match the actual response.

---

## Missing Query Parameters

### 1. Agent Activity Time Range

**Current**: `getAgentActivity()` supports `start_date` and `end_date` as date strings.

**Enhancement**: Consider adding `hours` parameter for consistency with other endpoints.

**Impact**: Low - current implementation works fine.

---

### 2. Contributions Time Range

**Current**: `getContributions()` supports `start_date` and `end_date` as date strings.

**Enhancement**: Consider adding `hours` parameter for consistency with other endpoints.

**Impact**: Low - current implementation works fine.

---

## Implementation Roadmap

### Phase 1: Critical Fixes (P0) - 20 minutes
1. ✅ Fix Settings API path mismatch
2. ✅ Fix Observability efficiency endpoint mismatch

### Phase 2: Important Features (P1) - 4-6 hours
3. Implement project overview versions endpoints
4. Verify/implement project task settings endpoint
5. Add compliance stats endpoint

### Phase 3: Enhancements (P2) - 6-9 hours
6. Add daily breakdown to contributions endpoint
7. Add monthly breakdown to model contributions endpoint
8. Add task completion history endpoint

## Testing Recommendations

1. **Settings API**: Test all settings endpoints through proxy to ensure paths work correctly
2. **Efficiency Metrics**: Verify `ServerEfficiencyChart` displays correct data after fix
3. **Project Versions**: Test version creation, listing, and restoration workflows
4. **Compliance Stats**: Load test with large number of rules/violations to verify performance improvement

## Documentation Updates Needed

1. Update API documentation to reflect correct endpoint paths
2. Document new endpoints (project versions, compliance stats)
3. Update frontend API client documentation with correct usage examples

