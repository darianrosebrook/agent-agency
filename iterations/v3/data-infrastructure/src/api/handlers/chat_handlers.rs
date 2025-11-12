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
use crate::chat_service::{ChatService, ChatSession, ChatMessage};
use axum::extract::Query;
use std::collections::HashMap;

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
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<ChatSessionResponse>>> {
    // Extract workspace_id from query parameters (TODO: Extract from authenticated user)
    let workspace_id = params.get("workspace_id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| super::super::ApiError::BadRequest("workspace_id query parameter required".to_string()))?;
    
    let archived = params.get("archived")
        .and_then(|s| s.parse::<bool>().ok());
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok());
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i32>().ok());
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Query sessions from database
    let sessions = chat_service.list_workspace_sessions(workspace_id, limit, offset, archived)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to query chat sessions: {}", e)))?;
    
    let responses: Vec<ChatSessionResponse> = sessions.into_iter()
        .map(session_to_response)
        .collect();
    
    ApiResult::Ok(Json(responses))
}

/// Search chat sessions
pub async fn search_chat_sessions(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<ChatSessionResponse>>> {
    // Extract parameters
    let workspace_id = params.get("workspace_id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| super::super::ApiError::BadRequest("workspace_id query parameter required".to_string()))?;
    
    let search_text = params.get("q")
        .ok_or_else(|| super::super::ApiError::BadRequest("q (search text) query parameter required".to_string()))?;
    
    let archived = params.get("archived")
        .and_then(|s| s.parse::<bool>().ok());
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok());
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i32>().ok());
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Search sessions in database
    let sessions = chat_service.search_sessions(workspace_id, search_text, archived, limit, offset)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to search chat sessions: {}", e)))?;
    
    let responses: Vec<ChatSessionResponse> = sessions.into_iter()
        .map(session_to_response)
        .collect();
    
    ApiResult::Ok(Json(responses))
}

/// Create a new chat session
pub async fn create_chat_session(
    State(state): State<ApiState>,
    Json(request): Json<CreateChatSessionRequest>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<ChatSessionResponse>> {
    // Extract workspace_id from query parameters (TODO: Extract from authenticated user)
    let workspace_id = params.get("workspace_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Create session in database
    let metadata = serde_json::json!({});
    let session = chat_service.create_session(workspace_id, None, request.title.clone(), &metadata)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to create chat session: {}", e)))?;
    
    ApiResult::Ok(Json(session_to_response(session)))
}

/// Get messages for a chat session
pub async fn get_chat_messages(
    State(state): State<ApiState>,
    Path(session_id_str): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<ChatMessageResponse>>> {
    // Parse session_id
    let session_id = Uuid::parse_str(&session_id_str)
        .map_err(|_| super::super::ApiError::BadRequest(format!("Invalid session_id: {}", session_id_str)))?;
    
    // Extract query parameters
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok());
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i32>().ok());
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Query messages from database
    let messages = chat_service.get_session_messages(session_id, limit, offset)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to query chat messages: {}", e)))?;
    
    let responses: Vec<ChatMessageResponse> = messages.into_iter()
        .map(message_to_response)
        .collect();
    
    ApiResult::Ok(Json(responses))
}

/// Search messages within a chat session
pub async fn search_chat_messages(
    State(state): State<ApiState>,
    Path(session_id_str): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<ChatMessageResponse>>> {
    // Parse session_id
    let session_id = Uuid::parse_str(&session_id_str)
        .map_err(|_| super::super::ApiError::BadRequest(format!("Invalid session_id: {}", session_id_str)))?;
    
    // Extract search text
    let search_text = params.get("q")
        .ok_or_else(|| super::super::ApiError::BadRequest("q (search text) query parameter required".to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok());
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i32>().ok());
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Search messages in database
    let messages = chat_service.search_messages(session_id, search_text, limit, offset)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to search chat messages: {}", e)))?;
    
    let responses: Vec<ChatMessageResponse> = messages.into_iter()
        .map(message_to_response)
        .collect();
    
    ApiResult::Ok(Json(responses))
}

/// Pin or unpin a chat session
pub async fn pin_chat_session(
    State(state): State<ApiState>,
    Path(session_id_str): Path<String>,
    Json(request): Json<PinSessionRequest>,
) -> ApiResult<Json<ChatSessionResponse>> {
    // Parse session_id
    let session_id = Uuid::parse_str(&session_id_str)
        .map_err(|_| super::super::ApiError::BadRequest(format!("Invalid session_id: {}", session_id_str)))?;
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Pin/unpin session in database
    chat_service.pin_session(session_id, request.pinned)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to pin/unpin chat session: {}", e)))?;
    
    // Fetch updated session
    let session = chat_service.get_session(session_id)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to fetch chat session: {}", e)))?
        .ok_or_else(|| super::super::ApiError::NotFound(format!("Chat session not found: {}", session_id_str)))?;
    
    ApiResult::Ok(Json(session_to_response(session)))
}

/// Bulk archive sessions
pub async fn bulk_archive_sessions(
    State(state): State<ApiState>,
    Json(request): Json<BulkOperationRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    // Parse session IDs
    let session_ids: std::result::Result<Vec<Uuid>, _> = request.session_ids.iter()
        .map(|s| Uuid::parse_str(s))
        .collect();
    
    let session_ids = session_ids
        .map_err(|_| super::super::ApiError::BadRequest("Invalid session_id format in request".to_string()))?;
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Bulk archive sessions in database
    let count = chat_service.bulk_archive_sessions(&session_ids)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to bulk archive sessions: {}", e)))?;
    
    ApiResult::Ok(Json(BulkOperationResponse { count }))
}

/// Bulk delete sessions
pub async fn bulk_delete_sessions(
    State(state): State<ApiState>,
    Json(request): Json<BulkOperationRequest>,
) -> ApiResult<Json<BulkOperationResponse>> {
    // Parse session IDs
    let session_ids: std::result::Result<Vec<Uuid>, _> = request.session_ids.iter()
        .map(|s| Uuid::parse_str(s))
        .collect();
    
    let session_ids = session_ids
        .map_err(|_| super::super::ApiError::BadRequest("Invalid session_id format in request".to_string()))?;
    
    // Create ChatService from database client
    let chat_service = ChatService::new(state.api.db_client.clone());
    
    // Bulk delete sessions in database
    let count = chat_service.bulk_delete_sessions(&session_ids)
        .await
        .map_err(|e| super::super::ApiError::InternalError(format!("Failed to bulk delete sessions: {}", e)))?;
    
    ApiResult::Ok(Json(BulkOperationResponse { count }))
}

/// Helper function to convert ChatSession to ChatSessionResponse
fn session_to_response(session: ChatSession) -> ChatSessionResponse {
    ChatSessionResponse {
        id: session.id.to_string(),
        title: session.title.unwrap_or_else(|| "Untitled".to_string()),
        created_at: session.created_at,
        updated_at: session.updated_at,
        message_count: session.message_count as u32,
    }
}

/// Helper function to convert ChatMessage to ChatMessageResponse
fn message_to_response(message: ChatMessage) -> ChatMessageResponse {
    ChatMessageResponse {
        id: message.id.to_string(),
        role: message.role,
        content: message.content,
        timestamp: message.created_at,
    }
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

