//! WebSocket API Interface for Real-Time Execution Monitoring
//!
//! Provides WebSocket endpoints for real-time streaming of execution events,
//! enabling live monitoring and interactive control of autonomous tasks.

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{ws::WebSocketUpgrade, Path, Query},
    response::Response,
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::orchestration::tracking::{EventBus, ProgressTracker};
use crate::orchestration::planning::types::ExecutionEvent;

/// WebSocket API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketApiConfig {
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_seconds: u64,

    /// Connection timeout (seconds)
    pub connection_timeout_seconds: u64,

    /// Maximum message size (bytes)
    pub max_message_size_bytes: usize,

    /// Enable connection metrics
    pub enable_metrics: bool,

    /// Allow task control commands
    pub allow_task_control: bool,
}

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Subscribe to task events
    Subscribe { task_id: Uuid },

    /// Unsubscribe from task events
    Unsubscribe { task_id: Uuid },

    /// Request task status update
    GetStatus { task_id: Uuid },

    /// Cancel a task (if allowed)
    CancelTask { task_id: Uuid },

    /// Heartbeat ping
    Ping,

    /// Error response
    Error { message: String, code: String },
}

/// WebSocket response types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    /// Subscription confirmed
    Subscribed { task_id: Uuid },

    /// Unsubscription confirmed
    Unsubscribed { task_id: Uuid },

    /// Task status information
    TaskStatus {
        task_id: Uuid,
        status: String,
        progress: f32,
        current_phase: Option<String>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        updated_at: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// Execution event
    ExecutionEvent(ExecutionEvent),

    /// Task cancelled confirmation
    TaskCancelled { task_id: Uuid },

    /// Heartbeat pong
    Pong,

    /// Error response
    Error { message: String, code: String },
}

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    /// Auto-subscribe to task (optional)
    pub task_id: Option<String>,

    /// Include historical events
    pub include_history: Option<bool>,

    /// Connection mode
    pub mode: Option<WebSocketMode>,
}

#[derive(Debug, Deserialize)]
pub enum WebSocketMode {
    /// Monitor mode (read-only)
    Monitor,
    /// Control mode (allows commands)
    Control,
}

/// WebSocket API server
pub struct WebSocketApi {
    config: WebSocketApiConfig,
    event_bus: Arc<EventBus>,
    progress_tracker: Arc<ProgressTracker>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
}

#[derive(Debug)]
struct ConnectionInfo {
    connection_id: Uuid,
    task_id: Option<Uuid>,
    mode: WebSocketMode,
    sender: mpsc::UnboundedSender<WebSocketResponse>,
    last_activity: chrono::DateTime<chrono::Utc>,
}

impl WebSocketApi {
    pub fn new(
        config: WebSocketApiConfig,
        event_bus: Arc<EventBus>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            config,
            event_bus,
            progress_tracker,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle WebSocket upgrade for task monitoring
    pub async fn handle_connection(
        &self,
        ws: WebSocketUpgrade,
        Query(query): Query<WebSocketQuery>,
    ) -> Result<Response, WebSocketError> {
        Ok(ws.on_upgrade(move |socket| self.handle_socket(socket, query)))
    }

    /// Handle WebSocket upgrade for specific task
    pub async fn handle_task_connection(
        &self,
        ws: WebSocketUpgrade,
        Path(task_id): Path<Uuid>,
        Query(query): Query<WebSocketQuery>,
    ) -> Result<Response, WebSocketError> {
        let mut query = query;
        query.task_id = Some(task_id.to_string());
        Ok(ws.on_upgrade(move |socket| self.handle_socket(socket, query)))
    }

    /// Handle individual WebSocket connection
    async fn handle_socket(&self, socket: WebSocket, query: WebSocketQuery) {
        let connection_id = Uuid::new_v4();
        let mode = query.mode.unwrap_or(WebSocketMode::Monitor);

        tracing::info!("New WebSocket connection: {} (mode: {:?})", connection_id, mode);

        // Create communication channel
        let (sender, mut receiver) = mpsc::unbounded_channel::<WebSocketResponse>();
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Store connection info
        let connection_info = ConnectionInfo {
            connection_id,
            task_id: None,
            mode,
            sender: sender.clone(),
            last_activity: chrono::Utc::now(),
        };

        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection_info);
        }

        // Auto-subscribe to task if specified
        if let Some(task_id_str) = query.task_id {
            if let Ok(task_id) = Uuid::parse_str(&task_id_str) {
                if let Err(e) = self.subscribe_to_task(connection_id, task_id, query.include_history.unwrap_or(false)).await {
                    tracing::error!("Auto-subscription failed: {:?}", e);
                    let _ = sender.send(WebSocketResponse::Error {
                        message: format!("Auto-subscription failed: {:?}", e),
                        code: "AUTO_SUBSCRIBE_FAILED".to_string(),
                    });
                }
            }
        }

        // Spawn heartbeat task
        let heartbeat_sender = sender.clone();
        let heartbeat_interval = std::time::Duration::from_secs(self.config.heartbeat_interval_seconds);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(heartbeat_interval).await;
                if heartbeat_sender.send(WebSocketResponse::Pong).is_err() {
                    break; // Connection closed
                }
            }
        });

        // Handle incoming WebSocket messages
        let connections = Arc::clone(&self.connections);
        let event_bus = Arc::clone(&self.event_bus);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming WebSocket messages
                    message = ws_receiver.next() => {
                        match message {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = Self::handle_message(&text, connection_id, &connections, &event_bus, mode).await {
                                    tracing::error!("Message handling error: {:?}", e);
                                    let _ = sender.send(WebSocketResponse::Error {
                                        message: format!("Message handling failed: {:?}", e),
                                        code: "MESSAGE_ERROR".to_string(),
                                    });
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                tracing::info!("WebSocket connection closed: {}", connection_id);
                                Self::cleanup_connection(connection_id, &connections, &event_bus).await;
                                break;
                            }
                            Some(Err(e)) => {
                                tracing::error!("WebSocket error for {}: {:?}", connection_id, e);
                                Self::cleanup_connection(connection_id, &connections, &event_bus).await;
                                break;
                            }
                            None => {
                                tracing::info!("WebSocket stream ended: {}", connection_id);
                                Self::cleanup_connection(connection_id, &connections, &event_bus).await;
                                break;
                            }
                            _ => {} // Ignore other message types
                        }
                    }

                    // Send outgoing messages to WebSocket
                    message = receiver.recv() => {
                        match message {
                            Some(response) => {
                                let json = match serde_json::to_string(&response) {
                                    Ok(json) => json,
                                    Err(e) => {
                                        tracing::error!("JSON serialization error: {:?}", e);
                                        continue;
                                    }
                                };

                                if ws_sender.send(Message::Text(json)).await.is_err() {
                                    tracing::error!("Failed to send message to WebSocket: {}", connection_id);
                                    Self::cleanup_connection(connection_id, &connections, &event_bus).await;
                                    break;
                                }
                            }
                            None => break, // Channel closed
                        }
                    }
                }
            }
        });
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        connection_id: Uuid,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        event_bus: &Arc<EventBus>,
        mode: WebSocketMode,
    ) -> Result<(), WebSocketError> {
        let message: WebSocketMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::ParseError(e.to_string()))?;

        // Update last activity
        {
            let mut connections = connections.write().await;
            if let Some(conn) = connections.get_mut(&connection_id) {
                conn.last_activity = chrono::Utc::now();
            }
        }

        match message {
            WebSocketMessage::Subscribe { task_id } => {
                self.subscribe_to_task(connection_id, task_id, false).await?;
            }
            WebSocketMessage::Unsubscribe { task_id } => {
                self.unsubscribe_from_task(connection_id, task_id).await?;
            }
            WebSocketMessage::GetStatus { task_id } => {
                self.send_task_status(connection_id, task_id).await?;
            }
            WebSocketMessage::CancelTask { task_id } => {
                if matches!(mode, WebSocketMode::Control) {
                    self.cancel_task(connection_id, task_id).await?;
                } else {
                    self.send_error(connection_id, "Task control not allowed in monitor mode".to_string(), "CONTROL_DISABLED".to_string()).await;
                }
            }
            WebSocketMessage::Ping => {
                self.send_pong(connection_id).await;
            }
            WebSocketMessage::Error { .. } => {
                // Client sent error, ignore
            }
        }

        Ok(())
    }

    /// Subscribe connection to task events
    async fn subscribe_to_task(
        &self,
        connection_id: Uuid,
        task_id: Uuid,
        include_history: bool,
    ) -> Result<(), WebSocketError> {
        // Subscribe to event bus
        let subscription = self.event_bus.subscribe(task_id).await
            .map_err(|e| WebSocketError::SubscriptionError(e.to_string()))?;

        // Update connection info
        {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(&connection_id) {
                conn.task_id = Some(task_id);
            }
        }

        // Send subscription confirmation
        self.send_response(connection_id, WebSocketResponse::Subscribed { task_id }).await;

        // Send current status
        self.send_task_status(connection_id, task_id).await?;

        // Send historical events if requested
        if include_history {
            // In practice, this would retrieve historical events from the progress tracker
            // For now, just confirm subscription
        }

        // Forward events to connection
        let connections = Arc::clone(&self.connections);
        tokio::spawn(async move {
            loop {
                match subscription.recv().await {
                    Ok(event) => {
                        let response = WebSocketResponse::ExecutionEvent(event);
                        if let Err(_) = Self::send_to_connection(&connections, connection_id, response).await {
                            break; // Connection closed
                        }
                    }
                    Err(_) => break, // Subscription ended
                }
            }
        });

        Ok(())
    }

    /// Unsubscribe connection from task events
    async fn unsubscribe_from_task(
        &self,
        connection_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), WebSocketError> {
        // Unsubscribe from event bus
        self.event_bus.unsubscribe(task_id).await
            .map_err(|e| WebSocketError::SubscriptionError(e.to_string()))?;

        // Update connection info
        {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(&connection_id) {
                conn.task_id = None;
            }
        }

        // Send unsubscription confirmation
        self.send_response(connection_id, WebSocketResponse::Unsubscribed { task_id }).await;

        Ok(())
    }

    /// Send current task status
    async fn send_task_status(&self, connection_id: Uuid, task_id: Uuid) -> Result<(), WebSocketError> {
        let progress = self.progress_tracker.get_progress(task_id).await
            .map_err(|e| WebSocketError::ProgressError(e.to_string()))?;

        let response = if let Some(progress) = progress {
            WebSocketResponse::TaskStatus {
                task_id,
                status: format!("{:?}", progress.status).to_lowercase(),
                progress: progress.completion_percentage,
                current_phase: progress.current_phase,
                started_at: Some(progress.start_time),
                updated_at: Some(progress.last_update),
            }
        } else {
            WebSocketResponse::TaskStatus {
                task_id,
                status: "unknown".to_string(),
                progress: 0.0,
                current_phase: None,
                started_at: None,
                updated_at: None,
            }
        };

        self.send_response(connection_id, response).await;
        Ok(())
    }

    /// Cancel a task
    async fn cancel_task(&self, connection_id: Uuid, task_id: Uuid) -> Result<(), WebSocketError> {
        // In practice, this would cancel the task through the orchestrator
        // For now, just cancel in progress tracker
        self.progress_tracker.cancel_execution(task_id).await
            .map_err(|e| WebSocketError::CancellationError(e.to_string()))?;

        self.send_response(connection_id, WebSocketResponse::TaskCancelled { task_id }).await;
        Ok(())
    }

    /// Send pong response
    async fn send_pong(&self, connection_id: Uuid) {
        self.send_response(connection_id, WebSocketResponse::Pong).await;
    }

    /// Send error response
    async fn send_error(&self, connection_id: Uuid, message: String, code: String) {
        self.send_response(connection_id, WebSocketResponse::Error { message, code }).await;
    }

    /// Send response to connection
    async fn send_response(&self, connection_id: Uuid, response: WebSocketResponse) {
        let _ = Self::send_to_connection(&self.connections, connection_id, response).await;
    }

    /// Send response to specific connection
    async fn send_to_connection(
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        connection_id: Uuid,
        response: WebSocketResponse,
    ) -> Result<(), WebSocketError> {
        let connections = connections.read().await;
        if let Some(conn) = connections.get(&connection_id) {
            conn.sender.send(response)
                .map_err(|_| WebSocketError::SendError("Failed to send response".to_string()))?;
        }
        Ok(())
    }

    /// Clean up connection
    async fn cleanup_connection(
        connection_id: Uuid,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        event_bus: &Arc<EventBus>,
    ) {
        let mut connections = connections.write().await;
        if let Some(conn) = connections.remove(&connection_id) {
            // Unsubscribe from any active task
            if let Some(task_id) = conn.task_id {
                let _ = event_bus.unsubscribe(task_id).await;
            }
        }
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> WebSocketStats {
        let connections = self.connections.read().await;

        let mut task_subscriptions = HashMap::new();
        let mut mode_counts = HashMap::new();

        for conn in connections.values() {
            if let Some(task_id) = conn.task_id {
                *task_subscriptions.entry(task_id).or_insert(0) += 1;
            }

            let mode_str = match conn.mode {
                WebSocketMode::Monitor => "monitor",
                WebSocketMode::Control => "control",
            };
            *mode_counts.entry(mode_str.to_string()).or_insert(0) += 1;
        }

        WebSocketStats {
            total_connections: connections.len(),
            active_subscriptions: task_subscriptions.len(),
            subscriptions_per_task: task_subscriptions,
            connections_by_mode: mode_counts,
        }
    }
}

/// WebSocket connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketStats {
    pub total_connections: usize,
    pub active_subscriptions: usize,
    pub subscriptions_per_task: HashMap<Uuid, usize>,
    pub connections_by_mode: HashMap<String, usize>,
}

pub type Result<T> = std::result::Result<T, WebSocketError>;

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("JSON parse error: {0}")]
    ParseError(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Progress tracking error: {0}")]
    ProgressError(String),

    #[error("Task cancellation error: {0}")]
    CancellationError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}
