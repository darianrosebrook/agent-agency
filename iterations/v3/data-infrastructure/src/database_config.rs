//! Database configuration types

use serde::{Deserialize, Serialize};

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    /// Hostname or IP address
    pub host: Option<String>,
    /// Port number
    pub port: Option<u16>,
    /// Database name
    pub database: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Maximum pool size (alias for max_connections)
    pub pool_max: Option<u32>,
    /// Connection timeout in seconds
    pub connection_timeout: Option<u64>,
    /// Connection timeout in seconds (alias)
    pub connection_timeout_seconds: Option<u64>,
    /// Query timeout in seconds
    pub query_timeout: Option<u64>,
    /// Enable SSL
    pub ssl_mode: Option<bool>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/test".to_string(),
            host: Some("localhost".to_string()),
            port: Some(5432),
            database: Some("test".to_string()),
            username: Some("postgres".to_string()),
            password: Some("password".to_string()),
            max_connections: Some(100),
            pool_max: Some(100),
            connection_timeout: Some(30),
            connection_timeout_seconds: Some(30),
            query_timeout: Some(60),
            ssl_mode: Some(false),
        }
    }
}

impl DatabaseConfig {
    /// Validate the database configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        if let Some(max_conn) = self.max_connections {
            if max_conn == 0 {
                return Err("Max connections must be greater than 0".to_string());
            }
        }

        if let Some(pool_max) = self.pool_max {
            if pool_max == 0 {
                return Err("Pool max must be greater than 0".to_string());
            }
        }

        if let Some(conn_timeout) = self.connection_timeout {
            if conn_timeout == 0 {
                return Err("Connection timeout must be greater than 0".to_string());
            }
        }

        if let Some(query_timeout) = self.query_timeout {
            if query_timeout == 0 {
                return Err("Query timeout must be greater than 0".to_string());
            }
        }

        Ok(())
    }
}
