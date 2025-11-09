# Open-WebUI Code Examples & Migration Guide

**Date**: 2025-01-27  
**Purpose**: Practical code examples and step-by-step migration guides for adopting open-webui patterns

**Author**: @darianrosebrook

## Table of Contents

1. [Redis-Backed WebSocket Session Management](#redis-backed-websocket-session-management)
2. [Channel-Based Routing](#channel-based-routing)
3. [Error Handling Standardization](#error-handling-standardization)
4. [State Management with Selectors](#state-management-with-selectors)
5. [Streaming Enhancements](#streaming-enhancements)
6. [Authentication Middleware](#authentication-middleware)
7. [Component Reorganization](#component-reorganization)

## Redis-Backed WebSocket Session Management

### Example Implementation

#### Backend: Redis Session Manager

```rust
// iterations/v3/data-infrastructure/src/websocket/redis_manager.rs

use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct RedisSessionManager {
    redis_client: redis::Client,
    local_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RedisSessionManager {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis_client: client,
            local_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register_session(&self, user_id: &str, session_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let key = format!("user_sessions:{}", user_id);
        
        // Add to Redis set
        conn.sadd(&key, session_id).await?;
        
        // Set expiration (24 hours)
        conn.expire(&key, 86400).await?;
        
        // Update local cache
        let mut sessions = self.local_sessions.write().await;
        sessions.entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(session_id.to_string());
        
        Ok(())
    }

    pub async fn unregister_session(&self, user_id: &str, session_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let key = format!("user_sessions:{}", user_id);
        
        // Remove from Redis set
        conn.srem(&key, session_id).await?;
        
        // Update local cache
        let mut sessions = self.local_sessions.write().await;
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            user_sessions.retain(|s| s != session_id);
            if user_sessions.is_empty() {
                sessions.remove(user_id);
            }
        }
        
        Ok(())
    }

    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<String>, redis::RedisError> {
        // Check local cache first
        {
            let sessions = self.local_sessions.read().await;
            if let Some(user_sessions) = sessions.get(user_id) {
                return Ok(user_sessions.clone());
            }
        }
        
        // Fallback to Redis
        let mut conn = self.redis_client.get_async_connection().await?;
        let key = format!("user_sessions:{}", user_id);
        let sessions: Vec<String> = conn.smembers(&key).await?;
        
        // Update local cache
        {
            let mut local = self.local_sessions.write().await;
            local.insert(user_id.to_string(), sessions.clone());
        }
        
        Ok(sessions)
    }

    pub async fn broadcast_to_user(&self, user_id: &str, event: &str, data: &serde_json::Value) -> Result<(), redis::RedisError> {
        let sessions = self.get_user_sessions(user_id).await?;
        
        // Emit to all user sessions
        for session_id in sessions {
            // Implementation depends on your WebSocket library
            // This is a placeholder for the actual emit logic
            tracing::info!("Broadcasting to session {}: {} - {:?}", session_id, event, data);
        }
        
        Ok(())
    }
}
```

#### Integration with WebSocket Manager

```rust
// iterations/v3/data-infrastructure/src/websocket/mod.rs

use crate::websocket::redis_manager::RedisSessionManager;

pub struct WebSocketManager {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
    connections: Arc<RwLock<HashMap<String, String>>>, // connection_id -> user_id
    redis_manager: Option<Arc<RedisSessionManager>>,
}

impl WebSocketManager {
    pub fn new(redis_manager: Option<Arc<RedisSessionManager>>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            redis_manager,
        }
    }

    pub async fn register_connection(&self, connection_id: String, user_id: String) {
        self.connections.write().await.insert(connection_id.clone(), user_id.clone());
        
        // Register with Redis if available
        if let Some(redis) = &self.redis_manager {
            let _ = redis.register_session(&user_id, &connection_id).await;
        }
    }

    pub async fn unregister_connection(&self, connection_id: &str) {
        if let Some(user_id) = self.connections.write().await.remove(connection_id) {
            // Unregister from Redis if available
            if let Some(redis) = &self.redis_manager {
                let _ = redis.unregister_session(&user_id, connection_id).await;
            }
        }
    }
}
```

### Migration Steps

1. **Add Redis dependency**
   ```toml
   # Cargo.toml
   [dependencies]
   redis = { version = "0.24", features = ["tokio-comp"] }
   ```

2. **Create Redis manager module**
   - Create `src/websocket/redis_manager.rs`
   - Copy example implementation
   - Add to `src/websocket/mod.rs`

3. **Update WebSocket manager**
   - Add Redis manager as optional dependency
   - Update `register_connection` and `unregister_connection`
   - Test with and without Redis

4. **Configure Redis URL**
   ```rust
   // config.rs
   pub struct Config {
       pub redis_url: Option<String>,
   }
   ```

5. **Test multi-instance**
   - Start two API server instances
   - Connect WebSocket to both
   - Verify sessions are shared

## Channel-Based Routing

### Example Implementation

#### Backend: Channel Manager

```rust
// iterations/v3/data-infrastructure/src/websocket/channel_manager.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, ChannelInfo>>>,
    max_channels_per_user: usize,
}

struct ChannelInfo {
    sender: broadcast::Sender<Message>,
    user_id: String,
    created_at: std::time::Instant,
    subscriber_count: usize,
}

impl ChannelManager {
    pub fn new(max_channels_per_user: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            max_channels_per_user,
        }
    }

    pub fn create_channel(&self, channel_id: &str, user_id: &str) -> Result<broadcast::Sender<Message>, String> {
        let mut channels = self.channels.write().unwrap();
        
        // Check user channel limit
        let user_channel_count = channels.values()
            .filter(|info| info.user_id == user_id)
            .count();
        
        if user_channel_count >= self.max_channels_per_user {
            return Err(format!("User {} has reached channel limit", user_id));
        }
        
        // Create new channel
        let (tx, _) = broadcast::channel(100);
        channels.insert(channel_id.to_string(), ChannelInfo {
            sender: tx.clone(),
            user_id: user_id.to_string(),
            created_at: std::time::Instant::now(),
            subscriber_count: 0,
        });
        
        Ok(tx)
    }

    pub fn subscribe_to_channel(&self, channel_id: &str) -> Result<broadcast::Receiver<Message>, String> {
        let channels = self.channels.read().unwrap();
        let channel_info = channels.get(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        
        Ok(channel_info.sender.subscribe())
    }

    pub fn cleanup_channel(&self, channel_id: &str) {
        let mut channels = self.channels.write().unwrap();
        channels.remove(channel_id);
    }

    pub fn cleanup_old_channels(&self, max_age: std::time::Duration) {
        let mut channels = self.channels.write().unwrap();
        let now = std::time::Instant::now();
        
        channels.retain(|_, info| {
            now.duration_since(info.created_at) < max_age
        });
    }

    pub fn get_channel_format(agent_id: &str, task_id: &str, session_id: &str) -> String {
        format!("agent:{}:task:{}:session:{}", agent_id, task_id, session_id)
    }
}
```

#### Usage in Handler

```rust
// iterations/v3/data-infrastructure/src/api/handlers/chat_handlers.rs

use crate::websocket::channel_manager::ChannelManager;

pub async fn stream_agent_response(
    State(state): State<ApiState>,
    Json(request): Json<StreamAgentRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create channel ID
    let channel_id = ChannelManager::get_channel_format(
        &request.agent_id,
        &request.session_id,
        &Uuid::new_v4().to_string(),
    );
    
    // Create channel
    let sender = state.channel_manager.create_channel(&channel_id, &request.user_id)
        .map_err(|e| ApiError::InvalidRequest(e))?;
    
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    
    // Spawn task to generate response
    let state_clone = state.clone();
    let channel_id_clone = channel_id.clone();
    tokio::spawn(async move {
        // Generate response and send to channel
        // ... response generation logic ...
        
        // Cleanup channel when done
        state_clone.channel_manager.cleanup_channel(&channel_id_clone);
    });
    
    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}
```

### Migration Steps

1. **Create channel manager module**
   - Create `src/websocket/channel_manager.rs`
   - Copy example implementation
   - Add to `src/websocket/mod.rs`

2. **Update handlers**
   - Replace direct channel creation with channel manager
   - Use standardized channel naming
   - Add channel cleanup

3. **Add channel limits**
   - Configure max channels per user
   - Add rate limiting
   - Monitor channel usage

4. **Test channel isolation**
   - Create multiple concurrent streams
   - Verify channels don't interfere
   - Test channel cleanup

## Error Handling Standardization

### Example Implementation

#### Backend: Standardized Error Response

```rust
// iterations/v3/data-infrastructure/src/api/api_errors.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub status: u16,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    RateLimitExceeded(String),
    InternalError(String),
}

impl ApiError {
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            ApiError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            ApiError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::RateLimitExceeded(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: error_message.clone(),
            code: self.error_code().to_string(),
            status: status.as_u16(),
            details: None,
            request_id: None,
        });

        (status, body).into_response()
    }
}
```

#### Frontend: Error Component

```typescript
// apps/agent_management_dashboard/src/components/chat/ChatMessageError.tsx

import { Info } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useMemo } from 'react';

interface ChatMessageErrorProps {
  error: unknown;
  onRetry?: () => void;
}

export function ChatMessageError({ error, onRetry }: ChatMessageErrorProps) {
  const errorMessage = useMemo(() => {
    if (typeof error === 'string') {
      return error;
    }
    
    if (error && typeof error === 'object') {
      // Handle ErrorResponse format
      if ('error' in error && typeof error.error === 'string') {
        return error.error;
      }
      
      // Handle nested error objects
      if ('error' in error && error.error && typeof error.error === 'object') {
        if ('message' in error.error && typeof error.error.message === 'string') {
          return error.error.message;
        }
      }
      
      // Handle detail field (FastAPI style)
      if ('detail' in error && typeof error.detail === 'string') {
        return error.detail;
      }
      
      // Handle message field
      if ('message' in error && typeof error.message === 'string') {
        return error.message;
      }
      
      // Fallback to JSON stringify
      return JSON.stringify(error);
    }
    
    return 'An unknown error occurred';
  }, [error]);

  const errorCode = useMemo(() => {
    if (error && typeof error === 'object' && 'code' in error) {
      return error.code as string;
    }
    return undefined;
  }, [error]);

  return (
    <div className="flex my-2 gap-2.5 border px-4 py-3 border-red-600/10 bg-red-600/10 rounded-lg">
      <Info className="size-5 text-red-700 dark:text-red-400 self-start mt-0.5" />
      <div className="self-center text-sm flex-1">
        {errorMessage}
        {errorCode && (
          <span className="ml-2 text-xs text-red-600/60">({errorCode})</span>
        )}
      </div>
      {onRetry && (
        <Button 
          onClick={onRetry} 
          variant="outline" 
          size="sm"
          className="self-center"
        >
          Retry
        </Button>
      )}
    </div>
  );
}
```

### Migration Steps

1. **Update error types**
   - Replace existing `ApiError` with standardized version
   - Add `ErrorResponse` struct
   - Update all error creation sites

2. **Update handlers**
   - Return `ApiError` instead of raw errors
   - Use appropriate error variants
   - Add error codes

3. **Create error component**
   - Create `ChatMessageError.tsx`
   - Handle all error formats
   - Add retry button support

4. **Update error parsing**
   - Update `parseApiError` utility
   - Handle new error format
   - Test with various error types

## State Management with Selectors

### Example Implementation

#### Store with Selectors

```typescript
// apps/agent_management_dashboard/src/lib/stores/chat/chatSelectors.ts

import { useChatStore } from './chatStore';

/**
 * Selector for current chat
 */
export const useCurrentChat = () => {
  return useChatStore(state => {
    if (!state.currentChatId) return null;
    return state.chats.find(chat => chat.id === state.currentChatId) ?? null;
  });
};

/**
 * Selector for current chat messages
 */
export const useCurrentChatMessages = () => {
  return useChatStore(state => {
    const chat = state.chats.find(c => c.id === state.currentChatId);
    return chat?.messages ?? [];
  });
};

/**
 * Selector for chat by ID
 */
export const useChatById = (chatId: string) => {
  return useChatStore(state => 
    state.chats.find(chat => chat.id === chatId)
  );
};

/**
 * Selector for chat list (sorted by last accessed)
 */
export const useChatList = () => {
  return useChatStore(state => 
    [...state.chats].sort((a, b) => 
      b.lastAccessed.getTime() - a.lastAccessed.getTime()
    )
  );
};

/**
 * Selector for unread message count
 */
export const useUnreadCount = () => {
  return useChatStore(state => 
    state.chats.reduce((count, chat) => {
      return count + (chat.unreadCount ?? 0);
    }, 0)
  );
};
```

#### Usage in Components

```typescript
// Before: Direct store access
const ChatComponent = () => {
  const chats = useChatStore(state => state.chats);
  const currentChatId = useChatStore(state => state.currentChatId);
  const currentChat = chats.find(chat => chat.id === currentChatId);
  
  // Component logic...
};

// After: Using selectors
const ChatComponent = () => {
  const currentChat = useCurrentChat();
  const messages = useCurrentChatMessages();
  
  // Component logic...
};
```

### Migration Steps

1. **Create selector files**
   - Create `stores/chat/chatSelectors.ts`
   - Create `stores/project/projectSelectors.ts`
   - Create `stores/agent/agentSelectors.ts`

2. **Define selectors**
   - Extract common selector patterns
   - Add memoization where needed
   - Document selector usage

3. **Update components**
   - Replace direct store access with selectors
   - Test performance improvements
   - Verify no regressions

4. **Add selector tests**
   - Test selector logic
   - Test memoization
   - Test edge cases

## Streaming Enhancements

### Example Implementation

#### Frontend: Enhanced Stream Hook

```typescript
// apps/agent_management_dashboard/src/lib/hooks/useStreamingResponse.ts

import { createParser, ParsedEvent, ReconnectInterval } from 'eventsource-parser';

export function useStreamingResponse(options: StreamingOptions) {
  // ... existing state ...
  
  const start = useCallback(async () => {
    // ... existing setup ...
    
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    const parser = createParser((event: ParsedEvent | ReconnectInterval) => {
      if (event.type === 'event') {
        if (event.data === '[DONE]') {
          setState(prev => ({ ...prev, isStreaming: false }));
          onComplete?.(contentRef.current);
          return;
        }
        
        try {
          const parsed = JSON.parse(event.data);
          
          if (parsed.done) {
            setState(prev => ({ ...prev, isStreaming: false }));
            onComplete?.(contentRef.current);
            return;
          }
          
          if (parsed.error) {
            throw new Error(parsed.error);
          }
          
          if (parsed.content) {
            // Handle chunk splitting for UX
            const chunks = splitLargeDeltas(parsed.content, 10);
            
            chunks.forEach(chunk => {
              contentRef.current += chunk;
              
              // Debounce fast updates
              if (chunks.length > 1) {
                debouncedUpdate(contentRef.current);
              } else {
                setState(prev => ({
                  ...prev,
                  content: contentRef.current,
                }));
              }
              
              onChunk?.(chunk);
            });
          }
        } catch (e) {
          // Handle parse errors
          console.error('Stream parse error:', e);
        }
      }
    });
    
    // Process stream
    try {
      while (true) {
        const { done, value } = await reader.read();
        
        if (done) {
          break;
        }
        
        const chunk = decoder.decode(value, { stream: true });
        parser.feed(chunk);
      }
    } catch (error) {
      // Handle errors
    }
  }, [/* dependencies */]);
  
  // ... rest of implementation ...
}

function splitLargeDeltas(chunk: string, maxChunkSize: number = 10): string[] {
  if (chunk.length <= maxChunkSize) {
    return [chunk];
  }
  
  const chunks: string[] = [];
  for (let i = 0; i < chunk.length; i += maxChunkSize) {
    chunks.push(chunk.slice(i, i + maxChunkSize));
  }
  return chunks;
}

const debouncedUpdate = debounce((content: string) => {
  // Update state
}, 50);
```

### Migration Steps

1. **Install eventsource-parser**
   ```bash
   npm install eventsource-parser
   ```

2. **Update useStreamingResponse**
   - Add parser integration
   - Add chunk splitting
   - Add debouncing

3. **Test stream parsing**
   - Test with various chunk sizes
   - Test with partial chunks
   - Test error handling

4. **Add stream cancellation**
   - Add cancel endpoint
   - Update hook to support cancellation
   - Test cancellation flow

## Authentication Middleware

### Example Implementation

#### Backend: Auth Middleware

```rust
// iterations/v3/data-infrastructure/src/api/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use headers::Authorization;
use headers::authorization::Bearer;
use crate::api::api_errors::{ApiError, Result};

pub async fn auth_middleware(
    State(state): State<ApiState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let token = bearer.token();
    
    // Decode and validate token
    let claims = decode_token(token)
        .map_err(|_| ApiError::AuthenticationError("Invalid token".to_string()))?;
    
    // Get user from database
    let user = state.db_client.get_user_by_id(&claims.user_id).await?
        .ok_or_else(|| ApiError::AuthenticationError("User not found".to_string()))?;
    
    // Add user to request extensions
    request.extensions_mut().insert(user);
    
    Ok(next.run(request).await)
}

pub async fn admin_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    // Get user from extensions (set by auth_middleware)
    let user = request.extensions()
        .get::<User>()
        .ok_or_else(|| ApiError::AuthenticationError("User not authenticated".to_string()))?;
    
    if !user.is_admin {
        return Err(ApiError::AuthorizationError("Admin access required".to_string()));
    }
    
    Ok(next.run(request).await)
}

// Helper to extract user from request
pub fn get_user_from_request(request: &Request) -> Option<&User> {
    request.extensions().get::<User>()
}
```

#### Usage in Handlers

```rust
// iterations/v3/data-infrastructure/src/api/handlers/chat_handlers.rs

use crate::api::middleware::auth::get_user_from_request;

pub async fn create_chat_session(
    State(state): State<ApiState>,
    request: Request,
    Json(form): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>> {
    let user = get_user_from_request(&request)
        .ok_or_else(|| ApiError::AuthenticationError("User not authenticated".to_string()))?;
    
    // Use user.id for chat creation
    let chat = state.db_client.create_chat(user.id, form).await?;
    
    Ok(Json(chat.into()))
}
```

### Migration Steps

1. **Create middleware module**
   - Create `src/api/middleware/auth.rs`
   - Copy example implementation
   - Add to `src/api/mod.rs`

2. **Update handlers**
   - Add auth middleware to routes
   - Extract user from request
   - Update handler signatures

3. **Add admin middleware**
   - Create admin middleware
   - Apply to admin routes
   - Test role-based access

4. **Update WebSocket auth**
   - Extract user from token
   - Store user_id with connection
   - Test authentication flow

## Component Reorganization

### Migration Steps

1. **Create new structure**
   ```
   components/
   ├── chat/
   │   ├── ChatMessage.tsx
   │   ├── ChatMessageError.tsx
   │   ├── ChatMessageLoading.tsx
   │   └── ChatInput.tsx
   ├── project/
   │   ├── ProjectList.tsx
   │   ├── ProjectCard.tsx
   │   └── ProjectView.tsx
   ├── agent/
   │   ├── AgentList.tsx
   │   └── AgentCard.tsx
   └── ui/
       └── [shadcn components]
   ```

2. **Move components**
   - Use IDE refactoring tools
   - Update imports automatically
   - Verify no broken imports

3. **Update imports**
   - Search and replace import paths
   - Test build
   - Fix any remaining imports

4. **Document structure**
   - Create component organization guide
   - Document naming conventions
   - Add examples

## Testing Examples

### Unit Test: Error Handling

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_error_response_format() {
        let error = ApiError::NotFound("Resource not found".to_string());
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        // Verify response body format
    }
}
```

### Integration Test: WebSocket

```rust
#[tokio::test]
async fn test_websocket_channel_isolation() {
    // Create two channels
    let channel1 = manager.create_channel("channel1", "user1").unwrap();
    let channel2 = manager.create_channel("channel2", "user1").unwrap();
    
    // Send message to channel1
    channel1.send("message1").unwrap();
    
    // Verify channel2 doesn't receive it
    // ...
}
```

### Frontend Test: Selectors

```typescript
describe('chatSelectors', () => {
  it('should return current chat', () => {
    const store = useChatStore.getState();
    store.setCurrentChatId('chat1');
    store.setChats([{ id: 'chat1', messages: [] }]);
    
    const currentChat = useCurrentChat();
    expect(currentChat?.id).toBe('chat1');
  });
});
```

## Conclusion

These code examples provide practical implementations of the patterns identified in the comparison document. Use them as starting points and adapt to your specific needs.

For questions or issues during migration, refer to:
- Architecture Comparison Document
- Implementation Roadmap
- Open-WebUI source code

