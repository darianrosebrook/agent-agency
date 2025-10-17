//! Database client implementation with connection pooling and query methods

use crate::{DatabaseConfig, models::*};
use anyhow::{Context, Result};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use sqlx::{PgPool, Postgres, Row};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Main database client with connection pooling
#[derive(Debug, Clone)]
pub struct DatabaseClient {
    pool: PgPool,
    config: DatabaseConfig,
}

impl DatabaseClient {
    /// Create a new database client with connection pooling
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}:{}", config.host, config.port);

        // Create connection pool
        let pool = PgPool::connect_with(
            sqlx::postgres::PgConnectOptions::new()
                .host(&config.host)
                .port(config.port)
                .database(&config.database)
                .username(&config.username)
                .password(&config.password)
                .application_name("agent-agency-v3")
        ).await
        .context("Failed to create database connection pool")?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .context("Failed to test database connection")?;

        info!("Successfully connected to database");
        Ok(Self { pool, config })
    }

    /// Create database client with deadpool (alternative implementation)
    pub async fn with_deadpool(config: DatabaseConfig) -> Result<Self> {
        let mut pg_config = Config::new();
        pg_config.host = Some(config.host.clone());
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.database.clone());
        pg_config.user = Some(config.username.clone());
        pg_config.password = Some(config.password.clone());
        pg_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        pg_config.pool = Some(deadpool_postgres::PoolConfig {
            max_size: config.pool_max as usize,
            min_size: Some(config.pool_min as usize),
            ..Default::default()
        });

        let pool = pg_config
            .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .context("Failed to create deadpool connection pool")?;

        // Convert deadpool to sqlx pool for compatibility
        // This is a simplified approach - in production you might want to use deadpool directly
        let sqlx_pool = PgPool::connect(&config.database_url()).await
            .context("Failed to create sqlx connection pool")?;

        Ok(Self { pool: sqlx_pool, config })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let pool_stats = self.pool.size();
        let idle_connections = self.pool.num_idle();
        
        // Get table row counts
        let tables = [
            "judges", "workers", "tasks", "task_executions",
            "council_verdicts", "judge_evaluations", "debate_sessions",
            "knowledge_entries", "performance_metrics", "caws_compliance", "audit_trail"
        ];

        let mut table_counts = std::collections::HashMap::new();
        for table in tables {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            table_counts.insert(table.to_string(), count);
        }

        Ok(DatabaseStats {
            pool_size: pool_stats,
            idle_connections,
            table_counts,
            uptime: None, // Could be implemented with a startup timestamp
        })
    }

    /// Execute a migration
    pub async fn migrate(&self, migration_sql: &str) -> Result<()> {
        info!("Executing database migration");
        
        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .context("Failed to execute migration")?;

        info!("Migration completed successfully");
        Ok(())
    }

    /// Create the database if it doesn't exist
    pub async fn ensure_database_exists(&self) -> Result<()> {
        let server_url = self.config.server_url();
        let db_name = &self.config.database;

        // Connect to postgres database to create our database
        let server_pool = PgPool::connect(&format!("{}/postgres", server_url)).await
            .context("Failed to connect to postgres database")?;

        // Check if database exists
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
        )
        .bind(db_name)
        .fetch_one(&server_pool)
        .await
        .context("Failed to check database existence")?;

        if !exists {
            info!("Creating database: {}", db_name);
            sqlx::query(&format!("CREATE DATABASE {}", db_name))
                .execute(&server_pool)
                .await
                .context("Failed to create database")?;
            info!("Database created successfully");
        } else {
            debug!("Database already exists: {}", db_name);
        }

        server_pool.close().await;
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStats {
    pub pool_size: u32,
    pub idle_connections: u32,
    pub table_counts: std::collections::HashMap<String, i64>,
    pub uptime: Option<Duration>,
}

/// Database operations trait for type-safe queries
#[async_trait]
pub trait DatabaseOperations {
    type Error;

    // Judge operations
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge, Self::Error>;
    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>, Self::Error>;
    async fn get_judges(&self) -> Result<Vec<Judge>, Self::Error>;
    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge, Self::Error>;
    async fn delete_judge(&self, id: Uuid) -> Result<(), Self::Error>;

    // Worker operations
    async fn create_worker(&self, worker: CreateWorker) -> Result<Worker, Self::Error>;
    async fn get_worker(&self, id: Uuid) -> Result<Option<Worker>, Self::Error>;
    async fn get_workers(&self) -> Result<Vec<Worker>, Self::Error>;
    async fn get_workers_by_type(&self, worker_type: &str) -> Result<Vec<Worker>, Self::Error>;
    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker, Self::Error>;
    async fn delete_worker(&self, id: Uuid) -> Result<(), Self::Error>;

    // Task operations
    async fn create_task(&self, task: CreateTask) -> Result<Task, Self::Error>;
    async fn get_task(&self, id: Uuid) -> Result<Option<Task>, Self::Error>;
    async fn get_tasks(&self, filters: Option<TaskFilters>, pagination: Option<PaginationParams>) -> Result<Vec<Task>, Self::Error>;
    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task, Self::Error>;
    async fn delete_task(&self, id: Uuid) -> Result<(), Self::Error>;

    // Task execution operations
    async fn create_task_execution(&self, execution: CreateTaskExecution) -> Result<TaskExecution, Self::Error>;
    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>, Self::Error>;
    async fn update_task_execution(&self, id: Uuid, update: UpdateTaskExecution) -> Result<TaskExecution, Self::Error>;

    // Council verdict operations
    async fn create_council_verdict(&self, verdict: CreateCouncilVerdict) -> Result<CouncilVerdict, Self::Error>;
    async fn get_council_verdict(&self, verdict_id: Uuid) -> Result<Option<CouncilVerdict>, Self::Error>;
    async fn get_council_verdicts(&self, filters: Option<VerdictFilters>, pagination: Option<PaginationParams>) -> Result<Vec<CouncilVerdict>, Self::Error>;

    // Judge evaluation operations
    async fn create_judge_evaluation(&self, evaluation: CreateJudgeEvaluation) -> Result<JudgeEvaluation, Self::Error>;
    async fn get_judge_evaluations(&self, verdict_id: Uuid) -> Result<Vec<JudgeEvaluation>, Self::Error>;

    // Knowledge entry operations
    async fn create_knowledge_entry(&self, entry: CreateKnowledgeEntry) -> Result<KnowledgeEntry, Self::Error>;
    async fn get_knowledge_entries(&self, filters: Option<KnowledgeFilters>, pagination: Option<PaginationParams>) -> Result<Vec<KnowledgeEntry>, Self::Error>;
    async fn search_knowledge(&self, query: &str, limit: Option<u32>) -> Result<Vec<KnowledgeEntry>, Self::Error>;

    // Performance metric operations
    async fn create_performance_metric(&self, metric: CreatePerformanceMetric) -> Result<PerformanceMetric, Self::Error>;
    async fn get_performance_metrics(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<PerformanceMetric>, Self::Error>;

    // CAWS compliance operations
    async fn create_caws_compliance(&self, compliance: CreateCawsCompliance) -> Result<CawsCompliance, Self::Error>;
    async fn get_caws_compliance(&self, task_id: Uuid) -> Result<Option<CawsCompliance>, Self::Error>;

    // Audit trail operations
    async fn create_audit_trail_entry(&self, entry: CreateAuditTrailEntry) -> Result<AuditTrailEntry, Self::Error>;
    async fn get_audit_trail(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<AuditTrailEntry>, Self::Error>;

    // Analytics and statistics
    async fn get_council_metrics(&self) -> Result<Vec<CouncilMetrics>, Self::Error>;
    async fn get_judge_performance(&self) -> Result<Vec<JudgePerformance>, Self::Error>;
    async fn get_worker_performance(&self) -> Result<Vec<WorkerPerformance>, Self::Error>;
    async fn get_task_execution_summary(&self, task_id: Uuid) -> Result<Option<TaskExecutionSummary>, Self::Error>;
}

#[async_trait]
impl DatabaseOperations for DatabaseClient {
    type Error = anyhow::Error;

    // Judge operations implementation
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge, Self::Error> {
        let created = sqlx::query_as::<_, Judge>(
            "INSERT INTO judges (name, model_name, endpoint, weight, timeout_ms, optimization_target) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING *"
        )
        .bind(&judge.name)
        .bind(&judge.model_name)
        .bind(&judge.endpoint)
        .bind(judge.weight)
        .bind(judge.timeout_ms)
        .bind(&judge.optimization_target)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create judge")?;

        info!("Created judge: {}", created.name);
        Ok(created)
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>, Self::Error> {
        let judge = sqlx::query_as::<_, Judge>(
            "SELECT * FROM judges WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get judge")?;

        Ok(judge)
    }

    async fn get_judges(&self) -> Result<Vec<Judge>, Self::Error> {
        let judges = sqlx::query_as::<_, Judge>(
            "SELECT * FROM judges ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get judges")?;

        Ok(judges)
    }

    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge, Self::Error> {
        // Build dynamic update query
        let mut query = "UPDATE judges SET updated_at = NOW()".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(name) = update.name {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
            params.push(Box::new(name));
        }

        if let Some(model_name) = update.model_name {
            param_count += 1;
            query.push_str(&format!(", model_name = ${}", param_count));
            params.push(Box::new(model_name));
        }

        if let Some(endpoint) = update.endpoint {
            param_count += 1;
            query.push_str(&format!(", endpoint = ${}", param_count));
            params.push(Box::new(endpoint));
        }

        if let Some(weight) = update.weight {
            param_count += 1;
            query.push_str(&format!(", weight = ${}", param_count));
            params.push(Box::new(weight));
        }

        if let Some(timeout_ms) = update.timeout_ms {
            param_count += 1;
            query.push_str(&format!(", timeout_ms = ${}", param_count));
            params.push(Box::new(timeout_ms));
        }

        if let Some(optimization_target) = update.optimization_target {
            param_count += 1;
            query.push_str(&format!(", optimization_target = ${}", param_count));
            params.push(Box::new(optimization_target));
        }

        if let Some(is_active) = update.is_active {
            param_count += 1;
            query.push_str(&format!(", is_active = ${}", param_count));
            params.push(Box::new(is_active));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(Box::new(id));

        // Execute the update
        let updated = sqlx::query_as::<_, Judge>(&query)
            .execute(&self.pool)
            .await
            .context("Failed to update judge")?;

        Ok(updated)
    }

    async fn delete_judge(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM judges WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete judge")?;

        info!("Deleted judge: {}", id);
        Ok(())
    }

    // Placeholder implementations for other operations
    // In a full implementation, these would be properly implemented
    async fn create_worker(&self, _worker: CreateWorker) -> Result<Worker, Self::Error> {
        todo!("Implement create_worker")
    }

    async fn get_worker(&self, _id: Uuid) -> Result<Option<Worker>, Self::Error> {
        todo!("Implement get_worker")
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, Self::Error> {
        todo!("Implement get_workers")
    }

    async fn get_workers_by_type(&self, _worker_type: &str) -> Result<Vec<Worker>, Self::Error> {
        todo!("Implement get_workers_by_type")
    }

    async fn update_worker(&self, _id: Uuid, _update: UpdateWorker) -> Result<Worker, Self::Error> {
        todo!("Implement update_worker")
    }

    async fn delete_worker(&self, _id: Uuid) -> Result<(), Self::Error> {
        todo!("Implement delete_worker")
    }

    async fn create_task(&self, _task: CreateTask) -> Result<Task, Self::Error> {
        todo!("Implement create_task")
    }

    async fn get_task(&self, _id: Uuid) -> Result<Option<Task>, Self::Error> {
        todo!("Implement get_task")
    }

    async fn get_tasks(&self, _filters: Option<TaskFilters>, _pagination: Option<PaginationParams>) -> Result<Vec<Task>, Self::Error> {
        todo!("Implement get_tasks")
    }

    async fn update_task(&self, _id: Uuid, _update: UpdateTask) -> Result<Task, Self::Error> {
        todo!("Implement update_task")
    }

    async fn delete_task(&self, _id: Uuid) -> Result<(), Self::Error> {
        todo!("Implement delete_task")
    }

    async fn create_task_execution(&self, _execution: CreateTaskExecution) -> Result<TaskExecution, Self::Error> {
        todo!("Implement create_task_execution")
    }

    async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<TaskExecution>, Self::Error> {
        todo!("Implement get_task_executions")
    }

    async fn update_task_execution(&self, _id: Uuid, _update: UpdateTaskExecution) -> Result<TaskExecution, Self::Error> {
        todo!("Implement update_task_execution")
    }

    async fn create_council_verdict(&self, _verdict: CreateCouncilVerdict) -> Result<CouncilVerdict, Self::Error> {
        todo!("Implement create_council_verdict")
    }

    async fn get_council_verdict(&self, _verdict_id: Uuid) -> Result<Option<CouncilVerdict>, Self::Error> {
        todo!("Implement get_council_verdict")
    }

    async fn get_council_verdicts(&self, _filters: Option<VerdictFilters>, _pagination: Option<PaginationParams>) -> Result<Vec<CouncilVerdict>, Self::Error> {
        todo!("Implement get_council_verdicts")
    }

    async fn create_judge_evaluation(&self, _evaluation: CreateJudgeEvaluation) -> Result<JudgeEvaluation, Self::Error> {
        todo!("Implement create_judge_evaluation")
    }

    async fn get_judge_evaluations(&self, _verdict_id: Uuid) -> Result<Vec<JudgeEvaluation>, Self::Error> {
        todo!("Implement get_judge_evaluations")
    }

    async fn create_knowledge_entry(&self, _entry: CreateKnowledgeEntry) -> Result<KnowledgeEntry, Self::Error> {
        todo!("Implement create_knowledge_entry")
    }

    async fn get_knowledge_entries(&self, _filters: Option<KnowledgeFilters>, _pagination: Option<PaginationParams>) -> Result<Vec<KnowledgeEntry>, Self::Error> {
        todo!("Implement get_knowledge_entries")
    }

    async fn search_knowledge(&self, _query: &str, _limit: Option<u32>) -> Result<Vec<KnowledgeEntry>, Self::Error> {
        todo!("Implement search_knowledge")
    }

    async fn create_performance_metric(&self, _metric: CreatePerformanceMetric) -> Result<PerformanceMetric, Self::Error> {
        todo!("Implement create_performance_metric")
    }

    async fn get_performance_metrics(&self, _entity_type: &str, _entity_id: Uuid) -> Result<Vec<PerformanceMetric>, Self::Error> {
        todo!("Implement get_performance_metrics")
    }

    async fn create_caws_compliance(&self, _compliance: CreateCawsCompliance) -> Result<CawsCompliance, Self::Error> {
        todo!("Implement create_caws_compliance")
    }

    async fn get_caws_compliance(&self, _task_id: Uuid) -> Result<Option<CawsCompliance>, Self::Error> {
        todo!("Implement get_caws_compliance")
    }

    async fn create_audit_trail_entry(&self, _entry: CreateAuditTrailEntry) -> Result<AuditTrailEntry, Self::Error> {
        todo!("Implement create_audit_trail_entry")
    }

    async fn get_audit_trail(&self, _entity_type: &str, _entity_id: Uuid) -> Result<Vec<AuditTrailEntry>, Self::Error> {
        todo!("Implement get_audit_trail")
    }

    async fn get_council_metrics(&self) -> Result<Vec<CouncilMetrics>, Self::Error> {
        todo!("Implement get_council_metrics")
    }

    async fn get_judge_performance(&self) -> Result<Vec<JudgePerformance>, Self::Error> {
        todo!("Implement get_judge_performance")
    }

    async fn get_worker_performance(&self) -> Result<Vec<WorkerPerformance>, Self::Error> {
        todo!("Implement get_worker_performance")
    }

    async fn get_task_execution_summary(&self, _task_id: Uuid) -> Result<Option<TaskExecutionSummary>, Self::Error> {
        todo!("Implement get_task_execution_summary")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "agent_agency_v3");
    }

    #[tokio::test]
    async fn test_database_url() {
        let config = DatabaseConfig::default();
        let url = config.database_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("localhost:5432"));
        assert!(url.contains("agent_agency_v3"));
    }

    #[tokio::test]
    async fn test_server_url() {
        let config = DatabaseConfig::default();
        let url = config.server_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("localhost:5432"));
        assert!(!url.contains("agent_agency_v3")); // Should not include database name
    }
}
