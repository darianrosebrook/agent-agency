//! WebSocket Handler for Real-Time Execution Events
//!
//! Provides WebSocket endpoints for subscribing to execution events,
//! enabling real-time monitoring of autonomous task execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use axum::{
    extract::{Path, Query},
    response::Response,
    Extension,
};
// Stub types for WebSocket functionality
pub struct WebSocketUpgrade;
pub struct WebSocket;
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Close,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

use crate::planning::types::ExecutionEvent;
use crate::tracking::event_bus::{EventBus, EventSubscription};

/// WebSocket handler configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSocketConfig {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_seconds: u64,
    /// Connection timeout (seconds)
    pub connection_timeout_seconds: u64,
    /// Enable connection metrics
    pub enable_metrics: bool,
}

/// Query parameters for WebSocket connection
#[derive(Debug, serde::Deserialize)]
pub struct WebSocketQuery {
    /// Filter events since timestamp (optional)
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// Include historical events (default: false)
    pub include_history: Option<bool>,
}

/// Active WebSocket connection
struct WebSocketConnection {
    task_id: Uuid,
    subscription: EventSubscription,
    sender: mpsc::UnboundedSender<Message>,
}

/// WebSocket handler for execution events
pub struct WebSocketHandler {
    event_bus: Arc<EventBus>,
    config: WebSocketConfig,
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
}

impl WebSocketHandler {
    pub fn new(event_bus: Arc<EventBus>, config: WebSocketConfig) -> Self {
        Self {
            event_bus,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle WebSocket upgrade for task events
    pub async fn handle_task_events(
        ws: WebSocketUpgrade,
        Path(task_id): Path<Uuid>,
        Query(query): Query<WebSocketQuery>,
        Extension(handler): Extension<Arc<WebSocketHandler>>,
    ) -> Result<Response, WebSocketError> {
        // Check connection limit
        let connection_count = handler.connections.read().await.len();
        if connection_count >= handler.config.max_connections {
            return Err(WebSocketError::ConnectionLimitExceeded);
        }

        Ok(ws.on_upgrade(move |socket| handler.handle_connection(socket, task_id, query)))
    }

    /// Handle individual WebSocket connection
    async fn handle_connection(
        self: Arc<Self>,
        socket: WebSocket,
        task_id: Uuid,
        query: WebSocketQuery,
    ) {
        tracing::info!("New WebSocket connection for task: {}", task_id);

        match self.setup_connection(socket, task_id, query).await {
            Ok(connection_id) => {
                tracing::info!("WebSocket connection established: {}", connection_id);

                // Connection cleanup happens automatically when the connection closes
                // due to the Drop implementation
            }
            Err(e) => {
                tracing::error!("Failed to setup WebSocket connection for task {}: {:?}", task_id, e);
            }
        }
    }

    /// Setup WebSocket connection with event subscription
    async fn setup_connection(
        &self,
        socket: WebSocket,
        task_id: Uuid,
        query: WebSocketQuery,
    ) -> Result<Uuid, WebSocketError> {
        // Subscribe to events
        let subscription = self.event_bus.subscribe(task_id).await
            .map_err(|_| WebSocketError::SubscriptionFailed)?;

        // Split socket into sender and receiver
        let (sender, mut receiver) = socket.split();

        // Create communication channel
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Send initial connection message
        let _ = tx.send(Message::Text(serde_json::json!({
            "type": "connection_established",
            "task_id": task_id,
            "timestamp": chrono::Utc::now()
        }).to_string()));

        // Send historical events if requested
        if query.include_history.unwrap_or(false) {
            self.send_historical_events(&tx, task_id, query.since).await?;
        }

        // Create connection record
        let connection_id = Uuid::new_v4();
        let connection = WebSocketConnection {
            task_id,
            subscription,
            sender: tx.clone(),
        };

        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection);
        }

        // Spawn task to forward events to WebSocket
        let event_bus = Arc::clone(&self.event_bus);
        let connections = Arc::clone(&self.connections);

        tokio::spawn(async move {
            let mut event_rx = {
                let connections = connections.read().await;
                if let Some(conn) = connections.get(&connection_id) {
                    conn.subscription.receiver.resubscribe()
                } else {
                    return;
                }
            };

            loop {
                tokio::select! {
                    // Forward execution events to WebSocket
                    event = event_rx.recv() => {
                        match event {
                            Ok(execution_event) => {
                                let message = serde_json::json!({
                                    "type": "execution_event",
                                    "event": execution_event,
                                    "timestamp": chrono::Utc::now()
                                });

                                if tx.send(Message::Text(message.to_string())).is_err() {
                                    break; // Connection closed
                                }
                            }
                            Err(_) => break, // Channel closed
                        }
                    }

                    // Forward messages from application to WebSocket
                    message = rx.recv() => {
                        match message {
                            Some(msg) => {
                                if sender.send(msg).await.is_err() {
                                    break; // Connection closed
                                }
                            }
                            None => break, // Channel closed
                        }
                    }
                }
            }

            // Cleanup connection
            let mut connections = connections.write().await;
            connections.remove(&connection_id);
            let _ = event_bus.unsubscribe(task_id).await;

            tracing::debug!("WebSocket connection closed: {}", connection_id);
        });

        Ok(connection_id)
    }

    /// Send historical events to new connection
    async fn send_historical_events(
        &self,
        sender: &mpsc::UnboundedSender<Message>,
        task_id: Uuid,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), WebSocketError> {
        // TODO: Integrate with progress tracker for real historical event retrieval
        // - [ ] Connect to progress tracker service for historical data queries
        // - [ ] Implement event pagination and time-based filtering
        // - [ ] Add event serialization and WebSocket transmission
        // - [ ] Handle large event histories efficiently
        // - [ ] Implement real-time event streaming and updates
        let history_message = serde_json::json!({
            "type": "historical_events",
            "task_id": task_id,
            "events": [],
            "note": "Historical event replay not yet implemented"
        });

        sender.send(Message::Text(history_message.to_string()))
            .map_err(|_| WebSocketError::SendFailed)?;

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> WebSocketStats {
        let connections = self.connections.read().await;

        let mut task_connections = HashMap::new();
        for connection in connections.values() {
            *task_connections.entry(connection.task_id).or_insert(0) += 1;
        }

        WebSocketStats {
            total_connections: connections.len(),
            unique_tasks: task_connections.len(),
            connections_per_task: task_connections,
        }
    }

    /// Broadcast a message to all connections for a task
    pub async fn broadcast_to_task(&self, task_id: Uuid, message: serde_json::Value) -> Result<usize, WebSocketError> {
        let connections = self.connections.read().await;

        let mut sent_count = 0;
        for connection in connections.values() {
            if connection.task_id == task_id {
                let msg = Message::Text(message.to_string());
                if connection.sender.send(msg).is_ok() {
                    sent_count += 1;
                }
            }
        }

        Ok(sent_count)
    }

    /// Close all connections for a task
    pub async fn close_task_connections(&self, task_id: Uuid) -> Result<usize, WebSocketError> {
        let mut connections = self.connections.write().await;

        let mut closed_count = 0;
        connections.retain(|_, connection| {
            if connection.task_id == task_id {
                // Send close message
                let close_msg = Message::Text(serde_json::json!({
                    "type": "connection_closed",
                    "reason": "task_completed",
                    "timestamp": chrono::Utc::now()
                }).to_string());
                let _ = connection.sender.send(close_msg);
                closed_count += 1;
                false // Remove from map
            } else {
                true // Keep in map
            }
        });

        Ok(closed_count)
    }
}

/// WebSocket connection statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSocketStats {
    pub total_connections: usize,
    pub unique_tasks: usize,
    pub connections_per_task: HashMap<Uuid, usize>,
}

pub type Result<T> = std::result::Result<T, WebSocketError>;

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,

    #[error("Failed to subscribe to events")]
    SubscriptionFailed,

    #[error("Failed to send message")]
    SendFailed,

    #[error("WebSocket operation failed: {0}")]
    OperationError(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
}
