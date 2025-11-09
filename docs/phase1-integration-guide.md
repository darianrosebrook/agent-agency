# Phase 1 Integration Guide: Real-Time Communication

**Status**: ✅ Complete  
**Date**: November 2025  
**Author**: @darianrosebrook

## Overview

Phase 1 implements real-time communication infrastructure for the agent dashboard, enabling streaming agent responses via WebSocket and Server-Sent Events (SSE). The implementation preserves your existing UI design while adding powerful real-time capabilities.

## What Was Implemented

### Backend (Rust)

#### 1. WebSocket Manager (`iterations/v3/data-infrastructure/src/websocket/mod.rs`)
- Channel-based routing for isolated communication streams
- Connection management with automatic cleanup
- Support for multiple concurrent connections
- Format: `agent:{agent_id}:session:{session_id}:request:{request_id}`

#### 2. SSE Streaming Endpoint (`iterations/v3/data-infrastructure/src/api/handlers/chat_handlers.rs`)
- `/api/chat/stream` endpoint for streaming agent responses
- Chunk aggregation and error recovery
- Channel-based isolation per request
- Graceful cleanup on completion

#### 3. Chat API Handlers
- `get_chat_sessions` - List chat sessions
- `create_chat_session` - Create new session
- `get_chat_messages` - Get messages for a session
- `stream_agent_response` - Stream agent response via SSE

### Frontend (React/TypeScript)

#### 1. `useWebSocket` Hook (`apps/agent_management_dashboard/src/lib/hooks/useWebSocket.ts`)
- Automatic reconnection with exponential backoff
- Channel subscription support
- Connection state management
- Error handling and recovery

#### 2. `useStreamingResponse` Hook (`apps/agent_management_dashboard/src/lib/hooks/useStreamingResponse.ts`)
- SSE stream consumption
- Chunk aggregation
- Error recovery
- Completion detection
- Supports override options for dynamic requests

#### 3. Chat Component Integration (`apps/agent_management_dashboard/src/components/Chat.tsx`)
- Integrated streaming hook
- Preserves existing UI design
- Fallback to simulation if API unavailable
- Real-time message updates

## Architecture

### Channel-Based Routing

Inspired by Open-WebUI's pattern, we use channel-based routing to isolate streams:

```
agent:{agent_id}:session:{session_id}:request:{request_id}
```

This allows:
- Multiple concurrent requests per session
- Isolated error handling
- Efficient cleanup
- Scalable architecture

### Streaming Flow

```
User sends message
  ↓
Chat component creates messages locally
  ↓
Starts SSE stream to /api/chat/stream
  ↓
Backend creates channel and starts streaming
  ↓
Chunks arrive → Frontend updates message content
  ↓
Stream completes → Message marked as done
```

## Usage

### Basic Streaming

The Chat component automatically uses streaming when you send a message:

```typescript
// Already integrated in Chat.tsx
const { start: startStreaming } = useStreamingResponse({
  url: '/api/chat/stream',
  method: 'POST',
  onChunk: (chunk: string) => {
    // Updates message content in real-time
  },
  onComplete: (fullContent: string) => {
    // Marks message as complete
  },
  onError: (error: Error) => {
    // Handles errors gracefully
  },
});
```

### Manual Streaming (Advanced)

If you need to stream from other components:

```typescript
import { useStreamingResponse } from '@/lib/hooks';

function MyComponent() {
  const { start, stop, reset, state } = useStreamingResponse({
    url: '/api/chat/stream',
    method: 'POST',
    onChunk: (chunk) => console.log('Chunk:', chunk),
    onComplete: (content) => console.log('Complete:', content),
  });

  const handleStream = () => {
    start({
      url: '/api/chat/stream',
      body: {
        agent_id: 'my-agent',
        session_id: 'session-123',
        message: 'Hello',
      },
    });
  };

  return (
    <div>
      <button onClick={handleStream}>Start Stream</button>
      {state.isStreaming && <div>Streaming...</div>}
      {state.error && <div>Error: {state.error.message}</div>}
    </div>
  );
}
```

### WebSocket Usage

For real-time bidirectional communication:

```typescript
import { useWebSocket } from '@/lib/hooks';

function MyComponent() {
  const { send, state, connect, disconnect } = useWebSocket({
    url: 'ws://localhost:3000/ws',
    token: 'your-auth-token',
    onMessage: (data) => {
      console.log('Received:', data);
    },
    onError: (error) => {
      console.error('WebSocket error:', error);
    },
  });

  const handleSubscribe = () => {
    send({
      action: 'subscribe',
      channel: 'agent:123:session:456',
    });
  };

  return (
    <div>
      <div>Status: {state.connected ? 'Connected' : 'Disconnected'}</div>
      <button onClick={handleSubscribe}>Subscribe</button>
    </div>
  );
}
```

## Configuration

### Environment Variables

Set these in your `.env.local`:

```bash
# API Base URL (defaults to http://localhost:3000)
NEXT_PUBLIC_API_URL=http://localhost:3000

# WebSocket URL (optional, defaults to ws://localhost:3000/ws)
NEXT_PUBLIC_WS_URL=ws://localhost:3000/ws
```

### Backend Configuration

The backend WebSocket manager is configured in `ApiState`:

```rust
let api_state = ApiState {
    api: Arc::new(rest_api),
    websocket_manager: Arc::new(WebSocketManager::new()),
};
```

## Error Handling

### Frontend

The streaming hook includes comprehensive error handling:

1. **Network Errors**: Automatically caught and reported via `onError`
2. **API Errors**: HTTP status codes trigger appropriate error messages
3. **Stream Errors**: Parsing errors are handled gracefully
4. **Fallback**: Falls back to simulation if API unavailable

### Backend

The SSE endpoint handles:

1. **Channel Creation**: Errors if channel creation fails
2. **Stream Errors**: Logged and reported to client
3. **Cleanup**: Automatic channel cleanup on completion or error

## Testing

### Manual Testing

1. **Start Backend**:
   ```bash
   cd iterations/v3/data-infrastructure
   cargo run
   ```

2. **Start Frontend**:
   ```bash
   cd apps/agent_management_dashboard
   npm run dev
   ```

3. **Test Streaming**:
   - Open chat interface
   - Send a message
   - Observe real-time streaming (or simulation fallback)

### API Testing

Test the streaming endpoint directly:

```bash
curl -X POST http://localhost:3000/api/chat/stream \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "test-agent",
    "session_id": "test-session",
    "message": "Hello, world!"
  }'
```

## Troubleshooting

### Stream Not Starting

1. Check `NEXT_PUBLIC_API_URL` is set correctly
2. Verify backend is running on correct port
3. Check browser console for CORS errors
4. Verify network tab shows SSE connection

### Chunks Not Updating

1. Check `onChunk` callback is being called
2. Verify message ID matches `streamingRef.current`
3. Check `updateMessageInCurrentChat` is working
4. Verify message exists in chat state

### Fallback to Simulation

If you see simulation instead of real streaming:

1. Backend may not be running
2. API endpoint may not be configured
3. Check console for connection errors
4. Verify environment variables are set

## Next Steps

### Phase 2: State Management

- Migrate from React Context to Zustand
- Implement optimistic updates
- Add offline support
- Improve error boundaries

### Phase 3: Database Integration

- Connect ChatContext to PostgreSQL
- Implement real-time session sync
- Add message persistence
- Enable cross-device sync

### Phase 4: Advanced Features

- Task execution tracking
- Real-time collaboration
- File upload streaming
- Voice input/output

## Performance Considerations

### Streaming Performance

- **Chunk Size**: Backend sends ~50ms delays between chunks for smooth UX
- **Buffer Size**: Frontend uses 100-item buffer for SSE events
- **Memory**: Automatic cleanup prevents memory leaks

### WebSocket Performance

- **Reconnection**: Exponential backoff prevents server overload
- **Connection Pool**: Backend manages connections efficiently
- **Channel Isolation**: Prevents cross-contamination of streams

## Security Considerations

### Authentication

Currently uses token-based auth (TODO: implement proper JWT validation):

```typescript
// WebSocket connection with token
const ws = new WebSocket(`ws://localhost:3000/ws?token=${token}`);
```

### CORS

Backend CORS is configured in middleware:

```rust
headers.insert(
    "access-control-allow-origin",
    HeaderValue::from_static("*"),
);
```

**TODO**: Restrict to specific origins in production.

## References

- [Open-WebUI Architecture Analysis](./open-webui-architecture-analysis.md)
- [Best Practices Guide](./open-webui-best-practices.md)
- [Implementation Roadmap](./open-webui-implementation-roadmap.md)
- [Code Patterns Library](./open-webui-code-patterns.md)

## Support

For issues or questions:
1. Check troubleshooting section above
2. Review error logs in browser console
3. Verify backend logs for server-side issues
4. Consult architecture documentation

---

**Status**: Phase 1 Complete ✅  
**Next**: Phase 2 - State Management Improvements

