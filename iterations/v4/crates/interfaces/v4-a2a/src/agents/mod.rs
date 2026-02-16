//! Pre-built Agent Implementations
//!
//! Ready-to-use A2A agents backed by various LLM providers.
//!
//! ## Available Agents
//!
//! - [`OpenAICompatibleAgent`] — Works with any OpenAI-compatible API
//!   (MiniMax, DeepSeek, OpenRouter, Ollama, vLLM, etc.)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use v4_a2a::agents::{OpenAICompatibleAgent, minimax_config};
//! use v4_a2a::{A2AServer, A2AServerConfig, AgentSkill};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = Arc::new(OpenAICompatibleAgent::new(
//!         minimax_config("your-api-key"),
//!         vec![AgentSkill {
//!             id: "draft".to_string(),
//!             name: "Draft Content".to_string(),
//!             description: "Generate draft text content".to_string(),
//!             tags: None, examples: None, input_modes: None, output_modes: None,
//!         }],
//!     ));
//!
//!     let server = A2AServer::new(agent, A2AServerConfig::default());
//!     server.serve().await?;
//!     Ok(())
//! }
//! ```

pub mod local_mlx;
pub mod openai_compatible;

pub use local_mlx::{LocalMLXAgent, LocalMLXConfig, local_mlx_config_from_env};
pub use openai_compatible::{
    OpenAICompatibleAgent, ProviderConfig, UsageStats,
    deepseek_openrouter_config, minimax_config, minimax_highspeed_config, ollama_config,
};
