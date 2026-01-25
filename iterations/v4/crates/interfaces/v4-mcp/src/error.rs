//! MCP Error Types

use thiserror::Error;

/// MCP server errors
#[derive(Debug, Error)]
pub enum MCPError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Unsupported tool: {0}")]
    UnsupportedTool(String),

    #[error("Governance violation: {0}")]
    GovernanceViolation(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl MCPError {
    /// Convert to JSON-RPC error code
    pub fn error_code(&self) -> i32 {
        match self {
            MCPError::ToolNotFound(_) => -32601, // Method not found
            MCPError::MissingParameter(_) => -32602, // Invalid params
            MCPError::InvalidParameter(_) => -32602, // Invalid params
            MCPError::ExecutionFailed(_) => -32000, // Server error
            MCPError::UnsupportedTool(_) => -32601, // Method not found
            MCPError::GovernanceViolation(_) => -32001, // Custom: governance
            MCPError::ServerError(_) => -32603, // Internal error
            MCPError::ProtocolError(_) => -32600, // Invalid request
            MCPError::Timeout(_) => -32002, // Custom: timeout
            MCPError::Serialization(_) => -32700, // Parse error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MCPError::ToolNotFound("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(MCPError::ToolNotFound("x".to_string()).error_code(), -32601);
        assert_eq!(
            MCPError::MissingParameter("x".to_string()).error_code(),
            -32602
        );
        assert_eq!(
            MCPError::GovernanceViolation("x".to_string()).error_code(),
            -32001
        );
    }
}
