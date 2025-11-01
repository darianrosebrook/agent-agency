# API Connection System

A comprehensive, production-ready connection system for the Agent Agency dashboard that provides reliable, scalable, and secure communication between the Next.js frontend and Rust backend.

## Features

- ✅ **Abort Controllers**: Proper request cancellation for all API calls
- ✅ **Connection Pooling**: Prevents server overload with intelligent connection management
- ✅ **Rate Limiting**: DDoS protection with configurable request limits
- ✅ **Retry Logic**: Exponential backoff with intelligent error classification
- ✅ **WebSocket Support**: Real-time task updates with auto-reconnection
- ✅ **SSE Integration**: Server-sent events for system monitoring
- ✅ **Webhook Handling**: Secure webhook processing with rate limiting
- ✅ **Error Recovery**: Automatic error classification and recovery strategies
- ✅ **Type Safety**: Full TypeScript integration with Rust backend

## Quick Start

### 1. Basic API Usage

```typescript
import { getApiClient } from '@/lib/api-client';

const apiClient = getApiClient();

// Get tasks with automatic error handling
const response = await apiClient.getTasks();
console.log(response.data.tasks);

// Create a task
const newTask = await apiClient.createTask({
  description: 'Implement new feature',
  execution_mode: 'strict',
  risk_tier: '2',
});
```

### 2. Real-Time WebSocket Updates

```typescript
import { useTaskWebSocket } from '@/hooks/useTaskWebSocket';

function TaskMonitor({ taskId }) {
  const { isConnected, taskUpdates, subscribeToTask } = useTaskWebSocket(taskId);

  useEffect(() => {
    subscribeToTask(taskId);
  }, [taskId]);

  return (
    <div>
      Status: {isConnected ? '🟢' : '🔴'}
      {taskUpdates.map(update => (
        <div key={update.timestamp}>
          Task {update.task_id}: {update.status} ({update.progress_percentage}%)
        </div>
      ))}
    </div>
  );
}
```

### 3. Server-Sent Events

```typescript
import { useSSEConnection } from '@/hooks/useSSEConnection';

function SystemMonitor() {
  const { isConnected, healthData, alerts } = useSSEConnection('/api/health/stream');

  return (
    <div>
      <p>SSE Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
      <p>Latest Health: {healthData[healthData.length - 1]?.status}</p>
      <p>Active Alerts: {alerts.length}</p>
    </div>
  );
}
```

### 4. Webhook Integration

```typescript
import { useWebhookHandler } from '@/hooks/useWebhookHandler';

function WebhookDemo() {
  const webhookHandler = useWebhookHandler({
    url: '/api/webhooks/tasks',
    rateLimit: { maxRequests: 30, windowMs: 60000 },
  });

  const sendNotification = async () => {
    await webhookHandler.sendWebhook({
      type: 'task_completed',
      payload: { taskId: 'task-123', status: 'success' },
    });
  };

  return (
    <div>
      <p>Status: {webhookHandler.connectionState}</p>
      <button onClick={sendNotification} disabled={webhookHandler.rateLimited}>
        Send Webhook
      </button>
    </div>
  );
}
```

## Configuration

### Environment Variables

```bash
# Backend API URL
NEXT_PUBLIC_V3_BACKEND_URL=http://localhost:8080

# WebSocket URL
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

### API Client Configuration

```typescript
const apiClient = new ApiClient({
  baseUrl: 'https://api.example.com',
  timeout: 30000,        // 30 seconds
  retryAttempts: 3,      // Max retries
  retryDelay: 1000,      // Base delay in ms
});
```

## Architecture

### Core Components

1. **ApiClient**: Main HTTP client with connection pooling and rate limiting
2. **WebSocket Hooks**: Real-time communication with auto-reconnection
3. **SSE Hooks**: Server-sent events for streaming data
4. **Webhook Handler**: Secure webhook processing
5. **Error Handler**: Intelligent error classification and recovery

### Anti-DDoS Protection

- **Rate Limiting**: Configurable per-endpoint limits
- **Connection Pooling**: Prevents resource exhaustion
- **Request Throttling**: Exponential backoff for retries
- **Message Filtering**: WebSocket/SSE spam prevention

### Error Handling

The system automatically classifies errors and suggests recovery strategies:

```typescript
import { useErrorHandler } from '@/lib/error-handling';

function MyComponent() {
  const { handleError } = useErrorHandler();

  const riskyOperation = async () => {
    try {
      await apiClient.createTask(taskData);
    } catch (error) {
      // Automatic classification and recovery attempt
      const appError = await handleError(error);
      console.log('Error category:', appError.category);
      console.log('User message:', appError.userMessage);
    }
  };
}
```

## API Reference

### ApiClient Methods

```typescript
interface ApiClient {
  // Task operations
  getTasks(options?: RequestOptions): Promise<ApiResponse<TaskListResponse>>;
  getTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<Task>>;
  createTask(taskData: TaskSubmissionRequest, options?: RequestOptions): Promise<ApiResponse<TaskSubmissionResponse>>;
  cancelTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>>;
  pauseTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>>;
  resumeTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>>;

  // Health check
  healthCheck(options?: RequestOptions): Promise<ApiResponse<{ status: string; version: string }>>;

  // Metrics
  getMetrics(options?: RequestOptions): Promise<ApiResponse<Record<string, any>>>;

  // Connection management
  getActiveConnections(): number;
  abortEndpointConnections(endpoint: string): void;
}
```

### Hook Interfaces

```typescript
interface UseTaskWebSocketReturn {
  connectionState: ConnectionState;
  isConnected: boolean;
  lastMessage?: WebSocketMessage;
  error?: string;
  subscribeToTask: (taskId: string) => void;
  unsubscribeFromTask: (taskId: string) => void;
  sendMessage: (message: any) => void;
}

interface UseSSEConnectionReturn {
  connectionState: SSEConnectionState;
  isConnected: boolean;
  lastEvent?: SSEMessage;
  error?: string;
  subscribe: (channels: string[]) => void;
  unsubscribe: (channels: string[]) => void;
}

interface UseWebhookHandlerReturn {
  isConnected: boolean;
  connectionState: string;
  lastMessage?: WebhookMessage;
  error?: string;
  messageCount: number;
  rateLimited: boolean;
  sendWebhook: (message: Partial<WebhookMessage>) => Promise<boolean>;
}
```

## Error Categories

| Category | Description | Recovery Strategy |
|----------|-------------|-------------------|
| `NETWORK` | Connection issues | Retry + Reconnect |
| `TIMEOUT` | Request timeout | Retry |
| `ABORTED` | User cancelled | Notify user |
| `RATE_LIMIT` | Too many requests | Retry with backoff |
| `AUTHENTICATION` | Invalid credentials | Reauthenticate |
| `AUTHORIZATION` | Permission denied | Notify user |
| `VALIDATION` | Invalid input | Notify user |
| `SERVER` | Server errors | Retry + Escalate |
| `CLIENT` | Client errors | Notify user |
| `UNKNOWN` | Unexpected errors | Retry + Escalate |

## Testing

Run the comprehensive integration tests:

```bash
npm test src/tests/integration/connections.test.ts
```

Tests cover:
- ✅ API client functionality
- ✅ WebSocket connections
- ✅ SSE streaming
- ✅ Webhook handling
- ✅ Error classification
- ✅ End-to-end data flow
- ✅ Connection pooling
- ✅ Rate limiting

## Production Deployment

### Security Considerations

1. **HTTPS Only**: Ensure all connections use HTTPS in production
2. **API Keys**: Configure backend authentication
3. **Rate Limiting**: Adjust limits based on server capacity
4. **CORS**: Configure proper CORS policies
5. **WebSocket Origin**: Validate WebSocket origins

### Monitoring

The system provides built-in monitoring:

```typescript
// Connection health
const activeConnections = apiClient.getActiveConnections();

// WebSocket status
const { isConnected, connectionStatus } = useTaskWebSocket();

// Error tracking
const { handleError } = useErrorHandler();
```

### Performance Tuning

```typescript
// Adjust connection pool size
const apiClient = new ApiClient({
  // Reduce for memory-constrained environments
  maxConnections: 5,
});

// Configure rate limits
const webhookHandler = useWebhookHandler({
  rateLimit: {
    maxRequests: 10,  // Reduce for high-security endpoints
    windowMs: 60000,
  },
});
```

## Troubleshooting

### Common Issues

**WebSocket disconnections:**
- Check `NEXT_PUBLIC_WS_URL` environment variable
- Verify backend WebSocket server is running
- Check browser network tab for connection errors

**Rate limiting:**
- Monitor `apiClient.getActiveConnections()`
- Adjust rate limits in configuration
- Check server capacity

**SSE not working:**
- Verify server supports SSE
- Check CORS configuration
- Ensure endpoint returns proper `text/event-stream` content type

**Webhook failures:**
- Verify webhook endpoint URL
- Check authentication/authorization
- Monitor rate limiting status

### Debug Mode

Enable detailed logging:

```typescript
// Enable debug logging
localStorage.setItem('connection-debug', 'true');

// Check connection status
console.log('Active connections:', apiClient.getActiveConnections());
console.log('WebSocket status:', websocketHook.connectionStatus);
```

## Contributing

### Adding New Endpoints

1. Add method to `ApiClient` class
2. Update TypeScript interfaces
3. Add tests to `connections.test.ts`
4. Update documentation

### Adding New Hooks

1. Follow existing hook patterns
2. Include error handling
3. Add connection state management
4. Write comprehensive tests

### Error Handling

When adding new error types:

1. Add to `ErrorCategory` enum
2. Update error patterns and messages
3. Define recovery strategies
4. Add tests for classification

## License

This connection system is part of the Agent Agency project.

---

For more examples, see `src/examples/connection-usage.tsx`.
