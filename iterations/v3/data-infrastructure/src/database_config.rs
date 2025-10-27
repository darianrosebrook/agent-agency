//! Database configuration types

use serde::{Deserialize, Serialize};

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Connection timeout in seconds
    pub connection_timeout: Option<u64>,
    /// Query timeout in seconds
    pub query_timeout: Option<u64>,
    /// Enable SSL
    pub ssl_mode: Option<bool>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/test".to_string(),
            max_connections: Some(100),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ssl_mode: Some(false),
        }
    }
}
