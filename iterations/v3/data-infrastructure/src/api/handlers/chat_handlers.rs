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
use tokio::time::{timeout, Instant};
use uuid::Uuid;

use super::super::Result as ApiResult;
use super::super::server::ApiState;
use std::sync::Arc;

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
/// Implements timeout handling to prevent long-running streams.
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
    
    let (channel, cancel_rx) = state
        .websocket_manager
        .create_channel(&request.agent_id, &task_id, &request.session_id)
        .await;
    
    // Get stream timeout from config (default: 300 seconds / 5 minutes)
    let timeout_duration = Duration::from_secs(
        state.api.config().stream_timeout_seconds
    );
    
    // Clone request for the spawned task
    let request_clone = request.clone();

    let (tx, rx) = mpsc::channel::<std::result::Result<Event, Infallible>>(100);

    // Spawn task to generate response with timeout and cancellation handling
    let state_clone = state.clone();
    let channel_clone = channel.clone();
    let timeout_duration_clone = timeout_duration;
    tokio::spawn(async move {
        let start_time = Instant::now();

        // Wrap the stream generation in a timeout with cancellation support
        let stream_result = timeout(timeout_duration_clone, async {
            tokio::select! {
                result = async {
                    // Try to use CoreML inference if available
                    let response_text = if let Some(ref callback) = state_clone.coreml_inference_callback {
                        match callback(request_clone.message.clone()).await {
                            Ok(text) => {
                                tracing::info!("CoreML inference successful for chat message");
                                text
                            }
                            Err(e) => {
                                tracing::warn!("CoreML inference failed: {}", e);
                                format!("I received your message: '{}'. (CoreML inference unavailable: {})", request_clone.message, e)
                            }
                        }
                    } else {
                        tracing::debug!("CoreML inference callback not available, using fallback");
                        format!("I received your message: '{}'. CoreML orchestrator is not available.", request_clone.message)
                    };
                    
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
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }

                    // Send done event
                    let done_event = StreamEvent {
                        content: None,
                        done: true,
                        error: None,
                    };
                    let _ = tx.send(std::result::Result::<Event, Infallible>::Ok(Event::default().json_data(done_event).unwrap())).await;
                    Ok::<(), ()>(())
                } => {
                    result
                }
                _ = cancel_rx => {
                    // Stream was cancelled
                    Err(())
                }
            }
        }).await;

        // Handle cancellation (check if cancel_rx was received)
        // Note: If cancellation happens, the select! will return Err(())
        // If timeout happens, stream_result will be Err(Elapsed)
        match stream_result {
            Ok(Ok(())) => {
                // Stream completed successfully
                let duration = start_time.elapsed();
                tracing::debug!(
                    "Stream completed for channel {} in {:?}",
                    channel_clone,
                    duration
                );
            }
            Ok(Err(())) => {
                // Stream was cancelled
                let cancel_event = StreamEvent {
                    content: None,
                    done: true,
                    error: Some("Stream cancelled by user".to_string()),
                };
                let _ = tx.send(std::result::Result::<Event, Infallible>::Ok(Event::default().json_data(cancel_event).unwrap())).await;
                
                let duration = start_time.elapsed();
                tracing::info!(
                    "Stream cancelled for channel {} after {:?}",
                    channel_clone,
                    duration
                );
            }
            Err(_) => {
                // Stream timed out
                let timeout_event = StreamEvent {
                    content: None,
                    done: true,
                    error: Some(format!(
                        "Stream timeout after {} seconds",
                        timeout_duration_clone.as_secs()
                    )),
                };
                let _ = tx.send(std::result::Result::<Event, Infallible>::Ok(Event::default().json_data(timeout_event).unwrap())).await;
                
                let duration = start_time.elapsed();
                tracing::warn!(
                    "Stream timeout for channel {} after {:?} (configured timeout: {:?})",
                    channel_clone,
                    duration,
                    timeout_duration_clone
                );
            }
        }

        // Cleanup channel
        state_clone.websocket_manager.cleanup_channel(&channel_clone).await;
    });

    ApiResult::Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

/// Cancel an active stream
///
/// Cancels a streaming agent response by channel ID.
/// Channel format: `agent:{agent_id}:task:{task_id}:session:{session_id}`
#[derive(Debug, Deserialize)]
pub struct CancelStreamRequest {
    pub agent_id: String,
    pub task_id: String,
    pub session_id: String,
}

pub async fn cancel_stream(
    State(state): State<ApiState>,
    Json(request): Json<CancelStreamRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let channel = format!(
        "agent:{}:task:{}:session:{}",
        request.agent_id, request.task_id, request.session_id
    );

    let cancelled = state.websocket_manager.cancel_channel(&channel).await;

    if cancelled {
        ApiResult::Ok(Json(serde_json::json!({
            "success": true,
            "message": "Stream cancelled successfully",
            "channel": channel,
        })))
    } else {
        ApiResult::Err(super::super::ApiError::NotFound(format!(
            "Stream not found for channel: {}",
            channel
        )))
    }
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

