//! WebSocket API Implementation
//!
//! Provides WebSocket support for real-time communication with channel-based routing.
//! Adapted from open-webui patterns for agent-agency.
//!
//! @author @darianrosebrook

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

#[cfg(feature = "orchestration")]
use crate::api::{middleware::auth::validate_token_and_get_user_id, ApiState};

mod redis_manager;
pub use redis_manager::RedisSessionManager;

/// WebSocket manager for handling connections and channel-based routing
#[derive(Clone)]
pub struct WebSocketManager {
    /// Active channels: channel_id -> broadcast sender
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
    /// Active connections: connection_id -> user_id
    connections: Arc<RwLock<HashMap<String, String>>>,
    /// Cancellation tokens: channel_id -> cancellation sender
    cancellation_tokens: Arc<RwLock<HashMap<String, oneshot::Sender<()>>>>,
    /// Redis session manager for distributed sessions (optional)
    redis_manager: Option<Arc<RedisSessionManager>>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager without Redis (single instance)
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            cancellation_tokens: Arc::new(RwLock::new(HashMap::new())),
            redis_manager: None,
        }
    }

    /// Create a new WebSocket manager with Redis support (multi-instance)
    pub async fn with_redis(redis_url: Option<&str>) -> Result<Self, redis::RedisError> {
        let redis_manager = if let Some(url) = redis_url {
            Some(Arc::new(RedisSessionManager::new(Some(url)).await?))
        } else {
            None
        };

        Ok(Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            cancellation_tokens: Arc::new(RwLock::new(HashMap::new())),
            redis_manager,
        })
    }

    /// Create a new channel for agent communication
    /// Format: `agent:{agent_id}:task:{task_id}:session:{session_id}`
    ///
    /// This standardized format matches open-webui patterns and enables:
    /// - Isolated streams per request
    /// - Multiple concurrent requests per user
    /// - Clean cleanup on task completion
    ///
    /// Returns the channel ID and a cancellation receiver
    pub async fn create_channel(
        &self,
        agent_id: &str,
        task_id: &str,
        session_id: &str,
    ) -> (String, oneshot::Receiver<()>) {
        let channel = format!("agent:{}:task:{}:session:{}", agent_id, task_id, session_id);

        let (tx, _rx) = broadcast::channel(100);
        let (cancel_tx, cancel_rx) = oneshot::channel();

        self.channels.write().await.insert(channel.clone(), tx);
        self.cancellation_tokens
            .write()
            .await
            .insert(channel.clone(), cancel_tx);

        tracing::debug!("Created channel: {}", channel);
        (channel, cancel_rx)
    }

    /// Send a message to a specific channel
    pub async fn send_to_channel(&self, channel: &str, message: Message) -> Result<(), String> {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(channel) {
            tx.send(message)
                .map_err(|e| format!("Failed to send to channel: {}", e))?;
        }
        Ok(())
    }

    /// Cancel a stream for a specific channel
    pub async fn cancel_channel(&self, channel: &str) -> bool {
        let mut cancellation_tokens = self.cancellation_tokens.write().await;
        if let Some(cancel_tx) = cancellation_tokens.remove(channel) {
            let _ = cancel_tx.send(());
            tracing::info!("Cancelled stream for channel: {}", channel);
            true
        } else {
            tracing::warn!("Channel not found for cancellation: {}", channel);
            false
        }
    }

    /// Clean up a channel when done
    pub async fn cleanup_channel(&self, channel: &str) {
        self.channels.write().await.remove(channel);
        self.cancellation_tokens.write().await.remove(channel);
    }

    /// Register a connection
    pub async fn register_connection(&self, connection_id: String, user_id: String) {
        // Store in local connections map
        self.connections
            .write()
            .await
            .insert(connection_id.clone(), user_id.clone());

        // Register with Redis if available
        if let Some(ref redis) = self.redis_manager {
            if let Err(e) = redis.register_session(&user_id, &connection_id).await {
                tracing::warn!(
                    "Failed to register session in Redis: {}. Continuing with local storage.",
                    e
                );
            }
        }
    }

    /// Unregister a connection
    pub async fn unregister_connection(&self, connection_id: &str) {
        // Get user_id before removing from local map
        let user_id = self.connections.read().await.get(connection_id).cloned();

        // Remove from local connections
        self.connections.write().await.remove(connection_id);

        // Unregister from Redis if available
        if let Some(ref redis) = self.redis_manager {
            if let Some(user_id) = user_id {
                if let Err(e) = redis.unregister_session(&user_id, connection_id).await {
                    tracing::warn!(
                        "Failed to unregister session from Redis: {}. Continuing.",
                        e
                    );
                }
            }
        }
    }

    /// Get all session IDs for a user (across all instances if Redis is enabled)
    pub async fn get_user_session_ids(&self, user_id: &str) -> Vec<String> {
        if let Some(ref redis) = self.redis_manager {
            // Get sessions from Redis (includes sessions from all instances)
            redis.get_user_session_ids(user_id).await
        } else {
            // Get sessions from local connections only
            let connections = self.connections.read().await;
            connections
                .iter()
                .filter_map(|(conn_id, uid)| {
                    if uid == user_id {
                        Some(conn_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    /// Check if Redis is available
    pub fn is_redis_enabled(&self) -> bool {
        self.redis_manager.is_some()
    }

    /// Get user ID for a connection
    pub async fn get_user_id(&self, connection_id: &str) -> Option<String> {
        self.connections.read().await.get(connection_id).cloned()
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket connection handler with authentication
///
/// Validates the token from query parameters before accepting the connection.
/// Rejects connection with 401 Unauthorized if token is invalid or missing.
///
/// Query parameters:
/// - `token`: Bearer token for authentication (required)
#[cfg(feature = "orchestration")]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let connection_id = Uuid::new_v4().to_string();

    // Extract token from query parameters
    let token = match params.get("token") {
        Some(t) => t.clone(),
        None => {
            warn!("WebSocket connection rejected: missing token");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    // Validate token and get user_id before accepting connection
    let user_id = match validate_token_and_get_user_id(&token, &state.api.db_client).await {
        Ok(uid) => uid.to_string(),
        Err(status) => {
            warn!(
                "WebSocket connection rejected: token validation failed (status: {})",
                status
            );
            return status.into_response();
        }
    };

    info!(
        "WebSocket connection authenticated: user_id={}, connection_id={}",
        user_id, connection_id
    );

    // Accept connection with validated user_id
    let manager = state.websocket_manager.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, manager, connection_id, user_id))
}

/// WebSocket connection handler without authentication (fallback for non-orchestration builds)
#[cfg(not(feature = "orchestration"))]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<WebSocketManager>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let connection_id = Uuid::new_v4().to_string();
    let token = params.get("token").cloned();
    let user_id = token.unwrap_or_else(|| connection_id.clone());

    warn!("WebSocket connection accepted without authentication (orchestration feature disabled)");
    ws.on_upgrade(move |socket| handle_socket(socket, manager, connection_id, user_id))
}

async fn handle_socket(
    socket: WebSocket,
    manager: Arc<WebSocketManager>,
    connection_id: String,
    user_id: String,
) {
    // Register connection with validated user_id
    manager
        .register_connection(connection_id.clone(), user_id.clone())
        .await;
    info!(
        "WebSocket connection registered: connection_id={}, user_id={}",
        connection_id, user_id
    );

    let (mut sender, mut receiver) = socket.split();

    // Spawn task to handle incoming messages
    let manager_clone = manager.clone();
    let connection_id_clone = connection_id.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle text messages (e.g., subscribe to channel)
                    if let Ok(subscribe) = serde_json::from_str::<SubscribeMessage>(&text) {
                        // Handle subscription logic
                        tracing::info!("Subscription request: {:?}", subscribe);
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup on disconnect
        manager_clone
            .unregister_connection(&connection_id_clone)
            .await;
    });

    // Keep connection alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        if sender.send(Message::Ping(vec![])).await.is_err() {
            break;
        }
    }
}

#[derive(Debug, Deserialize)]
struct SubscribeMessage {
    #[allow(dead_code)] // Reserved for future use
    action: String,
    #[allow(dead_code)] // Reserved for future use
    channel: Option<String>,
}
