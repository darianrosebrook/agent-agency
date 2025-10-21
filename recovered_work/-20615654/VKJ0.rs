//! Interface Layers for Agent Agency V3
//!
//! Provides REST API, CLI, MCP server, and WebSocket interfaces for
//! tool-agnostic task intake and execution monitoring.

pub mod api;
pub mod cli;
pub mod mcp;
pub mod websocket;

pub use api::{RestApi, ApiConfig};
pub use cli::{CliInterface, CliConfig};
pub use mcp::{McpServer, McpConfig};
pub use websocket::{WebSocketApi, WebSocketApiConfig};

