//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.

use std::sync::Arc;
use clap::Parser;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::env;

use agent_agency_v3::{
    interfaces::api::{RestApi, ApiConfig},
    orchestration::{orchestrate::Orchestrator, tracking::ProgressTracker},
};

#[derive(Parser)]
#[command(name = "agent-agency-api")]
#[command(about = "Agent Agency V3 REST API Server")]
struct Args {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Enable CORS
    #[arg(long)]
    enable_cors: bool,

    /// Require API key authentication
    #[arg(long)]
    require_api_key: bool,

    /// Config file path
    #[arg(long, default_value = "api-server-config.toml")]
    config_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    /// API keys for authentication
    api_keys: Option<Vec<String>>,
    /// Rate limiting enabled
    enable_rate_limiting: Option<bool>,
    /// Rate limit requests per minute
    rate_limit_per_minute: Option<u32>,
}

/// Load server configuration from file and environment variables
async fn load_server_config(config_file: &str) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut config = ServerConfig {
        api_keys: None,
        enable_rate_limiting: None,
        rate_limit_per_minute: None,
    };

    // Try to load from config file first
    if let Ok(config_content) = tokio::fs::read_to_string(config_file).await {
        if let Ok(file_config) = toml::from_str::<ServerConfig>(&config_content) {
            config = file_config;
        } else {
            println!("⚠️  Warning: Could not parse config file '{}', using defaults", config_file);
        }
    } else {
        println!("📄 No config file found at '{}', using environment variables", config_file);
    }

    // Override with environment variables if set
    if let Ok(env_keys) = env::var("AGENT_AGENCY_API_KEYS") {
        let keys: Vec<String> = env_keys.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !keys.is_empty() {
            config.api_keys = Some(keys);
        }
    }

    if let Ok(env_rate_limiting) = env::var("AGENT_AGENCY_ENABLE_RATE_LIMITING") {
        config.enable_rate_limiting = Some(env_rate_limiting.to_lowercase() == "true");
    }

    if let Ok(env_rate_limit) = env::var("AGENT_AGENCY_RATE_LIMIT_PER_MINUTE") {
        if let Ok(limit) = env_rate_limit.parse::<u32>() {
            config.rate_limit_per_minute = Some(limit);
        }
    }

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Starting Agent Agency V3 API Server");
    println!("📡 Server: {}:{}", args.host, args.port);

    // Initialize core components (simplified for MVP)
    let orchestrator = Arc::new(Orchestrator::new(
        // TODO: Initialize with proper configuration
        Default::default(),
        Arc::new(ProgressTracker::new(Default::default(), None)),
    ));

    let progress_tracker = Arc::new(ProgressTracker::new(Default::default(), None));

    // Configure API
    let api_config = ApiConfig {
        host: args.host.clone(),
        port: args.port,
        enable_cors: args.enable_cors,
        require_api_key: args.require_api_key,
        api_keys: vec![], // TODO: Load from config
        enable_rate_limiting: false,
        rate_limit_per_minute: 100,
    };

    // Create REST API instance
    let rest_api = RestApi::new(api_config, orchestrator, progress_tracker);

    // Create router
    let app = rest_api.create_router();

    // Add CORS if enabled
    let app = if args.enable_cors {
        app.layer(tower_http::cors::CorsLayer::permissive())
    } else {
        app
    };

    // Bind server
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("✅ API server ready at http://{}", addr);
    println!("📊 Health check: http://{}/health", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}

