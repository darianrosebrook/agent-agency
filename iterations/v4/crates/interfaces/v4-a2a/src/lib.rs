//! v4-a2a — Agent-to-Agent Protocol Server and Client
//!
//! This crate implements the [A2A protocol](https://a2a-protocol.org/) for
//! Agent Agency V4, enabling agents to discover each other, delegate work,
//! and collaborate as peers.
//!
//! ## Overview
//!
//! A2A complements MCP: where MCP connects agents to **tools**, A2A connects
//! agents to **other agents**. This enables multi-model orchestration where
//! an orchestrator (Claude/Agent Agency) can delegate work to cheaper worker
//! agents (MiniMax, local distilled models, etc.) via a standard protocol.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │              Agent Agency (Orchestrator)             │
//! │                                                     │
//! │  ┌───────────┐    ┌──────────┐    ┌──────────────┐  │
//! │  │ Sterling   │    │ Council  │    │ v4-a2a       │  │
//! │  │ Reasoning  │───▶│ Review   │───▶│ Client       │  │
//! │  └───────────┘    └──────────┘    └──────┬───────┘  │
//! └──────────────────────────────────────────┼──────────┘
//!                                            │ A2A Protocol
//!                    ┌───────────────────────┼───────────────┐
//!                    │                       │               │
//!              ┌─────▼──────┐        ┌──────▼─────┐  ┌─────▼──────┐
//!              │ MiniMax    │        │ Local 9B   │  │ DeepSeek   │
//!              │ v4-a2a     │        │ v4-a2a     │  │ v4-a2a     │
//!              │ Server     │        │ Server     │  │ Server     │
//!              └────────────┘        └────────────┘  └────────────┘
//! ```
//!
//! ## Usage — Exposing an Agent (Server)
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use v4_a2a::{A2AServer, A2AServerConfig, AgentCard, AgentSkill, EchoAgent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let card = AgentCard {
//!         name: "my-worker".to_string(),
//!         url: "http://localhost:3001".to_string(),
//!         version: "0.1.0".to_string(),
//!         skills: vec![AgentSkill {
//!             id: "draft".to_string(),
//!             name: "Draft Content".to_string(),
//!             description: "Generate draft text content".to_string(),
//!             tags: None, examples: None, input_modes: None, output_modes: None,
//!         }],
//!         description: None, documentation_url: None, provider: None,
//!         capabilities: None, security_schemes: None, security: None,
//!         default_input_modes: vec!["text".to_string()],
//!         default_output_modes: vec!["text".to_string()],
//!         supports_authenticated_extended_card: None,
//!     };
//!
//!     let agent = Arc::new(EchoAgent::new(card));
//!     let server = A2AServer::new(agent, A2AServerConfig::default());
//!     server.serve().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Usage — Calling a Remote Agent (Client)
//!
//! ```rust,no_run
//! use v4_a2a::A2AClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Discover agent from well-known URL
//!     let client = A2AClient::discover("http://localhost:3001").await?;
//!
//!     println!("Found agent: {}", client.card().name);
//!     println!("Skills: {:?}", client.card().skills.iter().map(|s| &s.id).collect::<Vec<_>>());
//!
//!     // Send work
//!     let task = client.send_message("Generate a blog post about Rust async").await?;
//!     println!("Task {} completed: {:?}", task.id, task.status.state);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Implementing a Custom Agent
//!
//! Implement the [`AgentHandler`] trait to create your own A2A-compatible agent:
//!
//! ```rust,no_run
//! use async_trait::async_trait;
//! use v4_a2a::{AgentHandler, AgentCard, A2AError, SendMessageRequest, GetTaskRequest, CancelTaskRequest, Task};
//!
//! struct MyAgent { /* ... */ }
//!
//! #[async_trait]
//! impl AgentHandler for MyAgent {
//!     fn agent_card(&self) -> &AgentCard { todo!() }
//!     async fn handle_message(&self, req: SendMessageRequest) -> Result<Task, A2AError> { todo!() }
//!     async fn get_task(&self, req: GetTaskRequest) -> Result<Task, A2AError> { todo!() }
//!     async fn cancel_task(&self, req: CancelTaskRequest) -> Result<Task, A2AError> { todo!() }
//! }
//! ```

pub mod agents;
pub mod client;
pub mod cost;
pub mod delegate;
pub mod error;
pub mod handler;
pub mod server;
pub mod types;

// Re-exports — server side
pub use error::A2AError;
pub use handler::{A2AHandler, AgentHandler, EchoAgent};
pub use server::{A2AServer, A2AServerConfig};

// Re-exports — client side
pub use client::A2AClient;

// Re-exports — cost tracking
pub use cost::{
    BalanceChecker, BalanceInfo, BudgetConfig, BudgetEnforcer, BudgetError, CostError, CostMonitor,
    CostSummary, MiniMaxBalance, OllamaBalance, OpenRouterBalance, ProviderCostInfo, ProviderRates,
};

// Re-exports — delegation
pub use delegate::DelegateTool;

// Re-exports — protocol types
pub use types::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Artifact, CancelTaskRequest,
    FileContent, GetTaskRequest, JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
    Message, MessageSendConfiguration, Part, PushNotificationAuth, PushNotificationConfig, Role,
    SecurityScheme, SendMessageRequest, StreamEvent, Task, TaskState, TaskStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_card() -> AgentCard {
        AgentCard {
            name: "integration-test".to_string(),
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
        }
    }

    #[test]
    fn test_crate_exports() {
        // Verify all key types are accessible from the crate root
        let card = test_card();
        let _agent = EchoAgent::new(card.clone());
        let _config = A2AServerConfig::default();
        let _client = A2AClient::from_card(card);
    }

    #[tokio::test]
    async fn test_end_to_end_echo() {
        let agent = Arc::new(EchoAgent::new(test_card()));
        let handler = A2AHandler::new(agent);

        // Send a message
        let request = JsonRpcRequest::new(
            "message/send",
            Some(
                serde_json::to_value(SendMessageRequest {
                    message: Message::text(Role::User, "Integration test"),
                    configuration: None,
                })
                .unwrap(),
            ),
        );

        let response = handler.handle(request).await;
        assert!(response.error.is_none());

        let task: Task = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(task.status.state, TaskState::Completed);
        assert!(task.artifacts.is_some());

        // Retrieve the task
        let get_request = JsonRpcRequest::new(
            "tasks/get",
            Some(
                serde_json::to_value(GetTaskRequest {
                    id: task.id.clone(),
                    history_length: None,
                })
                .unwrap(),
            ),
        );

        let get_response = handler.handle(get_request).await;
        assert!(get_response.error.is_none());

        let retrieved: Task = serde_json::from_value(get_response.result.unwrap()).unwrap();
        assert_eq!(retrieved.id, task.id);
    }
}
