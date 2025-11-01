// //! Data Processing Interfaces
// //!
// //! This module provides interfaces for multimodal data processing operations,
// //! allowing agent-orchestration to depend on abstractions rather than concrete implementations.
// //!
// //! The interfaces break circular dependencies by enabling dependency injection of
// //! data processing capabilities into the orchestration layer.

// use async_trait::async_trait;
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use uuid::Uuid;
// use chrono::{DateTime, Utc};
// use std::time::Duration;

// /// Processing ID for data processing operations
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
// pub struct ProcessingId(pub Uuid);

// impl ProcessingId {
//     pub fn new() -> Self {
//         Self(Uuid::new_v4())
//     }
// }

// /// Content type enumeration
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub enum ContentType {
//     Text,
//     Image,
//     Video,
//     Document,
//     Audio,
// }

// /// Processed content data variants
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum ProcessedContentData {
//     Text(String),
//     Binary(Vec<u8>),
//     Structured(serde_json::Value),
// }

// /// Block data for content processing
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum BlockData {
//     Text(String),
//     Binary(Vec<u8>),
//     Structured(serde_json::Value),
// }

// /// Processing block
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Block {
//     pub id: Uuid,
//     pub content_type: ContentType,
//     pub data: BlockData,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Processing output
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProcessingOutput {
//     pub id: ProcessingId,
//     pub processed_content: ProcessedContent,
//     pub extracted_metadata: HashMap<String, serde_json::Value>,
// }

// /// Processed content
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProcessedContent {
//     pub data: ProcessedContentData,
//     pub content_type: ContentType,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Data input for processing
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct DataInput {
//     pub source: DataSource,
//     pub content: Vec<u8>,
//     pub content_type: ContentType,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Data source
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum DataSource {
//     File(String),
//     Url(String),
//     Stream(String),
// }

// /// Processing context
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProcessingContext {
//     pub priority: ProcessingPriority,
//     pub timeout: Option<Duration>,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Processing priority
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub enum ProcessingPriority {
//     Low,
//     Normal,
//     High,
//     Critical,
// }

// /// Data processing interface for multimodal content processing
// #[async_trait]
// pub trait DataProcessor: Send + Sync {
//     /// Process multimodal data through the complete pipeline
//     async fn process_data(&self, input: DataInput) -> Result<ProcessingOutput, DataProcessingError>;

//     /// Convert processing output to content blocks for orchestration
//     async fn output_to_blocks(&self, output: ProcessingOutput) -> Result<Vec<Block>, DataProcessingError>;

//     /// Get processing statistics
//     async fn get_processing_stats(&self) -> Result<ProcessingStats, DataProcessingError>;
// }

// /// Processing statistics
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProcessingStats {
//     pub total_processed: u64,
//     pub average_processing_time_ms: f64,
//     pub success_rate: f64,
//     pub last_updated: DateTime<Utc>,
// }

// /// Data processing error types
// #[derive(thiserror::Error, Debug)]
// pub enum DataProcessingError {
//     #[error("Processing failed: {0}")]
//     ProcessingFailed(String),

//     #[error("Unsupported content type: {0}")]
//     UnsupportedContentType(String),

//     #[error("Configuration error: {0}")]
//     ConfigurationError(String),

//     #[error("Timeout error: {0}")]
//     Timeout(String),

//     #[error("Resource exhausted: {0}")]
//     ResourceExhausted(String),

//     #[error("Internal error: {0}")]
//     Internal(String),
// }

// impl From<anyhow::Error> for DataProcessingError {
//     fn from(err: anyhow::Error) -> Self {
//         DataProcessingError::Internal(err.to_string())
//     }
// }

// /// Ingestion stage interface
// #[async_trait]
// pub trait IngestionStage: Send + Sync {
//     async fn ingest(&self, input: DataInput) -> Result<IngestionResult, DataProcessingError>;
// }

// /// Enrichment stage interface
// #[async_trait]
// pub trait EnrichmentStage: Send + Sync {
//     async fn enrich(&self, input: ProcessedContent) -> Result<EnrichmentResult, DataProcessingError>;
// }

// /// Indexing stage interface
// #[async_trait]
// pub trait IndexingStage: Send + Sync {
//     async fn index(&self, input: EnrichedContent) -> Result<IndexingResult, DataProcessingError>;
//     async fn query(&self, query: IndexQuery) -> Result<IndexResult, DataProcessingError>;
// }

// /// Ingestion result
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IngestionResult {
//     pub processed_content: ProcessedContent,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Enrichment result
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct EnrichmentResult {
//     pub enriched_content: EnrichedContent,
//     pub extracted_entities: Vec<ExtractedEntity>,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Indexing result
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IndexingResult {
//     pub index_id: String,
//     pub indexed_items: u32,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Index query
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IndexQuery {
//     pub query_type: QueryType,
//     pub query: String,
//     pub limit: Option<u32>,
//     pub filters: Option<HashMap<String, serde_json::Value>>,
// }

// /// Index result
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IndexResult {
//     pub results: Vec<SearchResult>,
//     pub total_found: u64,
//     pub query_time_ms: u64,
// }

// /// Query types
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum QueryType {
//     Text,
//     Vector,
//     Hybrid,
// }

// /// Search result
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SearchResult {
//     pub id: String,
//     pub content: serde_json::Value,
//     pub score: f64,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Enriched content
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct EnrichedContent {
//     pub original: ProcessedContent,
//     pub enriched_data: HashMap<String, serde_json::Value>,
//     pub confidence_scores: HashMap<String, f64>,
// }

// /// Extracted entity
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExtractedEntity {
//     pub entity_type: String,
//     pub text: String,
//     pub confidence: f64,
//     pub position: Option<TextPosition>,
//     pub metadata: HashMap<String, serde_json::Value>,
// }

// /// Text position information
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TextPosition {
//     pub start: usize,
//     pub end: usize,
//     pub line: Option<usize>,
//     pub column: Option<usize>,
// }


