//! PostgreSQL Connection Pool
//!
//! Manages database connections with health checks.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

use crate::config::PostgresConfig;
use crate::error::PostgresError;

/// PostgreSQL connection pool wrapper
#[derive(Clone)]
pub struct ConnectionPool {
    pool: PgPool,
    config: Arc<PostgresConfig>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: PostgresConfig) -> Result<Self, PostgresError> {
        tracing::info!(
            host = %config.host,
            port = config.port,
            database = %config.database,
            "Creating PostgreSQL connection pool"
        );

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .test_before_acquire(true)
            .connect(&config.connection_url())
            .await
            .map_err(|e| PostgresError::ConnectionFailed(e.to_string()))?;

        tracing::info!("PostgreSQL connection pool created successfully");

        Ok(Self {
            pool,
            config: Arc::new(config),
        })
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the configuration
    pub fn config(&self) -> &PostgresConfig {
        &self.config
    }

    /// Check if the pool is healthy
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max_connections: self.config.max_connections,
        }
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<(), PostgresError> {
        tracing::info!("Running database migrations");

        // Create extensions
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&self.pool)
            .await
            .map_err(|e| PostgresError::MigrationFailed(format!("Failed to create vector extension: {}", e)))?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(&self.pool)
            .await
            .map_err(|e| PostgresError::MigrationFailed(format!("Failed to create uuid extension: {}", e)))?;

        // Create tables
        self.create_events_table().await?;
        self.create_tasks_table().await?;
        self.create_embeddings_table().await?;
        self.create_chunks_table().await?;

        tracing::info!("Migrations completed successfully");
        Ok(())
    }

    async fn create_events_table(&self) -> Result<(), PostgresError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sequence_number BIGSERIAL UNIQUE NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                aggregate_type VARCHAR(100),
                aggregate_id VARCHAR(255),
                payload JSONB NOT NULL,
                content_hash VARCHAR(64) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                -- Indexes
                CONSTRAINT events_sequence_idx UNIQUE (sequence_number)
            );

            CREATE INDEX IF NOT EXISTS events_aggregate_idx
                ON events(aggregate_type, aggregate_id);
            CREATE INDEX IF NOT EXISTS events_type_idx
                ON events(event_type);
            CREATE INDEX IF NOT EXISTS events_created_idx
                ON events(created_at);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PostgresError::MigrationFailed(format!("Events table: {}", e)))?;

        Ok(())
    }

    async fn create_tasks_table(&self) -> Result<(), PostgresError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id VARCHAR(255) PRIMARY KEY,
                title VARCHAR(500) NOT NULL,
                description TEXT,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                priority VARCHAR(20) NOT NULL DEFAULT 'normal',
                environment VARCHAR(50) NOT NULL DEFAULT 'development',
                constraints JSONB,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                completed_at TIMESTAMPTZ,

                -- Results
                result_hash VARCHAR(64),
                council_verdict JSONB,
                execution_time_ms BIGINT
            );

            CREATE INDEX IF NOT EXISTS tasks_status_idx ON tasks(status);
            CREATE INDEX IF NOT EXISTS tasks_priority_idx ON tasks(priority);
            CREATE INDEX IF NOT EXISTS tasks_created_idx ON tasks(created_at);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PostgresError::MigrationFailed(format!("Tasks table: {}", e)))?;

        Ok(())
    }

    async fn create_embeddings_table(&self) -> Result<(), PostgresError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS embeddings (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                source_type VARCHAR(50) NOT NULL,
                source_id VARCHAR(255) NOT NULL,
                content_hash VARCHAR(64) NOT NULL,
                embedding vector(1536),
                model VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                -- Prevent duplicate embeddings for same content
                CONSTRAINT embeddings_unique_content
                    UNIQUE (source_type, source_id, content_hash, model)
            );

            -- HNSW index for fast similarity search
            CREATE INDEX IF NOT EXISTS embeddings_vector_idx
                ON embeddings USING hnsw (embedding vector_cosine_ops);
            CREATE INDEX IF NOT EXISTS embeddings_source_idx
                ON embeddings(source_type, source_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PostgresError::MigrationFailed(format!("Embeddings table: {}", e)))?;

        Ok(())
    }

    async fn create_chunks_table(&self) -> Result<(), PostgresError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chunks (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                file_path VARCHAR(1000) NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                content_hash VARCHAR(64) NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                language VARCHAR(50),
                embedding_id UUID REFERENCES embeddings(id),
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                -- Unique chunk per file position
                CONSTRAINT chunks_unique_position
                    UNIQUE (file_path, chunk_index)
            );

            CREATE INDEX IF NOT EXISTS chunks_file_idx ON chunks(file_path);
            CREATE INDEX IF NOT EXISTS chunks_language_idx ON chunks(language);
            CREATE INDEX IF NOT EXISTS chunks_embedding_idx ON chunks(embedding_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PostgresError::MigrationFailed(format!("Chunks table: {}", e)))?;

        Ok(())
    }

    /// Close the pool
    pub async fn close(&self) {
        tracing::info!("Closing PostgreSQL connection pool");
        self.pool.close().await;
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Current pool size
    pub size: u32,
    /// Number of idle connections
    pub idle: usize,
    /// Maximum connections configured
    pub max_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            size: 5,
            idle: 3,
            max_connections: 10,
        };
        assert_eq!(stats.size, 5);
        assert_eq!(stats.idle, 3);
    }
}
