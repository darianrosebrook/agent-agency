# Error Resilience System

Comprehensive error handling and resilience patterns for the dashboard application.

## Overview

The error resilience system provides multiple layers of error protection:

1. **Scoped Error Boundaries** - Isolate errors to specific component trees
2. **Graceful Degradation** - Fallback UI when components fail
3. **Error Guards** - Defensive programming utilities
4. **Error Recovery** - Automatic retry with exponential backoff
5. **Circuit Breakers** - Prevent repeated failures

## Components

### ScopedErrorBoundary

Isolated error boundaries that prevent errors from propagating to parent components.

```tsx
import { ScopedErrorBoundary } from "@/components/errors/ScopedErrorBoundary";

<ScopedErrorBoundary
  scope="dashboard-chart"
  level="non-critical"
  resetKeys={[dataVersion]}
>
  <ChartComponent />
</ScopedErrorBoundary>
```

**Props:**
- `scope`: Unique identifier for error tracking
- `fallback`: Custom fallback UI (optional)
- `level`: "critical" | "non-critical" (affects logging)
- `resetKeys`: Array of values that trigger boundary reset
- `onError`: Error handler callback

### GracefulDegradation

Wraps components with automatic fallback UI on errors.

```tsx
import { GracefulDegradation } from "@/components/errors/GracefulDegradation";

<GracefulDegradation scope="sidebar-projects" showFallback={true}>
  <ProjectsList />
</GracefulDegradation>
```

### ChartErrorBoundary

Specialized error boundary for chart/visualization components.

```tsx
import { ChartErrorBoundary } from "@/components/errors/ChartErrorBoundary";

<ChartErrorBoundary chartName="Task Progress">
  <TaskProgressChart />
</ChartErrorBoundary>
```

### ErrorIsolation

Prevents errors from propagating to parent components.

```tsx
import { ErrorIsolation } from "@/components/errors/ErrorIsolation";

<ErrorIsolation scope="feature-widget" isolate={true}>
  <FeatureWidget />
</ErrorIsolation>
```

## Hooks

### useErrorRecovery

Automatic error recovery with retry logic and circuit breakers.

```tsx
import { useErrorRecovery } from "@/hooks/useErrorRecovery";

function MyComponent() {
  const { executeWithRecovery, error, isRecovering } = useErrorRecovery({
    maxRetries: 3,
    retryDelay: 1000,
    timeout: 5000,
  });

  const fetchData = async () => {
    const result = await executeWithRecovery(() => apiCall());
    if (result) {
      setData(result);
    }
  };
}
```

### useIsolatedError

Hook for isolated error handling within components.

```tsx
import { useIsolatedError } from "@/components/errors/ErrorIsolation";

function MyComponent() {
  const { error, handleError, clearError, hasError } = useIsolatedError("my-component");

  // Use handleError in try/catch blocks
}
```

## Utilities

### Error Guards (`lib/utils/errorGuards.ts`)

Defensive programming utilities for safe operations.

```tsx
import {
  safeAsync,
  safeSync,
  safeGet,
  safeRetry,
  CircuitBreaker,
  withTimeout,
} from "@/lib/utils/errorGuards";

// Safe async execution
const data = await safeAsync(
  () => fetchData(),
  defaultValue,
  "component-name"
);

// Safe property access
const value = safeGet(obj, "nested.property", fallback);

// Retry with exponential backoff
const result = await safeRetry(
  () => apiCall(),
  {
    maxRetries: 3,
    delay: 1000,
    backoff: true,
  }
);

// Circuit breaker pattern
const breaker = new CircuitBreaker();
const result = await breaker.execute(() => apiCall());

// Timeout wrapper
const result = await withTimeout(apiCall(), 5000);
```

## Best Practices

### 1. Scope Errors Appropriately

- Use scoped boundaries for non-critical features
- Use global boundaries for critical app sections
- Name scopes descriptively (e.g., "dashboard-chart-task-progress")

### 2. Provide Meaningful Fallbacks

- Always provide fallback UI for better UX
- Use `ChartErrorBoundary` for visualization components
- Use `GracefulDegradation` for feature sections

### 3. Use Error Guards for Data Access

- Always use `safeGet` for nested property access
- Use `safeAsync`/`safeSync` for operations that might fail
- Validate API responses before using them

### 4. Implement Retry Logic

- Use `useErrorRecovery` hook for data fetching
- Configure appropriate retry counts and delays
- Use circuit breakers for external API calls

### 5. Log Errors Appropriately

- Use `level="critical"` for errors that need immediate attention
- Use `level="non-critical"` for recoverable errors
- Include scope information in error logs

## Error Scoping Strategy

```
App (Global ErrorBoundary)
├── Navigation (ScopedErrorBoundary: "navigation")
│   └── Projects List (ErrorIsolation: "navigation-projects")
├── Dashboard (ScopedErrorBoundary: "dashboard")
│   ├── Chart 1 (ChartErrorBoundary: "chart-task-progress")
│   ├── Chart 2 (ChartErrorBoundary: "chart-agent-activity")
│   └── Metrics (ErrorIsolation: "dashboard-metrics")
└── Settings (ScopedErrorBoundary: "settings")
    └── Form (ErrorIsolation: "settings-form")
```

## Example: Complete Implementation

```tsx
import { ChartErrorBoundary } from "@/components/errors/ChartErrorBoundary";
import { useErrorRecovery } from "@/hooks/useErrorRecovery";
import { safeAsync, safeGet } from "@/lib/utils/errorGuards";

function TaskProgressChart() {
  const { executeWithRecovery, error } = useErrorRecovery({
    maxRetries: 3,
    timeout: 5000,
  });

  const [data, setData] = useState(null);

  useEffect(() => {
    async function fetchData() {
      const result = await safeAsync(
        () => executeWithRecovery(() => api.getTasks()),
        null,
        "TaskProgressChart"
      );

      if (result) {
        // Safely access nested properties
        const tasks = safeGet(result, "data.tasks", []);
        setData(tasks);
      }
    }

    fetchData();
  }, [executeWithRecovery]);

  if (error) {
    return <ErrorDisplay error={error} />;
  }

  return <Chart data={data} />;
}

// Usage in Dashboard
<ChartErrorBoundary chartName="Task Progress">
  <TaskProgressChart />
</ChartErrorBoundary>
```

## Migration Guide

### Before (No Error Handling)

```tsx
function MyChart() {
  const [data, setData] = useState(null);

  useEffect(() => {
    fetch("/api/data")
      .then(res => res.json())
      .then(data => setData(data.data.items)); // ❌ No error handling
  }, []);

  return <Chart data={data} />;
}
```

### After (With Error Resilience)

```tsx
import { ChartErrorBoundary } from "@/components/errors/ChartErrorBoundary";
import { useErrorRecovery } from "@/hooks/useErrorRecovery";
import { safeAsync, safeGet } from "@/lib/utils/errorGuards";

function MyChart() {
  const { executeWithRecovery } = useErrorRecovery();
  const [data, setData] = useState(null);

  useEffect(() => {
    async function fetchData() {
      const result = await safeAsync(
        () => executeWithRecovery(() => fetch("/api/data").then(r => r.json())),
        null,
        "MyChart"
      );

      const items = safeGet(result, "data.items", []); // ✅ Safe access
      setData(items);
    }

    fetchData();
  }, [executeWithRecovery]);

  return <Chart data={data || []} />; // ✅ Fallback to empty array
}

// Wrap in error boundary
<ChartErrorBoundary chartName="My Chart">
  <MyChart />
</ChartErrorBoundary>
```

