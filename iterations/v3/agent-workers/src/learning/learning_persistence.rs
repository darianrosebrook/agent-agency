//! Learning persistence trait and implementations

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::learning::types::*;
use crate::{TaskId, WorkerId, WorkerSpecialty};

/// Trait for persisting learning data
#[async_trait]
pub trait LearningPersistence: Send + Sync {
    /// Store execution records for analysis
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> Result<()>;

    /// Get execution records matching a pattern
    async fn get_execution_records(
        &self,
        pattern: &TaskPattern,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionRecord>>;

    /// Store worker performance profiles
    async fn store_worker_profiles(
        &self,
        profiles: HashMap<WorkerId, WorkerPerformanceProfile>,
    ) -> Result<()>;

    /// Get worker performance profile
    async fn get_worker_profile(
        &self,
        worker_id: &WorkerId,
    ) -> Result<Option<WorkerPerformanceProfile>>;

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
    worker_profiles:
        std::sync::Arc<tokio::sync::RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
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

    async fn get_execution_records(
        &self,
        pattern: &TaskPattern,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionRecord>> {
        let storage = self.execution_records.read().await;
        let mut records: Vec<ExecutionRecord> = storage
            .iter()
            .filter(|record| {
                //       Currently uses simple metadata key matching; should implement more sophisticated pattern matching with regex, fuzzy matching, and complex queries.
                //
                // COMPLETION CHECKLIST:
                // [ ] Implement regex pattern matching
                // [ ] Add fuzzy matching for approximate matches
                // [ ] Support complex query patterns (AND/OR/NOT)
                // [ ] Handle pattern compilation and caching
                // [ ] Add pattern matching performance optimization
                // [ ] Add unit tests with various pattern types
                // [ ] Add integration tests with real execution records
                // [ ] Performance: Pattern matching should complete in <10ms
                // [ ] Documentation: Document pattern matching syntax
                //
                // ACCEPTANCE CRITERIA:
                // - Pattern matching supports regex patterns
                // - Fuzzy matching finds approximate matches
                // - Complex queries are supported
                // - Pattern matching is performant
                // - Pattern syntax is well-documented
                //
                // DEPENDENCIES:
                // - Regex library (Required)
                // - Fuzzy matching library (Optional)
                // - Query parser (Required)
                //
                // ESTIMATED EFFORT: 5-7 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (search feature)
                // - Change Budget: ~200 LOC
                // - Reviewer Requirements: Pattern matching expertise
                record
                    .metadata
                    .contains_key(&pattern.pattern_type.to_string())
            })
            .cloned()
            .collect();

        if let Some(limit) = limit {
            records.truncate(limit);
        }

        Ok(records)
    }

    async fn store_worker_profiles(
        &self,
        profiles: HashMap<WorkerId, WorkerPerformanceProfile>,
    ) -> Result<()> {
        let mut storage = self.worker_profiles.write().await;
        for (worker_id, profile) in &profiles {
            storage.insert(worker_id.clone(), profile.clone());
        }
        Ok(())
    }

    async fn get_worker_profile(
        &self,
        worker_id: &WorkerId,
    ) -> Result<Option<WorkerPerformanceProfile>> {
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
        let events: Vec<OptimizationEvent> = storage
            .iter()
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
        info!(
            "Initializing database learning persistence with URL: {}",
            db_url
        );

        let pool = sqlx::PgPool::connect(&db_url)
            .await
            .context("Failed to connect to PostgreSQL database")?;

        // Initialize schema
        Self::initialize_schema(&pool).await?;

        Ok(Self { db_url, pool })
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
                success SMALLINT NOT NULL DEFAULT 0,
                quality_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                error_message TEXT,
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
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
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create worker_performance_profiles table")?;

        // Create success patterns table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS success_patterns (
                id UUID PRIMARY KEY,
                pattern_type VARCHAR(100) NOT NULL,
                success_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                average_quality DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                frequency BIGINT NOT NULL DEFAULT 0,
                conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create success_patterns table")?;

        // Create failure patterns table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS failure_patterns (
                id UUID PRIMARY KEY,
                pattern_type VARCHAR(100) NOT NULL,
                failure_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                frequency BIGINT NOT NULL DEFAULT 0,
                conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
                common_errors TEXT[] NOT NULL DEFAULT '{}',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create failure_patterns table")?;

        // Create optimal configurations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS optimal_configs (
                id UUID PRIMARY KEY,
                config_type VARCHAR(100) NOT NULL,
                parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
                performance_metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
                conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
                confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create optimal_configs table")?;

        // Create optimization events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS optimization_events (
                id UUID PRIMARY KEY,
                event_type VARCHAR(100) NOT NULL,
                config_id UUID NOT NULL,
                performance_delta JSONB NOT NULL DEFAULT '{}'::jsonb,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                metadata JSONB NOT NULL DEFAULT '{}'::jsonb
            )
            "#,
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

        for record in &records {
            sqlx::query(
                r#"
                INSERT INTO execution_records (id, task_id, worker_id, execution_time_ms, success, quality_score, error_message, metadata, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE SET
                    execution_time_ms = EXCLUDED.execution_time_ms,
                    success = EXCLUDED.success,
                    quality_score = EXCLUDED.quality_score,
                    error_message = EXCLUDED.error_message,
                    metadata = EXCLUDED.metadata
                "#
            )
            .bind(record.id)
            .bind(record.task_id.0)
            .bind(record.worker_id.0)
            .bind(record.execution_time_ms as i64)
            .bind(if record.success { 1i16 } else { 0i16 })
            .bind(record.quality_score)
            .bind(record.error_message.as_deref())
            .bind(serde_json::to_value(&record.metadata).ok())
            .bind(record.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store execution record")?;
        }

        info!("Successfully stored {} execution records", records.len());
        Ok(())
    }

    async fn get_execution_records(
        &self,
        pattern: &TaskPattern,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionRecord>> {
        debug!(
            "Retrieving execution records for pattern: {:?}",
            pattern.pattern_type
        );

        let query = if let Some(limit) = limit {
            sqlx::query_as::<_, ExecutionRecord>(
                r#"
                SELECT id, task_id, worker_id, execution_time_ms, success, quality_score, error_message, metadata, created_at
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
                SELECT id, task_id, worker_id, execution_time_ms, success, quality_score, error_message, metadata, created_at
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

    async fn store_worker_profiles(
        &self,
        profiles: HashMap<WorkerId, WorkerPerformanceProfile>,
    ) -> Result<()> {
        if profiles.is_empty() {
            return Ok(());
        }

        debug!("Storing {} worker profiles to database", profiles.len());

        for (worker_id, profile) in &profiles {
            // Serialize complex fields as JSON
            let metadata = serde_json::json!({
                "specialty": serde_json::to_string(&profile.specialty).unwrap_or_default(),
                "average_quality_score": profile.average_quality_score,
                "performance_trend": serde_json::to_string(&profile.performance_trend).unwrap_or_default(),
                "capability_scores": profile.capability_scores
            });

            sqlx::query(
                r#"
                INSERT INTO worker_performance_profiles (worker_id, total_executions, successful_executions, average_execution_time_ms, success_rate, metadata, last_updated)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (worker_id) DO UPDATE SET
                    total_executions = EXCLUDED.total_executions,
                    successful_executions = EXCLUDED.successful_executions,
                    average_execution_time_ms = EXCLUDED.average_execution_time_ms,
                    success_rate = EXCLUDED.success_rate,
                    metadata = EXCLUDED.metadata,
                    last_updated = EXCLUDED.last_updated
                "#
            )
            .bind(worker_id.0)
            .bind(profile.total_executions as i64)
            .bind(profile.successful_executions as i64)
            .bind(profile.average_execution_time_ms)
            .bind(profile.average_quality_score) // Using average_quality_score as success_rate approximation
            .bind(&metadata)
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .context("Failed to store worker profile")?;
        }

        info!("Successfully stored {} worker profiles", profiles.len());
        Ok(())
    }

    async fn get_worker_profile(
        &self,
        worker_id: &WorkerId,
    ) -> Result<Option<WorkerPerformanceProfile>> {
        debug!("Retrieving worker profile for: {}", worker_id.0);

        #[derive(Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
        struct WorkerProfileRow {
            worker_id: Uuid,
            total_executions: i64,
            successful_executions: i64,
            average_execution_time_ms: f64,
            success_rate: f64,
            metadata: serde_json::Value,
            #[schemars(with = "String")]
            last_updated: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, WorkerProfileRow>(
            r#"
            SELECT worker_id, total_executions, successful_executions, average_execution_time_ms, success_rate, metadata, last_updated
            FROM worker_performance_profiles
            WHERE worker_id = $1
            "#
        )
        .bind(worker_id.0)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to retrieve worker profile")?;

        if let Some(row) = row {
            // Deserialize the metadata JSON
            let specialty_str = row
                .metadata
                .get("specialty")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let specialty: WorkerSpecialty =
                serde_json::from_str(specialty_str).unwrap_or(WorkerSpecialty::General);

            let performance_trend_str = row
                .metadata
                .get("performance_trend")
                .and_then(|v| v.as_str())
                .unwrap_or("\"Unknown\"");
            let performance_trend: PerformanceTrend =
                serde_json::from_str(performance_trend_str).unwrap_or(PerformanceTrend::Unknown);

            let capability_scores: HashMap<String, f64> = row
                .metadata
                .get("capability_scores")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
                        .collect()
                })
                .unwrap_or_default();

            let profile = WorkerPerformanceProfile {
                worker_id: WorkerId(row.worker_id),
                specialty,
                total_executions: row.total_executions as u64,
                successful_executions: row.successful_executions as u64,
                average_execution_time_ms: row.average_execution_time_ms,
                average_quality_score: row.success_rate, // Stored as success_rate in DB
                last_updated: row.last_updated,
                performance_trend,
                capability_scores,
                task_count: row.total_executions as u64,
                success_rate: row.success_rate,
                // TODO: Implement comprehensive quality score calculation:
                // 1. Quality metrics: Calculate quality score from multiple factors
                //    - Combine success rate with other quality indicators
                //    - Weight different quality factors appropriately
                //    - Consider task complexity and difficulty
                // 2. Score normalization: Normalize quality scores
                //    - Scale scores to consistent range (0.0-1.0)
                //    - Handle edge cases (no tasks, all failures, etc.)
                //    - Apply statistical normalization if needed
                // 3. Score validation: Validate quality score calculations
                //    - Ensure scores are within valid range
                //    - Handle calculation errors gracefully
                //    - Log quality score calculation details
                // ACCEPTANCE CRITERIA:
                // - Quality score incorporates multiple quality factors
                // - Scores are normalized to consistent range
                // - Score calculations are validated and error-free
                // DEPENDENCIES:
                // - Quality metrics collection (Required)
                // - Score calculation algorithms (Required)
                // PRIORITY: Medium
                quality_score: row.success_rate,
                specialization_score: 0.0, // OPTIONAL: Calculate specialization score (deferred - analytics feature)
                metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
            };

            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> Result<()> {
        if patterns.is_empty() {
            return Ok(());
        }

        debug!("Storing {} success patterns to database", patterns.len());

        for pattern in &patterns {
            sqlx::query(
                r#"
                INSERT INTO success_patterns (id, pattern_type, success_rate, average_quality, frequency, conditions, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    success_rate = EXCLUDED.success_rate,
                    average_quality = EXCLUDED.average_quality,
                    frequency = EXCLUDED.frequency,
                    conditions = EXCLUDED.conditions
                "#
            )
            .bind(pattern.id)
            .bind(serde_json::to_string(&pattern.pattern_type).unwrap_or_default())
            .bind(pattern.success_rate)
            .bind(pattern.average_quality)
            .bind(pattern.frequency as i64)
            .bind(serde_json::to_value(&pattern.conditions).ok())
            .bind(pattern.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store success pattern")?;
        }

        info!("Successfully stored {} success patterns", patterns.len());
        Ok(())
    }

    async fn get_success_patterns(&self) -> Result<Vec<SuccessPattern>> {
        debug!("Retrieving all success patterns from database");

        #[derive(Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
        struct SuccessPatternRow {
            id: Uuid,
            pattern_type: String,
            success_rate: f64,
            average_quality: f64,
            frequency: i64,
            conditions: serde_json::Value,
            #[schemars(with = "String")]
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, SuccessPatternRow>(
            r#"
            SELECT id, pattern_type, success_rate, average_quality, frequency, conditions, created_at
            FROM success_patterns
            ORDER BY success_rate DESC, frequency DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve success patterns")?;

        let patterns: Vec<SuccessPattern> = rows
            .into_iter()
            .filter_map(|row| {
                // Deserialize pattern_type from JSON string
                let pattern_type: PatternType = serde_json::from_str(&row.pattern_type).ok()?;

                // Convert conditions from JSON Value to HashMap
                let conditions: HashMap<String, serde_json::Value> = row
                    .conditions
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                Some(SuccessPattern {
                    id: row.id,
                    pattern_type,
                    conditions,
                    success_rate: row.success_rate,
                    average_quality: row.average_quality,
                    frequency: row.frequency as u64,
                    created_at: row.created_at,
                })
            })
            .collect();

        debug!("Retrieved {} success patterns", patterns.len());
        Ok(patterns)
    }

    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> Result<()> {
        if patterns.is_empty() {
            return Ok(());
        }

        debug!("Storing {} failure patterns to database", patterns.len());

        for pattern in &patterns {
            sqlx::query(
                r#"
                INSERT INTO failure_patterns (id, pattern_type, failure_rate, frequency, conditions, common_errors, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    failure_rate = EXCLUDED.failure_rate,
                    frequency = EXCLUDED.frequency,
                    conditions = EXCLUDED.conditions,
                    common_errors = EXCLUDED.common_errors
                "#
            )
            .bind(pattern.id)
            .bind(serde_json::to_string(&pattern.pattern_type).unwrap_or_default())
            .bind(pattern.failure_rate)
            .bind(pattern.frequency as i64)
            .bind(serde_json::to_value(&pattern.conditions).ok())
            .bind(&pattern.common_errors)
            .bind(pattern.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store failure pattern")?;
        }

        info!("Successfully stored {} failure patterns", patterns.len());
        Ok(())
    }

    async fn get_failure_patterns(&self) -> Result<Vec<FailurePattern>> {
        debug!("Retrieving all failure patterns from database");

        #[derive(Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
        struct FailurePatternRow {
            id: Uuid,
            pattern_type: String,
            failure_rate: f64,
            frequency: i64,
            conditions: serde_json::Value,
            common_errors: Vec<String>,
            #[schemars(with = "String")]
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, FailurePatternRow>(
            r#"
            SELECT id, pattern_type, failure_rate, frequency, conditions, common_errors, created_at
            FROM failure_patterns
            ORDER BY failure_rate DESC, frequency DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve failure patterns")?;

        let patterns: Vec<FailurePattern> = rows
            .into_iter()
            .filter_map(|row| {
                // Deserialize pattern_type from JSON string
                let pattern_type: PatternType = serde_json::from_str(&row.pattern_type).ok()?;

                // Convert conditions from JSON Value to HashMap
                let conditions: HashMap<String, serde_json::Value> = row
                    .conditions
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                Some(FailurePattern {
                    id: row.id,
                    pattern_type,
                    conditions,
                    failure_rate: row.failure_rate,
                    common_errors: row.common_errors,
                    frequency: row.frequency as u64,
                    created_at: row.created_at,
                })
            })
            .collect();

        debug!("Retrieved {} failure patterns", patterns.len());
        Ok(patterns)
    }

    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> Result<()> {
        if configs.is_empty() {
            return Ok(());
        }

        debug!("Storing {} optimal configs to database", configs.len());

        for config in &configs {
            sqlx::query(
                r#"
                INSERT INTO optimal_configs (id, config_type, parameters, performance_metrics, conditions, confidence, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    parameters = EXCLUDED.parameters,
                    performance_metrics = EXCLUDED.performance_metrics,
                    conditions = EXCLUDED.conditions,
                    confidence = EXCLUDED.confidence
                "#
            )
            .bind(config.id)
            .bind(serde_json::to_string(&config.config_type).unwrap_or_default())
            .bind(serde_json::to_value(&config.parameters).ok())
            .bind(serde_json::to_value(&config.performance_metrics).ok())
            .bind(serde_json::to_value(&config.conditions).ok())
            .bind(config.confidence)
            .bind(config.created_at)
            .execute(&self.pool)
            .await
            .context("Failed to store optimal config")?;
        }

        info!("Successfully stored {} optimal configs", configs.len());
        Ok(())
    }

    async fn get_optimal_configs(&self) -> Result<Vec<OptimalConfig>> {
        debug!("Retrieving all optimal configs from database");

        #[derive(Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
        struct OptimalConfigRow {
            id: Uuid,
            config_type: String,
            parameters: serde_json::Value,
            performance_metrics: serde_json::Value,
            conditions: serde_json::Value,
            confidence: f64,
            #[schemars(with = "String")]
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, OptimalConfigRow>(
            r#"
            SELECT id, config_type, parameters, performance_metrics, conditions, confidence, created_at
            FROM optimal_configs
            ORDER BY confidence DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve optimal configs")?;

        let configs: Vec<OptimalConfig> = rows
            .into_iter()
            .filter_map(|row| {
                // Deserialize config_type from JSON string
                let config_type: ConfigType = serde_json::from_str(&row.config_type).ok()?;

                // Convert parameters and conditions from JSON Value to HashMap
                let parameters: HashMap<String, serde_json::Value> = row
                    .parameters
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let conditions: HashMap<String, serde_json::Value> = row
                    .conditions
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                // Deserialize performance_metrics from JSON
                let performance_metrics: PerformanceMetrics =
                    serde_json::from_value(row.performance_metrics).unwrap_or_else(|_| {
                        PerformanceMetrics {
                            execution_time_ms: 0.0,
                            quality_score: 0.0,
                            success_rate: 0.0,
                            resource_utilization: 0.0,
                            cost_score: 0.0,
                        }
                    });

                Some(OptimalConfig {
                    id: row.id,
                    config_type,
                    worker_type: "general".to_string(), // Default worker type
                    task_type: "general".to_string(),   // Default task type
                    config: serde_json::Value::Object(serde_json::Map::new()), // Empty config
                    parameters,
                    conditions,
                    performance_metrics,
                    confidence: row.confidence,
                    expires_at: None,
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    created_at: row.created_at,
                })
            })
            .collect();

        debug!("Retrieved {} optimal configs", configs.len());
        Ok(configs)
    }

    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        debug!("Storing {} optimization events to database", events.len());

        for event in &events {
            sqlx::query(
                r#"
                INSERT INTO optimization_events (id, event_type, config_id, performance_delta, timestamp, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(event.id)
            .bind(serde_json::to_string(&event.event_type).unwrap_or_default())
            .bind(event.config_id)
            .bind(serde_json::to_value(&event.performance_delta).ok())
            .bind(event.timestamp)
            .bind(serde_json::to_value(&event.metadata).ok())
            .execute(&self.pool)
            .await
            .context("Failed to store optimization event")?;
        }

        info!("Successfully stored {} optimization events", events.len());
        Ok(())
    }

    async fn get_optimization_events(&self, config_id: &Uuid) -> Result<Vec<OptimizationEvent>> {
        debug!("Retrieving optimization events for config: {}", config_id);

        #[derive(Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
        struct OptimizationEventRow {
            id: Uuid,
            event_type: String,
            config_id: Uuid,
            performance_delta: serde_json::Value,
            #[schemars(with = "String")]
            timestamp: DateTime<Utc>,
            metadata: serde_json::Value,
        }

        let rows = sqlx::query_as::<_, OptimizationEventRow>(
            r#"
            SELECT id, event_type, config_id, performance_delta, timestamp, metadata
            FROM optimization_events
            WHERE config_id = $1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(config_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to retrieve optimization events")?;

        let events: Vec<OptimizationEvent> = rows
            .into_iter()
            .filter_map(|row| {
                // Deserialize event_type from JSON string
                let event_type: OptimizationEventType =
                    serde_json::from_str(&row.event_type).ok()?;

                // Convert metadata from JSON Value to HashMap
                let metadata: HashMap<String, serde_json::Value> = row
                    .metadata
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                // Deserialize performance_delta from JSON
                let performance_delta: PerformanceMetrics =
                    serde_json::from_value(row.performance_delta).unwrap_or_else(|_| {
                        PerformanceMetrics {
                            execution_time_ms: 0.0,
                            quality_score: 0.0,
                            success_rate: 0.0,
                            resource_utilization: 0.0,
                            cost_score: 0.0,
                        }
                    });

                Some(OptimizationEvent {
                    id: row.id,
                    event_type,
                    config_id: row.config_id,
                    performance_delta,
                    timestamp: row.timestamp,
                    metadata,
                    config_before: None,
                    config_after: None,
                    performance_improvement: None,
                    optimization_type: None,
                })
            })
            .collect();

        debug!("Retrieved {} optimization events", events.len());
        Ok(events)
    }
}
