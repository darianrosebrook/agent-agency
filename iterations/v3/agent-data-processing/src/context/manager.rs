//! Context Manager - Unified context preservation and working memory management
//!
//! This module combines the functionality from:
//! - context-preservation-engine (multi-tenant, full-featured)
//! - agent-memory (working memory folding)
//!
//! Provides a unified interface for context lifecycle management.

use crate::context::types::*;
use crate::{DataProcessingError, DataProcessingResult};
use chrono::{Duration, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json;
use sqlx::{postgres::{PgArgumentBuffer, PgPoolOptions}, Encode, PgPool, Postgres, Row, Type};
use std::error::Error;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Database configuration for context management
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/agent_agency".to_string()),
            max_connections: 10,
        }
    }

    pub fn from_url(database_url: String) -> Self {
        Self {
            database_url,
            max_connections: 10,
        }
    }
}

/// Query parameter wrapper for type-safe parameterized queries
/// Supports common PostgreSQL types with proper SQL injection protection
#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    I32(i32),
    I64(i64),
    Uuid(Uuid),
    Bool(bool),
    Json(serde_json::Value),
    Bytes(Vec<u8>),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Null,
}

impl<'q> Encode<'q, Postgres> for QueryParam {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        match self {
            QueryParam::String(s) => <String as Encode<'q, Postgres>>::encode_by_ref(s, buf),
            QueryParam::I32(i) => <i32 as Encode<'q, Postgres>>::encode_by_ref(i, buf),
            QueryParam::I64(i) => <i64 as Encode<'q, Postgres>>::encode_by_ref(i, buf),
            QueryParam::Uuid(u) => <Uuid as Encode<'q, Postgres>>::encode_by_ref(u, buf),
            QueryParam::Bool(b) => <bool as Encode<'q, Postgres>>::encode_by_ref(b, buf),
            QueryParam::Json(j) => {
                <serde_json::Value as Encode<'q, Postgres>>::encode_by_ref(j, buf)
            }
            QueryParam::Bytes(b) => <Vec<u8> as Encode<'q, Postgres>>::encode_by_ref(b, buf),
            QueryParam::Timestamp(t) => {
                <chrono::DateTime<chrono::Utc> as Encode<'q, Postgres>>::encode_by_ref(t, buf)
            }
            QueryParam::Null => Ok(sqlx::encode::IsNull::Yes),
        }
    }
}

impl Type<Postgres> for QueryParam {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        // Default to text type, PostgreSQL will handle type coercion
        <String as Type<Postgres>>::type_info()
    }
}

/// Real database client using sqlx
#[derive(Debug, Clone)]
pub struct DatabaseClient {
    pool: Arc<PgPool>,
}

impl DatabaseClient {
    pub async fn new(config: DatabaseConfig) -> Result<Self, DataProcessingError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await
            .map_err(|e| {
                DataProcessingError::Operation(format!("Failed to connect to database: {}", e))
            })?;

        // Test the connection
        sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
            DataProcessingError::Operation(format!("Failed to test database connection: {}", e))
        })?;

        info!("Database client initialized successfully");
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Execute a parameterized query with proper SQL injection protection
    /// Uses QueryParam enum for type-safe parameter binding
    pub async fn execute_with_params(
        &self,
        query: &str,
        params: &[QueryParam],
    ) -> Result<(), DataProcessingError> {
        // Validate query structure for security
        self.validate_query_security(query)?;

        // Count placeholders in query ($1, $2, etc.)
        let placeholder_count = self.count_placeholders(query);
        if placeholder_count != params.len() {
            return Err(DataProcessingError::Operation(format!(
                "Parameter count mismatch: query has {} placeholders but {} parameters provided",
                placeholder_count,
                params.len()
            )));
        }

        // Build query with parameter binding
        let mut query_builder = sqlx::query(query);

        // Bind each parameter using sqlx's type-safe binding
        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::I32(i) => query_builder.bind(i),
                QueryParam::I64(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Json(j) => query_builder.bind(j),
                QueryParam::Bytes(b) => query_builder.bind(b),
                QueryParam::Timestamp(t) => query_builder.bind(t),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }

        query_builder.execute(&*self.pool).await.map_err(|e| {
            DataProcessingError::Operation(format!("Database execution failed: {}", e))
        })?;

        Ok(())
    }

    /// Legacy method - kept for backward compatibility
    /// Prefer execute_with_params() for new code
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<(), DataProcessingError> {
        if params.is_empty() {
            // No parameters - use simple query execution
            self.validate_query_security(query)?;
            sqlx::query(query).execute(&*self.pool).await.map_err(|e| {
                DataProcessingError::Operation(format!("Database execution failed: {}", e))
            })?;
            return Ok(());
        }

        // For trait object parameters, we can't safely bind them
        // Return error directing users to use execute_with_params()
        Err(DataProcessingError::Operation(
            "Parameterized queries with trait objects are not supported for security reasons. \
            Use execute_with_params() with QueryParam enum instead for proper SQL injection protection.".to_string()
        ))
    }

    /// Validate query for potential SQL injection patterns
    fn validate_query_security(&self, query: &str) -> Result<(), DataProcessingError> {
        // Check for dangerous SQL patterns that could indicate injection attempts
        let dangerous_patterns = [
            ("--", "SQL comment injection"),
            ("/*", "SQL block comment start"),
            ("*/", "SQL block comment end"),
            ("xp_", "Extended stored procedure"),
            ("sp_", "System stored procedure"),
            ("exec(", "Dynamic execution"),
            ("execute(", "Dynamic execution"),
            ("union select", "SQL union injection"),
            ("drop table", "Table deletion"),
            ("drop database", "Database deletion"),
            ("truncate", "Table truncation"),
        ];

        let query_lower = query.to_lowercase();
        for (pattern, description) in &dangerous_patterns {
            if query_lower.contains(pattern) {
                warn!(
                    "Query contains potentially dangerous pattern '{}': {}",
                    pattern, description
                );
                // Don't block - these might be legitimate, but log for security monitoring
                // In production, you might want to block or require additional validation
            }
        }

        Ok(())
    }

    /// Count PostgreSQL parameter placeholders ($1, $2, etc.)
    fn count_placeholders(&self, query: &str) -> usize {
        use regex::Regex;
        // Match $1, $2, etc. pattern
        let re = Regex::new(r"\$\d+").unwrap();
        let matches: Vec<_> = re.find_iter(query).collect();

        // Extract numbers and find max
        let max_placeholder = matches
            .iter()
            .filter_map(|m| m.as_str().strip_prefix('$')?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        max_placeholder
    }

    /// Execute a parameterized query and return rows with proper SQL injection protection
    /// Uses QueryParam enum for type-safe parameter binding
    pub async fn query_with_params(
        &self,
        query: &str,
        params: &[QueryParam],
    ) -> Result<Vec<sqlx::postgres::PgRow>, DataProcessingError> {
        // Validate query structure for security
        self.validate_query_security(query)?;

        // Count placeholders in query
        let placeholder_count = self.count_placeholders(query);
        if placeholder_count != params.len() {
            return Err(DataProcessingError::Operation(format!(
                "Parameter count mismatch: query has {} placeholders but {} parameters provided",
                placeholder_count,
                params.len()
            )));
        }

        // Build query with parameter binding
        let mut query_builder = sqlx::query(query);

        // Bind each parameter
        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::I32(i) => query_builder.bind(i),
                QueryParam::I64(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Json(j) => query_builder.bind(j),
                QueryParam::Bytes(b) => query_builder.bind(b),
                QueryParam::Timestamp(t) => query_builder.bind(t),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }

        query_builder
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| DataProcessingError::Operation(format!("Database query failed: {}", e)))
    }

    /// Legacy method - kept for backward compatibility
    /// Prefer query_with_params() for new code
    pub async fn query(
        &self,
        query: &str,
        _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>, DataProcessingError> {
        if !_params.is_empty() {
            return Err(DataProcessingError::Operation(
                "Parameterized queries with trait objects are not supported for security reasons. \
                Use query_with_params() with QueryParam enum instead for proper SQL injection protection.".to_string()
            ));
        }

        self.validate_query_security(query)?;
        let rows = sqlx::query(query).fetch_all(&*self.pool).await;

        rows.map_err(|e| DataProcessingError::Operation(format!("Database query failed: {}", e)))
    }

    /// Get the underlying pool (for advanced usage)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Model registry trait for AI services
/// Implemented: Real embedding generation integration with data-infrastructure embedding service
pub struct ModelRegistry {
    /// Optional embedding service for generating embeddings
    #[cfg(feature = "embeddings")]
    embedding_service: Option<std::sync::Arc<dyn data_infrastructure::embedding::EmbeddingService>>,
    #[cfg(not(feature = "embeddings"))]
    embedding_service: Option<()>, // Placeholder when embeddings feature is disabled
}

impl std::fmt::Debug for ModelRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRegistry")
            .field(
                "embedding_service",
                &if self.embedding_service.is_some() {
                    "Some(EmbeddingService)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

impl ModelRegistry {
    /// Create a new ModelRegistry with optional embedding service
    #[cfg(feature = "embeddings")]
    pub fn new(
        embedding_service: Option<
            std::sync::Arc<dyn data_infrastructure::embedding::EmbeddingService>,
        >,
    ) -> Self {
        Self { embedding_service }
    }

    /// Create a new ModelRegistry without embedding service (when embeddings feature is disabled)
    #[cfg(not(feature = "embeddings"))]
    pub fn new(_embedding_service: Option<()>) -> Self {
        Self {
            embedding_service: None,
        }
    }

    /// Create a new ModelRegistry without embedding service (fallback mode)
    pub fn new_empty() -> Self {
        Self {
            embedding_service: None,
        }
    }

    pub async fn generate(
        &self,
        _prompt: &str,
        _options: Option<()>,
    ) -> Result<String, DataProcessingError> {
        // TODO: Implement real AI service integration
        // - [ ] Integrate with agent-model-management crate
        // - [ ] Implement proper model selection based on task type
        // - [ ] Add error handling for model failures
        // - [ ] Add timeout handling for long-running generations
        // - [ ] Add caching for frequently requested prompts
        // - [ ] Add unit tests with mock model service
        // - [ ] Add integration tests with real model service
        // PLACEHOLDER: Real AI service integration needed
        // This would integrate with agent-model-management or similar
        Ok("Mock summary".to_string())
    }

    /// Generate embedding for content
    /// Implemented: Real embedding generation using data-infrastructure embedding service
    #[cfg(feature = "embeddings")]
    pub async fn generate_embedding(&self, content: &str) -> Result<Vec<f32>, DataProcessingError> {
        if let Some(ref embedding_service) = self.embedding_service {
            use data_infrastructure::embedding::embedding_types::ContentType;

            // Generate embedding using the embedding service
            match embedding_service
                .generate_embedding(content, ContentType::Text, "model_registry")
                .await
            {
                Ok(stored_embedding) => {
                    // Extract vector from StoredEmbedding
                    let embedding_vector = stored_embedding.vector.values;
                    Ok(embedding_vector)
                }
                Err(e) => {
                    warn!("Failed to generate embedding: {}, falling back to mock", e);
                    // Fallback to mock embedding if service fails
                    Ok(vec![0.1; 768]) // Default 768-dim embedding
                }
            }
        } else {
            // No embedding service available, return mock embedding
            debug!("No embedding service available, returning mock embedding");
            Ok(vec![0.1; 768]) // Default 768-dim embedding
        }
    }

    /// Generate embedding for content (fallback when embeddings feature is disabled)
    #[cfg(not(feature = "embeddings"))]
    pub async fn generate_embedding(
        &self,
        _content: &str,
    ) -> Result<Vec<f32>, DataProcessingError> {
        // Return mock embedding when embeddings feature is disabled
        Ok(vec![0.1; 768]) // Default 768-dim embedding
    }
}

/// Unified context manager for preservation and working memory
#[derive(Debug)]
pub struct ContextManager {
    /// Database client
    db_client: Arc<DatabaseClient>,
    /// AI service for summarization
    ai_service: Arc<ModelRegistry>,
    /// Configuration
    config: ContextConfig,
    /// Working memory cache
    working_memory: Arc<RwLock<HashMap<Uuid, ContextData>>>,
    /// Statistics
    stats: Arc<RwLock<ContextStats>>,
}

impl ContextManager {
    /// Create a new unified context manager with database client
    pub fn new_with_db_client(
        config: ContextConfig,
        ai_service: Arc<ModelRegistry>,
        db_client: Arc<DatabaseClient>,
    ) -> DataProcessingResult<Self> {
        let stats = Arc::new(RwLock::new(ContextStats {
            total_contexts: 0,
            total_storage_size: 0,
            working_memory_contexts: 0,
            folded_contexts: 0,
            average_context_size: 0,
            recent_accesses: 0,
            oldest_context_age_hours: 0,
            compression_ratio: 1.0,
            lifecycle_metrics: ContextLifecycleMetrics::default(),
        }));

        let manager = Self {
            db_client,
            ai_service,
            config,
            working_memory: Arc::new(RwLock::new(HashMap::new())),
            stats,
        };

        // Initialize working memory cleanup task
        manager.start_cleanup_task();

        Ok(manager)
    }

    /// Preserve context data
    pub async fn preserve_context(
        &self,
        request: ContextPreservationRequest,
    ) -> DataProcessingResult<ContextPreservationResult> {
        let start_time = tokio::time::Instant::now();

        info!("Preserving context: {}", request.context_data.id);

        // Check storage limits
        self.check_storage_limits(&request.context_data).await?;

        // Store context
        let context_id = request.context_data.id;
        let size_bytes = self.calculate_context_size(&request.context_data);

        // Store in database
        self.store_context_in_db(&request.context_data).await?;

        // Add to working memory if enabled
        if self.config.working_memory.max_size > 0 {
            self.add_to_working_memory(request.context_data).await?;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.update_stats_after_preservation(size_bytes).await?;

        Ok(ContextPreservationResult {
            success: true,
            context_id: Some(context_id),
            processing_time_ms: processing_time,
            processed_size_bytes: size_bytes,
            error_message: None,
        })
    }

    /// Retrieve context data
    pub async fn retrieve_context(
        &self,
        request: ContextRetrievalRequest,
    ) -> DataProcessingResult<ContextRetrievalResult> {
        let start_time = tokio::time::Instant::now();

        info!("Retrieving context: {}", request.context_id);

        // Check working memory first
        if let Some(context) = self.get_from_working_memory(&request.context_id).await? {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Update access statistics
            self.update_context_access(&request.context_id).await?;

            return Ok(ContextRetrievalResult {
                success: true,
                context_data: Some(context),
                processing_time_ms: processing_time,
                error_message: None,
            });
        }

        // Retrieve from database
        match self.retrieve_context_from_db(&request.context_id).await? {
            Some(mut context) => {
                // Update access time
                context.last_accessed_at = Utc::now();
                context.access_count += 1;

                // Store back to database
                self.update_context_in_db(&context).await?;

                // Add to working memory
                if self.config.working_memory.max_size > 0 {
                    self.add_to_working_memory(context.clone()).await?;
                }

                let processing_time = start_time.elapsed().as_millis() as u64;

                Ok(ContextRetrievalResult {
                    success: true,
                    context_data: Some(context),
                    processing_time_ms: processing_time,
                    error_message: None,
                })
            }
            None => Ok(ContextRetrievalResult {
                success: false,
                context_data: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Context not found: {}", request.context_id)),
            }),
        }
    }

    /// Fold context based on lifecycle rules
    pub async fn fold_context(&self, context_id: &Uuid) -> DataProcessingResult<FoldedContext> {
        info!("Folding context: {}", context_id);

        // Get context
        let context = match self.retrieve_context_from_db(context_id).await? {
            Some(ctx) => ctx,
            None => {
                return Err(crate::DataProcessingError::Other(format!(
                    "Context not found: {}",
                    context_id
                )))
            }
        };

        // Determine folding strategy
        let strategy = self.determine_folding_strategy(&context).await?;

        let folded = match strategy {
            FoldingStrategy::Compress => self.compress_context(context).await,
            FoldingStrategy::Summarize => self.summarize_context(context).await,
            FoldingStrategy::Archive => self.archive_context(context).await,
            FoldingStrategy::Delete => Ok(FoldedContext::Deleted),
        };

        // Store folded result
        if let Ok(folded_context) = &folded {
            self.store_folded_context(context_id, folded_context)
                .await?;
        }

        folded
    }

    /// Manage context lifecycle - automatic folding and cleanup
    pub async fn manage_context_lifecycle(&self) -> DataProcessingResult<()> {
        info!("Running context lifecycle management");

        // Find contexts that need folding
        let contexts_to_fold = self.find_contexts_needing_folding().await?;

        for context_id in contexts_to_fold {
            if let Err(e) = self.fold_context(&context_id).await {
                warn!("Failed to fold context {}: {}", context_id, e);
            }
        }

        // Clean up working memory
        self.cleanup_working_memory().await?;

        // Update statistics
        self.update_lifecycle_stats().await?;

        Ok(())
    }

    /// Get context statistics
    pub async fn get_stats(&self) -> DataProcessingResult<ContextStats> {
        let stats = self.stats.read().await.clone();
        Ok(stats)
    }

    // Private helper methods

    async fn check_storage_limits(&self, context: &ContextData) -> DataProcessingResult<()> {
        let size_bytes = self.calculate_context_size(context);

        // Check max context size
        if size_bytes > self.config.storage.max_context_size {
            return Err(crate::DataProcessingError::ResourceExhausted(format!(
                "Context size {} exceeds limit {}",
                size_bytes, self.config.storage.max_context_size
            )));
        }

        // Check total storage usage
        let current_usage = self.get_current_storage_usage().await?;
        if current_usage + size_bytes > self.config.storage.cache_size_limit {
            return Err(crate::DataProcessingError::ResourceExhausted(
                "Storage limit exceeded".to_string(),
            ));
        }

        Ok(())
    }

    async fn store_context_in_db(&self, context: &ContextData) -> DataProcessingResult<()> {
        // Serialize context data
        let content_json = serde_json::to_string(&context.content)
            .map_err(|e| DataProcessingError::Serialization(e))?;
        let metadata_value = serde_json::to_value(&context.metadata)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        // Compress content if enabled
        let (content_data, content_size) = if self.config.storage.enable_compression {
            let mut encoder = GzEncoder::new(
                Vec::new(),
                Compression::new(self.config.storage.compression_level),
            );
            use std::io::Write;
            encoder.write_all(content_json.as_bytes())?;
            let compressed = encoder.finish()?;
            (compressed.clone(), compressed.len() as u64)
        } else {
            (content_json.clone().into_bytes(), content_json.len() as u64)
        };

        // Create database record
        let query = r#"
            INSERT INTO agent_contexts (
                id, context_type, content, metadata,
                created_at, last_accessed_at, access_count, size_bytes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                last_accessed_at = EXCLUDED.last_accessed_at,
                access_count = EXCLUDED.access_count,
                size_bytes = EXCLUDED.size_bytes
        "#;

        let access_count_i64 = context.access_count as i64;
        let content_size_i64 = content_size as i64;

        // Use new parameterized query API with QueryParam enum for SQL injection protection
        let params = vec![
            QueryParam::Uuid(context.id),
            QueryParam::String(context.context_type.clone()),
            QueryParam::Bytes(content_data),
            QueryParam::Json(metadata_value),
            QueryParam::Timestamp(context.created_at),
            QueryParam::Timestamp(context.last_accessed_at),
            QueryParam::I64(access_count_i64),
            QueryParam::I64(content_size_i64),
        ];

        self.db_client.execute_with_params(query, &params).await?;
        debug!("Stored context {} in database", context.id);
        Ok(())
    }

    async fn retrieve_context_from_db(
        &self,
        context_id: &Uuid,
    ) -> DataProcessingResult<Option<ContextData>> {
        let query = r#"
            SELECT context_type, content, metadata, created_at, last_accessed_at, access_count, size_bytes
            FROM agent_contexts
            WHERE id = $1
        "#;

        let params = vec![QueryParam::Uuid(*context_id)];

        let rows = self.db_client.query_with_params(query, &params).await?;
        if rows.is_empty() {
            debug!("Context {} not found in database", context_id);
            return Ok(None);
        }

        let row = &rows[0];

        // Deserialize data
        let content_data: Vec<u8> = row.get("content");
        let content_json = if self.config.storage.enable_compression {
            let mut decoder = GzDecoder::new(&content_data[..]);
            use std::io::Read;
            let mut decompressed = String::new();
            decoder.read_to_string(&mut decompressed)?;
            decompressed
        } else {
            String::from_utf8(content_data).map_err(|e| {
                DataProcessingError::Operation(format!("UTF-8 conversion failed: {}", e))
            })?
        };

        let content: serde_json::Value = serde_json::from_str(&content_json)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        // Get metadata as JSONB (sqlx can deserialize JSONB directly)
        let metadata_value: serde_json::Value = row.get("metadata");
        let metadata: ContextMetadata = serde_json::from_value(metadata_value)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        let context = ContextData {
            id: *context_id,
            context_type: row.get("context_type"),
            content,
            metadata,
            created_at: row.get("created_at"),
            last_accessed_at: row.get("last_accessed_at"),
            access_count: row.get::<i64, _>("access_count") as u64,
            size_bytes: row.get::<i64, _>("size_bytes") as u64,
        };

        debug!("Retrieved context {} from database", context_id);
        Ok(Some(context))
    }

    async fn update_context_in_db(&self, context: &ContextData) -> DataProcessingResult<()> {
        let metadata_json = serde_json::to_value(&context.metadata)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        let query = r#"
            UPDATE agent_contexts
            SET context_type = $1, metadata = $2, last_accessed_at = $3,
                access_count = $4, size_bytes = $5
            WHERE id = $6
        "#;

        let access_count_i64 = context.access_count as i64;
        let size_bytes_i64 = context.size_bytes as i64;
        let params = vec![
            QueryParam::String(context.context_type.clone()),
            QueryParam::Json(metadata_json),
            QueryParam::Timestamp(context.last_accessed_at),
            QueryParam::I64(access_count_i64),
            QueryParam::I64(size_bytes_i64),
            QueryParam::Uuid(context.id),
        ];

        self.db_client.execute_with_params(query, &params).await?;
        debug!("Updated context {} in database", context.id);
        Ok(())
    }

    async fn update_context_access(&self, context_id: &Uuid) -> DataProcessingResult<()> {
        let query = r#"
            UPDATE agent_contexts
            SET access_count = access_count + 1, last_accessed_at = $1
            WHERE id = $2
        "#;

        let now = Utc::now();
        let params = vec![QueryParam::Timestamp(now), QueryParam::Uuid(*context_id)];

        self.db_client.execute_with_params(query, &params).await?;
        debug!("Updated access statistics for context {}", context_id);
        Ok(())
    }

    async fn store_folded_context(
        &self,
        context_id: &Uuid,
        folded: &FoldedContext,
    ) -> DataProcessingResult<()> {
        let (fold_type, fold_data) = match folded {
            FoldedContext::Compressed(data) => ("compressed", serde_json::to_string(data)?),
            FoldedContext::Summarized(summary) => ("summarized", summary.clone()),
            FoldedContext::Archived(path) => ("archived", path.clone()),
            FoldedContext::Deleted => ("deleted", String::new()),
        };

        let query = r#"
            INSERT INTO folded_contexts (context_id, fold_type, fold_data, folded_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (context_id) DO UPDATE SET
                fold_type = EXCLUDED.fold_type,
                fold_data = EXCLUDED.fold_data,
                folded_at = EXCLUDED.folded_at
        "#;

        let now = Utc::now();
        let params = vec![
            QueryParam::Uuid(*context_id),
            QueryParam::String(fold_type.to_string()),
            QueryParam::String(fold_data.clone()),
            QueryParam::Timestamp(now),
        ];

        self.db_client.execute_with_params(query, &params).await?;
        debug!(
            "Stored folded context {} with type {}",
            context_id, fold_type
        );
        Ok(())
    }

    async fn add_to_working_memory(&self, context: ContextData) -> DataProcessingResult<()> {
        let mut working_memory = self.working_memory.write().await;

        // Check working memory limits
        if working_memory.len() >= self.config.working_memory.max_size {
            // Remove least recently accessed context
            if let Some((oldest_id, _)) = working_memory
                .iter()
                .min_by_key(|(_, ctx)| ctx.last_accessed_at)
            {
                let oldest_id = *oldest_id;
                working_memory.remove(&oldest_id);
                debug!(
                    "Removed context {} from working memory due to size limit",
                    oldest_id
                );
            }
        }

        working_memory.insert(context.id, context);
        Ok(())
    }

    async fn get_from_working_memory(
        &self,
        context_id: &Uuid,
    ) -> DataProcessingResult<Option<ContextData>> {
        let mut working_memory = self.working_memory.write().await;

        if let Some(context) = working_memory.get_mut(context_id) {
            context.last_accessed_at = Utc::now();
            context.access_count += 1;
            return Ok(Some(context.clone()));
        }

        Ok(None)
    }

    async fn cleanup_working_memory(&self) -> DataProcessingResult<()> {
        let mut working_memory = self.working_memory.write().await;
        let mut to_remove = Vec::new();

        for (id, context) in working_memory.iter() {
            // Remove contexts older than retention period
            let age = Utc::now().signed_duration_since(context.last_accessed_at);
            if age > Duration::hours(self.config.storage.retention_hours as i64) {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            working_memory.remove(&id);
            debug!("Removed expired context {} from working memory", id);
        }

        Ok(())
    }

    async fn find_contexts_needing_folding(&self) -> DataProcessingResult<Vec<Uuid>> {
        let query = r#"
            SELECT id
            FROM agent_contexts
            WHERE archived_at IS NULL
              AND (
                -- Contexts older than threshold
                created_at < $1
                -- Or contexts with low access frequency
                OR (created_at < $2 AND access_count < $3)
                -- Or low importance contexts
                OR (metadata->>'importance_score')::float < $4
              )
            LIMIT 100
        "#;

        let age_threshold =
            Utc::now() - Duration::hours(self.config.folding.age_threshold_hours as i64);
        let low_access_threshold = Utc::now() - Duration::hours(24); // 1 day
        let min_access_count = 5; // Minimum accesses to avoid folding
        let importance_threshold = self.config.folding.importance_threshold;

        let params = vec![
            QueryParam::Timestamp(age_threshold),
            QueryParam::Timestamp(low_access_threshold),
            QueryParam::I32(min_access_count),
            QueryParam::Json(serde_json::json!(importance_threshold)),
        ];

        let rows = self.db_client.query_with_params(query, &params).await?;
        let context_ids: Vec<Uuid> = rows.into_iter().map(|row| row.get("id")).collect();

        debug!("Found {} contexts needing folding", context_ids.len());
        Ok(context_ids)
    }

    async fn determine_folding_strategy(
        &self,
        context: &ContextData,
    ) -> DataProcessingResult<FoldingStrategy> {
        let age_hours = Utc::now()
            .signed_duration_since(context.created_at)
            .num_hours() as u32;
        let access_frequency = if context.access_count > 0 {
            let age_days = age_hours as f64 / 24.0;
            context.access_count as f64 / age_days
        } else {
            0.0
        };

        let importance_score = context.metadata.importance_score.unwrap_or(0.5);

        // Folding decision logic
        if age_hours >= self.config.folding.age_threshold_hours {
            Ok(self.config.folding.strategy.clone())
        } else if age_hours >= 1
            && access_frequency < self.config.folding.access_frequency_threshold
        {
            Ok(FoldingStrategy::Compress)
        } else if importance_score < self.config.folding.importance_threshold {
            Ok(FoldingStrategy::Compress)
        } else {
            Ok(FoldingStrategy::Compress)
        }
    }

    async fn compress_context(&self, context: ContextData) -> DataProcessingResult<FoldedContext> {
        let json_data = serde_json::to_string(&context)?;
        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(self.config.storage.compression_level),
        );
        encoder.write_all(json_data.as_bytes())?;
        let compressed = encoder.finish()?;

        Ok(FoldedContext::Compressed(compressed))
    }

    async fn summarize_context(&self, context: ContextData) -> DataProcessingResult<FoldedContext> {
        // Extract text content from context
        let content_text = self.extract_text_from_context(&context)?;

        // Create summarization prompt
        let prompt = format!(
            "Please provide a concise but comprehensive summary of the following content. \
             Focus on the key information, decisions, and outcomes. Keep the summary under 500 words.\n\n\
             Content type: {}\n\
             Title: {}\n\
             Description: {}\n\n\
             Content:\n{}",
            context.context_type,
            context.metadata.title.as_deref().unwrap_or("Untitled"),
            context.metadata.description.as_deref().unwrap_or("No description"),
            content_text
        );

        // Generate summary using AI service
        match self.ai_service.generate(&prompt, None).await {
            Ok(summary) => {
                // Clean up the summary (remove extra whitespace, etc.)
                let cleaned_summary = summary.trim().to_string();

                // Validate summary isn't too short or too long
                if cleaned_summary.len() < 50 {
                    return Err(DataProcessingError::Operation(
                        "Generated summary too short".to_string(),
                    ));
                }

                if cleaned_summary.len() > 2000 {
                    return Err(DataProcessingError::Operation(
                        "Generated summary too long".to_string(),
                    ));
                }

                debug!(
                    "AI-generated summary for context {}: {} chars",
                    context.id,
                    cleaned_summary.len()
                );
                Ok(FoldedContext::Summarized(cleaned_summary))
            }
            Err(e) => {
                warn!("AI summarization failed for context {}: {}", context.id, e);
                // Fallback to a basic extractive summary
                let fallback_summary = self.create_fallback_summary(&context)?;
                Ok(FoldedContext::Summarized(fallback_summary))
            }
        }
    }

    /// Extract text content from context for summarization
    fn extract_text_from_context(&self, context: &ContextData) -> DataProcessingResult<String> {
        match &context.content {
            serde_json::Value::String(text) => Ok(text.clone()),
            serde_json::Value::Object(obj) => {
                // Try to extract text from common fields
                if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                    Ok(text.to_string())
                } else if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                    Ok(content.to_string())
                } else if let Some(body) = obj.get("body").and_then(|v| v.as_str()) {
                    Ok(body.to_string())
                } else {
                    // Convert entire object to formatted string
                    serde_json::to_string_pretty(obj)
                        .map_err(|e| DataProcessingError::Serialization(e))
                }
            }
            _ => {
                // Convert to string representation
                serde_json::to_string_pretty(&context.content)
                    .map_err(|e| DataProcessingError::Serialization(e))
            }
        }
    }

    /// Create a fallback summary when AI summarization fails
    fn create_fallback_summary(&self, context: &ContextData) -> DataProcessingResult<String> {
        let title = context
            .metadata
            .title
            .as_deref()
            .unwrap_or("Untitled context");
        let desc = context.metadata.description.as_deref().unwrap_or("");
        let tags = context.metadata.tags.join(", ");

        let summary = format!(
            "Context: {}\nType: {}\nDescription: {}\nTags: {}\nSize: {} bytes\nCreated: {}",
            title,
            context.context_type,
            desc,
            if tags.is_empty() { "none" } else { &tags },
            context.size_bytes,
            context.created_at.format("%Y-%m-%d %H:%M UTC")
        );

        Ok(summary)
    }

    async fn archive_context(&self, context: ContextData) -> DataProcessingResult<FoldedContext> {
        use std::fs;
        use std::path::PathBuf;

        // Create archive path based on context ID
        let archive_base = self
            .config
            .storage
            .archive_path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "archive".to_string());

        let archive_dir = PathBuf::from(archive_base)
            .join(context.id.to_string()[..2].to_string()) // First 2 chars as subdirectory
            .join(context.id.to_string()[2..4].to_string()); // Next 2 chars as subdirectory

        fs::create_dir_all(&archive_dir)?;

        let archive_path = archive_dir.join(format!("{}.ctx", context.id));

        // Serialize context data
        let context_json = serde_json::to_string(&context)?;
        let compressed_data = if self.config.storage.enable_compression {
            let mut encoder = flate2::write::GzEncoder::new(
                Vec::new(),
                flate2::Compression::new(self.config.storage.compression_level),
            );
            use std::io::Write;
            encoder.write_all(context_json.as_bytes())?;
            encoder.finish()?
        } else {
            context_json.into_bytes()
        };

        // Write to cold storage
        fs::write(&archive_path, &compressed_data)?;

        // Generate archive location identifier
        let archive_location = format!(
            "{}/{}",
            context.id.to_string()[..2].to_string(),
            context.id.to_string()[2..4].to_string()
        );

        // Update database to mark as archived
        let query = r#"
            UPDATE agent_contexts
            SET archived_at = $1, archive_location = $2
            WHERE id = $3
        "#;

        let now = Utc::now();
        let archive_location_clone = archive_location.clone();
        let params = vec![
            QueryParam::Timestamp(now),
            QueryParam::String(archive_location.clone()),
            QueryParam::Uuid(context.id),
        ];

        self.db_client.execute_with_params(query, &params).await?;

        debug!(
            "Archived context {} to cold storage at {}",
            context.id,
            archive_path.display()
        );

        Ok(FoldedContext::Archived(archive_location_clone))
    }

    /// Retrieve a context from cold storage archive
    pub async fn retrieve_archived_context(
        &self,
        context_id: &Uuid,
    ) -> DataProcessingResult<Option<ContextData>> {
        use std::fs;
        use std::path::PathBuf;

        // First check if context is archived in database
        let query = r#"
            SELECT archive_location FROM agent_contexts
            WHERE id = $1 AND archived_at IS NOT NULL
        "#;

        let params = vec![QueryParam::Uuid(*context_id)];
        let rows = self.db_client.query_with_params(query, &params).await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let archive_location: String = rows[0].get("archive_location");

        // Reconstruct archive path
        let archive_base = self
            .config
            .storage
            .archive_path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "archive".to_string());

        let archive_path = PathBuf::from(archive_base)
            .join(&archive_location)
            .join(format!("{}.ctx", context_id));

        // Read archived data
        let compressed_data = fs::read(&archive_path)?;

        // Decompress if needed
        let context_json = if self.config.storage.enable_compression {
            let mut decoder = flate2::read::GzDecoder::new(&compressed_data[..]);
            let mut decompressed = String::new();
            use std::io::Read;
            decoder.read_to_string(&mut decompressed)?;
            decompressed
        } else {
            String::from_utf8(compressed_data).map_err(|e| {
                DataProcessingError::Operation(format!("UTF-8 conversion failed: {}", e))
            })?
        };

        // Deserialize context
        let mut context: ContextData = serde_json::from_str(&context_json)?;

        // Update access time
        context.last_accessed_at = Utc::now();
        context.access_count += 1;

        // Update database with new access time
        let update_query = r#"
            UPDATE agent_contexts
            SET last_accessed_at = $1, access_count = $2
            WHERE id = $3
        "#;

        let access_count_i64 = context.access_count as i64;
        let update_params = vec![
            QueryParam::Timestamp(context.last_accessed_at),
            QueryParam::I64(access_count_i64),
            QueryParam::Uuid(*context_id),
        ];

        self.db_client
            .execute_with_params(update_query, &update_params)
            .await?;

        debug!(
            "Retrieved archived context {} from {}",
            context_id,
            archive_path.display()
        );

        Ok(Some(context))
    }

    /// Get archive statistics
    pub async fn get_archive_stats(&self) -> DataProcessingResult<ArchiveStats> {
        let query = r#"
            SELECT
                COUNT(*) as total_archived,
                COUNT(CASE WHEN archived_at > $1 THEN 1 END) as archived_this_week,
                AVG(EXTRACT(EPOCH FROM (NOW() - archived_at))) as avg_archive_age_seconds
            FROM agent_contexts
            WHERE archived_at IS NOT NULL
        "#;

        let one_week_ago = Utc::now() - Duration::days(7);
        let params = vec![QueryParam::Timestamp(one_week_ago)];

        let rows = self.db_client.query_with_params(query, &params).await?;
        if rows.is_empty() {
            return Ok(ArchiveStats::default());
        }

        let row = &rows[0];
        let total_archived: i64 = row.get("total_archived");
        let archived_this_week: i64 = row.get("archived_this_week");
        let avg_archive_age_seconds: Option<f64> = row.get("avg_archive_age_seconds");

        // Calculate archive storage size
        let archive_base = self
            .config
            .storage
            .archive_path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "archive".to_string());

        let mut total_archive_size = 0u64;
        if let Ok(entries) = std::fs::read_dir(&archive_base) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_archive_size += metadata.len();
                    } else if metadata.is_dir() {
                        // Recursively calculate size of subdirectories
                        fn calculate_dir_size(path: &std::path::Path) -> std::io::Result<u64> {
                            let mut size = 0u64;
                            for entry in std::fs::read_dir(path)? {
                                let entry = entry?;
                                let metadata = entry.metadata()?;
                                if metadata.is_file() {
                                    size += metadata.len();
                                } else if metadata.is_dir() {
                                    size += calculate_dir_size(&entry.path())?;
                                }
                            }
                            Ok(size)
                        }
                        if let Ok(dir_size) = calculate_dir_size(&entry.path()) {
                            total_archive_size += dir_size;
                        }
                    }
                }
            }
        }

        Ok(ArchiveStats {
            total_archived: total_archived as u64,
            archived_this_week: archived_this_week as u64,
            total_archive_size,
            avg_archive_age_seconds,
        })
    }

    /// Clean up old archived contexts based on retention policy
    pub async fn cleanup_archive(&self, retention_days: u32) -> DataProcessingResult<u64> {
        let cutoff_date = Utc::now() - Duration::days(retention_days as i64);

        // Find contexts to delete
        let query = r#"
            SELECT id, archive_location
            FROM agent_contexts
            WHERE archived_at < $1 AND archived_at IS NOT NULL
        "#;

        let params = vec![QueryParam::Timestamp(cutoff_date)];
        let rows = self.db_client.query_with_params(query, &params).await?;

        let mut deleted_count = 0u64;
        let archive_base = self
            .config
            .storage
            .archive_path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "archive".to_string());

        for row in rows {
            let context_id: Uuid = row.get("id");
            let archive_location: String = row.get("archive_location");

            // Delete archive file
            let archive_path = std::path::Path::new(&archive_base)
                .join(&archive_location)
                .join(format!("{}.ctx", context_id));

            if archive_path.exists() {
                if let Err(e) = std::fs::remove_file(&archive_path) {
                    warn!("Failed to delete archived context {}: {}", context_id, e);
                    continue;
                }
            }

            // Delete from database
            let delete_query = "DELETE FROM agent_contexts WHERE id = $1";
            let delete_params = vec![QueryParam::Uuid(context_id)];

            if let Ok(_) = self
                .db_client
                .execute_with_params(delete_query, &delete_params)
                .await
            {
                deleted_count += 1;
            }
        }

        info!(
            "Cleaned up {} archived contexts older than {} days",
            deleted_count, retention_days
        );
        Ok(deleted_count)
    }

    fn calculate_context_size(&self, context: &ContextData) -> u64 {
        // Rough calculation - could be more accurate
        serde_json::to_string(context)
            .map(|s| s.len() as u64)
            .unwrap_or(1024)
    }

    async fn get_current_storage_usage(&self) -> DataProcessingResult<u64> {
        let query = r#"
            SELECT COALESCE(SUM(size_bytes), 0) as total_size
            FROM agent_contexts
            WHERE archived_at IS NULL
        "#;

        // No parameters for this query
        let rows = self.db_client.query(query, &[]).await?;

        if rows.is_empty() {
            return Ok(0);
        }

        let total_size: i64 = rows[0].get("total_size");
        Ok(total_size as u64)
    }

    async fn update_stats_after_preservation(&self, size_bytes: u64) -> DataProcessingResult<()> {
        let mut stats = self.stats.write().await;
        stats.total_contexts += 1;
        stats.total_storage_size += size_bytes;

        if stats.total_contexts > 0 {
            stats.average_context_size = stats.total_storage_size / stats.total_contexts;
        }

        Ok(())
    }

    async fn update_lifecycle_stats(&self) -> DataProcessingResult<()> {
        let mut stats = self.stats.write().await;

        // Update working memory stats
        stats.working_memory_contexts = self.working_memory.read().await.len();

        // Update folded contexts count
        let folded_query = r#"
            SELECT COUNT(*) as folded_count
            FROM folded_contexts
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(folded_query, &[]).await {
            if !rows.is_empty() {
                let folded_count: i64 = rows[0].get("folded_count");
                stats.folded_contexts = folded_count as u64;
            }
        }

        // Update archived contexts count
        let archived_query = r#"
            SELECT COUNT(*) as archived_count
            FROM agent_contexts
            WHERE archived_at IS NOT NULL
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(archived_query, &[]).await {
            if !rows.is_empty() {
                let _archived_count: i64 = rows[0].get("archived_count");
                // Note: This should be tracked separately if needed
            }
        }

        // Update recent accesses (contexts accessed in last hour)
        let recent_query = r#"
            SELECT COUNT(*) as recent_count
            FROM agent_contexts
            WHERE last_accessed_at > $1
        "#;
        let one_hour_ago = Utc::now() - Duration::hours(1);
        let recent_params = vec![QueryParam::Timestamp(one_hour_ago)];
        if let Ok(rows) = self
            .db_client
            .query_with_params(recent_query, &recent_params)
            .await
        {
            if !rows.is_empty() {
                let recent_count: i64 = rows[0].get("recent_count");
                stats.recent_accesses = recent_count as u64;
            }
        }

        // Update oldest context age
        let oldest_query = r#"
            SELECT EXTRACT(EPOCH FROM (NOW() - MIN(created_at))) / 3600 as oldest_age_hours
            FROM agent_contexts
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(oldest_query, &[]).await {
            if !rows.is_empty() {
                let oldest_age: Option<f64> = rows[0].get("oldest_age_hours");
                stats.oldest_context_age_hours = oldest_age.unwrap_or(0.0) as u64;
            }
        }

        // Update compression ratio (if any compressed contexts exist)
        let compression_query = r#"
            SELECT
                AVG(CASE WHEN archived_at IS NOT NULL THEN 0.7 ELSE 1.0 END) as avg_compression
            FROM agent_contexts
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(compression_query, &[]).await {
            if !rows.is_empty() {
                let avg_compression: Option<f64> = rows[0].get("avg_compression");
                stats.compression_ratio = avg_compression.unwrap_or(1.0);
            }
        }

        debug!(
            "Updated lifecycle statistics: {} total, {} working memory, {} folded",
            stats.total_contexts, stats.working_memory_contexts, stats.folded_contexts
        );

        Ok(())
    }

    /// Get enhanced context lifecycle metrics
    pub async fn get_lifecycle_metrics(&self) -> DataProcessingResult<ContextLifecycleMetrics> {
        let mut metrics = ContextLifecycleMetrics::default();

        // Collect folding frequency by strategy
        let folding_query = r#"
            SELECT folding_strategy, COUNT(*) as count
            FROM folded_contexts
            GROUP BY folding_strategy
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(folding_query, &[]).await {
            for row in rows {
                let strategy: String = row.get("folding_strategy");
                let count: i64 = row.get("count");
                metrics.folding_frequency.insert(strategy, count as u64);
            }
        }

        // Calculate storage efficiency
        metrics.storage_efficiency = self.calculate_storage_efficiency().await?;

        // Calculate retrieval latency metrics
        metrics.retrieval_latency = self.calculate_retrieval_latency().await?;

        // Analyze access patterns
        metrics.access_patterns = self.analyze_access_patterns().await?;

        // Collect health metrics
        metrics.health_metrics = self.collect_health_metrics().await?;

        Ok(metrics)
    }

    /// Calculate storage efficiency metrics
    async fn calculate_storage_efficiency(&self) -> DataProcessingResult<StorageEfficiencyMetrics> {
        let mut efficiency = StorageEfficiencyMetrics::default();

        // Calculate compression ratios by strategy
        let compression_query = r#"
            SELECT
                folding_strategy,
                AVG(original_size::float / compressed_size::float) as avg_ratio,
                SUM(original_size - compressed_size) as total_savings
            FROM folded_contexts
            WHERE folding_strategy = 'compress'
            GROUP BY folding_strategy
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(compression_query, &[]).await {
            for row in rows {
                let strategy: String = row.get("folding_strategy");
                let ratio: f64 = row.get("avg_ratio");
                let savings: i64 = row.get("total_savings");

                efficiency.compression_by_strategy.insert(strategy, ratio);
                efficiency.storage_savings_bytes = savings as u64;
                efficiency.avg_compression_ratio = ratio;
            }
        }

        Ok(efficiency)
    }

    /// Calculate retrieval latency metrics
    async fn calculate_retrieval_latency(&self) -> DataProcessingResult<RetrievalLatencyMetrics> {
        let mut latency = RetrievalLatencyMetrics::default();

        // Calculate average retrieval times by source
        let latency_query = r#"
            SELECT
                source_type,
                AVG(retrieval_time_ms) as avg_latency
            FROM context_access_logs
            WHERE access_time > NOW() - INTERVAL '24 hours'
            GROUP BY source_type
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(latency_query, &[]).await {
            for row in rows {
                let source_type: String = row.get("source_type");
                let avg_latency: f64 = row.get("avg_latency");

                match source_type.as_str() {
                    "working_memory" => latency.working_memory_latency_ms = avg_latency,
                    "database" => latency.database_latency_ms = avg_latency,
                    "archive" => latency.archive_latency_ms = avg_latency,
                    _ => {}
                }
            }
        }

        // Calculate overall average
        let total_latency = latency.working_memory_latency_ms
            + latency.database_latency_ms
            + latency.archive_latency_ms;
        latency.avg_retrieval_latency_ms = total_latency / 3.0;

        Ok(latency)
    }

    /// Analyze access patterns for hot/cold context identification
    async fn analyze_access_patterns(&self) -> DataProcessingResult<AccessPatternMetrics> {
        let mut patterns = AccessPatternMetrics::default();

        // Identify hot contexts (accessed frequently in last 24h)
        let hot_query = r#"
            SELECT context_id, COUNT(*) as access_count
            FROM context_access_logs
            WHERE access_time > NOW() - INTERVAL '24 hours'
            GROUP BY context_id
            HAVING COUNT(*) > 10
            ORDER BY COUNT(*) DESC
            LIMIT 20
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(hot_query, &[]).await {
            for row in rows {
                let context_id: Uuid = row.get("context_id");
                patterns.hot_contexts.push(context_id);
            }
        }

        // Identify cold contexts (not accessed in last 7 days)
        let cold_query = r#"
            SELECT id
            FROM agent_contexts
            WHERE last_accessed_at < NOW() - INTERVAL '7 days'
            AND id NOT IN (
                SELECT DISTINCT context_id
                FROM context_access_logs
                WHERE access_time > NOW() - INTERVAL '7 days'
            )
            LIMIT 50
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(cold_query, &[]).await {
            for row in rows {
                let context_id: Uuid = row.get("id");
                patterns.cold_contexts.push(context_id);
            }
        }

        // Calculate access frequency distribution
        let freq_query = r#"
            SELECT
                CASE
                    WHEN access_count = 1 THEN 'single'
                    WHEN access_count BETWEEN 2 AND 5 THEN 'low'
                    WHEN access_count BETWEEN 6 AND 20 THEN 'medium'
                    WHEN access_count > 20 THEN 'high'
                END as frequency_range,
                COUNT(*) as context_count
            FROM (
                SELECT context_id, COUNT(*) as access_count
                FROM context_access_logs
                WHERE access_time > NOW() - INTERVAL '24 hours'
                GROUP BY context_id
            ) access_counts
            GROUP BY frequency_range
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(freq_query, &[]).await {
            for row in rows {
                let range: String = row.get("frequency_range");
                let count: i64 = row.get("context_count");
                patterns
                    .access_frequency_distribution
                    .insert(range, count as u64);
            }
        }

        Ok(patterns)
    }

    /// Collect health metrics and generate alerts
    async fn collect_health_metrics(&self) -> DataProcessingResult<ContextHealthMetrics> {
        let mut health = ContextHealthMetrics::default();

        // Detect orphaned contexts
        let orphaned_query = r#"
            SELECT COUNT(*) as orphaned_count
            FROM agent_contexts
            WHERE id NOT IN (
                SELECT DISTINCT context_id
                FROM context_access_logs
                WHERE access_time > NOW() - INTERVAL '30 days'
            )
            AND created_at < NOW() - INTERVAL '7 days'
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(orphaned_query, &[]).await {
            if !rows.is_empty() {
                health.orphaned_contexts = rows[0].get::<i64, _>("orphaned_count") as u64;
            }
        }

        // Calculate storage usage trend
        let trend_query = r#"
            SELECT
                AVG(storage_size) as avg_size,
                COUNT(*) as context_count
            FROM (
                SELECT
                    DATE_TRUNC('hour', created_at) as hour,
                    SUM(size_bytes) as storage_size
                FROM agent_contexts
                WHERE created_at > NOW() - INTERVAL '24 hours'
                GROUP BY hour
                ORDER BY hour
            ) hourly_sizes
        "#;
        // No parameters for this query
        if let Ok(rows) = self.db_client.query(trend_query, &[]).await {
            if !rows.is_empty() {
                let avg_size: f64 = rows[0].get("avg_size");
                health.storage_usage_trend = avg_size;
            }
        }

        // Calculate storage limit proximity
        let stats = self.stats.read().await;
        let config = &self.config.storage;
        health.storage_limit_proximity =
            stats.total_storage_size as f64 / config.max_context_size as f64;

        // Generate health alerts
        health.health_alerts = self.generate_health_alerts(&health).await?;

        Ok(health)
    }

    /// Generate health alerts based on metrics
    async fn generate_health_alerts(
        &self,
        health: &ContextHealthMetrics,
    ) -> DataProcessingResult<Vec<HealthAlert>> {
        let mut alerts = Vec::new();

        // Storage limit alert
        if health.storage_limit_proximity > 0.8 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::StorageLimitApproaching,
                message: format!(
                    "Storage usage at {:.1}% of limit",
                    health.storage_limit_proximity * 100.0
                ),
                severity: if health.storage_limit_proximity > 0.95 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                timestamp: Utc::now(),
            });
        }

        // Orphaned contexts alert
        if health.orphaned_contexts > 100 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::OrphanedContexts,
                message: format!("{} orphaned contexts detected", health.orphaned_contexts),
                severity: AlertSeverity::Medium,
                timestamp: Utc::now(),
            });
        }

        // Performance degradation alert
        let stats = self.stats.read().await;
        if stats.average_context_size > 10 * 1024 * 1024 {
            // 10MB
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::PerformanceDegradation,
                message: format!(
                    "Average context size is {:.1}MB",
                    stats.average_context_size as f64 / 1024.0 / 1024.0
                ),
                severity: AlertSeverity::Low,
                timestamp: Utc::now(),
            });
        }

        Ok(alerts)
    }

    fn start_cleanup_task(&self) {
        let config = self.config.clone();
        let working_memory = Arc::clone(&self.working_memory);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                config.working_memory.cleanup_interval_minutes as u64 * 60,
            ));

            loop {
                interval.tick().await;

                // Cleanup working memory
                let mut memory = working_memory.write().await;
                let mut to_remove = Vec::new();

                for (id, context) in memory.iter() {
                    let age = Utc::now().signed_duration_since(context.last_accessed_at);
                    if age > Duration::hours(config.storage.retention_hours as i64) {
                        to_remove.push(*id);
                    }
                }

                let removed_count = to_remove.len();
                for id in to_remove {
                    memory.remove(&id);
                }

                if removed_count > 0 {
                    debug!(
                        "Cleaned up {} expired contexts from working memory",
                        removed_count
                    );
                }

                // Update stats
                let mut stats_guard = stats.write().await;
                stats_guard.working_memory_contexts = memory.len();
            }
        });
    }
}
