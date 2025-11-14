# API Provider Documentation

**Author:** @darianrosebrook  
**Date:** 2025-01-28

## Overview

The API Provider provides centralized access to all API endpoints through a React Context provider. This ensures:

1. **No Hard-coded Endpoints** - All API calls go through the provider
2. **Type Safety** - TypeScript types for all requests/responses
3. **Runtime Validation** - Zod schemas validate all API data
4. **Consistent Error Handling** - Standardized error handling across all endpoints
5. **Easy Maintenance** - Single source of truth for API access

## Usage

### Setup

Wrap your app with the `ApiProvider`:

```tsx
import { ApiProvider } from '@/lib/providers/ApiProvider';

function App() {
  return (
    <ApiProvider>
      <YourApp />
    </ApiProvider>
  );
}
```

### Using the API

```tsx
import { useApi } from '@/lib/providers/ApiProvider';

function MyComponent() {
  const api = useApi();

  // Access specific API modules
  const tasks = api.tasks;
  const agents = api.agents;
  const chat = api.chat;

  // Use API methods
  const handleSubmit = async () => {
    const result = await tasks.submitTask({
      description: 'Fix bug',
      priority: 'high',
    });
  };
}
```

### Using Specific Modules

```tsx
import { useApiModule } from '@/lib/providers/ApiProvider';

function MyComponent() {
  const tasks = useApiModule('tasks');
  const agents = useApiModule('agents');

  // Use module methods directly
  const allTasks = await tasks.listTasks();
  const allAgents = await agents.getAgents();
}
```

## API Modules

### Health
- `api.health.getHealth()` - Basic health check
- `api.health.getSystemHealth()` - Detailed system health

### Tasks
- `api.tasks.listTasks()` - List all tasks
- `api.tasks.submitTask()` - Submit new task
- `api.tasks.getTaskStatus()` - Get task status
- `api.tasks.getTasksStats()` - Get task statistics
- `api.tasks.getTaskLogs()` - Get task logs
- `api.tasks.getTaskProgress()` - Get task progress
- And more...

### Chat
- `api.chat.listSessions()` - List chat sessions
- `api.chat.createSession()` - Create chat session
- `api.chat.getMessages()` - Get chat messages
- `api.chat.sendMessage()` - Send chat message

### Authentication
- `api.auth.login()` - User login
- `api.auth.logout()` - User logout
- `api.auth.refreshToken()` - Refresh token
- `api.auth.getCurrentUser()` - Get current user

### Agents
- `api.agents.getAgents()` - List all agents
- `api.agents.getAgentStats()` - Get agent statistics
- `api.agents.getAgentHealth()` - Get agent health
- `api.agents.getAgentMetrics()` - Get agent metrics
- And more...

### Judges
- `api.judges.listJudges()` - List all judges
- `api.judges.getJudgeStats()` - Get judge statistics
- `api.judges.getJudgeEvaluations()` - Get judge evaluations

### Observability
- `api.observability.getEfficiencyMetrics()` - Get efficiency metrics
- `api.observability.getSystemMetrics()` - Get system metrics
- `api.observability.getAlerts()` - Get system alerts

### System
- `api.system.getSystemHealth()` - Get system health
- `api.system.getSystemResources()` - Get system resources
- `api.system.getSystemMetrics()` - Get system metrics

### Sessions
- `api.sessions.getSessionStatus()` - Get session status
- `api.sessions.pauseSession()` - Pause session
- `api.sessions.resumeSession()` - Resume session
- `api.sessions.cancelSession()` - Cancel session

### Search & Queries
- `api.search.search()` - Global search
- `api.queries.listQueries()` - List saved queries
- `api.queries.saveQuery()` - Save query
- `api.queries.deleteQuery()` - Delete query

### Query Performance
- `api.queryPerformance.getQueryPerformanceSummary()` - Get summary
- `api.queryPerformance.getSlowQueries()` - Get slow queries
- `api.queryPerformance.getTopSlowQueries()` - Get top slow queries

### Provenance
- `api.provenance.listProvenance()` - List provenance records
- `api.provenance.linkProvenance()` - Link provenance to task
- `api.provenance.verifyProvenance()` - Verify provenance

### Waivers
- `api.waivers.listWaivers()` - List waivers
- `api.waivers.createWaiver()` - Create waiver
- `api.waivers.approveWaiver()` - Approve waiver

### SLOs
- `api.slos.listSLOs()` - List SLOs
- `api.slos.getSLOStatus()` - Get SLO status
- `api.slos.getSLOMeasurements()` - Get SLO measurements
- `api.slos.listSLOAlerts()` - List SLO alerts

### Projects
- `api.projects.listProjects()` - List all projects
- `api.projects.getProject()` - Get project details
- `api.projects.getProjectTasks()` - Get project tasks

### Database
- `api.database.listDatabaseTables()` - List database tables
- `api.database.getTableSchema()` - Get table schema
- `api.database.executeQuery()` - Execute database query

### Analytics
- `api.analytics.getTaskAnalytics()` - Get task analytics
- `api.analytics.getPerformanceAnalytics()` - Get performance analytics
- `api.analytics.getSuccessRates()` - Get success rates

## Type Safety

All API methods are fully typed with TypeScript. Request and response types are defined using Zod schemas and inferred TypeScript types.

## Validation

All API requests and responses are validated using Zod schemas at runtime. This ensures:

- Request data matches expected format
- Response data matches expected format
- Type errors are caught early
- API contract is enforced

## Error Handling

All API calls use consistent error handling:

- Errors are automatically parsed from API responses
- Toast notifications are shown for errors (configurable)
- Retry logic is available for retryable errors
- Custom error handlers can be provided

## Migration Guide

### Replacing Direct API Calls

**Before:**
```tsx
const response = await fetch('/api/proxy/api/v1/tasks');
const data = await response.json();
```

**After:**
```tsx
const api = useApi();
const data = await api.tasks.listTasks();
```

### Replacing Hard-coded URLs

**Before:**
```tsx
const url = `${process.env.NEXT_PUBLIC_API_URL}/api/v1/agents`;
```

**After:**
```tsx
const api = useApi();
const agents = await api.agents.getAgents();
```

### Replacing Mock APIs

**Before:**
```tsx
// Mock implementation
const data = simulateData();
```

**After:**
```tsx
const api = useApi();
const data = await api.tasks.listTasks();
```

## Best Practices

1. **Always use the provider** - Never make direct fetch calls
2. **Use TypeScript types** - Leverage the provided types
3. **Validate with Zod** - All schemas are provided
4. **Handle errors** - Use try/catch or error callbacks
5. **Use loading states** - Track loading state for better UX
6. **Cache when appropriate** - Use React Query or similar for caching

## Implementation Status

See `TODOS_API_INTEGRATION.md` for the current implementation status of all endpoints.

