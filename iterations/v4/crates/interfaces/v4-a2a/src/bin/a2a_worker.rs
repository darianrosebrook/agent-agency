//! A2A Worker Server
//!
//! Runs an LLM as an A2A agent server.
//! Supports MiniMax, DeepSeek, OpenRouter, Ollama, local MLX, or any OpenAI-compatible provider.
//!
//! ## Usage
//!
//! ```sh
//! # MiniMax (default)
//! MINIMAX_API_KEY=your-key cargo run --bin a2a_worker
//!
//! # DeepSeek via OpenRouter
//! PROVIDER=openrouter OPENROUTER_API_KEY=your-key cargo run --bin a2a_worker
//!
//! # Local Ollama
//! PROVIDER=ollama OLLAMA_MODEL=llama3.2 cargo run --bin a2a_worker
//!
//! # Local MLX (Apple Silicon, zero cost)
//! PROVIDER=local MODEL_PATH=/path/to/model cargo run --bin a2a_worker
//!
//! # Custom port
//! PORT=4000 cargo run --bin a2a_worker
//! ```
//!
//! ## Discovery
//!
//! Once running, the agent card is available at:
//! ```
//! GET http://localhost:3001/.well-known/agent-card.json
//! ```

use std::sync::Arc;

use v4_a2a::agents::{
    deepseek_openrouter_config, local_mlx_config_from_env, minimax_config,
    minimax_highspeed_config, ollama_config, LocalMLXAgent, OpenAICompatibleAgent, ProviderConfig,
};
use v4_a2a::{A2AServer, A2AServerConfig, AgentHandler, AgentSkill, CostMonitor};

fn worker_skills() -> Vec<AgentSkill> {
    vec![
        AgentSkill {
            id: "draft-content".to_string(),
            name: "Draft Content".to_string(),
            description: "Generate draft text content: blog posts, documentation, \
                          descriptions, summaries, and other written material."
                .to_string(),
            tags: Some(vec!["content".to_string(), "writing".to_string()]),
            examples: Some(vec![
                "Write a blog post about Rust's ownership model".to_string(),
                "Summarize this technical document".to_string(),
                "Draft a product description for a CLI tool".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        },
        AgentSkill {
            id: "generate-code".to_string(),
            name: "Generate Code".to_string(),
            description: "Generate boilerplate code, template variations, test scaffolding, \
                          API routes, CRUD operations, and other structured code."
                .to_string(),
            tags: Some(vec!["code".to_string(), "generation".to_string()]),
            examples: Some(vec![
                "Generate a REST API handler for user registration".to_string(),
                "Write unit tests for this function".to_string(),
                "Create 5 color scheme variations for this CSS".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        },
        AgentSkill {
            id: "transform-data".to_string(),
            name: "Transform Data".to_string(),
            description: "Convert, reformat, or restructure data between formats. \
                          JSON, YAML, CSV, markdown tables, and other transformations."
                .to_string(),
            tags: Some(vec!["data".to_string(), "transform".to_string()]),
            examples: Some(vec![
                "Convert this JSON to a markdown table".to_string(),
                "Reformat this API response into a TypeScript interface".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        },
        AgentSkill {
            id: "review-text".to_string(),
            name: "Review & Edit Text".to_string(),
            description: "Review, proofread, edit, and improve written content. \
                          Fix grammar, improve clarity, adjust tone."
                .to_string(),
            tags: Some(vec!["review".to_string(), "editing".to_string()]),
            examples: Some(vec![
                "Proofread this README for clarity and grammar".to_string(),
                "Make this technical explanation more accessible".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        },
    ]
}

/// Returns true if the selected provider is "local" (MLX).
fn is_local_provider() -> bool {
    std::env::var("PROVIDER")
        .map(|v| v.to_lowercase() == "local")
        .unwrap_or(false)
}

fn build_config() -> ProviderConfig {
    let provider = std::env::var("PROVIDER").unwrap_or_else(|_| "minimax".to_string());

    match provider.to_lowercase().as_str() {
        "minimax" => {
            let key = std::env::var("MINIMAX_API_KEY")
                .expect("MINIMAX_API_KEY environment variable required");
            let highspeed = std::env::var("MINIMAX_HIGHSPEED")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
            if highspeed {
                minimax_highspeed_config(key)
            } else {
                minimax_config(key)
            }
        }
        "openrouter" | "deepseek" => {
            let key = std::env::var("OPENROUTER_API_KEY")
                .expect("OPENROUTER_API_KEY environment variable required");
            deepseek_openrouter_config(key)
        }
        "ollama" => {
            let model =
                std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
            ollama_config(model)
        }
        // "local" is handled separately in main() via LocalMLXAgent
        "local" => unreachable!("local provider uses LocalMLXAgent path"),
        other => {
            // Custom provider: requires BASE_URL, API_KEY, MODEL
            let base_url =
                std::env::var("BASE_URL").expect("BASE_URL required for custom provider");
            let api_key =
                std::env::var("API_KEY").expect("API_KEY required for custom provider");
            let model = std::env::var("MODEL").expect("MODEL required for custom provider");
            ProviderConfig {
                base_url,
                api_key,
                model,
                display_name: other.to_string(),
                max_tokens: std::env::var("MAX_TOKENS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(4096),
                temperature: std::env::var("TEMPERATURE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.7),
                system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
                cost_per_m_input: None,
                cost_per_m_output: None,
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3001);
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let agent: Arc<dyn AgentHandler> = if is_local_provider() {
        // Local MLX inference — zero cost, on-device
        let mlx_config = local_mlx_config_from_env();
        tracing::info!(
            provider = "local-mlx",
            model = %mlx_config.model_path,
            host = %host,
            port = port,
            "Starting A2A worker (local MLX)"
        );

        let agent = LocalMLXAgent::new(mlx_config, worker_skills());

        // Load the model before accepting requests
        agent.load_model().await?;
        tracing::info!("Local model loaded, ready to serve");

        Arc::new(agent)
    } else {
        // Remote API provider (MiniMax, DeepSeek, Ollama, etc.)
        let config = build_config();
        tracing::info!(
            provider = %config.display_name,
            model = %config.model,
            host = %host,
            port = port,
            "Starting A2A worker"
        );

        let cost_monitor = Arc::new(CostMonitor::new());
        let agent = OpenAICompatibleAgent::new(config, worker_skills())
            .with_cost_monitor(cost_monitor.clone());
        tracing::info!("Cost monitoring enabled");

        Arc::new(agent)
    };

    let server_config = A2AServerConfig::default()
        .with_host(&host)
        .with_port(port);

    let server = A2AServer::new(agent, server_config);

    tracing::info!(
        "Agent card available at http://{}:{}/.well-known/agent-card.json",
        host,
        port
    );
    tracing::info!(
        "A2A endpoint at http://{}:{}/",
        host,
        port
    );

    server.serve().await?;
    Ok(())
}
