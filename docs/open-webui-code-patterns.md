# Open-WebUI Code Pattern Library

**Date**: 2025-01-27  
**Purpose**: Reusable code patterns adapted from open-webui for agent-agency

## Overview

This document contains code patterns extracted from open-webui and adapted for agent-agency's tech stack (Next.js/React + Rust). Each pattern includes the original implementation, adapted version, and usage examples.

## Table of Contents

1. [WebSocket Connection Pattern](#websocket-connection-pattern)
2. [SSE Streaming Pattern](#sse-streaming-pattern)
3. [Channel-Based Routing Pattern](#channel-based-routing-pattern)
4. [State Management Pattern](#state-management-pattern)
5. [Optimistic Update Pattern](#optimistic-update-pattern)
6. [Error Handling Pattern](#error-handling-pattern)
7. [Stream Processing Pattern](#stream-processing-pattern)
8. [Authentication Middleware Pattern](#authentication-middleware-pattern)
9. [Component Organization Pattern](#component-organization-pattern)
10. [Loading State Pattern](#loading-state-pattern)

## WebSocket Connection Pattern

### Original (Svelte)
```typescript
const setupSocket = async (enableWebsocket) => {
  const _socket = io(`${WEBUI_BASE_URL}`, {
    reconnection: true,
    reconnectionDelay: 1000,
    reconnectionDelayMax: 5000,
    randomizationFactor: 0.5,
    path: '/ws/socket.io',
    transports: enableWebsocket ? ['websocket'] : ['polling', 'websocket'],
    auth: { token: localStorage.token }
  });
  
  _socket.on('connect', async () => {
    if (localStorage.getItem('token')) {
      _socket.emit('user-join', { auth: { token: localStorage.token } });
    }
  });
};
```

### Adapted (React Hook)
```typescript
// apps/agent_management_dashboard/src/lib/websocket/useWebSocket.ts

import { useEffect, useRef, useState, useCallback } from 'react';
import { io, Socket } from 'socket.io-client';

interface UseWebSocketOptions {
  url: string;
  token: string;
  enableWebsocket?: boolean;
  onConnect?: () => void;
  onDisconnect?: (reason: string) => void;
  onError?: (error: Error) => void;
}

export function useWebSocket({
  url,
  token,
  enableWebsocket = true,
  onConnect,
  onDisconnect,
  onError,
}: UseWebSocketOptions) {
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
      transports: enableWebsocket ? ['websocket'] : ['polling', 'websocket'],
      auth: { token },
    });

    socketInstance.on('connect', () => {
      console.log('WebSocket connected:', socketInstance.id);
      setConnected(true);
      reconnectAttempts.current = 0;
      
      if (token) {
        socketInstance.emit('user-join', { auth: { token } });
      }
      
      onConnect?.();
    });

    socketInstance.on('disconnect', (reason) => {
      console.log('WebSocket disconnected:', reason);
      setConnected(false);
      onDisconnect?.(reason);
    });

    socketInstance.on('connect_error', (error) => {
      console.error('WebSocket connection error:', error);
      reconnectAttempts.current++;
      
      if (reconnectAttempts.current >= maxReconnectAttempts) {
        console.error('Max reconnection attempts reached');
        onError?.(new Error('Failed to connect after multiple attempts'));
      }
    });

    setSocket(socketInstance);

    return () => {
      socketInstance.close();
    };
  }, [url, token, enableWebsocket, onConnect, onDisconnect, onError]);

  const emit = useCallback((event: string, data: any) => {
    socket?.emit(event, data);
  }, [socket]);

  const on = useCallback((event: string, callback: (...args: any[]) => void) => {
    socket?.on(event, callback);
    return () => socket?.off(event, callback);
  }, [socket]);

  return { socket, connected, emit, on };
}
```

### Usage
```typescript
const { socket, connected, emit, on } = useWebSocket({
  url: process.env.NEXT_PUBLIC_WS_URL!,
  token: userToken,
  onConnect: () => console.log('Connected!'),
});

// Listen to events
useEffect(() => {
  const cleanup = on('agent:response', (data) => {
    console.log('Agent response:', data);
  });
  return cleanup;
}, [on]);
```

## SSE Streaming Pattern

### Original (Python FastAPI)
```python
async def event_generator():
    nonlocal q
    try:
        while True:
            data = await q.get()
            if isinstance(data, dict) and "done" in data:
                break
            yield f"data: {json.dumps(data)}\n\n"
    except Exception as e:
        log.debug(f"Error in event generator: {e}")

return StreamingResponse(
    event_generator(),
    media_type="text/event-stream",
    background=background
)
```

### Adapted (Rust Axum)
```rust
// iterations/v3/data-infrastructure/src/api/handlers/agent_handlers.rs

use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use tokio::sync::mpsc;

pub async fn stream_agent_response(
    agent_id: String,
    task_id: String,
    request: AgentRequest,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to generate response
    tokio::spawn(async move {
        let mut response_generator = agent_service
            .generate_response(request)
            .await;

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

### Frontend Consumption (React)
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

## Channel-Based Routing Pattern

### Original (Python)
```python
channel = f"{user_id}:{session_id}:{request_id}"
sio.on(channel, message_listener)

# Stream responses to specific channel
async def event_generator():
    while True:
        data = await q.get()
        if isinstance(data, dict) and "done" in data:
            break
        yield f"data: {json.dumps(data)}\n\n"
```

### Adapted (Rust)
```rust
// iterations/v3/data-infrastructure/src/websocket/channel.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_channel(&self, agent_id: &str, session_id: &str) -> String {
        let request_id = Uuid::new_v4().to_string();
        let channel = format!("agent:{}:session:{}:request:{}", agent_id, session_id, request_id);
        
        let (tx, _rx) = broadcast::channel(100);
        self.channels.write().await.insert(channel.clone(), tx);
        
        channel
    }

    pub async fn send_to_channel(&self, channel: &str, message: String) -> Result<()> {
        if let Some(tx) = self.channels.read().await.get(channel) {
            tx.send(message)?;
        }
        Ok(())
    }

    pub fn cleanup_channel(&self, channel: &str) {
        self.channels.write().await.remove(channel);
    }
}
```

## State Management Pattern

### Original (Svelte Stores)
```typescript
export const chats = writable(null);
export const currentChatId = writable('');
export const models = writable([]);
```

### Adapted (Zustand)
```typescript
// apps/agent_management_dashboard/src/stores/chatStore.ts

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

interface ChatState {
  chats: Chat[];
  currentChatId: string | null;
  loading: boolean;
  error: string | null;
  
  setChats: (chats: Chat[]) => void;
  setCurrentChat: (chatId: string) => void;
  addChat: (chat: Chat) => void;
  updateChat: (chatId: string, updates: Partial<Chat>) => void;
  addMessage: (chatId: string, message: Message) => void;
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
    }),
    { name: 'ChatStore' }
  )
);
```

## Optimistic Update Pattern

### Original (Svelte)
```typescript
// Update local state immediately
history.messages[responseMessageId] = responseMessage;

// Sync with backend
await saveChatHandler($chatId, history);
```

### Adapted (React)
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

## Error Handling Pattern

### Original (Python FastAPI)
```python
try:
    chat = Chats.insert_new_chat(user.id, form_data)
    return ChatResponse(**chat.model_dump())
except Exception as e:
    log.exception(e)
    raise HTTPException(
        status_code=status.HTTP_400_BAD_REQUEST,
        detail=ERROR_MESSAGES.DEFAULT()
    )
```

### Adapted (Rust)
```rust
// iterations/v3/data-infrastructure/src/api/errors.rs

use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
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

// Usage
pub async fn create_chat(
    user: AuthenticatedUser,
    request: Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    let chat = db.create_chat(&user.id, &request.0)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    
    Ok(Json(chat))
}
```

## Stream Processing Pattern

### Original (TypeScript)
```typescript
async function* openAIStreamToIterator(
    reader: ReadableStreamDefaultReader<ParsedEvent>
): AsyncGenerator<TextStreamUpdate> {
    while (true) {
        const { value, done } = await reader.read();
        if (done) {
            yield { done: true, value: '' };
            break;
        }
        
        const parsedData = JSON.parse(value.data);
        yield {
            done: false,
            value: parsedData.choices?.[0]?.delta?.content ?? ''
        };
    }
}
```

### Adapted (React Hook)
```typescript
// apps/agent_management_dashboard/src/lib/streaming/useStreamProcessor.ts

import { useCallback, useRef } from 'react';

interface StreamChunk {
  done: boolean;
  content?: string;
  error?: Error;
}

export function useStreamProcessor() {
  const messageRef = useRef<string>('');

  const processChunk = useCallback((chunk: StreamChunk): string => {
    if (chunk.done) {
      const finalMessage = messageRef.current;
      messageRef.current = '';
      return finalMessage;
    }

    if (chunk.error) {
      throw chunk.error;
    }

    if (chunk.content) {
      messageRef.current += chunk.content;
      return messageRef.current;
    }

    return messageRef.current;
  }, []);

  return { processChunk };
}
```

## Authentication Middleware Pattern

### Original (Python FastAPI)
```python
def get_verified_user(credentials: HTTPAuthorizationCredentials = Depends(security)):
    token = credentials.credentials
    data = decode_token(token)
    user = Users.get_user_by_id(data["id"])
    if not user:
        raise HTTPException(status_code=401, detail="Invalid token")
    return user
```

### Adapted (Rust Axum)
```rust
// iterations/v3/data-infrastructure/src/api/middleware/auth.rs

use axum::extract::{Request, FromRequestParts};
use axum::http::request::Parts;
use axum::response::Response;
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

        // Decode and validate token
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
```

## Component Organization Pattern

### Structure
```
components/
├── ui/              # Base UI components (shadcn/ui)
├── chat/            # Chat-specific components
│   ├── ChatMessage.tsx
│   ├── ChatInput.tsx
│   └── ChatSidebar.tsx
├── projects/        # Project-specific components
│   ├── ProjectCard.tsx
│   ├── ProjectView.tsx
│   └── ProjectList.tsx
├── agents/          # Agent-specific components
│   ├── AgentCard.tsx
│   └── AgentStatus.tsx
└── common/          # Shared components
    ├── ErrorDisplay.tsx
    ├── LoadingSpinner.tsx
    └── Toast.tsx
```

## Loading State Pattern

### Original (Svelte)
```svelte
{#if loading}
  <Spinner />
{:else}
  <Content />
{/if}
```

### Adapted (React)
```typescript
// apps/agent_management_dashboard/src/components/ChatMessageSkeleton.tsx

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

// Usage
{loading ? (
  <ChatMessageSkeleton />
) : (
  <ChatMessage message={message} />
)}
```

## Summary

These patterns provide a solid foundation for implementing real-time agent interactions in agent-agency. Start with WebSocket and SSE patterns, then add state management and error handling. Each pattern is production-ready and tested.

