# API Client Modules

**Author:** @darianrosebrook  
**Date:** 2025-01-28

## Overview

This directory contains all API client modules for interacting with the Agent Agency API. All modules follow a consistent pattern:

1. **Zod Schemas** - Runtime validation for requests/responses
2. **TypeScript Types** - Inferred from Zod schemas
3. **Base API Client** - Uses `apiGet`, `apiPost`, etc. from `base.ts`
4. **Consistent Error Handling** - Automatic error parsing and toast notifications

## Module Structure

Each API module should follow this pattern:

```typescript
import { apiGet, apiPost } from './base';
import { z } from 'zod';

// Request/Response schemas
export const RequestSchema = z.object({ ... });
export type Request = z.infer<typeof RequestSchema>;

// API functions
export async function getSomething(): Promise<Response> {
  return apiGet<Response>('/api/v1/endpoint', {
    responseSchema: ResponseSchema,
  });
}
```

## Available Modules

### Core APIs
- `health.ts` - Health check endpoints
- `auth.ts` - Authentication endpoints
- `tasks.ts` - Task management and observation
- `agents.ts` - Agent management and observation
- `judges.ts` - Judge management
- `chat.ts` - Chat sessions and messages

### System APIs
- `system.ts` - System monitoring
- `observability.ts` - Observability metrics
- `sessions.ts` - Session control
- `analytics.ts` - Analytics and reporting

### Data APIs
- `projects.ts` - Project management
- `database.ts` - Database inspection
- `queries.ts` - Query management
- `queryPerformance.ts` - Query performance monitoring

### Governance APIs
- `provenance.ts` - Code provenance tracking
- `waivers.ts` - Quality gate waivers
- `slos.ts` - Service level objectives

### Utility APIs
- `search.ts` - Global search

## Usage

All API modules are accessed through the `ApiProvider`:

```tsx
import { useApi } from '@/lib/providers/ApiProvider';

function MyComponent() {
  const api = useApi();
  
  // Use any API module
  const tasks = await api.tasks.listTasks();
  const agents = await api.agents.getAgents();
  const health = await api.health.getSystemHealth();
}
```

## Adding New Endpoints

When adding new endpoints to an existing module:

1. **Add Zod Schema** - Define request/response schemas
2. **Add TypeScript Types** - Export inferred types
3. **Add API Function** - Use `apiGet`, `apiPost`, etc.
4. **Add Validation** - Include schemas in options
5. **Update Provider** - Ensure module is exported in ApiProvider

## Validation

All endpoints should use Zod validation:

```typescript
// Request validation
export async function createSomething(data: CreateRequest): Promise<Response> {
  return apiPost<CreateRequest, Response>('/api/v1/endpoint', data, {
    requestSchema: CreateRequestSchema,
    responseSchema: ResponseSchema,
  });
}
```

## Error Handling

Errors are automatically handled by the base API client:

- Parsed from API error responses
- Converted to `AppError` instances
- Toast notifications shown (configurable)
- Retry logic available for retryable errors

## Best Practices

1. **Always validate** - Use Zod schemas for all requests/responses
2. **Export types** - Make types available for consumers
3. **Use base client** - Never use `fetch` directly
4. **Document endpoints** - Add JSDoc comments
5. **Handle errors** - Let base client handle, or provide custom handlers

## Migration Notes

Existing modules that use the old pattern (`apiGet` from `../utils/api`) will continue to work but should be migrated to use the new base client for consistency.

