//! Agent Agency V3 - Database Layer
//!
//! Provides database connectivity, connection pooling, and data access patterns
//! for the council-based arbiter system.

pub mod client;
pub mod migrations;
pub mod models;
pub mod queries;

pub use client::DatabaseClient;
pub use models::*;

/// Database configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_min: u32,
    pub pool_max: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "agent_agency_v3".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            pool_min: 2,
            pool_max: 20,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 3600,
        }
    }
}

impl DatabaseConfig {
    /// Create database URL for connection
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    /// Create database URL without database name (for creating database)
    pub fn server_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
