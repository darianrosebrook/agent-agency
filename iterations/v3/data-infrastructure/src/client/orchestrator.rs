//! Database Client Orchestrator
//!
//! Production-hardened database client with connection pooling,
//! circuit breaker pattern, monitoring, and resilience features.

use super::super::database_metrics::DatabaseMetrics;
use super::super::health::DatabaseHealthMonitor;
use crate::database_circuit_breaker::CircuitBreaker;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use super::super::database_audit::DatabaseAuditLogger;
use super::super::database_operations::{
    CreateApiKey, CreateAppSetting, CreateAuditTrailEntry, CreateCawsRule, CreateCawsSpecification,
    CreateCawsViolation, CreateCouncilSession, CreateCouncilVerdict, CreateEvidenceArtifact,
    CreateExecutionPlan, CreateIntegration, CreateJudge, CreateJudgeEvaluation, CreateMilestone,
    CreatePasswordResetToken, CreatePlanningAuditEvent, CreatePlanningSession,
    CreatePlanningTelemetry, CreateRuleTemplate, CreateSession, CreateTask, CreateTaskExecution,
    CreateTwoFactorAuth, CreateUser, CreateUserSetting, CreateWaiver, CreateWorker,
    DatabaseOperations, RuleEnforcementStatus, RuleHistory, RuleTemplate, UpdateApiKey,
    UpdateAppSetting, UpdateCawsRule, UpdateCawsSpecification, UpdateCawsViolation,
    UpdateCouncilSession, UpdateEvidenceArtifact, UpdateExecutionPlan, UpdateIntegration,
    UpdateJudge, UpdateMilestone, UpdatePlanningSession, UpdateRuleEnforcementStatus,
    UpdateSession, UpdateTask, UpdateTaskExecution, UpdateTwoFactorAuth, UpdateUser,
    UpdateUserSetting, UpdateWaiver, UpdateWorker,
};
use super::super::models::{
    ApiKey, AppSetting, AuditTrailEntry, CawsRule, CawsSpecification, CawsViolation,
    CouncilSession, CouncilVerdict, EvidenceArtifact, ExecutionPlan, Integration, Judge,
    JudgeEvaluation, Milestone, PasswordResetToken, PlanningAuditEvent, PlanningSession,
    PlanningTelemetry, Session, Task, TaskExecution, TwoFactorAuth, User, UserSetting, Waiver,
    Worker,
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
    ///
    /// Note: This implementation has limitations with dynamic parameter binding.
    /// For proper parameterized queries, consider using sqlx::query! macro at compile time
    /// or refactoring to use explicit types instead of trait objects.
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<sqlx::postgres::PgQueryResult> {
        // TODO: Fix dynamic parameter binding with trait objects
        //       sqlx doesn't support dynamic parameter binding with trait objects easily.
        //       Currently uses workaround for parameterless queries; parameterized queries return error.
        //
        // COMPLETION CHECKLIST:
        // [ ] Refactor to use sqlx::query! macro for compile-time query checking
        // [ ] OR refactor to use concrete types instead of trait objects
        // [ ] Support parameterized queries with proper type safety
        // [ ] Remove workaround and use proper sqlx API
        // [ ] Add unit tests with various query types
        // [ ] Add integration tests with real database queries
        //
        // ACCEPTANCE CRITERIA:
        // - Parameterized queries work with trait objects
        // - Type safety maintained with compile-time checking
        // - No workarounds or manual parameter handling
        //
        // DEPENDENCIES:
        // - sqlx query macro support (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours
        // PRIORITY: Medium
        // BLOCKING: No (workaround exists but limits functionality)
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (database integration)
        // - Change Budget: ~150 LOC

        if params.is_empty() {
            sqlx::query(query)
                .execute(&self.pool)
                .await
                .context("Failed to execute query")
        } else {
            // Use sqlx::query_scalar or query_as for parameterized queries
            // But for execute, we need to use query with bind
            // Parameterized queries with trait objects not supported - see TODO above
            Err(anyhow::anyhow!(
                "Parameterized queries with trait objects are not fully supported. \
                Consider using sqlx::query! macro for compile-time query checking, \
                or refactor to use concrete parameter types. \
                Query: {}, Parameters: {}",
                query.chars().take(100).collect::<String>(),
                params.len()
            ))
        }
    }

    /// Execute a query and return rows
    pub async fn query(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>> {
        sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute query")
    }

    /// Execute a parameterized query and return a single row
    ///
    /// Note: This method has limitations with trait object parameters.
    /// For proper parameterized queries, consider using sqlx::query! macro at compile time
    /// or refactoring to use explicit types instead of trait objects.
    pub async fn query_one_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Option<sqlx::postgres::PgRow>> {
        if params.is_empty() {
            sqlx::query(query)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to execute query")
        } else {
            // sqlx doesn't support binding trait objects directly
            // Return error indicating this limitation
            Err(anyhow::anyhow!(
                "Parameterized queries with trait objects are not fully supported. \
                Consider using sqlx::query! macro for compile-time query checking, \
                or refactor to use concrete parameter types. \
                Query: {}, Parameters: {}",
                query.chars().take(100).collect::<String>(),
                params.len()
            ))
        }
    }

    /// Execute a parameterized query and return rows
    ///
    /// Note: This method has limitations with trait object parameters.
    /// For proper parameterized queries, consider using sqlx::query! macro at compile time
    /// or refactoring to use explicit types instead of trait objects.
    pub async fn query_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        if params.is_empty() {
            sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .context("Failed to execute query")
        } else {
            // sqlx doesn't support binding trait objects directly
            // Return error indicating this limitation
            Err(anyhow::anyhow!(
                "Parameterized queries with trait objects are not fully supported. \
                Consider using sqlx::query! macro for compile-time query checking, \
                or refactor to use concrete parameter types. \
                Query: {}, Parameters: {}",
                query.chars().take(100).collect::<String>(),
                params.len()
            ))
        }
    }

    /// Execute a safe query (alias for execute with parameters)
    pub async fn execute_safe_query(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        self.execute(query, &[]).await
    }

    /// Execute a parameterized query (alias for execute)
    pub async fn execute_parameterized_query(
        &self,
        query: &str,
        params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)>,
    ) -> Result<sqlx::postgres::PgQueryResult> {
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
        // Parse JSON value into CreateAuditTrailEntry structure
        let entry: CreateAuditTrailEntry = serde_json::from_value(audit_entry)
            .context("Failed to parse audit entry JSON into CreateAuditTrailEntry")?;

        // Generate ID and timestamp
        let id = Uuid::new_v4();
        let created_at = entry.timestamp.unwrap_or_else(|| Utc::now());

        // Insert into audit_trail_entries table
        sqlx::query(
            r#"
            INSERT INTO audit_trail_entries (
                id, entity_type, entity_id, action, details,
                user_id, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(&entry.entity_type)
        .bind(entry.entity_id)
        .bind(&entry.action)
        .bind(&entry.details)
        .bind(&entry.user_id)
        .bind(&entry.ip_address)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .context("Failed to insert audit trail entry into database")?;

        tracing::debug!(
            audit_entry_id = %id,
            entity_type = %entry.entity_type,
            entity_id = %entry.entity_id,
            action = %entry.action,
            "Audit trail entry created successfully"
        );

        Ok(())
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new DatabaseClient with configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.database_url)
            .await
            .context("Failed to connect to database")?;

        let metrics = Arc::new(DatabaseMetrics::new());
        Ok(Self {
            pool,
            circuit_breaker: Some(Arc::new(CircuitBreaker::new())),
            metrics: Some(metrics.clone()),
            audit_logger: Some(Arc::new(DatabaseAuditLogger::new())),
            health_monitor: Some(Arc::new(DatabaseHealthMonitor::new(metrics))),
            connection_semaphore: Arc::new(Semaphore::new(
                config.max_connections.unwrap_or(100) as usize
            )),
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
            health_monitor: Some(Arc::new(DatabaseHealthMonitor::new(Arc::new(
                DatabaseMetrics::new(),
            )))),
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
            "#,
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
            "#,
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
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge> {
        let now = Utc::now();

        // Get current judge to merge with updates
        let current = self
            .get_judge(id)
            .await?
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
            "#,
        )
        .bind(update.name.as_ref().unwrap_or(&current.name))
        .bind(update.model_name.as_ref().unwrap_or(&current.model_name))
        .bind(update.endpoint.as_ref().unwrap_or(&current.endpoint))
        .bind(update.weight.unwrap_or(current.weight))
        .bind(update.timeout_ms.unwrap_or(current.timeout_ms))
        .bind(
            update
                .optimization_target
                .as_ref()
                .unwrap_or(&current.optimization_target),
        )
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
            "#,
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
            "#,
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
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker> {
        let now = Utc::now();

        // Get current worker to merge with updates
        let current = self
            .get_worker(id)
            .await?
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
            "#,
        )
        .bind(update.name.as_ref().unwrap_or(&current.name))
        .bind(update.worker_type.as_ref().unwrap_or(&current.worker_type))
        .bind(update.specialty.as_ref().or(current.specialty.as_ref()))
        .bind(update.model_name.as_ref().unwrap_or(&current.model_name))
        .bind(update.endpoint.as_ref().unwrap_or(&current.endpoint))
        .bind(
            update
                .capabilities
                .as_ref()
                .unwrap_or(&current.capabilities),
        )
        .bind(
            update
                .performance_history
                .as_ref()
                .unwrap_or(&current.performance_history),
        )
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
                context, caws_spec, status, assigned_worker_id, project_id, priority,
                deadline, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
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
        .bind(&task.project_id)
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
            project_id: task.project_id,
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
                   context, caws_spec, status, assigned_worker_id, project_id, priority,
                   deadline, metadata, created_at, updated_at, completed_at
            FROM tasks
            WHERE id = $1
            "#,
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
                   context, caws_spec, status, assigned_worker_id, project_id, priority,
                   deadline, metadata, created_at, updated_at, completed_at
            FROM tasks
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task> {
        let now = Utc::now();

        // Get current task to merge with updates
        let current = self
            .get_task(id)
            .await?
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
                project_id = $10,
                priority = $11,
                deadline = $12,
                metadata = $13,
                completed_at = $14,
                updated_at = $15
            WHERE id = $16
            RETURNING id, title, description, risk_tier, scope, acceptance_criteria,
                     context, caws_spec, status, assigned_worker_id, project_id, priority,
                     deadline, metadata, created_at, updated_at, completed_at
            "#,
        )
        .bind(update.title.as_ref().unwrap_or(&current.title))
        .bind(update.description.as_ref().unwrap_or(&current.description))
        .bind(update.risk_tier.as_ref().unwrap_or(&current.risk_tier))
        .bind(update.scope.as_ref().unwrap_or(&current.scope))
        .bind(
            update
                .acceptance_criteria
                .as_ref()
                .unwrap_or(&current.acceptance_criteria),
        )
        .bind(update.context.as_ref().unwrap_or(&current.context))
        .bind(update.caws_spec.as_ref().or(current.caws_spec.as_ref()))
        .bind(update.status.as_ref().unwrap_or(&current.status))
        .bind(update.assigned_worker_id.or(current.assigned_worker_id))
        .bind(update.project_id.or(current.project_id))
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
            "#,
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

    async fn update_task_execution(
        &self,
        id: Uuid,
        update: UpdateTaskExecution,
    ) -> Result<TaskExecution> {
        let now = Utc::now();

        // Get current execution to merge with updates
        let current = self
            .get_task_execution(id)
            .await?
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

    async fn create_council_verdict(
        &self,
        verdict: CreateCouncilVerdict,
    ) -> Result<CouncilVerdict> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO council_verdicts (
                id, task_id, verdict_id, consensus_score, final_verdict,
                individual_verdicts, debate_rounds, evaluation_time_ms,
                created_at, contract, updated_at, verdict_details
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
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
            "#,
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
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn create_council_session(
        &self,
        session: CreateCouncilSession,
    ) -> Result<CouncilSession> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, CouncilSession>(
            r#"
            INSERT INTO council_sessions (
                id, session_id, task_id, working_spec_id, review_context,
                status, selected_judges, contributions, progress,
                started_at, created_at, updated_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, session_id, task_id, working_spec_id, review_context,
                      status, selected_judges, contributions, aggregation_result,
                      final_decision, progress, started_at, completed_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(id)
        .bind(session.session_id)
        .bind(session.task_id)
        .bind(session.working_spec_id)
        .bind(session.review_context)
        .bind(session.status.unwrap_or_else(|| "initialized".to_string()))
        .bind(
            session
                .selected_judges
                .unwrap_or_else(|| serde_json::json!([])),
        )
        .bind(
            session
                .contributions
                .unwrap_or_else(|| serde_json::json!([])),
        )
        .bind(session.progress.unwrap_or(0.0))
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(session.metadata.unwrap_or_else(|| serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
        .context("Failed to create council session")
    }

    async fn get_council_session(&self, session_id: Uuid) -> Result<Option<CouncilSession>> {
        sqlx::query_as::<_, CouncilSession>(
            r#"
            SELECT id, session_id, task_id, working_spec_id, review_context,
                   status, selected_judges, contributions, aggregation_result,
                   final_decision, progress, started_at, completed_at,
                   created_at, updated_at, metadata
            FROM council_sessions
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get council session")
    }

    async fn get_council_session_by_task(&self, task_id: Uuid) -> Result<Option<CouncilSession>> {
        sqlx::query_as::<_, CouncilSession>(
            r#"
            SELECT id, session_id, task_id, working_spec_id, review_context,
                   status, selected_judges, contributions, aggregation_result,
                   final_decision, progress, started_at, completed_at,
                   created_at, updated_at, metadata
            FROM council_sessions
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get council session by task")
    }

    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSession,
    ) -> Result<CouncilSession> {
        sqlx::query_as::<_, CouncilSession>(
            r#"
            UPDATE council_sessions
            SET status = COALESCE($1, status),
                selected_judges = COALESCE($2, selected_judges),
                contributions = COALESCE($3, contributions),
                aggregation_result = COALESCE($4, aggregation_result),
                final_decision = COALESCE($5, final_decision),
                progress = COALESCE($6, progress),
                completed_at = COALESCE($7, completed_at),
                metadata = COALESCE($8, metadata),
                updated_at = NOW()
            WHERE session_id = $9
            RETURNING id, session_id, task_id, working_spec_id, review_context,
                      status, selected_judges, contributions, aggregation_result,
                      final_decision, progress, started_at, completed_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(update.status)
        .bind(update.selected_judges)
        .bind(update.contributions)
        .bind(update.aggregation_result)
        .bind(update.final_decision)
        .bind(update.progress)
        .bind(update.completed_at)
        .bind(update.metadata)
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Council session not found: {}", session_id))
    }

    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<JudgeEvaluation> {
        let id = Uuid::new_v4();

        // Retrieve verdict_id from council_verdicts table based on task_id
        // Query for the most recent verdict for this task (ordered by created_at DESC)
        let verdict_row = sqlx::query(
            r#"
            SELECT verdict_id
            FROM council_verdicts
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(evaluation.task_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query council_verdicts for verdict_id")?;

        let verdict_id = match verdict_row {
            Some(row) => row
                .try_get::<Uuid, &str>("verdict_id")
                .context("Failed to extract verdict_id from query result")?,
            None => {
                return Err(anyhow::anyhow!(
                    "No council verdict found for task_id: {}. \
                    A council verdict must exist before creating judge evaluations.",
                    evaluation.task_id
                ));
            }
        };

        sqlx::query(
            r#"
            INSERT INTO judge_evaluations (
                id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                tokens_used, confidence, created_at, evaluation_score,
                confidence_score, reasoning, evidence_used, evaluation_metadata,
                verdict_decision, risk_assessment, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
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
        // Query judge evaluations for a task by joining through council_verdicts
        // judge_evaluations.verdict_id references council_verdicts.verdict_id (not the primary key id)
        let rows = sqlx::query_as::<_, JudgeEvaluation>(
            r#"
            SELECT id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                   tokens_used, confidence, created_at, evaluation_score,
                   confidence_score, reasoning, evidence_used, evaluation_metadata,
                   verdict_decision, risk_assessment, updated_at
            FROM judge_evaluations
            WHERE verdict_id IN (
                SELECT verdict_id FROM council_verdicts WHERE task_id = $1
            )
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query judge evaluations for task")?;

        tracing::debug!(
            task_id = %task_id,
            evaluation_count = rows.len(),
            "Retrieved judge evaluations for task"
        );

        Ok(rows)
    }

    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<AuditTrailEntry> {
        let id = Uuid::new_v4();
        let timestamp = entry.timestamp.unwrap_or_else(|| Utc::now());

        sqlx::query(
            r#"
            INSERT INTO audit_trail_entries (
                id, entity_type, entity_id, action, details,
                user_id, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
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
            "#,
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
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    // Planning operations
    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetry,
    ) -> Result<PlanningTelemetry> {
        let id = Uuid::new_v4();
        let collected_at = telemetry.collected_at.unwrap_or_else(|| Utc::now());
        let metadata = telemetry.metadata.unwrap_or_else(|| serde_json::json!({}));

        sqlx::query_as::<_, PlanningTelemetry>(
            r#"
            INSERT INTO planning_telemetry (
                id, plan_id, metric_type, metric_value, collected_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, plan_id, metric_type, metric_value, collected_at, metadata
            "#,
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

    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<PlanningTelemetry>> {
        let query = match metric_type {
            Some(mt) => sqlx::query_as::<_, PlanningTelemetry>(
                r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1 AND metric_type = $2
                    ORDER BY collected_at DESC
                    "#,
            )
            .bind(plan_id)
            .bind(mt),
            None => sqlx::query_as::<_, PlanningTelemetry>(
                r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1
                    ORDER BY collected_at DESC
                    "#,
            )
            .bind(plan_id),
        };

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    // Milestone operations - full implementation
    async fn create_milestone(&self, milestone: CreateMilestone) -> Result<Milestone> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO milestones (
                id, plan_id, objective, scope, interfaces, tests, evidence_gate,
                rollback_plan, dependencies, state, assigned_worker_id, estimated_effort,
                priority, risk_tier, is_blocking, blocking_reason, metrics,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#
        )
        .bind(&milestone.id)
        .bind(milestone.plan_id)
        .bind(&milestone.objective)
        .bind(milestone.scope.unwrap_or_else(|| serde_json::json!({})))
        .bind(milestone.interfaces.unwrap_or_else(|| serde_json::json!([])))
        .bind(milestone.tests.unwrap_or_else(|| serde_json::json!([])))
        .bind(milestone.evidence_gate.unwrap_or_else(|| serde_json::json!({})))
        .bind(&milestone.rollback_plan)
        .bind(milestone.dependencies.unwrap_or_else(|| serde_json::json!([])))
        .bind(milestone.state.unwrap_or_else(|| "pending".to_string()))
        .bind(milestone.assigned_worker_id)
        .bind(milestone.estimated_effort)
        .bind(&milestone.priority)
        .bind(milestone.risk_tier)
        .bind(milestone.is_blocking)
        .bind(&milestone.blocking_reason)
        .bind(&milestone.metrics)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to create milestone")?;

        self.get_milestone(milestone.plan_id, milestone.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created milestone"))
    }

    async fn get_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
    ) -> Result<Option<Milestone>> {
        sqlx::query_as::<_, Milestone>(
            r#"
            SELECT id, plan_id, objective, scope, interfaces, tests, evidence_gate,
                   rollback_plan, dependencies, state, assigned_worker_id, estimated_effort,
                   priority, risk_tier, is_blocking, blocking_reason, metrics,
                   started_at, completed_at, created_at, updated_at
            FROM milestones
            WHERE plan_id = $1 AND id = $2
            "#,
        )
        .bind(plan_id)
        .bind(&milestone_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get milestone")
    }

    async fn get_milestones(&self, plan_id: Uuid) -> Result<Vec<Milestone>> {
        sqlx::query_as::<_, Milestone>(
            r#"
            SELECT id, plan_id, objective, scope, interfaces, tests, evidence_gate,
                   rollback_plan, dependencies, state, assigned_worker_id, estimated_effort,
                   priority, risk_tier, is_blocking, blocking_reason, metrics,
                   started_at, completed_at, created_at, updated_at
            FROM milestones
            WHERE plan_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get milestones")
    }

    async fn update_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
        update: UpdateMilestone,
    ) -> Result<Milestone> {
        let mut updates = Vec::new();
        let mut bind_index = 1;

        if let Some(ref _objective) = update.objective {
            updates.push(format!("objective = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _scope) = update.scope {
            updates.push(format!("scope = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _interfaces) = update.interfaces {
            updates.push(format!("interfaces = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _tests) = update.tests {
            updates.push(format!("tests = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _evidence_gate) = update.evidence_gate {
            updates.push(format!("evidence_gate = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _rollback_plan) = update.rollback_plan {
            updates.push(format!("rollback_plan = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _dependencies) = update.dependencies {
            updates.push(format!("dependencies = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _state) = update.state {
            updates.push(format!("state = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_assigned_worker_id) = update.assigned_worker_id {
            updates.push(format!("assigned_worker_id = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_estimated_effort) = update.estimated_effort {
            updates.push(format!("estimated_effort = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _priority) = update.priority {
            updates.push(format!("priority = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_risk_tier) = update.risk_tier {
            updates.push(format!("risk_tier = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_is_blocking) = update.is_blocking {
            updates.push(format!("is_blocking = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _blocking_reason) = update.blocking_reason {
            updates.push(format!("blocking_reason = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _metrics) = update.metrics {
            updates.push(format!("metrics = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_started_at) = update.started_at {
            updates.push(format!("started_at = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_completed_at) = update.completed_at {
            updates.push(format!("completed_at = ${}", bind_index));
            bind_index += 1;
        }

        if updates.is_empty() {
            let milestone_id_clone = milestone_id.clone();
            return self
                .get_milestone(plan_id, milestone_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Milestone not found: {} in plan {}",
                        milestone_id_clone,
                        plan_id
                    )
                });
        }

        updates.push(format!("updated_at = ${}", bind_index));
        bind_index += 1;

        let query = format!(
            "UPDATE milestones SET {} WHERE plan_id = ${} AND id = ${}",
            updates.join(", "),
            bind_index,
            bind_index + 1
        );

        let mut query_builder = sqlx::query(&query);
        if let Some(ref objective) = update.objective {
            query_builder = query_builder.bind(objective);
        }
        if let Some(ref scope) = update.scope {
            query_builder = query_builder.bind(scope);
        }
        if let Some(ref interfaces) = update.interfaces {
            query_builder = query_builder.bind(interfaces);
        }
        if let Some(ref tests) = update.tests {
            query_builder = query_builder.bind(tests);
        }
        if let Some(ref evidence_gate) = update.evidence_gate {
            query_builder = query_builder.bind(evidence_gate);
        }
        if let Some(ref rollback_plan) = update.rollback_plan {
            query_builder = query_builder.bind(rollback_plan);
        }
        if let Some(ref dependencies) = update.dependencies {
            query_builder = query_builder.bind(dependencies);
        }
        if let Some(ref state) = update.state {
            query_builder = query_builder.bind(state);
        }
        if let Some(assigned_worker_id) = update.assigned_worker_id {
            query_builder = query_builder.bind(assigned_worker_id);
        }
        if let Some(estimated_effort) = update.estimated_effort {
            query_builder = query_builder.bind(estimated_effort);
        }
        if let Some(ref priority) = update.priority {
            query_builder = query_builder.bind(priority);
        }
        if let Some(risk_tier) = update.risk_tier {
            query_builder = query_builder.bind(risk_tier);
        }
        if let Some(is_blocking) = update.is_blocking {
            query_builder = query_builder.bind(is_blocking);
        }
        if let Some(ref blocking_reason) = update.blocking_reason {
            query_builder = query_builder.bind(blocking_reason);
        }
        if let Some(ref metrics) = update.metrics {
            query_builder = query_builder.bind(metrics);
        }
        if let Some(started_at) = update.started_at {
            query_builder = query_builder.bind(started_at);
        }
        if let Some(completed_at) = update.completed_at {
            query_builder = query_builder.bind(completed_at);
        }
        query_builder = query_builder.bind(Utc::now());
        query_builder = query_builder.bind(plan_id);
        query_builder = query_builder.bind(&milestone_id);

        query_builder
            .execute(&self.pool)
            .await
            .context("Failed to update milestone")?;

        let milestone_id_clone = milestone_id.clone();
        self.get_milestone(plan_id, milestone_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Milestone not found after update: {} in plan {}",
                    milestone_id_clone,
                    plan_id
                )
            })
    }

    async fn delete_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<()> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM milestones
            WHERE plan_id = $1 AND id = $2
            "#,
        )
        .bind(plan_id)
        .bind(&milestone_id)
        .execute(&self.pool)
        .await
        .context("Failed to delete milestone")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!(
                "Milestone not found: {} in plan {}",
                milestone_id,
                plan_id
            ));
        }

        Ok(())
    }

    async fn create_planning_session(
        &self,
        session: CreatePlanningSession,
    ) -> Result<PlanningSession> {
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

    async fn update_planning_session(
        &self,
        id: Uuid,
        update: UpdatePlanningSession,
    ) -> Result<PlanningSession> {
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

    async fn create_evidence_artifact(
        &self,
        artifact: CreateEvidenceArtifact,
    ) -> Result<EvidenceArtifact> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO evidence_artifacts (
                id, milestone_id, plan_id, artifact_type, artifact_data,
                verified, collected_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(&artifact.milestone_id)
        .bind(artifact.plan_id)
        .bind(&artifact.artifact_type)
        .bind(&artifact.artifact_data)
        .bind(artifact.verified.unwrap_or(false))
        .bind(now)
        .bind(artifact.metadata.unwrap_or_else(|| serde_json::json!({})))
        .execute(&self.pool)
        .await
        .context("Failed to create evidence artifact")?;

        sqlx::query_as::<_, EvidenceArtifact>(
            r#"
            SELECT id, milestone_id, plan_id, artifact_type, artifact_data,
                   verified, collected_at, verified_at, metadata
            FROM evidence_artifacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to retrieve created evidence artifact")
    }

    async fn get_evidence_artifacts(&self, plan_id: Uuid) -> Result<Vec<EvidenceArtifact>> {
        sqlx::query_as::<_, EvidenceArtifact>(
            r#"
            SELECT id, milestone_id, plan_id, artifact_type, artifact_data,
                   verified, collected_at, verified_at, metadata
            FROM evidence_artifacts
            WHERE plan_id = $1
            ORDER BY collected_at DESC
            "#,
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get evidence artifacts")
    }

    async fn get_evidence_artifacts_for_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
    ) -> Result<Vec<EvidenceArtifact>> {
        sqlx::query_as::<_, EvidenceArtifact>(
            r#"
            SELECT id, milestone_id, plan_id, artifact_type, artifact_data,
                   verified, collected_at, verified_at, metadata
            FROM evidence_artifacts
            WHERE plan_id = $1 AND milestone_id = $2
            ORDER BY collected_at DESC
            "#,
        )
        .bind(plan_id)
        .bind(&milestone_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get evidence artifacts for milestone")
    }

    async fn update_evidence_artifact(
        &self,
        id: Uuid,
        update: UpdateEvidenceArtifact,
    ) -> Result<EvidenceArtifact> {
        let mut updates = Vec::new();
        let mut bind_index = 1;

        if let Some(ref _artifact_type) = update.artifact_type {
            updates.push(format!("artifact_type = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _artifact_data) = update.artifact_data {
            updates.push(format!("artifact_data = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_verified) = update.verified {
            updates.push(format!("verified = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(_verified_at) = update.verified_at {
            updates.push(format!("verified_at = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref _metadata) = update.metadata {
            updates.push(format!("metadata = ${}", bind_index));
            bind_index += 1;
        }

        if updates.is_empty() {
            return sqlx::query_as::<_, EvidenceArtifact>(
                r#"
                SELECT id, milestone_id, plan_id, artifact_type, artifact_data,
                       verified, collected_at, verified_at, metadata
                FROM evidence_artifacts
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get evidence artifact")?
            .ok_or_else(|| anyhow::anyhow!("Evidence artifact not found: {}", id));
        }

        let query = format!(
            "UPDATE evidence_artifacts SET {} WHERE id = ${}",
            updates.join(", "),
            bind_index
        );

        let mut query_builder = sqlx::query(&query);
        if let Some(ref artifact_type) = update.artifact_type {
            query_builder = query_builder.bind(artifact_type);
        }
        if let Some(ref artifact_data) = update.artifact_data {
            query_builder = query_builder.bind(artifact_data);
        }
        if let Some(verified) = update.verified {
            query_builder = query_builder.bind(verified);
        }
        if let Some(verified_at) = update.verified_at {
            query_builder = query_builder.bind(verified_at);
        }
        if let Some(ref metadata) = update.metadata {
            query_builder = query_builder.bind(metadata);
        }
        query_builder = query_builder.bind(id);

        query_builder
            .execute(&self.pool)
            .await
            .context("Failed to update evidence artifact")?;

        sqlx::query_as::<_, EvidenceArtifact>(
            r#"
            SELECT id, milestone_id, plan_id, artifact_type, artifact_data,
                   verified, collected_at, verified_at, metadata
            FROM evidence_artifacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get evidence artifact after update")
    }

    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEvent,
    ) -> Result<PlanningAuditEvent> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Verify schema before query to provide better error context
        let has_description: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'planning_audit_events'
                AND column_name = 'description'
                AND table_schema = 'public'
            )
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !has_description {
            return Err(anyhow::anyhow!(
                "CRITICAL: planning_audit_events table is missing 'description' column. \
                Please run migration 028 to add the column. \
                Event: plan_id={}, event_type={}",
                event.plan_id,
                event.event_type
            ));
        }

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
        .with_context(|| format!("Failed to create planning audit event: table 'planning_audit_events' may be missing 'description' column. Event: plan_id={}, event_type={}", event.plan_id, event.event_type))
    }

    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<PlanningAuditEvent>> {
        // Verify schema before query to provide better error context
        let has_description: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'planning_audit_events'
                AND column_name = 'description'
                AND table_schema = 'public'
            )
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !has_description {
            return Err(anyhow::anyhow!(
                "CRITICAL: planning_audit_events table is missing 'description' column. \
                Please run migration 028 to add the column. \
                Plan ID: {}",
                plan_id
            ));
        }

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
        .with_context(|| format!("Failed to get planning audit events: table 'planning_audit_events' may be missing 'description' column. Plan ID: {}", plan_id))
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
            "#,
        )
        .bind(plan.id)
        .bind(plan.session_id)
        .bind(&plan.working_spec_id)
        .bind(&plan.title)
        .bind(plan.overview.as_deref())
        .bind(plan.state.as_deref().unwrap_or("draft"))
        .bind(plan.milestones.unwrap_or_else(|| serde_json::json!([])))
        .bind(
            plan.dependency_graph
                .unwrap_or_else(|| serde_json::json!({})),
        )
        .bind(plan.change_budget.unwrap_or_else(|| serde_json::json!({})))
        .bind(plan.quality_gates.unwrap_or_else(|| serde_json::json!({})))
        .bind(
            plan.evidence_requirements
                .unwrap_or_else(|| serde_json::json!([])),
        )
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
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get execution plan")
    }

    async fn get_execution_plans(&self) -> Result<Vec<ExecutionPlan>> {
        sqlx::query_as::<_, ExecutionPlan>(
            r#"
            SELECT id, session_id, working_spec_id, title, overview, state,
                   milestones, dependency_graph, change_budget, quality_gates,
                   evidence_requirements, active_waivers, metadata, created_at, updated_at,
                   approved_at, completed_at
            FROM execution_plans
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get execution plans")
    }

    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlan,
    ) -> Result<ExecutionPlan> {
        let mut updates = Vec::new();
        let mut bind_index = 1;

        // Build dynamic UPDATE query based on provided fields
        if update.title.is_some() {
            updates.push(format!("title = ${}", bind_index));
            bind_index += 1;
        }
        if update.overview.is_some() {
            updates.push(format!("overview = ${}", bind_index));
            bind_index += 1;
        }
        if update.state.is_some() {
            updates.push(format!("state = ${}", bind_index));
            bind_index += 1;
        }
        if update.milestones.is_some() {
            updates.push(format!("milestones = ${}", bind_index));
            bind_index += 1;
        }
        if update.dependency_graph.is_some() {
            updates.push(format!("dependency_graph = ${}", bind_index));
            bind_index += 1;
        }
        if update.change_budget.is_some() {
            updates.push(format!("change_budget = ${}", bind_index));
            bind_index += 1;
        }
        if update.quality_gates.is_some() {
            updates.push(format!("quality_gates = ${}", bind_index));
            bind_index += 1;
        }
        if update.evidence_requirements.is_some() {
            updates.push(format!("evidence_requirements = ${}", bind_index));
            bind_index += 1;
        }
        if update.active_waivers.is_some() {
            updates.push(format!("active_waivers = ${}", bind_index));
            bind_index += 1;
        }
        if update.metadata.is_some() {
            updates.push(format!("metadata = ${}", bind_index));
            bind_index += 1;
        }
        if update.approved_at.is_some() {
            updates.push(format!("approved_at = ${}", bind_index));
            bind_index += 1;
        }
        if update.completed_at.is_some() {
            updates.push(format!("completed_at = ${}", bind_index));
            bind_index += 1;
        }

        // Always update updated_at timestamp
        updates.push(format!("updated_at = ${}", bind_index));
        bind_index += 1;

        if updates.is_empty() {
            // No fields to update, just return the existing plan
            return self
                .get_execution_plan(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Execution plan not found: {}", id));
        }

        // Build SQL query
        let sql = format!(
            "UPDATE execution_plans SET {} WHERE id = ${}",
            updates.join(", "),
            bind_index
        );

        // Execute update with bound parameters
        let mut query = sqlx::query(&sql);

        if let Some(ref title) = update.title {
            query = query.bind(title);
        }
        if let Some(ref overview) = update.overview {
            query = query.bind(overview);
        }
        if let Some(ref state) = update.state {
            query = query.bind(state);
        }
        if let Some(ref milestones) = update.milestones {
            query = query.bind(milestones);
        }
        if let Some(ref dependency_graph) = update.dependency_graph {
            query = query.bind(dependency_graph);
        }
        if let Some(ref change_budget) = update.change_budget {
            query = query.bind(change_budget);
        }
        if let Some(ref quality_gates) = update.quality_gates {
            query = query.bind(quality_gates);
        }
        if let Some(ref evidence_requirements) = update.evidence_requirements {
            query = query.bind(evidence_requirements);
        }
        if let Some(ref active_waivers) = update.active_waivers {
            query = query.bind(active_waivers);
        }
        if let Some(ref metadata) = update.metadata {
            query = query.bind(metadata);
        }
        if let Some(ref approved_at) = update.approved_at {
            query = query.bind(approved_at);
        }
        if let Some(ref completed_at) = update.completed_at {
            query = query.bind(completed_at);
        }

        // Bind updated_at timestamp
        query = query.bind(Utc::now());

        // Bind plan id
        query = query.bind(id);

        // Execute update
        query
            .execute(&self.pool)
            .await
            .context("Failed to update execution plan")?;

        // Return updated plan
        self.get_execution_plan(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Execution plan not found after update: {}", id))
    }

    async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
        // Check if plan exists first
        let plan = self.get_execution_plan(id).await?;
        if plan.is_none() {
            return Err(anyhow::anyhow!("Execution plan not found: {}", id));
        }

        // Delete the execution plan (CASCADE will handle related milestones)
        sqlx::query("DELETE FROM execution_plans WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete execution plan")?;

        Ok(())
    }

    async fn get_waivers(&self, status: Option<String>) -> Result<Vec<Waiver>> {
        // Verify schema before query to provide better error context
        let has_description: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'waivers'
                AND column_name = 'description'
                AND table_schema = 'public'
            )
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !has_description {
            return Err(anyhow::anyhow!(
                "CRITICAL: waivers table is missing 'description' column. \
                Please run migration to add the column."
            ));
        }

        let query = if let Some(status_filter) = status {
            sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                WHERE status = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status_filter)
        } else {
            sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                ORDER BY created_at DESC
                "#,
            )
        };

        let rows = query.fetch_all(&self.pool).await.with_context(|| {
            "Failed to query waivers: table 'waivers' may be missing 'description' column"
        })?;

        let mut waivers = Vec::new();
        for row in rows {
            let gates_json: serde_json::Value = row.try_get("gates")?;
            let gates: Vec<String> = gates_json
                .as_array()
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
            "#,
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

        if let Some(ref _title) = update.title {
            update_fields.push(format!("title = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _description) = update.description {
            update_fields.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _mitigation_plan) = update.mitigation_plan {
            update_fields.push(format!("mitigation_plan = ${}", param_count));
            param_count += 1;
        }
        if let Some(_expires_at) = update.expires_at {
            update_fields.push(format!("expires_at = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _status) = update.status {
            update_fields.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _metadata) = update.metadata {
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
            update_fields[..update_fields.len() - 1].join(", "),
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
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let gates_json: serde_json::Value = row.try_get("gates")?;
                let gates: Vec<String> = gates_json
                    .as_array()
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
            None => Err(anyhow::anyhow!("Waiver not found after update")),
        }
    }

    // User operations
    async fn create_user(&self, user: CreateUser) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, password_hash, name, roles,
                is_active, failed_attempts, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.roles)
        .bind(true)
        .bind(0)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(User {
            id,
            username: user.username,
            password_hash: user.password_hash,
            name: user.name,
            roles: user.roles,
            is_active: true,
            failed_attempts: 0,
            locked_until: None,
            last_login: None,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, name, roles,
                   is_active, failed_attempts, locked_until, last_login,
                   created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, name, roles,
                   is_active, failed_attempts, locked_until, last_login,
                   created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update_user(&self, id: Uuid, update: UpdateUser) -> Result<User> {
        let now = Utc::now();

        // Get current user to merge with updates
        let _current = self
            .get_user(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", id))?;

        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = COALESCE($1, email),
                username = COALESCE($2, username),
                password_hash = COALESCE($3, password_hash),
                name = COALESCE($4, name),
                roles = COALESCE($5, roles),
                is_active = COALESCE($6, is_active),
                failed_attempts = COALESCE($7, failed_attempts),
                locked_until = COALESCE($8, locked_until),
                last_login = COALESCE($9, last_login),
                updated_at = $10
            WHERE id = $11
            RETURNING id, username, password_hash, name, roles,
                     is_active, failed_attempts, locked_until, last_login,
                     created_at, updated_at
            "#,
        )
        .bind(&update.username)
        .bind(&update.password_hash)
        .bind(&update.name)
        .bind(&update.roles)
        .bind(update.is_active)
        .bind(update.failed_attempts)
        .bind(&update.locked_until)
        .bind(&update.last_login)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found after update: {}", id))
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Session operations
    async fn create_session(&self, session: CreateSession) -> Result<Session> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, user_id, token_hash, refresh_token_hash, expires_at,
                refresh_expires_at, ip_address, user_agent, is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(id)
        .bind(session.user_id)
        .bind(&session.token_hash)
        .bind(&session.refresh_token_hash)
        .bind(session.expires_at)
        .bind(&session.refresh_expires_at)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Session {
            id,
            user_id: session.user_id,
            token_hash: session.token_hash,
            refresh_token_hash: session.refresh_token_hash,
            expires_at: session.expires_at,
            refresh_expires_at: session.refresh_expires_at,
            ip_address: session.ip_address,
            user_agent: session.user_agent,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_session(&self, id: Uuid) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, token_hash, refresh_token_hash, expires_at,
                   refresh_expires_at, ip_address, user_agent, is_active,
                   created_at, updated_at
            FROM sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_session_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, token_hash, refresh_token_hash, expires_at,
                   refresh_expires_at, ip_address, user_agent, is_active,
                   created_at, updated_at
            FROM sessions
            WHERE token_hash = $1 AND is_active = true
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_session_by_refresh_token_hash(&self, refresh_token_hash: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, token_hash, refresh_token_hash, expires_at,
                   refresh_expires_at, ip_address, user_agent, is_active,
                   created_at, updated_at
            FROM sessions
            WHERE refresh_token_hash = $1 AND is_active = true
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let rows = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, token_hash, refresh_token_hash, expires_at,
                   refresh_expires_at, ip_address, user_agent, is_active,
                   created_at, updated_at
            FROM sessions
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn update_session(&self, id: Uuid, update: UpdateSession) -> Result<Session> {
        let now = Utc::now();

        // Get current session to merge with updates
        let _current = self
            .get_session(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", id))?;

        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET token_hash = COALESCE($1, token_hash),
                refresh_token_hash = COALESCE($2, refresh_token_hash),
                expires_at = COALESCE($3, expires_at),
                refresh_expires_at = COALESCE($4, refresh_expires_at),
                is_active = COALESCE($5, is_active),
                updated_at = $6
            WHERE id = $7
            RETURNING id, user_id, token_hash, refresh_token_hash, expires_at,
                     refresh_expires_at, ip_address, user_agent, is_active,
                     created_at, updated_at
            "#,
        )
        .bind(&update.token_hash)
        .bind(&update.refresh_token_hash)
        .bind(&update.expires_at)
        .bind(&update.refresh_expires_at)
        .bind(update.is_active)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found after update: {}", id))
    }

    async fn delete_session(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_user_sessions(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE sessions SET is_active = false WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let result = sqlx::query(
            "UPDATE sessions SET is_active = false WHERE expires_at < NOW() AND is_active = true",
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    // Password reset token operations
    async fn create_password_reset_token(
        &self,
        token: CreatePasswordResetToken,
    ) -> Result<PasswordResetToken> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (
                id, user_id, token_hash, expires_at, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(token.user_id)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(&token.ip_address)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PasswordResetToken {
            id,
            user_id: token.user_id,
            token_hash: token.token_hash,
            expires_at: token.expires_at,
            used_at: None,
            ip_address: token.ip_address,
            created_at: now,
        })
    }

    async fn get_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>> {
        let row = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT id, user_id, token_hash, expires_at, used_at, ip_address, created_at
            FROM password_reset_tokens
            WHERE token_hash = $1 AND expires_at > NOW() AND used_at IS NULL
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn mark_password_reset_token_used(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn cleanup_expired_password_reset_tokens(&self) -> Result<usize> {
        let result = sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    // User settings operations
    async fn create_user_setting(&self, setting: CreateUserSetting) -> Result<UserSetting> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO user_settings (
                id, user_id, setting_key, setting_value, setting_type, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, setting_key)
            DO UPDATE SET
                setting_value = EXCLUDED.setting_value,
                setting_type = EXCLUDED.setting_type,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(id)
        .bind(setting.user_id)
        .bind(&setting.setting_key)
        .bind(&setting.setting_value)
        .bind(&setting.setting_type)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_user_setting(setting.user_id, &setting.setting_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created user setting"))
    }

    async fn get_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
    ) -> Result<Option<UserSetting>> {
        sqlx::query_as::<_, UserSetting>(
            r#"
            SELECT id, user_id, setting_key, setting_value, setting_type, created_at, updated_at
            FROM user_settings
            WHERE user_id = $1 AND setting_key = $2
            "#,
        )
        .bind(user_id)
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query user setting")
    }

    async fn get_user_settings(
        &self,
        user_id: Uuid,
        setting_type: Option<&str>,
    ) -> Result<Vec<UserSetting>> {
        let query = if let Some(st) = setting_type {
            sqlx::query_as::<_, UserSetting>(
                r#"
                SELECT id, user_id, setting_key, setting_value, setting_type, created_at, updated_at
                FROM user_settings
                WHERE user_id = $1 AND setting_type = $2
                ORDER BY setting_key
                "#,
            )
            .bind(user_id)
            .bind(st)
        } else {
            sqlx::query_as::<_, UserSetting>(
                r#"
                SELECT id, user_id, setting_key, setting_value, setting_type, created_at, updated_at
                FROM user_settings
                WHERE user_id = $1
                ORDER BY setting_key
                "#,
            )
            .bind(user_id)
        };

        query
            .fetch_all(&self.pool)
            .await
            .context("Failed to query user settings")
    }

    async fn update_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
        update: UpdateUserSetting,
    ) -> Result<UserSetting> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE user_settings
            SET setting_value = COALESCE($1, setting_value),
                setting_type = COALESCE($2, setting_type),
                updated_at = $3
            WHERE user_id = $4 AND setting_key = $5
            "#,
        )
        .bind(&update.setting_value)
        .bind(&update.setting_type)
        .bind(now)
        .bind(user_id)
        .bind(setting_key)
        .execute(&self.pool)
        .await?;

        self.get_user_setting(user_id, setting_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User setting not found after update"))
    }

    async fn delete_user_setting(&self, user_id: Uuid, setting_key: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_settings WHERE user_id = $1 AND setting_key = $2")
            .bind(user_id)
            .bind(setting_key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // App settings operations
    async fn create_app_setting(&self, setting: CreateAppSetting) -> Result<AppSetting> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO app_settings (
                id, setting_key, setting_value, setting_type, description, is_public, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(id)
        .bind(&setting.setting_key)
        .bind(&setting.setting_value)
        .bind(&setting.setting_type)
        .bind(&setting.description)
        .bind(setting.is_public)
        .bind(&setting.created_by)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_app_setting(&setting.setting_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created app setting"))
    }

    async fn get_app_setting(&self, setting_key: &str) -> Result<Option<AppSetting>> {
        sqlx::query_as::<_, AppSetting>(
            r#"
            SELECT id, setting_key, setting_value, setting_type, description, is_public, created_by, created_at, updated_at, updated_by
            FROM app_settings
            WHERE setting_key = $1
            "#
        )
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query app setting")
    }

    async fn get_app_settings(
        &self,
        setting_type: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Vec<AppSetting>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, setting_key, setting_value, setting_type, description, is_public, created_by, created_at, updated_at, updated_by
            FROM app_settings
            WHERE 1=1
            "#,
        );

        if let Some(st) = setting_type {
            query_builder.push(" AND setting_type = ");
            query_builder.push_bind(st);
        }

        if let Some(ip) = is_public {
            query_builder.push(" AND is_public = ");
            query_builder.push_bind(ip);
        }

        query_builder.push(" ORDER BY setting_key");

        let query = query_builder.build_query_as::<AppSetting>();
        query
            .fetch_all(&self.pool)
            .await
            .context("Failed to query app settings")
    }

    async fn update_app_setting(
        &self,
        setting_key: &str,
        update: UpdateAppSetting,
    ) -> Result<AppSetting> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE app_settings
            SET setting_value = COALESCE($1, setting_value),
                setting_type = COALESCE($2, setting_type),
                description = COALESCE($3, description),
                is_public = COALESCE($4, is_public),
                updated_by = $5,
                updated_at = $6
            WHERE setting_key = $7
            "#,
        )
        .bind(&update.setting_value)
        .bind(&update.setting_type)
        .bind(&update.description)
        .bind(update.is_public)
        .bind(&update.updated_by)
        .bind(now)
        .bind(setting_key)
        .execute(&self.pool)
        .await?;

        self.get_app_setting(setting_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("App setting not found after update"))
    }

    async fn delete_app_setting(&self, setting_key: &str) -> Result<()> {
        sqlx::query("DELETE FROM app_settings WHERE setting_key = $1")
            .bind(setting_key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Integration operations
    async fn create_integration(&self, integration: CreateIntegration) -> Result<Integration> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO integrations (
                id, name, integration_type, provider, configuration, credentials, is_active, is_enabled, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(id)
        .bind(&integration.name)
        .bind(&integration.integration_type)
        .bind(&integration.provider)
        .bind(&integration.configuration)
        .bind(&integration.credentials)
        .bind(integration.is_active)
        .bind(integration.is_enabled)
        .bind(&integration.created_by)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_integration(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created integration"))
    }

    async fn get_integration(&self, id: Uuid) -> Result<Option<Integration>> {
        sqlx::query_as::<_, Integration>(
            r#"
            SELECT id, name, integration_type, provider, configuration, credentials, is_active, is_enabled,
                   last_sync_at, sync_status, sync_error, created_by, created_at, updated_at, updated_by
            FROM integrations
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query integration")
    }

    async fn get_integrations(
        &self,
        provider: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<Integration>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, name, integration_type, provider, configuration, credentials, is_active, is_enabled,
                   last_sync_at, sync_status, sync_error, created_by, created_at, updated_at, updated_by
            FROM integrations
            WHERE 1=1
            "#,
        );

        if let Some(p) = provider {
            query_builder.push(" AND provider = ");
            query_builder.push_bind(p);
        }

        if let Some(ia) = is_active {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(ia);
        }

        query_builder.push(" ORDER BY name");

        let query = query_builder.build_query_as::<Integration>();
        query
            .fetch_all(&self.pool)
            .await
            .context("Failed to query integrations")
    }

    async fn update_integration(&self, id: Uuid, update: UpdateIntegration) -> Result<Integration> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE integrations
            SET name = COALESCE($1, name),
                configuration = COALESCE($2, configuration),
                credentials = COALESCE($3, credentials),
                is_active = COALESCE($4, is_active),
                is_enabled = COALESCE($5, is_enabled),
                updated_by = $6,
                updated_at = $7
            WHERE id = $8
            "#,
        )
        .bind(&update.name)
        .bind(&update.configuration)
        .bind(&update.credentials)
        .bind(update.is_active)
        .bind(update.is_enabled)
        .bind(&update.updated_by)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_integration(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Integration not found after update"))
    }

    async fn delete_integration(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM integrations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // API key operations
    async fn create_api_key(&self, api_key: CreateApiKey) -> Result<ApiKey> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO api_keys (
                id, user_id, key_name, key_hash, key_prefix, scopes, rate_limit_per_minute,
                rate_limit_per_hour, rate_limit_per_day, expires_at, is_active, is_revoked,
                created_at, updated_at, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, false, $11, $12, $13)
            "#,
        )
        .bind(id)
        .bind(api_key.user_id)
        .bind(&api_key.key_name)
        .bind(&api_key.key_hash)
        .bind(&api_key.key_prefix)
        .bind(&api_key.scopes)
        .bind(api_key.rate_limit_per_minute)
        .bind(api_key.rate_limit_per_hour)
        .bind(api_key.rate_limit_per_day)
        .bind(&api_key.expires_at)
        .bind(now)
        .bind(now)
        .bind(&api_key.created_by)
        .execute(&self.pool)
        .await?;

        self.get_api_key(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created API key"))
    }

    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, key_name, key_hash, key_prefix, scopes, rate_limit_per_minute,
                   rate_limit_per_hour, rate_limit_per_day, last_used_at, expires_at, is_active,
                   is_revoked, revoked_at, revoked_reason, created_at, updated_at, created_by
            FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query API key")
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, key_name, key_hash, key_prefix, scopes, rate_limit_per_minute,
                   rate_limit_per_hour, rate_limit_per_day, last_used_at, expires_at, is_active,
                   is_revoked, revoked_at, revoked_reason, created_at, updated_at, created_by
            FROM api_keys
            WHERE key_hash = $1 AND is_active = true AND is_revoked = false
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query API key by hash")
    }

    async fn get_user_api_keys(
        &self,
        user_id: Uuid,
        is_active: Option<bool>,
    ) -> Result<Vec<ApiKey>> {
        let query = if let Some(ia) = is_active {
            sqlx::query_as::<_, ApiKey>(
                r#"
                SELECT id, user_id, key_name, key_hash, key_prefix, scopes, rate_limit_per_minute,
                       rate_limit_per_hour, rate_limit_per_day, last_used_at, expires_at, is_active,
                       is_revoked, revoked_at, revoked_reason, created_at, updated_at, created_by
                FROM api_keys
                WHERE user_id = $1 AND is_active = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(user_id)
            .bind(ia)
        } else {
            sqlx::query_as::<_, ApiKey>(
                r#"
                SELECT id, user_id, key_name, key_hash, key_prefix, scopes, rate_limit_per_minute,
                       rate_limit_per_hour, rate_limit_per_day, last_used_at, expires_at, is_active,
                       is_revoked, revoked_at, revoked_reason, created_at, updated_at, created_by
                FROM api_keys
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(user_id)
        };

        query
            .fetch_all(&self.pool)
            .await
            .context("Failed to query user API keys")
    }

    async fn update_api_key(&self, id: Uuid, update: UpdateApiKey) -> Result<ApiKey> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE api_keys
            SET key_name = COALESCE($1, key_name),
                scopes = COALESCE($2, scopes),
                rate_limit_per_minute = COALESCE($3, rate_limit_per_minute),
                rate_limit_per_hour = COALESCE($4, rate_limit_per_hour),
                rate_limit_per_day = COALESCE($5, rate_limit_per_day),
                expires_at = COALESCE($6, expires_at),
                is_active = COALESCE($7, is_active),
                updated_at = $8
            WHERE id = $9
            "#,
        )
        .bind(&update.key_name)
        .bind(&update.scopes)
        .bind(update.rate_limit_per_minute)
        .bind(update.rate_limit_per_hour)
        .bind(update.rate_limit_per_day)
        .bind(&update.expires_at)
        .bind(update.is_active)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_api_key(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("API key not found after update"))
    }

    async fn revoke_api_key(&self, id: Uuid, reason: Option<String>) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE api_keys
            SET is_revoked = true,
                is_active = false,
                revoked_at = $1,
                revoked_reason = $2,
                updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(now)
        .bind(&reason)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_api_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Two-factor authentication operations
    async fn create_two_factor_auth(&self, two_fa: CreateTwoFactorAuth) -> Result<TwoFactorAuth> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO two_factor_auth (
                id, user_id, method, secret_encrypted, backup_codes, is_enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, method)
            DO UPDATE SET
                secret_encrypted = EXCLUDED.secret_encrypted,
                backup_codes = EXCLUDED.backup_codes,
                is_enabled = EXCLUDED.is_enabled,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(id)
        .bind(two_fa.user_id)
        .bind(&two_fa.method)
        .bind(&two_fa.secret_encrypted)
        .bind(&two_fa.backup_codes)
        .bind(two_fa.is_enabled)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_two_factor_auth(two_fa.user_id, Some(&two_fa.method))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created two-factor auth"))
    }

    async fn get_two_factor_auth(
        &self,
        user_id: Uuid,
        method: Option<&str>,
    ) -> Result<Option<TwoFactorAuth>> {
        let query = if let Some(m) = method {
            sqlx::query_as::<_, TwoFactorAuth>(
                r#"
                SELECT id, user_id, method, secret_encrypted, backup_codes, is_enabled, last_used_at, created_at, updated_at
                FROM two_factor_auth
                WHERE user_id = $1 AND method = $2
                "#
            )
            .bind(user_id)
            .bind(m)
        } else {
            sqlx::query_as::<_, TwoFactorAuth>(
                r#"
                SELECT id, user_id, method, secret_encrypted, backup_codes, is_enabled, last_used_at, created_at, updated_at
                FROM two_factor_auth
                WHERE user_id = $1
                ORDER BY method
                LIMIT 1
                "#
            )
            .bind(user_id)
        };

        query
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query two-factor auth")
    }

    async fn update_two_factor_auth(
        &self,
        user_id: Uuid,
        method: &str,
        update: UpdateTwoFactorAuth,
    ) -> Result<TwoFactorAuth> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE two_factor_auth
            SET secret_encrypted = COALESCE($1, secret_encrypted),
                backup_codes = COALESCE($2, backup_codes),
                is_enabled = COALESCE($3, is_enabled),
                updated_at = $4
            WHERE user_id = $5 AND method = $6
            "#,
        )
        .bind(&update.secret_encrypted)
        .bind(&update.backup_codes)
        .bind(update.is_enabled)
        .bind(now)
        .bind(user_id)
        .bind(method)
        .execute(&self.pool)
        .await?;

        self.get_two_factor_auth(user_id, Some(method))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Two-factor auth not found after update"))
    }

    async fn delete_two_factor_auth(&self, user_id: Uuid, method: &str) -> Result<()> {
        sqlx::query("DELETE FROM two_factor_auth WHERE user_id = $1 AND method = $2")
            .bind(user_id)
            .bind(method)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ============================================================================
    // CAWS Rules Operations
    // ============================================================================

    async fn create_caws_rule(&self, rule: CreateCawsRule) -> Result<CawsRule> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO caws_rules (
                id, name, description, rule_type, severity, file_patterns,
                config, constitutional_reference, is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.rule_type)
        .bind(&rule.severity)
        .bind(&rule.file_patterns)
        .bind(&rule.config)
        .bind(&rule.constitutional_reference)
        .bind(rule.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CawsRule {
            id: rule.id,
            name: rule.name,
            description: rule.description,
            rule_type: rule.rule_type,
            severity: rule.severity,
            file_patterns: rule.file_patterns,
            config: rule.config,
            constitutional_reference: rule.constitutional_reference,
            is_active: rule.is_active,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_caws_rule(&self, id: &str) -> Result<Option<CawsRule>> {
        let row = sqlx::query_as::<_, CawsRule>("SELECT * FROM caws_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn get_caws_rules(
        &self,
        rule_type: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<CawsRule>> {
        let rules = match (rule_type, is_active) {
            (Some(rt), Some(active)) => {
                sqlx::query_as::<_, CawsRule>(
                    "SELECT * FROM caws_rules WHERE rule_type = $1 AND is_active = $2 ORDER BY created_at DESC"
                )
                .bind(rt)
                .bind(active)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(rt), None) => {
                sqlx::query_as::<_, CawsRule>(
                    "SELECT * FROM caws_rules WHERE rule_type = $1 ORDER BY created_at DESC"
                )
                .bind(rt)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(active)) => {
                sqlx::query_as::<_, CawsRule>(
                    "SELECT * FROM caws_rules WHERE is_active = $1 ORDER BY created_at DESC"
                )
                .bind(active)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, CawsRule>(
                    "SELECT * FROM caws_rules ORDER BY created_at DESC"
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rules)
    }

    async fn update_caws_rule(&self, id: &str, update: UpdateCawsRule) -> Result<CawsRule> {
        // Get old rule for history
        let old_rule = self.get_caws_rule(id).await?;

        // Build update query dynamically
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if update.name.is_some() {
            set_clauses.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if update.description.is_some() {
            set_clauses.push(format!("description = ${}", bind_count));
            bind_count += 1;
        }
        if update.rule_type.is_some() {
            set_clauses.push(format!("rule_type = ${}", bind_count));
            bind_count += 1;
        }
        if update.severity.is_some() {
            set_clauses.push(format!("severity = ${}", bind_count));
            bind_count += 1;
        }
        if update.file_patterns.is_some() {
            set_clauses.push(format!("file_patterns = ${}", bind_count));
            bind_count += 1;
        }
        if update.config.is_some() {
            set_clauses.push(format!("config = ${}", bind_count));
            bind_count += 1;
        }
        if update.constitutional_reference.is_some() {
            set_clauses.push(format!("constitutional_reference = ${}", bind_count));
            bind_count += 1;
        }
        if update.is_active.is_some() {
            set_clauses.push(format!("is_active = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            return old_rule.ok_or_else(|| anyhow::anyhow!("Rule not found"));
        }

        set_clauses.push(format!("updated_at = ${}", bind_count));
        bind_count += 1;

        let query_str = format!(
            "UPDATE caws_rules SET {} WHERE id = ${}",
            set_clauses.join(", "),
            bind_count
        );

        let mut query = sqlx::query(&query_str);
        if let Some(name) = &update.name {
            query = query.bind(name);
        }
        if let Some(description) = &update.description {
            query = query.bind(description);
        }
        if let Some(rule_type) = &update.rule_type {
            query = query.bind(rule_type);
        }
        if let Some(severity) = &update.severity {
            query = query.bind(severity);
        }
        if let Some(file_patterns) = &update.file_patterns {
            query = query.bind(file_patterns);
        }
        if let Some(config) = &update.config {
            query = query.bind(config);
        }
        if let Some(ref constitutional_reference) = update.constitutional_reference {
            query = query.bind(constitutional_reference);
        }
        if let Some(is_active) = update.is_active {
            query = query.bind(is_active);
        }
        query = query.bind(Utc::now());
        query = query.bind(id);

        query.execute(&self.pool).await?;

        // Record history
        if let Some(old) = &old_rule {
            sqlx::query(
                r#"
                INSERT INTO rule_history (rule_id, action, changed_by, old_values, new_values, change_reason)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(id)
            .bind("updated")
            .bind("system")
            .bind(serde_json::json!({
                "name": old.name,
                "description": old.description,
                "is_active": old.is_active,
            }))
            .bind(serde_json::json!({
                "name": update.name.as_ref().unwrap_or(&old.name),
                "description": update.description.as_ref().unwrap_or(&old.description),
                "is_active": update.is_active.unwrap_or(old.is_active),
            }))
            .bind(None::<String>)
            .execute(&self.pool)
            .await?;
        }

        self.get_caws_rule(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Rule not found after update"))
    }

    async fn delete_caws_rule(&self, id: &str) -> Result<()> {
        // Record history before deletion
        if let Some(rule) = self.get_caws_rule(id).await? {
            sqlx::query(
                r#"
                INSERT INTO rule_history (rule_id, action, changed_by, old_values, change_reason)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(id)
            .bind("deleted")
            .bind("system")
            .bind(serde_json::json!({
                "name": rule.name,
                "description": rule.description,
            }))
            .bind(None::<String>)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query("DELETE FROM caws_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ============================================================================
    // CAWS Violations Operations
    // ============================================================================

    async fn create_caws_violation(&self, violation: CreateCawsViolation) -> Result<CawsViolation> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO caws_violations (
                id, task_id, violation_code, severity, description, file_path,
                line_number, column_number, rule_id, constitutional_reference,
                status, created_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(id)
        .bind(violation.task_id)
        .bind(&violation.violation_code)
        .bind(&violation.severity)
        .bind(&violation.description)
        .bind(&violation.file_path)
        .bind(violation.line_number)
        .bind(violation.column_number)
        .bind(&violation.rule_id)
        .bind(&violation.constitutional_reference)
        .bind(&violation.status)
        .bind(now)
        .bind(&violation.metadata)
        .execute(&self.pool)
        .await?;

        Ok(CawsViolation {
            id,
            task_id: violation.task_id,
            violation_code: violation.violation_code,
            severity: violation.severity,
            description: violation.description,
            file_path: violation.file_path,
            line_number: violation.line_number,
            column_number: violation.column_number,
            rule_id: violation.rule_id,
            constitutional_reference: violation.constitutional_reference,
            status: violation.status,
            created_at: now,
            resolved_at: None,
            metadata: violation.metadata,
        })
    }

    async fn get_caws_violation(&self, id: Uuid) -> Result<Option<CawsViolation>> {
        let row = sqlx::query_as::<_, CawsViolation>("SELECT * FROM caws_violations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn get_caws_violations(
        &self,
        task_id: Option<Uuid>,
        rule_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<CawsViolation>> {
        let mut query = String::from("SELECT * FROM caws_violations WHERE 1=1");
        let mut bind_count = 1;

        if let Some(_tid) = task_id {
            query.push_str(&format!(" AND task_id = ${}", bind_count));
            bind_count += 1;
        }

        if let Some(_rid) = rule_id {
            query.push_str(&format!(" AND rule_id = ${}", bind_count));
            bind_count += 1;
        }

        if let Some(_st) = status {
            query.push_str(&format!(" AND status = ${}", bind_count));
            bind_count += 1;
        }

        // Acknowledge bind_count is no longer needed after query string construction
        let _ = bind_count;

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, CawsViolation>(&query);
        if let Some(tid) = task_id {
            query_builder = query_builder.bind(tid);
        }
        if let Some(rid) = rule_id {
            query_builder = query_builder.bind(rid);
        }
        if let Some(st) = status {
            query_builder = query_builder.bind(st);
        }

        let violations = query_builder.fetch_all(&self.pool).await?;
        Ok(violations)
    }

    async fn update_caws_violation(
        &self,
        id: Uuid,
        update: UpdateCawsViolation,
    ) -> Result<CawsViolation> {
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if update.status.is_some() {
            updates.push(format!("status = ${}", bind_count));
            bind_count += 1;
        }

        if update.resolved_at.is_some() {
            updates.push(format!("resolved_at = ${}", bind_count));
            bind_count += 1;
        }

        if update.metadata.is_some() {
            updates.push(format!("metadata = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            return self
                .get_caws_violation(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Violation not found"));
        }

        let query_str = format!(
            "UPDATE caws_violations SET {} WHERE id = ${}",
            updates.join(", "),
            bind_count
        );

        let mut query = sqlx::query(&query_str);
        if let Some(status) = &update.status {
            query = query.bind(status);
        }
        if let Some(resolved_at) = update.resolved_at {
            query = query.bind(resolved_at);
        } else if update
            .status
            .as_ref()
            .map(|s| s == "resolved")
            .unwrap_or(false)
        {
            query = query.bind(Utc::now());
        }
        if let Some(metadata) = &update.metadata {
            query = query.bind(metadata);
        }
        query = query.bind(id);

        query.execute(&self.pool).await?;

        self.get_caws_violation(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Violation not found after update"))
    }

    async fn resolve_caws_violation(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE caws_violations SET status = 'resolved', resolved_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============================================================================
    // CAWS Specifications Operations
    // ============================================================================

    async fn create_caws_specification(
        &self,
        spec: CreateCawsSpecification,
    ) -> Result<CawsSpecification> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO caws_specifications (
                id, name, version, description, rules, config, is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(&spec.name)
        .bind(&spec.version)
        .bind(&spec.description)
        .bind(&spec.rules)
        .bind(&spec.config)
        .bind(spec.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CawsSpecification {
            id,
            name: spec.name,
            version: spec.version,
            description: spec.description,
            rules: spec.rules,
            config: spec.config,
            is_active: spec.is_active,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_caws_specification(&self, id: Uuid) -> Result<Option<CawsSpecification>> {
        let row = sqlx::query_as::<_, CawsSpecification>(
            "SELECT * FROM caws_specifications WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_caws_specifications(
        &self,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<CawsSpecification>> {
        let specs = match (name, is_active) {
            (Some(n), Some(active)) => {
                sqlx::query_as::<_, CawsSpecification>(
                    "SELECT * FROM caws_specifications WHERE name = $1 AND is_active = $2 ORDER BY created_at DESC"
                )
                .bind(n)
                .bind(active)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(n), None) => {
                sqlx::query_as::<_, CawsSpecification>(
                    "SELECT * FROM caws_specifications WHERE name = $1 ORDER BY created_at DESC"
                )
                .bind(n)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(active)) => {
                sqlx::query_as::<_, CawsSpecification>(
                    "SELECT * FROM caws_specifications WHERE is_active = $1 ORDER BY created_at DESC"
                )
                .bind(active)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, CawsSpecification>(
                    "SELECT * FROM caws_specifications ORDER BY created_at DESC"
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(specs)
    }

    async fn update_caws_specification(
        &self,
        id: Uuid,
        update: UpdateCawsSpecification,
    ) -> Result<CawsSpecification> {
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if update.name.is_some() {
            updates.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if update.version.is_some() {
            updates.push(format!("version = ${}", bind_count));
            bind_count += 1;
        }
        if update.description.is_some() {
            updates.push(format!("description = ${}", bind_count));
            bind_count += 1;
        }
        if update.rules.is_some() {
            updates.push(format!("rules = ${}", bind_count));
            bind_count += 1;
        }
        if update.config.is_some() {
            updates.push(format!("config = ${}", bind_count));
            bind_count += 1;
        }
        if update.is_active.is_some() {
            updates.push(format!("is_active = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            return self
                .get_caws_specification(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Specification not found"));
        }

        updates.push(format!("updated_at = ${}", bind_count));
        bind_count += 1;

        let query_str = format!(
            "UPDATE caws_specifications SET {} WHERE id = ${}",
            updates.join(", "),
            bind_count
        );

        let mut query = sqlx::query(&query_str);
        if let Some(name) = &update.name {
            query = query.bind(name);
        }
        if let Some(version) = &update.version {
            query = query.bind(version);
        }
        if let Some(description) = &update.description {
            query = query.bind(description);
        }
        if let Some(rules) = &update.rules {
            query = query.bind(rules);
        }
        if let Some(config) = &update.config {
            query = query.bind(config);
        }
        if let Some(is_active) = update.is_active {
            query = query.bind(is_active);
        }
        query = query.bind(Utc::now());
        query = query.bind(id);

        query.execute(&self.pool).await?;

        self.get_caws_specification(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Specification not found after update"))
    }

    async fn delete_caws_specification(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM caws_specifications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ============================================================================
    // Rule Templates Operations
    // ============================================================================

    async fn get_rule_templates(&self, rule_type: Option<&str>) -> Result<Vec<RuleTemplate>> {
        let query = if let Some(rt) = rule_type {
            sqlx::query_as::<_, RuleTemplate>(
                "SELECT * FROM rule_templates WHERE rule_type = $1 ORDER BY created_at DESC",
            )
            .bind(rt)
        } else {
            sqlx::query_as::<_, RuleTemplate>(
                "SELECT * FROM rule_templates ORDER BY created_at DESC",
            )
        };

        let templates = query.fetch_all(&self.pool).await?;
        Ok(templates)
    }

    async fn create_rule_template(&self, template: CreateRuleTemplate) -> Result<RuleTemplate> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO rule_templates (
                id, name, description, rule_type, template_config, example_config,
                is_system, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.rule_type)
        .bind(&template.template_config)
        .bind(&template.example_config)
        .bind(template.is_system)
        .bind(&template.created_by)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(RuleTemplate {
            id: template.id,
            name: template.name,
            description: template.description,
            rule_type: template.rule_type,
            template_config: template.template_config,
            example_config: template.example_config,
            is_system: template.is_system,
            created_by: template.created_by,
            created_at: now,
            updated_at: now,
        })
    }

    // ============================================================================
    // Rule Enforcement Status Operations
    // ============================================================================

    async fn get_rule_enforcement_status(
        &self,
        rule_id: Option<&str>,
        task_id: Option<Uuid>,
    ) -> Result<Vec<RuleEnforcementStatus>> {
        let mut query = String::from("SELECT * FROM rule_enforcement_status WHERE 1=1");
        let mut bind_count = 1;

        if rule_id.is_some() {
            query.push_str(&format!(" AND rule_id = ${}", bind_count));
            bind_count += 1;
        }

        if task_id.is_some() {
            query.push_str(&format!(" AND task_id = ${}", bind_count));
            bind_count += 1;
        }

        // Acknowledge bind_count is no longer needed after query string construction
        let _ = bind_count;

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, RuleEnforcementStatus>(&query);
        if let Some(rid) = rule_id {
            query_builder = query_builder.bind(rid);
        }
        if let Some(tid) = task_id {
            query_builder = query_builder.bind(tid);
        }

        let statuses = query_builder.fetch_all(&self.pool).await?;
        Ok(statuses)
    }

    async fn update_rule_enforcement_status(
        &self,
        rule_id: &str,
        task_id: Option<Uuid>,
        status: UpdateRuleEnforcementStatus,
    ) -> Result<RuleEnforcementStatus> {
        // Check if record exists
        let existing = if let Some(tid) = task_id {
            sqlx::query_as::<_, RuleEnforcementStatus>(
                "SELECT * FROM rule_enforcement_status WHERE rule_id = $1 AND task_id = $2",
            )
            .bind(rule_id)
            .bind(tid)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RuleEnforcementStatus>(
                "SELECT * FROM rule_enforcement_status WHERE rule_id = $1 AND task_id IS NULL",
            )
            .bind(rule_id)
            .fetch_optional(&self.pool)
            .await?
        };

        if let Some(_existing) = existing {
            // Update existing
            let mut updates = Vec::new();
            let mut bind_count = 1;

            if status.enforcement_state.is_some() {
                updates.push(format!("enforcement_state = ${}", bind_count));
                bind_count += 1;
            }
            if status.paused_until.is_some() {
                updates.push(format!("paused_until = ${}", bind_count));
                bind_count += 1;
            }
            if status.paused_reason.is_some() {
                updates.push(format!("paused_reason = ${}", bind_count));
                bind_count += 1;
            }
            if status.override_reason.is_some() {
                updates.push(format!("override_reason = ${}", bind_count));
                bind_count += 1;
            }
            if status.metadata.is_some() {
                updates.push(format!("metadata = ${}", bind_count));
                bind_count += 1;
            }

            if !updates.is_empty() {
                updates.push(format!("updated_at = ${}", bind_count));
                bind_count += 1;

                let where_clause = if task_id.is_some() {
                    format!(
                        "rule_id = ${} AND task_id = ${}",
                        bind_count,
                        bind_count + 1
                    )
                } else {
                    format!("rule_id = ${} AND task_id IS NULL", bind_count)
                };

                let query_str = format!(
                    "UPDATE rule_enforcement_status SET {} WHERE {}",
                    updates.join(", "),
                    where_clause
                );

                let mut query = sqlx::query(&query_str);
                if let Some(state) = &status.enforcement_state {
                    query = query.bind(state);
                }
                if let Some(paused_until) = status.paused_until {
                    query = query.bind(paused_until);
                }
                if let Some(reason) = &status.paused_reason {
                    query = query.bind(reason);
                }
                if let Some(reason) = &status.override_reason {
                    query = query.bind(reason);
                }
                if let Some(metadata) = &status.metadata {
                    query = query.bind(metadata);
                }
                query = query.bind(Utc::now());
                query = query.bind(rule_id);
                if let Some(tid) = task_id {
                    query = query.bind(tid);
                }

                query.execute(&self.pool).await?;
            }

            // Return updated record
            if let Some(tid) = task_id {
                sqlx::query_as::<_, RuleEnforcementStatus>(
                    "SELECT * FROM rule_enforcement_status WHERE rule_id = $1 AND task_id = $2",
                )
                .bind(rule_id)
                .bind(tid)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch updated status: {}", e))
            } else {
                sqlx::query_as::<_, RuleEnforcementStatus>(
                    "SELECT * FROM rule_enforcement_status WHERE rule_id = $1 AND task_id IS NULL",
                )
                .bind(rule_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch updated status: {}", e))
            }
        } else {
            // Create new
            let id = Uuid::new_v4();
            let now = Utc::now();

            sqlx::query(
                r#"
                INSERT INTO rule_enforcement_status (
                    id, rule_id, task_id, enforcement_state, paused_until,
                    paused_reason, override_reason, metadata, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(id)
            .bind(rule_id)
            .bind(task_id)
            .bind(status.enforcement_state.as_deref().unwrap_or("active"))
            .bind(status.paused_until)
            .bind(&status.paused_reason)
            .bind(&status.override_reason)
            .bind(status.metadata.unwrap_or_else(|| serde_json::json!({})))
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            if task_id.is_some() {
                sqlx::query_as::<_, RuleEnforcementStatus>(
                    "SELECT * FROM rule_enforcement_status WHERE id = $1",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch created status: {}", e))
            } else {
                sqlx::query_as::<_, RuleEnforcementStatus>(
                    "SELECT * FROM rule_enforcement_status WHERE id = $1",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch created status: {}", e))
            }
        }
    }

    // ============================================================================
    // Rule History Operations
    // ============================================================================

    async fn get_rule_history(
        &self,
        rule_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<RuleHistory>> {
        let limit_val = limit.unwrap_or(100);
        let history = sqlx::query_as::<_, RuleHistory>(
            "SELECT * FROM rule_history WHERE rule_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(rule_id)
        .bind(limit_val as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
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
        let _client = DatabaseClient::default();

        let entry = CreateAuditTrailEntry {
            entity_type: "test_entity".to_string(),
            entity_id: Uuid::new_v4(),
            action: "test_action".to_string(),
            details: json!({"test": "data"}),
            user_id: Some("test_user".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            timestamp: Some(Utc::now()),
        };

        // TODO: Implement comprehensive test with real database connection
        //       Currently verifies struct creation only; should implement comprehensive test with real database connection for full audit trail functionality.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Test uses real database connection
        // - Audit trail insertion is tested
        // - Database queries are tested
        // - Test assertions are comprehensive
        //
        // DEPENDENCIES:
        // - Test database setup (Required)
        // - Database test utilities (Required)
        // - Test fixtures (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Database testing expertise
        assert_eq!(entry.entity_type, "test_entity"); // Temporary: struct verification until database test
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
