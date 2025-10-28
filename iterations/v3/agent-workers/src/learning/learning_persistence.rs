//! Learning persistence trait and implementations

use async_trait::async_trait;
use std::collections::HashMap;
use anyhow::{Context, Result};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error, debug};

use crate::parallel_types::{TaskId, WorkerId};
use crate::learning::types::*;

/// Trait for persisting learning data
#[async_trait]
pub trait LearningPersistence: Send + Sync {
    /// Store execution records for analysis
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> Result<()>;
    
    /// Get execution records matching a pattern
    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> Result<Vec<ExecutionRecord>>;
    
    /// Store worker performance profiles
    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> Result<()>;
    
    /// Get worker performance profile
    async fn get_worker_profile(&self, worker_id: &WorkerId) -> Result<Option<WorkerPerformanceProfile>>;
    
    /// Store success patterns
    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> Result<()>;
    
    /// Get all success patterns
    async fn get_success_patterns(&self) -> Result<Vec<SuccessPattern>>;
    
    /// Store failure patterns
    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> Result<()>;
    
    /// Get all failure patterns
    async fn get_failure_patterns(&self) -> Result<Vec<FailurePattern>>;
    
    /// Store optimal configurations
    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> Result<()>;
    
    /// Get all optimal configurations
    async fn get_optimal_configs(&self) -> Result<Vec<OptimalConfig>>;
    
    /// Store optimization events
    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> Result<()>;
    
    /// Get optimization events for a config
    async fn get_optimization_events(&self, config_id: &Uuid) -> Result<Vec<OptimizationEvent>>;
}

/// In-memory learning persistence implementation
pub struct InMemoryLearningPersistence {
    execution_records: std::sync::Arc<tokio::sync::RwLock<Vec<ExecutionRecord>>>,
    worker_profiles: std::sync::Arc<tokio::sync::RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
    success_patterns: std::sync::Arc<tokio::sync::RwLock<Vec<SuccessPattern>>>,
    failure_patterns: std::sync::Arc<tokio::sync::RwLock<Vec<FailurePattern>>>,
    optimal_configs: std::sync::Arc<tokio::sync::RwLock<Vec<OptimalConfig>>>,
    optimization_events: std::sync::Arc<tokio::sync::RwLock<Vec<OptimizationEvent>>>,
}

impl InMemoryLearningPersistence {
    pub fn new() -> Self {
        Self {
            execution_records: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            worker_profiles: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            success_patterns: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            failure_patterns: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            optimal_configs: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            optimization_events: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LearningPersistence for InMemoryLearningPersistence {
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> Result<()> {
        let mut storage = self.execution_records.write().await;
        storage.extend(records);
        Ok(())
    }
    
    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> Result<Vec<ExecutionRecord>> {
        let storage = self.execution_records.read().await;
        let mut records: Vec<ExecutionRecord> = storage.iter()
            .filter(|record| {
                // Simple pattern matching - in a real implementation, this would be more sophisticated
                record.metadata.contains_key(&pattern.pattern_type.to_string())
            })
            .cloned()
            .collect();
        
        if let Some(limit) = limit {
            records.truncate(limit);
        }
        
        Ok(records)
    }
    
    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> Result<()> {
        let mut storage = self.worker_profiles.write().await;
        for (worker_id, profile) in profiles {
            storage.insert(worker_id, profile);
        }
        Ok(())
    }
    
    async fn get_worker_profile(&self, worker_id: &WorkerId) -> Result<Option<WorkerPerformanceProfile>> {
        let storage = self.worker_profiles.read().await;
        Ok(storage.get(worker_id).cloned())
    }
    
    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> Result<()> {
        let mut storage = self.success_patterns.write().await;
        storage.extend(patterns);
        Ok(())
    }
    
    async fn get_success_patterns(&self) -> Result<Vec<SuccessPattern>> {
        let storage = self.success_patterns.read().await;
        Ok(storage.clone())
    }
    
    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> Result<()> {
        let mut storage = self.failure_patterns.write().await;
        storage.extend(patterns);
        Ok(())
    }
    
    async fn get_failure_patterns(&self) -> Result<Vec<FailurePattern>> {
        let storage = self.failure_patterns.read().await;
        Ok(storage.clone())
    }
    
    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> Result<()> {
        let mut storage = self.optimal_configs.write().await;
        storage.extend(configs);
        Ok(())
    }
    
    async fn get_optimal_configs(&self) -> Result<Vec<OptimalConfig>> {
        let storage = self.optimal_configs.read().await;
        Ok(storage.clone())
    }
    
    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> Result<()> {
        let mut storage = self.optimization_events.write().await;
        storage.extend(events);
        Ok(())
    }
    
    async fn get_optimization_events(&self, config_id: &Uuid) -> Result<Vec<OptimizationEvent>> {
        let storage = self.optimization_events.read().await;
        let events: Vec<OptimizationEvent> = storage.iter()
            .filter(|event| event.config_id == *config_id)
            .cloned()
            .collect();
        Ok(events)
    }
}

impl Default for InMemoryLearningPersistence {
    fn default() -> Self {
        Self::new()
    }
}

/// Real database learning persistence using PostgreSQL
pub struct DatabaseLearningPersistence {
    db_url: String,
    pool: sqlx::PgPool,
}

impl DatabaseLearningPersistence {
    /// Create new database persistence layer
    pub async fn new(db_url: String) -> Result<Self> {
        info!("Initializing database learning persistence with URL: {}", db_url);
        
        let pool = sqlx::PgPool::connect(&db_url).await
            .context("Failed to connect to PostgreSQL database")?;

        // Initialize schema
        Self::initialize_schema(&pool).await?;
        
        Ok(Self {
            db_url,
            pool,
        })
    }

    /// Initialize database schema for learning data
    async fn initialize_schema(pool: &sqlx::PgPool) -> Result<()> {
        info!("Initializing learning persistence schema");
        
        // Create execution records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS execution_records (
                id UUID PRIMARY KEY,
                task_id UUID NOT NULL,
                worker_id UUID NOT NULL,
                execution_time_ms BIGINT NOT NULL,
                success BOOLEAN NOT NULL,
                error_message TEXT,
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create execution_records table")?;

        // Create worker performance profiles table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS worker_performance_profiles (
                id UUID PRIMARY KEY,
                worker_id UUID NOT NULL UNIQUE,
                total_executions BIGINT DEFAULT 0,
                successful_executions BIGINT DEFAULT 0,
                failed_executions BIGINT DEFAULT 0,
                average_execution_time_ms DOUBLE PRECISION DEFAULT 0.0,
                success_rate DOUBLE PRECISION DEFAULT 0.0,
                last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                metadata JSONB
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create worker_performance_profiles table")?;

        // Create success patterns table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS success_patterns (
                id UUID PRIMARY KEY,
                pattern_name VARCHAR(255) NOT NULL,
                pattern_type VARCHAR(100) NOT NULL,
                confidence_score DOUBLE PRECISION NOT NULL,
                frequency INTEGER NOT NULL,
                conditions JSONB NOT NULL,
                outcomes JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create success_patterns table")?;

        // Create failure patterns table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS failure_patterns (
                id UUID PRIMARY KEY,
                pattern_name VARCHAR(255) NOT NULL,
                pattern_type VARCHAR(100) NOT NULL,
                confidence_score DOUBLE PRECISION NOT NULL,
                frequency INTEGER NOT NULL,
                conditions JSONB NOT NULL,
                outcomes JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create failure_patterns table")?;

        // Create optimal configurations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS optimal_configs (
                id UUID PRIMARY KEY,
                worker_type VARCHAR(100) NOT NULL,
                task_type VARCHAR(100) NOT NULL,
                parameters JSONB NOT NULL,
                performance_metrics JSONB NOT NULL,
                confidence DOUBLE PRECISION NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                expires_at TIMESTAMP WITH TIME ZONE
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create optimal_configs table")?;

        // Create optimization events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS optimization_events (
                id UUID PRIMARY KEY,
                config_id UUID NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                event_data JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create optimization_events table")?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_execution_records_task_id ON execution_records(task_id)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_execution_records_worker_id ON execution_records(worker_id)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_execution_records_created_at ON execution_records(created_at)")
            .execute(pool)
            .await?;

        info!("Learning persistence schema initialized successfully");
        Ok(())
    }
}

#[async_trait]
impl LearningPersistence for DatabaseLearningPersistence {
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        debug!("Storing {} execution records to database", records.len());

        for record in records {
            sqlx::query(
                r#"
                INSERT INTO execution_records (id, task_id, worker_id, execution_time_ms, success, error_message, metadata, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE SET
                    execution_time_ms = EXCLUDED.execution_time_ms,
                    success = EXCLUDED.success,
                    error_message = EXCLUDED.error_message,
                    metadata = EXCLUDED.metadata
                "#
            )
            .bind(record.id)
            .bind(record.task_id.0)
            .bind(record.worker_id.0)
            .bind(record.execution_time_ms as i64)
            .bind(record.success)
            .bind(record.error_message.as_deref())
            .bind(&record.metadata)
            .bind(record.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store execution record")?;
        }

        info!("Successfully stored {} execution records", records.len());
        Ok(())
    }

    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> Result<Vec<ExecutionRecord>> {
        debug!("Retrieving execution records for pattern: {}", pattern.pattern_name);

        let query = if let Some(limit) = limit {
            sqlx::query_as::<_, ExecutionRecord>(
                r#"
                SELECT id, task_id, worker_id, execution_time_ms, success, error_message, metadata, created_at
                FROM execution_records
                WHERE metadata @> $1
                ORDER BY created_at DESC
                LIMIT $2
                "#
            )
            .bind(serde_json::json!({ "pattern_type": pattern.pattern_type }))
            .bind(limit as i64)
        } else {
            sqlx::query_as::<_, ExecutionRecord>(
                r#"
                SELECT id, task_id, worker_id, execution_time_ms, success, error_message, metadata, created_at
                FROM execution_records
                WHERE metadata @> $1
                ORDER BY created_at DESC
                "#
            )
            .bind(serde_json::json!({ "pattern_type": pattern.pattern_type }))
        };

        let records = query
            .fetch_all(&self.pool)
            .await
            .context("Failed to retrieve execution records")?;

        debug!("Retrieved {} execution records", records.len());
        Ok(records)
    }

    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> Result<()> {
        if profiles.is_empty() {
            return Ok(());
        }

        debug!("Storing {} worker profiles to database", profiles.len());

        for (worker_id, profile) in profiles {
            sqlx::query(
                r#"
                INSERT INTO worker_performance_profiles (id, worker_id, total_executions, successful_executions, failed_executions, average_execution_time_ms, success_rate, metadata, last_updated)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (worker_id) DO UPDATE SET
                    total_executions = EXCLUDED.total_executions,
                    successful_executions = EXCLUDED.successful_executions,
                    failed_executions = EXCLUDED.failed_executions,
                    average_execution_time_ms = EXCLUDED.average_execution_time_ms,
                    success_rate = EXCLUDED.success_rate,
                    metadata = EXCLUDED.metadata,
                    last_updated = EXCLUDED.last_updated
                "#
            )
            .bind(profile.id)
            .bind(worker_id.0)
            .bind(profile.total_executions as i64)
            .bind(profile.successful_executions as i64)
            .bind(profile.failed_executions as i64)
            .bind(profile.average_execution_time_ms)
            .bind(profile.success_rate)
            .bind(&profile.metadata)
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .context("Failed to store worker profile")?;
        }

        info!("Successfully stored {} worker profiles", profiles.len());
        Ok(())
    }

    async fn get_worker_profile(&self, worker_id: &WorkerId) -> Result<Option<WorkerPerformanceProfile>> {
        debug!("Retrieving worker profile for: {}", worker_id.0);

        let profile = sqlx::query_as::<_, WorkerPerformanceProfile>(
            r#"
            SELECT id, worker_id, total_executions, successful_executions, failed_executions, average_execution_time_ms, success_rate, metadata, last_updated
            FROM worker_performance_profiles
            WHERE worker_id = $1
            "#
        )
        .bind(worker_id.0)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to retrieve worker profile")?;

        Ok(profile)
    }

    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> Result<()> {
        if patterns.is_empty() {
            return Ok(());
        }

        debug!("Storing {} success patterns to database", patterns.len());

        for pattern in patterns {
            sqlx::query(
                r#"
                INSERT INTO success_patterns (id, pattern_name, pattern_type, confidence_score, frequency, conditions, outcomes, created_at, last_seen)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE SET
                    confidence_score = EXCLUDED.confidence_score,
                    frequency = EXCLUDED.frequency,
                    conditions = EXCLUDED.conditions,
                    outcomes = EXCLUDED.outcomes,
                    last_seen = EXCLUDED.last_seen
                "#
            )
            .bind(pattern.id)
            .bind(&pattern.pattern_name)
            .bind(&pattern.pattern_type)
            .bind(pattern.confidence_score)
            .bind(pattern.frequency as i32)
            .bind(&pattern.conditions)
            .bind(&pattern.outcomes)
            .bind(pattern.created_at)
            .bind(pattern.last_seen)
            .execute(&self.pool)
            .await
            .context("Failed to store success pattern")?;
        }

        info!("Successfully stored {} success patterns", patterns.len());
        Ok(())
    }

    async fn get_success_patterns(&self) -> Result<Vec<SuccessPattern>> {
        debug!("Retrieving all success patterns from database");

        let patterns = sqlx::query_as::<_, SuccessPattern>(
            r#"
            SELECT id, pattern_name, pattern_type, confidence_score, frequency, conditions, outcomes, created_at, last_seen
            FROM success_patterns
            ORDER BY confidence_score DESC, frequency DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve success patterns")?;

        debug!("Retrieved {} success patterns", patterns.len());
        Ok(patterns)
    }

    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> Result<()> {
        if patterns.is_empty() {
            return Ok(());
        }

        debug!("Storing {} failure patterns to database", patterns.len());

        for pattern in patterns {
            sqlx::query(
                r#"
                INSERT INTO failure_patterns (id, pattern_name, pattern_type, confidence_score, frequency, conditions, outcomes, created_at, last_seen)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE SET
                    confidence_score = EXCLUDED.confidence_score,
                    frequency = EXCLUDED.frequency,
                    conditions = EXCLUDED.conditions,
                    outcomes = EXCLUDED.outcomes,
                    last_seen = EXCLUDED.last_seen
                "#
            )
            .bind(pattern.id)
            .bind(&pattern.pattern_name)
            .bind(&pattern.pattern_type)
            .bind(pattern.confidence_score)
            .bind(pattern.frequency as i32)
            .bind(&pattern.conditions)
            .bind(&pattern.outcomes)
            .bind(pattern.created_at)
            .bind(pattern.last_seen)
            .execute(&self.pool)
            .await
            .context("Failed to store failure pattern")?;
        }

        info!("Successfully stored {} failure patterns", patterns.len());
        Ok(())
    }

    async fn get_failure_patterns(&self) -> Result<Vec<FailurePattern>> {
        debug!("Retrieving all failure patterns from database");

        let patterns = sqlx::query_as::<_, FailurePattern>(
            r#"
            SELECT id, pattern_name, pattern_type, confidence_score, frequency, conditions, outcomes, created_at, last_seen
            FROM failure_patterns
            ORDER BY confidence_score DESC, frequency DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve failure patterns")?;

        debug!("Retrieved {} failure patterns", patterns.len());
        Ok(patterns)
    }

    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> Result<()> {
        if configs.is_empty() {
            return Ok(());
        }

        debug!("Storing {} optimal configs to database", configs.len());

        for config in configs {
            sqlx::query(
                r#"
                INSERT INTO optimal_configs (id, worker_type, task_type, parameters, performance_metrics, confidence, created_at, expires_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE SET
                    parameters = EXCLUDED.parameters,
                    performance_metrics = EXCLUDED.performance_metrics,
                    confidence = EXCLUDED.confidence,
                    expires_at = EXCLUDED.expires_at
                "#
            )
            .bind(config.id)
            .bind(&config.worker_type)
            .bind(&config.task_type)
            .bind(&config.parameters)
            .bind(&config.performance_metrics)
            .bind(config.confidence)
            .bind(config.created_at)
            .bind(config.expires_at)
            .execute(&self.pool)
            .await
            .context("Failed to store optimal config")?;
        }

        info!("Successfully stored {} optimal configs", configs.len());
        Ok(())
    }

    async fn get_optimal_configs(&self) -> Result<Vec<OptimalConfig>> {
        debug!("Retrieving all optimal configs from database");

        let configs = sqlx::query_as::<_, OptimalConfig>(
            r#"
            SELECT id, worker_type, task_type, parameters, performance_metrics, confidence, created_at, expires_at
            FROM optimal_configs
            WHERE expires_at IS NULL OR expires_at > NOW()
            ORDER BY confidence DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve optimal configs")?;

        debug!("Retrieved {} optimal configs", configs.len());
        Ok(configs)
    }

    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        debug!("Storing {} optimization events to database", events.len());

        for event in events {
            sqlx::query(
                r#"
                INSERT INTO optimization_events (id, config_id, event_type, event_data, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(event.id)
            .bind(event.config_id)
            .bind(&event.event_type)
            .bind(&event.event_data)
            .bind(event.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store optimization event")?;
        }

        info!("Successfully stored {} optimization events", events.len());
        Ok(())
    }

    async fn get_optimization_events(&self, config_id: &Uuid) -> Result<Vec<OptimizationEvent>> {
        debug!("Retrieving optimization events for config: {}", config_id);

        let events = sqlx::query_as::<_, OptimizationEvent>(
            r#"
            SELECT id, config_id, event_type, event_data, created_at
            FROM optimization_events
            WHERE config_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(config_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve optimization events")?;

        debug!("Retrieved {} optimization events", events.len());
        Ok(events)
    }
}
