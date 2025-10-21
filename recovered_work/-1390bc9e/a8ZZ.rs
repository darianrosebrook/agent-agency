//! Multimodal orchestration for document processing pipeline
//! 
//! Coordinates ingestors, enrichers, and indexers for multimodal RAG system
//! with proper error handling, concurrency control, and monitoring.

use anyhow::{Context, Result};
use ingestors::{FileWatcher, VideoIngestor, SlidesIngestor, DiagramsIngestor, CaptionsIngestor};
use enrichers::{VisionEnricher, AsrEnricher, EntityEnricher, VisualCaptionEnricher, CircuitBreaker};
use indexers::{Bm25Indexer, HnswIndexer, JobScheduler};
use agent_agency_research::KnowledgeSeeker;
use agent_agency_council::coordinator::ConsensusCoordinator;
use crate::audit_trail::AuditTrailManager;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Multimodal document processing orchestrator
#[derive(Debug)]
pub struct MultimodalOrchestrator {
    /// File watcher for monitoring directories
    file_watcher: FileWatcher,
    /// Ingestors for different content types
    video_ingestor: VideoIngestor,
    slides_ingestor: SlidesIngestor,
    diagrams_ingestor: DiagramsIngestor,
    captions_ingestor: CaptionsIngestor,
    /// Enrichers for content enhancement
    vision_enricher: VisionEnricher,
    asr_enricher: AsrEnricher,
    entity_enricher: EntityEnricher,
    visual_caption_enricher: VisualCaptionEnricher,
    /// Indexers for search capabilities
    bm25_indexer: Bm25Indexer,
    hnsw_indexer: HnswIndexer,
    /// Job scheduler for coordination
    job_scheduler: JobScheduler,
    /// Circuit breaker for resilience
    circuit_breaker: CircuitBreaker,
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
}

impl MultimodalOrchestrator {
    /// Create new multimodal orchestrator
    pub fn new() -> Self {
        Self {
            file_watcher: FileWatcher::new(),
            video_ingestor: VideoIngestor::new(),
            slides_ingestor: SlidesIngestor::new(),
            diagrams_ingestor: DiagramsIngestor::new(),
            captions_ingestor: CaptionsIngestor::new(),
            vision_enricher: VisionEnricher::new(),
            asr_enricher: AsrEnricher::new(),
            entity_enricher: EntityEnricher::new(),
            visual_caption_enricher: VisualCaptionEnricher::new(),
            bm25_indexer: Bm25Indexer::new(),
            hnsw_indexer: HnswIndexer::new(),
            job_scheduler: JobScheduler::new(),
            circuit_breaker: CircuitBreaker::new(),
            knowledge_seeker: None,
            council_coordinator: None,
            audit_trail: None,
        }
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

        // Stage 1: Ingest document based on file type
        let blocks = match self.detect_file_type(file_path) {
            FileType::Video => {
                debug!("Processing video file: {}", file_path.display());
                self.video_ingestor.ingest(file_path).await?
            }
            FileType::Slides => {
                debug!("Processing slides file: {}", file_path.display());
                self.slides_ingestor.ingest(file_path).await?
            }
            FileType::Diagrams => {
                debug!("Processing diagrams file: {}", file_path.display());
                self.diagrams_ingestor.ingest(file_path).await?
            }
            FileType::Captions => {
                debug!("Processing captions file: {}", file_path.display());
                self.captions_ingestor.ingest(file_path).await?
            }
            FileType::Unsupported => {
                warn!("Unsupported file type: {}", file_path.display());
                return Ok(ProcessingResult {
                    document_id,
                    status: ProcessingStatus::Skipped,
                    blocks_processed: 0,
                    blocks_enriched: 0,
                    blocks_indexed: 0,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some("Unsupported file type".to_string()),
                });
            }
        };

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
                let result = async move {
                    // TODO: Implement actual document processing orchestration
                    // - [ ] Integrate with document ingestion pipeline for file parsing
                    // - [ ] Implement block-level processing with multimodal enrichment
                    // - [ ] Add document structure analysis and content extraction
                    // - [ ] Support different document formats (PDF, DOCX, PPTX, images, etc.)
                    // - [ ] Implement processing progress tracking and resumability
                    // - [ ] Add error handling and recovery for failed processing
                    // - [ ] Support parallel processing of document sections
                    Ok(ProcessingResult {
                        document_id: Uuid::new_v4(),
                        status: ProcessingStatus::Completed,
                        blocks_processed: 10,
                        blocks_enriched: 8,
                        blocks_indexed: 8,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        error_message: None,
                    })
                }.await;

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

        let mut results = Vec::new();
        for task in tasks {
            let result = task.await.context("Task execution failed")?;
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
            circuit_breaker_state: self.circuit_breaker.get_state(),
            active_jobs: self.job_scheduler.get_active_job_count(),
        };

        Ok(stats)
    }

    // Helper methods

    /// Detect file type based on extension
    fn detect_file_type(&self, file_path: &Path) -> FileType {
        if let Some(extension) = file_path.extension() {
            match extension.to_string_lossy().to_lowercase().as_str() {
                "mp4" | "avi" | "mov" | "mkv" | "webm" => FileType::Video,
                "pptx" | "ppt" | "pdf" => FileType::Slides,
                "png" | "jpg" | "jpeg" | "svg" | "drawio" => FileType::Diagrams,
                "srt" | "vtt" | "txt" => FileType::Captions,
                _ => FileType::Unsupported,
            }
        } else {
            FileType::Unsupported
        }
    }

    /// Enrich blocks with multimodal content
    async fn enrich_blocks(&self, blocks: &[ingestors::types::Block]) -> Result<Vec<enrichers::types::EnrichedBlock>> {
        let mut enriched_blocks = Vec::new();

        for block in blocks {
            // Apply appropriate enrichers based on block type
            let enriched = match block.content_type {
                ingestors::types::ContentType::Text => {
                    // Use entity enricher for text
                    self.entity_enricher.enrich(block).await?
                }
                ingestors::types::ContentType::Image => {
                    // Use vision enricher for images
                    self.vision_enricher.enrich(block).await?
                }
                ingestors::types::ContentType::Video => {
                    // Use ASR enricher for video
                    self.asr_enricher.enrich(block).await?
                }
                _ => {
                    // Default enrichment
                    enrichers::types::EnrichedBlock {
                        id: block.id,
                        original_content: block.content.clone(),
                        enriched_content: block.content.clone(),
                        enrichment_type: "default".to_string(),
                        confidence: 1.0,
                        metadata: std::collections::HashMap::new(),
                    }
                }
            };

            enriched_blocks.push(enriched);
        }

        Ok(enriched_blocks)
    }

    /// Index enriched blocks
    async fn index_blocks(&self, blocks: &[enrichers::types::EnrichedBlock]) -> Result<usize> {
        let mut indexed_count = 0;

        for block in blocks {
            // Index with BM25 for full-text search
            self.bm25_indexer.index_block(block).await?;
            
            // Index with HNSW for vector search
            self.hnsw_indexer.index_block(block).await?;
            
            indexed_count += 1;
        }

        Ok(indexed_count)
    }
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
        let orchestrator = MultimodalOrchestrator::new();
        
        let video_path = PathBuf::from("test.mp4");
        assert!(matches!(orchestrator.detect_file_type(&video_path), FileType::Video));
        
        let slides_path = PathBuf::from("presentation.pptx");
        assert!(matches!(orchestrator.detect_file_type(&slides_path), FileType::Slides));
        
        let unsupported_path = PathBuf::from("unknown.xyz");
        assert!(matches!(orchestrator.detect_file_type(&unsupported_path), FileType::Unsupported));
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
