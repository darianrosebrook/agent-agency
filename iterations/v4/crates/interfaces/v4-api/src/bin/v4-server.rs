//! V4 API Server Binary
//!
//! Run with: `cargo run -p v4-api --bin v4-server`
//!
//! Environment variables:
//! - `V4_HOST`: Host to bind (default: 127.0.0.1)
//! - `V4_PORT`: Port to listen (default: 8080)
//! - `V4_LOG_LEVEL`: Log level (default: info)
//! - `RUST_LOG`: Alternative log level setting

use std::env;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use v4_api::{Server, ServerConfig, ServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let log_level = env::var("V4_LOG_LEVEL")
        .or_else(|_| env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".to_string());

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&log_level)))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build configuration from environment
    let host = env::var("V4_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("V4_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let config = ServerConfig {
        host,
        port,
        request_timeout: std::time::Duration::from_secs(30),
        enable_cors: true,
        service_config: ServiceConfig::default(),
    };

    tracing::info!(
        host = %config.host,
        port = config.port,
        "V4 API Server starting"
    );

    // Print available endpoints
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           Agent Agency V4 API Server                      ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║                                                           ║");
    println!("║  Endpoints:                                               ║");
    println!("║    GET  /health              - Health check               ║");
    println!("║    GET  /metrics             - Performance metrics        ║");
    println!("║    GET  /api/v1              - API info                   ║");
    println!("║    POST /api/v1/tasks        - Submit task                ║");
    println!("║    GET  /api/v1/tasks/:id    - Get task status            ║");
    println!("║    POST /api/v1/probe        - Probe LLM (mock)           ║");
    println!("║                                                           ║");
    println!(
        "║  Server: http://{}:{}                            ║",
        config.host, config.port
    );
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Run server
    let server = Server::new(config);
    server.run().await?;

    Ok(())
}
