# Open-WebUI Best Practices Recommendations

**Date**: 2025-01-27  
**Target**: Agent-Agency Dashboard Implementation

## Overview

This document provides prioritized, actionable recommendations for adopting open-webui's best practices in the agent-agency project. Each recommendation includes implementation guidance, code examples adapted for Next.js/React and Rust, and integration points.

## Priority Levels

- **P0**: Critical - Implement immediately for core functionality
- **P1**: High - Implement soon for production readiness
- **P2**: Medium - Implement for improved UX and performance
- **P3**: Low - Nice-to-have improvements

## P0: Critical Recommendations

### 1. Implement WebSocket with Channel-Based Routing

**Why**: Essential for real-time agent interactions and streaming responses.

**Implementation Guidance**:

**Backend (Rust)**:
```rust
// iterations/v3/data-infrastructure/src/websocket/mod.rs

use axum::extract::ws::{Message, WebSocket};
use tokio::sync::broadcast;
use uuid::Uuid;

pub struct WebSocketManager {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_channel(&self, agent_id: &str, session_id: &str) -> String {
        let request_id = Uuid::new_v4().to_string();
        let channel = format!("agent:{}:session:{}:request:{}", agent_id, session_id, request_id);
        
        let (tx, _rx) = broadcast::channel(100);
        self.channels.write().unwrap().insert(channel.clone(), tx);
        
        channel
    }

    pub async fn send_to_channel(&self, channel: &str, message: Message) -> Result<()> {
        if let Some(tx) = self.channels.read().unwrap().get(channel) {
            tx.send(message)?;
        }
        Ok(())
    }

    pub fn cleanup_channel(&self, channel: &str) {
        self.channels.write().unwrap().remove(channel);
    }
}
```

**Frontend (TypeScript/React)**:
```typescript
// apps/agent_management_dashboard/src/lib/websocket/useWebSocket.ts

import { useEffect, useRef, useState } from 'react';
import { io, Socket } from 'socket.io-client';

export function useWebSocket(url: string, token: string) {
  const [socket, setSocket] = useState<Socket | null>(null);
  const [connected, setConnected] = useState(false);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;

  useEffect(() => {
    const socketInstance = io(url, {
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionDelayMax: 5000,
      randomizationFactor: 0.5,
      path: '/ws/socket.io',
      transports: ['websocket', 'polling'],
      auth: { token },
    });

    socketInstance.on('connect', () => {
      console.log('WebSocket connected:', socketInstance.id);
      setConnected(true);
      reconnectAttempts.current = 0;
      
      // Emit user-join event
      socketInstance.emit('user-join', { auth: { token } });
    });

    socketInstance.on('disconnect', (reason) => {
      console.log('WebSocket disconnected:', reason);
      setConnected(false);
    });

    socketInstance.on('connect_error', (error) => {
      console.error('WebSocket connection error:', error);
      reconnectAttempts.current++;
      
      if (reconnectAttempts.current >= maxReconnectAttempts) {
        console.error('Max reconnection attempts reached');
      }
    });

    setSocket(socketInstance);

    return () => {
      socketInstance.close();
    };
  }, [url, token]);

  return { socket, connected };
}
```

**Integration Points**:
- Add WebSocket handler in `iterations/v3/data-infrastructure/src/api/server.rs`
- Create WebSocket hook in `apps/agent_management_dashboard/src/lib/websocket/`
- Update `ChatContext` to use WebSocket for real-time updates

**Testing Strategy**:
- Unit tests for channel creation and cleanup
- Integration tests for message routing
- E2E tests for reconnection scenarios

### 2. Implement SSE Streaming for Agent Responses

**Why**: Required for streaming AI agent responses in real-time.

**Implementation Guidance**:

**Backend (Rust)**:
```rust
// iterations/v3/data-infrastructure/src/api/handlers/agent_handlers.rs

use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use std::convert::Infallible;

pub async fn stream_agent_response(
    agent_id: String,
    task_id: String,
    request: AgentRequest,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let channel = format!("agent:{}:task:{}", agent_id, task_id);

    // Spawn task to generate response
    tokio::spawn(async move {
        let mut response_generator = agent_service.generate_response(request).await;
        
        while let Some(chunk) = response_generator.next().await {
            let event = Event::default()
                .json_data(chunk)
                .unwrap();
            
            if tx.send(Ok(event)).await.is_err() {
                break;
            }
        }
        
        // Send done event
        let done_event = Event::default()
            .json_data(json!({ "done": true }))
            .unwrap();
        let _ = tx.send(Ok(done_event)).await;
    });

    Sse::new(rx)
        .keep_alive(axum::response::sse::KeepAlive::default())
}
```

**Frontend (TypeScript/React)**:
```typescript
// apps/agent_management_dashboard/src/lib/streaming/useStreamingResponse.ts

import { useCallback, useRef } from 'react';
import { EventSourceParserStream } from 'eventsource-parser/stream';

export function useStreamingResponse() {
  const abortControllerRef = useRef<AbortController | null>(null);

  const streamResponse = useCallback(async (
    url: string,
    options: RequestInit,
    onChunk: (chunk: string) => void,
    onDone: () => void,
    onError: (error: Error) => void
  ) => {
    abortControllerRef.current = new AbortController();

    try {
      const response = await fetch(url, {
        ...options,
        signal: abortControllerRef.current.signal,
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const reader = response.body
        ?.pipeThrough(new TextDecoderStream())
        .pipeThrough(new EventSourceParserStream())
        .getReader();

      if (!reader) {
        throw new Error('Failed to create stream reader');
      }

      while (true) {
        const { value, done } = await reader.read();
        
        if (done) {
          onDone();
          break;
        }

        if (value?.data) {
          try {
            const data = JSON.parse(value.data);
            
            if (data.done) {
              onDone();
              break;
            }

            if (data.error) {
              onError(new Error(data.error.message || 'Stream error'));
              break;
            }

            if (data.content) {
              onChunk(data.content);
            }
          } catch (e) {
            console.error('Error parsing SSE data:', e);
          }
        }
      }
    } catch (error) {
      if (error instanceof Error && error.name !== 'AbortError') {
        onError(error);
      }
    }
  }, []);

  const cancel = useCallback(() => {
    abortControllerRef.current?.abort();
  }, []);

  return { streamResponse, cancel };
}
```

**Integration Points**:
- Add SSE endpoint in agent handlers
- Create streaming hook in dashboard
- Update Chat component to use streaming
- Add cancel functionality for user-initiated stops

**Testing Strategy**:
- Test stream parsing with various chunk sizes
- Test error handling and recovery
- Test cancellation
- Test reconnection after network failure

### 3. Implement Dependency Injection for Authentication

**Why**: Clean, testable authentication middleware.

**Implementation Guidance**:

**Backend (Rust)**:
```rust
// iterations/v3/data-infrastructure/src/api/middleware/auth.rs

use axum::extract::{Request, FromRequestParts};
use axum::http::request::Parts;
use axum::response::Response;
use axum::Extension;
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| Response::builder().status(401).body("Missing authorization header").unwrap())?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| Response::builder().status(401).body("Invalid authorization format").unwrap())?;

        let decoding_key = DecodingKey::from_secret(SECRET.as_ref());
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| Response::builder().status(401).body("Invalid token").unwrap())?;

        Ok(AuthenticatedUser {
            id: token_data.claims.sub,
            email: token_data.claims.email,
            role: token_data.claims.role,
        })
    }
}

// Usage in handlers
pub async fn get_projects(
    user: AuthenticatedUser,
    Extension(db): Extension<DbPool>,
) -> Result<Json<Vec<Project>>> {
    let projects = db.get_projects_by_user_id(&user.id).await?;
    Ok(Json(projects))
}
```

**Integration Points**:
- Create auth middleware module
- Add to all protected routes
- Create admin-only middleware variant
- Add to API server setup

**Testing Strategy**:
- Unit tests for token validation
- Integration tests for protected routes
- Test invalid token scenarios
- Test expired token handling

## P1: High Priority Recommendations

### 4. Migrate State Management to Zustand

**Why**: Better performance and scalability than React Context for global state.

**Implementation Guidance**:

```typescript
// apps/agent_management_dashboard/src/stores/chatStore.ts

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

interface ChatState {
  chats: Chat[];
  currentChatId: string | null;
  loading: boolean;
  error: string | null;
  
  // Actions
  setChats: (chats: Chat[]) => void;
  setCurrentChat: (chatId: string) => void;
  addChat: (chat: Chat) => void;
  updateChat: (chatId: string, updates: Partial<Chat>) => void;
  addMessage: (chatId: string, message: Message) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useChatStore = create<ChatState>()(
  devtools(
    (set) => ({
      chats: [],
      currentChatId: null,
      loading: false,
      error: null,

      setChats: (chats) => set({ chats }),
      
      setCurrentChat: (chatId) => set({ currentChatId: chatId }),
      
      addChat: (chat) => set((state) => ({
        chats: [chat, ...state.chats],
      })),
      
      updateChat: (chatId, updates) => set((state) => ({
        chats: state.chats.map((chat) =>
          chat.id === chatId ? { ...chat, ...updates } : chat
        ),
      })),
      
      addMessage: (chatId, message) => set((state) => ({
        chats: state.chats.map((chat) =>
          chat.id === chatId
            ? { ...chat, messages: [...chat.messages, message] }
            : chat
        ),
      })),
      
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),
    }),
    { name: 'ChatStore' }
  )
);
```

**Migration Path**:
1. Install Zustand: `npm install zustand`
2. Create store files for each domain (chat, project, agent)
3. Migrate Context providers to stores
4. Update components to use stores
5. Remove old Context files

**Integration Points**:
- Replace `ChatContext` with `useChatStore`
- Replace `ProjectContext` with `useProjectStore`
- Add selectors for computed values
- Add middleware for persistence if needed

### 5. Implement Optimistic Updates

**Why**: Instant UI feedback improves perceived performance.

**Implementation Guidance**:

```typescript
// apps/agent_management_dashboard/src/lib/hooks/useOptimisticUpdate.ts

import { useCallback } from 'react';
import { useChatStore } from '@/stores/chatStore';

export function useOptimisticUpdate() {
  const updateChat = useChatStore((state) => state.updateChat);
  const setError = useChatStore((state) => state.setError);

  const optimisticUpdate = useCallback(
    async <T>(
      chatId: string,
      optimisticUpdate: Partial<Chat>,
      apiCall: () => Promise<T>,
      rollback?: () => void
    ) => {
      // Apply optimistic update immediately
      updateChat(chatId, optimisticUpdate);

      try {
        // Perform actual API call
        const result = await apiCall();
        return result;
      } catch (error) {
        // Rollback on error
        if (rollback) {
          rollback();
        } else {
          // Default rollback: revert to previous state
          // This would require storing previous state
        }
        setError(error instanceof Error ? error.message : 'Update failed');
        throw error;
      }
    },
    [updateChat, setError]
  );

  return { optimisticUpdate };
}
```

**Usage Example**:
```typescript
const { optimisticUpdate } = useOptimisticUpdate();

const handleSendMessage = async (chatId: string, content: string) => {
  const tempMessage: Message = {
    id: `temp-${Date.now()}`,
    content,
    role: 'user',
    timestamp: new Date(),
  };

  await optimisticUpdate(
    chatId,
    { messages: [...currentChat.messages, tempMessage] },
    async () => {
      return await api.sendMessage(chatId, content);
    }
  );
};
```

**Integration Points**:
- Add to chat message sending
- Add to project updates
- Add to task completion
- Implement rollback for all optimistic updates

### 6. Implement Consistent Error Handling

**Why**: Better UX and easier debugging.

**Implementation Guidance**:

**Backend Error Types**:
```rust
// iterations/v3/data-infrastructure/src/api/errors.rs

use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": {
                "code": status.as_str(),
                "message": error_message,
            }
        }));

        (status, body).into_response()
    }
}
```

**Frontend Error Component**:
```typescript
// apps/agent_management_dashboard/src/components/ErrorDisplay.tsx

interface ErrorDisplayProps {
  error: Error | string | { error?: { message?: string }; detail?: string; message?: string };
  onRetry?: () => void;
}

export function ErrorDisplay({ error, onRetry }: ErrorDisplayProps) {
  const errorMessage = typeof error === 'string'
    ? error
    : error instanceof Error
    ? error.message
    : error?.error?.message || error?.detail || error?.message || 'An error occurred';

  return (
    <div className="flex my-2 gap-2.5 border px-4 py-3 border-red-600/10 bg-red-600/10 rounded-lg">
      <Info className="size-5 text-red-700 dark:text-red-400" />
      <div className="self-center text-sm">{errorMessage}</div>
      {onRetry && (
        <Button onClick={onRetry} variant="outline" size="sm">
          Retry
        </Button>
      )}
    </div>
  );
}
```

**Integration Points**:
- Create error types in Rust
- Add error handling to all API handlers
- Create ErrorDisplay component
- Add error boundaries in React
- Implement retry logic

## P2: Medium Priority Recommendations

### 7. Add Toast Notifications

**Why**: Non-intrusive user feedback.

**Implementation**:
```typescript
// Already have sonner installed
import { toast } from 'sonner';

// Usage
toast.success('Message sent successfully');
toast.error('Failed to send message');
toast.loading('Sending message...');
```

**Integration Points**:
- Add toasts for all user actions
- Add error toasts for API failures
- Add success toasts for confirmations
- Add loading toasts for long operations

### 8. Implement Loading States

**Why**: Better UX than blank screens.

**Implementation**:
```typescript
// Use shadcn/ui Skeleton component
import { Skeleton } from '@/components/ui/skeleton';

export function ChatMessageSkeleton() {
  return (
    <div className="flex gap-4 p-4">
      <Skeleton className="h-10 w-10 rounded-full" />
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-[250px]" />
        <Skeleton className="h-4 w-[200px]" />
      </div>
    </div>
  );
}
```

**Integration Points**:
- Add skeletons for chat messages
- Add skeletons for project lists
- Add loading spinners for buttons
- Add progress indicators for long operations

### 9. Optimize Database Queries

**Why**: Performance improvement.

**Implementation**:
```rust
// Add indexes in migrations
CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_created_at ON chat_sessions(created_at DESC);
CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_projects_user_id_updated_at ON projects(user_id, updated_at DESC);
```

**Integration Points**:
- Analyze query patterns
- Add indexes for frequently queried fields
- Use composite indexes for multi-column queries
- Monitor query performance

## P3: Low Priority Recommendations

### 10. Implement Chunk Splitting for Better UX

**Why**: Simulates more natural streaming.

**Implementation**:
```typescript
// Split large chunks into smaller pieces
function splitLargeChunks(content: string, maxChunkSize: number = 3): string[] {
  const chunks: string[] = [];
  let remaining = content;
  
  while (remaining.length > 0) {
    const chunkSize = Math.min(
      Math.floor(Math.random() * maxChunkSize) + 1,
      remaining.length
    );
    chunks.push(remaining.slice(0, chunkSize));
    remaining = remaining.slice(chunkSize);
  }
  
  return chunks;
}
```

### 11. Add Version Checking

**Why**: Useful for deployments and updates.

**Implementation**:
```typescript
// Check version on WebSocket connect
socket.on('connect', async () => {
  const version = await api.getVersion();
  if (version !== currentVersion) {
    // Trigger refresh or show update notification
    window.location.reload();
  }
});
```

## Implementation Checklist

### Phase 1: Core Functionality (Week 1-2)
- [ ] Implement WebSocket connection
- [ ] Implement SSE streaming
- [ ] Add authentication middleware
- [ ] Create error handling infrastructure

### Phase 2: State Management (Week 3)
- [ ] Migrate to Zustand
- [ ] Implement optimistic updates
- [ ] Add error recovery

### Phase 3: UX Improvements (Week 4)
- [ ] Add toast notifications
- [ ] Implement loading states
- [ ] Add error display components

### Phase 4: Performance (Week 5)
- [ ] Optimize database queries
- [ ] Add indexes
- [ ] Monitor performance

## Success Metrics

- WebSocket connection success rate > 99%
- Stream response latency < 100ms
- Error recovery rate > 95%
- User satisfaction with loading states
- Database query performance improvement

## Conclusion

Focus on P0 and P1 recommendations first. These provide the foundation for real-time agent interactions. P2 and P3 can be implemented incrementally as the system matures.

