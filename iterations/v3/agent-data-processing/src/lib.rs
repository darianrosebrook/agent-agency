//! Agent Data Processing Pipeline - Unified multimodal data processing for Agent Agency V3
//!
//! This crate consolidates the data processing pipeline from ingestion to knowledge integration:
//!
//! ## Pipeline Stages
//!
//! 1. **Ingestion** - Extract data from various sources (files, URLs, external APIs)
//! 2. **Enrichment** - Add semantic understanding (OCR, ASR, entity extraction, embeddings)
//! 3. **Indexing** - Create searchable indexes (vector, full-text, graph)
//! 4. **Knowledge** - Integrate with external knowledge sources (Wikidata, WordNet)
//! 5. **Operations** - Safe file/workspace operations with rollback capabilities
//! 6. **Context** - Context preservation, working memory management, and lifecycle folding
//!
//! ## Integration Hooks
//!
//! - **Agent Memory**: Store processed data, retrieve contextual memories, knowledge graphs
//! - **Workspace State**: Track processing changes, enable rollbacks, manage workspace views
//! - **Context Management**: Preserve, retrieve, and manage working memory contexts
//!
//! ## Architecture
//!
//! The pipeline uses a modular, pluggable architecture where each stage implements the
//! `PipelineStage` trait and can be composed into processing workflows.
//!
//! @author @darianrosebrook

pub mod enrichment;
pub mod indexing;
pub mod ingestion;
pub mod ingestion_runtime;
pub mod ingestion_util;
pub mod ingestion_cleanup;
pub mod knowledge;
pub mod operations;
pub mod pipeline;
pub mod data_processing_types;
pub mod context;

#[cfg(feature = "memory-integration")]
pub mod memory_hooks;

#[cfg(feature = "workspace-integration")]
pub mod workspace_hooks;

// Import schemars for JsonSchema derive
use schemars::JsonSchema;

// Re-export main types
pub use pipeline::{DataPipeline, PipelineConfig, PipelineResult};
pub use data_processing_types::ProcessingStats;
pub use data_processing_types::*;
#[cfg(feature = "embeddings")]
pub use context::{ContextManager, ContextConfig, ContextData, ContextStats};

// Re-export block and enrichment types for orchestration
pub use data_processing_types::{Block, BlockData, EnrichedBlock, EnrichedContent, ExtractedEntity, VisualElement, VisualElementType, ExtractedTopic, TextPosition};

// Re-export stage traits and implementations
pub use enrichment::{EnrichmentStage, EnrichmentResult};
pub use indexing::{IndexingStage, IndexingResult, IndexQuery, IndexResult};
pub use ingestion::{IngestionStage, IngestionResult};
pub use data_processing_types::DataSource;
pub use knowledge::{KnowledgeStage, KnowledgeResult, KnowledgeSource};
pub use operations::{OperationsStage, OperationResult, FileOperation};

// Consolidated enrichment functionality from enrichers crate
pub use enrichment::{
    AsrEnricher, VisionEnricher, EntityEnricher, VisualCaptioningEnricher,
    CircuitBreaker, CircuitState, EnrichmentCircuitBreakerConfig,
    AsrEnrichmentResult, VisionEnrichmentResult, EntityExtractionResult, VisualCaptioningResult,
    UnifiedEnrichmentStage, DefaultEnrichmentStage,
};

// Consolidated ingestion functionality from ingestors crate
pub use ingestion::{
    CaptionsIngestor, DiagramsIngestor, VideoIngestor, SlidesIngestor,
    FileWatcher, UnifiedIngestor,
};

// Consolidated indexing functionality from indexers crate
pub use indexing::{
    Bm25Indexer, HnswIndexer, DatabasePool, VectorStore, JobScheduler,
    JobType, JobPriority, JobStatus, UnifiedIndexer, SearchQuery, SearchResult,
    VectorQuery, VectorSearchResult, HybridSearchResult, UnifiedIndexerStats,
    IngestionJob, JobSchedulerStats,
};

/// Unified data processing result type
pub type DataProcessingResult<T> = Result<T, DataProcessingError>;

/// Comprehensive error type for data processing operations
#[derive(Debug, thiserror::Error)]
pub enum DataProcessingError {
    #[error("Ingestion error: {0}")]
    Ingestion(String),

    #[error("Enrichment error: {0}")]
    Enrichment(String),

    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Knowledge integration error: {0}")]
    Knowledge(String),

    #[error("File operation error: {0}")]
    Operation(String),

    #[error("Pipeline configuration error: {0}")]
    Config(String),

    #[error("Unsupported content type: {0}")]
    UnsupportedContentType(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Unknown data processing error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for DataProcessingError {
    fn from(err: anyhow::Error) -> Self {
        DataProcessingError::Other(err.to_string())
    }
}

/// Initialize the complete data processing system
pub struct DataProcessingSystem {
    pipeline: DataPipeline,
    #[cfg(feature = "memory-integration")]
    memory_hooks: memory_hooks::MemoryIntegrationHooks,
    #[cfg(feature = "workspace-integration")]
    workspace_hooks: workspace_hooks::WorkspaceIntegrationHooks,
}

impl DataProcessingSystem {
    /// Create a new data processing system with full integration
    pub async fn init(config: DataProcessingConfig) -> DataProcessingResult<Self> {
        let pipeline = DataPipeline::new(config.pipeline).await?;

        #[cfg(feature = "memory-integration")]
        let memory_hooks = memory_hooks::MemoryIntegrationHooks::new(&config.memory).await?;

        #[cfg(feature = "workspace-integration")]
        let workspace_hooks = workspace_hooks::WorkspaceIntegrationHooks::new(&config.workspace).await?;

        Ok(Self {
            pipeline,
            #[cfg(feature = "memory-integration")]
            memory_hooks,
            #[cfg(feature = "workspace-integration")]
            workspace_hooks,
        })
    }

    /// Process data through the complete pipeline
    pub async fn process_data(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        // Track workspace state before processing
        #[cfg(feature = "workspace-integration")]
        let workspace_snapshot = self.workspace_hooks.capture_pre_processing_state().await?;

        let result = self.pipeline.process(input).await;

        match &result {
            Ok(_output) => {
                // Store successful processing results in memory
                #[cfg(feature = "memory-integration")]
                self.memory_hooks.store_processing_result(_output).await?;

                // Commit workspace changes
                #[cfg(feature = "workspace-integration")]
                self.workspace_hooks.commit_processing_changes(workspace_snapshot).await?;
            }
            Err(_) => {
                // Rollback workspace changes on failure
                #[cfg(feature = "workspace-integration")]
                self.workspace_hooks.rollback_processing_changes(workspace_snapshot).await?;
            }
        }

        result
    }

    /// Query processed data using contextual retrieval
    pub async fn query_data(&self, query: DataQuery) -> DataProcessingResult<Vec<RetrievedData>> {
        // Get contextual memories from agent memory
        #[cfg(feature = "memory-integration")]
        let context_memories = self.memory_hooks.get_contextual_memories(&query).await?;

        // Perform pipeline-specific query
        let results = self.pipeline.query(query).await?;

        // Enhance results with contextual information
        #[cfg(feature = "memory-integration")]
        {
            // Convert AgentExperience to ContextualMemory once
            let contextual_memories: Vec<agent_memory::ContextualMemory> = context_memories.into_iter().map(|exp| {
                agent_memory::ContextualMemory {
                    memory: exp,
                    relevance_score: 0.8, // Default relevance
                    context_match: agent_memory::ContextMatch::Semantic,
                    reasoning_path: vec!["data_processing_context".to_string()],
                }
            }).collect();

            for result in &mut results {
                result.enhance_with_context(&contextual_memories);
            }
        }

        Ok(results)
    }

    /// Get pipeline statistics and health
    pub async fn get_stats(&self) -> DataProcessingResult<SystemStats> {
        let pipeline_stats = self.pipeline.get_stats().await?;

        #[cfg(feature = "memory-integration")]
        let memory_stats = self.memory_hooks.get_memory_stats().await?;

        #[cfg(feature = "workspace-integration")]
        let workspace_stats = self.workspace_hooks.get_workspace_stats().await?;

        Ok(SystemStats {
            pipeline: pipeline_stats,
            #[cfg(feature = "memory-integration")]
            memory: memory_stats,
            #[cfg(feature = "workspace-integration")]
            workspace: workspace_stats,
        })
    }
}

/// Configuration for the complete data processing system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct DataProcessingConfig {
    pub pipeline: PipelineConfig,
    #[cfg(feature = "memory-integration")]
    pub memory: memory_hooks::MemoryConfig,
    #[cfg(feature = "workspace-integration")]
    pub workspace: workspace_hooks::WorkspaceConfig,
}

impl Default for DataProcessingConfig {
    fn default() -> Self {
        Self {
            pipeline: PipelineConfig::default(),
            #[cfg(feature = "memory-integration")]
            memory: memory_hooks::MemoryConfig::default(),
            #[cfg(feature = "workspace-integration")]
            workspace: workspace_hooks::WorkspaceConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_data_processing_system_initialization() {
        let _temp_dir = TempDir::new().unwrap();
        let config = DataProcessingConfig::default();

        let system = DataProcessingSystem::init(config).await;
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_pipeline_stats() {
        let config = DataProcessingConfig::default();
        let system = DataProcessingSystem::init(config).await.unwrap();

        let stats = system.get_stats().await;
        assert!(stats.is_ok());
    }
}
