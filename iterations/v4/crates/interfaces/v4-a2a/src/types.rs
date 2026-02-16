//! A2A Protocol Types
//!
//! Wire types for the Agent-to-Agent protocol (v0.3).
//! Reference: <https://a2a-protocol.org/latest/specification/>
//!
//! All types use camelCase serialization to match the A2A JSON wire format.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Agent Card — served at /.well-known/agent-card.json
// ============================================================================

/// Agent Card describes an agent's capabilities for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    /// Human-readable agent name.
    pub name: String,
    /// Primary A2A endpoint URL.
    pub url: String,
    /// Agent version string.
    pub version: String,
    /// Skills this agent offers.
    pub skills: Vec<AgentSkill>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<AgentCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    /// Accepted input content types (default: `["text"]`).
    #[serde(default = "default_modes")]
    pub default_input_modes: Vec<String>,
    /// Produced output content types (default: `["text"]`).
    #[serde(default = "default_modes")]
    pub default_output_modes: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_authenticated_extended_card: Option<bool>,
}

fn default_modes() -> Vec<String> {
    vec!["text".to_string()]
}

/// Organization or individual that provides the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProvider {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_contact: Option<String>,
}

/// What the agent supports at the protocol level.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a2a_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_transition_history: Option<bool>,
}

/// A specific capability the agent advertises.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    /// Unique identifier for this skill.
    pub id: String,
    /// Human-readable skill name.
    pub name: String,
    /// What this skill does.
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modes: Option<Vec<String>>,
}

/// Authentication scheme (OpenAPI 3.2 aligned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        #[serde(rename = "in")]
        location: String,
        name: String,
    },
    #[serde(rename = "http")]
    Http {
        scheme: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },
    #[serde(rename = "oauth2")]
    OAuth2 { flows: serde_json::Value },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        open_id_connect_url: String,
    },
    #[serde(rename = "mutualTLS")]
    MutualTls {},
}

// ============================================================================
// Task — the unit of work in A2A
// ============================================================================

/// A2A Task: represents a unit of work delegated to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Groups related interactions across tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Current task status.
    pub status: TaskStatus,
    /// Literal discriminator.
    #[serde(default = "task_kind")]
    pub kind: String,
    /// Conversation history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,
    /// Generated output artifacts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    /// Arbitrary metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

fn task_kind() -> String {
    "task".to_string()
}

/// Current status of a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    /// Current state in the lifecycle.
    pub state: TaskState,
    /// Optional status message from the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    /// ISO 8601 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Task lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    /// Received, not yet processing.
    Submitted,
    /// Actively being processed.
    Working,
    /// Agent needs more input from the client.
    InputRequired,
    /// Terminal: completed successfully.
    Completed,
    /// Terminal: cancelled by client or server.
    Canceled,
    /// Terminal: error occurred.
    Failed,
    /// State is unknown.
    Unknown,
}

impl TaskState {
    /// Whether this is a terminal state (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Canceled | Self::Failed)
    }
}

// ============================================================================
// Message — conversation between client and agent
// ============================================================================

/// A message in the A2A conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Unique message identifier (UUID recommended).
    pub message_id: String,
    /// Who sent this message.
    pub role: Role,
    /// Content parts.
    pub parts: Vec<Part>,
    /// Literal discriminator.
    #[serde(default = "message_kind")]
    pub kind: String,
    /// Associate with an existing task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Group related interactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Reference other tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_task_ids: Option<Vec<String>>,
    /// Arbitrary metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

fn message_kind() -> String {
    "message".to_string()
}

impl Message {
    /// Create a simple text message.
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            role,
            parts: vec![Part::Text {
                text: text.into(),
                metadata: None,
            }],
            kind: "message".to_string(),
            task_id: None,
            context_id: None,
            reference_task_ids: None,
            metadata: None,
        }
    }
}

/// Message sender role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Client / human / orchestrator.
    User,
    /// Server / AI agent.
    Agent,
}

// ============================================================================
// Part — content within a message or artifact
// ============================================================================

/// Discriminated union of content types within messages and artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Part {
    /// Plain text content.
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    /// File content (inline bytes or URI reference).
    #[serde(rename = "file")]
    File {
        file: FileContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    /// Structured data (arbitrary JSON).
    #[serde(rename = "data")]
    Data {
        data: HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
}

/// File content: either inline bytes or a URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileContent {
    /// File referenced by URI.
    Uri {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    /// File content as base64-encoded bytes.
    Bytes {
        bytes: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

// ============================================================================
// Artifact — generated output from a task
// ============================================================================

/// An output artifact produced by a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Unique within the task.
    pub artifact_id: String,
    /// Content parts.
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// Streaming events (SSE)
// ============================================================================

/// Events sent over SSE for streaming responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StreamEvent {
    /// Task status changed.
    #[serde(rename = "status-update")]
    StatusUpdate {
        task_id: String,
        context_id: String,
        status: TaskStatus,
        /// True when this interaction cycle is complete.
        #[serde(rename = "final")]
        is_final: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    /// New or updated artifact chunk.
    #[serde(rename = "artifact-update")]
    ArtifactUpdate {
        task_id: String,
        context_id: String,
        artifact: Artifact,
        /// Append to an existing artifact with the same ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        append: Option<bool>,
        /// This is the final chunk for this artifact.
        #[serde(skip_serializing_if = "Option::is_none")]
        last_chunk: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
}

// ============================================================================
// Push notifications
// ============================================================================

/// Configuration for webhook-based push notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// HTTPS webhook URL to receive notifications.
    pub url: String,
    /// Opaque token for client-side validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<PushNotificationAuth>,
}

/// Authentication for push notification webhooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationAuth {
    pub schemes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,
}

// ============================================================================
// JSON-RPC request/response params
// ============================================================================

/// Params for `message/send` and `message/stream`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<MessageSendConfiguration>,
}

/// Configuration for a send-message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSendConfiguration {
    /// Content types the client accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_output_modes: Option<Vec<String>>,
    /// If true, client will wait for task completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    /// How many history messages to return (None=unlimited, 0=none).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<i32>,
    /// Inline push notification config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notification_config: Option<PushNotificationConfig>,
}

/// Params for `tasks/get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTaskRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<i32>,
}

/// Params for `tasks/cancel`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelTaskRequest {
    pub id: String,
}

// ============================================================================
// JSON-RPC envelope (shared with v4-mcp pattern)
// ============================================================================

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request.
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String(uuid::Uuid::new_v4().to_string())),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: Option<JsonRpcId>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC ID (number or string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// Numeric ID.
    Number(i64),
    /// String ID.
    String(String),
}

/// JSON-RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Parse error (-32700).
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self { code: -32700, message: message.into(), data: None }
    }

    /// Invalid request (-32600).
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self { code: -32600, message: message.into(), data: None }
    }

    /// Method not found (-32601).
    pub fn method_not_found(method: &str) -> Self {
        Self { code: -32601, message: format!("Method not found: {method}"), data: None }
    }

    /// Invalid params (-32602).
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self { code: -32602, message: message.into(), data: None }
    }

    /// Internal error (-32603).
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self { code: -32603, message: message.into(), data: None }
    }

    /// Task not found (-32001).
    pub fn task_not_found(task_id: &str) -> Self {
        Self { code: -32001, message: format!("Task not found: {task_id}"), data: None }
    }

    /// Task not cancelable (-32002).
    pub fn task_not_cancelable(task_id: &str) -> Self {
        Self { code: -32002, message: format!("Task not cancelable: {task_id}"), data: None }
    }

    /// Push notifications not supported (-32003).
    pub fn push_not_supported() -> Self {
        Self { code: -32003, message: "Push notifications not supported".to_string(), data: None }
    }

    /// Unsupported operation (-32004).
    pub fn unsupported_operation(op: &str) -> Self {
        Self { code: -32004, message: format!("Unsupported operation: {op}"), data: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_terminal() {
        assert!(TaskState::Completed.is_terminal());
        assert!(TaskState::Canceled.is_terminal());
        assert!(TaskState::Failed.is_terminal());
        assert!(!TaskState::Working.is_terminal());
        assert!(!TaskState::Submitted.is_terminal());
        assert!(!TaskState::InputRequired.is_terminal());
    }

    #[test]
    fn test_task_state_serialization() {
        let state = TaskState::InputRequired;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"input-required\"");

        let parsed: TaskState = serde_json::from_str("\"working\"").unwrap();
        assert_eq!(parsed, TaskState::Working);
    }

    #[test]
    fn test_message_text_helper() {
        let msg = Message::text(Role::User, "Hello agent");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.parts.len(), 1);
        if let Part::Text { ref text, .. } = msg.parts[0] {
            assert_eq!(text, "Hello agent");
        } else {
            panic!("Expected text part");
        }
    }

    #[test]
    fn test_part_discriminated_union() {
        let text = Part::Text { text: "hello".to_string(), metadata: None };
        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["kind"], "text");
        assert_eq!(json["text"], "hello");

        let data = Part::Data {
            data: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), serde_json::json!("value"));
                m
            },
            metadata: None,
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["kind"], "data");
    }

    #[test]
    fn test_agent_card_serialization() {
        let card = AgentCard {
            name: "test-agent".to_string(),
            url: "https://example.com/a2a".to_string(),
            version: "1.0.0".to_string(),
            skills: vec![AgentSkill {
                id: "draft".to_string(),
                name: "Draft Content".to_string(),
                description: "Generate draft content".to_string(),
                tags: Some(vec!["content".to_string()]),
                examples: None,
                input_modes: None,
                output_modes: None,
            }],
            description: Some("A test agent".to_string()),
            documentation_url: None,
            provider: None,
            capabilities: Some(AgentCapabilities {
                a2a_version: Some("0.3".to_string()),
                streaming: Some(true),
                push_notifications: Some(false),
                state_transition_history: None,
            }),
            security_schemes: None,
            security: None,
            default_input_modes: default_modes(),
            default_output_modes: default_modes(),
            supports_authenticated_extended_card: None,
        };

        let json = serde_json::to_value(&card).unwrap();
        assert_eq!(json["name"], "test-agent");
        assert!(json["skills"].is_array());
        assert_eq!(json["capabilities"]["streaming"], true);
    }

    #[test]
    fn test_stream_event_serialization() {
        let event = StreamEvent::StatusUpdate {
            task_id: "t-1".to_string(),
            context_id: "ctx-1".to_string(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            is_final: false,
            metadata: None,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "status-update");
        assert_eq!(json["status"]["state"], "working");
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::task_not_found("x").code, -32001);
        assert_eq!(JsonRpcError::task_not_cancelable("x").code, -32002);
        assert_eq!(JsonRpcError::push_not_supported().code, -32003);
        assert_eq!(JsonRpcError::unsupported_operation("x").code, -32004);
    }
}
