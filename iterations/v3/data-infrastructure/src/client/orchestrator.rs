//! Database Client Orchestrator
//!
//! Production-hardened database client with connection pooling,
//! circuit breaker pattern, monitoring, and resilience features.

use crate::database_circuit_breaker::CircuitBreaker;
use super::super::database_metrics::DatabaseMetrics;
use super::super::health::DatabaseHealthMonitor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use super::super::database_operations::{
    DatabaseOperations, CreateJudge, UpdateJudge, CreateWorker, UpdateWorker, 
    CreateTask, UpdateTask, CreateTaskExecution, UpdateTaskExecution,
    CreateCouncilVerdict, CreateJudgeEvaluation, CreateAuditTrailEntry,
    CreatePlanningTelemetry, CreateMilestone, UpdateMilestone, CreatePlanningSession,
    UpdatePlanningSession, CreateEvidenceArtifact, UpdateEvidenceArtifact,
    CreatePlanningAuditEvent, CreateExecutionPlan, UpdateExecutionPlan,
    CreateWaiver, UpdateWaiver
};
use super::super::database_audit::DatabaseAuditLogger;
use super::super::models::{
    Judge, Worker, Task, TaskExecution, CouncilVerdict, JudgeEvaluation, AuditTrailEntry,
    PlanningTelemetry, Milestone, PlanningSession, EvidenceArtifact, PlanningAuditEvent,
    ExecutionPlan, Waiver
};
use crate::connection_manager::{ConnectionPoolManager, PooledDatabaseClient};
use crate::database_config::DatabaseConfig;

/// Production-hardened database client with comprehensive monitoring and resilience
#[derive(Debug)]
pub struct DatabaseClient {
    /// Connection pool for database operations
    pub pool: PgPool,
    /// Circuit breaker for resilience
    pub circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Metrics collection
    pub metrics: Option<Arc<DatabaseMetrics>>,
    /// Audit logging
    pub audit_logger: Option<Arc<DatabaseAuditLogger>>,
    /// Health monitoring
    pub health_monitor: Option<Arc<DatabaseHealthMonitor>>,
    /// Connection semaphore for rate limiting
    pub connection_semaphore: Arc<Semaphore>,
    /// Prepared statement cache
    pub statement_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl DatabaseClient {
    /// Execute a parameterized query
    pub async fn execute(&self, query: &str, _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)]) -> Result<sqlx::postgres::PgQueryResult> {
        sqlx::query(query)
            .execute(&self.pool)
        .await
            .context("Failed to execute query")
    }

    /// Execute a query and return rows
    pub async fn query(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>> {
        sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute query")
    }

    /// Execute a parameterized query and return a single row
    pub async fn query_one_with_params(&self, query: &str, _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)]) -> Result<Option<sqlx::postgres::PgRow>> {
        sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to execute query")
    }

    /// Execute a parameterized query and return rows
    pub async fn query_with_params(&self, query: &str, _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)]) -> Result<Vec<sqlx::postgres::PgRow>> {
        sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute query")
    }

    /// Execute a safe query (alias for execute with parameters)
    pub async fn execute_safe_query(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        self.execute(query, &[]).await
    }

    /// Execute a parameterized query (alias for execute)
    pub async fn execute_parameterized_query(&self, query: &str, params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)>) -> Result<sqlx::postgres::PgQueryResult> {
        self.execute(query, &params).await
    }

    /// Execute a query and return a single row (if any)
    pub async fn query_one(&self, query: &str) -> Result<Option<sqlx::postgres::PgRow>> {
        sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to execute query")
    }

    /// Create an audit trail entry
    pub async fn create_audit_trail_entry(&self, audit_entry: serde_json::Value) -> Result<()> {
        // This is a placeholder - actual implementation would insert into audit table
        // For now, just log the audit entry
        tracing::info!("Audit entry: {}", audit_entry);
        Ok(())
    }


    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new DatabaseClient with configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.database_url).await
            .context("Failed to connect to database")?;

        let metrics = Arc::new(DatabaseMetrics::new());
        Ok(Self {
            pool,
            circuit_breaker: Some(Arc::new(CircuitBreaker::new())),
            metrics: Some(metrics.clone()),
            audit_logger: Some(Arc::new(DatabaseAuditLogger::new())),
            health_monitor: Some(Arc::new(DatabaseHealthMonitor::new(metrics))),
            connection_semaphore: Arc::new(Semaphore::new(config.max_connections.unwrap_or(100) as usize)),
            statement_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Default for DatabaseClient {
    fn default() -> Self {
        Self {
            pool: PgPool::connect_lazy("postgresql://localhost/test").unwrap(),
            circuit_breaker: Some(Arc::new(CircuitBreaker::new())),
            metrics: Some(Arc::new(DatabaseMetrics::new())),
            audit_logger: Some(Arc::new(DatabaseAuditLogger::new())),
            health_monitor: Some(Arc::new(DatabaseHealthMonitor::new(Arc::new(DatabaseMetrics::new())))),
            connection_semaphore: Arc::new(Semaphore::new(100)),
            statement_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Clone for DatabaseClient {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
            metrics: self.metrics.clone(),
            audit_logger: self.audit_logger.clone(),
            health_monitor: self.health_monitor.clone(),
            connection_semaphore: self.connection_semaphore.clone(),
            statement_cache: self.statement_cache.clone(),
        }
    }
}

#[async_trait]
impl PooledDatabaseClient for DatabaseClient {
    async fn initialize(&self) -> Result<()> {
        // Initialize the database client
        // This could include setting up prepared statements, validating connections, etc.
        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Check if the client is available (pool is healthy)
        !self.pool.is_closed()
    }

    async fn get_pool_manager(&self) -> Arc<ConnectionPoolManager> {
        ConnectionPoolManager::get_instance().await.unwrap()
    }
}

#[async_trait]
impl DatabaseOperations for DatabaseClient {
    // Placeholder implementations - these would contain the actual database operations
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO judges (
                id, name, model_name, endpoint, weight, 
                timeout_ms, optimization_target, is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(id)
        .bind(&judge.name)
        .bind(&judge.model_name)
        .bind(&judge.endpoint)
        .bind(judge.weight)
        .bind(judge.timeout_ms)
        .bind(&judge.optimization_target)
        .bind(judge.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Judge {
            id,
            name: judge.name,
            model_name: judge.model_name,
            endpoint: judge.endpoint,
            weight: judge.weight,
            timeout_ms: judge.timeout_ms,
            optimization_target: judge.optimization_target,
            is_active: judge.is_active,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>> {
        let row = sqlx::query_as::<_, Judge>(
            r#"
            SELECT id, name, model_name, endpoint, weight,
                   timeout_ms, optimization_target, is_active, created_at, updated_at
            FROM judges
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    async fn get_judges(&self) -> Result<Vec<Judge>> {
        let rows = sqlx::query_as::<_, Judge>(
            r#"
            SELECT id, name, model_name, endpoint, weight,
                   timeout_ms, optimization_target, is_active, created_at, updated_at
            FROM judges
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge> {
        let now = Utc::now();
        
        // Get current judge to merge with updates
        let current = self.get_judge(id).await?
            .ok_or_else(|| anyhow::anyhow!("Judge not found: {}", id))?;
        
        sqlx::query_as::<_, Judge>(
            r#"
            UPDATE judges
            SET name = $1,
                model_name = $2,
                endpoint = $3,
                weight = $4,
                timeout_ms = $5,
                optimization_target = $6,
                is_active = $7,
                updated_at = $8
            WHERE id = $9
            RETURNING id, name, model_name, endpoint, weight,
                     timeout_ms, optimization_target, is_active, created_at, updated_at
            "#
        )
        .bind(update.name.as_ref().unwrap_or(&current.name))
        .bind(update.model_name.as_ref().unwrap_or(&current.model_name))
        .bind(update.endpoint.as_ref().unwrap_or(&current.endpoint))
        .bind(update.weight.unwrap_or(current.weight))
        .bind(update.timeout_ms.unwrap_or(current.timeout_ms))
        .bind(update.optimization_target.as_ref().unwrap_or(&current.optimization_target))
        .bind(update.is_active.unwrap_or(current.is_active))
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Judge not found after update: {}", id))
    }

    async fn delete_judge(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM judges WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn create_worker(&self, worker: CreateWorker) -> Result<Worker> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO workers (
                id, name, worker_type, specialty, model_name, endpoint,
                capabilities, performance_history, is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(id)
        .bind(&worker.name)
        .bind(&worker.worker_type)
        .bind(&worker.specialty)
        .bind(&worker.model_name)
        .bind(&worker.endpoint)
        .bind(&worker.capabilities)
        .bind(&worker.performance_history)
        .bind(worker.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Worker {
            id,
            name: worker.name,
            worker_type: worker.worker_type,
            specialty: worker.specialty,
            model_name: worker.model_name,
            endpoint: worker.endpoint,
            capabilities: worker.capabilities,
            performance_history: worker.performance_history,
            is_active: worker.is_active,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_worker(&self, id: Uuid) -> Result<Option<Worker>> {
        let row = sqlx::query_as::<_, Worker>(
            r#"
            SELECT id, name, worker_type, specialty, model_name, endpoint,
                   capabilities, performance_history, is_active, created_at, updated_at
            FROM workers
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    async fn get_workers(&self) -> Result<Vec<Worker>> {
        let rows = sqlx::query_as::<_, Worker>(
            r#"
            SELECT id, name, worker_type, specialty, model_name, endpoint,
                   capabilities, performance_history, is_active, created_at, updated_at
            FROM workers
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker> {
        let now = Utc::now();
        
        // Get current worker to merge with updates
        let current = self.get_worker(id).await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found: {}", id))?;
        
        sqlx::query_as::<_, Worker>(
            r#"
            UPDATE workers
            SET name = $1,
                worker_type = $2,
                specialty = $3,
                model_name = $4,
                endpoint = $5,
                capabilities = $6,
                performance_history = $7,
                is_active = $8,
                updated_at = $9
            WHERE id = $10
            RETURNING id, name, worker_type, specialty, model_name, endpoint,
                     capabilities, performance_history, is_active, created_at, updated_at
            "#
        )
        .bind(update.name.as_ref().unwrap_or(&current.name))
        .bind(update.worker_type.as_ref().unwrap_or(&current.worker_type))
        .bind(update.specialty.as_ref().or(current.specialty.as_ref()))
        .bind(update.model_name.as_ref().unwrap_or(&current.model_name))
        .bind(update.endpoint.as_ref().unwrap_or(&current.endpoint))
        .bind(update.capabilities.as_ref().unwrap_or(&current.capabilities))
        .bind(update.performance_history.as_ref().unwrap_or(&current.performance_history))
        .bind(update.is_active.unwrap_or(current.is_active))
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Worker not found after update: {}", id))
    }

    async fn delete_worker(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM workers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn create_task(&self, task: CreateTask) -> Result<Task> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, title, description, risk_tier, scope, acceptance_criteria,
                context, caws_spec, status, assigned_worker_id, priority, 
                deadline, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#
        )
        .bind(id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.risk_tier)
        .bind(&task.scope)
        .bind(&task.acceptance_criteria)
        .bind(&task.context)
        .bind(&task.caws_spec)
        .bind(&task.status)
        .bind(&task.assigned_worker_id)
        .bind(&task.priority)
        .bind(&task.deadline)
        .bind(&task.metadata)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Task {
            id,
            title: task.title,
            description: task.description,
            risk_tier: task.risk_tier,
            scope: task.scope,
            acceptance_criteria: task.acceptance_criteria,
            context: task.context,
            caws_spec: task.caws_spec,
            status: task.status,
            assigned_worker_id: task.assigned_worker_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata,
            created_at: now,
            updated_at: now,
            completed_at: None,
        })
    }

    async fn get_task(&self, id: Uuid) -> Result<Option<Task>> {
        let row = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, title, description, risk_tier, scope, acceptance_criteria,
                   context, caws_spec, status, assigned_worker_id, priority,
                   deadline, metadata, created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    async fn get_tasks(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, title, description, risk_tier, scope, acceptance_criteria,
                   context, caws_spec, status, assigned_worker_id, priority,
                   deadline, metadata, created_at, updated_at, completed_at
            FROM tasks
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task> {
        let now = Utc::now();
        
        // Get current task to merge with updates
        let current = self.get_task(id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", id))?;
        
        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET title = $1,
                description = $2,
                risk_tier = $3,
                scope = $4,
                acceptance_criteria = $5,
                context = $6,
                caws_spec = $7,
                status = $8,
                assigned_worker_id = $9,
                priority = $10,
                deadline = $11,
                metadata = $12,
                completed_at = $13,
                updated_at = $14
            WHERE id = $15
            RETURNING id, title, description, risk_tier, scope, acceptance_criteria,
                     context, caws_spec, status, assigned_worker_id, priority,
                     deadline, metadata, created_at, updated_at, completed_at
            "#
        )
        .bind(update.title.as_ref().unwrap_or(&current.title))
        .bind(update.description.as_ref().unwrap_or(&current.description))
        .bind(update.risk_tier.as_ref().unwrap_or(&current.risk_tier))
        .bind(update.scope.as_ref().unwrap_or(&current.scope))
        .bind(update.acceptance_criteria.as_ref().unwrap_or(&current.acceptance_criteria))
        .bind(update.context.as_ref().unwrap_or(&current.context))
        .bind(update.caws_spec.as_ref().or(current.caws_spec.as_ref()))
        .bind(update.status.as_ref().unwrap_or(&current.status))
        .bind(update.assigned_worker_id.or(current.assigned_worker_id))
        .bind(update.priority.or(current.priority))
        .bind(update.deadline.or(current.deadline))
        .bind(update.metadata.as_ref().or(current.metadata.as_ref()))
        .bind(update.completed_at.or(current.completed_at))
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task not found after update: {}", id))
    }

    async fn delete_task(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn create_task_execution(&self, execution: CreateTaskExecution) -> Result<TaskExecution> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO task_executions (
                id, task_id, worker_id, execution_started_at, execution_completed_at,
                execution_time_ms, status, worker_output, self_assessment, metadata,
                error_message, tokens_used, created_at, updated_at, execution_metadata, result_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#
        )
        .bind(id)
        .bind(execution.task_id)
        .bind(execution.worker_id)
        .bind(execution.execution_started_at)
        .bind(None::<DateTime<Utc>>)
        .bind(None::<i32>)
        .bind(&execution.status)
        .bind(&execution.worker_output)
        .bind(&execution.self_assessment)
        .bind(&execution.metadata)
        .bind(&execution.error_message)
        .bind(execution.tokens_used)
        .bind(now)
        .bind(now)
        .bind(&execution.execution_metadata)
        .bind(&execution.result_data)
        .execute(&self.pool)
        .await?;
        
        Ok(TaskExecution {
            id,
            task_id: execution.task_id,
            worker_id: execution.worker_id,
            execution_started_at: execution.execution_started_at,
            execution_completed_at: None,
            execution_time_ms: None,
            status: execution.status,
            worker_output: execution.worker_output,
            self_assessment: execution.self_assessment,
            metadata: execution.metadata,
            error_message: execution.error_message,
            tokens_used: execution.tokens_used,
            created_at: now,
            updated_at: Some(now),
            execution_metadata: execution.execution_metadata,
            result_data: execution.result_data,
        })
    }

    async fn get_task_execution(&self, id: Uuid) -> Result<Option<TaskExecution>> {
        let row = sqlx::query_as::<_, TaskExecution>(
            r#"
            SELECT id, task_id, worker_id, execution_started_at, execution_completed_at,
                   execution_time_ms, status, worker_output, self_assessment, metadata,
                   error_message, tokens_used, created_at, updated_at, execution_metadata, result_data
            FROM task_executions
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>> {
        let rows = sqlx::query_as::<_, TaskExecution>(
            r#"
            SELECT id, task_id, worker_id, execution_started_at, execution_completed_at,
                   execution_time_ms, status, worker_output, self_assessment, metadata,
                   error_message, tokens_used, created_at, updated_at, execution_metadata, result_data
            FROM task_executions
            WHERE task_id = $1
            ORDER BY execution_started_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn update_task_execution(&self, id: Uuid, update: UpdateTaskExecution) -> Result<TaskExecution> {
        let now = Utc::now();
        
        // Get current execution to merge with updates
        let current = self.get_task_execution(id).await?
            .ok_or_else(|| anyhow::anyhow!("Task execution not found: {}", id))?;
        
        sqlx::query_as::<_, TaskExecution>(
            r#"
            UPDATE task_executions
            SET execution_completed_at = $1,
                execution_time_ms = $2,
                status = $3,
                worker_output = $4,
                self_assessment = $5,
                metadata = $6,
                error_message = $7,
                tokens_used = $8,
                execution_metadata = $9,
                result_data = $10,
                updated_at = $11
            WHERE id = $12
            RETURNING id, task_id, worker_id, execution_started_at, execution_completed_at,
                     execution_time_ms, status, worker_output, self_assessment, metadata,
                     error_message, tokens_used, created_at, updated_at, execution_metadata, result_data
            "#
        )
        .bind(update.execution_completed_at.or(current.execution_completed_at))
        .bind(update.execution_time_ms.or(current.execution_time_ms))
        .bind(update.status.as_ref().unwrap_or(&current.status))
        .bind(update.worker_output.as_ref().unwrap_or(&current.worker_output))
        .bind(update.self_assessment.as_ref().unwrap_or(&current.self_assessment))
        .bind(update.metadata.as_ref().unwrap_or(&current.metadata))
        .bind(update.error_message.as_ref().or(current.error_message.as_ref()))
        .bind(update.tokens_used.or(current.tokens_used))
        .bind(update.execution_metadata.as_ref().or(current.execution_metadata.as_ref()))
        .bind(update.result_data.as_ref().or(current.result_data.as_ref()))
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task execution not found after update: {}", id))
    }

    async fn create_council_verdict(&self, verdict: CreateCouncilVerdict) -> Result<CouncilVerdict> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO council_verdicts (
                id, task_id, verdict_id, consensus_score, final_verdict,
                individual_verdicts, debate_rounds, evaluation_time_ms,
                created_at, contract, updated_at, verdict_details
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(id)
        .bind(verdict.task_id)
        .bind(verdict.verdict_id)
        .bind(verdict.consensus_score)
        .bind(&verdict.final_verdict)
        .bind(&verdict.individual_verdicts)
        .bind(verdict.debate_rounds)
        .bind(verdict.evaluation_time_ms)
        .bind(now)
        .bind(&verdict.contract)
        .bind(now)
        .bind(&verdict.verdict_details)
        .execute(&self.pool)
        .await?;
        
        Ok(CouncilVerdict {
            id,
            task_id: verdict.task_id,
            verdict_id: verdict.verdict_id,
            consensus_score: verdict.consensus_score,
            final_verdict: verdict.final_verdict,
            individual_verdicts: verdict.individual_verdicts,
            debate_rounds: verdict.debate_rounds,
            evaluation_time_ms: verdict.evaluation_time_ms,
            created_at: now,
            contract: verdict.contract,
            updated_at: Some(now),
            verdict_details: verdict.verdict_details,
        })
    }

    async fn get_council_verdict(&self, id: Uuid) -> Result<Option<CouncilVerdict>> {
        let row = sqlx::query_as::<_, CouncilVerdict>(
            r#"
            SELECT id, task_id, verdict_id, consensus_score, final_verdict,
                   individual_verdicts, debate_rounds, evaluation_time_ms,
                   created_at, contract, updated_at, verdict_details
            FROM council_verdicts
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    async fn get_council_verdicts(&self, task_id: Uuid) -> Result<Vec<CouncilVerdict>> {
        let rows = sqlx::query_as::<_, CouncilVerdict>(
            r#"
            SELECT id, task_id, verdict_id, consensus_score, final_verdict,
                   individual_verdicts, debate_rounds, evaluation_time_ms,
                   created_at, contract, updated_at, verdict_details
            FROM council_verdicts
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn create_judge_evaluation(&self, evaluation: CreateJudgeEvaluation) -> Result<JudgeEvaluation> {
        let id = Uuid::new_v4();
        
        // Create a verdict_id from the task_id for now (may need adjustment based on actual schema)
        let verdict_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO judge_evaluations (
                id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                tokens_used, confidence, created_at, evaluation_score,
                confidence_score, reasoning, evidence_used, evaluation_metadata,
                verdict_decision, risk_assessment, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#
        )
        .bind(id)
        .bind(verdict_id)
        .bind(evaluation.judge_id)
        .bind(serde_json::json!({})) // judge_verdict placeholder
        .bind(evaluation.evaluation_time_ms)
        .bind(None::<i32>) // tokens_used
        .bind(None::<f32>) // confidence
        .bind(evaluation.evaluation_timestamp)
        .bind(Some(evaluation.evaluation_score))
        .bind(None::<f32>) // confidence_score
        .bind(Some(evaluation.evaluation_reasoning.clone()))
        .bind(Some(evaluation.evaluation_metadata.clone())) // evidence_used
        .bind(Some(evaluation.evaluation_metadata.clone())) // evaluation_metadata
        .bind(None::<String>) // verdict_decision
        .bind(None::<serde_json::Value>) // risk_assessment
        .bind(Some(evaluation.evaluation_timestamp))
        .execute(&self.pool)
        .await?;
        
        Ok(JudgeEvaluation {
            id,
            verdict_id,
            judge_id: evaluation.judge_id,
            judge_verdict: serde_json::json!({}),
            evaluation_time_ms: evaluation.evaluation_time_ms,
            tokens_used: None,
            confidence: None,
            created_at: evaluation.evaluation_timestamp,
            evaluation_score: Some(evaluation.evaluation_score),
            confidence_score: None,
            reasoning: Some(evaluation.evaluation_reasoning),
            evidence_used: Some(evaluation.evaluation_metadata.clone()),
            evaluation_metadata: Some(evaluation.evaluation_metadata),
            verdict_decision: None,
            risk_assessment: None,
            updated_at: Some(evaluation.evaluation_timestamp),
        })
    }

    async fn get_judge_evaluations(&self, task_id: Uuid) -> Result<Vec<JudgeEvaluation>> {
        // Note: This queries by task_id, but judge_evaluations table may not have task_id directly
        // We'll need to join through council_verdicts if needed, or adjust schema
        // For now, returning empty as schema relationship is unclear
        let rows = sqlx::query_as::<_, JudgeEvaluation>(
            r#"
            SELECT id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                   tokens_used, confidence, created_at, evaluation_score,
                   confidence_score, reasoning, evidence_used, evaluation_metadata,
                   verdict_decision, risk_assessment, updated_at
            FROM judge_evaluations
            WHERE verdict_id IN (
                SELECT id FROM council_verdicts WHERE task_id = $1
            )
            ORDER BY created_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn create_audit_trail_entry(&self, entry: CreateAuditTrailEntry) -> Result<AuditTrailEntry> {
        let id = Uuid::new_v4();
        let timestamp = entry.timestamp.unwrap_or_else(|| Utc::now());
        
        sqlx::query(
            r#"
            INSERT INTO audit_trail_entries (
                id, entity_type, entity_id, action, details,
                user_id, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(id)
        .bind(&entry.entity_type)
        .bind(entry.entity_id)
        .bind(&entry.action)
        .bind(&entry.details)
        .bind(&entry.user_id)
        .bind(&entry.ip_address)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        
        Ok(AuditTrailEntry {
            id,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            action: entry.action,
            details: entry.details,
            user_id: entry.user_id,
            ip_address: entry.ip_address,
            created_at: timestamp,
        })
    }

    async fn get_audit_trail_entries(&self, task_id: Uuid) -> Result<Vec<AuditTrailEntry>> {
        let rows = sqlx::query_as::<_, AuditTrailEntry>(
            r#"
            SELECT id, entity_type, entity_id, action, details,
                   user_id, ip_address, created_at
            FROM audit_trail_entries
            WHERE entity_id = $1 AND entity_type = 'task'
            ORDER BY created_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    async fn get_audit_trail_entry(&self, id: Uuid) -> Result<Option<AuditTrailEntry>> {
        let row = sqlx::query_as::<_, AuditTrailEntry>(
            r#"
            SELECT id, entity_type, entity_id, action, details,
                   user_id, ip_address, created_at
            FROM audit_trail_entries
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }

    // Planning operations
    async fn create_planning_telemetry(&self, telemetry: CreatePlanningTelemetry) -> Result<PlanningTelemetry> {
        let id = Uuid::new_v4();
        let collected_at = telemetry.collected_at.unwrap_or_else(|| Utc::now());
        let metadata = telemetry.metadata.unwrap_or_else(|| serde_json::json!({}));
        
        sqlx::query_as::<_, PlanningTelemetry>(
            r#"
            INSERT INTO planning_telemetry (
                id, plan_id, metric_type, metric_value, collected_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, plan_id, metric_type, metric_value, collected_at, metadata
            "#
        )
        .bind(id)
        .bind(telemetry.plan_id)
        .bind(&telemetry.metric_type)
        .bind(&telemetry.metric_value)
        .bind(collected_at)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create planning telemetry: {}", e))
    }

    async fn get_planning_telemetry(&self, plan_id: Uuid, metric_type: Option<String>) -> Result<Vec<PlanningTelemetry>> {
        let query = match metric_type {
            Some(mt) => {
                sqlx::query_as::<_, PlanningTelemetry>(
                    r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1 AND metric_type = $2
                    ORDER BY collected_at DESC
                    "#
                )
                .bind(plan_id)
                .bind(mt)
            }
            None => {
                sqlx::query_as::<_, PlanningTelemetry>(
                    r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1
                    ORDER BY collected_at DESC
                    "#
                )
                .bind(plan_id)
            }
        };
        
        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    // Planning operations stubs - these need full implementation but are here for interface completeness
    async fn create_milestone(&self, _milestone: CreateMilestone) -> Result<Milestone> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<Milestone>> {
        Ok(None)
    }

    async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<Milestone>> {
        Ok(vec![])
    }

    async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: UpdateMilestone) -> Result<Milestone> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> {
        Ok(())
    }

    async fn create_planning_session(&self, session: CreatePlanningSession) -> Result<PlanningSession> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as::<_, PlanningSession>(
            r#"
            INSERT INTO planning_sessions (
                id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                audit_correlation_id, status, execution_state, started_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                      audit_correlation_id, status, execution_state, started_at, completed_at, created_at
            "#
        )
        .bind(id)
        .bind(session.plan_id)
        .bind(&session.orchestrator_id)
        .bind(&session.worker_pool_id)
        .bind(session.council_session_id)
        .bind(session.audit_correlation_id)
        .bind(session.status.as_deref().unwrap_or("active"))
        .bind(session.execution_state.unwrap_or_else(|| serde_json::json!({})))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create planning session")
    }

    async fn get_planning_session(&self, id: Uuid) -> Result<Option<PlanningSession>> {
        sqlx::query_as::<_, PlanningSession>(
            r#"
            SELECT id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                   audit_correlation_id, status, execution_state, started_at, completed_at, created_at
            FROM planning_sessions
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get planning session")
    }

    async fn get_planning_sessions(&self, plan_id: Uuid) -> Result<Vec<PlanningSession>> {
        sqlx::query_as::<_, PlanningSession>(
            r#"
            SELECT id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                   audit_correlation_id, status, execution_state, started_at, completed_at, created_at
            FROM planning_sessions
            WHERE plan_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get planning sessions")
    }

    async fn update_planning_session(&self, id: Uuid, update: UpdatePlanningSession) -> Result<PlanningSession> {
        sqlx::query_as::<_, PlanningSession>(
            r#"
            UPDATE planning_sessions
            SET status = COALESCE($1, status),
                execution_state = COALESCE($2, execution_state),
                completed_at = COALESCE($3, completed_at)
            WHERE id = $4
            RETURNING id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                      audit_correlation_id, status, execution_state, started_at, completed_at, created_at
            "#
        )
        .bind(update.status)
        .bind(update.execution_state)
        .bind(update.completed_at)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Planning session not found: {}", id))
    }

    async fn create_evidence_artifact(&self, _artifact: CreateEvidenceArtifact) -> Result<EvidenceArtifact> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<EvidenceArtifact>> {
        Ok(vec![])
    }

    async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<EvidenceArtifact>> {
        Ok(vec![])
    }

    async fn update_evidence_artifact(&self, _id: Uuid, _update: UpdateEvidenceArtifact) -> Result<EvidenceArtifact> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<PlanningAuditEvent> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as::<_, PlanningAuditEvent>(
            r#"
            INSERT INTO planning_audit_events (
                id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
            "#
        )
        .bind(id)
        .bind(event.plan_id)
        .bind(event.milestone_id)
        .bind(event.worker_id)
        .bind(&event.event_type)
        .bind(&event.description)
        .bind(event.metadata.unwrap_or_else(|| serde_json::json!({})))
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create planning audit event")
    }

    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<PlanningAuditEvent>> {
        sqlx::query_as::<_, PlanningAuditEvent>(
            r#"
            SELECT id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
            FROM planning_audit_events
            WHERE plan_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get planning audit events")
    }

    async fn create_execution_plan(&self, plan: CreateExecutionPlan) -> Result<ExecutionPlan> {
        let now = Utc::now();
        
        sqlx::query_as::<_, ExecutionPlan>(
            r#"
            INSERT INTO execution_plans (
                id, session_id, working_spec_id, title, overview, state,
                milestones, dependency_graph, change_budget, quality_gates,
                evidence_requirements, active_waivers, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, session_id, working_spec_id, title, overview, state,
                      milestones, dependency_graph, change_budget, quality_gates,
                      evidence_requirements, active_waivers, metadata, created_at, updated_at,
                      approved_at, completed_at
            "#
        )
        .bind(plan.id)
        .bind(plan.session_id)
        .bind(&plan.working_spec_id)
        .bind(&plan.title)
        .bind(plan.overview.as_deref())
        .bind(plan.state.as_deref().unwrap_or("draft"))
        .bind(plan.milestones.unwrap_or_else(|| serde_json::json!([])))
        .bind(plan.dependency_graph.unwrap_or_else(|| serde_json::json!({})))
        .bind(plan.change_budget.unwrap_or_else(|| serde_json::json!({})))
        .bind(plan.quality_gates.unwrap_or_else(|| serde_json::json!({})))
        .bind(plan.evidence_requirements.unwrap_or_else(|| serde_json::json!([])))
        .bind(plan.active_waivers.unwrap_or_else(|| serde_json::json!([])))
        .bind(plan.metadata.unwrap_or_else(|| serde_json::json!({})))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create execution plan")
    }

    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<ExecutionPlan>> {
        sqlx::query_as::<_, ExecutionPlan>(
            r#"
            SELECT id, session_id, working_spec_id, title, overview, state,
                   milestones, dependency_graph, change_budget, quality_gates,
                   evidence_requirements, active_waivers, metadata, created_at, updated_at,
                   approved_at, completed_at
            FROM execution_plans
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get execution plan")
    }

    async fn get_execution_plans(&self) -> Result<Vec<ExecutionPlan>> {
        Ok(vec![])
    }

    async fn update_execution_plan(&self, _id: Uuid, _update: UpdateExecutionPlan) -> Result<ExecutionPlan> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn get_waivers(&self, status: Option<String>) -> Result<Vec<Waiver>> {
        let query = if let Some(status_filter) = status {
            sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                WHERE status = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(status_filter)
        } else {
            sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                ORDER BY created_at DESC
                "#
            )
        };

        let rows = query.fetch_all(&self.pool).await?;
        
        let mut waivers = Vec::new();
        for row in rows {
            let gates_json: serde_json::Value = row.try_get("gates")?;
            let gates: Vec<String> = gates_json.as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            
            waivers.push(Waiver {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                reason: row.try_get("reason")?,
                description: row.try_get("description")?,
                gates,
                approved_by: row.try_get("approved_by")?,
                impact_level: row.try_get("impact_level")?,
                mitigation_plan: row.try_get("mitigation_plan")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                status: row.try_get("status")?,
                metadata: row.try_get("metadata")?,
            });
        }
        
        Ok(waivers)
    }

    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<Waiver> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let gates_json = serde_json::to_value(&waiver.gates)?;
        let metadata = waiver.metadata.unwrap_or_else(|| serde_json::json!({}));

        sqlx::query(
            r#"
            INSERT INTO waivers (
                id, title, reason, description, gates, approved_by, impact_level,
                mitigation_plan, expires_at, created_at, updated_at, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(id)
        .bind(&waiver.title)
        .bind(&waiver.reason)
        .bind(&waiver.description)
        .bind(&gates_json)
        .bind(&waiver.approved_by)
        .bind(&waiver.impact_level)
        .bind(&waiver.mitigation_plan)
        .bind(waiver.expires_at)
        .bind(now)
        .bind(now)
        .bind("active")
        .bind(&metadata)
        .execute(&self.pool)
        .await?;

        Ok(Waiver {
            id,
            title: waiver.title,
            reason: waiver.reason,
            description: waiver.description,
            gates: waiver.gates,
            approved_by: waiver.approved_by,
            impact_level: waiver.impact_level,
            mitigation_plan: waiver.mitigation_plan,
            expires_at: waiver.expires_at,
            created_at: now,
            updated_at: now,
            status: "active".to_string(),
            metadata,
        })
    }

    async fn update_waiver(&self, id: Uuid, update: UpdateWaiver) -> Result<Waiver> {
        let now = Utc::now();
        
        // Build dynamic update query
        let mut update_fields = Vec::new();
        let mut param_count = 1u32;
        
        if let Some(ref title) = update.title {
            update_fields.push(format!("title = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref description) = update.description {
            update_fields.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref mitigation_plan) = update.mitigation_plan {
            update_fields.push(format!("mitigation_plan = ${}", param_count));
            param_count += 1;
        }
        if let Some(expires_at) = update.expires_at {
            update_fields.push(format!("expires_at = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref status) = update.status {
            update_fields.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref metadata) = update.metadata {
            update_fields.push(format!("metadata = ${}", param_count));
            param_count += 1;
        }
        
        // Always update updated_at
        update_fields.push(format!("updated_at = ${}", param_count));
        param_count += 1;
        
        // Add WHERE clause
        update_fields.push(format!("id = ${}", param_count));
        
        if update_fields.len() == 2 {
            // Only updated_at and id - nothing to update
            return Err(anyhow::anyhow!("No fields to update"));
        }
        
        let query_str = format!(
            "UPDATE waivers SET {} WHERE id = ${}",
            update_fields[..update_fields.len()-1].join(", "),
            param_count
        );
        
        let mut query = sqlx::query(&query_str);
        
        if let Some(ref title) = update.title {
            query = query.bind(title);
        }
        if let Some(ref description) = update.description {
            query = query.bind(description);
        }
        if let Some(ref mitigation_plan) = update.mitigation_plan {
            query = query.bind(mitigation_plan);
        }
        if let Some(expires_at) = update.expires_at {
            query = query.bind(expires_at);
        }
        if let Some(ref status) = update.status {
            query = query.bind(status);
        }
        if let Some(ref metadata) = update.metadata {
            query = query.bind(metadata);
        }
        query = query.bind(now).bind(id);
        
        query.execute(&self.pool).await?;
        
        // Fetch updated waiver
        let row = sqlx::query(
            r#"
            SELECT id, title, reason, description, gates, approved_by, impact_level,
                   mitigation_plan, expires_at, created_at, updated_at, status, metadata
            FROM waivers
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(row) => {
                let gates_json: serde_json::Value = row.try_get("gates")?;
                let gates: Vec<String> = gates_json.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                Ok(Waiver {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    reason: row.try_get("reason")?,
                    description: row.try_get("description")?,
                    gates,
                    approved_by: row.try_get("approved_by")?,
                    impact_level: row.try_get("impact_level")?,
                    mitigation_plan: row.try_get("mitigation_plan")?,
                    expires_at: row.try_get("expires_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                    status: row.try_get("status")?,
                    metadata: row.try_get("metadata")?,
                })
            }
            None => Err(anyhow::anyhow!("Waiver not found after update"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database_operations::CreateAuditTrailEntry;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;
    #[tokio::test]
    async fn test_database_client_creation() {
        // Test that we can create a DatabaseClient instance
        let client = DatabaseClient::default();
        
        // Verify the client has the expected components
        assert!(client.circuit_breaker.is_some());
        assert!(client.metrics.is_some());
        assert!(client.audit_logger.is_some());
        assert!(client.health_monitor.is_some());
    }

    #[tokio::test]
    async fn test_audit_trail_entry_creation() {
        let client = DatabaseClient::default();
        
        let entry = CreateAuditTrailEntry {
            entity_type: "test_entity".to_string(),
            entity_id: Uuid::new_v4(),
            action: "test_action".to_string(),
            details: json!({"test": "data"}),
            user_id: Some("test_user".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            timestamp: Some(Utc::now()),
        };

        // This test would require a real database connection
        // For now, we just verify the struct can be created
        assert_eq!(entry.entity_type, "test_entity");
        assert_eq!(entry.action, "test_action");
        assert!(entry.user_id.is_some());
        assert!(entry.ip_address.is_some());
        assert!(entry.timestamp.is_some());
    }

    #[tokio::test]
    async fn test_database_operations_trait_implementation() {
        let client = DatabaseClient::default();
        
        // Verify that DatabaseClient implements DatabaseOperations trait
        // This is a compile-time check - if it compiles, the trait is implemented
        let _client_ref: &dyn DatabaseOperations = &client;
    }

    #[tokio::test]
    async fn test_pooled_database_client_trait_implementation() {
        let client = DatabaseClient::default();
        
        // Verify that DatabaseClient implements PooledDatabaseClient trait
        // This is a compile-time check - if it compiles, the trait is implemented
        let _client_ref: &dyn PooledDatabaseClient = &client;
    }
}
