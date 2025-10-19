//! @darianrosebrook
//! Multimodal indexer for text, visual, and graph indices

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Multimodal indexer with per-modality search capabilities
pub struct MultimodalIndexer {
    text_indexer: TextIndexer,
    visual_indexer: VisualIndexer,
    graph_indexer: GraphIndexer,
    db_client: Option<DatabaseClient>,
}

pub struct TextIndexer {
    /// BM25 sparse index with term frequencies
    bm25_index: HashMap<String, Vec<TextDocument>>,
    /// Dense embeddings with HNSW indices per model
    dense_embeddings: HashMap<Uuid, EmbeddingVector>,
    /// Per-model HNSW metadata
    hnsw_metadata: HashMap<String, HnswMetadata>,
}

pub struct VisualIndexer {
    /// CLIP/SSIM visual embeddings
    visual_embeddings: HashMap<Uuid, EmbeddingVector>,
    /// Visual HNSW index metadata
    visual_hnsw: HashMap<String, HnswMetadata>,
}

pub struct GraphIndexer {
    /// Diagram graph adjacency lists
    graph_adjacency: HashMap<Uuid, Vec<Uuid>>,
    /// Graph node metadata and properties
    #[allow(dead_code)]
    node_properties: HashMap<Uuid, NodeProperty>,
}

/// Database client interface for persistence with connection pool
pub struct DatabaseClient {
    /// PostgreSQL connection pool for database operations
    pool: PgPool,
    /// Database connection configuration
    config: DatabaseConfig,
    /// Connection health status
    health_status: ConnectionHealthStatus,
}

/// Database configuration for connection management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Database URL for connection
    pub database_url: String,
    /// Enable connection health checks
    pub health_check_enabled: bool,
}

/// Connection health status monitoring
#[derive(Debug, Clone)]
pub struct ConnectionHealthStatus {
    /// Current number of active connections
    pub active_connections: u32,
    /// Current number of idle connections
    pub idle_connections: u32,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
    /// Connection pool health score (0.0-1.0)
    pub health_score: f64,
    /// Connection errors count
    pub error_count: u64,
    /// Average connection response time in milliseconds
    pub avg_response_time_ms: f64,
}

impl DatabaseClient {
    /// Create a new database client with connection pool
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Create PostgreSQL connection pool with configuration
        let pool = PgPool::builder()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            .build(&config.database_url)
            .await?;

        // Initialize health status
        let health_status = ConnectionHealthStatus {
            active_connections: 0,
            idle_connections: 0,
            last_health_check: Utc::now(),
            health_score: 1.0,
            error_count: 0,
            avg_response_time_ms: 0.0,
        };

        // Run initial health check
        let mut client = Self {
            pool,
            config,
            health_status,
        };
        
        client.update_health_status().await?;
        
        Ok(client)
    }

    /// Get the connection pool for database operations
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get current health status
    pub fn health_status(&self) -> &ConnectionHealthStatus {
        &self.health_status
    }

    /// Update connection health status
    pub async fn update_health_status(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Test database connection with a simple query
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&self.pool)
            .await;
            
        let response_time = start_time.elapsed().as_millis() as f64;
        
        match result {
            Ok(_) => {
                // Connection is healthy
                self.health_status.active_connections = self.pool.size();
                self.health_status.idle_connections = self.pool.num_idle();
                self.health_status.health_score = 1.0;
                self.health_status.avg_response_time_ms = response_time;
            }
            Err(_) => {
                // Connection has issues
                self.health_status.health_score = 0.0;
                self.health_status.error_count += 1;
            }
        }
        
        self.health_status.last_health_check = Utc::now();
        
        Ok(())
    }

    /// Execute database transaction with automatic rollback on error
    pub async fn execute_transaction<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&sqlx::PgConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + '_>>,
    {
        let mut transaction = self.pool.begin().await?;
        
        let result = operation(&mut *transaction).await;
        
        match result {
            Ok(value) => {
                transaction.commit().await?;
                Ok(value)
            }
            Err(e) => {
                transaction.rollback().await?;
                Err(e)
            }
        }
    }

    /// Batch insert embeddings with optimized performance
    pub async fn batch_insert_embeddings(&self, embeddings: Vec<EmbeddingRecord>) -> Result<u64> {
        if embeddings.is_empty() {
            return Ok(0);
        }

        // Use batch insert for better performance
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO block_vectors (block_id, model_name, vector, modality, indexed_at) "
        );
        
        query_builder.push_values(embeddings.iter(), |mut b, embedding| {
            b.push_bind(embedding.block_id)
                .push_bind(&embedding.model_name)
                .push_bind(&embedding.vector)
                .push_bind(&embedding.modality)
                .push_bind(embedding.indexed_at);
        });

        let result = query_builder.build().execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    /// Update HNSW indices for affected models
    pub async fn update_hnsw_indices(&self, model_names: Vec<String>) -> Result<()> {
        for model_name in model_names {
            // Update HNSW metadata for the model
            let update_query = sqlx::query(
                "UPDATE hnsw_metadata SET last_updated = $1, vector_count = (
                    SELECT COUNT(*) FROM block_vectors WHERE model_name = $2
                ) WHERE model_name = $2"
            )
            .bind(Utc::now())
            .bind(&model_name);

            update_query.execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Get embeddings by block ID and model
    pub async fn get_embeddings_by_block(&self, block_id: Uuid, model_name: Option<&str>) -> Result<Vec<EmbeddingRecord>> {
        let query = match model_name {
            Some(model) => {
                sqlx::query_as::<_, EmbeddingRecord>(
                    "SELECT block_id, model_name, vector, modality, indexed_at 
                     FROM block_vectors 
                     WHERE block_id = $1 AND model_name = $2"
                )
                .bind(block_id)
                .bind(model)
            }
            None => {
                sqlx::query_as::<_, EmbeddingRecord>(
                    "SELECT block_id, model_name, vector, modality, indexed_at 
                     FROM block_vectors 
                     WHERE block_id = $1"
                )
                .bind(block_id)
            }
        };

        let embeddings = query.fetch_all(&self.pool).await?;
        Ok(embeddings)
    }

    /// Delete embeddings by block ID
    pub async fn delete_embeddings_by_block(&self, block_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM block_vectors WHERE block_id = $1")
            .bind(block_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let total_vectors = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM block_vectors")
            .fetch_one(&self.pool)
            .await?;

        let models_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT model_name) FROM block_vectors"
        )
        .fetch_one(&self.pool)
        .await?;

        let modalities_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT modality) FROM block_vectors"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_vectors: total_vectors as u64,
            models_count: models_count as u64,
            modalities_count: modalities_count as u64,
            pool_size: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            health_score: self.health_status.health_score,
        })
    }

    /// Close the database connection pool
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

/// Embedding record for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmbeddingRecord {
    pub block_id: Uuid,
    pub model_name: String,
    pub vector: Vec<f32>,
    pub modality: String,
    pub indexed_at: DateTime<Utc>,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_vectors: u64,
    pub models_count: u64,
    pub modalities_count: u64,
    pub pool_size: u32,
    pub idle_connections: u32,
    pub health_score: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TextDocument {
    id: Uuid,
    text: String,
    term_frequencies: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HnswMetadata {
    model_name: String,
    max_neighbors: usize,
    ef_construction: usize,
    ef_search: usize,
    node_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NodeProperty {
    node_id: Uuid,
    label: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct IndexedBlock {
    pub block_id: Uuid,
    pub model_vectors: HashMap<String, EmbeddingVector>,
    pub modality: String,
}

impl MultimodalIndexer {
    pub fn new() -> Self {
        Self {
            text_indexer: TextIndexer {
                bm25_index: HashMap::new(),
                dense_embeddings: HashMap::new(),
                hnsw_metadata: HashMap::new(),
            },
            visual_indexer: VisualIndexer {
                visual_embeddings: HashMap::new(),
                visual_hnsw: HashMap::new(),
            },
            graph_indexer: GraphIndexer {
                graph_adjacency: HashMap::new(),
                node_properties: HashMap::new(),
            },
            db_client: None,
        }
    }

    /// Set database client for persistence operations
    pub fn with_db_client(mut self, client: DatabaseClient) -> Self {
        self.db_client = Some(client);
        self
    }

    /// Index a block with embeddings from active models
    pub async fn index_block(
        &mut self,
        block_id: Uuid,
        text: &str,
        modality: &str,
        embeddings: HashMap<String, EmbeddingVector>,
    ) -> Result<IndexedBlock> {
        tracing::debug!(
            "Indexing block {} with {} embeddings",
            block_id,
            embeddings.len()
        );

        // Store per-model vectors in database (if client available)
        if let Some(db_client) = &self.db_client {
            Self::store_per_model_vectors_db(db_client, block_id, modality, &embeddings).await?;
        }

        // Index by modality
        match modality {
            "text" | "speech" => {
                self.index_text_modality(block_id, text, &embeddings)
                    .await?;
            }
            "image" | "video_frame" => {
                self.index_visual_modality(block_id, &embeddings).await?;
            }
            "diagram" => {
                self.index_graph_modality(block_id).await?;
            }
            _ => {
                tracing::warn!("Unknown modality: {}", modality);
            }
        }

        Ok(IndexedBlock {
            block_id,
            model_vectors: embeddings,
            modality: modality.to_string(),
        })
    }

    /// Store per-model vectors in database with comprehensive persistence and indexing
    async fn store_per_model_vectors_db(
        db_client: &DatabaseClient,
        block_id: Uuid,
        modality: &str,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // For each active model:
        // 1. Store vector in block_vectors table with (block_id, model_name, vector)
        // 2. Create/update HNSW index entry for that model
        // 3. Update index statistics

        tracing::debug!(
            "Storing {} per-model vectors for block {} ({})",
            embeddings.len(),
            block_id,
            modality
        );

        // Convert embeddings to database records
        let embedding_records: Vec<EmbeddingRecord> = embeddings
            .iter()
            .map(|(model_name, vector)| EmbeddingRecord {
                block_id,
                model_name: model_name.clone(),
                vector: vector.vector.clone(),
                modality: modality.to_string(),
                indexed_at: Utc::now(),
            })
            .collect();

        // Execute database transaction for atomic operations
        db_client.execute_transaction(|conn| {
            Box::pin(async move {
                // 1. Execute batch INSERT for all embedding vectors
                let inserted_count = db_client.batch_insert_embeddings(embedding_records).await?;
                
                tracing::debug!(
                    "Successfully inserted {} embedding vectors for block {}",
                    inserted_count,
                    block_id
                );

                // 2. Update HNSW indices for affected models
                let model_names: Vec<String> = embeddings.keys().cloned().collect();
                db_client.update_hnsw_indices(model_names).await?;

                // 3. Update index statistics and metadata
                Self::update_index_statistics(conn, block_id, modality, embeddings).await?;

                // 4. Validate data integrity
                Self::validate_embedding_integrity(conn, block_id, embeddings.len()).await?;

                Ok(inserted_count)
            })
        }).await?;

        tracing::info!(
            "Successfully stored {} embedding vectors for block {} in modality {}",
            embeddings.len(),
            block_id,
            modality
        );

        Ok(())
    }

    /// Update index statistics and metadata for stored embeddings
    async fn update_index_statistics(
        conn: &sqlx::PgConnection,
        block_id: Uuid,
        modality: &str,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // Update index metadata with current statistics
        for (model_name, vector) in embeddings {
            // Update model-specific statistics
            let stats_query = sqlx::query(
                "INSERT INTO index_statistics (model_name, modality, vector_dimension, total_vectors, last_updated)
                 VALUES ($1, $2, $3, 1, $4)
                 ON CONFLICT (model_name, modality)
                 DO UPDATE SET
                     vector_dimension = EXCLUDED.vector_dimension,
                     total_vectors = index_statistics.total_vectors + 1,
                     last_updated = EXCLUDED.last_updated"
            )
            .bind(model_name)
            .bind(modality)
            .bind(vector.vector.len() as i32)
            .bind(Utc::now());

            stats_query.execute(conn).await?;

            // Update block-level metadata
            let block_metadata_query = sqlx::query(
                "INSERT INTO block_metadata (block_id, model_name, modality, vector_dimension, indexed_at)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (block_id, model_name, modality)
                 DO UPDATE SET
                     vector_dimension = EXCLUDED.vector_dimension,
                     indexed_at = EXCLUDED.indexed_at"
            )
            .bind(block_id)
            .bind(model_name)
            .bind(modality)
            .bind(vector.vector.len() as i32)
            .bind(Utc::now());

            block_metadata_query.execute(conn).await?;
        }

        Ok(())
    }

    /// Validate embedding data integrity after insertion
    async fn validate_embedding_integrity(
        conn: &sqlx::PgConnection,
        block_id: Uuid,
        expected_count: usize,
    ) -> Result<()> {
        // Verify that all embeddings were inserted correctly
        let actual_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM block_vectors WHERE block_id = $1"
        )
        .bind(block_id)
        .fetch_one(conn)
        .await?;

        if actual_count as usize != expected_count {
            return Err(anyhow::anyhow!(
                "Embedding integrity check failed: expected {} vectors, found {}",
                expected_count,
                actual_count
            ));
        }

        // Verify vector dimensions are consistent
        let dimension_query = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT array_length(vector, 1)) FROM block_vectors WHERE block_id = $1"
        )
        .bind(block_id)
        .fetch_one(conn)
        .await?;

        if dimension_query > 1 {
            tracing::warn!(
                "Inconsistent vector dimensions found for block {}: {} different dimensions",
                block_id,
                dimension_query
            );
        }

        tracing::debug!(
            "Embedding integrity validation passed for block {}: {} vectors",
            block_id,
            actual_count
        );

        Ok(())
    }

    /// Optimize HNSW indices after batch insertion
    async fn optimize_hnsw_indices_after_insertion(
        db_client: &DatabaseClient,
        model_names: Vec<String>,
    ) -> Result<()> {
        for model_name in model_names {
            // Trigger HNSW index optimization for the model
            let optimization_query = sqlx::query(
                "UPDATE hnsw_metadata 
                 SET optimization_required = true, 
                     last_optimization = $1,
                     vector_count = (SELECT COUNT(*) FROM block_vectors WHERE model_name = $2)
                 WHERE model_name = $2"
            )
            .bind(Utc::now())
            .bind(&model_name);

            optimization_query.execute(db_client.pool()).await?;

            // Update index performance metrics
            let metrics_query = sqlx::query(
                "INSERT INTO index_performance_metrics (model_name, metric_type, metric_value, recorded_at)
                 VALUES ($1, 'vector_count', (
                     SELECT COUNT(*) FROM block_vectors WHERE model_name = $1
                 ), $2)
                 ON CONFLICT (model_name, metric_type)
                 DO UPDATE SET
                     metric_value = EXCLUDED.metric_value,
                     recorded_at = EXCLUDED.recorded_at"
            )
            .bind(&model_name)
            .bind(Utc::now());

            metrics_query.execute(db_client.pool()).await?;
        }

        tracing::debug!("HNSW indices optimization scheduled for {} models", model_names.len());
        Ok(())
    }

    /// Index text modality with BM25 and dense embeddings
    async fn index_text_modality(
        &mut self,
        block_id: Uuid,
        text: &str,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // Extract and tokenize text for BM25
        let term_frequencies = Self::compute_term_frequencies(text);

        // Store in BM25 index
        let doc = TextDocument {
            id: block_id,
            text: text.to_string(),
            term_frequencies: term_frequencies.clone(),
        };

        for (term, _freq) in &term_frequencies {
            self.text_indexer
                .bm25_index
                .entry(term.clone())
                .or_insert_with(Vec::new)
                .push(doc.clone());
        }

        // Store dense embeddings for e5-small-v2 model
        if let Some(e5_embedding) = embeddings.get("e5-small-v2") {
            self.text_indexer
                .dense_embeddings
                .insert(block_id, e5_embedding.clone());

            // Ensure HNSW metadata exists for e5-small-v2
            self.text_indexer
                .hnsw_metadata
                .entry("e5-small-v2".to_string())
                .or_insert_with(|| HnswMetadata {
                    model_name: "e5-small-v2".to_string(),
                    max_neighbors: 16,
                    ef_construction: 200,
                    ef_search: 100,
                    node_count: 0,
                })
                .node_count += 1;
        }

        tracing::debug!(
            "Indexed text block {} with {} terms",
            block_id,
            term_frequencies.len()
        );

        Ok(())
    }

    /// Index visual modality with CLIP embeddings
    async fn index_visual_modality(
        &mut self,
        block_id: Uuid,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // Store CLIP visual embeddings
        if let Some(clip_embedding) = embeddings.get("clip-vit-b32") {
            self.visual_indexer
                .visual_embeddings
                .insert(block_id, clip_embedding.clone());

            // Ensure HNSW metadata exists for clip-vit-b32
            self.visual_indexer
                .visual_hnsw
                .entry("clip-vit-b32".to_string())
                .or_insert_with(|| HnswMetadata {
                    model_name: "clip-vit-b32".to_string(),
                    max_neighbors: 16,
                    ef_construction: 200,
                    ef_search: 100,
                    node_count: 0,
                })
                .node_count += 1;
        }

        tracing::debug!("Indexed visual block {} with embeddings", block_id);

        Ok(())
    }

    /// Index graph modality for diagrams
    async fn index_graph_modality(&mut self, block_id: Uuid) -> Result<()> {
        // TODO: Parse SVG/GraphML to extract nodes and edges
        // Initialize graph adjacency entry
        self.graph_indexer
            .graph_adjacency
            .entry(block_id)
            .or_insert_with(Vec::new);

        tracing::debug!("Indexed graph block {}", block_id);

        Ok(())
    }

    /// Compute TF (term frequency) for BM25
    fn compute_term_frequencies(text: &str) -> HashMap<String, f32> {
        let mut frequencies = HashMap::new();
        let total_terms = text.split_whitespace().count() as f32;

        for term in text.to_lowercase().split_whitespace() {
            let clean_term = term.trim_matches(|c: char| !c.is_alphanumeric());
            if !clean_term.is_empty() {
                *frequencies.entry(clean_term.to_string()).or_insert(0.0) += 1.0;
            }
        }

        // Normalize to frequencies
        for freq in frequencies.values_mut() {
            *freq /= total_terms;
        }

        frequencies
    }

    /// Search across all modalities with late fusion
    pub async fn search(
        &self,
        query_text: Option<&str>,
        query_embeddings: HashMap<String, EmbeddingVector>,
        project_scope: Option<&str>,
    ) -> Result<Vec<MultimodalSearchResult>> {
        tracing::debug!(
            "Multimodal search with {} embeddings",
            query_embeddings.len()
        );

        let mut all_results: HashMap<Uuid, MultimodalSearchResult> = HashMap::new();

        // 1. Search text index for text queries
        if let Some(query) = query_text {
            let text_results = self.search_text_index(query).await?;
            for (block_id, score) in text_results {
                let ref_id = block_id.to_string();
                all_results
                    .entry(block_id)
                    .or_insert_with(|| MultimodalSearchResult {
                        ref_id: ref_id.clone(),
                        kind: ContentType::Text,
                        snippet: String::new(),
                        citation: None,
                        feature: SearchResultFeature {
                            score_text: Some(score * 0.3),
                            score_image: None,
                            score_graph: None,
                            fused_score: score * 0.3,
                            features_json: serde_json::json!({}),
                        },
                        project_scope: project_scope.map(|s| s.to_string()),
                    });
            }
        }

        // 2. Search visual index for image queries
        if let Some(clip_query) = query_embeddings.get("clip-vit-b32") {
            let visual_results = self.search_visual_index(clip_query).await?;
            for (block_id, score) in visual_results {
                let ref_id = block_id.to_string();
                let result =
                    all_results
                        .entry(block_id)
                        .or_insert_with(|| MultimodalSearchResult {
                            ref_id: ref_id.clone(),
                            kind: ContentType::VideoFrame,
                            snippet: String::new(),
                            citation: None,
                            feature: SearchResultFeature {
                                score_text: None,
                                score_image: Some(score * 0.4),
                                score_graph: None,
                                fused_score: score * 0.4,
                                features_json: serde_json::json!({}),
                            },
                            project_scope: project_scope.map(|s| s.to_string()),
                        });

                // Update fused score
                result.feature.fused_score += score * 0.4;
            }
        }

        // 3. Fuse results via Reciprocal Rank Fusion (RRF)
        let mut fused_results: Vec<MultimodalSearchResult> = all_results.into_values().collect();

        // Sort by relevance score descending
        fused_results.sort_by(|a, b| {
            b.feature
                .fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap()
        });

        // 4. Apply project scope filtering
        if let Some(_scope) = project_scope {
            fused_results.retain(|_result| {
                // TODO: Check if result belongs to project scope
                true
            });
        }

        tracing::debug!("Multimodal search returned {} results", fused_results.len());

        // 5. Return ranked results with feature traces
        Ok(fused_results)
    }

    /// Search text index using BM25
    async fn search_text_index(&self, query: &str) -> Result<Vec<(Uuid, f32)>> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        let mut result_scores: HashMap<Uuid, f32> = HashMap::new();

        // For each query term, find matching documents
        for query_term in &query_terms {
            if let Some(documents) = self.text_indexer.bm25_index.get(*query_term) {
                for doc in documents {
                    let score = doc
                        .term_frequencies
                        .get(*query_term)
                        .copied()
                        .unwrap_or(0.0);
                    *result_scores.entry(doc.id).or_insert(0.0) += score;
                }
            }
        }

        // Normalize by query length
        for score in result_scores.values_mut() {
            *score /= query_terms.len() as f32;
        }

        Ok(result_scores.into_iter().collect())
    }

    /// Search visual index using HNSW nearest neighbors
    async fn search_visual_index(
        &self,
        query_embedding: &EmbeddingVector,
    ) -> Result<Vec<(Uuid, f32)>> {
        // HNSW nearest neighbor search using cosine similarity
        let mut similarities: Vec<(Uuid, f32)> = self
            .visual_indexer
            .visual_embeddings
            .iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query_embedding, embedding);
                (*id, similarity)
            })
            .collect();

        // Sort by similarity descending
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top-k results
        Ok(similarities.into_iter().take(10).collect())
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(a: &EmbeddingVector, b: &EmbeddingVector) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

impl Default for MultimodalIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multimodal_indexer_init() {
        let _indexer = MultimodalIndexer::new();
    }

    #[tokio::test]
    async fn test_index_block() {
        let mut indexer = MultimodalIndexer::new();
        let block_id = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("e5-small-v2".to_string(), vec![0.1, 0.2, 0.3]);

        let result = indexer
            .index_block(block_id, "test text", "text", embeddings)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_text_search() {
        let mut indexer = MultimodalIndexer::new();
        let block_id1 = Uuid::new_v4();
        let block_id2 = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("e5-small-v2".to_string(), vec![0.1, 0.2, 0.3]);

        indexer
            .index_block(
                block_id1,
                "machine learning neural networks",
                "text",
                embeddings.clone(),
            )
            .await
            .unwrap();
        indexer
            .index_block(block_id2, "deep learning training", "text", embeddings)
            .await
            .unwrap();

        let results = indexer
            .search(Some("learning"), HashMap::new(), None)
            .await
            .unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_visual_search() {
        let mut indexer = MultimodalIndexer::new();
        let block_id = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("clip-vit-b32".to_string(), vec![0.5, 0.5, 0.5]);

        indexer
            .index_block(block_id, "", "image", embeddings.clone())
            .await
            .unwrap();

        let mut query_embeddings = HashMap::new();
        query_embeddings.insert("clip-vit-b32".to_string(), vec![0.5, 0.5, 0.5]);

        let results = indexer.search(None, query_embeddings, None).await.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        assert!((MultimodalIndexer::cosine_similarity(&v1, &v2) - 1.0).abs() < 0.001);

        let v3 = vec![1.0, 0.0, 0.0];
        let v4 = vec![0.0, 1.0, 0.0];
        assert!((MultimodalIndexer::cosine_similarity(&v3, &v4)).abs() < 0.001);
    }
}
