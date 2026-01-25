//! PostgreSQL Configuration
//!
//! Connection settings and pool configuration.

use std::time::Duration;

/// PostgreSQL connection configuration
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Connection pool minimum size
    pub min_connections: u32,
    /// Connection pool maximum size
    pub max_connections: u32,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Idle timeout for connections
    pub idle_timeout: Duration,
    /// Whether to use SSL
    pub ssl_mode: SslMode,
    /// Application name for connection tracking
    pub application_name: String,
}

/// SSL connection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslMode {
    /// Disable SSL
    Disable,
    /// Prefer SSL but allow non-SSL
    Prefer,
    /// Require SSL
    Require,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "agent_agency_v4".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            min_connections: 1,
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            ssl_mode: SslMode::Prefer,
            application_name: "agent-agency-v4".to_string(),
        }
    }
}

impl PostgresConfig {
    /// Create config for local development
    pub fn local_dev() -> Self {
        Self {
            database: "agent_agency_v4_dev".to_string(),
            ssl_mode: SslMode::Disable,
            ..Default::default()
        }
    }

    /// Create config for testing
    pub fn test() -> Self {
        Self {
            database: "agent_agency_v4_test".to_string(),
            ssl_mode: SslMode::Disable,
            max_connections: 2,
            ..Default::default()
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("POSTGRES_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432);
        let database =
            std::env::var("POSTGRES_DB").unwrap_or_else(|_| "agent_agency_v4".to_string());
        let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_default();

        let ssl_mode = match std::env::var("POSTGRES_SSL_MODE")
            .unwrap_or_else(|_| "prefer".to_string())
            .as_str()
        {
            "disable" => SslMode::Disable,
            "require" => SslMode::Require,
            _ => SslMode::Prefer,
        };

        let max_connections = std::env::var("POSTGRES_MAX_CONNECTIONS")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(10);

        Ok(Self {
            host,
            port,
            database,
            username,
            password,
            ssl_mode,
            max_connections,
            ..Default::default()
        })
    }

    /// Build connection URL
    pub fn connection_url(&self) -> String {
        let ssl_param = match self.ssl_mode {
            SslMode::Disable => "sslmode=disable",
            SslMode::Prefer => "sslmode=prefer",
            SslMode::Require => "sslmode=require",
        };

        format!(
            "postgres://{}:{}@{}:{}/{}?{}&application_name={}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database,
            ssl_param,
            self.application_name
        )
    }

    /// Builder pattern - set host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Builder pattern - set port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Builder pattern - set database
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    /// Builder pattern - set credentials
    pub fn with_credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = username.into();
        self.password = password.into();
        self
    }

    /// Builder pattern - set max connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PostgresConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_local_dev_config() {
        let config = PostgresConfig::local_dev();
        assert_eq!(config.database, "agent_agency_v4_dev");
        assert_eq!(config.ssl_mode, SslMode::Disable);
    }

    #[test]
    fn test_connection_url() {
        let config = PostgresConfig::default()
            .with_credentials("testuser", "testpass")
            .with_database("testdb");

        let url = config.connection_url();
        assert!(url.contains("testuser"));
        assert!(url.contains("testdb"));
        assert!(url.contains("5432"));
    }

    #[test]
    fn test_builder_pattern() {
        let config = PostgresConfig::default()
            .with_host("db.example.com")
            .with_port(5433)
            .with_database("mydb")
            .with_max_connections(20);

        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.port, 5433);
        assert_eq!(config.database, "mydb");
        assert_eq!(config.max_connections, 20);
    }
}
