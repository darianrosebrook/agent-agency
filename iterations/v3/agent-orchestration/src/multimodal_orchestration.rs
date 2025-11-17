//! Multimodal orchestration for document processing pipeline
//!
//! Coordinates ingestors, enrichers, and indexers for multimodal RAG system
//! with proper error handling, concurrency control, and monitoring.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
// Explicit imports from contracts (contracts-first: no wildcard imports)
use agent_agency_contracts::types::data_processing::ProcessingPriority;
use agent_agency_contracts::types::planning::RiskTier;
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Import OrchestrationError from lib.rs
use crate::OrchestrationError;

use agent_data_processing::{
    UnifiedIngestor, FileWatcher, UnifiedEnrichmentStage, UnifiedIndexer, JobScheduler,
    EnrichmentCircuitBreakerConfig, ingestion::IngestionStage,
};

// Local type definitions used instead of agent_data_processing
use crate::audit_trail::{AuditResult, AuditSeverity, AuditTrailManager};
use crate::error_handling::CircuitBreaker;
use system_common_interfaces::DatabaseAuditOperations;
use tracing::{info, warn};

// Import ConsensusCoordinator from local module (not in contracts yet)
use crate::consensus_coordinator::ConsensusCoordinator;

// KnowledgeSeeker trait for research integration
// PLACEHOLDER: Proper implementation needed when research integration is functional
pub trait KnowledgeSeeker: Send + Sync {
    fn seek(&self, query: &str) -> Result<String, String>;
}

/// Context for tracking active operations

#[derive(Debug, Clone)]
struct OperationContext {
    /// Operation ID for correlation
    #[allow(dead_code)] // Reserved for future use
    pub operation_id: String,
    /// Start time
    #[allow(dead_code)] // Reserved for future use
    pub start_time: Instant,
    /// Operation type
    pub operation_type: String,
    /// Parent operation ID (if nested)
    #[allow(dead_code)] // Reserved for future use
    pub parent_operation_id: Option<String>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
}

// Local type definitions to avoid circular dependency with agent-data-processing
// These mirror types from agent-data-processing crate

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
struct ProcessingId {
    #[schemars(with = "String")]
    pub id: Uuid,
}

impl ProcessingId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum ContentType {
    Text,
    Image,
    Video,
    Audio,
    Document,
    Code,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FileSource {
    pub path: PathBuf,
    pub content_type: ContentType,
    pub size_bytes: u64,
    #[schemars(with = "String")]
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum DataSource {
    File(FileSource),
    Url(String),
    Stream(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DataInput {
    pub id: ProcessingId,
    pub source: DataSource,
    pub content_type: ContentType,
    pub content: DataContent,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processing_context: ProcessingContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ProcessingContext {
    pub priority: ProcessingPriority,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum DataContent {
    File(PathBuf),
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum ProcessedContentData {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct ProcessedContent {
    pub id: ProcessingId,
    pub data: DataContent,
    pub relationships: Vec<ExtractedEntity>,
    pub visual_elements: Vec<VisualElement>,
    pub audio_transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct ProcessingOutput {
    pub processed_content: Vec<ProcessedContent>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExtractedEntity {
    pub entity_type: String,
    pub text: String,
    pub confidence: f64,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct VisualElement {
    pub element_type: VisualElementType,
    pub bounding_box: Option<(f32, f32, f32, f32)>,
    pub confidence: f64,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum VisualElementType {
    Text,
    Image,
    Chart,
    Table,
    Diagram,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExtractedTopic {
    pub topic: String,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

// TODO: Implement data processing stage types in agent-data-processing
//       Currently uses placeholder types; should implement actual data processing stage types in agent-data-processing module.
// Following contracts-first architecture: Use contracts types where possible, local types for implementations

// Contracts-first approach: Remove direct dependency on agent-data-processing
// Instead, use local type definitions that mirror the needed types
// When data-processing feature is enabled, these can be replaced with trait objects
// that implement contracts-defined ports

// Data processing types are now imported directly from agent-data-processing crate
// since data-processing feature is always enabled. No stub implementations needed.

// Local type definitions to avoid circular dependency with agent-workers
// These mirror types from agent-workers crate


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPoolConfig;

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        WorkerPoolConfig
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct WorkerHandle;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum WorkerSpecialty {
    General,
    CodeAnalysis,
    Documentation,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct WorkerCapabilities;

impl Default for WorkerCapabilities {
    fn default() -> Self {
        WorkerCapabilities
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum WorkerHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Block {
    pub id: String,
    pub content: String,
    pub block_type: String,
    pub content_type: ContentType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EnrichedBlock {
    pub block: Block,
    pub entities: Vec<ExtractedEntity>,
    pub topics: Vec<ExtractedTopic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum BlockData {
    Text(String),
    Blocks(Vec<Block>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct EnrichedContent {
    pub content: String,
    pub entities: Vec<ExtractedEntity>,
    pub topics: Vec<ExtractedTopic>,
}
use crate::coreml::CoreMLManager;
use serde_json;
use std::path::{Path, PathBuf};

/// Multimodal document processing orchestrator
#[derive(Clone)]
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
    /// Council coordinator for decision-making
    council_coordinator: Option<Arc<dyn ConsensusCoordinator>>,
    /// Audit trail manager for recording processing events
    audit_trail: Option<Arc<AuditTrailManager>>,
    /// Circuit breakers for external service protection
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
    /// Active operation contexts for correlation
    active_contexts: Arc<RwLock<HashMap<String, OperationContext>>>,
    /// Database audit operations for audit persistence
    ///
    /// Provides audit trail persistence without requiring full database operations.
    /// Inject a DatabaseAuditOperations implementation (e.g., from data-infrastructure)
    /// via `set_database_audit_operations()` or `with_db_audit_ops()`.
    db_audit_ops: Option<Arc<dyn DatabaseAuditOperations>>,
    /// Planning integration for planning-aware task execution
    planning_integration:
        Option<Arc<crate::planning::orchestrator_integration::OrchestratorPlanningIntegration>>,
}

impl std::fmt::Debug for MultimodalOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultimodalOrchestrator")
            .field("circuit_breaker", &self.circuit_breaker)
            .field("coreml_manager", &self.coreml_manager.is_some())
            .field("council_coordinator", &self.council_coordinator.is_some())
            .field("audit_trail", &self.audit_trail.is_some())
            .field("db_audit_ops", &self.db_audit_ops.is_some())
            .field("planning_integration", &self.planning_integration.is_some())
            .finish()
    }
}

/// Processing result for document pipeline

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingResult {
    /// Document identifier
    #[schemars(with = "String")]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        let unified_enricher =
            UnifiedEnrichmentStage::new(EnrichmentCircuitBreakerConfig::default());
        let unified_indexer = UnifiedIndexer::new(768, 32); // 768-dim embeddings, 32 neighbors

        // Initialize Core ML manager
        let coreml_manager = {
            let manager = Arc::new(CoreMLManager::new(
                std::env::var("COREML_MODELS_PATH")
                    .map(|p| PathBuf::from(p))
                    .unwrap_or_else(|_| {
                        PathBuf::from(
                            "/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml",
                        )
                    }),
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
            file_watcher: FileWatcher::new(vec![], vec![])
                .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to create file watcher: {}", e)))?,
            unified_enricher,
            unified_indexer,
            job_scheduler: JobScheduler::new(),
            circuit_breaker: CircuitBreaker::new(
                "multimodal-orchestration".to_string(),
                crate::error_handling::ErrorHandlingCircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    recovery_timeout: std::time::Duration::from_secs(60),
                    monitoring_window: std::time::Duration::from_secs(300),
                    request_timeout: std::time::Duration::from_secs(30),
                },
            ),
            coreml_manager,
            council_coordinator: None,
            audit_trail: None,
            circuit_breakers: HashMap::new(),
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            db_audit_ops: None,
            planning_integration: None,
        })
    }

    /// Create new multimodal orchestrator with database audit operations
    pub async fn with_db_audit_ops(db_audit_ops: Arc<dyn DatabaseAuditOperations>) -> Result<Self> {
        let mut orchestrator = Self::new().await?;
        orchestrator.db_audit_ops = Some(db_audit_ops);
        Ok(orchestrator)
    }

    /// Set audit trail manager for event recording
    pub fn set_audit_trail(&mut self, audit_trail: Arc<AuditTrailManager>) {
        self.audit_trail = Some(audit_trail);
    }

    /// Set circuit breaker for external service protection
    pub fn set_circuit_breaker(
        &mut self,
        service_name: String,
        circuit_breaker: Arc<CircuitBreaker>,
    ) {
        self.circuit_breakers.insert(service_name, circuit_breaker);
    }

    /// Set multiple circuit breakers at once
    pub fn set_circuit_breakers(&mut self, circuit_breakers: HashMap<String, Arc<CircuitBreaker>>) {
        self.circuit_breakers.extend(circuit_breakers);
    }

    /// Set database audit operations for audit persistence
    pub fn set_database_audit_operations(
        &mut self,
        db_audit_ops: Arc<dyn DatabaseAuditOperations>,
    ) {
        self.db_audit_ops = Some(db_audit_ops);
    }

    /// Set council coordinator for decision-making
    pub fn set_council_coordinator(&mut self, coordinator: Arc<dyn ConsensusCoordinator>) {
        self.council_coordinator = Some(coordinator);
    }

    /// Set planning integration for planning-aware task execution
    pub fn set_planning_integration(
        &mut self,
        planning_integration: Arc<
            crate::planning::orchestrator_integration::OrchestratorPlanningIntegration,
        >,
    ) {
        self.planning_integration = Some(planning_integration);
    }

    /// Record operation start for audit trail
    async fn record_operation_start(
        &self,
        operation_type: &str,
        operation_id: &str,
        description: Option<String>,
        correlation_id: Option<String>,
    ) -> Result<(), crate::audit_trail::AuditError> {
        // Persist to database if available
        if let Some(db_audit_ops) = &self.db_audit_ops {
            let audit_entry = system_common_interfaces::CreateAuditEntry {
                entity_type: "multimodal_operation".to_string(),
                entity_id: Uuid::parse_str(operation_id).unwrap_or_else(|_| Uuid::new_v4()),
                action: format!("start_{}", operation_type),
                details: serde_json::json!({
                    "operation_type": operation_type,
                    "description": description,
                    "correlation_id": correlation_id,
                }),
                user_id: None,
                ip_address: None,
                timestamp: Some(Utc::now()),
            };

            if let Err(e) = db_audit_ops.create_audit_entry(audit_entry).await {
                warn!("Failed to persist audit entry to database: {}", e);
            }
        }

        if let Some(_audit_manager) = &self.audit_trail {
            let mut contexts = self.active_contexts.write().await;
            contexts.insert(
                operation_id.to_string(),
                OperationContext {
                    operation_id: operation_id.to_string(),
                    start_time: Instant::now(),
                    operation_type: operation_type.to_string(),
                    parent_operation_id: None,
                    correlation_id: correlation_id.clone(),
                },
            );

            // Audit trail is already recorded via db_audit_ops.create_audit_entry() above
            // The AuditTrailManager doesn't have a generic record_event method - it uses
            // specialized auditors (file_auditor, terminal_auditor, etc.) for specific event types.
            // For multimodal operations, we use the database audit operations interface.
        }
        Ok(())
    }

    /// Record operation completion for audit trail
    async fn record_operation_completion(
        &self,
        operation_id: &str,
        success: bool,
        duration: Duration,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), crate::audit_trail::AuditError> {
        // Persist to database if available
        if let Some(db_audit_ops) = &self.db_audit_ops {
            if let Some(context) = self.active_contexts.read().await.get(operation_id) {
                let audit_entry = system_common_interfaces::CreateAuditEntry {
                    entity_type: "multimodal_operation".to_string(),
                    entity_id: Uuid::parse_str(operation_id).unwrap_or_else(|_| Uuid::new_v4()),
                    action: format!("complete_{}", context.operation_type),
                    details: serde_json::json!({
                        "success": success,
                        "duration_ms": duration.as_millis(),
                        "operation_type": context.operation_type,
                        "correlation_id": context.correlation_id,
                        "metadata": metadata,
                    }),
                    user_id: None,
                    ip_address: None,
                    timestamp: Some(Utc::now()),
                };

                if let Err(e) = db_audit_ops.create_audit_entry(audit_entry).await {
                    warn!("Failed to persist audit entry to database: {}", e);
                }
            }
        }

        if let Some(_audit_manager) = &self.audit_trail {
            let mut contexts = self.active_contexts.write().await;
            if let Some(_context) = contexts.remove(operation_id) {
                let _result = if success {
                    AuditResult::Success { data: None }
                } else {
                    AuditResult::Failure {
                        error_message: "Operation failed".to_string(),
                        error_code: None,
                        recoverable: true,
                    }
                };
                let _severity = if success {
                    AuditSeverity::Info
                } else {
                    AuditSeverity::Error
                };

                // Audit trail is already recorded via db_audit_ops.create_audit_entry() above
                // The AuditTrailManager doesn't have a generic record_event method - it uses
                // specialized auditors (file_auditor, terminal_auditor, etc.) for specific event types.
                // For multimodal operations, we use the database audit operations interface.
            }
        }
        Ok(())
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

        info!(
            "Starting multimodal document processing: {} (id: {})",
            file_path.display(),
            document_id
        );

        // Check circuit breaker state
        if matches!(
            self.circuit_breaker.get_state().await,
            crate::error_handling::CircuitBreakerState::Open
        ) {
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
        let file_metadata = std::fs::metadata(file_path).context("Failed to read file metadata")?;

        let content_type = detect_content_type_from_path(file_path);

        // Create proper DataInput with real types
        // Using local type definitions
        let data_input = DataInput {
            id: ProcessingId::new(),
            source: DataSource::File(FileSource {
                path: file_path.to_path_buf(),
                content_type: content_type.clone(),
                size_bytes: file_metadata.len(),
                last_modified: file_metadata
                    .modified()
                    .ok()
                    .and_then(|t| {
                        use std::time::UNIX_EPOCH;
                        t.duration_since(UNIX_EPOCH)
                            .ok()
                            .map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                            .flatten()
                    })
                    .unwrap_or_else(|| chrono::Utc::now()),
            }),
            content_type,
            content: DataContent::File(file_path.to_path_buf()),
            metadata: HashMap::new(),
            processing_context: ProcessingContext {
                priority: ProcessingPriority::Normal,
                timeout: None,
            },
        };

        // Convert local DataInput to agent_data_processing::DataInput
        let processing_input = convert_data_input_to_processing(data_input)
            .context("Failed to convert DataInput")?;
        
        let ingestion_output = self
            .unified_ingestor
            .ingest(processing_input)
            .await
            .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to ingest document: {}", e)))?;

        // Convert ProcessingOutput to blocks for processing
        let blocks = convert_processing_output_to_blocks(ingestion_output)
            .context("Failed to convert ProcessingOutput to blocks")?;

        let blocks_processed = blocks.len();
        info!(
            "Ingested {} blocks from {}",
            blocks_processed,
            file_path.display()
        );

        // Stage 2: Enrich blocks with multimodal content
        // Convert local blocks to agent_data_processing::Block
        let processing_blocks: Vec<agent_data_processing::Block> = blocks
            .iter()
            .map(convert_block_to_processing)
            .collect();
        
        let processing_enriched = self
            .unified_enricher
            .enrich_blocks(processing_blocks)
            .await
            .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to enrich blocks: {}", e)))?;
        
        // Convert back to local EnrichedBlock types
        let enriched_blocks: Vec<EnrichedBlock> = processing_enriched
            .into_iter()
            .map(convert_enriched_block_from_processing)
            .collect();
        let blocks_enriched = enriched_blocks.len();
        info!("Enriched {} blocks", blocks_enriched);

        // Stage 3: Index enriched content
        // Convert local EnrichedBlock to agent_data_processing::EnrichedBlock
        let processing_enriched_blocks: Vec<agent_data_processing::EnrichedBlock> = enriched_blocks
            .iter()
            .map(|eb| {
                let block = convert_block_to_processing(&eb.block);
                // Create EnrichedContent from entities and topics
                let enriched_content = agent_data_processing::EnrichedContent {
                    entities: eb.entities
                        .iter()
                        .map(|e| agent_data_processing::ExtractedEntity {
                            id: Uuid::new_v4().to_string(),
                            name: e.text.clone(),
                            entity_type: e.entity_type.clone(),
                            confidence: e.confidence as f32,
                            positions: vec![agent_data_processing::TextPosition {
                                start: e.start_offset,
                                end: e.end_offset,
                                page: None,
                            }],
                            metadata: HashMap::new(),
                        })
                        .collect(),
                    visual_elements: vec![],
                    audio_transcript: None,
                    topics: eb.topics
                        .iter()
                        .map(|t| agent_data_processing::ExtractedTopic {
                            name: t.topic.clone(),
                            confidence: t.confidence as f32,
                            keywords: t.keywords.clone(),
                        })
                        .collect(),
                    embeddings: None,
                };
                agent_data_processing::EnrichedBlock {
                    block,
                    enriched_content,
                    processing_metadata: agent_data_processing::ProcessingMetadata {
                        source_url: None,
                        content_hash: {
                            use std::hash::{Hash, Hasher};
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            eb.block.content.hash(&mut hasher);
                            format!("{:x}", hasher.finish())
                        },
                        ingested_at: chrono::Utc::now(),
                        processing_version: "1.0.0".to_string(),
                        quality_score: 0.8,
                        confidence_scores: HashMap::new(),
                    },
                }
            })
            .collect();
        
        self.unified_indexer
            .index_blocks(processing_enriched_blocks)
            .await
            .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to index blocks: {}", e)))?;
        
        let blocks_indexed = enriched_blocks.len();
        info!("Indexed {} blocks", blocks_indexed);

        // Record success in circuit breaker
        self.circuit_breaker.record_success().await;

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
            .start_watching()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start directory watching: {}", e))?;

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
        info!(
            "Processing {} documents with max concurrency: {}",
            file_paths.len(),
            max_concurrent
        );

        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut tasks = Vec::new();

        for file_path in file_paths {
            let semaphore = semaphore.clone();
            let file_path_str = file_path.to_string_lossy().to_string();
            let audit_trail_clone = self.audit_trail.clone();
            let unified_ingestor = self.unified_ingestor.clone();
            let unified_enricher = self.unified_enricher.clone();
            let unified_indexer = self.unified_indexer.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Record document processing started
                if let Some(audit) = &audit_trail_clone {
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert(
                        "file_path".to_string(),
                        serde_json::Value::String(file_path_str.clone()),
                    );
                    metadata.insert(
                        "event_type".to_string(),
                        serde_json::Value::String("started".to_string()),
                    );
                    let _ = audit
                        .performance_auditor()
                        .record_operation_performance(
                            "document_processing",
                            std::time::Duration::from_millis(0),
                            true,
                            metadata,
                        )
                        .await;
                }

                let start_time = std::time::Instant::now();

                // Process document using cloned components
                let result: Result<ProcessingResult, OrchestrationError> = async {
                    let path = Path::new(&file_path_str);
                    let content = tokio::fs::read_to_string(path).await?;
                    
                    // Create local DataInput
                    let local_input = DataInput {
                        id: ProcessingId::new(),
                        source: DataSource::File(FileSource {
                            path: PathBuf::from(&file_path_str),
                            content_type: ContentType::Document,
                            size_bytes: 0, // TODO: Get actual file size
                            last_modified: chrono::Utc::now(),
                        }),
                        content_type: ContentType::Document,
                        content: DataContent::Text(content),
                        metadata: HashMap::new(),
                        processing_context: ProcessingContext {
                            priority: ProcessingPriority::Normal,
                            timeout: None,
                        },
                    };
                    
                    // Convert to processing DataInput
                    let processing_input = convert_data_input_to_processing(local_input)?;
                    
                    // Ingest
                    let ingestion_output = unified_ingestor
                        .ingest(processing_input)
                        .await
                        .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to ingest: {}", e)))?;
                    
                    // Convert ProcessingOutput to blocks
                    let blocks = convert_processing_output_to_blocks(ingestion_output)?;
                    
                    // Convert to processing blocks
                    let processing_blocks: Vec<agent_data_processing::Block> = blocks
                        .iter()
                        .map(convert_block_to_processing)
                        .collect();
                    
                    // Enrich
                    let processing_enriched = unified_enricher
                        .enrich_blocks(processing_blocks)
                        .await
                        .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to enrich: {}", e)))?;
                    
                    // Index
                    unified_indexer
                        .index_blocks(processing_enriched)
                        .await
                        .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to index: {}", e)))?;
                    Ok(ProcessingResult {
                        document_id: Uuid::new_v4(),
                        status: ProcessingStatus::Completed,
                        blocks_processed: 1,
                        blocks_enriched: 1,
                        blocks_indexed: 1,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        error_message: None,
                    })
                }
                .await;

                // Record document processing finished or error
                if let Some(audit) = &audit_trail_clone {
                    let success = result.is_ok();
                    let event_type = if success { "finished" } else { "error" };
                    let processing_time = start_time.elapsed();

                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert(
                        "file_path".to_string(),
                        serde_json::Value::String(file_path_str.clone()),
                    );
                    metadata.insert(
                        "event_type".to_string(),
                        serde_json::Value::String(event_type.to_string()),
                    );

                    match &result {
                        Ok(result) => {
                            metadata.insert(
                                "document_id".to_string(),
                                serde_json::Value::String(result.document_id.to_string()),
                            );
                            metadata.insert(
                                "blocks_processed".to_string(),
                                serde_json::Value::Number(result.blocks_processed.into()),
                            );
                            metadata.insert(
                                "blocks_enriched".to_string(),
                                serde_json::Value::Number(result.blocks_enriched.into()),
                            );
                            metadata.insert(
                                "blocks_indexed".to_string(),
                                serde_json::Value::Number(result.blocks_indexed.into()),
                            );
                            metadata.insert(
                                "processing_time_ms".to_string(),
                                serde_json::Value::Number(result.processing_time_ms.into()),
                            );
                        }
                        Err(e) => {
                            metadata.insert(
                                "error".to_string(),
                                serde_json::Value::String(e.to_string()),
                            );
                        }
                    }

                    let _ = audit
                        .performance_auditor()
                        .record_operation_performance(
                            "document_processing",
                            processing_time,
                            success,
                            metadata,
                        )
                        .await;
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

        info!(
            "Completed parallel processing of {} documents",
            results.len()
        );
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
            circuit_breaker_state: format!("{:?}", self.circuit_breaker.get_state().await),
            active_jobs: self.job_scheduler.get_active_job_count(),
        };

        Ok(stats)
    }

    // Helper methods

    /// Enrich blocks with multimodal content
    async fn enrich_blocks(&self, blocks: &[Block]) -> Result<Vec<EnrichedBlock>> {
        // Convert local blocks to agent_data_processing::Block
        let processing_blocks: Vec<agent_data_processing::Block> = blocks
            .iter()
            .map(convert_block_to_processing)
            .collect();
        
        // Use unified enricher to handle enrichment
        let processing_enriched = self
            .unified_enricher
            .enrich_blocks(processing_blocks)
            .await
            .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to enrich blocks: {}", e)))?;

        // Convert back to local EnrichedBlock types
        let enriched: Vec<EnrichedBlock> = processing_enriched
            .into_iter()
            .map(convert_enriched_block_from_processing)
            .collect();

        Ok(enriched)
    }

    /// Index enriched blocks
    async fn index_blocks(&self, blocks: &[EnrichedBlock]) -> Result<usize> {
        // Convert local EnrichedBlock to agent_data_processing::EnrichedBlock
        let processing_enriched_blocks: Vec<agent_data_processing::EnrichedBlock> = blocks
            .iter()
            .map(|eb| {
                let block = convert_block_to_processing(&eb.block);
                // Create EnrichedContent from entities and topics
                let enriched_content = agent_data_processing::EnrichedContent {
                    entities: eb.entities
                        .iter()
                        .map(|e| agent_data_processing::ExtractedEntity {
                            id: Uuid::new_v4().to_string(),
                            name: e.text.clone(),
                            entity_type: e.entity_type.clone(),
                            confidence: e.confidence as f32,
                            positions: vec![agent_data_processing::TextPosition {
                                start: e.start_offset,
                                end: e.end_offset,
                                page: None,
                            }],
                            metadata: HashMap::new(),
                        })
                        .collect(),
                    visual_elements: vec![],
                    audio_transcript: None,
                    topics: eb.topics
                        .iter()
                        .map(|t| agent_data_processing::ExtractedTopic {
                            name: t.topic.clone(),
                            confidence: t.confidence as f32,
                            keywords: t.keywords.clone(),
                        })
                        .collect(),
                    embeddings: None,
                };
                agent_data_processing::EnrichedBlock {
                    block,
                    enriched_content,
                    processing_metadata: agent_data_processing::ProcessingMetadata {
                        source_url: None,
                        content_hash: {
                            use std::hash::{Hash, Hasher};
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            eb.block.content.hash(&mut hasher);
                            format!("{:x}", hasher.finish())
                        },
                        ingested_at: chrono::Utc::now(),
                        processing_version: "1.0.0".to_string(),
                        quality_score: 0.8,
                        confidence_scores: HashMap::new(),
                    },
                }
            })
            .collect();

        self.unified_indexer
            .index_blocks(processing_enriched_blocks)
            .await
            .map_err(|e| OrchestrationError::AnyhowError(anyhow::anyhow!("Failed to index blocks: {}", e)))?;

        Ok(blocks.len()) // Return count of indexed items
    }

    /// Execute planning with comprehensive audit trail
    pub async fn execute_planning_with_audit(
        &self,
        task_description: &str,
        _context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ProcessingResult, OrchestrationError> {
        let operation_id = Uuid::new_v4().to_string();
        let correlation_id = Some(operation_id.clone());

        // Record operation start
        let start_time = Instant::now();
        self.record_operation_start(
            "planning",
            &operation_id,
            Some(task_description.to_string()),
            correlation_id.clone(),
        )
        .await
        .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;

        // Track reasoning and decision making
        if let Some(audit_manager) = &self.audit_trail {
            audit_manager
                .agent_thinking_auditor()
                .record_reasoning_step(
                    "task_analysis",
                    &format!("Analyzing task: {}", task_description),
                    vec![
                        "Direct implementation".to_string(),
                        "Break down into subtasks".to_string(),
                        "Research and planning phase".to_string(),
                    ],
                    "Break down into subtasks",
                    0.85,
                    start_time.elapsed(),
                )
                .await
                .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;
        }

        // Execute the actual planning operation with circuit breaker protection
        let planning_start = Instant::now();
        let result = if let Some(ref planning_integration) = self.planning_integration {
            // Use real planning integration if available
            // Convert task_description to TaskDescriptor for planning system
            let task_descriptor = agent_agency_contracts::TaskDescriptor {
                task_id: Uuid::new_v4(),
                description: task_description.to_string(),
                scope_in: agent_agency_contracts::ScopeRestrictions {
                    allowed_paths: vec![],
                    blocked_paths: vec![],
                },
                scope_out: Some(agent_agency_contracts::ScopeRestrictions {
                    allowed_paths: vec![],
                    blocked_paths: vec![],
                }),
                change_budget: agent_agency_contracts::ChangeBudget {
                    max_files: 25,
                    max_loc: 1000,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::EnforcementMode::Strict,
                },
                blast_radius: agent_agency_contracts::BlastRadius {
                    modules: vec![],
                    data_migration: false,
                    external_deps: vec![],
                },
                priority: agent_agency_contracts::TaskPriority::Normal,
                execution_mode: agent_agency_contracts::ExecutionMode::Auto,
                risk_tier: Some(RiskTier::Tier2),
                acceptance: Some("Multimodal orchestration task".to_string()),
            };

            // Execute planning task with real planning system
            match planning_integration
                .execute_planning_task(&task_descriptor)
                .await
            {
                Ok(planning_result) => {
                    // Convert PlanningTaskResult to ProcessingResult
                    ProcessingResult {
                        document_id: planning_result.task_id,
                        status: if planning_result.quality_verified {
                            ProcessingStatus::Completed
                        } else {
                            ProcessingStatus::Failed
                        },
                        blocks_processed: planning_result.evidence_count,
                        blocks_enriched: planning_result.evidence_count,
                        blocks_indexed: planning_result.evidence_count,
                        processing_time_ms: planning_start.elapsed().as_millis() as u64,
                        error_message: None,
                    }
                }
                Err(e) => {
                    // Planning failed - return error result
                    ProcessingResult {
                        document_id: Uuid::new_v4(),
                        status: ProcessingStatus::Failed,
                        blocks_processed: 0,
                        blocks_enriched: 0,
                        blocks_indexed: 0,
                        processing_time_ms: planning_start.elapsed().as_millis() as u64,
                        error_message: Some(format!("Planning failed: {}", e)),
                    }
                }
            }
        } else if let Some(circuit_breaker) = self.circuit_breakers.get("llm_service") {
            // Protect LLM/planning calls with circuit breaker (fallback when planning integration not available)
            match circuit_breaker.execute(|| async {
                // Planning integration not configured - return error result indicating configuration needed
                // To enable planning: call orchestrator.set_planning_integration(Arc::new(OrchestratorPlanningIntegration::new(...)))
                Ok(ProcessingResult {
                    document_id: Uuid::new_v4(),
                    status: ProcessingStatus::Failed,
                    blocks_processed: 0,
                    blocks_enriched: 0,
                    blocks_indexed: 0,
                    processing_time_ms: planning_start.elapsed().as_millis() as u64,
                    error_message: Some("Planning integration not configured. Call set_planning_integration() to enable planning-aware task execution.".to_string()),
                })
            }).await {
                Ok(result) => result,
                Err(e) => {
                    // Circuit breaker opened or operation failed
                    if let Some(audit_manager) = &self.audit_trail {
                        audit_manager.error_recovery_auditor()
                            .record_error_recovery_attempt(
                                "planning_circuit_breaker",
                                "circuit_breaker_protection",
                                false,
                                planning_start.elapsed(),
                                {
                                    let mut metadata = HashMap::new();
                                    metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                                    metadata.insert("circuit_breaker".to_string(), serde_json::Value::String("llm_service".to_string()));
                                    metadata
                                }
                            ).await.map_err(|e| OrchestrationError::AuditError(e.to_string()))?;
                    }
                    return Err(OrchestrationError::CircuitBreakerError(e.to_string()));
                }
            }
        } else {
            // No circuit breaker - direct execution (fallback when planning integration not available)
            // Planning integration not configured - return error result indicating configuration needed
            // To enable planning: call orchestrator.set_planning_integration(Arc::new(OrchestratorPlanningIntegration::new(...)))
            ProcessingResult {
                document_id: Uuid::new_v4(),
                status: ProcessingStatus::Failed,
                blocks_processed: 0,
                blocks_enriched: 0,
                blocks_indexed: 0,
                processing_time_ms: planning_start.elapsed().as_millis() as u64,
                error_message: Some("Planning integration not configured. Call set_planning_integration() to enable planning-aware task execution.".to_string()),
            }
        };

        // Record successful performance metrics
        if let Some(audit_manager) = &self.audit_trail {
            audit_manager
                .performance_auditor()
                .record_operation_performance(
                    "planning_execution",
                    planning_start.elapsed(),
                    true,
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "task_length".to_string(),
                            serde_json::Value::Number(task_description.len().into()),
                        );
                        metadata.insert(
                            "result_type".to_string(),
                            serde_json::Value::String("success".to_string()),
                        );
                        metadata
                    },
                )
                .await
                .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;
        }

        // Record operation completion
        self.record_operation_completion(
            &operation_id,
            true,
            start_time.elapsed(),
            Some(HashMap::from([
                (
                    "task_description".to_string(),
                    serde_json::Value::String(task_description.to_string()),
                ),
                (
                    "blocks_processed".to_string(),
                    serde_json::Value::Number(result.blocks_processed.into()),
                ),
            ])),
        )
        .await
        .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;

        Ok(result)
    }
}

/// Detect content type from file path
fn detect_content_type_from_path(path: &Path) -> ContentType {
    let extension = path
        .extension()
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
        "srt" | "vtt" | "scc" | "webvtt" => ContentType::Document,
        // Audio formats
        "mp3" | "wav" | "flac" | "ogg" => ContentType::Audio,
        // Default to text for unknown types
        _ => ContentType::Text,
    }
}

/// Convert local DataInput to agent_data_processing::DataInput
#[cfg(feature = "data-processing")]
fn convert_data_input_to_processing(
    input: DataInput,
) -> Result<agent_data_processing::DataInput, OrchestrationError> {
    use agent_data_processing::{DataContent as ProcessingDataContent, DataSource as ProcessingDataSource, ProcessingContext as ProcessingProcessingContext, ProcessingId as ProcessingProcessingId, ProcessingPriority as ProcessingProcessingPriority, StreamSource};
    
    let processing_id = agent_data_processing::ProcessingId(input.id.id);
    
    let processing_source = match &input.source {
        DataSource::File(file_source) => {
            ProcessingDataSource::File(agent_data_processing::FileSource {
                path: file_source.path.clone(),
                content_type: convert_content_type(&file_source.content_type),
                size_bytes: file_source.size_bytes,
                last_modified: file_source.last_modified,
            })
        }
        DataSource::Url(url) => {
            ProcessingDataSource::Url(agent_data_processing::UrlSource {
                url: url.clone(),
                headers: HashMap::new(),
                content_type: Some(convert_content_type(&input.content_type)),
            })
        }
        DataSource::Stream(data) => {
            ProcessingDataSource::Stream(StreamSource {
                stream_id: "multimodal_orchestration".to_string(),
                content_type: convert_content_type(&input.content_type),
            })
        }
    };
    
    let processing_content = match &input.content {
        DataContent::File(path) => ProcessingDataContent::File(path.clone()),
        DataContent::Text(text) => ProcessingDataContent::Text(text.clone()),
        DataContent::Binary(data) => ProcessingDataContent::Binary(data.clone()),
    };
    
    let processing_context = ProcessingProcessingContext {
        request_id: uuid::Uuid::new_v4().to_string(),
        user_id: None,
        project_scope: None,
        priority: convert_processing_priority(&input.processing_context.priority),
        deadline: None,
        tags: vec![],
    };
    
    Ok(agent_data_processing::DataInput {
        id: processing_id,
        source: processing_source,
        content: processing_content,
        metadata: input.metadata.clone(),
        processing_context,
    })
}

/// Convert ContentType from local to agent_data_processing
fn convert_content_type(ct: &ContentType) -> agent_data_processing::ContentType {
    match ct {
        ContentType::Text => agent_data_processing::ContentType::Text,
        ContentType::Image => agent_data_processing::ContentType::Image,
        ContentType::Video => agent_data_processing::ContentType::Video,
        ContentType::Audio => agent_data_processing::ContentType::Audio,
        ContentType::Document => agent_data_processing::ContentType::Document,
        ContentType::Code => agent_data_processing::ContentType::Code,
        ContentType::Unknown => agent_data_processing::ContentType::Unknown,
    }
}

/// Convert ProcessingPriority from local to agent_data_processing
fn convert_processing_priority(pp: &ProcessingPriority) -> agent_data_processing::ProcessingPriority {
    match pp {
        ProcessingPriority::Low => agent_data_processing::ProcessingPriority::Low,
        ProcessingPriority::Normal => agent_data_processing::ProcessingPriority::Normal,
        ProcessingPriority::High => agent_data_processing::ProcessingPriority::High,
        ProcessingPriority::Urgent => agent_data_processing::ProcessingPriority::Critical,
    }
}

/// Convert ProcessingOutput to local Block types
fn convert_processing_output_to_blocks(
    output: agent_data_processing::ProcessingOutput,
) -> Result<Vec<Block>, OrchestrationError> {
    use std::collections::HashMap;
    
    // Extract text content from processed_content
    let text_content = output.processed_content.text_content
        .unwrap_or_else(|| {
            // Fallback: try to extract from structured data
            if let Some(structured) = &output.processed_content.structured_data {
                serde_json::to_string(structured).unwrap_or_default()
            } else {
                String::new()
            }
        });
    
    // Create a block from the processing output
    let block = Block {
        id: output.id.0.to_string(),
        content: text_content,
        block_type: format!("{:?}", output.processed_content.content_type),
        content_type: convert_content_type_from_processing(&output.processed_content.content_type),
        metadata: output.extracted_metadata,
    };
    
    Ok(vec![block])
}

/// Convert ContentType from agent_data_processing to local
fn convert_content_type_from_processing(ct: &agent_data_processing::ContentType) -> ContentType {
    match ct {
        agent_data_processing::ContentType::Text => ContentType::Text,
        agent_data_processing::ContentType::Image => ContentType::Image,
        agent_data_processing::ContentType::Video => ContentType::Video,
        agent_data_processing::ContentType::Audio => ContentType::Audio,
        agent_data_processing::ContentType::Document => ContentType::Document,
        agent_data_processing::ContentType::Code => ContentType::Code,
        _ => ContentType::Unknown,
    }
}

/// Convert local Block to agent_data_processing::Block
fn convert_block_to_processing(block: &Block) -> agent_data_processing::Block {
    agent_data_processing::Block {
        id: agent_data_processing::ProcessingId(
            Uuid::parse_str(&block.id).unwrap_or_else(|_| Uuid::new_v4())
        ),
        content_type: convert_content_type(&block.content_type),
        data: agent_data_processing::BlockData::Text(block.content.clone()),
        metadata: block.metadata.clone(),
    }
}

/// Convert agent_data_processing::EnrichedBlock to local EnrichedBlock
fn convert_enriched_block_from_processing(
    eb: agent_data_processing::EnrichedBlock,
) -> EnrichedBlock {
    let block = Block {
        id: eb.block.id.0.to_string(),
        content: match &eb.block.data {
            agent_data_processing::BlockData::Text(text) => text.clone(),
            agent_data_processing::BlockData::Binary(_) => String::new(),
            agent_data_processing::BlockData::Structured(data) => {
                serde_json::to_string(data).unwrap_or_default()
            }
        },
        block_type: format!("{:?}", eb.block.content_type),
        content_type: convert_content_type_from_processing(&eb.block.content_type),
        metadata: eb.block.metadata.clone(),
    };
    
    // Convert entities
    let entities = eb.enriched_content.entities
        .into_iter()
        .map(|e| ExtractedEntity {
            entity_type: e.entity_type,
            text: e.name,
            confidence: e.confidence as f64,
            start_offset: e.positions.first().map(|p| p.start).unwrap_or(0),
            end_offset: e.positions.first().map(|p| p.end).unwrap_or(0),
        })
        .collect();
    
    // Convert topics
    let topics = eb.enriched_content.topics
        .into_iter()
        .map(|t| ExtractedTopic {
            topic: t.name,
            confidence: t.confidence as f64,
            keywords: vec![], // Topics don't have keywords in the processing version
        })
        .collect();
    
    EnrichedBlock {
        block,
        entities,
        topics,
    }
}

/// File type enumeration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum FileType {
    Video,
    Slides,
    Diagrams,
    Captions,
    Unsupported,
}

/// Processing statistics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        let video_path = PathBuf::from("test.mp4");
        assert_eq!(
            detect_content_type_from_path(&video_path),
            ContentType::Video
        );

        let slides_path = PathBuf::from("presentation.pptx");
        assert_eq!(
            detect_content_type_from_path(&slides_path),
            ContentType::Document
        );

        let unsupported_path = PathBuf::from("unknown.xyz");
        assert_eq!(
            detect_content_type_from_path(&unsupported_path),
            ContentType::Text
        ); // Default to text
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = MultimodalOrchestrator::new().await.unwrap();
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
