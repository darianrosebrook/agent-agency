//! Interface Layers for Autonomous Task Execution
//!
//! Provides REST API, CLI, MCP server, and WebSocket interfaces for
//! tool-agnostic task intake and execution monitoring.

pub mod api;
pub mod cli_interface;
#[cfg(feature = "mcp")]
pub mod mcp;
// websocket module is declared in lib.rs, not here

pub use api::{RestApi, ApiConfig};
pub use cli_interface::{CliInterface, CliConfig};
#[cfg(feature = "mcp")]
pub use mcp::{McpServer, McpConfig};
// websocket exports are in lib.rs
pub struct WebSocketApiConfig;
