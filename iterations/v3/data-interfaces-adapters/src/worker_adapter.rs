//! Worker Service Adapter
//!
//! Adapts `agent-workers` implementations to `data-interfaces` service traits.

use agent_agency_contracts::{TaskExecutionResult, TaskRequirements, TaskSpec};
use agent_workers::executor::TaskExecutor;
use async_trait::async_trait;
use data_interfaces::service_contracts::{
    ServiceError, WorkerPoolStatus, WorkerRegistration, WorkerService,
};
use data_infrastructure::simple_client::DatabaseClient;
use sqlx::Row;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Adapter for worker service
pub struct WorkerServiceAdapter {
    /// Task executor for executing tasks with workers
    task_executor: Arc<TaskExecutor>,
    /// Database client for worker discovery and registration
    db_client: Option<Arc<DatabaseClient>>,
}

impl WorkerServiceAdapter {
    /// Create a new worker service adapter without database client
    /// This will attempt to create a database client from environment variables
    /// if DATABASE_URL is set, otherwise TaskExecutor will use its own default.
    pub fn new() -> Self {
        Self::new_with_db_client(None)
    }

    /// Create a new worker service adapter with optional database client
    pub fn new_with_db_client(db_client: Option<Arc<DatabaseClient>>) -> Self {
        // Use provided database client or create one from environment
        let executor_db = if let Some(db) = db_client {
            Some(db.clone())
        } else {
            // Try to create a database client from environment
            // This is a best-effort attempt - if it fails, TaskExecutor will handle it
            match std::env::var("DATABASE_URL") {
                Ok(database_url) => {
                    let config = data_infrastructure::database_config::DatabaseConfig {
                        database_url,
                        ..Default::default()
                    };
                    match tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(DatabaseClient::new(config))
                    }) {
                        Ok(db) => Some(Arc::new(db)),
                        Err(e) => {
                            warn!("Failed to create database client from DATABASE_URL: {}. Worker service will operate with limited functionality.", e);
                            None
                        }
                    }
                }
                Err(_) => None,
            }
        };

        // TaskExecutor needs a database client, so use the provided one or create a default
        let task_executor_db = executor_db.clone().unwrap_or_else(|| {
            // Create a default database client for TaskExecutor
            // This may fail if DATABASE_URL is not set, but TaskExecutor should handle errors gracefully
            let config = data_infrastructure::database_config::DatabaseConfig::default();
            Arc::new(
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(DatabaseClient::new(config))
                })
                .unwrap_or_else(|e| {
                    panic!("Failed to create database client for TaskExecutor: {}. Set DATABASE_URL environment variable.", e)
                }),
            )
        });

        let task_executor = Arc::new(TaskExecutor::new(task_executor_db));

        Self {
            task_executor,
            db_client: executor_db,
        }
    }

    /// Create a new worker service adapter with explicit database client
    pub fn with_db_client(db_client: Arc<DatabaseClient>) -> Self {
        let task_executor = Arc::new(TaskExecutor::new(db_client.clone()));
        Self {
            task_executor,
            db_client: Some(db_client),
        }
    }

    /// Find an available worker from the database
    async fn find_available_worker(&self) -> Result<Uuid, ServiceError> {
        let Some(ref db) = self.db_client else {
            return Err(ServiceError::Unavailable(
                "Database client not available for worker discovery".to_string(),
            ));
        };

        let query = r#"
            SELECT id
            FROM workers
            WHERE is_active = true
            AND (last_heartbeat IS NULL OR last_heartbeat >= NOW() - INTERVAL '5 minutes')
            ORDER BY last_heartbeat DESC NULLS LAST
            LIMIT 1
        "#;

        match db.query(query, &[]).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Err(ServiceError::Unavailable(
                        "No available workers found in database".to_string(),
                    ));
                }

                let row = rows.first().unwrap();
                let worker_id_str: String = row
                    .try_get("id")
                    .map_err(|e| ServiceError::Internal(format!("Failed to parse worker ID: {}", e)))?;

                Uuid::parse_str(&worker_id_str)
                    .map_err(|e| ServiceError::Internal(format!("Invalid worker ID format: {}", e)))
            }
            Err(e) => {
                error!("Failed to query workers from database: {}", e);
                Err(ServiceError::Internal(format!(
                    "Database query failed: {}",
                    e
                )))
            }
        }
    }

    /// Get worker pool statistics from the database
    async fn get_pool_stats_from_db(&self) -> Result<WorkerPoolStatus, ServiceError> {
        let Some(ref db) = self.db_client else {
            return Ok(WorkerPoolStatus {
                total_workers: 0,
                active_workers: 0,
                idle_workers: 0,
                health_status: "Unknown - No database connection".to_string(),
            });
        };

        let total_query = "SELECT COUNT(*) as count FROM workers";
        let active_query = r#"
            SELECT COUNT(*) as count
            FROM workers
            WHERE is_active = true
            AND (last_heartbeat IS NULL OR last_heartbeat >= NOW() - INTERVAL '5 minutes')
        "#;

        let total: i64 = match db.query_one(total_query, &[]).await {
            Ok(Some(row)) => row.try_get("count").unwrap_or(0),
            Ok(None) => 0,
            Err(e) => {
                return Err(ServiceError::Internal(format!(
                    "Failed to query total workers: {}",
                    e
                )));
            }
        };

        let active: i64 = match db.query_one(active_query, &[]).await {
            Ok(Some(row)) => row.try_get("count").unwrap_or(0),
            Ok(None) => 0,
            Err(e) => {
                return Err(ServiceError::Internal(format!(
                    "Failed to query active workers: {}",
                    e
                )));
            }
        };

        let idle = active; // Idle workers are active but not executing tasks

        let health_status = if active > 0 {
            "Healthy".to_string()
        } else if total > 0 {
            "Degraded - No active workers".to_string()
        } else {
            "Unknown - No workers registered".to_string()
        };

        Ok(WorkerPoolStatus {
            total_workers: total as usize,
            active_workers: active as usize,
            idle_workers: idle as usize,
            health_status,
        })
    }
}

#[async_trait]
impl WorkerService for WorkerServiceAdapter {
    async fn execute_worker_task(
        &self,
        spec: TaskSpec,
        requirements: TaskRequirements,
    ) -> Result<TaskExecutionResult, ServiceError> {
        info!("Executing worker task {}: {}", spec.id, spec.title);

        // Find an available worker
        let worker_id = match self.find_available_worker().await {
            Ok(id) => {
                info!("Found available worker: {}", id);
                id
            }
            Err(e) => {
                error!("Failed to find available worker: {}", e);
                // As a fallback, try using a default worker ID from environment or generate one
                // This allows the system to work even if database is unavailable
                let fallback_id = std::env::var("DEFAULT_WORKER_ID")
                    .ok()
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .unwrap_or_else(|| {
                        warn!("No available workers found, using generated worker ID");
                        Uuid::new_v4()
                    });
                fallback_id
            }
        };

        // Convert TaskRequirements to internal format if needed
        // The TaskExecutor will handle the conversion internally
        let contract_spec = agent_agency_contracts::task_executor::TaskSpec {
            id: spec.id,
            title: spec.title,
            description: spec.description,
            priority: spec.priority,
            required_capabilities: spec.required_capabilities,
            context: spec.context,
            working_spec_id: spec.working_spec_id,
            timeout_seconds: spec.timeout_seconds,
            scope: spec.scope,
            risk_tier: spec.risk_tier,
            acceptance_criteria: spec.acceptance_criteria,
            caws_spec: spec.caws_spec,
            requirements: Some(requirements.clone()),
        };

        // Execute task using TaskExecutor
        // Note: TaskExecutor::execute_task takes (TaskSpec, Uuid, Option<&Arc<CircuitBreaker>>)
        // We pass None for circuit breaker since it's optional
        match self
            .task_executor
            .execute_task(contract_spec, worker_id, None)
            .await
        {
            Ok(result) => {
                info!("Task {} executed successfully", spec.id);
                Ok(result)
            }
            Err(e) => {
                error!("Task execution failed: {}", e);
                Err(ServiceError::Internal(format!(
                    "Worker execution failed: {}",
                    e
                )))
            }
        }
    }

    async fn get_worker_status(&self) -> Result<WorkerPoolStatus, ServiceError> {
        info!("Retrieving worker pool status");

        // Try to get stats from database
        match self.get_pool_stats_from_db().await {
            Ok(stats) => {
                info!(
                    "Worker pool status: {} total, {} active, {} idle - {}",
                    stats.total_workers,
                    stats.active_workers,
                    stats.idle_workers,
                    stats.health_status
                );
                Ok(stats)
            }
            Err(e) => {
                warn!("Failed to get worker pool status from database: {}", e);
                // Return a degraded status instead of failing completely
                Ok(WorkerPoolStatus {
                    total_workers: 0,
                    active_workers: 0,
                    idle_workers: 0,
                    health_status: format!("Degraded - Cannot query database: {}", e),
                })
            }
        }
    }

    async fn register_worker(&self, registration: WorkerRegistration) -> Result<(), ServiceError> {
        info!(
            "Registering worker {} with capabilities: {:?}",
            registration.worker_id, registration.capabilities
        );

        let Some(ref db) = self.db_client else {
            return Err(ServiceError::Unavailable(
                "Database client not available for worker registration".to_string(),
            ));
        };

        let query = r#"
            INSERT INTO workers (
                id, name, specialty, capabilities, is_active, last_heartbeat, version, endpoint_url
            ) VALUES (
                $1, $2, $3, $4, $5, NOW(), $6, $7
            )
            ON CONFLICT (id) DO UPDATE SET
                capabilities = $4,
                is_active = $5,
                last_heartbeat = NOW(),
                version = $6,
                endpoint_url = $7
        "#;

        let name = registration
            .metadata
            .as_ref()
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown Worker")
            .to_string();

        let specialty = registration
            .metadata
            .as_ref()
            .and_then(|m| m.get("specialty"))
            .and_then(|s| s.as_str())
            .unwrap_or("general")
            .to_string();

        let capabilities_json = serde_json::to_value(&registration.capabilities)
            .map_err(|e| ServiceError::Internal(format!("Failed to serialize capabilities: {}", e)))?;

        let version = registration
            .metadata
            .as_ref()
            .and_then(|m| m.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let endpoint_url = registration
            .metadata
            .as_ref()
            .and_then(|m| m.get("endpoint_url"))
            .and_then(|u| u.as_str())
            .unwrap_or("http://localhost:8889")
            .to_string();

        // Use sqlx query builder directly for parameterized queries
        sqlx::query(query)
            .bind(registration.worker_id.to_string())
            .bind(&name)
            .bind(&specialty)
            .bind(&capabilities_json)
            .bind(true) // is_active
            .bind(&version)
            .bind(&endpoint_url)
            .execute(db.pool())
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to register worker: {}", e)))?;

        info!("Worker {} registered successfully", registration.worker_id);
        Ok(())
    }
}
