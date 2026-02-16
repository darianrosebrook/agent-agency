//! A2A Agent Handler
//!
//! Defines the trait that agent implementations must satisfy, and the
//! JSON-RPC dispatch layer that routes incoming requests to it.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::A2AError;
use crate::types::{
    AgentCard, Artifact, CancelTaskRequest, GetTaskRequest, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, Message, SendMessageRequest, StreamEvent, Task,
};

// ============================================================================
// Agent trait — implement this to make your agent A2A-compatible
// ============================================================================

/// Trait that any A2A-compatible agent must implement.
///
/// The server calls these methods in response to incoming JSON-RPC requests.
/// Implementors handle the actual work: delegating to local models, calling
/// external APIs, running tools, etc.
#[async_trait]
pub trait AgentHandler: Send + Sync {
    /// Return this agent's card (capabilities, skills, metadata).
    fn agent_card(&self) -> &AgentCard;

    /// Handle `message/send` — process a message and return the updated task.
    async fn handle_message(&self, request: SendMessageRequest) -> Result<Task, A2AError>;

    /// Handle `tasks/get` — retrieve current state of a task.
    async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError>;

    /// Handle `tasks/cancel` — cancel a running task.
    async fn cancel_task(&self, request: CancelTaskRequest) -> Result<Task, A2AError>;

    /// Handle `message/stream` — process a message and return a stream of events.
    ///
    /// Default implementation sends a single completed task (no streaming).
    /// Override for real streaming support.
    async fn handle_message_stream(
        &self,
        request: SendMessageRequest,
    ) -> Result<Vec<StreamEvent>, A2AError> {
        let task = self.handle_message(request).await?;
        let context_id = task.context_id.clone().unwrap_or_default();
        Ok(vec![StreamEvent::StatusUpdate {
            task_id: task.id.clone(),
            context_id,
            status: task.status,
            is_final: true,
            metadata: None,
        }])
    }
}

// ============================================================================
// JSON-RPC dispatcher
// ============================================================================

/// Dispatches JSON-RPC requests to the appropriate `AgentHandler` method.
pub struct A2AHandler {
    agent: Arc<dyn AgentHandler>,
}

impl A2AHandler {
    /// Create a new handler wrapping an agent implementation.
    pub fn new(agent: Arc<dyn AgentHandler>) -> Self {
        Self { agent }
    }

    /// Dispatch a JSON-RPC request and return a response.
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();
        let method = request.method.clone();
        let span = tracing::info_span!(
            "a2a_request",
            method = method.as_str(),
            request_id = ?id,
        );
        let _guard = span.enter();

        match self.dispatch(request).await {
            Ok(result) => {
                tracing::info!(method = method.as_str(), "A2A request succeeded");
                JsonRpcResponse::success(id, result)
            }
            Err(err) => {
                tracing::warn!(method = method.as_str(), error = %err, "A2A request failed");
                JsonRpcResponse::error(
                    id,
                    JsonRpcError {
                        code: err.error_code(),
                        message: err.to_string(),
                        data: None,
                    },
                )
            }
        }
    }

    async fn dispatch(
        &self,
        request: JsonRpcRequest,
    ) -> Result<serde_json::Value, A2AError> {
        match request.method.as_str() {
            "message/send" => {
                let params: SendMessageRequest = self.parse_params(request.params)?;
                let task = self.agent.handle_message(params).await?;
                serde_json::to_value(task).map_err(A2AError::from)
            }
            "tasks/get" => {
                let params: GetTaskRequest = self.parse_params(request.params)?;
                let task = self.agent.get_task(params).await?;
                serde_json::to_value(task).map_err(A2AError::from)
            }
            "tasks/cancel" => {
                let params: CancelTaskRequest = self.parse_params(request.params)?;
                let task = self.agent.cancel_task(params).await?;
                serde_json::to_value(task).map_err(A2AError::from)
            }
            // message/stream is handled separately via SSE in the server
            "message/stream" => Err(A2AError::UnsupportedOperation(
                "Use the streaming endpoint for message/stream".to_string(),
            )),
            method => Err(A2AError::UnsupportedOperation(method.to_string())),
        }
    }

    fn parse_params<T: serde::de::DeserializeOwned>(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<T, A2AError> {
        let value = params.ok_or_else(|| A2AError::MissingParameter("params".to_string()))?;
        serde_json::from_value(value).map_err(|e| A2AError::InvalidParameter(e.to_string()))
    }
}

// ============================================================================
// Simple in-memory agent for testing and as a starting template
// ============================================================================

/// A minimal agent that stores tasks in memory.
///
/// Useful for testing the A2A server or as a starting point for custom agents.
pub struct EchoAgent {
    card: AgentCard,
    tasks: std::sync::RwLock<std::collections::HashMap<String, Task>>,
}

impl EchoAgent {
    /// Create a new echo agent with the given card.
    pub fn new(card: AgentCard) -> Self {
        Self {
            card,
            tasks: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl AgentHandler for EchoAgent {
    fn agent_card(&self) -> &AgentCard {
        &self.card
    }

    async fn handle_message(&self, request: SendMessageRequest) -> Result<Task, A2AError> {
        let task_id = request
            .message
            .task_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let context_id = request.message.context_id.clone();

        // Echo back the message content as an artifact
        let response_text = match &request.message.parts.first() {
            Some(crate::types::Part::Text { text, .. }) => {
                format!("Echo: {text}")
            }
            _ => "Echo: (non-text content)".to_string(),
        };

        let task = Task {
            id: task_id.clone(),
            context_id,
            status: crate::types::TaskStatus {
                state: crate::types::TaskState::Completed,
                message: Some(Message::text(
                    crate::types::Role::Agent,
                    &response_text,
                )),
                timestamp: Some(chrono::Utc::now()),
            },
            kind: "task".to_string(),
            history: Some(vec![request.message]),
            artifacts: Some(vec![Artifact {
                artifact_id: "echo-1".to_string(),
                parts: vec![crate::types::Part::Text {
                    text: response_text,
                    metadata: None,
                }],
                name: None,
                description: None,
                metadata: None,
            }]),
            metadata: None,
        };

        self.tasks.write().unwrap().insert(task_id, task.clone());
        Ok(task)
    }

    async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError> {
        self.tasks
            .read()
            .unwrap()
            .get(&request.id)
            .cloned()
            .ok_or_else(|| A2AError::TaskNotFound(request.id))
    }

    async fn cancel_task(&self, request: CancelTaskRequest) -> Result<Task, A2AError> {
        let mut tasks = self.tasks.write().unwrap();
        let task = tasks
            .get_mut(&request.id)
            .ok_or_else(|| A2AError::TaskNotFound(request.id.clone()))?;

        if task.status.state.is_terminal() {
            return Err(A2AError::TaskNotCancelable(request.id));
        }

        task.status = crate::types::TaskStatus {
            state: crate::types::TaskState::Canceled,
            message: None,
            timestamp: Some(chrono::Utc::now()),
        };

        Ok(task.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_card() -> AgentCard {
        AgentCard {
            name: "test".to_string(),
            url: "http://localhost:3001".to_string(),
            version: "0.1.0".to_string(),
            skills: vec![AgentSkill {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                description: "Echoes input".to_string(),
                tags: None,
                examples: None,
                input_modes: None,
                output_modes: None,
            }],
            description: None,
            documentation_url: None,
            provider: None,
            capabilities: None,
            security_schemes: None,
            security: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        }
    }

    #[tokio::test]
    async fn test_echo_agent_message() {
        let agent = EchoAgent::new(test_card());
        let request = SendMessageRequest {
            message: Message::text(Role::User, "Hello"),
            configuration: None,
        };

        let task = agent.handle_message(request).await.unwrap();
        assert_eq!(task.status.state, TaskState::Completed);
        assert!(task.artifacts.is_some());

        let artifacts = task.artifacts.unwrap();
        assert_eq!(artifacts.len(), 1);
        if let Part::Text { ref text, .. } = artifacts[0].parts[0] {
            assert!(text.contains("Echo: Hello"));
        }
    }

    #[tokio::test]
    async fn test_echo_agent_get_task() {
        let agent = EchoAgent::new(test_card());

        // Send a message to create a task
        let request = SendMessageRequest {
            message: Message::text(Role::User, "Test"),
            configuration: None,
        };
        let task = agent.handle_message(request).await.unwrap();

        // Retrieve it
        let retrieved = agent
            .get_task(GetTaskRequest {
                id: task.id.clone(),
                history_length: None,
            })
            .await
            .unwrap();
        assert_eq!(retrieved.id, task.id);
    }

    #[tokio::test]
    async fn test_echo_agent_task_not_found() {
        let agent = EchoAgent::new(test_card());
        let result = agent
            .get_task(GetTaskRequest {
                id: "nonexistent".to_string(),
                history_length: None,
            })
            .await;
        assert!(matches!(result, Err(A2AError::TaskNotFound(_))));
    }

    #[tokio::test]
    async fn test_handler_dispatch() {
        let agent = Arc::new(EchoAgent::new(test_card()));
        let handler = A2AHandler::new(agent);

        let request = JsonRpcRequest::new(
            "message/send",
            Some(serde_json::to_value(SendMessageRequest {
                message: Message::text(Role::User, "Hello"),
                configuration: None,
            }).unwrap()),
        );

        let response = handler.handle(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handler_unknown_method() {
        let agent = Arc::new(EchoAgent::new(test_card()));
        let handler = A2AHandler::new(agent);

        let request = JsonRpcRequest::new("unknown/method", None);
        let response = handler.handle(request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32004);
    }
}
