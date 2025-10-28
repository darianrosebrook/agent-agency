//! Simple database client wrapper
//!
//! This provides a simple interface to the complex DatabaseClient
//! for backwards compatibility with existing code.

use crate::client::orchestrator::DatabaseClient as ComplexDatabaseClient;
use crate::database_config::DatabaseConfig;
use crate::database_operations::DatabaseOperations;
use anyhow::Result;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use uuid::Uuid;

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

    /// Get a reference to the health monitor (if available)
    pub fn health_monitor(&self) -> Option<&crate::health::DatabaseHealthMonitor> {
        self.inner.health_monitor.as_ref().map(|arc| arc.as_ref())
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

    /// List all waivers
    pub async fn list_waivers(&self) -> Result<Vec<crate::models::Waiver>> {
        // TODO: Implement waiver listing
        Ok(vec![])
    }

    /// Create a new waiver
    pub async fn create_waiver(&self, _waiver: &crate::models::Waiver) -> Result<Uuid> {
        // TODO: Implement waiver creation
        Ok(Uuid::new_v4())
    }

    /// Approve a waiver
    pub async fn approve_waiver(&self, _waiver_id: &Uuid) -> Result<()> {
        // TODO: Implement waiver approval
        Ok(())
    }

    /// Get task provenance
    pub async fn get_task_provenance(&self, _task_id: &Uuid) -> Result<Vec<crate::models::ProvenanceEntry>> {
        // TODO: Implement task provenance retrieval
        Ok(vec![])
    }

    /// Create a task
    pub async fn create_task(&self, task: &crate::models::Task) -> Result<Uuid> {
        // Convert models::Task to database_operations::CreateTask
        let create_task = crate::database_operations::CreateTask {
            title: task.title.clone(),
            description: task.description.clone(),
            risk_tier: task.risk_tier.clone(),
            scope: task.scope.clone(),
            acceptance_criteria: task.acceptance_criteria.clone(),
            context: task.context.clone(),
            caws_spec: task.caws_spec.clone(),
            status: task.status.clone(),
            assigned_worker_id: task.assigned_worker_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata.clone(),
        };
        
        let created_task = self.inner.create_task(create_task).await?;
        Ok(created_task.id)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &Uuid) -> Result<Option<crate::models::Task>> {
        let task = self.inner.get_task(*task_id).await?;
        Ok(task.map(|t| crate::models::Task {
            id: t.id,
            title: t.title,
            description: t.description,
            risk_tier: t.risk_tier,
            scope: t.scope,
            acceptance_criteria: t.acceptance_criteria,
            context: t.context,
            caws_spec: t.caws_spec,
            status: t.status,
            assigned_worker_id: t.assigned_worker_id,
            priority: t.priority,
            deadline: t.deadline,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
            completed_at: t.completed_at,
        }))
    }
}
