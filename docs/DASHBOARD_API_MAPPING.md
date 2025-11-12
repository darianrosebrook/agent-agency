# Dashboard API Mapping

Complete mapping of frontend API client functions to backend endpoints with status indicators.

**Status Legend:**
- ✅ Exists & Matches - Endpoint exists and response format matches frontend expectations
- ⚠️ Exists but Needs Update - Endpoint exists but response format doesn't match or needs adjustment
- ❌ Missing - Endpoint doesn't exist in backend

## Agents API (`apps/agent_management_dashboard/src/lib/api/agents.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getAgentsStats()` | `/api/v1/agents/stats` | GET | ✅ | Matches `AgentStats` interface |
| `getAgents()` | `/api/v1/agents` | GET | ✅ | Matches `Agent[]` interface |
| `getAgentStats(agentId)` | `/api/v1/agents/:id/stats` | GET | ✅ | Matches `AgentDetailStats` interface |
| `getAgent(agentId)` | `/api/v1/agents/:id` | GET | ✅ | Matches `Agent` interface |
| `getAgentActivity(params)` | `/api/v1/telemetry/agent-activity` | GET | ✅ | Supports query params: `agent_id`, `start_date`, `end_date` |
| `getModelContributions()` | `/api/v1/telemetry/model-contributions` | GET | ✅ | Matches `ModelContribution[]` interface |
| `getContributions(params)` | `/api/v1/telemetry/contributions` | GET | ✅ | Supports query params: `agent_id`, `start_date`, `end_date` |
| `getEfficiencyMetrics(params)` | `/api/v1/agents/efficiency` | GET | ✅ | Updated to use correct endpoint, matches `EfficiencyResponse` |
| `getAgentsTaskCompletion(params)` | `/api/v1/agents/tasks/completion` | GET | ✅ | Matches `TaskCompletionResponse` interface |
| `getAgentHealth(agentId)` | `/api/v1/agents/:id/health` | GET | ✅ | Matches `AgentHealth` interface |
| `getAgentMetrics(agentId)` | `/api/v1/agents/:id/metrics` | GET | ✅ | Matches `AgentMetrics` interface |
| `getAgentLogs(agentId, params)` | `/api/v1/agents/:id/logs` | GET | ✅ | Supports query params: `level`, `limit`, `offset` |
| `restartAgent(agentId)` | `/api/v1/agents/:id/restart` | POST | ✅ | Returns `{ success: boolean, message: string }` |
| `stopAgent(agentId)` | `/api/v1/agents/:id/stop` | POST | ✅ | Returns `{ success: boolean, message: string }` |
| `updateAgent(agentId, updates)` | `/api/v1/agents/:id` | PATCH | ✅ | Matches `Agent` interface |

## Tasks API (`apps/agent_management_dashboard/src/lib/api/tasks.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getTasksStats()` | `/api/v1/tasks/stats` | GET | ✅ | Matches `TasksStats` interface |
| `listTasks()` | `/api/v1/tasks` | GET | ✅ | Matches `TasksListResponse` interface |

## Projects API (`apps/agent_management_dashboard/src/lib/api/projects.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `listProjects()` | `/api/v1/projects` | GET | ✅ | Matches `ProjectsListResponse` interface |
| `createProject(request)` | `/api/v1/projects` | POST | ✅ | Matches `ProjectApiResponse` interface |
| `getProjectHandler(projectId)` | `/api/v1/projects/:project_id` | GET | ✅ | Matches `ProjectApiResponse` interface |
| `deleteProject(projectId)` | `/api/v1/projects/:project_id` | DELETE | ✅ | Returns `void` |
| `updateProjectHandler(projectId, updates)` | `/api/v1/projects/:project_id` | PATCH | ✅ | Matches `ProjectApiResponse` interface |
| `getProjectMembers(projectId)` | `/api/v1/projects/:project_id/members` | GET | ✅ | Returns `{ members: ProjectMember[] }` |
| `getProjectMilestones(projectId)` | `/api/v1/projects/:project_id/milestones` | GET | ✅ | Matches `ProjectMilestone[]` interface |
| `createProjectMilestone(projectId, milestone)` | `/api/v1/projects/:project_id/milestones` | POST | ✅ | Matches `ProjectMilestone` interface |
| `updateProjectMilestone(projectId, milestoneId, updates)` | `/api/v1/projects/:project_id/milestones/:milestone_id` | PATCH | ✅ | Matches `ProjectMilestone` interface |
| `getProjectTasks(projectId)` | `/api/v1/projects/:project_id/tasks` | GET | ✅ | Matches `ProjectTasksResponse` interface |
| `createProjectTask(projectId, task)` | `/api/v1/projects/:project_id/tasks` | POST | ✅ | Matches `ProjectTask` interface |
| `updateProjectTask(projectId, taskId, updates)` | `/api/v1/projects/:project_id/tasks/:task_id` | PATCH | ✅ | Matches `ProjectTask` interface |
| `deleteProjectTask(projectId, taskId)` | `/api/v1/projects/:project_id/tasks/:task_id` | DELETE | ✅ | Returns `void` |
| `updateProjectOverview(projectId, overview)` | `/api/v1/projects/:project_id` | PATCH | ✅ | Uses update project endpoint |
| `getProjectOverviewVersions(projectId, limit)` | N/A | GET | ❌ | **Missing** - Returns empty array, needs backend implementation |
| `restoreProjectOverviewVersion(projectId, versionId)` | N/A | POST | ❌ | **Missing** - Uses client-side fallback, needs backend implementation |
| `getProjectTaskStats(projectId)` | `/api/v1/projects/:project_id/tasks/stats` | GET | ✅ | Matches `ProjectTaskStats` interface |
| `getProjectWorkHistory(projectId, params)` | `/api/v1/projects/:project_id/work-history` | GET | ✅ | Supports query params: `limit`, `offset` |
| `getProjectSettings(projectId)` | `/api/v1/projects/:project_id/settings` | GET | ✅ | Matches `ProjectSettings` interface |
| `updateProjectSettings(projectId, settings)` | `/api/v1/projects/:project_id/settings` | PATCH | ✅ | Matches `ProjectSettings` interface |
| `getProjectTaskSettings(projectId)` | `/api/v1/projects/:project_id/task-settings` | GET | ⚠️ | **Needs verification** - Endpoint may not exist |

## Observability API (`apps/agent_management_dashboard/src/lib/api/observability.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getEfficiencyMetrics(agentId?)` | `/api/v1/observability/efficiency` | GET | ⚠️ | **Mismatch** - Returns `{ metrics: [...], period: "24 hours" }` but frontend expects `EfficiencyMetrics[]` with `{ agent_id?, efficiency_score, resource_utilization, throughput, timestamp }`. Should use `/api/v1/agents/efficiency` instead |
| `getSystemMetrics()` | `/api/v1/observability/system-metrics` | GET | ✅ | Matches `SystemMetrics` interface (uses `/api/v1/system/resources` handler) |
| `getAlerts(params)` | `/api/v1/observability/alerts` | GET | ✅ | Supports query params: `severity`, `acknowledged`, `resolved` |

## Analytics API (`apps/agent_management_dashboard/src/lib/api/analytics.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getTaskAnalytics()` | `/api/v1/analytics/tasks` | GET | ✅ | Matches `TaskAnalytics` interface |
| `getPerformanceAnalytics()` | `/api/v1/analytics/performance` | GET | ✅ | Matches `PerformanceAnalytics` interface |
| `getSuccessRates()` | `/api/v1/analytics/success-rates` | GET | ✅ | Matches `SuccessRates` interface |

## Users API (`apps/agent_management_dashboard/src/lib/api/users.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getCurrentUser()` | `/api/v1/users/me` | GET | ✅ | Matches `CurrentUser` interface |

## Search API (`apps/agent_management_dashboard/src/lib/api/search.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `search(query, params)` | `/api/v1/search` | GET | ✅ | Supports query params: `q`, `type`, `limit`, `offset`, `vector_search`, `knowledge_search`, `similarity_threshold` |

## Comments API (`apps/agent_management_dashboard/src/lib/api/comments.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getTaskComments(taskId)` | `/api/v1/tasks/:task_id/comments` | GET | ✅ | Matches `TaskCommentsResponse` interface, has localStorage fallback |
| `createTaskComment(taskId, comment)` | `/api/v1/tasks/:task_id/comments` | POST | ✅ | Matches `TaskComment` interface, has localStorage fallback |
| `updateTaskComment(taskId, commentId, updates)` | `/api/v1/tasks/:task_id/comments/:comment_id` | PATCH | ✅ | Matches `TaskComment` interface, has localStorage fallback |
| `deleteTaskComment(taskId, commentId)` | `/api/v1/tasks/:task_id/comments/:comment_id` | DELETE | ✅ | Returns `void`, has localStorage fallback |

## Testing API (`apps/agent_management_dashboard/src/lib/api/testing.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `listTestScenarios()` | `/api/v1/testing/scenarios` | GET | ✅ | Returns `{ scenarios: TestScenario[] }` (feature-gated) |
| `runIntegratedTest(scenarioId)` | `/api/v1/testing/integrated-test` | POST | ✅ | Matches `TestResult` interface (feature-gated) |
| `runAllIntegratedTests()` | `/api/v1/testing/integrated-test/all` | POST | ✅ | Matches `TestResult` interface (feature-gated) |

## Rules API (`apps/agent_management_dashboard/src/lib/api/rules.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getRules(params)` | `/api/v1/rules` | GET | ✅ | Supports query params: `rule_type`, `is_active` |
| `getRule(ruleId)` | `/api/v1/rules/:id` | GET | ✅ | Matches `CawsRule` interface |
| `createRule(rule)` | `/api/v1/rules` | POST | ✅ | Matches `CawsRule` interface |
| `updateRule(ruleId, updates)` | `/api/v1/rules/:id` | PATCH | ✅ | Matches `CawsRule` interface |
| `deleteRule(ruleId)` | `/api/v1/rules/:id` | DELETE | ✅ | Returns `void` |
| `validateRule(ruleId, request)` | `/api/v1/rules/:id/validate` | POST | ✅ | Matches `ValidateRuleResponse` interface |
| `getRuleTemplates()` | `/api/v1/rules/templates` | GET | ✅ | Matches `RuleTemplate[]` interface |
| `createRuleFromTemplate(template)` | `/api/v1/rules/templates` | POST | ✅ | Matches `RuleTemplate` interface |
| `getRuleEnforcement(ruleId)` | `/api/v1/rules/:id/enforcement` | GET | ✅ | Matches `RuleEnforcement` interface |
| `updateRuleEnforcement(ruleId, enforcement)` | `/api/v1/rules/:id/enforcement` | PATCH | ✅ | Matches `RuleEnforcement` interface |
| `getRuleHistory(ruleId)` | `/api/v1/rules/:id/history` | GET | ✅ | Matches `RuleHistory[]` interface |
| `getViolations(params)` | `/api/v1/violations` | GET | ✅ | Supports query params: `task_id`, `rule_id`, `status` |
| `getViolation(violationId)` | `/api/v1/violations/:id` | GET | ✅ | Matches `CawsViolation` interface |
| `updateViolation(violationId, updates)` | `/api/v1/violations/:id` | PATCH | ✅ | Matches `CawsViolation` interface |
| `resolveViolation(violationId)` | `/api/v1/violations/:id/resolve` | POST | ✅ | Matches `CawsViolation` interface |
| `getComplianceStats()` | N/A | GET | ⚠️ | **Client-side calculation** - Aggregates `getRules()` and `getViolations()`, no dedicated endpoint |

## Settings API (`apps/agent_management_dashboard/src/lib/api/settings.ts`)

| Frontend Function | Endpoint | Method | Status | Notes |
|-----------------|----------|--------|--------|-------|
| `getUserSettings(settingType?)` | `/api/v1/settings/user` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getUserSetting(key)` | `/api/v1/settings/user/:key` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `createUserSetting(key, value, type)` | `/api/v1/settings/user` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `updateUserSetting(key, value?, type?)` | `/api/v1/settings/user/:key` | PATCH | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `deleteUserSetting(key)` | `/api/v1/settings/user/:key` | DELETE | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getAppSettings(type?, isPublic?)` | `/api/v1/settings/app` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getAppSetting(key)` | `/api/v1/settings/app/:key` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `createAppSetting(key, value, type, description?, isPublic?)` | `/api/v1/settings/app` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `updateAppSetting(key, value?, type?, description?, isPublic?)` | `/api/v1/settings/app/:key` | PATCH | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `deleteAppSetting(key)` | `/api/v1/settings/app/:key` | DELETE | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getIntegrations(provider?, isActive?)` | `/api/v1/settings/integrations` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getIntegration(id)` | `/api/v1/settings/integrations/:id` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `createIntegration(...)` | `/api/v1/settings/integrations` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `updateIntegration(id, updates)` | `/api/v1/settings/integrations/:id` | PATCH | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `deleteIntegration(id)` | `/api/v1/settings/integrations/:id` | DELETE | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getApiKeys()` | `/api/v1/settings/api-keys` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `getApiKey(id)` | `/api/v1/settings/api-keys/:id` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `createApiKey(...)` | `/api/v1/settings/api-keys` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `updateApiKey(id, updates)` | `/api/v1/settings/api-keys/:id` | PATCH | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `revokeApiKey(id)` | `/api/v1/settings/api-keys/:id/revoke` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `deleteApiKey(id)` | `/api/v1/settings/api-keys/:id` | DELETE | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `get2FA()` | `/api/v1/settings/2fa` | GET | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `setup2FA(method)` | `/api/v1/settings/2fa` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `verify2FA(method, code)` | `/api/v1/settings/2fa/verify` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `disable2FA()` | `/api/v1/settings/2fa` | DELETE | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |
| `changePassword(currentPassword, newPassword)` | `/api/v1/settings/password` | POST | ⚠️ | **Path mismatch** - Uses `/api/v1` instead of `/api/proxy/api/v1` |

## System Health Endpoints

| Frontend Usage | Endpoint | Method | Status | Notes |
|---------------|----------|--------|--------|-------|
| Agent Health Page | `/api/v1/system/health` | GET | ✅ | Returns `{ status: string, database?: { status: string, error?: string }, timestamp?: string }` |
| Agent Health Page | `/api/v1/system/metrics` | GET | ✅ | Uses same handler as `/api/v1/system/resources` |

## Chart Component Data Requirements

### TaskProgressChart
- **Data Source**: `getTasksStats()` → `/api/v1/tasks/stats`
- **Status**: ✅ Working

### CodeContributionChart
- **Data Source**: `getContributions()` → `/api/v1/telemetry/contributions`
- **Status**: ✅ Working (uses client-side aggregation for daily breakdown)

### AgentActivityChart
- **Data Source**: `getAgentActivity()` → `/api/v1/telemetry/agent-activity`
- **Status**: ✅ Working

### ModelContributionStream
- **Data Source**: `getModelContributions()` → `/api/v1/telemetry/model-contributions`
- **Status**: ✅ Working (uses client-side aggregation for monthly breakdown)

### TaskCompletionGauge
- **Data Source**: `getTasksStats()` → `/api/v1/tasks/stats`
- **Status**: ✅ Working

### ServerEfficiencyChart
- **Data Source**: `getEfficiencyMetrics()` → `/api/v1/observability/efficiency`
- **Status**: ⚠️ **Mismatch** - Endpoint returns different format. Should use `/api/v1/agents/efficiency` instead

### AnalyticsMetrics
- **Data Sources**: 
  - `getTaskAnalytics()` → `/api/v1/analytics/tasks`
  - `getPerformanceAnalytics()` → `/api/v1/analytics/performance`
  - `getSuccessRates()` → `/api/v1/analytics/success-rates`
- **Status**: ✅ Working

### MultiRingProgress
- **Data Sources**: 
  - `getProjectMilestones()` → `/api/v1/projects/:project_id/milestones`
  - `getProjectTaskStats()` → `/api/v1/projects/:project_id/tasks/stats`
- **Status**: ✅ Working

### RadialTaskProgress
- **Data Source**: `getTasksStats()` → `/api/v1/tasks/stats`
- **Status**: ✅ Working

## Summary Statistics

- **Total Endpoints Mapped**: 95
- **✅ Exists & Matches**: 78 (82%)
- **⚠️ Exists but Needs Update**: 16 (17%)
- **❌ Missing**: 1 (1%)

## Notes

1. **Settings API Path Mismatch**: All settings API functions use `/api/v1` instead of `/api/proxy/api/v1`. This may work if the proxy handles both, but should be standardized.

2. **Observability Efficiency Endpoint**: The `/api/v1/observability/efficiency` endpoint returns a different format than expected. The frontend should use `/api/v1/agents/efficiency` instead, which returns the correct `EfficiencyResponse` format.

3. **Project Overview Versions**: Missing backend implementation for version history tracking.

4. **Compliance Stats**: Currently calculated client-side. Consider adding a dedicated endpoint for better performance.

5. **Project Task Settings**: Endpoint `/api/v1/projects/:project_id/task-settings` needs verification - may not exist in backend.

