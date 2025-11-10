# API Integration Updates

**Date**: 2025-01-27  
**Status**: API URLs Updated  
**Author**: @darianrosebrook

## Summary

Updated all frontend stores to use the correct backend API endpoints. The stores were previously using incorrect URLs that wouldn't connect to the v3 backend API server.

## Changes Made

### Chat Store (`apps/agent_management_dashboard/src/lib/stores/chatStore.ts`)

**Updated API URLs**:
- `GET /api/chat/sessions` → `GET /api/v1/chat/sessions`
- `POST /api/chat/sessions` → `POST /api/v1/chat/sessions`
- `GET /api/chat/sessions/:id/messages` → `GET /api/v1/chat/sessions/:id/messages`
- `POST /api/chat/sessions/:id/messages` → `POST /api/v1/chat/sessions/:id/messages`

**Updated Default API URL**:
- `http://localhost:3000` → `http://localhost:8080` (matches backend server port)

### Project Store (`apps/agent_management_dashboard/src/lib/stores/projectStore.ts`)

**Updated API URLs**:
- `GET /api/projects` → `GET /api/v1/projects`
- `POST /api/projects` → `POST /api/v1/projects`
- `PATCH /api/projects/:id` → `PATCH /api/v1/projects/:id`
- `GET /api/projects/:id` → `GET /api/v1/projects/:id`
- `GET /api/projects/:id/tasks` → `GET /api/v1/projects/:id/tasks`
- `POST /api/projects/:id/tasks` → `POST /api/v1/projects/:id/tasks`
- `PATCH /api/projects/:id/tasks/:taskId` → `PATCH /api/v1/projects/:id/tasks/:taskId`

**Updated Default API URL**:
- `http://localhost:3000` → `http://localhost:8080` (matches backend server port)

## Backend API Endpoints Available

### Chat Endpoints
- `GET /api/v1/chat/sessions` - List chat sessions (stub - returns empty array)
- `POST /api/v1/chat/sessions` - Create chat session (stub - generates UUID)
- `GET /api/v1/chat/sessions/:session_id` - Get chat session (stub)
- `GET /api/v1/chat/sessions/:session_id/messages` - Get messages (stub - returns empty array)
- `POST /api/v1/chat/stream` - Stream agent response (fully implemented)
- `POST /api/v1/chat/stream/cancel` - Cancel active stream (fully implemented)

### Project Endpoints
- `GET /api/v1/projects` - List projects (implemented)
- `POST /api/v1/projects` - Create project (scaffold_project_handler)
- `GET /api/v1/projects/:project_id` - Get project (implemented)
- `PATCH /api/v1/projects/:project_id` - Update project (implemented)
- `DELETE /api/v1/projects/:project_id` - Delete project (implemented)
- `GET /api/v1/projects/:project_id/tasks` - Get project tasks (implemented)
- `POST /api/v1/projects/:project_id/tasks` - Create project task (implemented)
- `PATCH /api/v1/projects/:project_id/tasks/:task_id` - Update project task (implemented)
- `DELETE /api/v1/projects/:project_id/tasks/:task_id` - Delete project task (implemented)

## Next Steps

### Backend Implementation Needed

1. **Chat Service Database Integration**:
   - Implement `get_chat_sessions` handler to query database
   - Implement `create_chat_session` handler to save to database
   - Implement `get_chat_messages` handler to query messages
   - Add user authentication and workspace filtering

2. **Streaming Integration**:
   - Connect `Chat.tsx` component to use `useStreamingResponse` hook
   - Replace `simulateAIResponse` with real streaming API call
   - Connect to `/api/v1/chat/stream` endpoint

### Frontend Integration Needed

1. **Environment Configuration**:
   - Set `NEXT_PUBLIC_API_URL` environment variable in `.env.local`
   - Defaults to `http://localhost:8080` if not set

2. **Authentication Integration**:
   - Add auth token to API requests via `apiFetch` headers
   - Handle 401/403 errors and redirect to login
   - Store auth token securely

3. **Streaming Chat**:
   - Update `Chat.tsx` to use `useStreamingResponse` hook
   - Connect to `/api/v1/chat/stream` endpoint
   - Handle stream events and update messages in real-time

## Testing

To test the API integration:

1. **Start Backend Server**:
   ```bash
   cd iterations/v3/data-interfaces-adapters
   cargo run --bin api-server
   ```

2. **Start Frontend Dashboard**:
   ```bash
   cd apps/agent_management_dashboard
   npm run dev
   ```

3. **Verify API Calls**:
   - Open browser DevTools → Network tab
   - Navigate to chat/projects pages
   - Verify API calls are made to `http://localhost:8080/api/v1/*`
   - Check for CORS errors (backend should have CORS enabled)

## Notes

- All API URLs now match the backend server routes
- Default port changed from 3000 (Next.js) to 8080 (Rust API server)
- Stores are ready to connect once backend handlers are fully implemented
- Error handling and retry logic are already in place
- Loading states are implemented and ready

