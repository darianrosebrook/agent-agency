//! Multimodal orchestration for document processing pipeline
//! 
//! Coordinates ingestors, enrichers, and indexers for multimodal RAG system
//! with proper error handling, concurrency control, and monitoring.

use anyhow::{Context, Result};
// TODO: Re-enable when agent_data_processing dependency is added
// use agent_data_processing::{
//     ingestion::{IngestionStage, UnifiedIngestor, CaptionsIngestor, DiagramsIngestor, VideoIngestor, SlidesIngestor, FileWatcher},
//     enrichment::{EnrichmentStage, UnifiedEnrichmentStage, VisionEnricher, AsrEnricher, EntityEnricher, VisualCaptioningEnricher, CircuitBreaker},
//     indexing::{IndexingStage, UnifiedIndexer, Bm25Indexer, HnswIndexer, JobScheduler},
//     Block, EnrichedBlock, BlockData, EnrichedContent, ExtractedEntity, VisualElement, VisualElementType, ExtractedTopic,
//     DataInput, DataSource, ContentType,
// };

// Temporary stub types until agent_data_processing is available
#[derive(Debug)]
pub struct UnifiedIngestor;
#[derive(Debug)]
pub struct FileWatcher;
#[derive(Debug)]
pub struct UnifiedEnrichmentStage;
#[derive(Debug)]
pub struct UnifiedIndexer;
#[derive(Debug)]
pub struct JobScheduler;
#[derive(Debug)]
pub struct CircuitBreaker;
use crate::coreml::{CoreMLManager, CoreMLModelType, InferenceResult};
// Stub types until agent_data_processing is available
pub struct Block {
    pub id: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct EnrichedBlock {
    pub id: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct BlockData {
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct EnrichedContent {
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
}

pub struct VisualElement {
    pub element_type: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub enum VisualElementType {
    Diagram,
    Chart,
    Image,
    Table,
}

pub struct ExtractedTopic {
    pub topic: String,
    pub confidence: f64,
}

pub struct DataInput {
    pub id: String,
    pub source: DataSource,
    pub content: String,
    pub processing_context: ProcessingContext,
}

pub enum DataSource {
    File(String),
    Url(String),
    Text(String),
}

pub struct ProcessingContext {
    pub priority: ProcessingPriority,
    pub metadata: std::collections::HashMap<String, String>,
}

pub enum ProcessingPriority {
    Low,
    Normal,
    High,
    Critical,
}

pub enum ContentType {
    Text,
    Image,
    Video,
    Audio,
    Document,
}

pub struct ProcessingId(String);

impl ProcessingId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

pub struct ProcessingOutput {
    pub id: ProcessingId,
    pub blocks: Vec<Block>,
    pub metadata: std::collections::HashMap<String, String>,
}

// Use available crates instead
// ConsensusCoordinator is not available in contracts, use placeholder
pub type ConsensusCoordinator = String;

// Placeholder types for missing modules
pub type KnowledgeSeeker = String;
pub type OrchestratorConfig = String;
use crate::audit_trail::AuditTrailManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use serde_json;

/// Multimodal document processing orchestrator
#[derive(Debug)]
pub struct MultimodalOrchestrator {
    /// Unified ingestor for all content types
    unified_ingestor: UnifiedIngestor,
    /// File watcher for monitoring directories
    file_watcher: FileWatcher,
    /// Enrichers for content enhancement
    unified_enricher: UnifiedEnrichmentStage,
    /// Unified indexer for search capabilities
    unified_indexer: UnifiedIndexer,
    /// Job scheduler for coordination
    job_scheduler: JobScheduler,
    /// Circuit breaker for resilience
    circuit_breaker: CircuitBreaker,
    /// Core ML model manager for accelerated inference
    coreml_manager: Option<Arc<CoreMLManager>>,
    /// Knowledge seeker for research integration
    knowledge_seeker: Option<Arc<KnowledgeSeeker>>,
    /// Council coordinator for decision-making
    council_coordinator: Option<Arc<ConsensusCoordinator>>,
    /// Audit trail manager for recording processing events
    audit_trail: Option<Arc<AuditTrailManager>>,
}

/// Processing result for document pipeline
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Document identifier
    pub document_id: Uuid,
    /// Processing status
    pub status: ProcessingStatus,
    /// Number of blocks processed
    pub blocks_processed: usize,
    /// Number of blocks enriched
    pub blocks_enriched: usize,
    /// Number of blocks indexed
    pub blocks_indexed: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Error message if processing failed
    pub error_message: Option<String>,
}

/// Processing status
#[derive(Debug, Clone)]
pub enum ProcessingStatus {
    /// Processing completed successfully
    Completed,
    /// Processing failed
    Failed,
    /// Processing in progress
    InProgress,
    /// Processing skipped (e.g., unsupported format)
    Skipped,
    /// Processing pending (queued)
    Pending,
    /// Processing actively running
    Running,
    /// Processing cancelled by user/system
    Cancelled,
}

impl MultimodalOrchestrator {
    /// Create new multimodal orchestrator
    pub async fn new() -> Result<Self> {
        // Initialize unified components
        let unified_ingestor = UnifiedIngestor::new();
        let unified_enricher = UnifiedEnrichmentStage::new().await?;
        let unified_indexer = UnifiedIndexer::new(768, 32); // 768-dim embeddings, 32 neighbors

        // Initialize Core ML manager
        let coreml_manager = {
            let manager = Arc::new(CoreMLManager::new(
                std::env::var("COREML_MODELS_PATH")
                    .map(|p| PathBuf::from(p))
                    .unwrap_or_else(|_| PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml"))
            ));

            // Try to load available models
            if let Err(e) = manager.load_available_models().await {
                warn!("Failed to load Core ML models: {}", e);
            } else {
                info!("Core ML models loaded successfully");
            }

            Some(manager)
        };

        Ok(Self {
            unified_ingestor,
            file_watcher: FileWatcher::new(vec![], vec![]),
            unified_enricher,
            unified_indexer,
            job_scheduler: JobScheduler::new(),
            circuit_breaker: CircuitBreaker::new(),
            coreml_manager,
            knowledge_seeker: None,
            council_coordinator: None,
            audit_trail: None,
        })
    }

    /// Set knowledge seeker for research integration
    pub fn set_knowledge_seeker(&mut self, knowledge_seeker: Arc<KnowledgeSeeker>) {
        self.knowledge_seeker = Some(knowledge_seeker);
    }

    /// Set audit trail manager for event recording
    pub fn set_audit_trail(&mut self, audit_trail: Arc<AuditTrailManager>) {
        self.audit_trail = Some(audit_trail);
    }

    /// Set council coordinator for decision-making
    pub fn set_council_coordinator(&mut self, coordinator: Arc<ConsensusCoordinator>) {
        self.council_coordinator = Some(coordinator);
    }

    /// Orchestrate document processing pipeline
    ///
    /// # Arguments
    /// * `file_path` - Path to document to process
    ///
    /// # Returns
    /// Processing result with statistics
    pub async fn orchestrate_document_processing(
        &self,
        file_path: &Path,
    ) -> Result<ProcessingResult> {
        let document_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        
        info!("Starting multimodal document processing: {} (id: {})", file_path.display(), document_id);

        // Check circuit breaker state
        if self.circuit_breaker.is_open() {
            warn!("Circuit breaker is open, skipping processing");
            return Ok(ProcessingResult {
                document_id,
                status: ProcessingStatus::Skipped,
                blocks_processed: 0,
                blocks_enriched: 0,
                blocks_indexed: 0,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some("Circuit breaker open".to_string()),
            });
        }

        // Stage 1: Ingest document using UnifiedIngestor
        let file_metadata = std::fs::metadata(file_path)
            .context("Failed to read file metadata")?;
        
        let content_type = detect_content_type_from_path(file_path);
        
        let data_input = DataInput {
            id: ProcessingId::new().0,
            source: DataSource::File(file_path.to_string()),
            content: file_path.to_string(),
            processing_context: ProcessingContext {
                priority: ProcessingPriority::Normal,
                metadata: std::collections::HashMap::new(),
            },
        };

        let ingestion_output = self.unified_ingestor
            .ingest(data_input)
            .await
            .context("Failed to ingest document")?;

        // Convert ingestion output to blocks for processing
        let blocks = convert_ingestion_output_to_blocks(ingestion_output)
            .context("Failed to convert ingestion output to blocks")?;

        let blocks_processed = blocks.len();
        info!("Ingested {} blocks from {}", blocks_processed, file_path.display());

        // Stage 2: Enrich blocks with multimodal content
        let enriched_blocks = self.enrich_blocks(&blocks).await?;
        let blocks_enriched = enriched_blocks.len();
        info!("Enriched {} blocks", blocks_enriched);

        // Stage 3: Index enriched content
        let blocks_indexed = self.index_blocks(&enriched_blocks).await?;
        info!("Indexed {} blocks", blocks_indexed);

        // Record success in circuit breaker
        self.circuit_breaker.record_success();

        let processing_time = start_time.elapsed().as_millis() as u64;
        info!(
            "Completed multimodal document processing: {} ({}ms, {} blocks)",
            file_path.display(),
            processing_time,
            blocks_processed
        );

        Ok(ProcessingResult {
            document_id,
            status: ProcessingStatus::Completed,
            blocks_processed,
            blocks_enriched,
            blocks_indexed,
            processing_time_ms: processing_time,
            error_message: None,
        })
    }

    /// Watch directory for new documents
    ///
    /// # Arguments
    /// * `directory_path` - Directory to watch
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn watch_directory(&self, directory_path: &Path) -> Result<()> {
        info!("Starting directory watch: {}", directory_path.display());
        
        self.file_watcher
            .watch(directory_path)
            .await
            .context("Failed to start directory watching")?;

        info!("Directory watch started successfully");
        Ok(())
    }

    /// Process multiple documents in parallel
    ///
    /// # Arguments
    /// * `file_paths` - List of file paths to process
    /// * `max_concurrent` - Maximum concurrent processing jobs
    ///
    /// # Returns
    /// Vector of processing results
    pub async fn process_documents_parallel(
        &self,
        file_paths: &[&Path],
        max_concurrent: usize,
    ) -> Result<Vec<ProcessingResult>> {
        info!("Processing {} documents with max concurrency: {}", file_paths.len(), max_concurrent);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut tasks = Vec::new();

        for file_path in file_paths {
            let semaphore = semaphore.clone();
            let file_path_str = file_path.to_string_lossy().to_string();
            let audit_trail = self.audit_trail.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Record document processing started
                if let Some(audit) = &audit_trail {
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert("file_path".to_string(), serde_json::Value::String(file_path_str.clone()));
                    metadata.insert("event_type".to_string(), serde_json::Value::String("started".to_string()));
                    let _ = audit.record_operation_performance(
                        "document_processing",
                        std::time::Duration::from_millis(0),
                        true,
                        metadata
                    ).await;
                }

                let start_time = std::time::Instant::now();
                // Use orchestrator's orchestrate_document_processing which has full integration
                let result = self.orchestrate_document_processing(file_path).await;

                // Record document processing finished or error
                if let Some(audit) = &audit_trail {
                    let success = result.is_ok();
                    let event_type = if success { "finished" } else { "error" };
                    let processing_time = start_time.elapsed();

                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert("file_path".to_string(), serde_json::Value::String(file_path_str.clone()));
                    metadata.insert("event_type".to_string(), serde_json::Value::String(event_type.to_string()));

                    match &result {
                        Ok(result) => {
                            metadata.insert("document_id".to_string(), serde_json::Value::String(result.document_id.to_string()));
                            metadata.insert("blocks_processed".to_string(), serde_json::Value::Number(result.blocks_processed.into()));
                            metadata.insert("blocks_enriched".to_string(), serde_json::Value::Number(result.blocks_enriched.into()));
                            metadata.insert("blocks_indexed".to_string(), serde_json::Value::Number(result.blocks_indexed.into()));
                            metadata.insert("processing_time_ms".to_string(), serde_json::Value::Number(result.processing_time_ms.into()));
                        }
                        Err(e) => {
                            metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                        }
                    }

                    let _ = audit.record_operation_performance(
                        "document_processing",
                        processing_time,
                        success,
                        metadata
                    ).await;
                }

                result
            });
            
            tasks.push(task);
        }

        let mut results: Vec<ProcessingResult> = Vec::new();
        for task in tasks {
            let result = task.await.context("Task execution failed")??;
            results.push(result);
        }

        info!("Completed parallel processing of {} documents", results.len());
        Ok(results)
    }

    /// Get processing statistics
    ///
    /// # Returns
    /// Statistics about the orchestrator
    pub async fn get_processing_stats(&self) -> Result<ProcessingStats> {
        let stats = ProcessingStats {
            total_documents_processed: 0, // Would be tracked in real implementation
            total_blocks_processed: 0,
            total_blocks_enriched: 0,
            total_blocks_indexed: 0,
            average_processing_time_ms: 0,
            circuit_breaker_state: self.circuit_breaker.state(),
            active_jobs: self.job_scheduler.get_active_job_count(),
        };

        Ok(stats)
    }

    // Helper methods

    /// Enrich blocks with multimodal content
    async fn enrich_blocks(&self, blocks: &[Block]) -> Result<Vec<EnrichedBlock>> {
        // Use unified enricher to handle enrichment
        let enriched = self.unified_enricher
            .enrich_blocks(blocks)
            .await
            .context("Failed to enrich blocks")?;
        
        Ok(enriched)
    }

    /// Index enriched blocks
    async fn index_blocks(&self, blocks: &[EnrichedBlock]) -> Result<usize> {
        // Use unified indexer to handle indexing
        let indexed = self.unified_indexer
            .index_blocks(blocks)
            .await
            .context("Failed to index blocks")?;
        
        Ok(indexed)
    }
}

/// Detect content type from file path
fn detect_content_type_from_path(path: &Path) -> ContentType {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        // Video formats
        "mp4" | "mov" | "avi" | "mkv" | "webm" => ContentType::Video,
        // Document formats
        "pdf" | "docx" | "pptx" | "key" => ContentType::Document,
        // Image formats
        "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" => ContentType::Image,
        // Text formats
        "txt" | "md" | "rst" => ContentType::Text,
        // Caption/subtitle formats
        "srt" | "vtt" | "scc" | "webvtt" => ContentType::Structured,
        // Audio formats
        "mp3" | "wav" | "flac" | "ogg" => ContentType::Audio,
        // Default to text for unknown types
        _ => ContentType::Text,
    }
}

/// Convert ingestion output to blocks
fn convert_ingestion_output_to_blocks(output: ProcessingOutput) -> Result<Vec<Block>> {
    use agent_data_processing::DataContent;
    
    let mut blocks = Vec::new();
    
    // Extract text content as blocks
    if let Some(text) = &output.processed_content.text_content {
        // Split text into chunks (simple implementation - in production would use proper chunking)
        let chunk_size = 1000; // characters per block
        let mut start = 0;
        
        while start < text.len() {
            let end = std::cmp::min(start + chunk_size, text.len());
            let chunk = text[start..end].to_string();
            
            let block = Block {
                id: agent_data_processing::ProcessingId(Uuid::new_v4()),
                content_type: output.processed_content.content_type.clone(),
                metadata: output.extracted_metadata.clone(),
                data: BlockData::Text(chunk),
            };
            
            blocks.push(block);
            start = end;
        }
    }
    
    // If no text content, create a single block from structured data
    if blocks.is_empty() {
        let content = serde_json::to_string(&output.processed_content.structured_data)
            .unwrap_or_else(|_| "No content".to_string());
        
        blocks.push(Block {
            id: agent_data_processing::ProcessingId(Uuid::new_v4()),
            content_type: output.processed_content.content_type,
            metadata: output.extracted_metadata,
            data: BlockData::Text(content),
        });
    }
    
    Ok(blocks)
}

/// File type enumeration
#[derive(Debug, Clone)]
enum FileType {
    Video,
    Slides,
    Diagrams,
    Captions,
    Unsupported,
}

/// Processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// Total documents processed
    pub total_documents_processed: u64,
    /// Total blocks processed
    pub total_blocks_processed: u64,
    /// Total blocks enriched
    pub total_blocks_enriched: u64,
    /// Total blocks indexed
    pub total_blocks_indexed: u64,
    /// Average processing time in milliseconds
    pub average_processing_time_ms: u64,
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    /// Number of active jobs
    pub active_jobs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_file_type_detection() {
        use agent_data_processing::ContentType;
        
        let video_path = PathBuf::from("test.mp4");
        assert_eq!(detect_content_type_from_path(&video_path), ContentType::Video);
        
        let slides_path = PathBuf::from("presentation.pptx");
        assert_eq!(detect_content_type_from_path(&slides_path), ContentType::Document);
        
        let unsupported_path = PathBuf::from("unknown.xyz");
        assert_eq!(detect_content_type_from_path(&unsupported_path), ContentType::Text); // Default to text
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = MultimodalOrchestrator::new();
        let stats = orchestrator.get_processing_stats().await.unwrap();
        
        assert_eq!(stats.total_documents_processed, 0);
        assert_eq!(stats.total_blocks_processed, 0);
    }

    #[test]
    fn test_processing_result_creation() {
        let result = ProcessingResult {
            document_id: Uuid::new_v4(),
            status: ProcessingStatus::Completed,
            blocks_processed: 10,
            blocks_enriched: 8,
            blocks_indexed: 8,
            processing_time_ms: 1000,
            error_message: None,
        };

        assert_eq!(result.blocks_processed, 10);
        assert!(matches!(result.status, ProcessingStatus::Completed));
    }
}
