use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;
use agent_agency_database::{DatabaseClient, DatabaseConfig};
use chrono::{DateTime, Utc};

/// Context store for persistent storage and retrieval of contexts
#[derive(Debug)]
pub struct ContextStore {
    /// Store configuration
    config: ContextPreservationConfig,
    /// Database client for persistent storage
    database_client: Arc<DatabaseClient>,
    /// In-memory context storage (context_id -> (data, metadata)) - for caching
    context_storage: Arc<RwLock<HashMap<Uuid, (ContextData, ContextMetadata)>>>,
    /// In-memory relationship storage (context_id -> relationships) - for caching
    relationship_storage: Arc<RwLock<HashMap<Uuid, Vec<ContextRelationship>>>>,
    /// In-memory cross-reference storage (context_id -> cross_references) - for caching
    cross_reference_storage: Arc<RwLock<HashMap<Uuid, Vec<CrossReference>>>>,
    /// In-memory synthesis result storage (context_id -> synthesis_results) - for caching
    synthesis_storage: Arc<RwLock<HashMap<Uuid, Vec<SynthesisResult>>>>,
}

impl ContextStore {
    /// Convert string to ContextFormat
    fn context_format_from_string(s: &str) -> ContextFormat {
        match s.to_lowercase().as_str() {
            "json" => ContextFormat::Json,
            "yaml" => ContextFormat::Yaml,
            "text" => ContextFormat::Text,
            "binary" => ContextFormat::Binary,
            _ => ContextFormat::Other,
        }
    }

    /// Convert string to ContextType
    fn context_type_from_string(s: &str) -> ContextType {
        match s.to_lowercase().as_str() {
            "task" => ContextType::Task,
            "worker" => ContextType::Worker,
            "council" => ContextType::Council,
            "research" => ContextType::Research,
            "security" => ContextType::Security,
            "performance" => ContextType::Performance,
            "user" => ContextType::User,
            "system" => ContextType::System,
            _ => ContextType::Other,
        }
    }

    /// Convert string to ContextPriority
    fn context_priority_from_string(s: &str) -> ContextPriority {
        match s.to_lowercase().as_str() {
            "low" => ContextPriority::Low,
            "medium" => ContextPriority::Medium,
            "high" => ContextPriority::High,
            "critical" => ContextPriority::Critical,
            _ => ContextPriority::Medium,
        }
    }

    /// Ensure all required database tables exist
    async fn ensure_database_tables(database_client: &DatabaseClient) -> Result<()> {
        debug!("Ensuring context store database tables exist");

        // Create contexts table
        let create_contexts_sql = r#"
            CREATE TABLE IF NOT EXISTS contexts (
                context_id UUID PRIMARY KEY,
                tenant_id VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                format VARCHAR(50) NOT NULL,
                encoding VARCHAR(50) NOT NULL,
                compression VARCHAR(50),
                checksum VARCHAR(255) NOT NULL,
                context_type VARCHAR(50) NOT NULL,
                priority VARCHAR(20) NOT NULL,
                tags JSONB DEFAULT '[]'::jsonb,
                description TEXT,
                source VARCHAR(255) NOT NULL,
                version VARCHAR(50) NOT NULL,
                dependencies JSONB DEFAULT '[]'::jsonb,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_contexts_tenant_id ON contexts(tenant_id);
            CREATE INDEX IF NOT EXISTS idx_contexts_type ON contexts(context_type);
            CREATE INDEX IF NOT EXISTS idx_contexts_priority ON contexts(priority);
            CREATE INDEX IF NOT EXISTS idx_contexts_source ON contexts(source);
            CREATE INDEX IF NOT EXISTS idx_contexts_created_at ON contexts(created_at DESC);
        "#;

        database_client.execute_parameterized_query(create_contexts_sql, vec![]).await?;

        // Create context_relationships table
        let create_relationships_sql = r#"
            CREATE TABLE IF NOT EXISTS context_relationships (
                id SERIAL PRIMARY KEY,
                context_id UUID NOT NULL REFERENCES contexts(context_id) ON DELETE CASCADE,
                related_context_id UUID NOT NULL,
                relationship_type VARCHAR(100) NOT NULL,
                description TEXT,
                strength REAL DEFAULT 1.0,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_relationships_context_id ON context_relationships(context_id);
            CREATE INDEX IF NOT EXISTS idx_relationships_related_context_id ON context_relationships(related_context_id);
            CREATE INDEX IF NOT EXISTS idx_relationships_type ON context_relationships(relationship_type);
        "#;

        database_client.execute_parameterized_query(create_relationships_sql, vec![]).await?;

        // Create cross_references table
        let create_cross_refs_sql = r#"
            CREATE TABLE IF NOT EXISTS cross_references (
                id SERIAL PRIMARY KEY,
                context_id UUID NOT NULL REFERENCES contexts(context_id) ON DELETE CASCADE,
                reference_type VARCHAR(100) NOT NULL,
                reference_target VARCHAR(500) NOT NULL,
                metadata JSONB DEFAULT '{}'::jsonb,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_cross_refs_context_id ON cross_references(context_id);
            CREATE INDEX IF NOT EXISTS idx_cross_refs_type ON cross_references(reference_type);
            CREATE INDEX IF NOT EXISTS idx_cross_refs_target ON cross_references(reference_target);
        "#;

        database_client.execute_parameterized_query(create_cross_refs_sql, vec![]).await?;

        // Create synthesis_results table
        let create_synthesis_sql = r#"
            CREATE TABLE IF NOT EXISTS synthesis_results (
                id SERIAL PRIMARY KEY,
                context_id UUID NOT NULL REFERENCES contexts(context_id) ON DELETE CASCADE,
                synthesis_type VARCHAR(100) NOT NULL,
                content TEXT NOT NULL,
                confidence REAL DEFAULT 1.0,
                metadata JSONB DEFAULT '{}'::jsonb,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_synthesis_context_id ON synthesis_results(context_id);
            CREATE INDEX IF NOT EXISTS idx_synthesis_type ON synthesis_results(synthesis_type);
            CREATE INDEX IF NOT EXISTS idx_synthesis_created_at ON synthesis_results(created_at DESC);
        "#;

        database_client.execute_parameterized_query(create_synthesis_sql, vec![]).await?;

        debug!("Context store database tables created successfully");
        Ok(())
    }

    /// Store context in database and cache
    async fn store_context_in_database(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
        context_data: &ContextData,
        metadata: &ContextMetadata,
    ) -> Result<()> {
        // Insert context data
        let insert_context_sql = r#"
            INSERT INTO contexts (
                context_id, tenant_id, content, format, encoding, compression, checksum,
                context_type, priority, tags, description, source, version, dependencies
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (context_id) DO UPDATE SET
                content = EXCLUDED.content,
                format = EXCLUDED.format,
                encoding = EXCLUDED.encoding,
                compression = EXCLUDED.compression,
                checksum = EXCLUDED.checksum,
                context_type = EXCLUDED.context_type,
                priority = EXCLUDED.priority,
                tags = EXCLUDED.tags,
                description = EXCLUDED.description,
                source = EXCLUDED.source,
                version = EXCLUDED.version,
                dependencies = EXCLUDED.dependencies,
                updated_at = NOW()
        "#;

        let tags_json = serde_json::to_value(&metadata.tags)?;
        let dependencies_json = serde_json::to_value(&metadata.dependencies)?;

        self.database_client
            .execute_parameterized_query(
                insert_context_sql,
                vec![
                    serde_json::Value::String(context_id.to_string()),
                    serde_json::Value::String(tenant_id.to_string()),
                    serde_json::Value::String(context_data.content.clone()),
                    serde_json::Value::String(context_data.format.to_string()),
                    serde_json::Value::String(context_data.encoding.clone()),
                    context_data.compression.as_ref().map(|c| serde_json::Value::String(c.clone())).unwrap_or(serde_json::Value::Null),
                    serde_json::Value::String(context_data.checksum.clone()),
                    serde_json::Value::String(metadata.context_type.to_string()),
                    serde_json::Value::String(metadata.priority.to_string()),
                    tags_json,
                    metadata.description.as_ref().map(|d| serde_json::Value::String(d.clone())).unwrap_or(serde_json::Value::Null),
                    serde_json::Value::String(metadata.source.clone()),
                    serde_json::Value::String(metadata.version.clone()),
                    dependencies_json,
                ],
            )
            .await?;

        // Insert relationships
        for relationship in &metadata.relationships {
            let insert_relationship_sql = r#"
                INSERT INTO context_relationships (
                    context_id, related_context_id, relationship_type, description, strength
                ) VALUES ($1, $2, $3, $4, $5)
            "#;

            self.database_client
                .execute_parameterized_query(
                    insert_relationship_sql,
                    vec![
                        serde_json::Value::String(context_id.to_string()),
                        serde_json::Value::String(relationship.related_context_id.to_string()),
                        serde_json::Value::String(relationship.relationship_type.clone()),
                        relationship.description.as_ref().map(|d| serde_json::Value::String(d.clone())).unwrap_or(serde_json::Value::Null),
                        serde_json::Value::Number(serde_json::Number::from_f64(relationship.strength).unwrap_or(serde_json::Number::from(1))),
                    ],
                )
                .await?;
        }

        debug!("Stored context {} in database for tenant {}", context_id, tenant_id);
        Ok(())
    }

    /// Retrieve context from database
    async fn retrieve_context_from_database(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
    ) -> Result<Option<(ContextData, ContextMetadata)>> {
        // Query context data
        let query_context_sql = r#"
            SELECT
                content, format, encoding, compression, checksum,
                context_type, priority, tags, description, source, version, dependencies
            FROM contexts
            WHERE context_id = $1 AND tenant_id = $2
        "#;

        let row = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(query_context_sql)
                        .bind(context_id)
                        .bind(tenant_id)
                        .fetch_optional(self.database_client.pool())
                        .await
                })
            })
            .await?;

        if let Some(row) = row {
            // Parse context data
            let content: String = row.try_get("content")?;
            let format: String = row.try_get("format")?;
            let encoding: String = row.try_get("encoding")?;
            let compression: Option<String> = row.try_get("compression").ok();
            let checksum: String = row.try_get("checksum")?;

            let context_data = ContextData {
                content,
                format: Self::context_format_from_string(&format),
                encoding,
                compression,
                checksum,
            };

            // Parse metadata
            let context_type: String = row.try_get("context_type")?;
            let priority: String = row.try_get("priority")?;
            let tags_json: serde_json::Value = row.try_get("tags").unwrap_or(serde_json::Value::Array(vec![]));
            let description: Option<String> = row.try_get("description").ok();
            let source: String = row.try_get("source")?;
            let version: String = row.try_get("version")?;
            let dependencies_json: serde_json::Value = row.try_get("dependencies").unwrap_or(serde_json::Value::Array(vec![]));

            let tags: Vec<String> = serde_json::from_value(tags_json).unwrap_or_default();
            let dependencies: Vec<String> = serde_json::from_value(dependencies_json).unwrap_or_default();

            let metadata = ContextMetadata {
                context_type: Self::context_type_from_string(&context_type),
                priority: Self::context_priority_from_string(&priority),
                tags,
                description,
                source,
                version,
                dependencies,
                relationships: vec![], // Will be populated separately
            };

            // Query relationships
            let relationships = self.retrieve_context_relationships_from_database(context_id).await?;

            Ok(Some((context_data, ContextMetadata { relationships, ..metadata })))
        } else {
            Ok(None)
        }
    }

    /// Retrieve context relationships from database
    async fn retrieve_context_relationships_from_database(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<ContextRelationship>> {
        let query_relationships_sql = r#"
            SELECT related_context_id, relationship_type, description, strength
            FROM context_relationships
            WHERE context_id = $1
        "#;

        let rows = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(query_relationships_sql)
                        .bind(context_id)
                        .fetch_all(self.database_client.pool())
                        .await
                })
            })
            .await?;

        let mut relationships = Vec::new();
        for row in rows {
            let related_context_id: String = row.try_get("related_context_id")?;
            let relationship_type: String = row.try_get("relationship_type")?;
            let description: Option<String> = row.try_get("description").ok();
            let strength: f32 = row.try_get("strength").unwrap_or(1.0);

            if let Ok(related_id) = Uuid::parse_str(&related_context_id) {
                relationships.push(ContextRelationship {
                    related_context_id: related_id,
                    relationship_type,
                    description,
                    strength,
                });
            }
        }

        Ok(relationships)
    }

    /// Store synthesis result in database
    async fn store_synthesis_result_in_database(
        &self,
        context_id: &Uuid,
        synthesis_result: &SynthesisResult,
    ) -> Result<()> {
        let insert_synthesis_sql = r#"
            INSERT INTO synthesis_results (
                context_id, synthesis_type, content, confidence, metadata
            ) VALUES ($1, $2, $3, $4, $5)
        "#;

        let metadata_json = serde_json::to_value(&synthesis_result.metadata)?;

        self.database_client
            .execute_parameterized_query(
                insert_synthesis_sql,
                vec![
                    serde_json::Value::String(context_id.to_string()),
                    serde_json::Value::String(synthesis_result.synthesis_type.clone()),
                    serde_json::Value::String(synthesis_result.content.clone()),
                    serde_json::Value::Number(serde_json::Number::from_f64(synthesis_result.confidence).unwrap_or(serde_json::Number::from(1))),
                    metadata_json,
                ],
            )
            .await?;

        Ok(())
    }

    /// Retrieve synthesis results from database
    async fn retrieve_synthesis_results_from_database(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<SynthesisResult>> {
        let query_synthesis_sql = r#"
            SELECT synthesis_type, content, confidence, metadata
            FROM synthesis_results
            WHERE context_id = $1
            ORDER BY created_at DESC
        "#;

        let rows = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(query_synthesis_sql)
                        .bind(context_id)
                        .fetch_all(self.database_client.pool())
                        .await
                })
            })
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let synthesis_type: String = row.try_get("synthesis_type")?;
            let content: String = row.try_get("content")?;
            let confidence: f32 = row.try_get("confidence").unwrap_or(1.0);
            let metadata_json: serde_json::Value = row.try_get("metadata").unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let metadata: HashMap<String, serde_json::Value> = serde_json::from_value(metadata_json).unwrap_or_default();

            results.push(SynthesisResult {
                synthesis_type,
                content,
                confidence,
                metadata,
            });
        }

        Ok(results)
    }

    /// Check tenant access control using database
    async fn check_tenant_access(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
    ) -> Result<bool> {
        // Query tenant access from database
        let query_access_sql = r#"
            SELECT tenant_id FROM contexts
            WHERE context_id = $1 AND tenant_id = $2
        "#;

        let result = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query_scalar::<_, String>(query_access_sql)
                        .bind(context_id)
                        .bind(tenant_id)
                        .fetch_optional(self.database_client.pool())
                        .await
                })
            })
            .await?;

        Ok(result.is_some())
    }

    /// Store synthesis result in database and cache
    pub async fn store_synthesis_result(
        &self,
        context_id: &Uuid,
        synthesis_result: &SynthesisResult,
    ) -> Result<()> {
        debug!("Storing synthesis result for context: {}", context_id);

        // Store in database
        self.store_synthesis_result_in_database(context_id, synthesis_result).await?;

        // Cache in memory
        let mut synthesis_storage = self.synthesis_storage.write().await;
        let results = synthesis_storage.entry(*context_id).or_insert_with(Vec::new);
        results.push(synthesis_result.clone());

        debug!("Successfully stored synthesis result for context {}", context_id);
        Ok(())
    }

    /// Create a new context store with database integration
    pub async fn new(config: ContextPreservationConfig, database_client: Arc<DatabaseClient>) -> Result<Self> {
        debug!("Initializing context store with database integration");

        // Ensure database tables exist
        Self::ensure_database_tables(&database_client).await?;

        Ok(Self {
            config,
            database_client,
            context_storage: Arc::new(RwLock::new(HashMap::new())),
            relationship_storage: Arc::new(RwLock::new(HashMap::new())),
            cross_reference_storage: Arc::new(RwLock::new(HashMap::new())),
            synthesis_storage: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store context
    pub async fn store_context(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
        context_data: &ContextData,
        metadata: &ContextMetadata,
    ) -> Result<StorageResult> {
        debug!("Storing context: {} for tenant: {}", context_id, tenant_id);

        // Validate context size against configuration limits
        if context_data.content.len() as u64 > self.config.storage.max_context_size {
            return Err(anyhow::anyhow!(
                "Context size {} exceeds maximum allowed size {}",
                context_data.content.len(),
                self.config.storage.max_context_size
            ));
        }

        // Check tenant limits if multi-tenant is enabled
        if self.config.multi_tenant.enabled {
            let default_limits = TenantLimits {
                max_contexts: self.config.storage.max_contexts_per_tenant,
                max_context_size: self.config.storage.max_context_size,
                retention_hours: self.config.storage.retention_hours,
                max_concurrent_operations: 10,
            };
            let tenant_limits = self
                .config
                .multi_tenant
                .tenant_limits
                .get(tenant_id)
                .unwrap_or(&default_limits);

            if context_data.content.len() as u64 > tenant_limits.max_context_size {
                return Err(anyhow::anyhow!(
                    "Context size {} exceeds tenant limit {}",
                    context_data.content.len(),
                    tenant_limits.max_context_size
                ));
            }

            // Implement proper tenant context limits checking with dynamic limits
            let storage = self.context_storage.read().await;

            // 1. Calculate comprehensive tenant context metrics
            let tenant_contexts: Vec<_> = storage
                .values()
                .filter(|(_, meta)| {
                    meta.relationships
                        .iter()
                        .any(|rel| rel.description.contains(tenant_id))
                })
                .collect();

            let tenant_context_count = tenant_contexts.len();

            // 2. Calculate total context size with compression awareness
            let mut total_compressed_size = 0u64;
            let mut total_uncompressed_size = 0u64;
            let mut context_sizes = Vec::new();

            for (data, _) in &tenant_contexts {
                let data_size = data.content.len() as u64;
                total_uncompressed_size += data_size;

                // Estimate compression ratio based on content type
                let compression_ratio = if data.content_type.contains("json") {
                    0.7 // JSON compresses well
                } else if data.content_type.contains("text") {
                    0.6 // Text compresses moderately
                } else {
                    0.9 // Binary data compresses poorly
                };

                let compressed_size = (data_size as f64 * compression_ratio) as u64;
                total_compressed_size += compressed_size;
                context_sizes.push(compressed_size);
            }

            // 3. Apply dynamic limits based on tenant tier and usage patterns
            let effective_limits = self.calculate_dynamic_limits(
                tenant_id,
                &tenant_limits,
                tenant_context_count,
                total_compressed_size,
                total_uncompressed_size
            );

            // 4. Check context count limit
            if tenant_context_count >= effective_limits.max_contexts as usize {
                return Err(anyhow::anyhow!(
                    "Tenant {} has reached maximum context limit {} (current: {})",
                    tenant_id,
                    effective_limits.max_contexts,
                    tenant_context_count
                ));
            }

            // 5. Check total size limit (using compressed size for storage efficiency)
            if total_compressed_size >= effective_limits.max_total_size {
                return Err(anyhow::anyhow!(
                    "Tenant {} has reached maximum total context size limit {}MB (current: {:.1}MB)",
                    tenant_id,
                    effective_limits.max_total_size / (1024 * 1024),
                    total_compressed_size as f64 / (1024.0 * 1024.0)
                ));
            }

            // 6. Check individual context size limits
            for (data, meta) in &tenant_contexts {
                let context_size = data.content.len() as u64;
                if context_size > effective_limits.max_individual_size {
                    return Err(anyhow::anyhow!(
                        "Context '{}' exceeds maximum individual size limit {}MB (size: {:.1}MB)",
                        meta.id,
                        effective_limits.max_individual_size / (1024 * 1024),
                        context_size as f64 / (1024.0 * 1024.0)
                    ));
                }
            }

            // 7. Implement context aging check (remove old contexts if approaching limits)
            if tenant_context_count > (effective_limits.max_contexts as usize * 80 / 100) {
                self.perform_context_aging_cleanup(tenant_id, &effective_limits).await?;
            }

            // 8. Log usage analytics for monitoring
            tracing::info!(
                tenant = %tenant_id,
                context_count = %tenant_context_count,
                total_compressed_size_mb = %format!("{:.2}", total_compressed_size as f64 / (1024.0 * 1024.0)),
                total_uncompressed_size_mb = %format!("{:.2}", total_uncompressed_size as f64 / (1024.0 * 1024.0)),
                compression_ratio = %format!("{:.2}", total_compressed_size as f64 / total_uncompressed_size as f64),
                "Tenant context usage within limits"
            );
        }

        // Store context data and metadata in database
        let start_time = std::time::Instant::now();
        self.store_context_in_database(context_id, tenant_id, context_data, metadata).await?;

        // Also cache in memory for faster access
        {
            let mut storage = self.context_storage.write().await;
            storage.insert(*context_id, (context_data.clone(), metadata.clone()));
        }

        // Store relationships if present
        if !metadata.relationships.is_empty() {
            let mut relationship_storage = self.relationship_storage.write().await;
            relationship_storage.insert(*context_id, metadata.relationships.clone());
        }

        let storage_time_ms = start_time.elapsed().as_millis() as u64;

        debug!(
            "Successfully stored context {} for tenant {} in {}ms",
            context_id, tenant_id, storage_time_ms
        );

        Ok(StorageResult {
            stored: true,
            storage_id: *context_id,
            storage_time_ms,
        })
    }

    /// Retrieve context
    pub async fn retrieve_context(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
    ) -> Result<Option<(ContextData, ContextMetadata)>> {
        debug!(
            "Retrieving context: {} for tenant: {}",
            context_id, tenant_id
        );

        // Check tenant access first
        if self.config.multi_tenant.enabled {
            let has_tenant_access = self.check_tenant_access(context_id, tenant_id).await?;

            if !has_tenant_access && self.config.multi_tenant.isolation_level == TenantIsolationLevel::Strict {
                warn!(
                    "Access denied: tenant {} cannot access context {}",
                    tenant_id, context_id
                );
                return Ok(None);
            }
        }

        // Try to get from cache first
        let storage = self.context_storage.read().await;
        if let Some((context_data, metadata)) = storage.get(context_id) {
            debug!(
                "Successfully retrieved context {} for tenant {} from cache",
                context_id, tenant_id
            );
            return Ok(Some((context_data.clone(), metadata.clone())));
        }

        // Not in cache, query from database
        if let Some((context_data, metadata)) = self.retrieve_context_from_database(context_id, tenant_id).await? {
            // Cache for future access
            {
                let mut storage = self.context_storage.write().await;
                storage.insert(*context_id, (context_data.clone(), metadata.clone()));
            }

            debug!(
                "Successfully retrieved context {} for tenant {} from database",
                context_id, tenant_id
            );
            Ok(Some((context_data, metadata)))
        } else {
            debug!("Context {} not found", context_id);
            Ok(None)
        }
    }

    /// Get context relationships
    pub async fn get_context_relationships(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<ContextRelationship>> {
        debug!("Getting relationships for context: {}", context_id);

        // Try cache first
        let relationship_storage = self.relationship_storage.read().await;
        if let Some(relationships) = relationship_storage.get(context_id) {
            debug!(
                "Found {} relationships for context {} from cache",
                relationships.len(),
                context_id
            );
            return Ok(relationships.clone());
        }

        // Not in cache, query from database
        let relationships = self.retrieve_context_relationships_from_database(context_id).await?;

        // Cache for future access
        if !relationships.is_empty() {
            let mut relationship_storage = self.relationship_storage.write().await;
            relationship_storage.insert(*context_id, relationships.clone());
        }

        debug!(
            "Found {} relationships for context {} from database",
            relationships.len(),
            context_id
        );
        Ok(relationships)
    }

    /// Get context cross-references
    pub async fn get_context_cross_references(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<CrossReference>> {
        debug!("Getting cross-references for context: {}", context_id);

        // Query cross-reference storage
        let cross_reference_storage = self.cross_reference_storage.read().await;

        if let Some(cross_references) = cross_reference_storage.get(context_id) {
            debug!(
                "Found {} cross-references for context {}",
                cross_references.len(),
                context_id
            );
            Ok(cross_references.clone())
        } else {
            debug!("No cross-references found for context {}", context_id);
            Ok(Vec::new())
        }
    }

    /// Get context synthesis results
    pub async fn get_context_synthesis_results(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<SynthesisResult>> {
        debug!("Getting synthesis results for context: {}", context_id);

        // Try cache first
        let synthesis_storage = self.synthesis_storage.read().await;
        if let Some(synthesis_results) = synthesis_storage.get(context_id) {
            debug!(
                "Found {} synthesis results for context {} from cache",
                synthesis_results.len(),
                context_id
            );
            return Ok(synthesis_results.clone());
        }

        // Not in cache, query from database
        let synthesis_results = self.retrieve_synthesis_results_from_database(context_id).await?;

        // Cache for future access
        if !synthesis_results.is_empty() {
            let mut synthesis_storage = self.synthesis_storage.write().await;
            synthesis_storage.insert(*context_id, synthesis_results.clone());
        }

        debug!(
            "Found {} synthesis results for context {} from database",
            synthesis_results.len(),
            context_id
        );
        Ok(synthesis_results)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing context store health check");

        // Check in-memory storage availability
        let context_count = self.context_storage.read().await.len();
        let relationship_count = self.relationship_storage.read().await.len();
        let cross_ref_count = self.cross_reference_storage.read().await.len();
        let synthesis_count = self.synthesis_storage.read().await.len();

        debug!(
            "Context store health check: {} contexts, {} relationships, {} cross-refs, {} synthesis results",
            context_count, relationship_count, cross_ref_count, synthesis_count
        );

        // Check configuration validity
        if self.config.storage.max_context_size == 0 {
            warn!("Invalid configuration: max_context_size is 0");
            return Ok(false);
        }

        // Check if we can perform basic operations
        let test_context_id = Uuid::new_v4();
        let test_data = ContextData {
            content: "health_check_test".to_string(),
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            checksum: "test".to_string(),
        };

        let test_metadata = ContextMetadata {
            context_type: ContextType::System,
            priority: ContextPriority::Low,
            tags: vec!["health_check".to_string()],
            description: "Health check test".to_string(),
            source: "system".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            relationships: vec![],
        };

        // Test storage operation
        match self
            .store_context(
                &test_context_id,
                "health_check_tenant",
                &test_data,
                &test_metadata,
            )
            .await
        {
            Ok(result) => {
                if !result.stored {
                    warn!("Health check failed: context storage returned stored=false");
                    return Ok(false);
                }
            }
            Err(e) => {
                error!("Health check failed during storage test: {}", e);
                return Ok(false);
            }
        }

        // Test retrieval operation
        match self
            .retrieve_context(&test_context_id, "health_check_tenant")
            .await
        {
            Ok(Some(_)) => {
                debug!("Health check passed: storage and retrieval working");
                Ok(true)
            }
            Ok(None) => {
                warn!("Health check failed: context not found after storage");
                Ok(false)
            }
            Err(e) => {
                error!("Health check failed during retrieval test: {}", e);
                Ok(false)
            }
        }
    }
}

/// Storage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    /// Whether storage was successful
    pub stored: bool,
    /// Storage ID
    pub storage_id: Uuid,
    /// Storage time (milliseconds)
    pub storage_time_ms: u64,
}

impl ContextStore {
    /// Calculate dynamic limits based on tenant tier, usage patterns, and system capacity
    fn calculate_dynamic_limits(
        &self,
        tenant_id: &str,
        base_limits: &TenantLimits,
        current_count: usize,
        current_compressed_size: u64,
        current_uncompressed_size: u64,
    ) -> TenantLimits {
        let mut effective_limits = base_limits.clone();

        // 1. Apply tenant tier-based scaling
        let tier_multiplier = match tenant_id {
            id if id.starts_with("premium-") => 2.0,
            id if id.starts_with("enterprise-") => 5.0,
            id if id.starts_with("trial-") => 0.5,
            _ => 1.0, // Standard tier
        };

        effective_limits.max_contexts = (effective_limits.max_contexts as f64 * tier_multiplier) as u32;
        effective_limits.max_total_size = (effective_limits.max_total_size as f64 * tier_multiplier) as u64;
        effective_limits.max_individual_size = (effective_limits.max_individual_size as f64 * tier_multiplier) as u64;

        // 2. Apply usage-based scaling (reward efficient usage)
        let compression_ratio = if current_uncompressed_size > 0 {
            current_compressed_size as f64 / current_uncompressed_size as f64
        } else {
            1.0
        };

        // Bonus for good compression ratios (efficient storage)
        if compression_ratio < 0.8 {
            let efficiency_bonus = 1.2;
            effective_limits.max_contexts = (effective_limits.max_contexts as f64 * efficiency_bonus) as u32;
            effective_limits.max_total_size = (effective_limits.max_total_size as f64 * efficiency_bonus) as u64;
        }

        // 3. Apply system capacity scaling (reduce limits if system is under pressure)
        let system_load_factor = 0.8; // Could be calculated from actual system metrics
        if system_load_factor > 0.9 {
            effective_limits.max_contexts = (effective_limits.max_contexts as f64 * 0.8) as u32;
            effective_limits.max_total_size = (effective_limits.max_total_size as f64 * 0.8) as u64;
        }

        // 4. Apply time-based scaling (gradual limit increases for established tenants)
        // This would typically check tenant creation date from a tenant registry
        let tenant_age_days = 30; // Placeholder - would come from tenant metadata
        if tenant_age_days > 7 {
            let age_bonus = 1.1;
            effective_limits.max_contexts = (effective_limits.max_contexts as f64 * age_bonus) as u32;
        }

        effective_limits
    }

    /// Perform automatic cleanup of old contexts when approaching limits
    async fn perform_context_aging_cleanup(
        &self,
        tenant_id: &str,
        limits: &TenantLimits,
    ) -> Result<(), anyhow::Error> {
        let mut storage = self.context_storage.write().await;

        // Find contexts for this tenant, sorted by age (oldest first)
        let mut tenant_contexts: Vec<_> = storage
            .iter()
            .filter(|(_, (_, meta))| {
                meta.relationships
                    .iter()
                    .any(|rel| rel.description.contains(tenant_id))
            })
            .map(|(key, (_, meta))| (key.clone(), meta.created_at))
            .collect();

        tenant_contexts.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by creation time (oldest first)

        // Remove oldest contexts until we're below 70% of the limit
        let target_count = (limits.max_contexts as usize * 70 / 100).max(1);
        let mut removed_count = 0;

        for (context_key, _) in tenant_contexts {
            if storage.len() <= target_count {
                break;
            }

            if storage.remove(&context_key).is_some() {
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            tracing::info!(
                tenant = %tenant_id,
                removed_contexts = %removed_count,
                remaining_contexts = %storage.len(),
                "Performed automatic context cleanup for tenant"
            );
        }

        Ok(())
    }
}
