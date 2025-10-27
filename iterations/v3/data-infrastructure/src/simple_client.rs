//! Simple database client wrapper
//!
//! This provides a simple interface to the complex DatabaseClient
//! for backwards compatibility with existing code.

use crate::client::orchestrator::DatabaseClient as ComplexDatabaseClient;
use crate::database_config::DatabaseConfig;
use anyhow::Result;
use sqlx::postgres::PgPool;
use std::sync::Arc;

/// Simple database client that wraps the complex DatabaseClient
#[derive(Clone, Debug)]
pub struct DatabaseClient {
    inner: Arc<ComplexDatabaseClient>,
}

impl DatabaseClient {
    /// Create a new database client with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let inner = ComplexDatabaseClient::new(config).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        self.inner.pool()
    }

    /// Execute a parameterized query
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute(query, params).await
    }

    /// Execute a query and return rows
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        self.inner.query_with_params(query, params).await
    }

    /// Execute a query and return a single row (if any)
    pub async fn query_one(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Option<sqlx::postgres::PgRow>> {
        self.inner.query_one_with_params(query, params).await
    }

    /// Execute a parameterized query and return rows (alias for query)
    pub async fn query_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        self.inner.query_with_params(query, params).await
    }

    /// Execute a parameterized query and return a single row (if any)
    pub async fn query_one_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Option<sqlx::postgres::PgRow>> {
        self.inner.query_one_with_params(query, params).await
    }

    /// Execute a safe query (alias for execute with empty params)
    pub async fn execute_safe_query(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute_safe_query(query).await
    }

    /// Execute a parameterized query (alias for execute)
    pub async fn execute_parameterized_query(
        &self,
        query: &str,
        params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)>,
    ) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute(query, &params).await
    }
}
