//! PostgreSQL service management for E2E testing
//!
//! Provides real database integration with local PostgreSQL instance.
//! NO mocks - actual database connections, queries, and transactions.

use std::sync::Arc;
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn, error};
use std::time::Duration;
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use refinery::embed_migrations;

// Embed migrations from the migrations directory
embed_migrations!("migrations");

/// PostgreSQL service for real database operations with connection pooling and migrations
pub struct PostgresService {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    process_handle: Option<std::process::Child>,
    pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
    migrations_applied: bool,
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
            pool: None,
            migrations_applied: false,
        })
    }

    /// Start local PostgreSQL service with connection pooling
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting PostgreSQL service...");

        // Check if PostgreSQL is already running and pool exists
        if self.pool.is_some() && self.health_check().await {
            info!("PostgreSQL service already running with connection pool");
            return Ok(());
        }

        // For local testing, we'll assume PostgreSQL is already installed and running
        // In a real CI environment, you might need to start it via docker-compose
        warn!("PostgreSQL service not running. In CI, start with: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=test postgres:13");

        // Wait for PostgreSQL to be available
        for _ in 0..10 {
            if self.health_check().await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        // Initialize connection pool
        self.initialize_pool().await?;

        // Apply migrations
        self.apply_migrations().await?;

        info!("PostgreSQL service started with connection pooling and migrations");
        Ok(())
    }

    /// Stop PostgreSQL service and close connection pool
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping PostgreSQL service...");

        // Close connection pool
        self.pool = None;
        self.migrations_applied = false;

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
        match self.get_connection().await {
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

    /// Initialize connection pool
    async fn initialize_pool(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing PostgreSQL connection pool...");

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.username, self.password
        );

        let manager = PostgresConnectionManager::new_from_stringlike(&connection_string, NoTls)
            .map_err(|e| format!("Failed to create connection manager: {}", e))?;

        let pool = Pool::builder()
            .max_size(10) // Maximum connections in pool
            .min_idle(Some(1)) // Minimum idle connections
            .build(manager)
            .await
            .map_err(|e| format!("Failed to create connection pool: {}", e))?;

        // Test the pool by getting a connection
        let cloned_pool = pool.clone();
        let _connection = cloned_pool.get().await
            .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

        self.pool = Some(pool);
        info!("PostgreSQL connection pool initialized");
        Ok(())
    }

    /// Apply database migrations
    async fn apply_migrations(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.migrations_applied {
            info!("Migrations already applied");
            return Ok(());
        }

        info!("Applying database migrations...");

        // Format connection string for tokio_postgres::connect (requires postgres:// URI format)
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        );

        // Apply migrations using refinery
        // Create a direct Client connection for migrations (refinery needs Client, not PooledConnection)
        let (mut client, connection) = tokio_postgres::connect(
            &connection_string,
            NoTls,
        )
        .await
        .map_err(|e| format!("Failed to create migration client: {}", e))?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("PostgreSQL migration connection error: {}", e);
            }
        });

        let migrations = vec![
            refinery::Migration::unapplied(
                "V1__initial_test_schema",
                include_str!("../../migrations/V1__initial_test_schema.sql"),
            )?,
        ];

        let report = refinery::Runner::new(&migrations)
            .run_async(&mut client)
            .await
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        info!("Applied {} migration(s)", report.applied_migrations().len());
        self.migrations_applied = true;
        Ok(())
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>, Box<dyn std::error::Error + Send + Sync>> {
        match self.pool {
            Some(ref pool) => {
                pool.get().await.map_err(|e| format!("Failed to get connection from pool: {}", e).into())
            }
            None => Err("Connection pool not initialized".into())
        }
    }

    /// Get database connection (legacy method for backward compatibility)
    pub async fn connect(&self) -> Result<Arc<Client>, Box<dyn std::error::Error + Send + Sync>> {
        // For backward compatibility, create a new connection
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

        let client_arc = Arc::new(client);
        Ok(client_arc)
    }

    /// Execute a query and return results (using connection pool)
    pub async fn execute_query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.get_connection().await?;
        let rows = conn.query(query, params).await?;
        Ok(rows)
    }

    /// Execute a query that doesn't return results (using connection pool)
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.get_connection().await?;
        let result = conn.execute(query, params).await?;
        Ok(result)
    }

    /// Execute a query with pooled connection (alternative method)
    pub async fn execute_pooled<F, T>(
        &self,
        f: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&Client) -> futures::future::BoxFuture<'_, Result<T, tokio_postgres::Error>> + Send,
    {
        let conn = self.get_connection().await?;
        f(&*conn).await.map_err(|e| format!("Query execution failed: {}", e).into())
    }

    /// Execute multiple queries in a transaction
    pub async fn execute_transaction<F, T>(
        &self,
        f: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: for<'a> FnOnce(&'a tokio_postgres::Transaction<'a>) -> futures::future::BoxFuture<'a, Result<T, tokio_postgres::Error>> + Send,
    {
        // Get a connection from the pool - PooledConnection dereferences to Client
        let pool = self.pool.as_ref().ok_or("Connection pool not initialized")?;
        let mut client = pool.get().await
            .map_err(|e| format!("Failed to get connection from pool: {}", e))?;
        
        // Start a transaction (PooledConnection derefs to Client)
        let transaction = client.transaction().await
            .map_err(|e| format!("Failed to start transaction: {}", e))?;
        
        // Execute the closure with the transaction
        let result = f(&transaction).await
            .map_err(|e| format!("Transaction execution failed: {}", e))?;
        
        // Commit the transaction
        transaction.commit().await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        
        Ok(result)
    }

    /// Create test tables and data (using migrations)
    pub async fn setup_test_schema(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up test schema via migrations...");

        // Migrations are now handled automatically in start()
        // This method is kept for backward compatibility and additional setup
        if !self.migrations_applied {
            return Err("Migrations not applied. Call start() first.".into());
        }

        // Additional setup can be done here if needed
        info!("Test schema setup completed (migrations applied)");
        Ok(())
    }

    /// Clean up test data (drop tables created by migrations)
    pub async fn cleanup_test_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up test data...");

        self.execute_transaction(|client| {
            Box::pin(async move {
                client.execute("DROP TABLE IF EXISTS test_research CASCADE", &[]).await?;
                client.execute("DROP TABLE IF EXISTS test_code_changes CASCADE", &[]).await?;
                client.execute("DROP TABLE IF EXISTS test_agent_runs CASCADE", &[]).await?;
                Ok(())
            })
        }).await?;

        info!("Test data cleaned up");
        Ok(())
    }

    /// Get connection pool statistics
    pub fn pool_stats(&self) -> Option<bb8::State> {
        self.pool.as_ref().map(|pool| pool.state())
    }

    /// Check if migrations have been applied
    pub fn migrations_applied(&self) -> bool {
        self.migrations_applied
    }

    /// Manually apply migrations (useful for testing)
    pub async fn apply_migrations_manual(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.apply_migrations().await
    }

    /// Create a test database manager with lifecycle management
    /// 
    /// This creates an isolated test database for better test isolation and parallel execution.
    pub async fn create_lifecycle_manager(&self, test_id: Option<String>) -> Result<crate::database_lifecycle::TestDatabaseManager, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = format!(
            "postgres://{}:{}@{}:{}/postgres",
            self.username, self.password, self.host, self.port
        );
        
        crate::database_lifecycle::TestDatabaseManager::new(&base_url, test_id)
            .await
            .map_err(|e| format!("Failed to create test database manager: {}", e).into())
    }
}

impl Drop for PostgresService {
    fn drop(&mut self) {
        // Note: Connection pool is automatically closed when dropped
        // Process handle cleanup
        if let Some(mut handle) = self.process_handle.take() {
            let _ = handle.kill();
        }
    }
}
