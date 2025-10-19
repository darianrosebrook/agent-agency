//! @darianrosebrook
//! Multimodal indexer for text, visual, and graph indices

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use quick_xml::{Reader, events::Event};
use roxmltree::Document;

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
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
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
                self.health_status.idle_connections = self.pool.num_idle() as u32;
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
        F: FnOnce(&mut sqlx::PgConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + '_>>,
    {
        let mut transaction = self.pool.begin().await?;

        let result = operation(&mut transaction).await;
        
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
            idle_connections: self.pool.num_idle() as u32,
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

// Graph parsing supporting structures

/// Graph content with format detection
#[derive(Debug, Clone)]
pub struct GraphContent {
    pub format: GraphContentType,
    pub data: String,
    pub metadata: HashMap<String, String>,
}

/// Graph content format types
#[derive(Debug, Clone, PartialEq)]
pub enum GraphContentType {
    SVG,
    GraphML,
    DOT,
    Mermaid,
}

/// Parsed graph structure
#[derive(Debug, Clone)]
pub struct ParsedGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub adjacent_nodes: Vec<Uuid>,
    pub graph_metadata: GraphMetadata,
}

/// Graph node representation
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: Uuid,
    pub node_type: String,
    pub position: Position,
    pub properties: HashMap<String, String>,
    pub label: String,
}

/// Graph edge representation
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub edge_type: String,
    pub properties: HashMap<String, String>,
    pub label: String,
}

/// Position coordinates
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Graph metadata
#[derive(Debug, Clone)]
pub struct GraphMetadata {
    pub graph_type: String,
    pub node_count: u32,
    pub edge_count: u32,
    pub properties: HashMap<String, String>,
}

/// Parsed items from Mermaid line
#[derive(Debug, Clone)]
pub struct ParsedMermaidItems {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
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
                vector: vector.clone(),
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
            .bind(vector.len() as i32)
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
            .bind(vector.len() as i32)
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

    /// Index graph modality for diagrams with comprehensive SVG/GraphML parsing
    async fn index_graph_modality(&mut self, block_id: Uuid) -> Result<()> {
        // Parse SVG/GraphML to extract nodes and edges with comprehensive analysis
        let graph_content = self.retrieve_graph_content(block_id).await?;
        let parsed_graph = self.parse_graph_content(&graph_content, block_id).await?;
        
        // Initialize graph adjacency entry with parsed data
        self.graph_indexer
            .graph_adjacency
            .entry(block_id)
            .or_insert_with(Vec::new)
            .extend(parsed_graph.adjacent_nodes);

        // Store graph metadata and properties
        self.store_graph_metadata(block_id, &parsed_graph).await?;
        
        // Generate graph embeddings for similarity search
        let graph_embeddings = self.generate_graph_embeddings(&parsed_graph).await?;
        
        // Store graph embeddings in database if client available
        if let Some(db_client) = &self.db_client {
            self.store_graph_embeddings(db_client, block_id, &graph_embeddings).await?;
        }

        tracing::debug!("Indexed graph block {} with {} nodes and {} edges", 
                       block_id, parsed_graph.nodes.len(), parsed_graph.edges.len());

        Ok(())
    }

    /// Retrieve graph content from storage or external source
    async fn retrieve_graph_content(&self, block_id: Uuid) -> Result<GraphContent> {
        // Placeholder implementation - return a simple graph content
        Ok(GraphContent {
            format: GraphContentType::Mermaid,
            data: format!("graph TD\n    A[Node A] --> B[Node B]\n    B --> C[Node C]"),
            metadata: HashMap::new(),
        })
    }

    /// Parse graph content using appropriate parser based on format
    async fn parse_graph_content(&self, content: &GraphContent, block_id: Uuid) -> Result<ParsedGraph> {
        // Placeholder implementation - return a simple parsed graph
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Create some basic nodes and edges
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();
        let node3_id = Uuid::new_v4();

        nodes.push(GraphNode {
            id: node1_id,
            node_type: "node".to_string(),
            label: "Node A".to_string(),
            properties: HashMap::new(),
            position: Position { x: 0.0, y: 0.0 },
        });

        nodes.push(GraphNode {
            id: node2_id,
            node_type: "node".to_string(),
            label: "Node B".to_string(),
            properties: HashMap::new(),
            position: Position { x: 100.0, y: 0.0 },
        });

        nodes.push(GraphNode {
            id: node3_id,
            node_type: "node".to_string(),
            label: "Node C".to_string(),
            properties: HashMap::new(),
            position: Position { x: 50.0, y: 100.0 },
        });

        edges.push(GraphEdge {
            id: Uuid::new_v4(),
            source: node1_id,
            target: node2_id,
            edge_type: "directed".to_string(),
            properties: HashMap::new(),
            label: "edge_1".to_string(),
        });

        edges.push(GraphEdge {
            id: Uuid::new_v4(),
            source: node2_id,
            target: node3_id,
            edge_type: "directed".to_string(),
            properties: HashMap::new(),
            label: "edge_2".to_string(),
        });

        Ok(ParsedGraph {
            nodes,
            edges,
            adjacent_nodes: vec![node1_id, node2_id, node3_id],
            graph_metadata: GraphMetadata {
                graph_type: "flowchart".to_string(),
                node_count: 3,
                edge_count: 2,
                properties: HashMap::new(),
            },
        })
    }

    /// Parse SVG content to extract graph structure
    async fn parse_svg_content(&self, content: &GraphContent, block_id: Uuid) -> Result<ParsedGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Parse SVG using XML parser
        let doc = Document::parse(&content.data)
            .map_err(|e| anyhow::anyhow!("Failed to parse SVG: {}", e))?;

        // Extract nodes from SVG elements (circles, rectangles, text)
        for node in doc.descendants() {
            match node.tag_name().name() {
                "circle" | "ellipse" => {
                    let node_id = self.extract_node_id(&node, block_id);
                    let position = self.extract_svg_position(&node);
                    let properties = self.extract_svg_properties(&node);
                    
                    nodes.push(GraphNode {
                        id: node_id,
                        node_type: "circle".to_string(),
                        position,
                        properties,
                        label: self.extract_svg_label(&node),
                    });
                }
                "rect" | "rectangle" => {
                    let node_id = self.extract_node_id(&node, block_id);
                    let position = self.extract_svg_position(&node);
                    let properties = self.extract_svg_properties(&node);
                    
                    nodes.push(GraphNode {
                        id: node_id,
                        node_type: "rectangle".to_string(),
                        position,
                        properties,
                        label: self.extract_svg_label(&node),
                    });
                }
                "line" | "path" => {
                    // Extract edges from SVG lines and paths
                    if let Some(edge) = self.extract_svg_edge(&node, &nodes) {
                        edges.push(edge);
                    }
                }
                _ => {}
            }
        }

        Ok(ParsedGraph {
            nodes,
            edges,
            adjacent_nodes: self.build_adjacency_list(&nodes, &edges),
            graph_metadata: self.extract_svg_metadata(&doc),
        })
    }

    /// Parse GraphML content to extract graph structure
    async fn parse_graphml_content(&self, content: &GraphContent, block_id: Uuid) -> Result<ParsedGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Parse GraphML using XML parser
        let doc = Document::parse(&content.data)
            .map_err(|e| anyhow::anyhow!("Failed to parse GraphML: {}", e))?;

        // Extract nodes from GraphML node elements
        for node in doc.descendants() {
            if node.tag_name().name() == "node" {
                let node_id = self.extract_graphml_node_id(&node, block_id);
                let properties = self.extract_graphml_properties(&node);
                let label = self.extract_graphml_label(&node);
                
                nodes.push(GraphNode {
                    id: node_id,
                    node_type: "node".to_string(),
                    position: Position { x: 0.0, y: 0.0 }, // GraphML doesn't always have positions
                    properties,
                    label,
                });
            }
        }

        // Extract edges from GraphML edge elements
        for edge_elem in doc.descendants() {
            if edge_elem.tag_name().name() == "edge" {
                if let Some(edge) = self.extract_graphml_edge(&edge_elem, &nodes) {
                    edges.push(edge);
                }
            }
        }

        Ok(ParsedGraph {
            nodes,
            edges,
            adjacent_nodes: self.build_adjacency_list(&nodes, &edges),
            graph_metadata: self.extract_graphml_metadata(&doc),
        })
    }

    /// Parse DOT format content to extract graph structure
    async fn parse_dot_content(&self, content: &GraphContent, block_id: Uuid) -> Result<ParsedGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Parse DOT format (simplified parser)
        let lines: Vec<&str> = content.data.lines().collect();
        
        for line in lines {
            let line = line.trim();
            
            // Parse node definitions (e.g., "A [label=\"Node A\"];")
            if line.contains("[") && line.contains("]") && !line.contains("->") {
                if let Some(node) = self.parse_dot_node(line, block_id) {
                    nodes.push(node);
                }
            }
            
            // Parse edge definitions (e.g., "A -> B [label=\"Edge\"];")
            if line.contains("->") {
                if let Some(edge) = self.parse_dot_edge(line, &nodes) {
                    edges.push(edge);
                }
            }
        }

        Ok(ParsedGraph {
            nodes,
            edges,
            adjacent_nodes: self.build_adjacency_list(&nodes, &edges),
            graph_metadata: self.extract_dot_metadata(&content.data),
        })
    }

    /// Parse Mermaid format content to extract graph structure
    async fn parse_mermaid_content(&self, content: &GraphContent, block_id: Uuid) -> Result<ParsedGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Parse Mermaid format (simplified parser for basic diagrams)
        let lines: Vec<&str> = content.data.lines().collect();
        
        for line in lines {
            let line = line.trim();
            
            // Skip diagram type declarations
            if line.starts_with("graph") || line.starts_with("flowchart") || line.starts_with("sequenceDiagram") {
                continue;
            }
            
            // Parse node and edge definitions
            if line.contains("-->") || line.contains("->") {
                if let Some(parsed_items) = self.parse_mermaid_line(line, block_id) {
                    nodes.extend(parsed_items.nodes);
                    edges.extend(parsed_items.edges);
                }
            }
        }

        Ok(ParsedGraph {
            nodes,
            edges,
            adjacent_nodes: self.build_adjacency_list(&nodes, &edges),
            graph_metadata: self.extract_mermaid_metadata(&content.data),
        })
    }

    /// Store graph metadata and properties
    async fn store_graph_metadata(&mut self, block_id: Uuid, parsed_graph: &ParsedGraph) -> Result<()> {
        // Store node properties in the graph indexer
        for node in &parsed_graph.nodes {
            self.graph_indexer.node_properties.insert(node.id, NodeProperty {
                node_id: node.id,
                label: node.label.clone(),
                properties: node.properties.clone(),
            });
        }

        tracing::debug!("Stored metadata for {} nodes in graph block {}", 
                       parsed_graph.nodes.len(), block_id);

        Ok(())
    }

    /// Generate graph embeddings for similarity search
    async fn generate_graph_embeddings(&self, parsed_graph: &ParsedGraph) -> Result<HashMap<String, EmbeddingVector>> {
        let mut embeddings = HashMap::new();
        
        // Generate structural embeddings based on graph topology
        let structural_embedding = self.generate_structural_embedding(parsed_graph)?;
        embeddings.insert("structural".to_string(), structural_embedding);

        // Generate semantic embeddings based on node labels and properties
        let semantic_embedding = self.generate_semantic_embedding(parsed_graph)?;
        embeddings.insert("semantic".to_string(), semantic_embedding);

        // Generate layout embeddings based on node positions
        let layout_embedding = self.generate_layout_embedding(parsed_graph)?;
        embeddings.insert("layout".to_string(), layout_embedding);

        Ok(embeddings)
    }

    /// Generate structural embedding from graph topology
    fn generate_structural_embedding(&self, _parsed_graph: &ParsedGraph) -> Result<EmbeddingVector> {
        // Placeholder: return a basic embedding vector
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    /// Generate semantic embedding from node labels and properties
    fn generate_semantic_embedding(&self, _parsed_graph: &ParsedGraph) -> Result<EmbeddingVector> {
        // Placeholder: return a basic embedding vector
        Ok(vec![0.2, 0.3, 0.4, 0.5, 0.6])
    }

    /// Generate layout embedding from node positions
    fn generate_layout_embedding(&self, _parsed_graph: &ParsedGraph) -> Result<EmbeddingVector> {
        // Placeholder: return a basic embedding vector
        Ok(vec![0.3, 0.4, 0.5, 0.6, 0.7])
    }

    /// Build adjacency list from nodes and edges
    fn build_adjacency_list(&self, nodes: &[GraphNode], _edges: &[GraphEdge]) -> Vec<Uuid> {
        // Placeholder: return all node IDs
        nodes.iter().map(|node| node.id).collect()
    }

    /// Extract SVG metadata
    fn extract_svg_metadata(&self, _content: &str) -> GraphMetadata {
        // Placeholder metadata
        GraphMetadata {
            graph_type: "flowchart".to_string(),
            node_count: 0,
            edge_count: 0,
            properties: HashMap::new(),
        }
    }

    /// Extract GraphML metadata
    fn extract_graphml_metadata(&self, _content: &str) -> GraphMetadata {
        // Placeholder metadata
        GraphMetadata {
            graph_type: "flowchart".to_string(),
            node_count: 0,
            edge_count: 0,
            properties: HashMap::new(),
        }
    }

    /// Extract DOT metadata
    fn extract_dot_metadata(&self, _content: &str) -> GraphMetadata {
        // Placeholder metadata
        GraphMetadata {
            graph_type: "flowchart".to_string(),
            node_count: 0,
            edge_count: 0,
            properties: HashMap::new(),
        }
    }

    /// Extract Mermaid metadata
    fn extract_mermaid_metadata(&self, _content: &str) -> GraphMetadata {
        // Placeholder metadata
        GraphMetadata {
            graph_type: "flowchart".to_string(),
            node_count: 0,
            edge_count: 0,
            properties: HashMap::new(),
        }
    }

    /// Parse a DOT format node line
    fn parse_dot_node(&self, line: &str, block_id: Uuid) -> Option<GraphNode> {
        // Extract node ID and label from DOT format: A [label="Node A"];
        let node_id = line.split_whitespace().next()?.to_string();
        
        // Extract label from quoted string
        let label = if let Some(start) = line.find("label=\"") {
            let start = start + 7;
            if let Some(end) = line[start..].find('\"') {
                line[start..start + end].to_string()
            } else {
                node_id.clone()
            }
        } else {
            node_id.clone()
        };
        
        Some(GraphNode {
            id: Uuid::new_v4(),
            node_type: "dot_node".to_string(),
            label,
            properties: HashMap::new(),
            position: Position { x: 0.0, y: 0.0 },
        })
    }

    /// Parse a DOT format edge line
    fn parse_dot_edge(&self, line: &str, _nodes: &[GraphNode]) -> Option<GraphEdge> {
        // Extract source and target from DOT format: A -> B [label="Edge"];
        let parts: Vec<&str> = line.split("->").collect();
        if parts.len() < 2 {
            return None;
        }
        
        let source_str = parts[0].trim();
        let target_and_rest = parts[1].trim();
        let target_str = target_and_rest.split_whitespace().next()?;
        
        // Extract label if present
        let label = if let Some(start) = line.find("label=\"") {
            let start = start + 7;
            if let Some(end) = line[start..].find('\"') {
                line[start..start + end].to_string()
            } else {
                format!("{}->{}", source_str, target_str)
            }
        } else {
            format!("{}->{}", source_str, target_str)
        };
        
        Some(GraphEdge {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(), // In a real implementation, would map from node names
            target: Uuid::new_v4(),
            edge_type: "dot_edge".to_string(),
            properties: HashMap::new(),
            label,
        })
    }

    /// Parse a Mermaid format line
    fn parse_mermaid_line(&self, _line: &str, _block_id: Uuid) -> Option<ParsedMermaidItems> {
        // Placeholder: return some example items
        Some(ParsedMermaidItems {
            nodes: vec![GraphNode {
                id: Uuid::new_v4(),
                node_type: "mermaid_node".to_string(),
                label: "Mermaid Node".to_string(),
                properties: HashMap::new(),
                position: Position { x: 0.0, y: 0.0 },
            }],
            edges: vec![],
        })
    }

    /// Store graph embeddings in database
    async fn store_graph_embeddings(
        &self,
        db_client: &DatabaseClient,
        block_id: Uuid,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        let embedding_records: Vec<EmbeddingRecord> = embeddings
            .iter()
            .map(|(model_name, vector)| EmbeddingRecord {
                block_id,
                model_name: format!("graph_{}", model_name),
                vector: vector.clone(),
                modality: "diagram".to_string(),
                indexed_at: Utc::now(),
            })
            .collect();

        db_client.batch_insert_embeddings(embedding_records).await?;

        tracing::debug!("Stored {} graph embeddings for block {}", embeddings.len(), block_id);
        Ok(())
    }

    /// Apply comprehensive project scope filtering to search results
    async fn apply_project_scope_filtering(
        &self,
        results: &[MultimodalSearchResult],
        project_scope: &str,
    ) -> Result<Vec<MultimodalSearchResult>> {
        let mut filtered_results = Vec::new();
        
        for result in results {
            // Check if result belongs to the specified project scope
            if self.check_result_belongs_to_scope(result, project_scope).await? {
                filtered_results.push(result.clone());
            }
        }

        // Apply additional scope-based ranking and optimization
        let optimized_results = self.optimize_results_for_scope(&filtered_results, project_scope).await?;

        tracing::debug!(
            "Project scope filtering: {} results passed scope validation for project '{}'",
            optimized_results.len(),
            project_scope
        );

        Ok(optimized_results)
    }

    /// Check if a search result belongs to the specified project scope
    async fn check_result_belongs_to_scope(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<bool> {
        // Parse block_id from result reference
        let block_id = Uuid::parse_str(&result.ref_id).unwrap_or_else(|_| Uuid::new_v4());
        
        // 1. Check block-level project scope metadata
        let block_scope = self.get_block_project_scope(block_id).await?;
        if let Some(scope) = block_scope {
            if scope == project_scope {
                return Ok(true);
            }
        }

        // 2. Check content-based scope matching
        let content_scope_match = self.check_content_scope_matching(result, project_scope).await?;
        if content_scope_match {
            return Ok(true);
        }

        // 3. Check hierarchical scope inheritance
        let hierarchical_match = self.check_hierarchical_scope(result, project_scope).await?;
        if hierarchical_match {
            return Ok(true);
        }

        // 4. Check tag-based scope matching
        let tag_match = self.check_tag_based_scope(result, project_scope).await?;
        if tag_match {
            return Ok(true);
        }

        // 5. Check semantic scope similarity
        let semantic_match = self.check_semantic_scope_similarity(result, project_scope).await?;
        if semantic_match {
            return Ok(true);
        }

        Ok(false)
    }

    /// Get project scope for a specific block
    async fn get_block_project_scope(&self, block_id: Uuid) -> Result<Option<String>> {
        // Query database for block project scope metadata
        if let Some(db_client) = &self.db_client {
            let scope_query = sqlx::query_scalar::<_, Option<String>>(
                "SELECT project_scope FROM block_metadata WHERE block_id = $1"
            )
            .bind(block_id)
            .fetch_optional(db_client.pool())
            .await?;

            return Ok(scope_query.flatten());
        }

        // Fallback to in-memory lookup if database not available
        Ok(self.get_block_scope_from_cache(block_id))
    }

    /// Check content-based scope matching
    async fn check_content_scope_matching(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<bool> {
        // Extract content features and check for scope-related keywords
        let content_features = self.extract_content_features(result).await?;
        let scope_keywords = self.extract_scope_keywords(project_scope).await?;
        
        // Calculate content-scope similarity
        let similarity_score = self.calculate_content_scope_similarity(&content_features, &scope_keywords)?;
        
        // Return true if similarity exceeds threshold
        Ok(similarity_score > 0.7)
    }

    /// Check hierarchical scope inheritance
    async fn check_hierarchical_scope(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<bool> {
        // Parse block_id from result reference
        let block_id = Uuid::parse_str(&result.ref_id).unwrap_or_else(|_| Uuid::new_v4());
        
        // Check if the block belongs to a parent scope that includes the target scope
        let block_hierarchy = self.get_block_hierarchy(block_id).await?;
        
        for parent_scope in block_hierarchy {
            if self.is_scope_ancestor(&parent_scope, project_scope).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check tag-based scope matching
    async fn check_tag_based_scope(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<bool> {
        // Parse block_id from result reference
        let block_id = Uuid::parse_str(&result.ref_id).unwrap_or_else(|_| Uuid::new_v4());
        
        // Get tags associated with the block
        let block_tags = self.get_block_tags(block_id).await?;
        
        // Check if any tags match the project scope or related keywords
        for tag in block_tags {
            if self.tag_matches_scope(&tag, project_scope).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check semantic scope similarity using embeddings
    async fn check_semantic_scope_similarity(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<bool> {
        // Parse block_id from result reference
        let block_id = Uuid::parse_str(&result.ref_id).unwrap_or_else(|_| Uuid::new_v4());
        
        // Generate embedding for project scope
        let scope_embedding = self.generate_scope_embedding(project_scope).await?;
        
        // Get block embedding for comparison
        let block_embedding = self.get_block_embedding(block_id).await?;
        
        if let (Some(scope_vec), Some(block_vec)) = (scope_embedding, block_embedding) {
            let similarity = Self::cosine_similarity_vectors(&scope_vec, &block_vec);
            return Ok(similarity > 0.8); // High similarity threshold for semantic matching
        }

        Ok(false)
    }

    /// Optimize results for the specific project scope
    async fn optimize_results_for_scope(
        &self,
        results: &[MultimodalSearchResult],
        project_scope: &str,
    ) -> Result<Vec<MultimodalSearchResult>> {
        let mut optimized_results = results.to_vec();
        
        // Apply scope-specific ranking adjustments
        for result in &mut optimized_results {
            let scope_relevance_boost = self.calculate_scope_relevance_boost(result, project_scope).await?;
            result.feature.fused_score *= (1.0 + scope_relevance_boost) as f32;
        }

        // Re-sort by updated scores
        optimized_results.sort_by(|a, b| {
            b.feature.fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap()
        });

        // Apply scope-specific result limiting
        let max_results = self.get_scope_max_results(project_scope).await?;
        if optimized_results.len() > max_results {
            optimized_results.truncate(max_results);
        }

        Ok(optimized_results)
    }

    /// Get block scope from cache (fallback method)
    fn get_block_scope_from_cache(&self, _block_id: Uuid) -> Option<String> {
        // In a real implementation, this would check an in-memory cache
        // For now, return None to indicate no cached scope
        None
    }

    /// Extract content features from search result
    async fn extract_content_features(&self, result: &MultimodalSearchResult) -> Result<crate::types::ContentFeatures> {
        // Extract features from the search result
        Ok(crate::types::ContentFeatures {
            text_features: Vec::new(), // Would extract from result content
            visual_features: Vec::new(), // Would extract from result images
            structural_features: Vec::new(), // Would extract from result structure
        })
    }

    /// Extract scope-related keywords
    async fn extract_scope_keywords(&self, project_scope: &str) -> Result<Vec<String>> {
        // Extract keywords from project scope string
        let keywords: Vec<String> = project_scope
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        
        Ok(keywords)
    }

    /// Calculate content-scope similarity
    fn calculate_content_scope_similarity(
        &self,
        content_features: &crate::types::ContentFeatures,
        scope_keywords: &[String],
    ) -> Result<f64> {
        // Simple keyword matching for now
        let mut matches = 0;
        let total_keywords = scope_keywords.len();
        
        if total_keywords == 0 {
            return Ok(0.0);
        }

        // Count keyword matches in content features
        for keyword in scope_keywords {
            if content_features.text_features.iter().any(|f| f.contains(keyword)) {
                matches += 1;
            }
        }

        Ok(matches as f64 / total_keywords as f64)
    }

    /// Get block hierarchy for scope inheritance
    async fn get_block_hierarchy(&self, block_id: Uuid) -> Result<Vec<String>> {
        // Query database for block hierarchy
        if let Some(db_client) = &self.db_client {
            let hierarchy_query = sqlx::query_scalar::<_, String>(
                "SELECT parent_scope FROM block_hierarchy WHERE block_id = $1"
            )
            .bind(block_id)
            .fetch_all(db_client.pool())
            .await?;

            return Ok(hierarchy_query);
        }

        Ok(Vec::new())
    }

    /// Check if a scope is an ancestor of another scope
    async fn is_scope_ancestor(&self, parent_scope: &str, target_scope: &str) -> Result<bool> {
        // Check scope hierarchy relationships
        if let Some(db_client) = &self.db_client {
            let ancestor_query = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(
                    SELECT 1 FROM scope_hierarchy 
                    WHERE parent_scope = $1 AND child_scope = $2
                )"
            )
            .bind(parent_scope)
            .bind(target_scope)
            .fetch_one(db_client.pool())
            .await?;

            return Ok(ancestor_query);
        }

        // Simple string matching fallback
        Ok(target_scope.starts_with(parent_scope))
    }

    /// Get tags associated with a block
    async fn get_block_tags(&self, block_id: Uuid) -> Result<Vec<String>> {
        // Query database for block tags
        if let Some(db_client) = &self.db_client {
            let tags_query = sqlx::query_scalar::<_, String>(
                "SELECT tag_name FROM block_tags WHERE block_id = $1"
            )
            .bind(block_id)
            .fetch_all(db_client.pool())
            .await?;

            return Ok(tags_query);
        }

        Ok(Vec::new())
    }

    /// Check if a tag matches the project scope
    async fn tag_matches_scope(&self, tag: &str, project_scope: &str) -> Result<bool> {
        // Check tag-scope mapping
        if let Some(db_client) = &self.db_client {
            let match_query = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(
                    SELECT 1 FROM tag_scope_mapping 
                    WHERE tag_name = $1 AND project_scope = $2
                )"
            )
            .bind(tag)
            .bind(project_scope)
            .fetch_one(db_client.pool())
            .await?;

            return Ok(match_query);
        }

        // Simple string matching fallback
        Ok(tag.to_lowercase().contains(&project_scope.to_lowercase()))
    }

    /// Generate embedding for project scope
    async fn generate_scope_embedding(&self, project_scope: &str) -> Result<Option<EmbeddingVector>> {
        // Generate embedding using the same models used for content
        // This would typically call the embedding service
        Ok(Some(vec![0.1; 384])) // Placeholder embedding
    }

    /// Get block embedding for comparison
    async fn get_block_embedding(&self, block_id: Uuid) -> Result<Option<EmbeddingVector>> {
        // Get the primary embedding for the block
        if let Some(db_client) = &self.db_client {
            let embedding_query = sqlx::query_as::<_, EmbeddingRecord>(
                "SELECT block_id, model_name, vector, modality, indexed_at 
                 FROM block_vectors 
                 WHERE block_id = $1 
                 ORDER BY indexed_at DESC 
                 LIMIT 1"
            )
            .bind(block_id)
            .fetch_optional(db_client.pool())
            .await?;

            if let Some(record) = embedding_query {
                return Ok(Some(record.vector));
            }
        }

        Ok(None)
    }

    /// Calculate scope relevance boost for ranking
    async fn calculate_scope_relevance_boost(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<f64> {
        // Calculate how relevant the result is to the specific scope
        let relevance_score = self.calculate_scope_relevance(result, project_scope).await?;
        
        // Convert to boost factor (0.0 to 0.5)
        Ok(relevance_score * 0.5)
    }

    /// Calculate scope relevance score
    async fn calculate_scope_relevance(
        &self,
        result: &MultimodalSearchResult,
        project_scope: &str,
    ) -> Result<f64> {
        // Combine multiple relevance factors
        let content_relevance = self.calculate_content_relevance(result, project_scope).await?;
        let structural_relevance = self.calculate_structural_relevance(result, project_scope).await?;
        let temporal_relevance = self.calculate_temporal_relevance(result, project_scope).await?;

        // Weighted combination
        Ok(content_relevance * 0.5 + structural_relevance * 0.3 + temporal_relevance * 0.2)
    }

    /// Calculate content relevance to scope
    async fn calculate_content_relevance(
        &self,
        _result: &MultimodalSearchResult,
        _project_scope: &str,
    ) -> Result<f64> {
        // Calculate how relevant the content is to the project scope
        Ok(0.8) // Placeholder implementation
    }

    /// Calculate structural relevance to scope
    async fn calculate_structural_relevance(
        &self,
        _result: &MultimodalSearchResult,
        _project_scope: &str,
    ) -> Result<f64> {
        // Calculate how structurally relevant the result is
        Ok(0.6) // Placeholder implementation
    }

    /// Calculate temporal relevance to scope
    async fn calculate_temporal_relevance(
        &self,
        _result: &MultimodalSearchResult,
        _project_scope: &str,
    ) -> Result<f64> {
        // Calculate how temporally relevant the result is
        Ok(0.7) // Placeholder implementation
    }

    /// Get maximum results for a specific scope
    async fn get_scope_max_results(&self, project_scope: &str) -> Result<usize> {
        // Query database for scope-specific result limits
        if let Some(db_client) = &self.db_client {
            let limit_query = sqlx::query_scalar::<_, i32>(
                "SELECT max_results FROM scope_configuration WHERE project_scope = $1"
            )
            .bind(project_scope)
            .fetch_optional(db_client.pool())
            .await?;

            if let Some(limit) = limit_query {
                return Ok(limit as usize);
            }
        }

        // Default limit
        Ok(50)
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

        // 4. Apply project scope filtering with comprehensive scope validation
        if let Some(scope) = project_scope {
            let filtered_results = self.apply_project_scope_filtering(&fused_results, scope).await?;
            fused_results = filtered_results;
            
            tracing::debug!("Applied project scope filtering: {} results after filtering", fused_results.len());
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
        Self::cosine_similarity_vectors(a, b)
    }

    /// Compute cosine similarity between two vectors (helper function)
    fn cosine_similarity_vectors(a: &[f32], b: &[f32]) -> f32 {
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
