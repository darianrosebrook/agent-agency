# API Integration Readiness

**Date**: 2025-01-27  
**Status**: Infrastructure Ready for API Integration  
**Author**: @darianrosebrook

## Overview

The Agent Agency dashboard has been prepared with all the infrastructure needed for seamless API integration. All Open-WebUI patterns have been implemented, providing a solid foundation for connecting to the v3 backend API.

## Ready Infrastructure

### ✅ Frontend API Utilities

**Location**: `apps/agent_management_dashboard/src/lib/utils/api.ts`

**Features**:
- `apiFetch()` - Standardized fetch wrapper with error handling
- `apiGet()`, `apiPost()`, `apiPatch()`, `apiDelete()` - Convenience methods
- Automatic error parsing and conversion to `AppError`
- Retry logic with exponential backoff for retryable errors
- Consistent error handling across all API calls

**Usage Example**:
```typescript
import { apiGet, apiPost } from '@/lib/utils/api';

// GET request with automatic retry
const projects = await apiGet<Project[]>('/api/v1/projects');

// POST request with error handling
const newProject = await apiPost<Project>('/api/v1/projects', {
  name: 'My Project',
  description: 'Project description'
});
```

### ✅ Error Handling System

**Location**: `apps/agent_management_dashboard/src/lib/errors/types.ts`

**Features**:
- Standardized `ErrorResponse` format matching backend
- Error code mapping from backend to frontend
- User-friendly error messages
- Retry detection for retryable errors
- Toast notification integration

**Error Format**:
```typescript
interface ApiErrorResponse {
  error: string;        // Human-readable message
  code: string;         // Machine-readable code
  status: number;       // HTTP status code
  details?: Record<string, unknown>;
  request_id?: string;  // Request ID for correlation
}
```

### ✅ State Management

**Location**: `apps/agent_management_dashboard/src/lib/stores/`

**Features**:
- Zustand stores for chat and projects
- Optimistic updates with rollback
- Loading states
- Error handling integrated
- Uses `apiFetch` utility for all API calls

**Stores Ready**:
- `chatStore.ts` - Chat sessions and messages
- `projectStore.ts` - Projects and tasks

### ✅ Streaming Support

**Location**: `apps/agent_management_dashboard/src/lib/hooks/useStreamingResponse.ts`

**Features**:
- SSE stream consumption with `eventsource-parser`
- Configurable chunk splitting
- Stream debouncing for smooth UX
- Stream cancellation support
- Error handling and cleanup

**Usage Example**:
```typescript
const { start, stop, content, isStreaming } = useStreamingResponse({
  url: '/api/v1/chat/stream',
  method: 'POST',
  body: { agent_id, message, session_id },
  onChunk: (chunk) => updateMessage(chunk),
  onComplete: (fullContent) => saveMessage(fullContent),
  onError: (error) => handleError(error),
  debounce: true,
  debounceDelay: 16
});
```

### ✅ WebSocket Support

**Location**: `apps/agent_management_dashboard/src/lib/hooks/useWebSocket.ts`

**Features**:
- WebSocket connection management
- Automatic reconnection with exponential backoff
- Transport fallback (WebSocket → polling)
- Authentication token support
- Event handlers for messages, errors, open, close

**Usage Example**:
```typescript
const { connected, transport, error } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  token: authToken,
  reconnect: true,
  onMessage: (data) => handleMessage(data),
  onError: (error) => handleError(error),
  transport: 'auto' // Auto fallback to polling if WebSocket fails
});
```

### ✅ Loading States

**Components Ready**:
- `ChatListSkeleton` - Loading skeleton for chat lists
- `ProjectListSkeleton` - Loading skeleton for project lists
- `ProgressIndicator` - Progress bar component
- `ChatMessageSkeleton` - Loading skeleton for messages

**Integration**: All stores expose `isLoading` state for easy integration.

### ✅ Error Components

**Components Ready**:
- `ChatMessageError` - Error display with retry button
- Toast notifications via `sonner`
- Error boundary for React error catching

## Backend API Endpoints Available

### Authentication
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/logout` - User logout
- `GET /api/v1/auth/me` - Get current user (requires auth)
- `POST /api/v1/auth/refresh` - Refresh token

### Chat
- `POST /api/v1/chat` - Send chat message
- `POST /api/v1/chat/stream` - Stream agent response (SSE)
- `POST /api/v1/chat/stream/cancel` - Cancel active stream
- `GET /api/v1/chat/sessions` - List chat sessions
- `GET /api/v1/chat/sessions/:session_id` - Get chat session
- `GET /api/v1/chat/sessions/:session_id/messages` - Get messages

### WebSocket
- `WS /ws?token=<token>` - WebSocket connection (requires auth)

### Query Performance
- `GET /api/v1/query-performance/summary` - Performance summary
- `GET /api/v1/query-performance/metrics` - All query metrics
- `GET /api/v1/query-performance/slow-queries` - Slow query alerts
- `GET /api/v1/query-performance/top-slow?limit=10` - Top slow queries

## Integration Points

### 1. User Authentication

**Current State**: Auth middleware ready, endpoints available

**Integration Steps**:
1. Create login page component (if not exists)
2. Store auth token securely (localStorage/sessionStorage)
3. Add token to all API requests via `apiFetch` headers
4. Handle 401/403 errors and redirect to login

**Example**:
```typescript
// In api.ts, add token to headers
export async function apiFetch<T = unknown>(
  url: string,
  options: ApiFetchOptions = {}
): Promise<T> {
  const token = localStorage.getItem('auth_token');
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { Authorization: `Bearer ${token}` }),
    ...options.headers,
  };
  // ... rest of implementation
}
```

### 2. Chat Integration

**Current State**: Chat store ready, streaming hook ready

**Integration Steps**:
1. Update `chatStore.ts` to use real API endpoints
2. Replace mock data with `apiGet('/api/v1/chat/sessions')`
3. Use `useStreamingResponse` for agent responses
4. Handle WebSocket for real-time updates

**Example**:
```typescript
// In chatStore.ts
const fetchChats = async () => {
  setIsLoading(true);
  try {
    const sessions = await apiGet<ChatSession[]>('/api/v1/chat/sessions');
    setChats(sessions);
  } catch (error) {
    setError(parseApiError(error));
  } finally {
    setIsLoading(false);
  }
};
```

### 3. Project Integration

**Current State**: Project store ready, components ready

**Integration Steps**:
1. Update `projectStore.ts` to use real API endpoints
2. Replace mock data with `apiGet('/api/v1/projects')`
3. Use optimistic updates for create/update operations
4. Handle pagination using `PaginationParams`

**Example**:
```typescript
// In projectStore.ts
const fetchProjects = async (params?: PaginationParams) => {
  setIsLoading(true);
  try {
    const response = await apiGet<PaginatedResponse<Project>>(
      '/api/v1/projects',
      { params }
    );
    setProjects(response.items);
  } catch (error) {
    setError(parseApiError(error));
  } finally {
    setIsLoading(false);
  }
};
```

### 4. Dashboard Metrics

**Current State**: Dashboard components ready, API endpoints available

**Integration Steps**:
1. Create API calls for dashboard metrics
2. Use `apiGet` for telemetry data
3. Transform API response to component props
4. Add loading states and error handling

**Example**:
```typescript
// In Dashboard.tsx
const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);

useEffect(() => {
  const fetchMetrics = async () => {
    try {
      const data = await apiGet<DashboardMetrics>('/api/v1/dashboard/metrics');
      setMetrics(data);
    } catch (error) {
      toast.error('Failed to load dashboard metrics');
    }
  };
  fetchMetrics();
}, []);
```

## Backend Handler Status

### ✅ Fully Implemented
- Authentication handlers (`auth_handlers.rs`)
- Chat streaming (`chat_handlers.rs` - stream endpoints)
- Query performance monitoring (`query_performance.rs`)
- WebSocket authentication (`websocket/mod.rs`)

### 🚧 Partially Implemented (Stubs)
- Chat session management (handlers exist but return empty data)
- Project management (handlers may need implementation)
- User management (handlers exist but may need database integration)

### 📋 TODO: Backend Implementation Needed
1. **Chat Service**: Implement database queries for chat sessions and messages
2. **Project Service**: Implement database queries for projects and tasks
3. **User Service**: Complete user profile endpoints
4. **Dashboard Metrics**: Implement telemetry aggregation endpoints
5. **File Operations**: Implement file tree API for WorkspaceTab

## Integration Checklist

### Frontend Ready ✅
- [x] API fetch utility with error handling
- [x] Error parsing and user-friendly messages
- [x] Retry logic for retryable errors
- [x] Streaming response hook
- [x] WebSocket hook with fallback
- [x] Loading states and skeletons
- [x] Error components with retry
- [x] Zustand stores with optimistic updates
- [x] Toast notifications

### Backend Ready ✅
- [x] Authentication middleware
- [x] WebSocket authentication
- [x] Role-based access control
- [x] Stream endpoints with timeout/cancellation
- [x] Query performance monitoring
- [x] Pagination utilities
- [x] Standardized error responses

### Integration Needed 🚧
- [ ] Connect chat store to `/api/v1/chat/sessions`
- [ ] Connect project store to `/api/v1/projects`
- [ ] Implement login flow with token storage
- [ ] Add auth token to API requests
- [ ] Connect dashboard metrics to telemetry API
- [ ] Implement file tree API for WorkspaceTab
- [ ] Complete backend handlers for chat/project services

## Next Steps

1. **Start with Authentication**: Implement login flow and token management
2. **Connect Chat**: Replace mock data with real API calls
3. **Connect Projects**: Replace mock data with real API calls
4. **Add Dashboard Metrics**: Connect dashboard components to telemetry API
5. **Complete Backend Handlers**: Implement database queries for stubbed endpoints

## Notes

- All infrastructure is in place for seamless API integration
- Error handling is standardized and ready
- Loading states are implemented throughout
- Streaming and WebSocket support is ready
- Backend has authentication and authorization ready
- Some backend handlers return stubs - these need database integration

The dashboard is architecturally ready for API integration. The remaining work is connecting the existing infrastructure to the backend endpoints.

