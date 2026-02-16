//! A2A Client
//!
//! HTTP client for discovering and communicating with remote A2A agents.
//! This is what the orchestrator uses to delegate work to worker agents
//! (MiniMax, DeepSeek, local models wrapped in A2A, etc.).

use crate::error::A2AError;
use crate::types::{
    AgentCard, CancelTaskRequest, GetTaskRequest, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
    Message, Role, SendMessageRequest, Task,
};

/// Client for communicating with a remote A2A agent.
pub struct A2AClient {
    /// The remote agent's card (fetched during discovery).
    card: AgentCard,
    /// HTTP client.
    http: reqwest::Client,
}

impl A2AClient {
    /// Discover an agent by fetching its card from the well-known URL.
    ///
    /// # Arguments
    /// * `base_url` - The agent's base URL (e.g., `https://agent.example.com`)
    pub async fn discover(base_url: &str) -> Result<Self, A2AError> {
        let http = reqwest::Client::new();
        let card_url = format!(
            "{}/.well-known/agent-card.json",
            base_url.trim_end_matches('/')
        );

        tracing::info!(url = %card_url, "Discovering A2A agent");

        let response = http
            .get(&card_url)
            .send()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(A2AError::Http(format!(
                "Agent card fetch failed: {}",
                response.status()
            )));
        }

        let mut card: AgentCard = response
            .json()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        // If the card doesn't include a URL, use the discovery base URL
        if card.url.is_empty() {
            card.url = base_url.trim_end_matches('/').to_string();
        }

        tracing::info!(
            name = %card.name,
            skills = card.skills.len(),
            "Discovered A2A agent"
        );

        Ok(Self { card, http })
    }

    /// Create a client from an already-known agent card (skip discovery).
    pub fn from_card(card: AgentCard) -> Self {
        Self {
            card,
            http: reqwest::Client::new(),
        }
    }

    /// Get the remote agent's card.
    pub fn card(&self) -> &AgentCard {
        &self.card
    }

    /// Check if the remote agent supports a specific skill.
    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.card.skills.iter().any(|s| s.id == skill_id)
    }

    /// Check if the remote agent supports streaming.
    pub fn supports_streaming(&self) -> bool {
        self.card
            .capabilities
            .as_ref()
            .and_then(|c| c.streaming)
            .unwrap_or(false)
    }

    /// Send a text message to the remote agent.
    pub async fn send_message(&self, text: impl Into<String>) -> Result<Task, A2AError> {
        let request = SendMessageRequest {
            message: Message::text(Role::User, text),
            configuration: None,
        };
        self.send(request).await
    }

    /// Send a structured message request to the remote agent.
    pub async fn send(&self, request: SendMessageRequest) -> Result<Task, A2AError> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String(uuid::Uuid::new_v4().to_string())),
            method: "message/send".to_string(),
            params: Some(
                serde_json::to_value(&request).map_err(A2AError::from)?,
            ),
        };

        self.rpc_call(rpc_request).await
    }

    /// Get the current state of a task.
    pub async fn get_task(&self, task_id: impl Into<String>) -> Result<Task, A2AError> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String(uuid::Uuid::new_v4().to_string())),
            method: "tasks/get".to_string(),
            params: Some(serde_json::to_value(GetTaskRequest {
                id: task_id.into(),
                history_length: None,
            }).map_err(A2AError::from)?),
        };

        self.rpc_call(rpc_request).await
    }

    /// Cancel a running task.
    pub async fn cancel_task(&self, task_id: impl Into<String>) -> Result<Task, A2AError> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String(uuid::Uuid::new_v4().to_string())),
            method: "tasks/cancel".to_string(),
            params: Some(serde_json::to_value(CancelTaskRequest {
                id: task_id.into(),
            }).map_err(A2AError::from)?),
        };

        self.rpc_call(rpc_request).await
    }

    /// Make a raw JSON-RPC call to the remote agent.
    async fn rpc_call(&self, request: JsonRpcRequest) -> Result<Task, A2AError> {
        let response = self
            .http
            .post(&self.card.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(A2AError::Http(format!(
                "A2A request failed: {}",
                response.status()
            )));
        }

        let rpc_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        if let Some(error) = rpc_response.error {
            return Err(A2AError::HandlerError(format!(
                "[{}] {}",
                error.code, error.message
            )));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| A2AError::Internal("Empty result from agent".to_string()))?;

        serde_json::from_value(result).map_err(|e| A2AError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_client_from_card() {
        let card = AgentCard {
            name: "remote-worker".to_string(),
            url: "http://localhost:4000".to_string(),
            version: "1.0.0".to_string(),
            skills: vec![
                AgentSkill {
                    id: "draft-content".to_string(),
                    name: "Draft Content".to_string(),
                    description: "Generate draft text".to_string(),
                    tags: Some(vec!["content".to_string()]),
                    examples: None,
                    input_modes: None,
                    output_modes: None,
                },
                AgentSkill {
                    id: "generate-code".to_string(),
                    name: "Generate Code".to_string(),
                    description: "Write boilerplate code".to_string(),
                    tags: Some(vec!["code".to_string()]),
                    examples: None,
                    input_modes: None,
                    output_modes: None,
                },
            ],
            description: None,
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
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        };

        let client = A2AClient::from_card(card);
        assert_eq!(client.card().name, "remote-worker");
        assert!(client.has_skill("draft-content"));
        assert!(client.has_skill("generate-code"));
        assert!(!client.has_skill("nonexistent"));
        assert!(client.supports_streaming());
    }

    #[test]
    fn test_client_no_streaming() {
        let card = AgentCard {
            name: "basic".to_string(),
            url: "http://localhost:4000".to_string(),
            version: "1.0.0".to_string(),
            skills: vec![],
            description: None,
            documentation_url: None,
            provider: None,
            capabilities: None,
            security_schemes: None,
            security: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        };

        let client = A2AClient::from_card(card);
        assert!(!client.supports_streaming());
    }
}
