//! V4 MCP Server Binary
//!
//! Run with: `cargo run -p v4-mcp --bin v4-mcp-server`
//!
//! Environment variables:
//! - `V4_MCP_HOST`: Host to bind (default: 127.0.0.1)
//! - `V4_MCP_PORT`: Port to listen (default: 3000)
//! - `V4_LOG_LEVEL`: Log level (default: info)
//! - `RUST_LOG`: Alternative log level setting

use std::env;
use std::sync::Arc;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use v4_mcp::{MCPServer, MCPServerConfig, ToolAdapter};
use v4_tools::ToolRegistry;

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
    let host = env::var("V4_MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("V4_MCP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Create tool registry with built-in tools
    let registry = Arc::new(ToolRegistry::new());
    // Register built-in tools
    v4_tools::register_builtin_tools(&registry);

    // Create adapter and server
    let adapter = Arc::new(ToolAdapter::new(registry));
    let config = MCPServerConfig::default()
        .with_host(&host)
        .with_port(port)
        .with_server_info("Agent Agency V4 MCP", env!("CARGO_PKG_VERSION"));

    tracing::info!(
        host = %host,
        port = port,
        "V4 MCP Server starting"
    );

    // Print available endpoints
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           Agent Agency V4 MCP Server                      ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║                                                           ║");
    println!("║  JSON-RPC Endpoints:                                      ║");
    println!("║    POST /       - JSON-RPC endpoint                       ║");
    println!("║    POST /mcp    - JSON-RPC endpoint (alias)               ║");
    println!("║    GET  /health - Health check                            ║");
    println!("║    GET  /info   - Server capabilities                     ║");
    println!("║                                                           ║");
    println!("║  MCP Methods:                                             ║");
    println!("║    initialize     - Initialize connection                 ║");
    println!("║    initialized    - Confirm initialization                ║");
    println!("║    tools/list     - List available tools                  ║");
    println!("║    tools/call     - Execute a tool                        ║");
    println!("║    ping           - Health check                          ║");
    println!("║                                                           ║");
    println!("║  Server: http://{}:{}                            ║", host, port);
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Run server
    let server = MCPServer::new(adapter, config);
    server.serve().await?;

    Ok(())
}
