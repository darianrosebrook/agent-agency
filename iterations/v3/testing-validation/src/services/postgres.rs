//! PostgreSQL service management for E2E testing
//!
//! Provides real database integration with local PostgreSQL instance.
//! NO mocks - actual database connections, queries, and transactions.

use std::process::{Command, Stdio};
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn};
use std::time::Duration;

/// PostgreSQL service for real database operations
pub struct PostgresService {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    process_handle: Option<std::process::Child>,
    client: Option<Client>,
}

impl PostgresService {
    /// Create new PostgreSQL service instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            host: "localhost".to_string(),
            port: 5433,
            database: "test_db".to_string(),
            username: "test_user".to_string(),
            password: "test_password".to_string(),
            process_handle: None,
            client: None,
        })
    }

    /// Start local PostgreSQL service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting PostgreSQL service...");

        // Check if PostgreSQL is already running
        if self.health_check().await {
            info!("PostgreSQL service already running");
            return Ok(());
        }

        // For local testing, we'll assume PostgreSQL is already installed and running
        // In a real CI environment, you might need to start it via docker-compose
        warn!("PostgreSQL service not running. In CI, start with: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=test postgres:13");

        // For now, just check if we can connect to an existing instance
        for _ in 0..10 {
            if self.health_check().await {
                info!("Connected to PostgreSQL service");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        Err("PostgreSQL service not available".into())
    }

    /// Stop PostgreSQL service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping PostgreSQL service...");

        // Disconnect client
        self.client = None;

        if let Some(mut handle) = self.process_handle.take() {
            match handle.kill() {
                Ok(_) => info!("PostgreSQL service stopped"),
                Err(e) => warn!("Failed to kill PostgreSQL process: {}", e),
            }
        }

        Ok(())
    }

    /// Check if PostgreSQL service is healthy
    pub async fn is_healthy(&self) -> bool {
        self.health_check().await
    }

    /// Check if PostgreSQL service is healthy
    pub async fn health_check(&self) -> bool {
        match self.connect().await {
            Ok(_) => {
                info!("PostgreSQL health check passed");
                true
            }
            Err(_) => {
                warn!("PostgreSQL health check failed");
                false
            }
        }
    }

    /// Get database connection
    pub async fn connect(&self) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref client) = self.client {
            // Test the connection
            match client.query("SELECT 1", &[]).await {
                Ok(_) => return Ok(client.clone()),
                Err(_) => {
                    self.client = None;
                }
            }
        }

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.username, self.password
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

        // Spawn the connection to run in background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                warn!("PostgreSQL connection error: {}", e);
            }
        });

        // Test the connection
        client.query("SELECT 1", &[]).await?;

        let client_clone = client.clone();
        // Note: In real implementation, we'd store this properly, but for simplicity:
        // self.client = Some(client);

        Ok(client_clone)
    }

    /// Execute a query and return results
    pub async fn execute_query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.connect().await?;
        let rows = client.query(query, params).await?;
        Ok(rows)
    }

    /// Execute a query that doesn't return results
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.connect().await?;
        let result = client.execute(query, params).await?;
        Ok(result)
    }

    /// Create test tables and data
    pub async fn setup_test_schema(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up test schema...");

        let client = self.connect().await?;

        // Create test tables
        client.execute(
            "CREATE TABLE IF NOT EXISTS test_research (
                id SERIAL PRIMARY KEY,
                topic TEXT NOT NULL,
                content TEXT,
                citations JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        ).await?;

        client.execute(
            "CREATE TABLE IF NOT EXISTS test_code_changes (
                id SERIAL PRIMARY KEY,
                file_path TEXT NOT NULL,
                old_content TEXT,
                new_content TEXT,
                change_type TEXT,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        ).await?;

        info!("Test schema created successfully");
        Ok(())
    }

    /// Clean up test data
    pub async fn cleanup_test_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up test data...");

        let client = self.connect().await?;

        client.execute("DROP TABLE IF EXISTS test_research", &[]).await?;
        client.execute("DROP TABLE IF EXISTS test_code_changes", &[]).await?;

        info!("Test data cleaned up");
        Ok(())
    }
}

impl Drop for PostgresService {
    fn drop(&mut self) {
        if let Some(mut handle) = self.process_handle.take() {
            let _ = handle.kill();
        }
    }
}
