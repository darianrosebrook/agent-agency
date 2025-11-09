//! Chat API Handlers
//!
//! Handlers for chat sessions and streaming agent responses.
//! Preserves existing UI design while adding real-time functionality.

use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::super::Result as ApiResult;
use super::super::server::ApiState;

/// Request to stream agent response
#[derive(Debug, Clone, Deserialize)]
pub struct StreamAgentRequest {
    pub agent_id: String,
    pub session_id: String,
    pub task_id: Option<String>, // Optional task ID, defaults to generated UUID if not provided
    pub message: String,
    pub context_files: Option<Vec<String>>,
}

/// SSE event data
#[derive(Debug, Serialize)]
struct StreamEvent {
    content: Option<String>,
    done: bool,
    error: Option<String>,
}

/// Stream agent response via SSE
///
/// Creates a Server-Sent Events stream for real-time agent responses.
/// Uses channel-based routing to isolate streams per request.
/// 
/// Channel format: `agent:{agent_id}:task:{task_id}:session:{session_id}`
pub async fn stream_agent_response(
    State(state): State<ApiState>,
    Json(request): Json<StreamAgentRequest>,
) -> ApiResult<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    // Generate task_id if not provided (for chat messages, each message gets its own task_id)
    let task_id = request.task_id.as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let channel = state
        .websocket_manager
        .create_channel(&request.agent_id, &task_id, &request.session_id)
        .await;
    
    // Clone request for the spawned task
    let request_clone = request.clone();

    let (tx, rx) = mpsc::channel::<std::result::Result<Event, Infallible>>(100);

    // Spawn task to generate response
    let state_clone = state.clone();
    let channel_clone = channel.clone();
    tokio::spawn(async move {
        // TODO: Replace with actual agent service call
        // For now, simulate streaming response
        let response_text = format!("Agent response to: {}", request_clone.message);
        let words: Vec<&str> = response_text.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            let is_last = i == words.len() - 1;
            
            let event = StreamEvent {
                content: Some(format!("{} ", word)),
                done: is_last,
                error: None,
            };

            if tx
                .send(std::result::Result::<Event, Infallible>::Ok(Event::default().json_data(event).unwrap()))
                .await
                .is_err()
            {
                break;
            }

            // Small delay to simulate streaming
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Send done event
        let done_event = StreamEvent {
            content: None,
            done: true,
            error: None,
        };
        let _ = tx.send(std::result::Result::<Event, Infallible>::Ok(Event::default().json_data(done_event).unwrap())).await;

        // Cleanup channel
        state_clone.websocket_manager.cleanup_channel(&channel_clone).await;
    });

    ApiResult::Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

/// Get chat sessions for a workspace with optional search
pub async fn get_chat_sessions(
    State(_state): State<ApiState>,
    // TODO: Add user authentication and extract workspace_id
    // For now, accept workspace_id as query parameter
    // Query parameters: search, archived, limit, offset
) -> ApiResult<Json<Vec<ChatSessionResponse>>> {
    // TODO: Use ChatService to query database
    // TODO: Extract workspace_id from authenticated user
    // For now, return empty list
    ApiResult::Ok(Json(vec![]))
}

/// Search chat sessions
pub async fn search_chat_sessions(
    State(_state): State<ApiState>,
    // TODO: Add user authentication
    // Query parameters: q (search text), archived, limit, offset
) -> ApiResult<Json<Vec<ChatSessionResponse>>> {
    // TODO: Use ChatService.search_sessions()
    // For now, return empty list
    ApiResult::Ok(Json(vec![]))
}

/// Create a new chat session
pub async fn create_chat_session(
    State(_state): State<ApiState>,
    Json(request): Json<CreateChatSessionRequest>,
) -> ApiResult<Json<ChatSessionResponse>> {
    // TODO: Use ChatService to create session in database
    let session = ChatSessionResponse {
        id: Uuid::new_v4().to_string(),
        title: request.title.unwrap_or_else(|| "New Chat".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 0,
    };

    ApiResult::Ok(Json(session))
}

/// Get messages for a chat session
pub async fn get_chat_messages(
    State(_state): State<ApiState>,
    Path(_session_id): Path<String>,
    // TODO: Add query parameters: search, limit, offset
) -> ApiResult<Json<Vec<ChatMessageResponse>>> {
    // TODO: Use ChatService to query messages from database
    ApiResult::Ok(Json(vec![]))
}

/// Search messages within a chat session
pub async fn search_chat_messages(
    State(_state): State<ApiState>,
    Path(_session_id): Path<String>,
    // TODO: Add query parameter: q (search text), limit, offset
) -> ApiResult<Json<Vec<ChatMessageResponse>>> {
    // TODO: Use ChatService.search_messages()
    ApiResult::Ok(Json(vec![]))
}

/// Pin or unpin a chat session
pub async fn pin_chat_session(
    State(_state): State<ApiState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<PinSessionRequest>,
) -> ApiResult<Json<ChatSessionResponse>> {
    // TODO: Use ChatService.pin_session()
    // For now, return placeholder
    ApiResult::Ok(Json(ChatSessionResponse {
        id: _session_id,
        title: "Pinned Session".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 0,
    }))
}

/// Bulk archive sessions
pub async fn bulk_archive_sessions(
    State(_state): State<ApiState>,
    Json(_request): Json<BulkOperationRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    // TODO: Use ChatService.bulk_archive_sessions()
    ApiResult::Ok(Json(BulkOperationResponse {
        count: 0,
    }))
}

/// Bulk delete sessions
pub async fn bulk_delete_sessions(
    State(_state): State<ApiState>,
    Json(_request): Json<BulkOperationRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    // TODO: Use ChatService.bulk_delete_sessions()
    ApiResult::Ok(Json(BulkOperationResponse {
        count: 0,
    }))
}

#[derive(Debug, Serialize)]
pub struct ChatSessionResponse {
    pub id: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub message_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PinSessionRequest {
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct BulkOperationRequest {
    pub session_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    pub count: i32,
}

